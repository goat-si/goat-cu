// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Cua AI, Inc.

use crate::goat::action_latency as latency;

use async_trait::async_trait;
use cua_driver_core::{
    protocol::ToolResult,
    tool::{Tool, ToolDef},
};
use serde_json::Value;
use std::sync::Arc;

use crate::ax::bindings::{kAXErrorSuccess, perform_action, AXUIElementRef};

use super::ToolState;

pub(super) async fn dispatch_open<C, P, A, F, T>(
    clock: &(impl cua_driver_core::actionability::Clock + Sync),
    budget: std::time::Duration,
    use_ax_open: bool,
    context: C,
    probe: P,
    perform: A,
) -> Result<T, ToolResult>
where
    C: Sync,
    P: for<'a> FnMut(
            &'a C,
            std::time::Duration,
        ) -> std::pin::Pin<
            Box<
                dyn std::future::Future<
                        Output = cua_driver_core::actionability::ActionabilityReport,
                    > + Send
                    + 'a,
            >,
        > + Send,
    A: FnOnce(C) -> F,
    F: std::future::Future<Output = T>,
{
    if use_ax_open {
        super::actionability::dispatch_semantic(clock, budget, context, probe, perform).await
    } else {
        Ok(perform(context).await)
    }
}

pub struct DoubleClickTool {
    state: Arc<ToolState>,
}

pub(super) fn background_action_for_element(
    has_ax_open: bool,
) -> cua_driver_core::background_input::BackgroundAction {
    if has_ax_open {
        cua_driver_core::background_input::BackgroundAction::AxSemantic
    } else {
        cua_driver_core::background_input::BackgroundAction::WindowPointer
    }
}

impl DoubleClickTool {
    pub fn new(state: Arc<ToolState>) -> Self {
        Self { state }
    }
}

static DEF: std::sync::OnceLock<ToolDef> = std::sync::OnceLock::new();

fn def() -> &'static ToolDef {
    DEF.get_or_init(|| ToolDef {
        name: "double_click".into(),
        description:
            "Double-click at (x, y) or on an AX element identified by element_token.\n\n\
             AX path (element_token provided): performs `AXOpen` when the element advertises it \
             (Finder items, openable list rows/cells); otherwise resolves the element's on-screen \
             center and falls back to a pixel double-click there.\n\n\
             Pixel path (x, y provided): two down/up pairs ~80 ms apart at the given coordinates."
            .into(),
        input_schema: serde_json::json!({
            "type": "object",
            "required": [],
            "properties": {
                "session": { "type": "string", "description": "For multi-call work, prefer a short public session label and repeat it on every call that accepts it. Omit it to use the authenticated transport's implicit lifecycle session." },
                "pid":           { "type": "integer", "description": "Target process ID. Optional with element_token (the token carries the pid)." },
                "x":             { "type": "number",  "description": "Screen X coordinate (pixel path)." },
                "y":             { "type": "number",  "description": "Screen Y coordinate (pixel path)." },
                "window_id":     { "type": "integer", "description": "CGWindowID. Omit when element_token is supplied (the token carries it)." },
                "element_token": cua_driver_core::tool_schema::element_token_schema(),
                "coordinate_space": super::window_points::coordinate_space_schema(),
                "window_basis": super::window_points::window_basis_schema(),
                "delivery_mode": cua_driver_core::tool_schema::delivery_mode_schema()
            },
            "additionalProperties": false
        }),
        read_only:   false,
        destructive: true,
        idempotent:  false,
        open_world:  true,
    })
}

#[async_trait]
impl Tool for DoubleClickTool {
    fn def(&self) -> &ToolDef {
        def()
    }

    async fn invoke(&self, args: Value) -> ToolResult {
        if args.get("simulate").is_some() {
            return super::invalid_arguments(
                "double_click does not support simulate; nothing was sent.",
            );
        }
        // Checked before any route, element or pixel: window points need a
        // window target and x, y (the pixel path re-checks in click).
        if let Err(refusal) = super::window_points::point_space(&args) {
            return refusal;
        }
        use cua_driver_core::tool_args::ArgsExt;
        let pid = match super::target_pid(&self.state, &args) {
            Ok(v) => v,
            Err(e) => return e,
        };
        // delivery_mode: foreground briefly fronts the window before the pixel
        // double-click (the explicit last resort for surfaces that drop
        // background CGEvents), via the same skylight assist click uses.
        let delivery_mode = super::DeliveryMode::parse(args.opt_str("delivery_mode").as_deref());
        let cursor_key = super::cursor_tools::resolve_cursor_key(&args);
        let window_id_arg = args.opt_u64("window_id");
        let evidence = crate::process_identity::evidence(&args, pid).await;
        let resolved = match latency::resolve(|| {
            self.state
                .snapshots
                .resolve_for_instance(pid, &args, evidence)
        }) {
            Ok(r) => r,
            Err(e) => return e,
        };
        let (element_index, window_id, element_guard) = resolved.into_parts(window_id_arg);
        let window_id = match super::native_window_id(window_id) {
            Ok(window_id) => window_id,
            Err(error) => return error,
        };

        // ── AX element path ──────────────────────────────────────────────────
        if let (Some(idx), Some(wid), Some(element_guard)) =
            (element_index, window_id, element_guard)
        {
            let element_ptr = element_guard.as_ptr();

            // Choose one background actuator before dispatch. An element that
            // advertises AXOpen uses the exact semantic route; all other
            // elements require the stricter routed-pointer proof. Do not let a
            // failed AXOpen silently cross into an ungated pointer fallback.
            let probe_guard = element_guard.clone();
            let advertised = latency::spawn_action_blocking(move || {
                super::actionability::action_advertised(
                    probe_guard.as_ptr() as AXUIElementRef,
                    "AXOpen",
                )
            })
            .await
            .unwrap_or_else(|error| {
                cua_driver_core::actionability::Evidence::Unknown(format!(
                    "action read worker failed: {error}"
                ))
            });
            let has_ax_open = super::actionability::may_use_semantic_route(&advertised);
            let mutation_lease = if !delivery_mode.is_foreground() {
                let action = background_action_for_element(has_ax_open);
                match super::gate_background_window_action(pid, wid, Some(element_ptr), action)
                    .await
                {
                    Ok(lease) => Some(lease),
                    Err(refusal_result) => return refusal_result,
                }
            } else {
                None
            };
            if let Some(refusal) = super::refuse_if_disabled(
                pid,
                wid,
                &element_guard,
                "double_click",
                delivery_mode.is_foreground(),
            )
            .await
            {
                return refusal;
            }

            // Thread the resolved session cursor key into the blocking AX path
            // so its ClickPulse lands on THIS session's cursor, not "default".
            let ck = cursor_key.clone();
            let foreground = delivery_mode.is_foreground();
            // AXOpen or the routed double-click can make the app activate
            // itself (opening a document or a register). In the background,
            // revert that and keep the user's key window, as `click` does.
            let probe_target = element_guard.clone();
            let clock = super::actionability::semantic_clock();
            let perform = |lease| async move {
                let prior_front = crate::apps::frontmost_pid();
                let snapshot = if foreground {
                    crate::window_change_detector::WindowChangeDetector::snapshot_without_suppression(
                    prior_front,
                )
                } else {
                    crate::window_change_detector::WindowChangeDetector::snapshot(prior_front)
                };
                let result = crate::focus_guard::with_focus_suppressed(
                    (!foreground).then_some(pid),
                    prior_front,
                    "double_click.ax",
                    || async move {
                        latency::spawn_action_blocking(move || {
                            let _lease = lease;
                            let act = || {
                                ax_double_click(
                                    pid,
                                    wid,
                                    element_guard.as_ptr(),
                                    idx,
                                    &ck,
                                    has_ax_open,
                                    foreground,
                                )
                            };
                            if !foreground {
                                return act();
                            }
                            // Upstream #6947: front, act, restore, as click's
                            // foreground AX path does; activation can enable
                            // the control.
                            let mut outcome = None;
                            crate::input::skylight::with_foreground_assist(
                                pid as libc::pid_t,
                                wid,
                                || {
                                    outcome = Some(act()?);
                                    Ok(())
                                },
                            )?;
                            outcome.ok_or_else(|| {
                                anyhow::anyhow!("foreground double_click did not execute")
                            })
                        })
                        .await
                    },
                )
                .await;
                let changes = super::finish_window_observation(snapshot).await;
                (result, changes)
            };
            let (result, changes) = match dispatch_open(
                &clock,
                cua_driver_core::actionability::DEFAULT_BUDGET,
                has_ax_open,
                mutation_lease,
                |lease, _| {
                    super::actionability::route_semantic_probe(
                        &advertised,
                        lease.as_ref(),
                        probe_target.clone(),
                        pid,
                        wid,
                        super::actionability::semantic::SemanticIntent::Open,
                    )
                },
                perform,
            )
            .await
            {
                Ok(result) => result,
                Err(refusal) => return refusal,
            };

            return match result {
                Ok(Ok(ElementDoubleClick::Ax(msg))) => {
                    ax_double_click_result(&msg, &changes.result_suffix())
                }
                Ok(Ok(ElementDoubleClick::Pixel(msg))) => {
                    ToolResult::text(format!("{msg}{}", changes.result_suffix()))
                }
                // In the background a double-click that provably sent nothing
                // is a coded refusal; in the foreground the window was fronted.
                Ok(Err(e)) => super::element_disabled_error(pid, wid, &e, foreground)
                    .or_else(|| {
                        (!foreground)
                            .then(|| super::not_delivered_refusal(&e))
                            .flatten()
                    })
                    .unwrap_or_else(|| ToolResult::error(format!("double_click failed: {e}"))),
                Err(e) => ToolResult::error(format!("Task error: {e}")),
            };
        }

        // ── Pixel path ───────────────────────────────────────────────────────
        // A pixel double-click is a click with count 2. Route it through
        // `click` so it gets the same coordinate mapping, exact-window gate,
        // window-change observation, key-focus guard and focus-steal restore.
        // Some apps activate themselves when a double-click opens something
        // (GnuCash opening an account register); before this, the pixel
        // double-click had none of those and the activation stayed (Bench v2
        // CDB-G03, CUA-1341).
        if args.get("x").and_then(|v| v.as_f64()).is_none() {
            return ToolResult::error("Either element_token or x + y must be provided.");
        }
        if args.get("y").and_then(|v| v.as_f64()).is_none() {
            return ToolResult::error("Missing required parameter: y");
        }
        let mut click_args = args;
        if let Some(map) = click_args.as_object_mut() {
            map.insert("count".into(), serde_json::json!(2));
        }
        super::click::ClickTool::new(self.state.clone())
            .invoke(click_args)
            .await
    }
}

// ── Blocking AX path ─────────────────────────────────────────────────────────

/// How an element double-click was delivered.
#[derive(Debug, PartialEq, Eq)]
pub(super) enum ElementDoubleClick<P = String> {
    /// AXOpen was sent; the message says whether the app reported an error.
    Ax(String),
    /// A pixel double-click at the element's center (the caller's report).
    Pixel(P),
}

/// A double-click delivered as AXOpen. Nothing reads back what it opened.
fn ax_double_click_result(message: &str, suffix: &str) -> ToolResult {
    ToolResult::text(format!("{message}{suffix}"))
        .with_structured(serde_json::json!({ "path": "ax", "effect": "unverifiable" }))
}

/// AXOpen first when the element advertises it, else a pixel double-click.
///
/// The AXOpen return is classified by the core write-outcome rule
/// (`click::perform_outcome`, #279). An error that may have acted (Finder
/// opens a row and still returns an error; IllegalArgument and
/// InvalidUIElement included) may have opened the item: report that, and never
/// send a pixel double-click after it, which would land in whatever opened. A
/// proved non-delivery stops with nothing else sent. Only a fall-through
/// (nothing was sent) falls back to pixels, and only in the foreground.
pub(super) fn double_click_element<P, T: Copy>(
    target: T,
    idx: usize,
    has_ax_open: bool,
    foreground: bool,
    open: impl FnOnce(T) -> crate::ax::bindings::AXError,
    pixel: impl FnOnce(T) -> anyhow::Result<P>,
) -> anyhow::Result<ElementDoubleClick<P>> {
    if has_ax_open {
        let err = open(target);
        if err == kAXErrorSuccess {
            return Ok(ElementDoubleClick::Ax(format!(
                "AXOpen performed on element [{idx}]."
            )));
        }
        match super::click::perform_outcome(err) {
            cua_driver_core::mutation::WriteOutcome::FallThrough(_) => {}
            cua_driver_core::mutation::WriteOutcome::StopNotDelivered(_) => {
                return Err(crate::input::ax_actions::ActionNotDelivered {
                    code: "action_unavailable",
                    message: format!(
                        "AXOpen was not delivered to element [{idx}] (AX error {err}); nothing \
                         was sent."
                    ),
                }
                .into());
            }
            _ => {
                return Ok(ElementDoubleClick::Ax(format!(
                    "⚠️ Sent AXOpen to element [{idx}], but the app returned AX error {err}, \
                     which does not mean it failed: it may have opened. Read the window before \
                     repeating it."
                )));
            }
        }
        if !foreground {
            return Err(crate::input::ax_actions::ActionNotDelivered {
                code: "background_unavailable",
                message: format!(
                    "AXOpen returned {err} for element [{idx}] and nothing was sent; background \
                     delivery will not improvise a pointer fallback after choosing the semantic \
                     route"
                ),
            }
            .into());
        }
        tracing::debug!(
            "AXOpen returned {err} for element [{idx}], falling back to pixel double-click"
        );
    }
    pixel(target).map(ElementDoubleClick::Pixel)
}

fn ax_double_click(
    pid: i32,
    wid: u32,
    element_ptr: usize,
    idx: usize,
    cursor_key: &str,
    has_ax_open: bool,
    foreground: bool,
) -> anyhow::Result<ElementDoubleClick> {
    let element = element_ptr as AXUIElementRef;
    // Upstream #6702/#7041: read AXEnabled right before input; after a
    // foreground activation, wait for the app to enable the control.
    if foreground {
        crate::input::ax_actions::ensure_ax_action_enabled_after_activation(
            element_ptr,
            "double_click",
        )?;
    } else {
        crate::input::ax_actions::ensure_ax_action_enabled(element_ptr, "double_click", false)?;
    }
    double_click_element(
        element,
        idx,
        has_ax_open,
        foreground,
        |target| unsafe { perform_action(target, "AXOpen") },
        |target| pixel_double_click(pid, wid, target, idx, cursor_key, foreground),
    )
}

fn pixel_double_click(
    pid: i32,
    wid: u32,
    element: AXUIElementRef,
    idx: usize,
    cursor_key: &str,
    foreground: bool,
) -> anyhow::Result<String> {
    // SAFETY: the caller holds a live reference to `element` for this call.
    let retained = unsafe { crate::ax::snapshot::RetainedElement::retain(element as usize) };
    let target = super::element_pointer::target_for(pid, wid, idx, retained);
    pixel_double_click_at(target.as_ref(), idx, foreground, |point| {
        // Drive THIS session's cursor (threaded in via `cursor_key`), not "default".
        crate::cursor::overlay::send_command(
            cursor_key.to_owned(),
            cursor_overlay::OverlayCommand::ClickPulse {
                x: point.screen.0,
                y: point.screen.1,
            },
        );
        // Use the window-local primitive (not bare click_at_xy): a plain
        // click_at_xy does NOT reliably reach a backgrounded / non-key window —
        // it no-ops on AppKit controls that hit-test the window-local stamp.
        crate::input::mouse::click_at_xy_with_window_local(
            pid,
            point.screen.0,
            point.screen.1,
            point.window_local.0,
            point.window_local.1,
            wid,
            2,
            &[],
            crate::input::mouse::WindowClickDelivery::from_foreground(foreground),
        )
    })
}

/// The pixel double-click at an element, only at its checked center
/// (`element_pointer::checked_point`: pointer admission for the exact window
/// in the background, a readable element frame and window bounds, the center
/// inside the window and on the element). A foreground fallback after AXOpen
/// and a background pointer route both pass it; anything else refuses before
/// an event is posted (#272 review).
pub(super) fn pixel_double_click_at(
    target: &dyn super::element_pointer::ElementTarget,
    idx: usize,
    foreground: bool,
    post: impl FnOnce(&super::element_pointer::ElementPoint) -> anyhow::Result<()>,
) -> anyhow::Result<String> {
    let point = super::element_pointer::checked_point(target, foreground).map_err(|refusal| {
        crate::input::ax_actions::ActionNotDelivered {
            code: refusal.code,
            message: format!("element [{idx}]: {}", refusal.message),
        }
    })?;
    post(&point)?;
    Ok(format!(
        "✅ Double-clicked element [{idx}] at ({:.1}, {:.1}).",
        point.screen.0, point.screen.1
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_axopen_error_that_may_have_opened_sends_no_pixel_double_click() {
        use crate::ax::bindings::{
            kAXErrorAPIDisabled, kAXErrorActionUnsupported, kAXErrorAttributeUnsupported,
            kAXErrorCannotComplete, kAXErrorIllegalArgument, kAXErrorInvalidUIElement,
        };
        // #279: IllegalArgument and InvalidUIElement after a sent AXOpen may
        // have opened the item too.
        for err in [
            kAXErrorCannotComplete,
            kAXErrorAttributeUnsupported,
            kAXErrorIllegalArgument,
            kAXErrorInvalidUIElement,
        ] {
            for foreground in [false, true] {
                let outcome = double_click_element::<String, _>(
                    73_u32,
                    3,
                    true,
                    foreground,
                    |target| {
                        assert_eq!(target, 73);
                        err
                    },
                    |_| panic!("pixel double-click after an AXOpen that may have opened"),
                )
                .expect("reported, not failed");
                assert!(
                    matches!(&outcome, ElementDoubleClick::Ax(message) if message.contains("may have opened")),
                    "{outcome:?}"
                );
            }
        }
        // A proved non-delivery sends nothing else, in either mode.
        for foreground in [false, true] {
            let error = double_click_element::<String, _>(
                73_u32,
                3,
                true,
                foreground,
                |_| kAXErrorAPIDisabled,
                |_| panic!("no pointer fallback after a proved non-delivery"),
            )
            .expect_err("not delivered");
            assert!(error.to_string().contains("nothing was sent"), "{error}");
        }
        // A fall-through proves nothing was sent: background fails,
        // foreground falls back to one pixel double-click.
        assert!(double_click_element::<String, _>(
            73_u32,
            3,
            true,
            false,
            |target| {
                assert_eq!(target, 73);
                kAXErrorActionUnsupported
            },
            |_| { panic!("no pointer fallback in the background") }
        )
        .is_err());
        let pixels = std::cell::Cell::new(0);
        let outcome = double_click_element::<String, _>(
            73_u32,
            3,
            true,
            true,
            |target| {
                assert_eq!(target, 73);
                kAXErrorActionUnsupported
            },
            |target| {
                assert_eq!(target, 73);
                pixels.set(pixels.get() + 1);
                Ok(String::from("pixel"))
            },
        )
        .unwrap();
        assert_eq!(outcome, ElementDoubleClick::Pixel("pixel".into()));
        assert_eq!(pixels.get(), 1);
    }

    struct AxOpenProbe;

    #[async_trait]
    impl Tool for AxOpenProbe {
        fn def(&self) -> &ToolDef {
            def()
        }

        async fn invoke(&self, _args: Value) -> ToolResult {
            ax_double_click_result("AXOpen performed on element [3].", "")
        }
    }

    #[tokio::test]
    async fn an_axopen_double_click_is_published_as_accessibility() {
        let mut registry = cua_driver_core::tool::ToolRegistry::new();
        registry.register(Box::new(AxOpenProbe));
        let result = registry
            .invoke(
                "double_click",
                serde_json::json!({"pid": 1, "window_id": 2, "element_token": "s1:3"}),
            )
            .await;
        let structured = result.structured_content.expect("published action result");
        assert_eq!(structured["route"], "accessibility", "{structured}");
        assert_eq!(structured["effect"], "unverifiable", "{structured}");
    }

    #[test]
    fn background_element_route_does_not_guess_across_actuator_classes() {
        assert_eq!(
            background_action_for_element(true),
            cua_driver_core::background_input::BackgroundAction::AxSemantic
        );
        assert_eq!(
            background_action_for_element(false),
            cua_driver_core::background_input::BackgroundAction::WindowPointer
        );
    }
}

#[cfg(test)]
mod wp3c_review_tests {
    use super::*;

    /// wp3c review: an AXOpen the app provably did not receive, and a
    /// background fall-through that stops before any pointer input, answer
    /// coded refusals with `dispatched: false`.
    #[test]
    fn an_undelivered_or_unsent_double_click_is_a_coded_refusal() {
        use crate::ax::bindings::{kAXErrorAPIDisabled, kAXErrorActionUnsupported};
        for (err, foreground, code) in [
            (kAXErrorAPIDisabled, false, "action_unavailable"),
            (kAXErrorAPIDisabled, true, "action_unavailable"),
            (kAXErrorActionUnsupported, false, "background_unavailable"),
        ] {
            let error = double_click_element::<String, _>(
                73_u32,
                3,
                true,
                foreground,
                |_| err,
                |_| panic!("no pointer fallback"),
            )
            .expect_err("nothing was sent");
            let refusal = super::super::not_delivered_refusal(&error).expect("coded refusal");
            let fields = refusal.structured_content.expect("structured refusal");
            assert_eq!(fields["code"], code, "{err} {foreground}");
            assert_eq!(fields["effect"], "refused");
            assert_eq!(fields["dispatched"], false);
        }
    }
}

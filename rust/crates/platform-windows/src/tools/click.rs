// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Cua AI, Inc.

use async_trait::async_trait;
use cua_driver_contract::ClickButton;
use cua_driver_core::{
    action_record::ActionTransport,
    protocol::ToolResult,
    tool::{Tool, ToolDef},
    tool_args::parse_legacy_click_input,
};
use serde_json::{json, Value};
use std::sync::Arc;
use windows::core::Interface as _;

use super::impl_::{
    bitmap_to_screen, element_target_rect, is_windowless_desktop_action, minimized_element_refusal,
    overlay_glide_to, overlay_glide_to_target, pin_overlay_above, resolve_cursor_key,
    resolve_onscreen_point_with_scroll, restore_foreground_polling_best_effort, reveal_uncertain_result,
    screenshot_scale,
    winui3_background_gesture, ToolState,
};

fn capture_admission_refusal(error: anyhow::Error) -> ToolResult {
    let code = cua_driver_core::capture_runtime::admission_error_code(&error);
    ToolResult::error(format!("Capture-bound click refused: {error}")).with_structured(json!({
        "code": code,
        "effect": "refused",
        "detail": error.to_string(),
    }))
}

// ── click ─────────────────────────────────────────────────────────────────────

fn posted_pixel_click_result(pid: u32, click_word: &str) -> ToolResult {
    ToolResult::text(format!("✅ Posted {click_word} to pid {pid}.")).with_structured(json!({
        "path": "post_message",
        "verified": false,
        "effect": "unverifiable"
    }))
}

/// Only a completed miss permits another transport. A provider may finish an
/// activation after the deadline, so errors must not replay the click.
fn finish_pixel_uia_attempt(
    outcome: crate::uia::windows_enum::PointInvokeOutcome,
    pid: u32,
    sx: i32,
    sy: i32,
) -> Option<ToolResult> {
    use crate::uia::windows_enum::PointInvokeOutcome;
    match outcome {
        PointInvokeOutcome::Miss => None,
        PointInvokeOutcome::Invoked => Some(
            ToolResult::text(format!(
                "✅ Performed UIA Invoke at ({sx},{sy}) for pid {pid}."
            ))
            .with_structured(json!({
                "path": "ax", "verified": false, "effect": "unverifiable"
            })),
        ),
        PointInvokeOutcome::SentFailure
        | PointInvokeOutcome::Busy
        | PointInvokeOutcome::Unavailable
        | PointInvokeOutcome::Timeout => {
            use crate::uia_mutation::PointFailure;
            let failure = match outcome {
                PointInvokeOutcome::SentFailure => PointFailure::SentFailure,
                PointInvokeOutcome::Busy => PointFailure::Busy,
                PointInvokeOutcome::Unavailable => PointFailure::Unavailable,
                PointInvokeOutcome::Timeout => PointFailure::Timeout,
                _ => unreachable!(),
            };
            Some(crate::uia_mutation::point_failure_result(failure, pid))
        }
        PointInvokeOutcome::NotResponding => {
            Some(cua_driver_core::delivery::background_unavailable_result(
                "the app did not answer a no-op message; nothing was sent".to_owned(),
                "background_unavailable",
                "the app did not answer a no-op message; nothing was sent".to_owned(),
                json!({"uia_status": "not_responding", "path": "ax", "verified": false, "effect": "refused", "nothing_sent": true}),
            ))
        }
        PointInvokeOutcome::WindowGone => {
            Some(cua_driver_core::delivery::background_unavailable_result(
                "the target window no longer exists; nothing was sent".to_owned(),
                "background_unavailable",
                "the target window no longer exists; nothing was sent".to_owned(),
                json!({"uia_status": "window_gone", "path": "ax", "verified": false, "effect": "refused", "nothing_sent": true}),
            ))
        }
    }
}

enum BackgroundElementClick {
    Refused {
        message: String,
        transport: ActionTransport,
        kind: crate::uia_mutation::RefusalKind,
        failed_calls: Vec<ActionTransport>,
    },
    Uncertain {
        message: String,
        transport: ActionTransport,
        failed_calls: Vec<ActionTransport>,
    },
    Semantic {
        message: String,
        transport: ActionTransport,
        failed_calls: Vec<ActionTransport>,
    },
    Inject {
        x: i32,
        y: i32,
        failed_calls: Vec<ActionTransport>,
    },
    Post {
        x: i32,
        y: i32,
        failed_calls: Vec<ActionTransport>,
    },
}

fn background_element_click_result(
    message: String,
    transport: ActionTransport,
    failed_calls: Vec<ActionTransport>,
) -> ToolResult {
    use cua_driver_core::action_record::{
        ActionAttempt, ActionEffect, ActionExecutionRecord, ActionFallback, ActualDelivery,
        RequestedDelivery,
    };
    let path = match transport {
        ActionTransport::WindowsTargetedInjection => "pixel",
        ActionTransport::WindowsPostMessage => "post_message",
        _ => "ax",
    };
    let mut record = ActionExecutionRecord::new(
        ActionEffect::Unverifiable,
        transport,
        RequestedDelivery::Background,
    );
    record.actual_delivery = Some(ActualDelivery::Background);
    for (index, attempted) in failed_calls.iter().copied().enumerate() {
        record.attempts.push(ActionAttempt {
            transport: attempted,
            delivery: ActualDelivery::Background,
            detail: Some("UIA provider action returned an error".into()),
        });
        record.fallbacks.push(ActionFallback {
            from: attempted,
            to: failed_calls.get(index + 1).copied().unwrap_or(transport),
            reason: "Previous UIA provider action failed".into(),
        });
    }
    ToolResult::text(message)
        .with_structured(json!({ "path": path, "verified": false, "effect": "unverifiable" }))
        .with_action_record(record)
}

fn uncertain_element_click_result(
    message: String,
    transport: ActionTransport,
    failed_calls: Vec<ActionTransport>,
) -> ToolResult {
    use cua_driver_core::action_record::{
        ActionAttempt, ActionEffect, ActionExecutionRecord, ActualDelivery, RequestedDelivery,
    };
    let mut record = ActionExecutionRecord::new(
        ActionEffect::Unverifiable,
        transport,
        RequestedDelivery::Background,
    );
    record.actual_delivery = Some(ActualDelivery::Unknown);
    for attempted in failed_calls.into_iter().chain(std::iter::once(transport)) {
        record.attempts.push(ActionAttempt {
            transport: attempted,
            delivery: ActualDelivery::Unknown,
            detail: Some("UIA provider write may have landed".into()),
        });
    }
    crate::uia_mutation::chain_error(
        message,
        json!({"path":"ax", "verified":false}),
        [cua_driver_core::mutation::WriteOutcome::StopUncertain(
            cua_driver_core::mutation::Reason::Unknown,
        )],
        true,
    )
    .with_action_record(record)
}

pub(crate) fn mutation_refusal(
    message: String,
    transport: ActionTransport,
    requested: cua_driver_core::action_record::RequestedDelivery,
    prologue: bool,
    reason: cua_driver_core::mutation::Reason,
) -> ToolResult {
    use cua_driver_core::action_record::{ActionEffect, ActionExecutionRecord};
    let effect = if prologue {
        ActionEffect::Unverifiable
    } else {
        ActionEffect::Refused
    };
    crate::uia_mutation::chain_error(
        message,
        json!({"path":"uia", "verified":false}),
        [cua_driver_core::mutation::WriteOutcome::StopNotDelivered(
            reason,
        )],
        prologue,
    )
    .with_action_record(ActionExecutionRecord::new(effect, transport, requested))
}

pub(crate) fn mutation_uncertain(
    message: String,
    path: &str,
    transport: ActionTransport,
    requested: cua_driver_core::action_record::RequestedDelivery,
    _actual: cua_driver_core::action_record::ActualDelivery,
) -> ToolResult {
    use cua_driver_core::action_record::{ActionEffect, ActionExecutionRecord};
    let mut record = ActionExecutionRecord::new(ActionEffect::Unverifiable, transport, requested);
    record.actual_delivery = Some(cua_driver_core::action_record::ActualDelivery::Unknown);
    crate::uia_mutation::chain_error(
        message,
        json!({"path":path, "verified":false}),
        [cua_driver_core::mutation::WriteOutcome::StopUncertain(
            cua_driver_core::mutation::Reason::Unknown,
        )],
        false,
    )
    .with_action_record(record)
}

/// Read a UIA pattern, telling "the element has no such pattern" apart from
/// "the read failed". Only the first may let a caller try another rung.
pub(crate) fn acquire_uia_pattern<T: windows::core::Interface>(
    element: &windows::Win32::UI::Accessibility::IUIAutomationElement,
    pattern_id: windows::Win32::UI::Accessibility::UIA_PATTERN_ID,
) -> Result<Option<T>, i32> {
    let classify = |error: windows::core::Error| {
        let code = error.code().0;
        if crate::uia_mutation::pattern_absent(code) {
            Ok(None)
        } else {
            Err(code)
        }
    };
    match unsafe { element.GetCurrentPattern(pattern_id) } {
        Ok(pattern) => match pattern.cast::<T>() {
            Ok(pattern) => Ok(Some(pattern)),
            Err(error) => classify(error),
        },
        Err(error) => classify(error),
    }
}

/// A UIA click rung that nothing was delivered through: the element refused
/// the write or its pattern could not be read.
fn refused_element_click_result(
    message: String,
    transport: ActionTransport,
    _failed_calls: Vec<ActionTransport>,
    kind: crate::uia_mutation::RefusalKind,
) -> ToolResult {
    let cua_driver_core::mutation::WriteOutcome::StopNotDelivered(reason) =
        crate::uia_mutation::AcquireVerdict::NotDelivered(kind).to_core()
    else {
        unreachable!()
    };
    mutation_refusal(
        message,
        transport,
        cua_driver_core::action_record::RequestedDelivery::Background,
        true,
        reason,
    )
}

pub(crate) async fn basis_click(state: &ToolState, tool: &str, args: &Value) -> Option<ToolResult> {
    use crate::basis_click::{self, Transport};
    use crate::basis_decision::{select_click_basis, BasisSelection};
    use crate::input::guarded_dispatch::GuardedResult;
    use cua_driver_core::window_basis::{basis_refusal, BasisRefusal, DispatchEffect};
    match select_click_basis(tool, args) {
        BasisSelection::Legacy => return None,
        BasisSelection::Invalid => return Some(basis_click::invalid_basis_arguments()),
        BasisSelection::Basis => {}
    }
    let basis_click::ClickArguments {
        pid,
        snapshot_id,
        count,
        button,
    } = match basis_click::validate_arguments(tool, args) {
        Ok(arguments) => arguments,
        Err(result) => return Some(result),
    };
    let refuse = || basis_refusal("stale_basis", "unknown_snapshot", None, None);
    let hwnd = args.get("window_id").and_then(Value::as_u64).or_else(|| {
        let (id, _) = cua_driver_core::window_basis::parse_basis_id(snapshot_id)?;
        state.snapshots.window_for_snapshot(pid, id)
    });
    let Some(hwnd) = hwnd else {
        return Some(refuse());
    };
    let session = args.get("_session_id").and_then(Value::as_str);
    let basis = match state.snapshots.basis_for(pid, hwnd, snapshot_id, session) {
        Ok(basis) => basis,
        Err(result) => return Some(result),
    };
    let frame = match crate::capture::read_live_frame(hwnd) {
        Ok(frame) => frame,
        Err(_) => return Some(basis.refusal("stale_basis", "live_frame_unavailable", None)),
    };
    let (x, y) = (args["x"].as_f64().unwrap(), args["y"].as_f64().unwrap());
    let (sx, sy) = match basis_click::early_point(&basis, Ok(frame), x, y) {
        Ok(point) => point,
        Err(refusal) => return Some(basis.refusal(refusal.code, refusal.reason, Some(&frame))),
    };
    let (Ok(sx32), Ok(sy32)) = (i32::try_from(sx), i32::try_from(sy)) else {
        return Some(basis.refusal("point_outside_window", "point_outside_window", Some(&frame)));
    };
    let modifiers: Vec<String> = args
        .get("modifier")
        .and_then(Value::as_array)
        .map(|a| {
            a.iter()
                .filter_map(Value::as_str)
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_default();
    let unavailable = crate::input::delivery::is_winui3_target_window(hwnd)
        || crate::input::has_chromium_descendant(hwnd)
        || crate::input::delivery::would_be_silently_dropped(
            hwnd,
            crate::input::delivery::EventKind::MouseClick,
        );
    let mode = basis_click::transport(
        cua_driver_core::delivery::DeliveryMode::from_args(args).is_foreground(),
        !modifiers.is_empty(),
        unavailable,
    );
    let background_only = cua_driver_core::batch_scope::current().is_some_and(|scope| {
        scope.delivery() == cua_driver_core::batch_scope::ScopeDelivery::Background
    });
    if !basis_click::transport_allowed(&mode, background_only) {
        return Some(ToolResult::error("This batch allows background delivery only.").with_structured(json!({
            "code": "foreground_not_allowed", "effect": "refused", "nothing_sent": true, "prepared_side_effects": []
        })));
    }
    if mode == Transport::Unavailable {
        return Some(basis_click::background_refusal(
            crate::input::delivery::background_unavailable_error(
                hwnd,
                crate::input::delivery::EventKind::MouseClick,
            ),
        ));
    }
    let button = button.to_owned();
    crate::recording_hooks::capture_dispatch_click_target(hwnd, pid as u32, sx32, sy32);
    let cursor_key = resolve_cursor_key(args);
    pin_overlay_above(&cursor_key, hwnd);
    overlay_glide_to(&cursor_key, sx as f64, sy as f64).await;
    crate::overlay::send_command(
        cursor_key,
        cursor_overlay::OverlayCommand::ClickPulse {
            x: sx as f64,
            y: sy as f64,
        },
    );
    let snapshot_id = snapshot_id.to_owned();
    let sent = tokio::task::spawn_blocking(move || {
        let guard = || {
            let live = crate::capture::read_live_frame(hwnd).map_err(|_| BasisRefusal {
                code: "stale_basis",
                reason: "live_frame_unavailable",
            });
            basis_click::dispatch_check(&basis, live)
        };
        let result = match mode {
            Transport::Foreground => {
                let refs: Vec<&str> = modifiers.iter().map(String::as_str).collect();
                crate::input::send_click_guarded(hwnd, sx32, sy32, count, &button, &refs, guard)
            }
            Transport::Background => Ok(crate::input::post_click_guarded(
                hwnd, sx32, sy32, count, &button, guard,
            )),
            Transport::Unavailable => unreachable!(),
        };
        (result, snapshot_id, mode)
    })
    .await;
    Some(match sent {
        Ok((Ok(result), snapshot_id, mode)) => {
            let error = matches!(&result, GuardedResult::Refused { .. })
                || match &result {
                    GuardedResult::Sent { outcome } => {
                        outcome.effect != DispatchEffect::Unverifiable
                    }
                    _ => true,
                };
            let value = basis_click::result_json(&result, &snapshot_id, (sx, sy), mode);
            if error {
                ToolResult::error("Basis click was refused or delivery could not be completed.")
                    .with_structured(value)
            } else {
                ToolResult::text("Basis click sent.").with_structured(value)
            }
        }
        Ok((Err(error), _, _)) => ToolResult::error(error.to_string()).with_structured(
            json!({"code":"dispatch_failed","effect":"refused","nothing_sent":true}),
        ),
        Err(error) => ToolResult::error(error.to_string())
            .with_structured(json!({"code":"uncertain","effect":"uncertain"})),
    })
}

pub struct ClickTool {
    pub(crate) state: Arc<ToolState>,
}

static CLICK_DEF: std::sync::OnceLock<ToolDef> = std::sync::OnceLock::new();

#[async_trait]
impl Tool for ClickTool {
    fn def(&self) -> &ToolDef {
        CLICK_DEF.get_or_init(|| ToolDef {
            name: "click".into(),
            // Description matches Swift `ClickTool.swift` semantics (left-click
            // primitive with two addressing modes — element_token via UIA, or
            // window-local pixel coords via PostMessage).  Windows-only schema
            // extras (`button`; `action` limited to "expand"; no
            // `debug_image_out`) apply here.
            description: "Left-click against a target pid. **Prefer `element_token` over \
                pixel coordinates** — element_token works on backgrounded / minimized / \
                hidden / off-desktop windows, surfaces a stable handle that survives \
                rebuilds, and tells you what you're clicking via the cached element's \
                role + label. Reach for `x, y` only when the target is a canvas / video / \
                WebGL / custom-drawn surface that doesn't appear in the UIA tree.\n\n\
                Two addressing modes:\n\n\
                - `element_token` (from the last `get_window_state` snapshot \
                of that window) — performs the UIA Invoke pattern on the cached element via \
                PostMessage. No cursor move, no focus steal. Requires a prior \
                `get_window_state(pid, window_id)` in this turn; the token is stale once \
                the next snapshot of the same window replaces it.\n\n\
                - `x`, `y` (window-local screenshot pixels, top-left origin of the PNG \
                returned by `get_window_state`). On `delivery_mode:\"background\"` (the \
                default), cua-driver FIRST does a UIA hit-test at the resolved \
                screen position: if the deepest invokable element under that point exposes \
                `InvokePattern`, it's invoked through the accessibility channel — same \
                background-safe path as the `element_token` mode, no foreground swap, no \
                visible flash. **This makes pixel clicks on UWP / WinUI3 / Win11 packaged \
                apps work flash-free out of the box** — agents should default to \
                `delivery_mode:\"background\"` even on XAML hosts whose CoreInput dispatcher \
                drops raw PostMessage. The PostMessage(WM_LBUTTONDOWN/UP) fallback only \
                runs when the UIA hit-test misses (canvas / video / WebGL / custom-drawn \
                surfaces with no UIA peer), and on those targets `delivery_mode:\"background\"` \
                returns a structured `background_unavailable` error if PostMessage is \
                known to drop too — at which point you switch to `delivery_mode:\"foreground\"`. \
                **Always try the default `background` click first and let that error be \
                your signal — do NOT pass `foreground` preemptively because the target \
                'looks like' GTK/Chromium/etc. The driver decides when background is \
                impossible; a guessed foreground click needlessly steals the user's focus.** \
                `count: 2` posts two down/up pairs for a double-click. Pixel clicks need \
                a visible on-screen window to anchor the coordinate conversion (errors \
                with `pid X has no on-screen window` otherwise).\n\n\
                Exactly one of `element_token` or (`x` AND `y`) must be provided. \
                `pid` is required for window scope and omitted for desktop scope. After a `zoom` call, \
                pass `from_zoom=true` to auto-translate zoom-image coords back to \
                full-window space.\n\n\
                Windows-only convenience: `button: \"left\"|\"right\"|\"middle\"` switches \
                the mouse button (Swift exposes right-click as a separate `right_click` \
                tool). `action: \"expand\"` on an `element_token` opens the element \
                through UIA ExpandCollapsePattern (the dropdown half of an MSAA split \
                button); other actions use the default invoke. `debug_image_out` isn't \
                supported yet.".into(),
            input_schema: json!({
                "type":"object","properties":{
                    "session": cua_driver_core::tool_schema::session_schema(),
                    "pid":{"type":"integer","description":"Target process ID for window scope. Optional with element_token (the token carries the pid). Omit with scope=desktop for screen-absolute coordinates from get_desktop_state."},
                    "window_id":{"type":"integer","description":"HWND of the target window. Omit when element_token is supplied (the token carries it)."},
                    "element_token": cua_driver_core::tool_schema::element_token_schema(),
                "snapshot_id":{"type":"string","description":"Point basis for x,y: window_basis.snapshot_id from get_window_state. Refused with stale_basis, nothing sent, if the window moved, resized or changed scale since that read."},
                    "action":{"type":"string","description":"Accessibility action for an element_token target. \"expand\" opens the element through UIA ExpandCollapsePattern, or clicks the dropdown half of an MSAA split button. Omit for the default invoke."},
                    "capture_id":{"type":"string","minLength":1,"description":"Optional ID from get_window_state or get_desktop_state. When present, x and y are admitted against that exact capture and the ID is consumed before native dispatch."},
                    "x":{"type":"number","description":"X in window-local screenshot pixels — same space as the PNG get_window_state returns. Must be provided together with y."},
                    "y":{"type":"number","description":"Y in window-local screenshot pixels. Must be provided together with x."},
                    "button": cua_driver_core::tool_schema::button_schema(),
                    "count":{"type":"integer","minimum":1,"maximum":3,"description":"Click count — 1 (single), 2 (double), 3 (triple). Default 1."},
                    "modifier": cua_driver_core::tool_schema::modifier_schema(),
                    "from_zoom":{"type":"boolean","description":"When true, x and y are pixel coordinates in the last `zoom` image for this pid. The driver maps them back to window coords."},
                    "scope":{"type":"string","enum":["window","desktop"],"default":"window","description":"Coordinate frame (default \"window\"). Pass \"desktop\" with x,y and no pid/window_id for a screen-absolute click in get_desktop_state coordinates."},
                    "delivery_mode": crate::input::delivery::delivery_mode_schema()
                },"additionalProperties":false
            }),
            read_only: false, destructive: true, idempotent: false, open_world: true,
        })
    }

    async fn invoke(&self, args: Value) -> ToolResult {
        if let Some(result) = basis_click(&self.state, "click", &args).await {
            return result;
        }
        use crate::input::delivery::{DeliveryMode, EventKind};
        use crate::uia::snapshot::SnapshotKind;
        use cua_driver_core::tool_args::ArgsExt;
        let cursor_key = resolve_cursor_key(&args);

        // ── Window-less screen-absolute branch (desktop target) ───────────────
        // When the caller gives x,y with NO pid/window_id, treat x,y as TRUE
        // SCREEN pixels. The core registry has already enforced the session's
        // effective capture scope; this local check validates the action form.
        let has_pid = args.get("pid").map(|v| !v.is_null()).unwrap_or(false);
        let has_window_id = args.get("window_id").map(|v| !v.is_null()).unwrap_or(false);
        let has_xy = args.get("x").map(|v| v.is_number()).unwrap_or(false)
            && args.get("y").map(|v| v.is_number()).unwrap_or(false);
        if !has_pid && !has_window_id && has_xy {
            if !is_windowless_desktop_action(&args) {
                return ToolResult::error(
                    "click: x,y given with no pid/window_id requires scope=\"desktop\"; \
                     use get_desktop_state to pick coordinates, or pass a pid/window_id.",
                )
                .with_structured(json!({
                    "code": "desktop_coordinate_scope_required",
                    "suggestion": "pass scope=desktop",
                }));
            }
            let input = match parse_legacy_click_input(&args) {
                Ok(input) => input,
                Err(result) => return result,
            };
            let button = input
                .button
                .unwrap_or(ClickButton::Left)
                .as_str()
                .to_owned();
            let count = input.count.unwrap_or(1) as usize;
            if count == 0 {
                return ToolResult::error("click.count must be at least 1.")
                    .with_structured(json!({ "code": "invalid_arguments" }));
            }
            let modifiers: Vec<String> = args.str_array("modifier");
            let capture_point = match self.state.capture_bridge.as_ref() {
                Some(bridge) => match bridge.admit_click(
                    &args,
                    crate::capture_admission::WindowsCaptureTarget::PrimaryDesktop,
                    input.x,
                    input.y,
                ) {
                    Ok(point) => point,
                    Err(error) => return capture_admission_refusal(error),
                },
                None if args.get("capture_id").is_some() => {
                    return capture_admission_refusal(anyhow::anyhow!(
                        "capture service is unavailable"
                    ))
                }
                None => None,
            };
            let (sx, sy) = match capture_point {
                Some((action_x, action_y)) => {
                    match crate::capture_admission::round_action_point(action_x, action_y) {
                        Ok(point) => point,
                        Err(error) => return capture_admission_refusal(error),
                    }
                }
                None => (input.x as i32, input.y as i32),
            };

            // Resolve the application window before moving the agent cursor.
            // WindowFromPoint can return a transparent layered overlay, and the
            // cursor overlay is about to occupy this exact screen point.
            let root = unsafe {
                use windows::Win32::Foundation::POINT;
                use windows::Win32::UI::WindowsAndMessaging::{
                    GetAncestor, WindowFromPoint, GA_ROOT,
                };
                let target = WindowFromPoint(POINT { x: sx, y: sy });
                if target.0.is_null() {
                    return ToolResult::error(format!("No window under screen point ({sx},{sy})."));
                }
                let root = GetAncestor(target, GA_ROOT);
                if root.0.is_null() {
                    target
                } else {
                    root
                }
            };
            let hwnd_u = root.0 as u64;

            // Animate the agent cursor to the screen point, then click.
            overlay_glide_to(&cursor_key, sx as f64, sy as f64).await;
            crate::overlay::send_command(
                cursor_key.clone(),
                cursor_overlay::OverlayCommand::ClickPulse {
                    x: sx as f64,
                    y: sy as f64,
                },
            );

            // Click the HWND that owned the pixel before the driver overlay
            // moved there. The active SendInput path performs the foreground
            // swap and UIPI checks needed for Chromium and retained-mode apps.
            let send_result = tokio::task::spawn_blocking(move || -> anyhow::Result<u64> {
                let mod_refs: Vec<&str> = modifiers.iter().map(String::as_str).collect();
                crate::input::send_click_synthesized_desktop_mods(
                    hwnd_u, sx, sy, count, &button, &mod_refs,
                )?;
                Ok(hwnd_u)
            })
            .await;
            return match send_result {
                Ok(Ok(hwnd_u)) => {
                    let click_word = match count {
                        2 => "double-click",
                        3 => "triple-click",
                        _ => "click",
                    };
                    ToolResult::text(format!(
                        "✅ Sent {click_word} via SendInput at screen ({sx},{sy}) on HWND 0x{hwnd_u:x} (desktop scope)."
                    ))
                    // SendInput dispatched but is not driver-verifiable (no read-back).
                    .with_structured(json!({ "path": "pixel", "verified": false, "effect": "unverifiable" }))
                }
                Ok(Err(e)) => ToolResult::error(e.to_string()),
                Err(e) => ToolResult::error(format!("Task error: {e}")),
            };
        }

        let pid = match args.require_u32("pid") {
            Ok(v) => v,
            Err(e) => return e,
        };
        let resolved =
            match crate::process_instance::resolve_live(&self.state.snapshots, pid as i32, &args)
                .await
            {
                Ok(r) => r,
                Err(e) => return e,
            };
        let (elem_idx, hwnd_opt, admitted) = resolved.into_parts(args.opt_u64("window_id"));
        if let Some(refusal) = super::impl_::refuse_if_disabled(pid, &admitted, "click").await {
            return refusal;
        }
        let x = args.opt_f64("x");
        let y = args.opt_f64("y");
        // Surface 5: explicit rejection of unknown buttons so a typo doesn't fall
        // through to the SendInput `_ => MOUSEEVENTF_LEFTDOWN/UP` default and
        // silently emit a left-click. Empty / missing keeps back-compat.
        let button_raw = args.str_or("button", "left").to_lowercase();
        if !matches!(button_raw.as_str(), "" | "left" | "right" | "middle") {
            return ToolResult::error(format!(
                "click: unknown button \"{button_raw}\" — expected one of left, right, middle."
            ));
        }
        let button = if button_raw.is_empty() {
            "left".to_string()
        } else {
            button_raw
        };
        let count = args.u64_or("count", 1) as usize;
        // macOS click `modifier` surface: held keys (cmd/shift/option/ctrl). Only
        // the SendInput rung (delivery_mode:"foreground" pixel tap, and the
        // window-less desktop-scope click above) can hold them; the background
        // UIA-Invoke / PostMessage rungs have no keyboard state to carry them, so
        // a modifier passed to a background click is necessarily ignored there.
        let modifiers: Vec<String> = args.str_array("modifier");
        let delivery = DeliveryMode::from_args(&args);
        if !modifiers.is_empty() && delivery != DeliveryMode::Foreground {
            return ToolResult::error(
                "click modifiers require delivery_mode:\"foreground\" on Windows; UIA and \
                 PostMessage cannot carry live modifier-key state",
            )
            .with_structured(json!({
                "code": "background_unavailable",
                "effect": "refused",
                "escalation": {
                    "recommended": "foreground",
                    "reason": "Windows modifier clicks require SendInput so the target observes live modifier state"
                }
            }));
        }
        // Optional `action` arg picks among the actions exposed in the
        // accessibility tree. Today this only changes behavior for MSAA
        // BUTTONDROPDOWN: `"expand"` clicks the right-edge (dropdown arrow
        // half) instead of the center (press half). Defaults to first
        // action in the element's `actions=[...]` list, which preserves
        // existing semantics for UIA elements.
        let action_req = args
            .get("action")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        // Resolve HWND: explicit, or auto from pid.
        let hwnd = match hwnd_opt {
            Some(h) => h,
            None => {
                let windows = tokio::task::spawn_blocking({
                    let admitted = admitted.clone();
                    move || {
                        let _admission = &admitted;
                        crate::win32::list_windows_via_win32(Some(pid))
                    }
                })
                .await
                .unwrap_or_default();
                match windows.first() {
                    Some(w) => w.hwnd,
                    None => {
                        return ToolResult::error(format!(
                            "No windows found for pid {pid}. Provide window_id."
                        ))
                    }
                }
            }
        };
        let capture_point = if args.get("capture_id").is_some() {
            if elem_idx.is_some()
                || x.is_none()
                || y.is_none()
                || hwnd_opt.is_none()
                || args.get("from_zoom").and_then(Value::as_bool) == Some(true)
            {
                return capture_admission_refusal(anyhow::anyhow!(
                    "capture_id requires x, y, and an exact window_id; element and zoom coordinates are not capture-bound"
                ));
            }
            let Some(bridge) = &self.state.capture_bridge else {
                return capture_admission_refusal(anyhow::anyhow!(
                    "capture service is unavailable"
                ));
            };
            match bridge.admit_click(
                &args,
                crate::capture_admission::WindowsCaptureTarget::Window {
                    pid,
                    window_id: hwnd,
                },
                x.unwrap(),
                y.unwrap(),
            ) {
                Ok(Some(point)) => Some(point),
                Ok(None) => unreachable!("capture_id was present"),
                Err(error) => return capture_admission_refusal(error),
            }
        } else {
            None
        };

        // Arm this only after capture admission so stale or mismatched IDs have
        // no native side effect before the click is refused.
        let _noact = match (delivery != DeliveryMode::Foreground, hwnd_opt) {
            (true, Some(h)) => {
                match crate::input::NoActivateGuard::arm(windows::Win32::Foundation::HWND(
                    h as *mut _,
                )) {
                    Ok(guard) => Some(guard),
                    Err(error) => return ToolResult::error(error.to_string()),
                }
            }
            _ => None,
        };

        if let Some(idx) = elem_idx {
            // ── MSAA dispatch (SAL/VCL targets) ────────────────────────────
            // For MSAA elements, route by role:
            //   - `action:"expand"` on BUTTONDROPDOWN → SendInput at the
            //     cached rect's right-edge (the dropdown arrow half).
            //     Opens the picker (e.g. LO Writer Font Color → SALTMPSUBFRAME).
            //   - default action / `"invoke"` → SendInput at center
            //     (matches `accDoDefaultAction` semantics for VCL controls
            //     and works through delivery_mode:"foreground"). We DO NOT call
            //     `accDoDefaultAction` here because LO's MSAA impl applies
            //     the change asynchronously and returns S_OK either way,
            //     so the visible behavior is the same as a center click.
            if let Some((SnapshotKind::Msaa, role)) = admitted
                .as_ref()
                .map(|element| (element.kind, element.msaa_role))
            {
                const ROLE_BUTTONDROPDOWN: i32 = 0x38;
                const ROLE_BUTTONMENU: i32 = 0x39;
                const ROLE_BUTTONDROPDOWNGRID: i32 = 0x3A;
                const ROLE_SPLITBUTTON: i32 = 0x3E;
                let want_expand = action_req.as_deref() == Some("expand");
                let is_dropdown_role = matches!(
                    role,
                    Some(
                        ROLE_BUTTONDROPDOWN
                            | ROLE_BUTTONMENU
                            | ROLE_BUTTONDROPDOWNGRID
                            | ROLE_SPLITBUTTON
                    )
                );
                let (tx, ty) = if want_expand && is_dropdown_role {
                    match admitted.as_ref().and_then(|element| element.rect) {
                        Some((_l, t, r, b)) => {
                            // Right-edge of the SplitButton — the dropdown
                            // arrow half. -4 puts the click safely inside
                            // the arrow region for the typical ~12-16 px
                            // arrow width VCL uses.
                            (r - 4, (t + b) / 2)
                        }
                        None => {
                            return ToolResult::error(format!(
                                "MSAA element [{idx}] has no cached rect — \
                                 cannot dispatch action:\"expand\". Re-run \
                                 get_window_state to refresh the cache."
                            ));
                        }
                    }
                } else if want_expand && !is_dropdown_role {
                    return ToolResult::error(format!(
                        "action:\"expand\" requested for MSAA element [{idx}] \
                         but its role ({role:?}) is not a dropdown button. \
                         Use action:\"invoke\" or omit the action arg."
                    ));
                } else {
                    match admitted.as_ref().map(|element| element.center) {
                        Some(v) => v,
                        None => {
                            return ToolResult::error(format!(
                                "MSAA element [{idx}] not in cache for hwnd={hwnd}. \
                             Call get_window_state first."
                            ))
                        }
                    }
                };
                if admitted
                    .as_ref()
                    .is_some_and(|element| element.read_while_minimized)
                {
                    if let Err(refusal) =
                        resolve_onscreen_point_with_scroll(&admitted, hwnd, idx, tx, ty, "clicking")
                    {
                        return refusal;
                    }
                }
                pin_overlay_above(&cursor_key, hwnd);
                overlay_glide_to_target(
                    &cursor_key,
                    tx as f64,
                    ty as f64,
                    element_target_rect(admitted.as_ref(), (0, 0)),
                )
                .await;
                crate::overlay::send_command(
                    cursor_key.clone(),
                    cursor_overlay::OverlayCommand::ClickPulse {
                        x: tx as f64,
                        y: ty as f64,
                    },
                );
                let btn_fg = button.clone();
                let prev_fg_addr = unsafe {
                    windows::Win32::UI::WindowsAndMessaging::GetForegroundWindow().0 as usize
                };
                let mods_owned = modifiers.clone();
                let activate = delivery == DeliveryMode::Foreground;
                let send_result = tokio::task::spawn_blocking({
                    let admitted = admitted.clone();
                    move || {
                        let _admission = &admitted;
                        let mod_refs: Vec<&str> = mods_owned.iter().map(String::as_str).collect();
                        if activate {
                            crate::input::send_click_synthesized_active_mods(
                                hwnd, tx, ty, count, &btn_fg, &mod_refs,
                            )
                        } else {
                            crate::input::send_click_synthesized_mods(
                                hwnd, tx, ty, count, &btn_fg, &mod_refs,
                            )
                        }
                    }
                })
                .await;
                tokio::spawn(restore_foreground_polling_best_effort(prev_fg_addr, pid));
                let half = if want_expand { "dropdown" } else { "press" };
                return match send_result {
                    Ok(Ok(())) => ToolResult::text(format!(
                        "✅ Performed SendInput click on MSAA [{idx}] {half} half at ({tx},{ty})."
                    ))
                    .with_structured(
                        json!({ "path": "msaa", "verified": false, "effect": "unverifiable" }),
                    ),
                    Ok(Err(e)) => ToolResult::error(e.to_string()),
                    Err(e) => ToolResult::error(format!("Task error: {e}")),
                };
            }

            // UIA path: get cached center (no COM call needed — captured at walk time).
            let (cx, cy) = match admitted.as_ref().map(|element| element.center) {
                Some(v) => v,
                None => {
                    return ToolResult::error(format!(
                        "Element {idx} not in cache for hwnd={hwnd}. Call get_window_state first."
                    ))
                }
            };
            // NB: the off-screen guard is applied per-delivery-path below (the
            // foreground SendInput tap and the background coordinate injection),
            // NOT here — a background UIA Invoke fires the element's handler
            // regardless of whether the element is scrolled into view, so a broad
            // guard would wrongly block legitimate actions like opening a modal
            // from an off-screen button.
            // Step 2: pin overlay to target window, then animate to screen coords.
            pin_overlay_above(&cursor_key, hwnd);
            overlay_glide_to_target(
                &cursor_key,
                cx as f64,
                cy as f64,
                element_target_rect(admitted.as_ref(), (0, 0)),
            )
            .await;
            // Step 3: click pulse + actual click.
            crate::overlay::send_command(
                cursor_key.clone(),
                cursor_overlay::OverlayCommand::ClickPulse {
                    x: cx as f64,
                    y: cy as f64,
                },
            );
            let btn = button.clone();

            // An explicit accessibility action is a semantic request, not a
            // pixel gesture hint. Route expand through ExpandCollapsePattern
            // even when foreground delivery was allowed; this reliably opens
            // WPF/WinUI menus and tree nodes whose visual click target is
            // transient or scroll-adjusted.
            if action_req.as_deref() == Some("expand") {
                use crate::uia_mutation::{
                    classify_acquisition, classify_sent_call, expand_route, refusal_text,
                    uncertain_text, AcquireVerdict, MutationRoute, RungOutcome, SentVerdict,
                };
                let expand =
                    tokio::task::spawn_blocking({
                        let admitted = admitted.clone();
                        move || -> Result<RungOutcome, String> {
                            let retained = admitted.as_ref().ok_or_else(|| {
                                format!("element [{idx}] is not in the UIA cache")
                            })?;
                            if !retained.is_uia() {
                                return Err(format!("element [{idx}] is not a UIA element"));
                            }
                            use windows::core::Interface;
                            use windows::Win32::UI::Accessibility::{
                                IUIAutomationElement, IUIAutomationExpandCollapsePattern,
                                UIA_ExpandCollapsePatternId,
                            };
                            let element = std::mem::ManuallyDrop::new(unsafe {
                                IUIAutomationElement::from_raw(retained.as_ptr() as *mut _)
                            });
                            let pattern = match acquire_uia_pattern::<
                                IUIAutomationExpandCollapsePattern,
                            >(
                                &element, UIA_ExpandCollapsePatternId
                            ) {
                                Ok(Some(pattern)) => pattern,
                                Ok(None) => return Ok(RungOutcome::Absent),
                                Err(hr) => return Ok(RungOutcome::AcquireFailed(hr)),
                            };
                            let result = crate::uia::fg_bypass::run_with_uwp_bypass(
                                hwnd as isize,
                                || unsafe { crate::uia::sent_call::expand(&pattern) },
                            );
                            Ok(RungOutcome::Called(result))
                        }
                    })
                    .await;
                use cua_driver_core::action_record::{ActualDelivery, RequestedDelivery};
                let (requested, actual) = if delivery == DeliveryMode::Foreground {
                    (RequestedDelivery::Foreground, ActualDelivery::Foreground)
                } else {
                    (RequestedDelivery::Background, ActualDelivery::Background)
                };
                let uncertain = |message: String| {
                    mutation_uncertain(
                        message,
                        "uia_expand_collapse",
                        ActionTransport::WindowsUiaExpandCollapse,
                        requested,
                        actual,
                    )
                };
                return match expand {
                    Ok(Ok(outcome)) => match expand_route(outcome) {
                        MutationRoute::Done => ToolResult::text(format!("✅ Expanded UIA element [{idx}] via ExpandCollapsePattern."))
                            .with_structured(json!({ "path": "uia_expand_collapse", "verified": false, "effect": "unverifiable" })),
                        MutationRoute::FallThrough => ToolResult::error("ExpandCollapsePattern unavailable"),
                        MutationRoute::Refused => {
                            let (kind, hr) = match outcome {
                                RungOutcome::AcquireFailed(hr) => match classify_acquisition(hr) {
                                    AcquireVerdict::NotDelivered(kind) => (kind, hr),
                                    AcquireVerdict::Absent => unreachable!("absent acquisition cannot be refused"),
                                },
                                RungOutcome::Called(_) => unreachable!("sent writes cannot be refused"),
                                RungOutcome::Absent => unreachable!("absent expand route is not refused"),
                            };
                            mutation_refusal(refusal_text(kind, hr), ActionTransport::WindowsUiaExpandCollapse, requested, true,
                                match crate::uia_mutation::AcquireVerdict::NotDelivered(kind).to_core() { cua_driver_core::mutation::WriteOutcome::StopNotDelivered(reason) => reason, _ => unreachable!() })
                        },
                        MutationRoute::Uncertain => {
                            let RungOutcome::Called(hr) = outcome else { unreachable!("uncertain expand route requires a write") };
                            let SentVerdict::Uncertain(kind) = classify_sent_call(hr) else { unreachable!("uncertain expand route requires an uncertain HRESULT") };
                            uncertain(format!("ExpandCollapse.Expand on [{idx}]: {}", uncertain_text(kind, hr)))
                        },
                        MutationRoute::Partial => unreachable!("expand makes only one write"),
                    },
                    Ok(Err(error)) => ToolResult::error(error),
                    Err(error) => uncertain(format!("Expand worker failed: {error}; it may have expanded. Read the state before acting again.")),
                };
            }

            // delivery_mode:"foreground" — skip UIA Invoke and use SendInput at the
            // cached element center. The caller explicitly chose foreground
            // delivery; UIA Invoke would be background-safe (which they
            // rejected) and might fail on elements that don't support
            // InvokePattern anyway.
            if delivery == DeliveryMode::Foreground {
                // Foreground delivery is a real SendInput tap at (cx,cy); if the
                // element is off-screen, scroll it into view and re-resolve so the
                // tap doesn't land on the taskbar, else keep the clean failure.
                // Modified selection needs one stronger preflight: some WPF
                // ScrollViewers report a clipped item as on-screen because its
                // stale rectangle is still inside the outer HWND. Ask the item
                // to scroll itself into view before trusting that rectangle.
                let recorded_center = (cx, cy);
                let (cx, cy) = if !modifiers.is_empty() {
                    if let Some(retained) = admitted.as_ref().filter(|element| element.is_uia()) {
                        use crate::uia::scroll::RevealOutcome;
                        match unsafe {
                            crate::uia::scroll::scroll_into_view_and_recenter(
                                hwnd,
                                retained.as_ptr(),
                            )
                        } {
                            RevealOutcome::Revealed(point) | RevealOutcome::NotNeeded(point) => {
                                point
                            }
                            RevealOutcome::NotRevealed { writes_sent: false } => (cx, cy),
                            RevealOutcome::Uncertain
                            | RevealOutcome::NotRevealed { writes_sent: true } => {
                                return reveal_uncertain_result()
                            }
                        }
                    } else {
                        (cx, cy)
                    }
                } else {
                    (cx, cy)
                };
                let (cx, cy) = match resolve_onscreen_point_with_scroll(
                    &admitted,
                    hwnd,
                    idx,
                    cx,
                    cy,
                    "a foreground click",
                ) {
                    Ok(p) => p,
                    Err(result) => return crate::uia_mutation::after_prologue(result),
                };
                if (cx, cy) != recorded_center {
                    crate::recording_hooks::capture_dispatch_click_target(hwnd, pid, cx, cy);
                }
                let btn_fg = btn.clone();
                let mods_owned = modifiers.clone();
                let prev_fg_addr = unsafe {
                    windows::Win32::UI::WindowsAndMessaging::GetForegroundWindow().0 as usize
                };
                let send_result = tokio::task::spawn_blocking({
                    let admitted = admitted.clone();
                    move || {
                        let _admission = &admitted;
                        let mod_refs: Vec<&str> = mods_owned.iter().map(String::as_str).collect();
                        crate::input::send_click_synthesized_active_mods(
                            hwnd, cx, cy, count, &btn_fg, &mod_refs,
                        )
                    }
                })
                .await;
                tokio::spawn(restore_foreground_polling_best_effort(prev_fg_addr, pid));
                return match send_result {
                    Ok(Ok(())) => ToolResult::text(format!(
                        "✅ Performed SendInput click on [{idx}] at screen ({cx},{cy}) (delivery_mode:foreground)."
                    ))
                    .with_structured(json!({ "path": "pixel", "verified": false, "effect": "unverifiable" })),
                    Ok(Err(e)) => ToolResult::error(e.to_string()),
                    Err(e) => ToolResult::error(format!("Task error: {e}")),
                };
            }
            // UIA and posted-message routes can report success without a
            // user-visible effect on an iconic target. Refuse before any
            // background route selects or dispatches its transport.
            if let Some(result) = minimized_element_refusal(hwnd, idx, cx, cy, "clicking") {
                return result;
            }
            // delivery_mode:"background" on WinUI3 for double / right / middle: single
            // left falls through to the UIA Invoke path below (already drives
            // WinUI3). Double-left lands via a double UIA Invoke; right/middle
            // can't both land and hold the contract → structured error.
            if delivery == DeliveryMode::Background
                && (count > 1 || btn == "right" || btn == "middle")
            {
                if let Some(r) =
                    winui3_background_gesture(&admitted, pid, hwnd, Some(idx), count, &btn).await
                {
                    return r;
                }
            }
            // Chromium's UIA Invoke raises a fully occluded renderer, while a
            // targeted PostMessage left click reaches the renderer without
            // changing foreground, z-order, or the real cursor. Keep AX for
            // target resolution and use the posted-message transport only for
            // the empirically verified single-left-click shape.
            if delivery == DeliveryMode::Background
                && btn == "left"
                && count == 1
                && crate::input::is_chromium_target_window(hwnd)
            {
                // This route posts at the cached center without
                // `resolve_onscreen_point_with_scroll`, so check it here.
                if admitted
                    .as_ref()
                    .is_some_and(|element| element.read_while_minimized)
                {
                    let message = format!(
                        "Element [{idx}] resolves to ({cx},{cy}), which was read while \
                         window 0x{hwnd:x} was minimized. Re-snapshot with get_window_state \
                         before retrying."
                    );
                    return ToolResult::error(message.clone()).with_structured(json!({
                        "status": "refused",
                        "code": "element_not_visible",
                        "effect": "refused",
                        "refusal": {
                            "code": "element_not_visible",
                            "message": message,
                        }
                    }));
                }
                let posted = tokio::task::spawn_blocking({
                    let admitted = admitted.clone();
                    move || {
                        let _admission = &admitted;
                        crate::input::post_click_screen(hwnd, cx, cy, count, &btn)
                    }
                })
                .await;
                return match posted {
                    Ok(Ok(())) => background_element_click_result(
                        format!(
                            "✅ Posted click on Chromium element [{idx}] at screen ({cx},{cy}) \
                         (background, no foreground swap)."
                        ),
                        ActionTransport::WindowsPostMessage,
                        vec![],
                    ),
                    Ok(Err(error)) => ToolResult::error(error.to_string()),
                    Err(error) => ToolResult::error(format!("Task error: {error}")),
                };
            }
            // Try UIA Invoke first (it works for UWP / modern XAML / web
            // content where PostMessage(WM_LBUTTONDOWN) hits the outer
            // HWND but never reaches the inner XAML/composition element).
            // Fall through to PostMessage when:
            //   - element doesn't support InvokePattern (most text inputs,
            //     non-interactive elements)
            //   - right-click (UIA's ExpandCollapse / ShowContextMenu are
            //     different patterns; keep PostMessage for now)
            //   - count > 1 (double-click semantics aren't an Invoke
            //     concept — PostMessage produces the actual WM_LBUTTONDBLCLK)
            let use_uia_invoke = (btn == "left" || btn == "middle") && count == 1;
            let result = tokio::task::spawn_blocking({ let admitted = admitted.clone(); move || -> Result<BackgroundElementClick, ToolResult> {
                let _admission = &admitted;
                let mut failed_calls = Vec::new();
                // Direct Chromium UIA Invoke can return S_OK without firing a
                // DOM event while occluded. Try the honest coordinate actuator
                // first: it lands while visible and reports occlusion without
                // raising the window when hidden.
                if delivery == DeliveryMode::Background
                    && crate::input::is_chromium_target_window(hwnd)
                {
                    let (cx, cy) = resolve_onscreen_point_with_scroll(
                        &admitted, hwnd, idx, cx, cy, "clicking",
                    ).map_err(crate::uia_mutation::after_prologue)?;
                    return Ok(BackgroundElementClick::Inject { x: cx, y: cy, failed_calls });
                }
                if use_uia_invoke {
                    // Retain the element out of the cache (AddRef under the
                    // cache lock) so it can't be freed by a concurrent
                    // get_window_state on the same (pid, hwnd) while this click
                    // is mid-flight (use-after-free → daemon crash). The guard
                    // lives to the end of this `if let` block, past every UIA
                    // pattern dispatch below; its Release fires when it drops.
                    if let Some(element_guard) = admitted.as_ref() {
                        let ptr = element_guard.as_ptr();
                        use windows::Win32::UI::Accessibility::{
                            IUIAutomationElement, IUIAutomationInvokePattern,
                            IUIAutomationTogglePattern, IUIAutomationSelectionItemPattern,
                            IUIAutomationExpandCollapsePattern,
                            UIA_InvokePatternId, UIA_TogglePatternId,
                            UIA_SelectionItemPatternId, UIA_ExpandCollapsePatternId,
                        };
                        use windows::core::Interface;
                        let elem: IUIAutomationElement =
                            unsafe { IUIAutomationElement::from_raw(ptr as *mut _) };

                        use crate::uia_mutation::{click_plan_from_chain, run_rungs, ClickPlan, RungOutcome};
                        macro_rules! uia_rung {
                            ($id:expr, $ty:ty, $call:path) => {
                                Box::new(|| {
                                    match acquire_uia_pattern::<$ty>(&elem, $id) {
                                        Ok(Some(pattern)) => RungOutcome::Called(
                                            crate::uia::fg_bypass::run_with_uwp_bypass(hwnd as isize, || unsafe { $call(&pattern) })),
                                        Ok(None) => RungOutcome::Absent,
                                        Err(hresult) => RungOutcome::AcquireFailed(hresult),
                                    }
                                })
                            };
                        }
                        let plan = click_plan_from_chain(run_rungs(vec![
                            uia_rung!(UIA_InvokePatternId, IUIAutomationInvokePattern, crate::uia::sent_call::invoke),
                            uia_rung!(UIA_TogglePatternId, IUIAutomationTogglePattern, crate::uia::sent_call::toggle),
                            uia_rung!(UIA_SelectionItemPatternId, IUIAutomationSelectionItemPattern, crate::uia::sent_call::select),
                            uia_rung!(UIA_ExpandCollapsePatternId, IUIAutomationExpandCollapsePattern, crate::uia::sent_call::expand),
                        ]));
                        std::mem::forget(elem);
                        let names = ["Invoke", "Toggle", "SelectionItem.Select", "ExpandCollapse.Expand"];
                        let transports = [ActionTransport::WindowsUiaInvoke, ActionTransport::WindowsUiaToggle,
                            ActionTransport::WindowsUiaSelection, ActionTransport::WindowsUiaExpandCollapse];
                        match plan {
                            ClickPlan::Semantic { rung, failed } => {
                                return Ok(BackgroundElementClick::Semantic {
                                    message: format!("✅ Performed UIA {} on [{idx}] (screen ({cx},{cy})).", names[rung]),
                                    transport: transports[rung], failed_calls: failed.into_iter().map(|i| transports[i]).collect(),
                                });
                            }
                            ClickPlan::Physical { failed } => failed_calls.extend(failed.into_iter().map(|i| transports[i])),
                            ClickPlan::Refused { rung, kind, hresult, failed } => return Ok(BackgroundElementClick::Refused {
                                message: format!("UIA {} on [{idx}]: {}", names[rung], crate::uia_mutation::refusal_text(kind, hresult)),
                                transport: transports[rung], kind, failed_calls: failed.into_iter().map(|i| transports[i]).collect(),
                            }),
                            ClickPlan::Uncertain { rung, kind, hresult, failed } => return Ok(BackgroundElementClick::Uncertain {
                                message: format!("UIA {} on [{idx}] returned 0x{:08X} ({kind:?}); the action may have taken effect. Read the window state before acting again.", names[rung], hresult as u32),
                                transport: transports[rung], failed_calls: failed.into_iter().map(|i| transports[i]).collect(),
                            }),
                        }
                    }
                }
                if delivery == DeliveryMode::Background
                    && crate::input::delivery::would_be_silently_dropped(
                        hwnd,
                        EventKind::MouseClick,
                    )
                {
                    let (cx, cy) = resolve_onscreen_point_with_scroll(
                        &admitted,
                        hwnd,
                        idx,
                        cx,
                        cy,
                        "clicking",
                    ).map_err(crate::uia_mutation::after_prologue)?;
                    return Ok(BackgroundElementClick::Inject { x: cx, y: cy, failed_calls });
                }
                if admitted.as_ref().is_some_and(|element| element.read_while_minimized) {
                    let (x, y) =
                        resolve_onscreen_point_with_scroll(&admitted, hwnd, idx, cx, cy, "clicking")
                            .map_err(crate::uia_mutation::after_prologue)?;
                    return Ok(BackgroundElementClick::Post { x, y, failed_calls });
                }
                Ok(BackgroundElementClick::Post { x: cx, y: cy, failed_calls })
            } }).await;
            let plan = match result {
                Ok(Ok(plan)) => plan,
                Ok(Err(result)) => return result,
                Err(e) => return crate::uia_mutation::chain_error(format!("UIA click worker failed: {e}; the action may have landed. Read the state before acting again."), json!({"path":"ax", "verified":false}),
                    [cua_driver_core::mutation::WriteOutcome::StopUncertain(cua_driver_core::mutation::Reason::Unknown)], true),
            };
            let (x, y, failed_calls, inject) = match plan {
                BackgroundElementClick::Semantic {
                    message,
                    transport,
                    failed_calls,
                } => {
                    return background_element_click_result(message, transport, failed_calls);
                }
                BackgroundElementClick::Refused {
                    message,
                    transport,
                    kind,
                    failed_calls,
                } => return refused_element_click_result(message, transport, failed_calls, kind),
                BackgroundElementClick::Uncertain {
                    message,
                    transport,
                    failed_calls,
                } => return uncertain_element_click_result(message, transport, failed_calls),
                BackgroundElementClick::Inject { x, y, failed_calls } => (x, y, failed_calls, true),
                BackgroundElementClick::Post { x, y, failed_calls } => (x, y, failed_calls, false),
            };
            // Return to the dispatch task before capturing: task-local recording
            // context does not propagate into the blocking scroll resolver.
            if (x, y) != (cx, cy) {
                crate::recording_hooks::capture_dispatch_click_target(hwnd, pid, x, y);
            }
            let btn = button.clone();
            let action_name = if btn == "right" {
                "ShowMenu"
            } else {
                "PostMessage click"
            };
            let sent = tokio::task::spawn_blocking({
                let admitted = admitted.clone();
                move || {
                    let _admission = &admitted;
                    if inject {
                        crate::input::inject_click_screen(hwnd, x, y, count, &btn)
                    } else {
                        crate::input::post_click_screen(hwnd, x, y, count, &btn)
                    }
                }
            })
            .await;
            match sent {
                Ok(Ok(())) => {
                    let (message, transport) = if inject {
                        (format!("✅ Injected click on [{idx}] (screen ({x},{y}), background, no foreground swap)."),
                            ActionTransport::WindowsTargetedInjection)
                    } else {
                        (
                            format!("✅ Performed {action_name} on [{idx}] (screen ({x},{y}))."),
                            ActionTransport::WindowsPostMessage,
                        )
                    };
                    background_element_click_result(message, transport, failed_calls)
                }
                Ok(Err(error)) if inject => {
                    crate::input::delivery::background_unavailable_error_with_cause(
                        hwnd,
                        EventKind::MouseClick,
                        error.to_string(),
                    )
                }
                Ok(Err(error)) => ToolResult::error(error.to_string()),
                Err(error) => ToolResult::error(format!("Task error: {error}")),
            }
        } else if let (Some(mut px), Some(mut py)) = (x, y) {
            let from_zoom = args
                .get("from_zoom")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            if let Some((mapped_x, mapped_y)) = capture_point {
                let (mapped_x, mapped_y) =
                    match crate::capture_admission::round_action_point(mapped_x, mapped_y) {
                        Ok(point) => point,
                        Err(error) => return capture_admission_refusal(error),
                    };
                px = f64::from(mapped_x);
                py = f64::from(mapped_y);
            } else if from_zoom {
                match self.state.zoom_context(&args, pid, Some(hwnd)) {
                    Ok(ctx) => {
                        let (wx, wy) = ctx.zoom_to_window(px, py);
                        px = wx;
                        py = wy;
                    }
                    Err(refusal) => return refusal,
                }
            } else if !args.bool_or("_native_coordinates", false) {
                let ratio = match screenshot_scale(&self.state, &args, pid, Some(hwnd)) {
                    Ok(ratio) => ratio,
                    Err(refusal) => return refusal,
                };
                px *= ratio;
                py *= ratio;
            }
            // px/py are bitmap pixels — see `bitmap_to_screen` for the
            // mapping (DWM-frame top-left + 1-px inset, NOT ClientToScreen).
            let (sx_i, sy_i) = bitmap_to_screen(hwnd, px as i32, py as i32);
            let (sx, sy) = (sx_i as f64, sy_i as f64);
            pin_overlay_above(&cursor_key, hwnd);
            overlay_glide_to(&cursor_key, sx, sy).await;
            crate::overlay::send_command(
                cursor_key.clone(),
                cursor_overlay::OverlayCommand::ClickPulse { x: sx, y: sy },
            );
            let btn = button.clone();
            // Vision-mode (x, y) dispatch is **layered**:
            //
            //   1. UIA hit-test in the target HWND's subtree. If the
            //      deepest InvokePattern-bearing element at (sx, sy) is
            //      found, fire `Invoke()`. Works occluded, no focus
            //      steal, and is the **only** path that lands on UWP /
            //      WebView2 / DirectComposition surfaces — those route
            //      input through `Windows.UI.Input`, not WM_LBUTTONDOWN.
            //
            //   2. If UIA completes with a miss (no Invokable element under the
            //      pixel inside `hwnd`, or `hwnd` has no useful UIA
            //      tree at all), fall through to
            //      `PostMessage(WM_LBUTTONDOWN/UP)` against the deepest
            //      child HWND at the screen point. Covers plain Win32
            //      controls without UIA InvokePattern (most edit
            //      fields, custom-drawn widgets, native containers) and
            //      apps that don't expose UIA at all.
            //
            // The combination closes the gap macOS gets for free from
            // `CGEventPostToPSN` (per-pid event routing): UIA covers
            // UWP-style targets without z-order tricks, PostMessage
            // covers plain Win32 without z-order tricks. Neither path
            // moves the OS cursor or activates the receiver.
            //
            // UIA path runs only for single left/middle clicks —
            // right-click (UIA has no by-coord `ShowContextMenu`) and
            // multi-click (`Invoke()` is single-fire by definition)
            // skip directly to PostMessage.
            // delivery_mode:"foreground" short-circuit — caller explicitly chose
            // foreground delivery. Skip the UIA hit-test (UIA Invoke is
            // background-safe and would deliver the click without the
            // foreground swap the caller asked for).
            if delivery == DeliveryMode::Foreground {
                let prev_fg_addr = unsafe {
                    windows::Win32::UI::WindowsAndMessaging::GetForegroundWindow().0 as usize
                };
                let mods_owned = modifiers.clone();
                let send_result = tokio::task::spawn_blocking({
                    let admitted = admitted.clone();
                    move || {
                        let _admission = &admitted;
                        let mod_refs: Vec<&str> = mods_owned.iter().map(String::as_str).collect();
                        crate::input::send_click_synthesized_active_mods(
                            hwnd, sx as i32, sy as i32, count, &btn, &mod_refs,
                        )
                    }
                })
                .await;
                tokio::spawn(restore_foreground_polling_best_effort(prev_fg_addr, pid));
                return match send_result {
                    Ok(Ok(())) => {
                        let click_word = match count {
                            2 => "double-click",
                            3 => "triple-click",
                            _ => "click",
                        };
                        ToolResult::text(format!(
                            "✅ Sent {click_word} via SendInput to pid {pid} at ({sx},{sy}) (delivery_mode:foreground)."
                        ))
                        .with_structured(json!({ "path": "pixel", "verified": false, "effect": "unverifiable" }))
                    }
                    Ok(Err(e)) => ToolResult::error(e.to_string()),
                    Err(e) => ToolResult::error(format!("Task error: {e}")),
                };
            }

            // delivery_mode:"background" on WinUI3 for double / right / middle (pixel
            // path, no element_token → no cached element to UIA-Invoke): these
            // can't both land and hold the contract on WinUI3 (posted input is
            // ignored, the pen injector steals foreground) → structured error.
            if delivery == DeliveryMode::Background
                && (count > 1 || btn == "right" || btn == "middle")
            {
                if let Some(r) =
                    winui3_background_gesture(&admitted, pid, hwnd, None, count, &btn).await
                {
                    return r;
                }
            }
            // Match the AX-addressed Chromium route above: the point remains
            // PX-resolved, but transport uses the background-safe posted
            // message path proven against the fully occluded fixture.
            if delivery == DeliveryMode::Background
                && btn == "left"
                && count == 1
                && crate::input::is_chromium_target_window(hwnd)
            {
                let posted = tokio::task::spawn_blocking({
                    let admitted = admitted.clone();
                    move || {
                        let _admission = &admitted;
                        crate::input::post_click_screen(hwnd, sx_i, sy_i, count, &btn)
                    }
                })
                .await;
                return match posted {
                    Ok(Ok(())) => ToolResult::text(format!(
                        "✅ Posted click to Chromium pid {pid} at ({sx},{sy}) \
                         (background, no foreground swap)."
                    ))
                    .with_structured(json!({
                        "path": "post_message",
                        "verified": false,
                        "effect": "unverifiable"
                    })),
                    Ok(Err(error)) => ToolResult::error(error.to_string()),
                    Err(error) => ToolResult::error(format!("Task error: {error}")),
                };
            }
            // As above, bypass Chromium's false-positive UIA Invoke and use
            // the coordinate actuator before attempting any accessibility hit
            // test. The actuator itself distinguishes visible delivery from a
            // fully occluded structured refusal.
            if delivery == DeliveryMode::Background && crate::input::is_chromium_target_window(hwnd)
            {
                let btn2 = btn.clone();
                let inj = tokio::task::spawn_blocking({
                    let admitted = admitted.clone();
                    move || {
                        let _admission = &admitted;
                        crate::input::inject_click_screen(hwnd, sx as i32, sy as i32, count, &btn2)
                    }
                })
                .await;
                return match inj {
                    Ok(Ok(())) => ToolResult::text(format!(
                        "✅ Injected click to pid {pid} at ({sx},{sy}) (background, no foreground swap)."
                    ))
                    .with_structured(json!({ "path": "pixel", "verified": false, "effect": "unverifiable" })),
                    Ok(Err(error)) => crate::input::delivery::background_unavailable_error_with_cause(
                        hwnd,
                        EventKind::MouseClick,
                        error.to_string(),
                    ),
                    Err(error) => ToolResult::error(format!("Task error: {error}")),
                };
            }
            let use_uia = (btn == "left" || btn == "middle") && count == 1;
            if use_uia {
                let outcome = tokio::task::spawn_blocking({
                    let admitted = admitted.clone();
                    move || {
                        let _admission = &admitted;
                        crate::uia::windows_enum::try_invoke_in_window_at_point(
                            hwnd as isize,
                            sx as i32,
                            sy as i32,
                        )
                    }
                })
                .await
                .unwrap_or(crate::uia::windows_enum::PointInvokeOutcome::SentFailure);
                if let Some(result) = finish_pixel_uia_attempt(outcome, pid, sx_i, sy_i) {
                    return result;
                }
            }

            // UIA completed with a miss or was not applicable. Known dropped surfaces other than direct
            // Chromium (handled above) get one targeted injection attempt.
            if delivery == DeliveryMode::Background
                && crate::input::delivery::would_be_silently_dropped(hwnd, EventKind::MouseClick)
            {
                let btn2 = btn.clone();
                let inj = tokio::task::spawn_blocking({
                    let admitted = admitted.clone();
                    move || {
                        let _admission = &admitted;
                        crate::input::inject_click_screen(hwnd, sx as i32, sy as i32, count, &btn2)
                    }
                })
                .await;
                return match inj {
                    Ok(Ok(())) => ToolResult::text(format!(
                        "✅ Injected click to pid {pid} at ({sx},{sy}) (background, no foreground swap)."
                    ))
                    .with_structured(json!({ "path": "pixel", "verified": false, "effect": "unverifiable" })),
                    Ok(Err(error)) => crate::input::delivery::background_unavailable_error_with_cause(
                        hwnd,
                        EventKind::MouseClick,
                        error.to_string(),
                    ),
                    Err(error) => ToolResult::error(format!("Task error: {error}")),
                };
            }

            // delivery_mode:"background", non-dropped + non-UIA click. This
            // includes plain Win32 targets with no invokable UIA element.
            // Attempt PostMessage without a
            // foreground swap; the caller must verify the resulting fixture state.
            // Foreground was handled at the top, and known dropped surfaces use
            // the targeted-injection path above.

            // bitmap pixels -> screen (DWM-frame origin + inset). Use
            // post_click_screen so we don't double-ClientToScreen.
            let result = tokio::task::spawn_blocking({
                let admitted = admitted.clone();
                move || {
                    let _admission = &admitted;
                    crate::input::post_click_screen(hwnd, sx_i, sy_i, count, &btn)
                }
            })
            .await;
            match result {
                Ok(Ok(())) => {
                    // Match Swift text format: "✅ Posted click/double-click/triple-click to pid X."
                    let click_word = match count {
                        2 => "double-click",
                        3 => "triple-click",
                        _ => "click",
                    };
                    posted_pixel_click_result(pid, click_word)
                }
                Ok(Err(e)) => ToolResult::error(e.to_string()),
                Err(e) => ToolResult::error(format!("Task error: {e}")),
            }
        } else {
            // Match Swift's error wording for the missing-target case.
            ToolResult::error("Provide element_token or (x, y) to address the click target.")
        }
    }
}

#[cfg(test)]
mod pixel_click_transport_tests {
    use super::{finish_pixel_uia_attempt, posted_pixel_click_result};
    use crate::uia::windows_enum::PointInvokeOutcome;
    use cua_driver_core::action_record::{
        ActionExecutionRecord, ActionTransport, ActualDelivery, RequestedDelivery,
    };

    #[test]
    fn no_answer_refuses_without_fallback_or_uncertain_effect() {
        let result = finish_pixel_uia_attempt(PointInvokeOutcome::NotResponding, 7, 3, 4)
            .expect("precheck must stop the route");
        assert!(result.is_error.unwrap_or(false));
        let data = result.structured_content.expect("structured refusal");
        assert_eq!(data["effect"], "refused");
        assert_eq!(data["nothing_sent"], true);
        assert_eq!(data["uia_status"], "not_responding");
    }

    #[test]
    fn gone_window_refuses_without_claiming_an_unknown_effect() {
        let result = finish_pixel_uia_attempt(PointInvokeOutcome::WindowGone, 7, 3, 4)
            .expect("a gone window must stop the route");
        assert!(result.is_error.unwrap_or(false));
        let data = result.structured_content.expect("structured refusal");
        assert_eq!(data["effect"], "refused");
        assert_eq!(data["nothing_sent"], true);
        assert_eq!(data["uia_status"], "window_gone");
    }

    #[test]
    fn point_failures_finalize_with_truthful_delivery_and_no_replay() {
        use cua_driver_contract::ErrorDisposition;
        for (outcome, status, effect, disposition) in [
            (
                PointInvokeOutcome::Busy,
                "busy",
                "refused",
                ErrorDisposition::NOT_DELIVERED,
            ),
            (
                PointInvokeOutcome::Unavailable,
                "unavailable",
                "refused",
                ErrorDisposition::NOT_DELIVERED,
            ),
            (
                PointInvokeOutcome::Timeout,
                "timeout",
                "unverifiable",
                ErrorDisposition::SUSPECTED_NOOP,
            ),
            (
                PointInvokeOutcome::SentFailure,
                "sent_failure",
                "unverifiable",
                ErrorDisposition::SUSPECTED_NOOP,
            ),
        ] {
            let result =
                finish_pixel_uia_attempt(outcome, 7, 3, 4).expect("failure must not fall through");
            let wire = cua_driver_core::mcp_result::conforming_tool_result(
                "click",
                serde_json::to_value(result).unwrap(),
            );
            let data = &wire["structuredContent"];
            assert_eq!(data["uia_status"], status);
            assert_eq!(data["effect"], effect);
            assert_eq!(
                serde_json::from_value::<ErrorDisposition>(data["disposition"].clone()).unwrap(),
                disposition
            );
        }
        assert!(finish_pixel_uia_attempt(PointInvokeOutcome::Miss, 7, 3, 4).is_none());
    }

    #[test]
    fn post_message_pixel_click_reports_synthetic_transport() {
        let result = posted_pixel_click_result(42, "click");
        let structured = result
            .structured_content
            .as_ref()
            .expect("posted click should expose legacy transport metadata");
        assert_eq!(structured["path"], "post_message");

        let record = ActionExecutionRecord::from_legacy(
            "click",
            &serde_json::json!({ "delivery_mode": "background" }),
            structured,
        )
        .expect("posted click should normalize into the public action contract");
        assert_eq!(record.transport, ActionTransport::WindowsPostMessage);
        assert_eq!(record.requested_delivery, RequestedDelivery::Background);
        assert_eq!(record.actual_delivery, Some(ActualDelivery::Background));

        let public = serde_json::to_value(record.public_result().expect("valid ActionResult"))
            .expect("serialize ActionResult");
        assert_eq!(public["route"], "synthetic_events");
        assert_eq!(public["delivery"]["mode"], "background");
    }

    #[test]
    fn sent_point_failure_is_uncertain_and_never_replays() {
        let result = finish_pixel_uia_attempt(PointInvokeOutcome::SentFailure, 7, 3, 4)
            .expect("sent failure is terminal");
        assert!(result.is_error.unwrap_or(false));
        assert_eq!(
            result.disposition,
            Some(cua_driver_contract::ErrorDisposition::SUSPECTED_NOOP)
        );
        let data = result.structured_content.unwrap();
        assert_eq!(data["effect"], "unverifiable");
        assert!(data.get("nothing_sent").is_none());
    }
}

#[cfg(test)]
mod background_element_click_record_tests {
    use super::{background_element_click_result, refused_element_click_result, ActionTransport};

    #[test]
    fn raw_fallback_reports_its_physical_transport() {
        for (transport, path) in [
            (ActionTransport::WindowsTargetedInjection, "pixel"),
            (ActionTransport::WindowsPostMessage, "post_message"),
        ] {
            let result = background_element_click_result(
                "physical fallback".into(),
                transport,
                vec![ActionTransport::WindowsUiaInvoke],
            );
            assert_eq!(result.structured_content.as_ref().unwrap()["path"], path);
            let record = result.action_record.unwrap();
            assert_eq!(record.transport, transport);
            let truth = record.debug_json();
            assert_ne!(truth["route"], "accessibility");
            assert_eq!(
                record.attempts[0].transport,
                ActionTransport::WindowsUiaInvoke
            );
            assert_eq!(record.fallbacks[0].to, transport);
        }
    }

    #[test]
    fn failed_provider_calls_disqualify_single_action_marker_exemption() {
        let result = background_element_click_result(
            "semantic success after provider failure".into(),
            ActionTransport::WindowsUiaToggle,
            vec![ActionTransport::WindowsUiaInvoke],
        );
        let record = result.action_record.unwrap();
        assert_eq!(record.transport, ActionTransport::WindowsUiaToggle);
        assert_eq!(record.attempts.len(), 1);
        assert_eq!(record.fallbacks.len(), 1);
        assert_eq!(record.fallbacks[0].from, ActionTransport::WindowsUiaInvoke);
        assert_eq!(record.fallbacks[0].to, ActionTransport::WindowsUiaToggle);
        // Point-free recording requires an empty fallback journal.
        assert!(!record.debug_json()["fallbacks"]
            .as_array()
            .unwrap()
            .is_empty());
    }

    #[test]
    fn single_provider_success_has_no_invented_attempts() {
        for transport in [
            ActionTransport::WindowsUiaInvoke,
            ActionTransport::WindowsUiaToggle,
            ActionTransport::WindowsUiaSelection,
            ActionTransport::WindowsUiaExpandCollapse,
        ] {
            let result =
                background_element_click_result("semantic success".into(), transport, vec![]);
            assert_eq!(result.structured_content.as_ref().unwrap()["path"], "ax");
            let record = result.action_record.unwrap();
            assert_eq!(record.transport, transport);
            assert!(record.attempts.is_empty());
            assert!(record.fallbacks.is_empty());
        }
    }

    #[test]
    fn write_failure_records_the_failing_value_transport_and_request() {
        use crate::tools::set_value::set_value_write_failure_result;
        use crate::uia_mutation::{RefusalKind, SetValueDecision, UncertainKind};
        use cua_driver_core::action_record::{ActionEffect, RequestedDelivery};
        let cases = [
            (
                SetValueDecision::Refused {
                    rung: 0,
                    kind: RefusalKind::PermissionDenied,
                    hresult: 0x8007_0005_u32 as i32,
                },
                ActionTransport::WindowsUiaValue,
                ActionEffect::Unverifiable,
            ),
            (
                SetValueDecision::Uncertain {
                    rung: 1,
                    kind: UncertainKind::Timeout,
                    hresult: 0x8013_1505_u32 as i32,
                },
                ActionTransport::WindowsUiaRangeValue,
                ActionEffect::Unverifiable,
            ),
        ];
        for (decision, transport, effect) in cases {
            let result = set_value_write_failure_result(
                decision,
                &serde_json::json!({"delivery_mode": "foreground"}),
            );
            let record = result.action_record.unwrap();
            assert_eq!(record.effect, effect);
            assert!(record.validate().is_ok());
            assert_eq!(record.transport, transport);
            assert_eq!(record.requested_delivery, RequestedDelivery::Foreground);
        }
    }

    #[test]
    fn refused_click_records_the_refusing_rung_and_delivers_nothing() {
        use cua_driver_core::action_record::ActionEffect;
        let result = refused_element_click_result(
            "UIA Toggle on [3]: refused".into(),
            ActionTransport::WindowsUiaToggle,
            vec![],
            crate::uia_mutation::RefusalKind::PermissionDenied,
        );
        assert!(result.is_error.unwrap_or(false));
        let record = result.action_record.unwrap();
        assert_eq!(record.effect, ActionEffect::Unverifiable);
        assert_eq!(record.transport, ActionTransport::WindowsUiaToggle);
        assert!(record.actual_delivery.is_none());
        assert!(record.attempts.is_empty());
        assert!(record.validate().is_ok());
    }

    #[test]
    fn sent_click_failure_records_uncertain_delivery_without_fallback() {
        use cua_driver_core::action_record::{ActionEffect, ActualDelivery};
        let result = super::uncertain_element_click_result(
            "write may have landed".into(),
            ActionTransport::WindowsUiaInvoke,
            vec![],
        );
        assert_eq!(
            result.disposition,
            Some(cua_driver_contract::ErrorDisposition::SUSPECTED_NOOP)
        );
        let record = result.action_record.unwrap();
        assert_eq!(record.effect, ActionEffect::Unverifiable);
        assert_eq!(record.actual_delivery, Some(ActualDelivery::Unknown));
        assert_eq!(record.attempts.len(), 1);
        assert!(record.fallbacks.is_empty());
        assert!(record.validate().is_ok());
    }
}

#[cfg(test)]
mod click_capture_id_schema_tests {
    use super::ClickTool;
    use cua_driver_core::tool::Tool;

    /// The portable click contract pins `button` and the other accepted
    /// fields. A live schema may broaden `capture_id`, which the subset gate
    /// allows, so the non-empty bound is pinned here.
    #[test]
    fn schema_requires_non_empty_capture_id() {
        let tool = ClickTool {
            state: super::ToolState::new(None),
        };
        let d = tool.def();
        let props = d.input_schema.get("properties").expect("properties");
        let capture_id = props.get("capture_id").expect("capture_id field present");
        assert_eq!(capture_id["type"], "string");
        assert_eq!(capture_id["minLength"], 1);
    }
}

#[cfg(test)]
mod w2b_action_record_tests {
    use super::*;
    use crate::tools::invoke_menu::{menu_refusal, menu_uncertain};
    use cua_driver_core::action_record::{ActionEffect, ActualDelivery, RequestedDelivery};

    #[test]
    fn w2b_refusal_and_uncertain_records_validate() {
        let menu = menu_refusal("not found".into());
        for result in [
            menu,
            mutation_refusal(
                "refused".into(),
                ActionTransport::WindowsUiaExpandCollapse,
                RequestedDelivery::Background,
                false,
                cua_driver_core::mutation::Reason::StaleElement,
            ),
            mutation_refusal(
                "refused".into(),
                ActionTransport::WindowsUiaScroll,
                RequestedDelivery::Foreground,
                false,
                cua_driver_core::mutation::Reason::PermissionDenied,
            ),
        ] {
            let record = result.action_record.unwrap();
            assert_eq!(record.effect, ActionEffect::Refused);
            assert_eq!(record.actual_delivery, None);
            assert!(record.evidence.is_empty());
            assert!(record.validate().is_ok());
        }
        // A pre-send stop after the no-activate guard ran is not a safe refusal.
        let after_prologue = mutation_refusal(
            "refused after guard".into(),
            ActionTransport::WindowsUiaScroll,
            RequestedDelivery::Background,
            true,
            cua_driver_core::mutation::Reason::StaleElement,
        );
        let record = after_prologue.action_record.unwrap();
        assert_eq!(record.effect, ActionEffect::Unverifiable);
        assert!(record.validate().is_ok());
        for result in [
            menu_uncertain("unknown".into(), ActionTransport::WindowsUiaInvoke),
            mutation_uncertain(
                "unknown".into(),
                "uia_expand_collapse",
                ActionTransport::WindowsUiaExpandCollapse,
                RequestedDelivery::Background,
                ActualDelivery::Background,
            ),
            mutation_uncertain(
                "unknown".into(),
                "uia",
                ActionTransport::WindowsUiaScroll,
                RequestedDelivery::Foreground,
                ActualDelivery::Foreground,
            ),
        ] {
            let record = result.action_record.unwrap();
            assert_eq!(record.effect, ActionEffect::Unverifiable);
            assert!(record.actual_delivery.is_some());
            assert!(record.validate().is_ok());
        }
    }
}

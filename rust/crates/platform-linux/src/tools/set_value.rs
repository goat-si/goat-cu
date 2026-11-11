// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Cua AI, Inc.

use async_trait::async_trait;
use cua_driver_core::{
    protocol::ToolResult,
    tool::{Tool, ToolDef},
};
use serde_json::{json, Value};
use std::sync::Arc;

use crate::atspi::native::{OptionPick, OptionPickSent};

use super::impl_::{
    attach_focus_guard, background_virtual_key, background_virtual_text, foreground_budget,
    input_error_result, non_ax_escalation, position_named_session_keyboard_cursor,
    prove_token_element, resolve_cursor_key, resolve_element_local_coords,
    spawn_text_write_bounded, stale_observed_element, text_write_stop_result, values_agree,
    window_local_to_screen, x11_pixel_click_no_focus_steal, ToolState,
};

// ── set_value ─────────────────────────────────────────────────────────────────

pub struct SetValueTool {
    pub(crate) state: Arc<ToolState>,
    #[cfg(test)]
    pub(crate) test_hooks: Option<Arc<SetValueTestHooks>>,
}

#[cfg(test)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SetValueTestOutcome {
    Uncertain,
    Partial,
    UncertainAfterTimeout,
    NoRoute,
}

#[cfg(test)]
pub(crate) struct SetValueTestHooks {
    pub(crate) ax: SetValueTestOutcome,
    pub(crate) keyboard_fallbacks: std::sync::atomic::AtomicUsize,
}

impl SetValueTool {
    /// The focus-free accessibility write on the token's object. Every write
    /// it makes (EditableText, Value, a selection pick) shares one dispatch
    /// gate, closed if the call stops waiting.
    async fn ax_write(
        &self,
        pid: u32,
        target: crate::atspi::TokenElement,
        value: String,
    ) -> Result<anyhow::Result<()>, tokio::task::JoinError> {
        #[cfg(test)]
        if let Some(hooks) = &self.test_hooks {
            use crate::text_write_policy::{Step, WriteStep};
            let step = WriteStep {
                step: Step::SetText,
                bytes: value.len(),
                units: None,
            };
            return Ok(Err(match hooks.ax {
                SetValueTestOutcome::Uncertain => {
                    crate::atspi::native::WriteUncertain { step }.into()
                }
                SetValueTestOutcome::Partial => crate::atspi::native::WritePartial { step }.into(),
                SetValueTestOutcome::UncertainAfterTimeout => {
                    anyhow::Error::from(crate::atspi::native::WriteUncertain { step })
                        .context("set_value did not complete within 12s")
                }
                SetValueTestOutcome::NoRoute => {
                    anyhow::anyhow!("{}: test", crate::atspi::native::NO_VALUE_ROUTE)
                }
            }));
        }
        let dispatch = Arc::new(crate::atspi::native::TextWriteDispatch::default());
        let task_dispatch = dispatch.clone();
        let bytes = value.len();
        spawn_text_write_bounded(
            "set_value",
            foreground_budget(0),
            &[dispatch],
            bytes,
            move || {
                crate::atspi::set_value_token_with_dispatch(pid, &target, &value, task_dispatch)
            },
        )
        .await
    }

    /// Test seam: counts entries into the keyboard fallback and stops there.
    fn keyboard_fallback_entered(&self) -> Option<ToolResult> {
        #[cfg(test)]
        if let Some(hooks) = &self.test_hooks {
            hooks
                .keyboard_fallbacks
                .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            return Some(
                ToolResult::text("test fallthrough")
                    .with_structured(json!({"test_fallthrough": true})),
            );
        }
        None
    }

    fn positions_cursor(&self) -> bool {
        #[cfg(test)]
        if self.test_hooks.is_some() {
            return false;
        }
        true
    }
}
static SV_DEF: std::sync::OnceLock<ToolDef> = std::sync::OnceLock::new();

#[cfg(test)]
#[path = "impl_/text_write_stop_tests.rs"]
mod text_write_stop_tests;

#[async_trait]
impl Tool for SetValueTool {
    fn def(&self) -> &ToolDef {
        SV_DEF.get_or_init(|| ToolDef {
            name: "set_value".into(),
            description: "Set value of an AT-SPI element via SetValue action.".into(),
            input_schema: json!({
                "type":"object","required":["value"],"properties":{
                    "session": cua_driver_core::tool_schema::session_schema(),
                    "pid":{"type":"integer","description":"Target process ID. Optional with element_token (the token carries the pid)."},
                    "window_id":{"type":"integer","description":"Omit when element_token is supplied (the token carries it)."},
                    "element_token": cua_driver_core::tool_schema::element_token_schema(),
                    "value":{"type":"string","description":"New value. Written through AT-SPI EditableText/Value when the element exposes them; otherwise (GTK2 spin scales, VCL spin buttons) the field is clicked with the session's real pointer, its text selected and replaced through the virtual keyboard and committed with Tab, then read back."},
                    "delivery_mode": crate::input::delivery::delivery_mode_schema()
                },"additionalProperties":false
            }),
            read_only: false, destructive: true, idempotent: false, open_world: true,
        })
    }

    async fn invoke(&self, args: Value) -> ToolResult {
        use cua_driver_core::tool_args::ArgsExt;
        let pid = match args.require_u32("pid") {
            Ok(v) => v,
            Err(e) => return e,
        };
        let value = match args.get("value") {
            Some(Value::String(text)) => text.clone(),
            Some(Value::Number(number)) => number.to_string(),
            _ => {
                return ToolResult::error("set_value: 'value' must be a string or a number.")
                    .with_structured(json!({ "code": "invalid_arguments" }))
            }
        };
        let resolved = match self.state.snapshots.resolve(pid as i32, &args) {
            Ok(r) => r,
            Err(e) => return e,
        };
        let token = match prove_token_element(pid, &resolved, "set_value").await {
            Ok(token) => token,
            Err(refusal) => return refusal,
        };
        let (token, xid) = match (token, &resolved) {
            (
                Some(token),
                cua_driver_core::element_token::ResolvedElement::Element { window_id, .. },
            ) => (token, *window_id),
            _ => {
                return ToolResult::error(
                    "set_value requires element_token to address the target element.",
                )
            }
        };
        let idx = token.index;
        // A password field's value never goes back into the reply.
        let secret = {
            let target = token.clone();
            tokio::task::spawn_blocking(move || crate::atspi::is_password_token(&target))
                .await
                .unwrap_or(true)
        };
        let value_for_task = value.clone();
        let xid_opt = Some(xid);
        let delivery = crate::input::delivery::DeliveryMode::from_args(&args);
        let cursor_id = resolve_cursor_key(&args);
        if self.positions_cursor() {
            position_named_session_keyboard_cursor(
                &self.state,
                &args,
                pid,
                xid,
                Some(token.clone()),
                None,
                true,
            )
            .await;
        }
        // 1. Focus-free accessibility write on the exact snapshot object.
        let ax = self.ax_write(pid, token.clone(), value_for_task).await;
        let ax_error = match ax {
            Ok(Ok(())) => {
                let target = token.clone();
                let readback =
                    tokio::task::spawn_blocking(move || crate::atspi::read_value_token(&target))
                        .await
                        .ok()
                        .flatten();
                return set_value_result(idx, &value, "ax", readback, None, secret);
            }
            // A selection pick was sent and not answered: read the option
            // back. Nothing after it may write again.
            Ok(Err(error)) if error.is::<OptionPickSent>() => {
                let readback = read_back(token).await;
                return pick_sent_result(idx, &value, "ax", readback, &error, secret);
            }
            // The write may have run, or the field is partly changed: no
            // further route (keys, selection, pointer) may write again.
            Ok(Err(error)) if crate::atspi::native::text_write_stop(&error).is_some() => {
                return text_write_stop_result("set_value", Some(idx), &error)
                    .unwrap_or_else(|| input_error_result(error))
            }
            Ok(Err(error)) if crate::atspi::native::is_no_value_route(&error) => error,
            Ok(Err(error)) if crate::atspi::native::is_no_such_option(&error) => {
                return no_such_option_result(idx, &error, secret)
            }
            Ok(Err(error)) => {
                return stale_observed_element("set_value", idx, &error)
                    .unwrap_or_else(|| input_error_result(error))
            }
            Err(error) => return ToolResult::error(format!("Task error: {error}")),
        };
        if let Some(entered) = self.keyboard_fallback_entered() {
            return entered;
        }
        // 2. No accessibility write route. A combo box is opened with a click
        //    and the option picked from its popup (whose items only exist and
        //    act while it is open), then the popup is closed. Anything else is
        //    typed like a user would: click the field (real pointer), select
        //    all, type, commit with Tab.
        let is_combo = token.observed.role.eq_ignore_ascii_case("combo box");
        let wayland = crate::wayland::is_wayland();
        let background_route = !delivery.is_foreground()
            && !wayland
            && crate::input::real_pointer_input_available()
            && crate::input::real_keyboard_input_available();
        let foreground_route = delivery.is_foreground() && !wayland;
        if !background_route && !foreground_route {
            let detail = format!(
                "{ax_error}; the keyboard fallback needs {}",
                if wayland {
                    "an X11 session"
                } else {
                    "the focus-free real-input route (a writable /dev/uinput), or delivery_mode:\"foreground\""
                }
            );
            let mut structured = json!({
                "code": "set_value_unavailable",
                "detail": detail,
                "effect": "none",
            });
            // Only X11 can run the foreground keyboard fallback.
            if !wayland {
                structured["escalation"] = json!({ "recommended": "foreground", "reason": "click the field and type the value with delivery_mode:\"foreground\"" });
            }
            return ToolResult::error(format!(
                "set_value: element [{idx}] has no accessibility write route and no keyboard \
                 fallback is available: {detail}."
            ))
            .with_structured(structured);
        }
        let value_for_keys = value.clone();
        let target = token.clone();
        // The click/select/type/commit sequence runs behind a gate the caller
        // closes on timeout: a late fallback never types after the call
        // returned, and a timeout mid-sequence is unverifiable.
        let key_dispatch = Arc::new(crate::atspi::native::TextWriteDispatch::default());
        let key_gate = key_dispatch.clone();
        let typed = spawn_text_write_bounded(
            "set_value keyboard fallback",
            foreground_budget(value.len()),
            &[key_dispatch],
            value.len(),
            move || -> anyhow::Result<Option<crate::input::FocusGuardReport>> {
                let (win, lx, ly) = resolve_element_local_coords(pid, &target, xid_opt)?;
                if is_combo {
                    return pick_from_popup(
                        &cursor_id,
                        pid,
                        &target,
                        (win, lx, ly),
                        &value_for_keys,
                        foreground_route,
                        &key_gate,
                    );
                }
                if foreground_route {
                    let (sx, sy) = window_local_to_screen(win, lx, ly)?;
                    crate::interaction::with_x11_foreground(
                        win,
                        crate::input::ForegroundOptions::pointer(),
                        crate::interaction::session_policy().menu,
                        || {
                            gated_steps(
                                &key_gate,
                                vec![
                                    Box::new(|| {
                                        crate::input::send_click_xtest_desktop(
                                            sx.round() as i32,
                                            sy.round() as i32,
                                            1,
                                            1,
                                        )?;
                                        std::thread::sleep(std::time::Duration::from_millis(80));
                                        Ok(())
                                    }),
                                    Box::new(|| crate::input::send_key_xtest("a", &["ctrl"])),
                                    Box::new(|| {
                                        crate::input::send_type_text_xtest(&value_for_keys)
                                    }),
                                    Box::new(|| crate::input::send_key_xtest("Tab", &[])),
                                ],
                            )
                        },
                    )?;
                    return Ok(None);
                }
                let (_, guard) = crate::input::focus_guard::guarded(Some(pid), || {
                    gated_steps(
                        &key_gate,
                        vec![
                            Box::new(|| {
                                x11_pixel_click_no_focus_steal(
                                    &cursor_id, win, lx as i32, ly as i32, 1, 1,
                                )?;
                                std::thread::sleep(std::time::Duration::from_millis(80));
                                Ok(())
                            }),
                            Box::new(|| {
                                background_virtual_key(&cursor_id, pid, win, "a", &["ctrl"])
                                    .map(|_| ())
                            }),
                            Box::new(|| {
                                background_virtual_text(
                                    &cursor_id,
                                    pid,
                                    win,
                                    &value_for_keys,
                                    crate::input::TypePacing::default(),
                                )
                                .map(|_| ())
                            }),
                            Box::new(|| {
                                background_virtual_key(&cursor_id, pid, win, "Tab", &[]).map(|_| ())
                            }),
                        ],
                    )
                })?;
                Ok(guard)
            },
        )
        .await;
        match typed {
            Ok(Ok(guard)) => {
                // Let the toolkit commit on focus-out before reading back.
                tokio::time::sleep(std::time::Duration::from_millis(150)).await;
                let readback =
                    tokio::task::spawn_blocking(move || crate::atspi::read_value_token(&token))
                        .await
                        .ok()
                        .flatten();
                set_value_result(
                    idx,
                    &value,
                    match (is_combo, foreground_route) {
                        (true, true) => "click_select_fg",
                        (true, false) => "click_select_mpx",
                        (false, true) => "click_type_fg",
                        (false, false) => "click_type_mpx",
                    },
                    readback,
                    guard.as_ref(),
                    secret,
                )
            }
            Ok(Err(error)) if crate::atspi::native::is_no_such_option(&error) => {
                no_such_option_result(idx, &error, secret)
            }
            Ok(Err(error)) if error.is::<OptionPickSent>() => {
                let path = if foreground_route {
                    "click_select_fg"
                } else {
                    "click_select_mpx"
                };
                let readback = read_back(token).await;
                pick_sent_result(idx, &value, path, readback, &error, secret)
            }
            Ok(Err(error)) => text_write_stop_result("set_value", Some(idx), &error)
                .unwrap_or_else(|| input_error_result(error)),
            Err(error) => ToolResult::error(format!("Task error: {error}")),
        }
    }
}

/// `set_value` named an option the combo box or list does not have.
fn no_such_option_result(idx: usize, error: &anyhow::Error, secret: bool) -> ToolResult {
    // The detail echoes the requested value and the option list: withhold it
    // for a secret field.
    let detail = if secret {
        "no option matching the requested value (withheld: secret field)".to_owned()
    } else {
        let detail = error.to_string();
        detail
            .strip_prefix(crate::atspi::native::NO_SUCH_OPTION)
            .map(|rest| rest.trim_start_matches(':').trim())
            .unwrap_or(&detail)
            .to_owned()
    };
    ToolResult::error(format!(
        "set_value: element [{idx}] has {detail}; nothing was changed."
    ))
    .with_structured(json!({ "code": "option_not_found", "detail": detail, "effect": "none" }))
}

/// How long `set_value` waits for an opened combo box popup to list its
/// options.
const POPUP_OPTIONS_WAIT: std::time::Duration = std::time::Duration::from_millis(1500);

/// Keyboard-fallback `set_value` on a combo box whose options cannot be
/// selected while it is closed: click it open (the real pointer: XTest in
/// the foreground, the session's MPX pointer in the background), activate
/// the option `value` names from the popup, and press Escape if the popup
/// is still open afterwards (an unknown option, or a toolkit that keeps it
/// open), so no popup is left behind.
fn pick_from_popup(
    cursor_id: &str,
    pid: u32,
    token: &crate::atspi::TokenElement,
    (win, lx, ly): (u64, f64, f64),
    value: &str,
    foreground: bool,
    gate: &crate::atspi::native::TextWriteDispatch,
) -> anyhow::Result<Option<crate::input::FocusGuardReport>> {
    use crate::atspi::native::gated_route;
    // Reads (popup state, coordinates) stay outside the gate; every write
    // (open click, option activation, Escape) checks it, so a timeout stops
    // the next one.
    let close_popup = || -> anyhow::Result<()> {
        if !crate::atspi::options_showing_token(token) {
            return Ok(());
        }
        if foreground {
            // Inside the click's transaction: the target still has the focus.
            // The gate still covers it, so a closed gate sends nothing.
            gated_route(gate, || crate::input::send_key_xtest("Escape", &[]))
        } else {
            gated_route(gate, || {
                background_virtual_key(cursor_id, pid, win, "Escape", &[]).map(|_| ())
            })
        }
    };
    let pick = || {
        // The native call enters the gate only right before DoAction.
        pick_listed_option(value, POPUP_OPTIONS_WAIT, || {
            crate::atspi::activate_option_token_gated(token, value, gate)
        })
    };
    let open_and_pick = || -> anyhow::Result<()> {
        // A pick that may have changed the control ends here: no Escape
        // follows it.
        crate::token_action_policy::pick_then_close(
            pick,
            // Let the toolkit close the popup on activation before checking.
            || std::thread::sleep(std::time::Duration::from_millis(150)),
            close_popup,
            crate::atspi::native::text_write_stop,
        )
        .map(|_| ())
    };
    if foreground {
        // Open, pick and close in one transaction, so the restore runs once
        // the popup is gone instead of closing it before the pick.
        let (sx, sy) = window_local_to_screen(win, lx, ly)?;
        crate::interaction::with_x11_foreground(
            win,
            crate::input::ForegroundOptions::pointer(),
            crate::interaction::session_policy().menu,
            || {
                gated_route(gate, || {
                    crate::input::send_click_xtest_desktop(
                        sx.round() as i32,
                        sy.round() as i32,
                        1,
                        1,
                    )
                })?;
                open_and_pick()
            },
        )?;
        return Ok(None);
    }
    let (_, guard) = crate::input::focus_guard::guarded(Some(pid), || {
        gated_route(gate, || {
            x11_pixel_click_no_focus_steal(cursor_id, win, lx as i32, ly as i32, 1, 1)
        })?;
        open_and_pick()
    })?;
    Ok(guard)
}

/// Run a sequence of writes (click, select-all, type, commit) with the gate
/// checked before each one: once the caller timed out and closed it, no
/// further step runs. A step already in flight still finishes; the caller
/// reported that as unverifiable.
fn gated_steps<'a>(
    gate: &crate::atspi::native::TextWriteDispatch,
    steps: Vec<Box<dyn FnOnce() -> anyhow::Result<()> + 'a>>,
) -> anyhow::Result<()> {
    for step in steps {
        crate::atspi::native::gated_route(gate, step)?;
    }
    Ok(())
}

/// Retry `activate` until the opened popup answers with its options or
/// `wait` runs out, then pick the option `value` names. A popup whose every
/// read answered and that still lists no options has no option `value`
/// names: the same `no_such_option` refusal as an unknown option. One whose
/// options could not be read stays an error, since a populated popup can be
/// unreadable too.
fn pick_listed_option(
    value: &str,
    wait: std::time::Duration,
    mut activate: impl FnMut() -> anyhow::Result<OptionPick>,
) -> anyhow::Result<String> {
    let deadline = std::time::Instant::now() + wait;
    loop {
        let pick = activate()?;
        let waiting = std::time::Instant::now() < deadline;
        match pick {
            OptionPick::Picked { name } => return Ok(name),
            OptionPick::NoMatch { options } => {
                return Err(crate::atspi::no_such_option(value, &options))
            }
            OptionPick::NoOptions | OptionPick::Unavailable if waiting => {
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
            OptionPick::NoOptions => return Err(crate::atspi::no_such_option(value, &[])),
            OptionPick::Unavailable => {
                anyhow::bail!("the combo box popup's options could not be read")
            }
        }
    }
}
async fn read_back(token: crate::atspi::TokenElement) -> Option<String> {
    tokio::task::spawn_blocking(move || crate::atspi::read_value_token(&token))
        .await
        .ok()
        .flatten()
}

/// `set_value` result for a combo box pick that was sent but not answered
/// ([`OptionPickSent`]): the option may be set, so it is reported like a
/// write with its readback, never as a failure that invites picking again.
/// Without a matching readback the effect is unknown, not a no-op, and no
/// escalation is offered; the code then names the unacknowledged write
/// (goat: as every other stopped write does).
fn pick_sent_result(
    idx: usize,
    value: &str,
    path: &str,
    readback: Option<String>,
    error: &anyhow::Error,
    secret: bool,
) -> ToolResult {
    let mut result = set_value_result(idx, value, path, readback, None, secret);
    // The error names the option: withhold it for a secret field.
    let note = if secret {
        "the option pick was sent but not acknowledged; it may have taken effect".to_owned()
    } else {
        format!("the option pick was sent, but {error}; it may have taken effect")
    };
    if let Some(structured) = result.structured_content.as_mut() {
        if structured["verified"] != true {
            structured["effect"] = json!("unverifiable");
            structured["code"] = json!("element_write_unacknowledged");
            if let Some(fields) = structured.as_object_mut() {
                fields.remove("escalation");
            }
        }
        structured["note"] = json!(note);
    }
    if let Some(cua_driver_core::protocol::Content::Text { text, .. }) = result.content.first_mut()
    {
        text.push_str(&format!(
            " Note: {note}. Read it back before picking again."
        ));
    }
    result
}

/// `set_value` result with an honest verification: `verified` only when the
/// element read back the requested value. For a password field (`secret`) the
/// reply names the value `[REDACTED]`.
fn set_value_result(
    idx: usize,
    value: &str,
    path: &str,
    readback: Option<String>,
    guard: Option<&crate::input::FocusGuardReport>,
    secret: bool,
) -> ToolResult {
    let normalised_path = readback
        .as_deref()
        .is_some_and(|seen| path_normalised_to_basename(value, seen));
    let verified = readback
        .as_deref()
        .is_some_and(|seen| values_agree(value, seen))
        || normalised_path;
    let mut structured = json!({
        "path": path,
        "verified": verified,
        "effect": if verified { "confirmed" } else if readback.is_some() { "suspected_noop" } else { "unverifiable" },
    });
    let shown = crate::atspi_text::echoed_value(value, secret);
    let mut text = format!("Set value of element [{idx}] to '{shown}' (path={path})");
    // Verified above against the real read-back; only what is shown is masked.
    let readback = readback.map(|seen| crate::atspi_text::echoed_value(&seen, secret).to_owned());
    match readback {
        Some(seen) if normalised_path => {
            structured["readback"] = json!(seen);
            text.push_str(&format!(
                "; the field reads back '{seen}': a file chooser navigated to the path's \
                 directory and kept the file name (the write committed)."
            ));
        }
        Some(seen) if verified => text.push_str(&format!("; read back '{seen}'.")),
        Some(seen) => {
            structured["readback"] = json!(seen);
            structured["escalation"] = non_ax_escalation();
            text.push_str(&format!(
                "; but the element reads back '{seen}' — the write may not have committed. \
                 Confirm with get_window_state or a screenshot."
            ));
        }
        None => {
            structured["readback"] = Value::Null;
            text.push_str("; not verified — confirm with get_window_state.");
        }
    }
    attach_focus_guard(ToolResult::text(text).with_structured(structured), guard)
}

/// A Qt/GTK file chooser given an absolute path in its "File name" field
/// moves to that directory and shows only the final component: the readback
/// is the basename, and the write committed.
fn path_normalised_to_basename(value: &str, seen: &str) -> bool {
    let value = value.trim();
    let seen = seen.trim();
    value.contains('/')
        && !seen.is_empty()
        && value
            .rsplit('/')
            .next()
            .is_some_and(|base| !base.is_empty() && base == seen)
}

#[cfg(test)]
mod set_value_tests {
    use super::*;

    #[test]
    fn a_file_chooser_keeping_the_basename_counts_as_committed() {
        assert!(path_normalised_to_basename(
            "/home/user/Desktop/tone.wav",
            "tone.wav"
        ));
        assert!(!path_normalised_to_basename(
            "/home/user/Desktop/tone.wav",
            "other.wav"
        ));
        assert!(!path_normalised_to_basename("tone.wav", "tone.wav"));
        let ok = set_value_result(
            3,
            "/home/user/Desktop/tone.wav",
            "ax",
            Some("tone.wav".into()),
            None,
            false,
        );
        let s = ok.structured_content.unwrap();
        assert_eq!(s["verified"], true);
        assert_eq!(s["effect"], "confirmed");
    }

    #[test]
    fn readback_agreement_is_numeric_aware() {
        assert!(values_agree("40", "40.0"));
        assert!(values_agree("40", " 40 "));
        assert!(values_agree("abc", "abc"));
        assert!(!values_agree("40", "41"));
        assert!(!values_agree("40", "forty"));
    }

    #[test]
    fn set_value_result_is_verified_only_on_matching_readback() {
        let ok = set_value_result(3, "40", "click_type_mpx", Some("40.0".into()), None, false);
        let s = ok.structured_content.unwrap();
        assert_eq!(s["verified"], true);
        assert_eq!(s["effect"], "confirmed");
        let noop = set_value_result(3, "40", "ax", Some("10.0".into()), None, false);
        let s = noop.structured_content.unwrap();
        assert_eq!(s["verified"], false);
        assert_eq!(s["effect"], "suspected_noop");
        assert_eq!(s["readback"], "10.0");
        let blind = set_value_result(3, "40", "ax", None, None, false);
        assert_eq!(blind.structured_content.unwrap()["effect"], "unverifiable");
    }

    #[test]
    fn a_password_field_write_does_not_repeat_its_value() {
        let result = set_value_result(1, "set-secret-2", "ax", None, None, true);
        let text = format!("{:?}", result.content);
        assert!(!text.contains("set-secret-2"), "{text}");
        assert!(text.contains("[REDACTED]"), "{text}");
        let plain = set_value_result(1, "draft", "ax", None, None, false);
        assert!(format!("{:?}", plain.content).contains("draft"));
    }

    #[test]
    fn a_password_field_write_masks_its_read_back_too() {
        for readback in ["set-secret-2", "other-secret"] {
            let result =
                set_value_result(1, "set-secret-2", "ax", Some(readback.into()), None, true);
            let shown = format!("{:?} {:?}", result.content, result.structured_content);
            assert!(!shown.contains("secret"), "{shown}");
        }
        // Verification still uses the real read-back.
        let matched = set_value_result(
            1,
            "set-secret-2",
            "ax",
            Some("set-secret-2".into()),
            None,
            true,
        );
        assert_eq!(matched.structured_content.unwrap()["verified"], true);
    }

    #[test]
    fn a_combo_box_listing_no_options_refuses_the_value_and_changes_nothing() {
        let mut attempts = 0;
        let error = pick_listed_option("Blue", std::time::Duration::ZERO, || {
            attempts += 1;
            Ok(OptionPick::NoOptions)
        })
        .unwrap_err();
        assert_eq!(attempts, 1);
        assert!(crate::atspi::native::is_no_such_option(&error), "{error}");
        // goat: the refusal never echoes the requested value.
        assert!(!error.to_string().contains("Blue"), "{error}");
        let refused = no_such_option_result(3, &error, false);
        assert_eq!(refused.is_error, Some(true));
        let s = refused.structured_content.unwrap();
        assert_eq!(s["code"], "option_not_found");
        assert_eq!(s["effect"], "none");
        assert_eq!(
            s["detail"],
            "no option matching the requested value; it lists no options"
        );
    }

    #[test]
    fn an_unreadable_popup_is_never_reported_as_having_no_such_option() {
        let error = pick_listed_option("Blue", std::time::Duration::ZERO, || {
            Ok(OptionPick::Unavailable)
        })
        .unwrap_err();
        assert!(!crate::atspi::native::is_no_such_option(&error), "{error}");
    }

    #[test]
    fn a_popup_that_lists_its_options_late_still_gets_the_pick() {
        let mut attempts = 0;
        let picked = pick_listed_option("Blue", std::time::Duration::from_secs(5), || {
            attempts += 1;
            Ok(match attempts {
                1 => OptionPick::Unavailable,
                2 => OptionPick::NoOptions,
                _ => OptionPick::Picked {
                    name: "Blue".to_owned(),
                },
            })
        })
        .unwrap();
        assert_eq!(picked, "Blue");
        assert_eq!(attempts, 3);
    }

    #[test]
    fn a_sent_pick_is_never_reported_as_failed_or_a_noop() {
        let sent = anyhow::Error::new(OptionPickSent("the option's action did not answer".into()));
        // The combo box already shows the option: the pick landed.
        let landed = pick_sent_result(3, "Medium", "ax", Some("Medium".into()), &sent, false);
        assert_ne!(landed.is_error, Some(true));
        let s = landed.structured_content.unwrap();
        assert_eq!(s["verified"], true);
        assert_eq!(s["effect"], "confirmed");
        // It shows the old option, or nothing: unknown, with no escalation
        // that would send the pick again.
        for readback in [Some("Small".to_owned()), None] {
            let unknown = pick_sent_result(3, "Medium", "click_select_mpx", readback, &sent, false);
            assert_ne!(unknown.is_error, Some(true));
            let s = unknown.structured_content.unwrap();
            assert_eq!(s["verified"], false);
            assert_eq!(s["effect"], "unverifiable");
            assert_eq!(s["code"], "element_write_unacknowledged");
            assert!(s.get("escalation").is_none(), "{s}");
            assert!(s["note"]
                .as_str()
                .unwrap()
                .contains("may have taken effect"));
        }
        // A secret field's pick names neither the value nor the option.
        let secret = pick_sent_result(3, "hunter2", "ax", None, &sent, true);
        let shown = format!("{:?} {:?}", secret.content, secret.structured_content);
        assert!(!shown.contains("hunter2"), "{shown}");
    }
}

//! Interruptible physical key delivery and balanced transition cleanup.

use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;

use core_graphics::event::{CGEventFlags, CGEventTapLocation};
use core_graphics::event_source::{CGEventSource, CGEventSourceStateID};

use super::keyboard;

#[derive(Default)]
pub(crate) struct HoldControl {
    cancelled: Mutex<bool>,
    wake: Condvar,
}

impl HoldControl {
    pub(crate) fn cancel(&self) {
        *self.cancelled.lock().unwrap_or_else(|e| e.into_inner()) = true;
        self.wake.notify_all();
    }

    pub(crate) fn is_cancelled(&self) -> bool {
        *self.cancelled.lock().unwrap_or_else(|e| e.into_inner())
    }

    pub(crate) fn wait(&self, duration: Duration) -> bool {
        let state = self.cancelled.lock().unwrap_or_else(|e| e.into_inner());
        let (state, _) = self
            .wake
            .wait_timeout_while(state, duration, |cancelled| !*cancelled)
            .unwrap_or_else(|e| e.into_inner());
        *state
    }
}

pub(crate) struct CancelOnDrop(pub(crate) Arc<HoldControl>);
impl Drop for CancelOnDrop {
    fn drop(&mut self) {
        self.0.cancel();
    }
}

#[derive(Clone, Copy)]
pub(crate) enum KeyRoute {
    Pid(i32),
    /// The HID queue with flags. `Some(pid)` is the app the chord is aimed
    /// at; `None` (desktop scope) means whichever app is frontmost.
    Global(Option<i32>),
    /// Desktop HID: refuse each key-down if the admitted app is no longer frontmost.
    GuardedGlobal(i32, u64),
    /// The HID queue without flags, aimed at `pid` (the route Screen Sharing
    /// targets use).
    BareGlobal(i32),
}

pub(crate) fn deliver_menu(
    pid: i32,
    key: &str,
    modifiers: &[&str],
    duration: Option<Duration>,
    control: &HoldControl,
) -> keyboard::ChordPost {
    deliver_menu_with(
        keyboard::chord_events(pid, key, modifiers),
        |event| keyboard::post_chord_event_no_auth(pid, event),
        || control.is_cancelled(),
        || control.wait(duration.unwrap_or_else(keyboard::key_gap)),
    )
}

fn deliver_menu_with<E>(
    built: anyhow::Result<(E, E)>,
    mut post: impl FnMut(&E),
    cancelled: impl FnOnce() -> bool,
    wait: impl FnOnce() -> bool,
) -> keyboard::ChordPost {
    let (down, up) = match built {
        Ok(events) => events,
        Err(error) => return keyboard::ChordPost::NotPosted(error.to_string()),
    };
    if cancelled() {
        return keyboard::ChordPost::NotPosted("key delivery cancelled".into());
    }
    // A poster may panic after sending the down event, so arm cleanup first.
    let delivery = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        post(&down);
        wait()
    }));
    post(&up);
    let interrupted = match delivery {
        Ok(interrupted) => interrupted,
        Err(panic) => std::panic::resume_unwind(panic),
    };
    if interrupted {
        keyboard::ChordPost::Uncertain("key delivery cancelled after posting".into())
    } else {
        keyboard::ChordPost::Posted
    }
}

pub(crate) fn deliver(
    route: KeyRoute,
    key: &str,
    modifiers: &[&str],
    duration: Option<Duration>,
    control: &HoldControl,
) -> anyhow::Result<()> {
    let source = if matches!(route, KeyRoute::Global(_) | KeyRoute::GuardedGlobal(..)) {
        Some(
            CGEventSource::new(CGEventSourceStateID::HIDSystemState)
                .map_err(|_| anyhow::anyhow!("CGEventSource::new failed"))?,
        )
    } else {
        None
    };
    // Resolve the base key where the target's layout types it (upstream
    // #6659), Screen Sharing keeping US positions. A desktop-scope chord
    // reaches the frontmost app, which may itself be Screen Sharing.
    let target = layout_target(route, crate::apps::frontmost_pid);
    let mut guarded_down = std::collections::HashMap::new();
    deliver_with_pacing(
        key,
        modifiers,
        duration.unwrap_or_else(keyboard::key_gap),
        (
            matches!(
                route,
                KeyRoute::Global(_) | KeyRoute::GuardedGlobal(..) | KeyRoute::BareGlobal(_)
            ),
            matches!(route, KeyRoute::BareGlobal(_)),
        ),
        |key, modifiers| keyboard::target_key_code(target, key, modifiers),
        |code, down, flags| match route {
            KeyRoute::Pid(pid) => keyboard::post_key(pid, code, down, flags),
            KeyRoute::Global(_) => keyboard::post_global_key(
                source.as_ref().expect("global source"),
                code,
                down,
                flags,
            ),
            KeyRoute::GuardedGlobal(pid, start) => post_if_foreground(
                (pid, start),
                current_frontmost_instance,
                code,
                down,
                &mut guarded_down,
                || {
                    keyboard::post_global_key(
                        source.as_ref().expect("global source"),
                        code,
                        down,
                        flags,
                    )
                },
            ),
            KeyRoute::BareGlobal(_) => {
                let event = keyboard::create_bare_keyboard_event(code, down)?;
                event.set_flags(flags);
                post_hid(|| event.post(CGEventTapLocation::HID));
                Ok(())
            }
        },
        || control.is_cancelled(),
        |hold| control.wait(hold),
    )
}

fn instance_matches(expected: Option<u64>, actual: Option<u64>) -> bool {
    expected.is_some() && expected == actual
}

fn post_if_foreground(
    expected: (i32, u64),
    frontmost: impl FnOnce() -> Option<(i32, u64)>,
    code: u16,
    down: bool,
    posted_down: &mut std::collections::HashMap<u16, Vec<bool>>,
    post: impl FnOnce() -> anyhow::Result<()>,
) -> anyhow::Result<()> {
    if down {
        let current = frontmost();
        if current.map(|instance| instance.0) != Some(expected.0)
            || !instance_matches(Some(expected.1), current.map(|instance| instance.1))
        {
            posted_down.entry(code).or_default().push(false);
            anyhow::bail!("the frontmost application changed during the chord; modifier events may have been sent, do not resend");
        }
        // A failed post may still have sent the event, so arm cleanup first.
        posted_down.entry(code).or_default().push(true);
        return post();
    }
    let Some(was_posted) = posted_down.get_mut(&code).and_then(Vec::pop) else {
        return Ok(());
    };
    if !was_posted {
        return Ok(());
    }
    let result = post();
    if result.is_err() {
        posted_down.entry(code).or_default().push(true);
    }
    result
}

fn current_frontmost_instance() -> Option<(i32, u64)> {
    let pid = crate::apps::frontmost_pid()?;
    let start = crate::process_identity::process_fingerprint(i64::from(pid))?.start_time?;
    (crate::apps::frontmost_pid() == Some(pid)).then_some((pid, start))
}

/// The app whose keyboard layout resolves a chord's key on `route`.
fn layout_target(route: KeyRoute, frontmost: impl FnOnce() -> Option<i32>) -> Option<i32> {
    match route {
        KeyRoute::Pid(pid)
        | KeyRoute::BareGlobal(pid)
        | KeyRoute::Global(Some(pid))
        | KeyRoute::GuardedGlobal(pid, _) => Some(pid),
        KeyRoute::Global(None) => frontmost(),
    }
}

/// A key posted to the HID queue. Recorded so a pending focus restoration
/// does not read the driver's own key as the person's input.
fn post_hid(send: impl FnOnce()) {
    crate::input::hid_history::post(send);
}

#[cfg(test)]
pub(crate) fn deliver_with(
    key: &str,
    modifiers: &[&str],
    duration: Duration,
    post: impl FnMut(u16, bool, CGEventFlags) -> anyhow::Result<()>,
    cancelled: impl FnMut() -> bool,
    wait: impl FnMut(Duration) -> bool,
) -> anyhow::Result<()> {
    deliver_with_pacing(
        key,
        modifiers,
        duration,
        (false, false),
        // Tests pin US key positions: the host layout is per-user state.
        |key, _| keyboard::key_name_to_code(key),
        post,
        cancelled,
        wait,
    )
}

/// Press `key` with `modifiers` held for `duration`, then release everything
/// in reverse order. `resolve` maps the base key name and the chord's
/// modifiers to its virtual key; it runs before anything is posted, so a
/// failure leaves nothing down.
#[allow(clippy::too_many_arguments)]
fn deliver_with_pacing(
    key: &str,
    modifiers: &[&str],
    duration: Duration,
    pacing: (bool, bool),
    resolve: impl FnOnce(&str, &[&str]) -> anyhow::Result<u16>,
    mut post: impl FnMut(u16, bool, CGEventFlags) -> anyhow::Result<()>,
    mut cancelled: impl FnMut() -> bool,
    mut wait: impl FnMut(Duration) -> bool,
) -> anyhow::Result<()> {
    let (pace_modifiers, pace_base_up) = pacing;
    let plus = key == "+" || key.eq_ignore_ascii_case("plus");
    let base = resolve(if plus { "=" } else { key }, modifiers)?;
    let mut keys = Vec::new();
    for modifier in modifiers.iter().copied().chain(plus.then_some("shift")) {
        if let Some((code, flag)) = keyboard::modifier_key_code_and_flag(modifier) {
            if !keys.iter().any(|(old, _)| *old == code) {
                keys.push((code, flag));
            }
        }
    }
    let mut pressed = Vec::new();
    let mut flags = CGEventFlags::CGEventFlagNull;
    let mut base_attempted = false;
    let mut result: anyhow::Result<()> = Ok(());
    let delivery = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        for (code, flag) in keys {
            if cancelled() {
                break;
            }
            flags |= flag;
            pressed.push((code, flag));
            if let Err(error) = post(code, true, flags) {
                result = Err(error);
                break;
            }
            if pace_modifiers && wait(keyboard::key_gap()) {
                break;
            }
        }
        if result.is_ok() && !cancelled() {
            base_attempted = true;
            if let Err(error) = post(base, true, flags) {
                result = Err(error);
            } else if wait(duration) {
                result = Err(anyhow::anyhow!("key delivery cancelled"));
            }
        }
    }));
    if base_attempted {
        if let Err(error) = post(base, false, flags) {
            let _ = post(base, false, flags);
            if result.is_ok() {
                result = Err(error);
            }
        }
    }
    if base_attempted && pace_base_up {
        wait(keyboard::key_gap());
    }
    for (code, flag) in pressed.into_iter().rev() {
        flags.remove(flag);
        if let Err(error) = post(code, false, flags) {
            let _ = post(code, false, flags);
            if result.is_ok() {
                result = Err(error);
            }
        }
        if pace_modifiers {
            wait(keyboard::key_gap());
        }
    }
    if let Err(panic) = delivery {
        std::panic::resume_unwind(panic);
    }
    if result.is_ok() && cancelled() {
        return Err(anyhow::anyhow!("key delivery cancelled"));
    }
    result
}

#[cfg(test)]
mod u29b_cleanup_tests {
    use super::*;
    use std::cell::RefCell;
    use std::panic::{catch_unwind, AssertUnwindSafe};

    #[test]
    fn poster_panic_after_base_down_releases_before_unwind() {
        let events = RefCell::new(Vec::new());
        let outcome = catch_unwind(AssertUnwindSafe(|| {
            deliver_with(
                "a",
                &["cmd", "shift"],
                Duration::ZERO,
                |code, down, _| {
                    events.borrow_mut().push((code, down));
                    if code == 0 && down {
                        panic!("poster panic after down");
                    }
                    Ok(())
                },
                || false,
                |_| false,
            )
        }));
        assert!(outcome.is_err());
        assert_eq!(
            *events.borrow(),
            vec![
                (55, true),
                (56, true),
                (0, true),
                (0, false),
                (56, false),
                (55, false)
            ]
        );
    }

    #[test]
    fn menu_wait_panic_still_posts_up() {
        let events = RefCell::new(Vec::new());
        let outcome = catch_unwind(AssertUnwindSafe(|| {
            deliver_menu_with(
                Ok((1, 2)),
                |event| events.borrow_mut().push(*event),
                || false,
                || panic!("wait panic"),
            )
        }));
        assert!(outcome.is_err());
        assert_eq!(*events.borrow(), vec![1, 2]);
    }

    #[test]
    fn failed_key_up_remains_an_error_after_modifier_cleanup() {
        let events = RefCell::new(Vec::new());
        let result = deliver_with(
            "a",
            &["cmd", "shift"],
            Duration::ZERO,
            |code, down, _| {
                events.borrow_mut().push((code, down));
                if code == 0 && !down {
                    anyhow::bail!("base up failed");
                }
                Ok(())
            },
            || false,
            |_| false,
        );
        assert_eq!(result.unwrap_err().to_string(), "base up failed");
        assert_eq!(
            *events.borrow(),
            vec![
                (55, true),
                (56, true),
                (0, true),
                (0, false),
                (0, false),
                (56, false),
                (55, false)
            ]
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn desktop_chord_stops_on_foreground_switch_and_releases_modifiers() {
        let frontmost = std::cell::Cell::new(7);
        let events = std::cell::RefCell::new(Vec::new());
        let mut posted_down = std::collections::HashMap::new();
        let result = deliver_with(
            "a",
            &["cmd"],
            Duration::ZERO,
            |code, down, _| {
                post_if_foreground(
                    (7, 11),
                    || Some((frontmost.get(), 11)),
                    code,
                    down,
                    &mut posted_down,
                    || {
                        events.borrow_mut().push((code, down));
                        if code == 55 && down {
                            frontmost.set(8);
                        }
                        Ok(())
                    },
                )
            },
            || false,
            |_| false,
        );
        assert!(result.is_err());
        assert_eq!(*events.borrow(), vec![(55, true), (55, false)]);
    }

    #[test]
    fn desktop_chord_duplicate_modifier_and_base_keeps_both_release_transitions() {
        for switch_before_base in [false, true] {
            let frontmost = std::cell::Cell::new(7);
            let events = std::cell::RefCell::new(Vec::new());
            let mut posted_down = std::collections::HashMap::new();
            let result = deliver_with(
                "shift",
                &["shift"],
                Duration::ZERO,
                |code, down, _| {
                    post_if_foreground(
                        (7, 11),
                        || Some((frontmost.get(), 11)),
                        code,
                        down,
                        &mut posted_down,
                        || {
                            events.borrow_mut().push((code, down));
                            if switch_before_base && down {
                                frontmost.set(8);
                            }
                            Ok(())
                        },
                    )
                },
                || false,
                |_| false,
            );
            if switch_before_base {
                assert!(result.is_err());
                assert_eq!(*events.borrow(), vec![(56, true), (56, false)]);
            } else {
                assert!(result.is_ok());
                assert_eq!(
                    *events.borrow(),
                    vec![(56, true), (56, true), (56, false), (56, false)]
                );
            }
        }
    }

    #[test]
    fn desktop_chord_refuses_reused_or_unreadable_process_instance() {
        assert!(instance_matches(Some(11), Some(11)));
        assert!(!instance_matches(Some(11), Some(12)));
        assert!(!instance_matches(Some(11), None));
        assert!(!instance_matches(None, None));
        let mut sent = false;
        let mut posted_down = std::collections::HashMap::new();
        let result = post_if_foreground(
            (7, 11),
            || Some((7, 12)),
            0,
            true,
            &mut posted_down,
            || {
                sent = true;
                Ok(())
            },
        );
        assert!(result.is_err());
        assert!(!sent, "a reused PID must not receive a key-down");
    }

    #[test]
    fn duplicate_base_release_failures_still_send_modifier_release() {
        let events = std::cell::RefCell::new(Vec::new());
        let mut posted_down = std::collections::HashMap::new();
        let mut failed_ups = 0;
        let result = deliver_with(
            "shift",
            &["shift"],
            Duration::ZERO,
            |code, down, flags| {
                post_if_foreground(
                    (7, 11),
                    || Some((7, 11)),
                    code,
                    down,
                    &mut posted_down,
                    || {
                        events.borrow_mut().push((code, down, flags));
                        if !down && failed_ups < 2 {
                            failed_ups += 1;
                            anyhow::bail!("base release failed");
                        }
                        Ok(())
                    },
                )
            },
            || false,
            |_| false,
        );
        assert!(result.is_err());
        let events = events.borrow();
        assert_eq!(events.len(), 5);
        assert_eq!(
            events.last().map(|event| (event.0, event.1)),
            Some((56, false))
        );
        assert!(!events
            .last()
            .unwrap()
            .2
            .contains(CGEventFlags::CGEventFlagShift));
    }

    /// A bare-global key reaches the HID queue: it must be in the HID history,
    /// or a pending pixel focus restoration abandons with reason user_input.
    #[test]
    #[ignore = "live: reads HID idle time; runs alone in the macOS live lane"]
    fn bare_global_hid_post_is_recorded_as_the_drivers_own() {
        let since = std::time::Instant::now();
        let mut sent = false;
        post_hid(|| sent = true);
        assert!(sent);
        assert!(
            crate::input::hid_history::recorded_since(since) >= 1,
            "bare-global HID post missing from the HID history"
        );
    }

    #[test]
    fn global_chord_paces_each_modifier_transition() {
        let waits = std::cell::RefCell::new(Vec::new());
        deliver_with_pacing(
            "a",
            &["cmd", "shift"],
            Duration::from_millis(99),
            (true, false),
            |key, _| keyboard::key_name_to_code(key),
            |_, _, _| Ok(()),
            || false,
            |duration| {
                waits.borrow_mut().push(duration);
                false
            },
        )
        .unwrap();
        assert_eq!(
            *waits.borrow(),
            vec![
                keyboard::key_gap(),
                keyboard::key_gap(),
                Duration::from_millis(99),
                keyboard::key_gap(),
                keyboard::key_gap()
            ]
        );
    }

    #[test]
    fn bare_global_paces_after_base_up() {
        let events = std::cell::RefCell::new(Vec::new());
        deliver_with_pacing(
            "a",
            &["cmd"],
            Duration::from_millis(9),
            (true, true),
            |key, _| keyboard::key_name_to_code(key),
            |_, down, _| {
                events.borrow_mut().push(if down { "down" } else { "up" });
                Ok(())
            },
            || false,
            |_| {
                events.borrow_mut().push("wait");
                false
            },
        )
        .unwrap();
        assert_eq!(
            *events.borrow(),
            vec!["down", "wait", "down", "wait", "up", "wait", "up", "wait"]
        );
    }

    #[test]
    fn menu_construction_failure_is_clean_and_interruption_releases() {
        let events = std::cell::RefCell::new(Vec::new());
        let failed = deliver_menu_with::<u8>(
            Err(anyhow::anyhow!("build failed")),
            |event| events.borrow_mut().push(*event),
            || false,
            || false,
        );
        assert_eq!(
            failed,
            keyboard::ChordPost::NotPosted("build failed".into())
        );
        assert!(events.borrow().is_empty());
        let interrupted = deliver_menu_with(
            Ok((1, 2)),
            |event| events.borrow_mut().push(*event),
            || false,
            || true,
        );
        assert!(matches!(interrupted, keyboard::ChordPost::Uncertain(_)));
        assert_eq!(*events.borrow(), vec![1, 2]);
    }

    /// The base key is the one the resolver reads from the target's layout
    /// (AZERTY types `a` with Command on US Q, key 12), and `+` resolves as
    /// `=` with the chord's own modifiers while Shift is still pressed.
    #[test]
    fn base_key_is_resolved_on_the_target_layout_before_posting() {
        let events = std::cell::RefCell::new(Vec::new());
        let asked = std::cell::RefCell::new(Vec::new());
        deliver_with_pacing(
            "a",
            &["cmd"],
            Duration::ZERO,
            (false, false),
            |key, modifiers| {
                asked
                    .borrow_mut()
                    .push((key.to_owned(), modifiers.join("+")));
                Ok(12)
            },
            |code, down, _| {
                events.borrow_mut().push((code, down));
                Ok(())
            },
            || false,
            |_| false,
        )
        .unwrap();
        assert_eq!(
            *events.borrow(),
            vec![(55, true), (12, true), (12, false), (55, false)]
        );

        events.borrow_mut().clear();
        deliver_with_pacing(
            "+",
            &["cmd"],
            Duration::ZERO,
            (false, false),
            |key, modifiers| {
                asked
                    .borrow_mut()
                    .push((key.to_owned(), modifiers.join("+")));
                Ok(24)
            },
            |code, down, _| {
                events.borrow_mut().push((code, down));
                Ok(())
            },
            || false,
            |_| false,
        )
        .unwrap();
        assert_eq!(
            *asked.borrow(),
            vec![("a".into(), "cmd".into()), ("=".into(), "cmd".into())]
        );
        assert_eq!(
            *events.borrow(),
            vec![
                (55, true),
                (56, true),
                (24, true),
                (24, false),
                (56, false),
                (55, false)
            ]
        );
    }

    #[test]
    fn a_key_that_does_not_resolve_posts_nothing() {
        let events = std::cell::RefCell::new(Vec::new());
        let result = deliver_with_pacing(
            "asterisk",
            &["cmd"],
            Duration::ZERO,
            (false, false),
            |_, _| Err(anyhow::anyhow!("Unknown key name")),
            |code, down, _| {
                events.borrow_mut().push((code, down));
                Ok(())
            },
            || false,
            |_| false,
        );
        assert!(result.is_err());
        assert!(events.borrow().is_empty());
    }

    /// A HID-queue chord aimed at an app reads that app's layout (a Screen
    /// Sharing target keeps US positions even when another app is frontmost);
    /// only a desktop-scope chord falls back to the frontmost app.
    #[test]
    fn layout_target_is_the_routes_own_pid() {
        let frontmost = || Some(99);
        assert_eq!(layout_target(KeyRoute::Pid(7), frontmost), Some(7));
        assert_eq!(layout_target(KeyRoute::BareGlobal(7), frontmost), Some(7));
        assert_eq!(layout_target(KeyRoute::Global(Some(7)), frontmost), Some(7));
        assert_eq!(layout_target(KeyRoute::Global(None), frontmost), Some(99));
        assert_eq!(layout_target(KeyRoute::Global(None), || None), None);
    }

    #[test]
    fn explicit_hold_is_passed_unclamped_to_the_waiter() {
        for ms in [0, 30_000] {
            let waited = std::cell::Cell::new(None);
            deliver_with(
                "a",
                &[],
                Duration::from_millis(ms),
                |_, _, _| Ok(()),
                || false,
                |duration| {
                    waited.set(Some(duration));
                    false
                },
            )
            .unwrap();
            assert_eq!(waited.get(), Some(Duration::from_millis(ms)));
        }
    }

    #[test]
    fn cancellation_after_modifier_down_releases_without_base_down() {
        let events = std::cell::RefCell::new(Vec::new());
        let cancelled = std::cell::Cell::new(false);
        let result = deliver_with(
            "a",
            &["cmd", "shift"],
            Duration::from_millis(8),
            |code, down, _| {
                events.borrow_mut().push((code, down));
                if code == 55 && down {
                    cancelled.set(true);
                }
                Ok(())
            },
            || cancelled.get(),
            |_| panic!("hold should not start"),
        );
        assert!(result.is_err());
        assert_eq!(*events.borrow(), vec![(55, true), (55, false)]);
    }
    #[test]
    fn physical_chord_posts_modifiers_and_releases_in_reverse_order() {
        let events = std::cell::RefCell::new(Vec::new());
        deliver_with(
            "a",
            &["cmd", "shift"],
            std::time::Duration::ZERO,
            |code, down, _| {
                events.borrow_mut().push((code, down));
                Ok(())
            },
            || false,
            |_| false,
        )
        .unwrap();
        assert_eq!(
            *events.borrow(),
            vec![
                (55, true),
                (56, true),
                (0, true),
                (0, false),
                (56, false),
                (55, false)
            ]
        );
    }

    #[test]
    fn failed_base_key_down_releases_pressed_modifiers() {
        let events = std::cell::RefCell::new(Vec::new());
        let result = deliver_with(
            "a",
            &["cmd", "shift"],
            std::time::Duration::ZERO,
            |code, down, _| {
                events.borrow_mut().push((code, down));
                if code == 0 && down {
                    anyhow::bail!("injected failure");
                }
                Ok(())
            },
            || false,
            |_| false,
        );
        assert!(result.is_err());
        assert_eq!(
            *events.borrow(),
            vec![
                (55, true),
                (56, true),
                (0, true),
                (0, false),
                (56, false),
                (55, false)
            ]
        );
    }

    #[test]
    fn failed_base_key_up_still_releases_modifiers_in_reverse_order() {
        let events = std::cell::RefCell::new(Vec::new());
        let result = deliver_with(
            "a",
            &["cmd", "shift"],
            std::time::Duration::ZERO,
            |code, down, _| {
                events.borrow_mut().push((code, down));
                if code == 0 && !down {
                    anyhow::bail!("injected failure");
                }
                Ok(())
            },
            || false,
            |_| false,
        );
        assert!(result.is_err());
        assert_eq!(
            *events.borrow(),
            vec![
                (55, true),
                (56, true),
                (0, true),
                (0, false),
                (0, false),
                (56, false),
                (55, false)
            ]
        );
    }
}

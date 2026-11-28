//! Ownership boundary for a key delivery and its focus guards.

use std::sync::Arc;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum FocusConfirmation {
    Confirmed,
    SecureInput,
    WrongProcess,
    WrongWindow,
    Unprocessed,
}

pub(super) fn classify_focus_confirmation(
    expected_pid: i32,
    observed_pid: Option<i32>,
    window_matches: bool,
    secure_input_turned_on: bool,
) -> FocusConfirmation {
    if secure_input_turned_on {
        FocusConfirmation::SecureInput
    } else if observed_pid != Some(expected_pid) {
        FocusConfirmation::WrongProcess
    } else if !window_matches {
        FocusConfirmation::WrongWindow
    } else {
        FocusConfirmation::Confirmed
    }
}

pub(super) fn split_key_segments(text: &str) -> Vec<&str> {
    let mut segments = Vec::new();
    let mut start = 0;
    for (index, ch) in text.char_indices() {
        if matches!(ch, '\t' | '\n' | '\r') {
            let end = index + ch.len_utf8();
            segments.push(&text[start..end]);
            start = end;
        }
    }
    if start < text.len() {
        segments.push(&text[start..]);
    }
    segments
}

pub(super) fn has_checked_boundary(text: &str, check_last: bool) -> bool {
    text.char_indices().any(|(index, ch)| {
        matches!(ch, '\t' | '\n' | '\r') && (check_last || index + ch.len_utf8() < text.len())
    })
}

pub(super) fn drive_key_segments<E, P>(
    text: &str,
    check_last: bool,
    mut probe: impl FnMut() -> P,
    mut check: impl FnMut(P) -> FocusConfirmation,
    mut send: impl FnMut(&str, bool) -> Result<usize, E>,
) -> Result<(usize, Option<FocusConfirmation>), E> {
    let segments = split_key_segments(text);
    let mut delivered = 0;
    let mut first = true;
    let mut send_unit = |unit: &str, delivered: &mut usize| -> Result<bool, E> {
        let sent = send(unit, first)?;
        first = false;
        *delivered += sent;
        Ok(sent >= unit.chars().count())
    };
    for (index, segment) in segments.iter().enumerate() {
        let boundary = segment
            .chars()
            .last()
            .filter(|ch| matches!(ch, '\t' | '\n' | '\r'))
            .filter(|_| check_last || index + 1 < segments.len());
        let Some(boundary) = boundary else {
            if !send_unit(segment, &mut delivered)? {
                return Ok((delivered, None));
            }
            continue;
        };
        let split = segment.len() - boundary.len_utf8();
        let (body, key) = segment.split_at(split);
        if !body.is_empty() && !send_unit(body, &mut delivered)? {
            return Ok((delivered, None));
        }
        let before = probe();
        if !send_unit(key, &mut delivered)? {
            return Ok((delivered, None));
        }
        let focus = check(before);
        if focus != FocusConfirmation::Confirmed {
            return Ok((delivered, Some(focus)));
        }
    }
    Ok((delivered, None))
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct BoundaryProbe {
    element: Option<usize>,
    window: Option<(u32, u32)>,
    value: Option<String>,
    secure_input: bool,
}

impl BoundaryProbe {
    pub(super) fn sample(pid: i32, value: Option<String>) -> Self {
        let element = unsafe {
            crate::ax::bindings::focused_element_of_pid(pid).map(|element| {
                let hash = core_foundation::base::CFHash(element as _);
                core_foundation::base::CFRelease(element as _);
                hash as usize
            })
        };
        Self {
            element,
            window: crate::ax::bindings::focused_window_ids_of_pid(pid)
                .map(|window| (window.own, window.root)),
            value,
            secure_input: crate::input::keyboard::secure_event_input_active(),
        }
    }

    pub(super) fn secure_input(&self) -> bool {
        self.secure_input
    }

    pub(super) fn shows_boundary_since(&self, before: &Self) -> bool {
        observed_change(self.element, before.element)
            || observed_change(self.window, before.window)
            || self.secure_input != before.secure_input
            || self
                .value
                .as_deref()
                .zip(before.value.as_deref())
                .is_some_and(|(after, before)| value_shows_boundary(before, after))
    }
}

fn observed_change<T: PartialEq>(after: Option<T>, before: Option<T>) -> bool {
    after
        .zip(before)
        .is_some_and(|(after, before)| after != before)
}

pub(super) struct SecureInputLatch {
    seen_off: std::cell::Cell<bool>,
    turned_on: std::cell::Cell<bool>,
}

impl SecureInputLatch {
    pub(super) fn new(active: bool) -> Self {
        Self {
            seen_off: std::cell::Cell::new(!active),
            turned_on: std::cell::Cell::new(false),
        }
    }

    pub(super) fn observe(&self, active: bool) {
        if !active {
            self.seen_off.set(true);
        } else if self.seen_off.get() {
            self.turned_on.set(true);
        }
    }

    pub(super) fn turned_on(&self) -> bool {
        self.turned_on.get()
    }
}

fn boundary_marks(value: &str) -> usize {
    value
        .chars()
        .filter(|ch| matches!(ch, '\t' | '\n' | '\r'))
        .count()
}

pub(super) fn value_shows_boundary(before: &str, after: &str) -> bool {
    boundary_marks(after) > boundary_marks(before)
}

pub(super) fn await_boundary<P>(
    before: &P,
    mut sample: impl FnMut() -> P,
    shows_boundary: impl Fn(&P, &P) -> bool,
    mut wait: impl FnMut() -> bool,
) -> bool {
    loop {
        if shows_boundary(&sample(), before) {
            return true;
        }
        if !wait() {
            return false;
        }
    }
}

pub(super) fn capture_post_focus(
    result: anyhow::Result<()>,
    pid: i32,
    window: Option<u32>,
    global: bool,
    sample: &std::sync::Mutex<Option<bool>>,
) -> anyhow::Result<()> {
    if result.is_ok() {
        if let Some(wid) = window {
            let observed_pid = if global {
                crate::input::skylight::front_process_pid()
            } else {
                Some(pid)
            };
            let routed = !crate::input::skylight::sibling_window_focused(pid, wid);
            let confirmation = classify_focus_confirmation(pid, observed_pid, routed, false);
            *sample.lock().unwrap_or_else(|e| e.into_inner()) =
                Some(confirmation == FocusConfirmation::Confirmed);
        }
    }
    result
}

pub(super) fn add_focus_confirmation(value: &mut serde_json::Value, sample: Option<bool>) {
    if let Some(confirmed) = sample {
        value["focus_confirmed"] = serde_json::json!(confirmed);
    }
}

pub(super) fn final_window_sample(
    pid: i32,
    wid: u32,
    scope: &mut Option<crate::input::skylight::WindowKeyScope>,
) -> anyhow::Result<Option<u32>> {
    let cached = scope.as_ref().is_some_and(|scope| scope.used_belief());
    final_window_sample_with(
        wid,
        cached,
        || crate::ax::bindings::focused_window_id_of_pid(pid),
        || crate::input::skylight::invalidate_key_belief(pid),
        || {
            drop(scope.take());
            *scope = Some(
                crate::input::skylight::enter_exact_window_key_inline_detailed(pid, wid)
                    .map_err(|error| anyhow::anyhow!(error.reason))?,
            );
            crate::goat::action_latency::key_belief(true, false);
            Ok(())
        },
    )
}

fn final_window_sample_with(
    wid: u32,
    cached: bool,
    mut read: impl FnMut() -> Option<u32>,
    invalidate: impl FnOnce(),
    prime: impl FnOnce() -> anyhow::Result<()>,
) -> anyhow::Result<Option<u32>> {
    let mut sample = read();
    if sample != Some(wid) && cached {
        invalidate();
        prime()?;
        sample = read();
    }
    Ok(sample)
}

pub(super) fn duration_from_args(
    args: &serde_json::Value,
) -> Result<Option<std::time::Duration>, cua_driver_core::protocol::ToolResult> {
    let Some(raw) = args.get("duration_ms") else {
        return Ok(None);
    };
    let Some(value) = raw.as_u64().filter(|value| *value <= 30_000) else {
        return Err(super::invalid_arguments(
            "duration_ms must be an integer between 0 and 30000",
        ));
    };
    Ok(Some(std::time::Duration::from_millis(value)))
}

pub(super) fn semantic_duration(
    mut result: cua_driver_core::protocol::ToolResult,
    duration: Option<std::time::Duration>,
) -> cua_driver_core::protocol::ToolResult {
    if duration.is_some() {
        if let Some(serde_json::Value::Object(fields)) = result.structured_content.as_mut() {
            fields.insert("duration_applied".into(), serde_json::json!(false));
        } else {
            result.structured_content = Some(serde_json::json!({"duration_applied": false}));
        }
    }
    result
}

/// Keep ownership in the blocking worker until it has released all keys.
pub(super) async fn run_owned<L, S, F, R>(
    lease: L,
    scope: S,
    control: Arc<crate::input::key_hold::HoldControl>,
    work: F,
) -> Result<R, tokio::task::JoinError>
where
    L: Send + 'static,
    S: Send + 'static,
    F: FnOnce(Arc<crate::input::key_hold::HoldControl>, &mut S) -> R + Send + 'static,
    R: Send + 'static,
{
    let worker_control = control.clone();
    with_disconnect_control(
        control,
        crate::goat::action_latency::spawn_action_blocking(move || {
            let _lease = lease;
            let mut scope = scope;
            let result = work(worker_control, &mut scope);
            crate::goat::action_latency::section(
                crate::goat::action_latency::Stage::KeyHandBack,
                || drop(scope),
            );
            result
        }),
    )
    .await
}

/// Signal the worker without dropping its future or releasing its guards early.
pub(super) async fn with_disconnect_control<F: std::future::Future>(
    control: Arc<crate::input::key_hold::HoldControl>,
    future: F,
) -> F::Output {
    let _cancel_on_drop = crate::input::key_hold::CancelOnDrop(control.clone());
    let Some(mut disconnect) = cua_driver_core::key_cancellation::subscribe() else {
        return future.await;
    };
    tokio::pin!(future);
    if *disconnect.borrow() {
        control.cancel();
    } else {
        tokio::select! {
            result = &mut future => return result,
            changed = disconnect.changed() => {
                if changed.is_ok() && *disconnect.borrow() {
                    control.cancel();
                }
            }
        }
    }
    future.await
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    #[test]
    fn split_key_segments_preserves_boundaries_and_unicode() {
        assert_eq!(split_key_segments(""), Vec::<&str>::new());
        assert_eq!(split_key_segments("한글"), vec!["한글"]);
        assert_eq!(split_key_segments("a\t"), vec!["a\t"]);
        assert_eq!(split_key_segments("a\r\nb"), vec!["a\r", "\n", "b"]);
        assert_eq!(split_key_segments("\t\tb"), vec!["\t", "\t", "b"]);
    }

    #[test]
    fn focus_confirmation_requires_process_window_and_no_secure_input() {
        use FocusConfirmation::*;
        assert_eq!(
            classify_focus_confirmation(7, Some(7), true, false),
            Confirmed
        );
        assert_eq!(
            classify_focus_confirmation(7, Some(7), false, false),
            WrongWindow
        );
        assert_eq!(
            classify_focus_confirmation(7, Some(8), true, false),
            WrongProcess
        );
        assert_eq!(
            classify_focus_confirmation(7, None, true, false),
            WrongProcess
        );
        assert_eq!(
            classify_focus_confirmation(7, Some(7), true, true),
            SecureInput
        );
    }

    #[test]
    fn segmented_delivery_stops_at_first_focus_failure() {
        let mut sent = Vec::new();
        let result = drive_key_segments(
            "한\tremaining\nlast",
            false,
            || (),
            |()| FocusConfirmation::WrongWindow,
            |segment, _| {
                sent.push(segment.to_owned());
                Ok::<usize, ()>(segment.chars().count())
            },
        );
        assert_eq!(result, Ok((2, Some(FocusConfirmation::WrongWindow))));
        assert_eq!(sent, vec!["한", "\t"]);
    }

    #[test]
    fn segmented_delivery_checks_boundaries_and_stops_on_each_failure() {
        for failure in [
            FocusConfirmation::SecureInput,
            FocusConfirmation::WrongProcess,
            FocusConfirmation::WrongWindow,
            FocusConfirmation::Unprocessed,
        ] {
            let mut sent = Vec::new();
            let result = drive_key_segments(
                "a\tb\nc",
                false,
                || (),
                |()| failure,
                |unit, first| {
                    sent.push((unit.to_owned(), first));
                    Ok::<_, ()>(unit.chars().count())
                },
            );
            assert_eq!(result, Ok((2, Some(failure))));
            assert_eq!(sent, vec![("a".to_owned(), true), ("\t".to_owned(), false)]);
        }
        let log = std::cell::RefCell::new(Vec::new());
        let result = drive_key_segments(
            "a\tb\nc",
            false,
            || log.borrow_mut().push("probe".to_owned()),
            |()| {
                log.borrow_mut().push("check".to_owned());
                FocusConfirmation::Confirmed
            },
            |unit, first| {
                log.borrow_mut().push(format!("{unit:?} {first}"));
                Ok::<_, ()>(unit.chars().count())
            },
        );
        assert_eq!(result, Ok((5, None)));
        assert_eq!(
            log.into_inner(),
            vec![
                "\"a\" true",
                "probe",
                "\"\\t\" false",
                "check",
                "\"b\" false",
                "probe",
                "\"\\n\" false",
                "check",
                "\"c\" false",
            ]
        );
    }

    #[test]
    fn trailing_boundary_is_checked_only_when_more_input_follows() {
        for (check_last, checks) in [(false, 0), (true, 1)] {
            let mut count = 0;
            let result = drive_key_segments(
                "ab\t",
                check_last,
                || (),
                |()| {
                    count += 1;
                    FocusConfirmation::Confirmed
                },
                |unit, _| Ok::<_, ()>(unit.chars().count()),
            );
            assert_eq!(result, Ok((3, None)));
            assert_eq!(count, checks);
        }
    }

    #[test]
    fn short_send_stops_before_the_boundary_key() {
        let mut sent = Vec::new();
        let result = drive_key_segments(
            "abc\td",
            false,
            || (),
            |()| FocusConfirmation::Confirmed,
            |unit, _| {
                sent.push(unit.to_owned());
                Ok::<_, ()>(1)
            },
        );
        assert_eq!(result, Ok((1, None)));
        assert_eq!(sent, vec!["abc"]);
    }

    #[test]
    fn boundary_wait_requires_an_observed_change() {
        let mut samples = [1, 1, 2].into_iter();
        let mut waits = 0;
        assert!(await_boundary(
            &1,
            || samples.next().unwrap(),
            |after, before| after != before,
            || {
                waits += 1;
                true
            }
        ));
        assert_eq!(waits, 2);
        let mut waits = 0;
        assert!(!await_boundary(
            &1,
            || 1,
            |after, before| after != before,
            || {
                waits += 1;
                waits < 3
            }
        ));
        assert_eq!(waits, 3);
    }

    #[test]
    fn only_boundaries_followed_by_more_input_need_checks() {
        assert!(!has_checked_boundary("plain", true));
        assert!(!has_checked_boundary("line\n", false));
        assert!(has_checked_boundary("line\n", true));
        assert!(has_checked_boundary("user\tpass", false));
        assert!(has_checked_boundary("한\r글", false));
    }

    #[test]
    fn unavailable_focus_reads_are_not_change_evidence() {
        assert!(!observed_change(None, Some(4)));
        assert!(!observed_change(Some(4), None));
        assert!(!observed_change::<usize>(None, None));
        assert!(!observed_change(Some(4), Some(4)));
        assert!(observed_change(Some(5), Some(4)));
    }

    #[test]
    fn secure_input_reactivation_stays_latched() {
        let latch = SecureInputLatch::new(true);
        latch.observe(true);
        assert!(!latch.turned_on());
        latch.observe(false);
        assert!(!latch.turned_on());
        latch.observe(true);
        assert!(latch.turned_on());
        latch.observe(false);
        assert!(latch.turned_on());
        let latch = SecureInputLatch::new(false);
        latch.observe(true);
        assert!(latch.turned_on());
    }

    #[test]
    fn late_preceding_text_is_not_boundary_evidence() {
        assert!(!value_shows_boundary("user", "username"));
        assert!(!value_shows_boundary("", "한글"));
        assert!(!value_shows_boundary("a\tb", "a\tbc"));
        assert!(!value_shows_boundary("abcdef", "x"));
        assert!(!value_shows_boundary("a\tb", "x"));
    }

    #[test]
    fn inserted_boundary_is_evidence() {
        assert!(value_shows_boundary("username", "username\t"));
        assert!(value_shows_boundary("line", "line\n"));
        assert!(value_shows_boundary("line", "line\r"));
    }

    #[test]
    fn focus_confirmation_payload_is_optional_and_preserves_fields() {
        for confirmed in [true, false] {
            let mut value = serde_json::json!({"effect": "unverifiable"});
            add_focus_confirmation(&mut value, Some(confirmed));
            assert_eq!(value["focus_confirmed"], confirmed);
            assert_eq!(value["effect"], "unverifiable");
        }
    }

    #[tokio::test]
    async fn disconnect_signals_worker_without_dropping_its_future() {
        let (disconnect, receiver) = tokio::sync::watch::channel(false);
        let control = Arc::new(crate::input::key_hold::HoldControl::default());
        let (finish, finished) = tokio::sync::oneshot::channel::<()>();
        let mut delivery = Box::pin(cua_driver_core::key_cancellation::scope(
            receiver,
            with_disconnect_control(control.clone(), async { finished.await.unwrap() }),
        ));
        use std::future::Future;
        assert!(
            std::future::poll_fn(|cx| std::task::Poll::Ready(
                delivery.as_mut().poll(cx).is_pending()
            ))
            .await
        );
        disconnect.send(true).unwrap();
        assert!(
            std::future::poll_fn(|cx| std::task::Poll::Ready(
                delivery.as_mut().poll(cx).is_pending()
            ))
            .await
        );
        assert!(control.is_cancelled());
        finish.send(()).unwrap();
        delivery.await;
    }

    #[test]
    fn explicit_duration_on_semantic_action_is_disclosed_as_unapplied() {
        let result = cua_driver_core::protocol::ToolResult::text("done")
            .with_structured(serde_json::json!({"effect": "unverifiable"}));
        let result = semantic_duration(result, Some(std::time::Duration::ZERO));
        assert_eq!(
            result.structured_content.unwrap()["duration_applied"],
            false
        );
        let result = cua_driver_core::protocol::ToolResult::text("done")
            .with_structured(serde_json::json!({"effect": "unverifiable"}));
        let result = semantic_duration(result, None);
        assert!(result
            .structured_content
            .unwrap()
            .get("duration_applied")
            .is_none());
    }

    #[test]
    fn final_mismatch_invalidates_then_reprimes_before_resampling() {
        let events = std::cell::RefCell::new(Vec::new());
        let mut samples = [Some(888), Some(700)].into_iter();
        let result = final_window_sample_with(
            700,
            true,
            || {
                events.borrow_mut().push("read");
                samples.next().unwrap()
            },
            || events.borrow_mut().push("invalidate"),
            || {
                events.borrow_mut().push("prime");
                Ok(())
            },
        )
        .unwrap();
        assert_eq!(result, Some(700));
        assert_eq!(
            *events.borrow(),
            vec!["read", "invalidate", "prime", "read"]
        );
    }

    #[test]
    fn final_match_uses_one_window_sample() {
        let reads = std::cell::Cell::new(0);
        let result = final_window_sample_with(
            700,
            true,
            || {
                reads.set(reads.get() + 1);
                Some(700)
            },
            || panic!("matched belief cannot be invalidated"),
            || panic!("matched belief cannot be primed"),
        )
        .unwrap();
        assert_eq!(result, Some(700));
        assert_eq!(reads.get(), 1);
    }

    #[test]
    fn final_mismatch_propagates_reprime_failure_without_resampling() {
        let reads = std::cell::Cell::new(0);
        let result = final_window_sample_with(
            700,
            true,
            || {
                reads.set(reads.get() + 1);
                Some(888)
            },
            || {},
            || anyhow::bail!("reprime failed"),
        );
        assert_eq!(result.unwrap_err().to_string(), "reprime failed");
        assert_eq!(reads.get(), 1);
    }

    struct TrackedScope(Arc<Mutex<Vec<&'static str>>>);
    impl Drop for TrackedScope {
        fn drop(&mut self) {
            self.0.lock().unwrap().push("hand_back");
        }
    }

    struct TrackedLease {
        _guard: tokio::sync::OwnedMutexGuard<()>,
        events: Arc<Mutex<Vec<&'static str>>>,
    }
    impl Drop for TrackedLease {
        fn drop(&mut self) {
            self.events.lock().unwrap().push("lease_release");
        }
    }

    async fn cancel_invoking_future(base_down: bool, pid: i32, hotkey: bool) {
        let events = Arc::new(Mutex::new(Vec::new()));
        let lease = TrackedLease {
            _guard: crate::background_mutation::acquire(pid).await,
            events: events.clone(),
        };
        let scope = TrackedScope(events.clone());
        let (ready_tx, ready_rx) = tokio::sync::oneshot::channel();
        let (done_tx, done_rx) = tokio::sync::oneshot::channel();
        let (resume_tx, resume_rx) = std::sync::mpsc::channel();
        let worker_events = events.clone();
        let control = Arc::new(crate::input::key_hold::HoldControl::default());
        let work = move |control: Arc<crate::input::key_hold::HoldControl>,
                         _scope: &mut TrackedScope| {
            let mut ready = Some(ready_tx);
            let result = crate::input::key_hold::deliver_with(
                "a",
                &["cmd"],
                std::time::Duration::from_secs(30),
                |code, down, _| {
                    let event = match (code, down) {
                        (55, true) => "modifier_down",
                        (55, false) => "modifier_up",
                        (_, true) => "base_down",
                        (_, false) => "base_up",
                    };
                    worker_events.lock().unwrap().push(event);
                    if down && ((code == 55 && !base_down) || (code != 55 && base_down)) {
                        ready.take().unwrap().send(()).unwrap();
                        resume_rx.recv().unwrap();
                    }
                    Ok(())
                },
                || control.is_cancelled(),
                |_| control.is_cancelled(),
            );
            assert!(result.is_err());
            let _ = done_tx.send(());
        };
        let invoking = tokio::spawn(async move {
            if hotkey {
                super::super::hotkey::run_hotkey_owned(lease, scope, control, work).await
            } else {
                run_owned(lease, scope, control, work).await
            }
        });
        ready_rx.await.expect("worker reached the barrier");
        invoking.abort();
        assert!(matches!(invoking.await, Err(error) if error.is_cancelled()));
        let admitted_early = crate::background_mutation::try_acquire(pid);
        let was_admitted_early = admitted_early.is_some();
        drop(admitted_early);
        resume_tx.send(()).unwrap();
        done_rx.await.expect("worker completed cleanup");
        let final_lease = crate::background_mutation::acquire(pid).await;
        let events = events.lock().unwrap().clone();
        let mut expected = vec!["modifier_down"];
        if base_down {
            expected.extend(["base_down", "base_up"]);
        }
        expected.extend(["modifier_up", "hand_back", "lease_release"]);
        assert_eq!(
            (events, was_admitted_early),
            (expected, false),
            "cleanup must precede hand-back and lease release; same-pid work must wait"
        );
        drop(final_lease);
    }

    #[tokio::test]
    async fn cancelling_invoking_future_after_modifier_down_releases_before_hand_back() {
        cancel_invoking_future(false, -92_001, false).await;
    }

    #[tokio::test]
    async fn cancelling_invoking_future_after_base_down_releases_before_hand_back() {
        cancel_invoking_future(true, -92_002, false).await;
    }

    #[tokio::test]
    async fn cancelling_hotkey_future_after_modifier_down_releases_before_hand_back() {
        cancel_invoking_future(false, -92_003, true).await;
    }

    #[tokio::test]
    async fn cancelling_hotkey_future_after_base_down_releases_before_hand_back() {
        cancel_invoking_future(true, -92_004, true).await;
    }

    #[tokio::test]
    async fn cancelling_invoking_future_signals_and_wakes_a_pending_hold() {
        use crate::input::key_hold::HoldControl;
        let control = Arc::new(HoldControl::default());
        let (ready_tx, ready_rx) = tokio::sync::oneshot::channel();
        let (done_tx, done_rx) = tokio::sync::oneshot::channel();
        let invoking = tokio::spawn(run_owned(
            (),
            (),
            control.clone(),
            move |worker_control, _scope| {
                let _ = ready_tx.send(());
                let cancelled = worker_control.wait(std::time::Duration::from_secs(30));
                let _ = done_tx.send(cancelled);
            },
        ));
        ready_rx.await.unwrap();
        invoking.abort();
        assert!(matches!(invoking.await, Err(error) if error.is_cancelled()));
        let signalled_by_drop = control.is_cancelled();
        // Ensure the baseline worker can finish even when the signal is absent.
        control.cancel();
        assert!(done_rx.await.unwrap());
        assert!(
            signalled_by_drop,
            "dropping the invoking future did not wake the hold"
        );
    }
}

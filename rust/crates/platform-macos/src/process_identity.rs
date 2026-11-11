//! Kernel-backed process instance identity for protected snapshots.

use cua_driver_core::browser::ProcessFingerprint;
use cua_driver_core::snapshot_store::{
    InstanceCapture, InstanceEvidence, SnapshotPayload, SnapshotStore,
};

use crate::ax::enablement::{process_start_stamp_with, read_process_info};

trait ProcessReader {
    fn info(&self, pid: i32, info: &mut libc::proc_bsdinfo) -> libc::c_int;
    fn executable(&self, pid: i32) -> Option<String>;
}

struct KernelReader;

impl ProcessReader for KernelReader {
    fn info(&self, pid: i32, info: &mut libc::proc_bsdinfo) -> libc::c_int {
        read_process_info(pid, info)
    }

    fn executable(&self, pid: i32) -> Option<String> {
        crate::apps::path_of_pid(pid)
    }
}

pub(crate) fn process_fingerprint(pid: i64) -> Option<ProcessFingerprint> {
    fingerprint_with(pid, &KernelReader)
}

/// #299: opaque label of the process instance `pid` names now (its kernel
/// start time in microseconds), for answers that report a pid. Compared for
/// equality only; `None` when the start time could not be read twice alike.
pub(crate) fn instance_label(pid: i32) -> Option<String> {
    process_fingerprint(i64::from(pid))?
        .start_time
        .map(|start| start.to_string())
}

/// The label a read's instance capture reports: only a capture that proved
/// one process start for the whole read names it.
pub(crate) fn capture_label(capture: &InstanceCapture) -> Option<String> {
    match capture {
        InstanceCapture::Captured(fingerprint) => {
            fingerprint.start_time.map(|start| start.to_string())
        }
        InstanceCapture::Failed(_) => None,
    }
}

/// The process instance a read observed: the fingerprints taken before and
/// after it must name the same start of `pid`. Anything else is a failed
/// capture, which the store keeps protected (its tokens refuse) until a fresh
/// read replaces it.
pub(crate) fn instance_capture(
    pid: i32,
    before: Option<ProcessFingerprint>,
    after: Option<ProcessFingerprint>,
) -> InstanceCapture {
    match (before, after) {
        (Some(before), Some(after))
            if before.pid == i64::from(pid)
                && before.start_time.is_some()
                && before.matches(&after) =>
        {
            InstanceCapture::Captured(after)
        }
        (Some(_), Some(_)) => InstanceCapture::Failed("the process changed during the read".into()),
        _ => InstanceCapture::Failed("process identity could not be read".into()),
    }
}

/// Live evidence for an element-token action on `pid`, read off the async
/// runtime. A call without a token resolves nothing and needs none.
pub(crate) async fn evidence(args: &serde_json::Value, pid: i32) -> InstanceEvidence {
    if args.get("element_token").is_none() {
        return InstanceEvidence::NotChecked;
    }
    match crate::goat::action_latency::spawn_blocking(move || {
        process_fingerprint(i64::from(pid))
    })
    .await
    {
        Ok(Some(fingerprint)) => InstanceEvidence::Live(fingerprint),
        Ok(None) => InstanceEvidence::Unavailable("process identity could not be read".into()),
        Err(error) => {
            InstanceEvidence::Unavailable(format!("process identity task failed: {error}"))
        }
    }
}

/// Publish a window read's element snapshot with the process instance it was
/// read from, so a token from it refuses once that process is gone.
#[allow(clippy::too_many_arguments)]
pub(crate) fn publish_read<S: SnapshotPayload>(
    store: &SnapshotStore<S>,
    pid: i32,
    window_id: u64,
    payload: S,
    session: Option<&str>,
    screenshot_scale: Option<f64>,
    window_size: Option<(f64, f64)>,
    capture: InstanceCapture,
) -> Option<(u32, Vec<u32>)> {
    store.publish_sized_for_session_with_instance(
        pid,
        window_id,
        payload,
        session,
        screenshot_scale,
        window_size,
        capture,
    )
}

fn fingerprint_with(pid: i64, reader: &impl ProcessReader) -> Option<ProcessFingerprint> {
    let native_pid = i32::try_from(pid).ok().filter(|pid| *pid > 0)?;
    let (seconds, micros) =
        process_start_stamp_with(native_pid, |pid, info| reader.info(pid, info))?;
    if micros >= 1_000_000 {
        return None;
    }
    let start_time = seconds.checked_mul(1_000_000)?.checked_add(micros)?;
    let executable = reader.executable(native_pid);
    let after = process_start_stamp_with(native_pid, |pid, info| reader.info(pid, info))?;
    if after != (seconds, micros) {
        return None;
    }
    Some(ProcessFingerprint {
        pid,
        start_time: Some(start_time),
        executable,
    })
}

#[cfg(test)]
mod u33_identity_tests {
    use super::*;
    use std::cell::{Cell, RefCell};
    use std::collections::VecDeque;

    type InfoResult = Option<(i32, u64, u64)>;

    struct FakeReader {
        info: RefCell<VecDeque<InfoResult>>,
        executable: Option<String>,
        calls: Cell<usize>,
    }

    impl FakeReader {
        fn new(info: &[InfoResult], executable: Option<&str>) -> Self {
            Self {
                info: RefCell::new(info.iter().copied().collect()),
                executable: executable.map(str::to_owned),
                calls: Cell::new(0),
            }
        }
    }

    impl ProcessReader for FakeReader {
        fn info(&self, _pid: i32, info: &mut libc::proc_bsdinfo) -> libc::c_int {
            self.calls.set(self.calls.get() + 1);
            match self
                .info
                .borrow_mut()
                .pop_front()
                .expect("unexpected info read")
            {
                Some((length, sec, usec)) => {
                    info.pbi_start_tvsec = sec;
                    info.pbi_start_tvusec = usec;
                    length
                }
                None => 0,
            }
        }
        fn executable(&self, _pid: i32) -> Option<String> {
            self.executable.clone()
        }
    }

    fn full(sec: u64, usec: u64) -> InfoResult {
        Some((std::mem::size_of::<libc::proc_bsdinfo>() as i32, sec, usec))
    }

    fn stable(sec: u64, usec: u64, exe: Option<&str>) -> FakeReader {
        FakeReader::new(&[full(sec, usec), full(sec, usec)], exe)
    }

    #[test]
    fn packs_seconds_and_microseconds() {
        assert_eq!(
            fingerprint_with(42, &stable(12, 345_678, None))
                .unwrap()
                .start_time,
            Some(12_345_678)
        );
    }

    #[test]
    fn distinguishes_starts_within_one_second() {
        let a = fingerprint_with(42, &stable(12, 1, Some("/a"))).unwrap();
        let b = fingerprint_with(42, &stable(12, 2, Some("/a"))).unwrap();
        assert_ne!(a.start_time, b.start_time);
    }

    #[test]
    fn rejects_out_of_range_microseconds() {
        assert_eq!(fingerprint_with(42, &stable(12, 1_000_000, None)), None);
    }

    #[test]
    fn rejects_start_time_overflow() {
        assert_eq!(fingerprint_with(42, &stable(u64::MAX, 1, None)), None);
        assert_eq!(
            fingerprint_with(
                42,
                &stable(u64::MAX / 1_000_000, u64::MAX % 1_000_000 + 1, None)
            ),
            None
        );
    }

    #[test]
    fn rejects_short_proc_info() {
        let size = std::mem::size_of::<libc::proc_bsdinfo>() as i32;
        let reader = FakeReader::new(&[Some((size - 1, 12, 345))], None);
        assert_eq!(fingerprint_with(42, &reader), None);
    }

    #[test]
    fn invalid_pid_is_not_coerced() {
        for pid in [0, -1, i64::MAX] {
            let reader = FakeReader::new(&[], None);
            assert_eq!(fingerprint_with(pid, &reader), None);
            assert_eq!(reader.calls.get(), 0);
        }
    }

    #[test]
    fn unreadable_start_has_no_identity() {
        assert_eq!(
            fingerprint_with(42, &FakeReader::new(&[None], Some("/a"))),
            None
        );
        assert_eq!(
            fingerprint_with(42, &FakeReader::new(&[full(12, 1), None], Some("/a"))),
            None
        );
    }

    #[test]
    fn executable_is_optional_and_exact() {
        assert_eq!(
            fingerprint_with(42, &stable(12, 1, None))
                .unwrap()
                .executable,
            None
        );
        let path = "/Applications/Example.app/Contents/MacOS/Example";
        assert_eq!(
            fingerprint_with(42, &stable(12, 1, Some(path)))
                .unwrap()
                .executable
                .as_deref(),
            Some(path)
        );
    }

    #[test]
    fn stamp_change_during_executable_read_is_unavailable() {
        let reader = FakeReader::new(&[full(12, 1), full(12, 2)], Some("/a"));
        assert_eq!(fingerprint_with(42, &reader), None);
    }

    struct Rows(Vec<u64>);

    impl SnapshotPayload for Rows {
        type Element = u64;
        fn len(&self) -> usize {
            self.0.len()
        }
        fn retain(&self, index: usize) -> Option<u64> {
            self.0.get(index).copied()
        }
    }

    fn started(start_time: u64) -> ProcessFingerprint {
        ProcessFingerprint {
            pid: 42,
            start_time: Some(start_time),
            executable: Some("/Applications/App.app/Contents/MacOS/App".into()),
        }
    }

    /// AD-G2 / #299: a token minted from a read of one process start refuses
    /// once the pid names another start (the app quit and the pid was reused).
    #[test]
    fn a_token_from_an_earlier_start_of_the_pid_is_refused() {
        let store = SnapshotStore::new();
        let capture = instance_capture(42, Some(started(1)), Some(started(1)));
        let (id, _) = publish_read(&store, 42, 7, Rows(vec![10]), Some("s"), None, None, capture)
            .unwrap();
        let args = serde_json::json!({
            "element_token": cua_driver_core::element_token::token_for(id, 0),
            "_session_id": "s",
        });
        assert!(store
            .resolve_for_instance(42, &args, InstanceEvidence::Live(started(1)))
            .is_ok());
        let refused = store
            .resolve_for_instance(42, &args, InstanceEvidence::Live(started(2)))
            .err()
            .expect("a restarted process refuses the old token");
        assert_eq!(
            refused.structured_content.unwrap()["refusal"]["code"],
            "process_instance_mismatch"
        );
    }

    #[test]
    fn a_read_that_spans_a_restart_is_a_failed_capture() {
        assert!(matches!(
            instance_capture(42, Some(started(1)), Some(started(2))),
            InstanceCapture::Failed(_)
        ));
        assert!(matches!(
            instance_capture(42, None, Some(started(1))),
            InstanceCapture::Failed(_)
        ));
        let other_pid = ProcessFingerprint { pid: 43, ..started(1) };
        assert!(matches!(
            instance_capture(42, Some(other_pid.clone()), Some(other_pid)),
            InstanceCapture::Failed(_)
        ));
    }
}

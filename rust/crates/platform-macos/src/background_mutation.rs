//! Per-process serialization for exact-target background mutations.
//!
//! macOS keyboard delivery and focus suppression are process-scoped. Two
//! concurrent window-addressed mutations for one pid must not interleave their
//! fresh target proof, focus changes, dispatch, restoration, or verification.
//! Different processes remain independent.

use std::collections::HashMap;
use std::future::Future;
use std::sync::{Arc, Mutex, OnceLock, Weak};
use tokio::sync::{Mutex as AsyncMutex, OwnedMutexGuard};

tokio::task_local! {
    static HELD_CONTEXT: (crate::tools::BackgroundMutationLease, cua_driver_core::mutation::PriorInput);
    static PRIOR_CONTEXT: (i32, cua_driver_core::mutation::PriorInput);
}

fn process_locks() -> &'static Mutex<HashMap<i32, Weak<AsyncMutex<()>>>> {
    static LOCKS: OnceLock<Mutex<HashMap<i32, Weak<AsyncMutex<()>>>>> = OnceLock::new();
    LOCKS.get_or_init(|| Mutex::new(HashMap::new()))
}

fn process_lock(pid: i32) -> Arc<AsyncMutex<()>> {
    let mut locks = process_locks()
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    locks.retain(|_, lock| lock.strong_count() > 0);
    if let Some(lock) = locks.get(&pid).and_then(Weak::upgrade) {
        return lock;
    }
    let lock = Arc::new(AsyncMutex::new(()));
    locks.insert(pid, Arc::downgrade(&lock));
    lock
}

/// Acquire exclusive background-mutation ownership for one process.
///
/// The returned guard must live from immediately before fresh fact gathering
/// until dispatch, focus restoration, and postcondition verification finish.
pub(crate) async fn acquire(pid: i32) -> OwnedMutexGuard<()> {
    process_lock(pid).lock_owned().await
}

/// Take this process's mutation lease only if no action holds it, without
/// waiting. An action that starts meanwhile waits until the guard drops.
pub(crate) fn try_acquire(pid: i32) -> Option<OwnedMutexGuard<()>> {
    process_lock(pid).try_lock_owned().ok()
}

/// Run one nested tool call with non-forgeable proof that its caller already
/// owns this process's mutation lease.
pub(crate) async fn with_held_lease<T>(
    lease: crate::tools::BackgroundMutationLease,
    prior: cua_driver_core::mutation::PriorInput,
    future: impl Future<Output = T>,
) -> T {
    HELD_CONTEXT.scope((lease, prior), future).await
}

pub(crate) fn held_by_current_task(pid: i32) -> bool {
    HELD_CONTEXT
        .try_with(|(lease, _)| lease.pid() == pid)
        .unwrap_or(false)
}

pub(crate) fn current_lease(pid: i32) -> Option<crate::tools::BackgroundMutationLease> {
    HELD_CONTEXT
        .try_with(|(lease, _)| (lease.pid() == pid).then(|| lease.clone()))
        .ok()
        .flatten()
}

pub(crate) fn prior_for_current_task(pid: i32) -> cua_driver_core::mutation::PriorInput {
    HELD_CONTEXT
        .try_with(|(lease, prior)| (lease.pid() == pid).then_some(*prior))
        .ok()
        .flatten()
        .or_else(|| {
            PRIOR_CONTEXT
                .try_with(|(context_pid, prior)| (*context_pid == pid).then_some(*prior))
                .ok()
                .flatten()
        })
        .unwrap_or(cua_driver_core::mutation::PriorInput::None)
}

pub(crate) async fn with_input_prior<T>(
    pid: i32,
    prior: cua_driver_core::mutation::PriorInput,
    future: impl Future<Output = T>,
) -> T {
    PRIOR_CONTEXT.scope((pid, prior), future).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn a_lease_is_taken_without_waiting_only_when_free() {
        let held = acquire(-91_101).await;
        assert!(try_acquire(-91_101).is_none());
        drop(held);
        let taken = try_acquire(-91_101).expect("a free lease is taken");
        // An action starting now waits for the guard.
        let mut waiting = tokio::spawn(acquire(-91_101));
        assert!(
            tokio::time::timeout(Duration::from_millis(30), &mut waiting)
                .await
                .is_err()
        );
        drop(taken);
        drop(waiting.await.unwrap());
    }

    #[tokio::test]
    async fn same_pid_mutations_serialize_until_the_first_guard_drops() {
        let first = acquire(-91_001).await;
        let mut waiting = tokio::spawn(acquire(-91_001));

        assert!(
            tokio::time::timeout(Duration::from_millis(30), &mut waiting)
                .await
                .is_err(),
            "a second mutation for the same pid must remain queued"
        );
        drop(first);

        let second = tokio::time::timeout(Duration::from_secs(1), waiting)
            .await
            .expect("same-pid waiter must proceed after the first guard drops")
            .expect("same-pid waiter task must not panic");
        drop(second);
    }

    #[tokio::test]
    async fn different_pids_do_not_block_each_other() {
        let first = acquire(-91_002).await;
        let second = tokio::time::timeout(Duration::from_millis(100), acquire(-91_003))
            .await
            .expect("independent pids must acquire concurrently");
        drop((first, second));
    }

    #[tokio::test]
    async fn nested_lease_proof_is_task_local_and_pid_bound() {
        assert!(!held_by_current_task(-91_004));
        let lease = crate::tools::acquire_background_mutation(-91_004).await;
        with_held_lease(lease, cua_driver_core::mutation::PriorInput::None, async {
            assert!(held_by_current_task(-91_004));
            assert!(!held_by_current_task(-91_005));
        })
        .await;
        assert!(!held_by_current_task(-91_004));
    }

    #[tokio::test(flavor = "current_thread")]
    async fn input_history_without_a_lease_stays_task_local_and_pid_bound() {
        use cua_driver_core::mutation::PriorInput;
        with_input_prior(-91_007, PriorInput::MayHaveDelivered, async {
            assert!(!held_by_current_task(-91_007));
            assert_eq!(
                prior_for_current_task(-91_007),
                PriorInput::MayHaveDelivered
            );
            assert_eq!(prior_for_current_task(-91_008), PriorInput::None);
        })
        .await;
        assert_eq!(prior_for_current_task(-91_007), PriorInput::None);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn nested_lease_survives_caller_cancellation_until_worker_ends() {
        let pid = -91_006;
        let lease = crate::tools::acquire_background_mutation(pid).await;
        let (started_tx, started_rx) = tokio::sync::oneshot::channel();
        let (done_tx, done_rx) = tokio::sync::oneshot::channel();
        let (release_tx, release_rx) = std::sync::mpsc::channel();
        let caller = tokio::spawn(with_held_lease(
            lease,
            cua_driver_core::mutation::PriorInput::None,
            async move {
                let held = current_lease(pid).expect("nested lease");
                let worker = crate::goat::action_latency::spawn_action_blocking(move || {
                    let held = held;
                    let _ = started_tx.send(());
                    release_rx.recv().expect("release worker");
                    drop(held);
                    let _ = done_tx.send(());
                });
                let _ = worker.await;
            },
        ));
        started_rx.await.expect("worker started");
        caller.abort();
        assert!(caller.await.unwrap_err().is_cancelled());
        assert!(try_acquire(pid).is_none());
        release_tx.send(()).expect("release worker");
        done_rx.await.expect("worker finished");
        assert!(try_acquire(pid).is_some());
    }
}

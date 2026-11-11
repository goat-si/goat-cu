# 020 — wp2: process identity and session scope (#299, AD-G2, AD-G3, #288)

> Re-validated on the merged tree in 012: §B is wiring only (store already has `publish_*_with_instance` and `resolve_for_instance`), must also cover `refresh_screenshot_for_session` (get_window_state.rs:937), and the ten resolve sites are listed there with current lines.

Stale check at wp2 P: re-locate every cited symbol on the synced tree (wp1 merged `apps/mod.rs`, `get_window_state.rs`,
`tools/mod.rs`). Class C3 (protocol-additive field + cross-session state).

## §A — #299/AD-G3: `process_instance` beside `pid` (macOS)

Field: `process_instance` — opaque string, compare for equality only; macOS value is the kernel start time in microseconds
(`pbi_start_tvsec * 1_000_000 + pbi_start_tvusec`, decimal). Omitted when unreadable or when the stamp changed during the read.

- MODIFY `rust/crates/platform-macos/src/process_identity.rs`: drop the dead-code allow on `process_fingerprint`; add

```rust
/// Opaque per-process instance label for answers that name a pid. Equality only.
pub(crate) fn instance_label(pid: i32) -> Option<String> {
    fingerprint_with(i64::from(pid), &KernelReader)?.start_time.map(|t| t.to_string())
}
```

- MODIFY `rust/crates/platform-macos/src/apps/mod.rs` `AppInfo`: add
  `#[serde(skip_serializing_if = "Option::is_none")] pub process_instance: Option<String>,`; running-app constructors set
  `process_instance: before.filter(|b| crate::process_identity::instance_label(pid).as_ref() == Some(b))` where `before` is
  read before the entry's name/bundle reads; installed-only entries set `None`.
- MODIFY `tools/list_apps.rs` `app_record_json`: add `"process_instance": a.process_instance` (null is stripped).
- MODIFY `tools/list_windows.rs` window record: `"process_instance"` per distinct pid, read before and after the window
  enumeration, kept only when equal.
- MODIFY `tools/get_window_state.rs` structured output: `"process_instance"` from the same capture §B uses.
- Tool descriptions mention the field; `docs/tool-output-format.md` gets one line.
- Tests (failing first): `tools/list_apps.rs` tests module — `a_running_app_record_carries_its_process_instance`
  (AppInfo with `process_instance: Some("17")` serializes it; `None` omits it). `process_identity.rs` `u33_identity_tests` —
  `instance_label_is_none_when_the_start_stamp_changes` (FakeReader returns two different stamps).
- Linux/Windows: `proc_fs::process_instance_id` and `process_instance::process_fingerprint` exist; wiring them is the same
  shape but cannot be compiled here. Recorded as follow-up, not claimed.

## §B — AD-G2: bind macOS element snapshots to the process instance (U33B)

Mirror `platform-windows/src/process_instance.rs` (U15 #260):

- MODIFY `process_identity.rs`: add `pub(crate) fn capture(pid: i32) -> InstanceCapture` (fingerprint or
  `Failed("process identity could not be read")`), and
  `pub(crate) async fn evidence(args: &Value, pid: i32) -> InstanceEvidence` (`NotChecked` without `element_token`; otherwise
  `spawn_blocking(process_fingerprint)` → `Live` / `Unavailable`).
- MODIFY `get_window_state.rs` publication (`publish_sized_for_session` at 40c6f6eb :956): capture before the AX walk; after the
  walk re-capture; equal → `publish_sized_for_session_with_instance(..., InstanceCapture::Captured(fp))`, else
  `Failed("process changed during the read")`.
- MODIFY the ten resolve sites (002 #299 list): `let evidence = crate::process_identity::evidence(&args, pid).await;` then
  `resolve_for_instance(pid, &args, evidence)` inside the existing `latency::resolve` closure.
- Tests (failing first, deterministic, no live app): in `process_identity.rs` tests, publish a snapshot with captured
  fingerprint A into a `SnapshotStore<AxSnapshot>` and resolve with `Live(B)` → refusal code `process_instance_mismatch`
  (today the macOS path calls `resolve`, which never checks; the failing assertion is on a helper
  `resolve_token(store, pid, args, evidence)` that the tools will call).

## §C — #288: session-scoped fast-path store

- MODIFY `rust/crates/cua-driver-core/src/fast_path/store.rs`: `SnapshotFingerprints` gains `session: Option<String>`; images
  and windows are keyed `(Option<String>, i32, u64)`. Lookups `element`, `stats`, `latest_stats`, `element_at`, `window`, `image`
  take `session: Option<&str>` and return `None` for an entry another session published. `forget_window` drops all sessions'
  entries for the window.
- MODIFY `fast_path/wire.rs` `skip_reread_screenshot` / `delta_target`: pass `args["_session_id"]`.
- MODIFY `platform-macos/src/fast_path_guard.rs` publish (:77, :108, :134) and lookups (:395, :510): thread the session id the
  tool already has in `args`.
- Test (failing first): `store.rs` tests — `another_sessions_snapshot_is_not_visible`: put_snapshot(id 1, session "a"),
  `element(1, 0, Some("b"))` is `None`, `element(1, 0, Some("a"))` is `Some`; same for `element_at` and `image`.

## Verification

`cargo test -p cua-driver-core --lib fast_path`, `cargo test -p platform-macos --lib process_identity list_apps list_windows`,
`cargo check -p platform-macos --all-targets`, protocol check (B4 of 010) listing `process_instance` as additive output.

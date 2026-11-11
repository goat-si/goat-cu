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

## wp2 P (on committed wp1 HEAD 20449d9f) — settled design

Core facts (cua-driver-core/src/snapshot_store.rs): `instance_failure` (:44-69) refuses a token on a snapshot published with an
instance unless live evidence matches: (Captured, Live) compares `ProcessFingerprint::matches` (pid, start time, executable when both
present); (Captured, NotChecked | Unavailable) and (Failed, _) refuse `process_instance_unverified`. A token-free call returns
`ResolvedElement::None` before any instance check (:1319-1324). So once macOS publishes with an instance, every macOS token
resolution must pass live evidence: the ten `snapshots.resolve(pid, &args)` sites (012) are the complete inventory
(`pid_for_token`, `token_owner`, zoom and session-retire calls read metadata only; macOS has no rooted reads).

§B implementation:
- process_identity.rs: `instance_capture(pid, before, after) -> InstanceCapture` (Captured only when both reads exist, name
  `pid`, carry a start time and `matches`), `evidence(args, pid) -> InstanceEvidence` (NotChecked without `element_token`;
  otherwise `process_fingerprint` off the runtime via `action_latency::spawn_blocking`), `instance_label(pid)` for §A.
- get_window_state.rs: fingerprint before the walk (beside the prefetched capture), again before publication;
  `publish_sized_for_session` → `publish_sized_for_session_with_instance(.., instance_capture(..))`. The screenshot-only
  `refresh_screenshot_for_session` keeps the snapshot's original capture; a token from a restarted process is still refused at
  resolve (mismatch), so no change there.
- the ten resolve sites: `let evidence = crate::process_identity::evidence(&args, pid).await;` then `resolve_for_instance`.
§A: `process_instance` (decimal kernel start time in µs, read twice, omitted when unreadable or changed) on list_apps records,
list_windows records and the get_window_state structured answer; additive output (protocol record in D).
§C (#288): dispatched to a child (core fast_path store + platform-macos fast_path_guard.rs; disjoint from §A/§B files).
Failing-first: (1) process_identity test publishing through the macOS publish helper and resolving with a different live instance
must refuse `process_instance_mismatch` (helper first written with today's unprotected publish → test fails); (2) list_apps
record carries `process_instance`.

## wp2 B/C evidence — §A/§B (#299, AD-G2), lead-implemented

Changes: process_identity.rs (`instance_capture`, `evidence`, `publish_read`, `instance_label`, `capture_label`; dead-code allow
removed); get_window_state.rs (fingerprint before the walk and before publication → `publish_read` with the capture; structured
`process_instance` from `capture_label`); the ten resolve sites → `evidence(&args, pid).await` + `resolve_for_instance`
(click.rs:1540, scroll.rs:233, set_value.rs:132, double_click.rs:132, type_text.rs:332, press_key.rs:736, hotkey.rs:334,
select_text.rs:355, type_text_chars.rs:228, right_click.rs:316); apps/mod.rs `AppInfo.process_instance`; list_apps.rs
`label_running_apps` in the inventory task + record field; list_windows.rs `label_window_records`; cua-driver-contract
windows.rs `process_instance_schema` on list_apps items, list_windows items, get_window_state; docs/tool-output-format.md.

| Command (rust/) | Exit | Result |
|---|---|---|
| `cargo test -p platform-macos --lib process_identity` (publish_read still unprotected) | 101 | `a_token_from_an_earlier_start_of_the_pid_is_refused` panicked at process_identity.rs:286 ("a restarted process refuses the old token") |
| same, after the protected publish | 0 | 24 passed |
| `cargo test -p cua-driver-contract --lib answers_naming_a_pid` (before the schema) | 101 | `answers_naming_a_pid_document_its_process_instance` FAILED |
| `cargo test -p platform-macos --lib -- process_instance` (before the helpers) | 101 | E0425 `label_running_apps`, `label_window_records` not found |
| `cargo test -p cua-driver-contract --lib` after | 0 | 91 passed |
| `cargo test -p platform-macos --lib -- process_instance list_apps list_windows process_identity` after | 0 | 39 passed, 1 ignored |
| `cargo test -p platform-macos --lib` (after §B wiring) | 0 | 2002 passed, 48 ignored |
| `cargo check -p platform-macos -p cua-driver-contract -p cua-driver-core -p cua-driver --all-targets` | 0 | no warnings |
| (repo) check_protocol.py vs frozen baseline | 0 | compatible in every launch form |
| (repo) proto_delta vs 0.40.1-goat.1 | 0 | 65 = 65 tools, inputs unchanged, problems 0 |
| (repo) output-schema delta vs 0.40.1-goat.1 (/tmp/goat-cu-sync/wp2-output-delta.log) | 0 | both launch forms: list_apps items, list_windows items, get_window_state each add `process_instance`; nothing removed |

Protocol record (c-3): additive output only — `process_instance` (string, optional) on three tools; no `allowed-changes.json` entry
(check_protocol.py reports no non-additive change). Linux/Windows answers do not report it yet (Linux `proc_fs::process_instance_id`
and Windows `process_instance::process_fingerprint` exist; wiring them is a follow-up, not claimed here).

## wp2 B/C evidence — §C (#288), child-implemented, lead-audited

Child `cu-wp2-288-s24-261112` (codex / system24/deepseek-v4.1-flash / high). Lead read the full diff:

- `cua-driver-core/src/fast_path/store.rs`: `SnapshotFingerprints.session`; images/windows keyed `(Option<String>, pid, window)`
  (`WindowKey`, `window_key`, `same_window`); `element`, `stats`, `latest_stats`, `element_at`, `window`, `image` match only the
  caller's session (`None` matches only `None`); `forget_window` drops the window for every session.
- `fast_path/wire.rs`: `session_of(args)` reads `args["_session_id"]`, used by `skip_reread_screenshot` and `delta_target`.
- `platform-macos/src/fast_path_guard.rs`: `record_snapshot` / `record_read_image` / `record_window` store the session;
  `guard_element` looks up with `wire::session_of(args)`; `guard_pixel` takes the session (click.rs:2587 passes `args["_session_id"]`).
- `get_window_state.rs` records under `session_id = args.opt_str("_session_id")` (:483), the same daemon-stamped key the snapshot
  store already scopes by (`snapshot_store.rs` resolve paths). A caller-supplied `_session_id` is never trusted:
  `tool_args::apply_session_identity` (:120) replaces it with the call's effective session (its public `session` label, else the
  transport session) and `sanitize_reserved_args` (:104) drops other `_`-prefixed keys. Fast-path scoping is therefore exactly as
  strong as snapshot-token scoping: same key, same derivation.
- Lease-wiring source inventory: the two read-only `spawn_blocking` sites whose closure text changed (`fast_path_guard.rs::guard_pixel`
  now passes the session; `tools/list_apps.rs::inventory_result` now also reads process-instance labels for §A) re-pinned in
  `tools/lease_wiring_tests/source_exemptions.rs` (fingerprints 11082233245211704923 and 399154389028660781), category still Read.

| Command (rust/) | Exit | Result |
|---|---|---|
| child: `cargo test -p cua-driver-core --lib fast_path::store` before the store change | 101 | compile red (E0560 `session` field, E0061 arity) |
| lead: same test with the session filter neutralised (filters → `true`, keys → `None`), fix restored after (`cmp` identical) | 101 | `another_sessions_snapshot_is_not_visible` panicked at store.rs:348 `store.element(1, 0, Some("b")).is_none()` |
| `cargo test -p cua-driver-core --lib fast_path` | 0 | 55 passed |
| `cargo test -p platform-macos --lib fast_path_guard` | 0 | 1 passed |
| `cargo test -p platform-macos --lib lease_wiring` before the re-pin | 101 | 2 failed (`guard_pixel`, `inventory_result` fingerprints) |
| same, after the re-pin | 0 | 22 passed |

Pre-receipt dry run of `/tmp/goat-cu-sync/receipt-wp2.sh` on the uncommitted wp2 tree (not a recorded receipt): failures=0 —
kache doctor 0; check (macos/core/contract/driver, all targets) 0, no warnings; core fast_path 55; core snapshot_store 74; contract 91;
macOS lib 2004 passed / 48 ignored; Windows host 203; build-driver 0; check_protocol compatible in every launch form; proto_delta 65 = 65,
problems 0; output_delta (`/tmp/goat-cu-sync/output_delta.py`) adds only `process_instance` on get_window_state, list_apps items and
list_windows items, nothing removed. wp2 committed locally ahead of the two reviews so wp1b/wp3 can proceed; any accepted finding lands
as a follow-up commit with its own failing-first test, and the recorded C receipt runs after the reviews.

## Verification

## #299 independent review (`cu-wp2-review-ab-s24-261112`, on e0851769) — NEAR-PASS, 4 findings, all accepted

| # | Sev | Finding | Fix (follow-up commit) |
|---|---|---|---|
| 1 | P2 | recording_hooks.rs:78 resolved the caller's token with `resolve` (= `NotChecked`); every snapshot now carries a capture, so element-addressed recordings silently lost click.png | `process_identity::resolve_now` (:103) reads live evidence on the calling thread; the hook (recording_hooks.rs:80) uses it |
| 2 | P2 | list_windows labelled on the async runtime (and its whole listing, AX included, ran there) | enumeration, composition and labels in one `tokio::task::spawn_blocking` (list_windows.rs:65); lease-wiring exemption added (Read) |
| 3 | P3 | a screenshot-only refresh of an old snapshot reported this read's label beside that snapshot's tokens | `reported_label` (:56): no label when the read refreshed an existing snapshot (get_window_state.rs uses `refreshed.is_some()`) |
| 4 | P3 | labels were read once after the listing, so a pid reused between enumeration and labelling paired the old record with the new label | `labels_before` (:69, one start-stamp read per pid before the listing, all pids or the filtered pid) + `bracketed_label` (:77): label only when before == after; list_apps.rs:54 and list_windows.rs:65 |

| Command (rust/) | Exit | Result |
|---|---|---|
| `cargo test -p platform-macos --lib -- a_synchronous_resolve_reads_live_evidence a_refreshed_snapshot_reports_no_new_label a_pid_that_restarts_during_a_listing_gets_no_label` (helpers mirroring the committed behaviour) | 101 | 3 failed: process_identity.rs:404 `resolve_now` not `Ok(Element)`; :427 `Some("1")` for a refreshed snapshot; :440 `Some("2")` across a restart |
| `cargo test -p platform-macos --lib -- process_identity list_apps list_windows recording_hooks lease_wiring get_window_state` before the inventory update | 101 | 2 lease-wiring failures (list_apps fingerprint, new list_windows site) |
| same, after | 0 | 152 passed, 1 ignored |
| `cargo check -p platform-macos --all-targets` | 0 | no warnings |

## #288 independent review (`cu-wp2-review-288-s24-261112`, on e0851769) — NEAR-PASS, 4 findings

| # | Sev | Finding | Disposition |
|---|---|---|---|
| 1 | P2 | `delta_target` (wire.rs) runs in `invoke_authorized` (tool.rs:2105) on args whose `_session_id` was already stripped, while `get_window_state` records under the namespaced runtime session (tool.rs:2320-2331): with `fast_path=on` an element-token action without `window_id` silently lost its delta | ACCEPTED, fixed: `GuardStore::snapshot_window` maps the token's snapshot to its window for this purpose only; safe because the delta is attached only after the action succeeded, i.e. after the snapshot store resolved the token in the caller's session, and snapshot ids are daemon-unique; the follow-up read is authorized as the caller. Fingerprint lookups stay session-scoped |
| 2 | P3 | eviction paths untested with mixed sessions | ACCEPTED: `evictions_keep_each_entry_in_its_own_session` (image capacity, window retain past 4×capacity, snapshot capacity) — coverage only, passes on the fix |
| 3 | P3 | `forget_window` has no production caller | ACCEPTED as a record correction: pre-existing on origin/main 40c6f6eb (no caller there either); the all-sessions clear is a store property, runtime replacement is still caught by the guard's pid/frame/visibility checks. Not wired here (new behaviour beyond #288) |
| 4 | P3 | `record_window` stores scale 0.0 for a session without an image | REBUTTED: 0.0 means "this session read no image" instead of borrowing another session's scale (the #288 leak); no consumer divides by it — `guard_element` and the non-pixel branches pass `relocate_scale` 1.0 (fast_path_guard.rs:548, :556), the pixel branch uses the image's own scale `.max(1.0)` (:558), `window_observation` echoes `want.scale` |

| Command (rust/) | Exit | Result |
|---|---|---|
| `cargo test -p cua-driver-core --lib -- an_element_token_delta_finds_its_window_before_the_session_is_named` (before the fix) | 101 | wire.rs:345 `left: None, right: Some((77, 9))` |
| `cargo test -p cua-driver-core --lib fast_path` after | 0 | 57 passed |
| `cargo test -p platform-macos --lib fast_path_guard` | 0 | 1 passed |
| `cargo check -p cua-driver-core -p platform-macos --all-targets` | 0 | no warnings |

`cargo test -p cua-driver-core --lib fast_path`, `cargo test -p platform-macos --lib process_identity list_apps list_windows`,
`cargo check -p platform-macos --all-targets`, protocol check (B4 of 010) listing `process_instance` as additive output.

# 050 — wp5: final re-fetch and report

1. `git fetch origin --prune`; `gh api repos/trycua/cua/commits/main`; `gh api repos/vercel-labs/agent-desktop/commits/main`.
2. New goat-cu `origin/main` commits: merge into the branch (local), re-run the focused tests touched by both.
3. New upstream `libs/cua-driver` commits after `c0d3b919`: classify like 001; in-scope ones become a new work phase
   (`cxc loop add-work-phase`) with a 06x doc; out-of-scope ones are listed with reasons.
4. New agent-desktop core commits after `54855b62`: same, against 003's areas.
5. Final record: base/head/upstream SHAs, local commit list, per-issue dispositions, protocol record, receipts, NOT RUN,
   sent with `origin.report`.

## Final record (draft; completed at the last D)

### Local commits on cu/261112-upstream-core-r2 (base origin/main 40c6f6eb)

| Commit | Unit | Content |
|---|---|---|
| e3dd8120, 5b07f0b4 | wp0 | roadmap, stale check |
| 1feaeffb | wp1 | sync libs/cua-driver 566a4ce6 → c0d3b919 (0.47.12), GOAT patches kept, vendor refreshed |
| 20449d9f | wp1 D | wp1 record, wp1b retarget |
| e0851769 | wp2 | #299 macOS process_instance + instance-bound tokens; #288 session-scoped fast-path store |
| ef7a3646 | wp1b | sync c0d3b919 → e51f5740 (0.47.14) |
| 393c49e2 | wp3 | #286 relocation frame re-check, #279 hotkey focus retry + one-meaning perform rule, #281 not-settable message / gated dispatcher, #283 dead hand_back; GOAT.md refusals note |
| 68771fdb | wp4 | #272 / AD-G1 semantic visibility without ancestor clipping |
| 09484534 | wp2 review | #299 round-1 findings (recording hook evidence, list_windows off runtime, refresh label, bracketed labels) |
| a5e2bbde | wp3 | #287 bounded resting float + GNOME helper park |
| 0d163dd1, fb4f50e0 | wp2 review | #288 rounds 1–2 (delta session derivation, eviction tests) |
| 0654beae | wp2b | #299 Linux/Windows process_instance |
| ece9964a | docs | dispatch log |
| e894eabd | wp2b review | #299 round-3 findings (Linux/Windows second pass off the runtime, first read before metadata, refresh answers unlabelled, contract text; 020 corrected: Linux tokens are not instance-bound) |
| e354f982 | wp3 review | #287 round-1 findings (macOS parks without frame ticks via an idle-fade deadline; a parked float restarts on a moving SnapTo, a re-show and a theme reselect; docs) |
| fd7ef859 | wp2 review | #288 round-3 notes (session derivation only with deltas on; mode-free wire test; two more session shapes) |
| 4fc4252b | wp1b review | Windows unused_mut on the browser launch command |
| ad5f6193 | docs | #299 round-4 docs (process_instance per platform), 020 protocol record scope, 040 #272 case-3 correction |
| 19fcc334 | wp1c | sync e51f5740 → 465aab0b (0.47.15): 33102290 single-line submit ported onto GOAT's AX rung, GTK3 harness, CHANGELOG, vendor README |
| 39cabbe0 | wp2 review | #288 round-4 test robustness |
| 98632052 | wp3 review | #279 one meaning for AX perform errors in press_key/right_click/double_click; pointer gate (checked_point) before right/double-click pixel routes; APIDisabled click refusal tests; GOAT.md |
| b724ece4 | style | rustfmt of this branch's macOS files that were clean on origin/main |
| 458ff8f6 | docs | wp1b lead-side B2 review, dispatch log, final-record draft (pushed as PR #300 under the earlier authority) |
| d0e90760 | CI fix | goat-ci on PR #300: Windows serve.rs cfg(unix) from the wp1 sync, clippy items_after_test_module (ax/bindings.rs), expect_err, is_multiple_of |
| 2a726372 | test | re-pin the ax/bindings.rs glob audit after the pure move |
| 133b2ef9 | wp1d | sync 465aab0b → 4cf88bae (0.47.16): Linux GTK3 overlay hit test, macOS e2e occluder test, vendored cua-telemetry 0.25.0 |
| 1ec667ea | wp1c review | a line that landed without its Return is LineWithoutReturn (unverified, not the retryable type_text_incomplete); submit_step / completed_text_result pure decisions with call-site tests; focus written only when needed and disclosed; per-read-back withhold reasons |
| 0f0d4b70 | wp3c review | #279 rule extended: a native scroll-bar AXPress error that may have acted ends the call (no second wheel scroll); proven non-delivery and pointer-gate refusals in right/double-click are coded refusals; press_key's semantic AXConfirm/AXCancel perform goes through the write-outcome classifier; GOAT.md |
| 019c3858 | CI fix | Linux `mutter_display` descriptor-count test runs alone in a child test process (goat-ci run 43156456005) |
| cd815162, 5c760643, d5e23ca6, d0fb0350 | docs | wp3, wp4, wp1b, wp1c close-outs with path:line; final-review replacement record; receipt-helper fix; browser flake |
| 88c70460 | docs | third re-fetch; unit wp1e (015) |
| b5aa4734 | contract | `contract/manifest.json` regenerated for the #299 `process_instance` outputs (`cua-contract-gen -- all --check` exit 1 → 0) |
| 068c9a00 | wp1e | sync 4cf88bae → 4f2aa028 (0.47.18): `invoke_menu` `delivery_mode` contract (manifest and UniFFI bindings regenerated), hyprland requalify channel, CHANGELOG, installer version, vendored cua-telemetry 0.25.2 |
| e3dcec4d | docs | final record at 068c9a00 |
| 3be03b46 | wp1f | sync 4f2aa028 → b9e38d1a: libxkbcommon 1.0 text functions resolved at run time (Linux; Ubuntu 20.04 loads the driver) |
| c0205638 | docs | final re-fetch; unit wp1f |
| b0c7a85a | CI fix | paused-clock batch pacing test runs on its own desktop lane (goat-ci run 43183852970) |
| 7c5cfcd1 | docs | batch pacing CI fix record; reviews re-dispatched after the GOAT restart |
| 57e9cae1 | test fix | libei.rs tests compile with `portal-input` again (pre-existing on origin/main since 87e2e869) |
| ff3338e7 | docs | libei portal-input test fix and CI follow-up |
| 895e6ad2 | wp1c round-2 review | a single-line submit's line that landed without its Return carries no re-send escalation (was `recommended: foreground`) |
| 888d0811, 09a0e5dd, 495f9c31 | docs | review records (libei PASS, wp1c r2, CI/wp1d, wp1e) |
| 83aeb256 | wp1c final review | a landed line sets its own hint (press Return with press_key, do not type again) over the seam's "read before typing again"; `completed_delivery` pins the delivery mapping |
| 1f2060af | #287 round-2 review | macOS parked loop keeps the ~1 s repin of a revealed pinned cursor; SetMotion/ApplyMotion restart a parked float; docs |
| 648ee8ec | docs | wp1c final-review and #287 round-2 decisions |
| ff6658cc | wp3c delta review | coded undelivered AX refusals (right_click, double_click, press_key, non-fronted scroll) are typed NOT_DELIVERED, so a run_steps step that sent nothing counts as not delivered (was DeliveredUnverified / retry Unsafe); GOAT.md fronted-window exception; Linux fd test marker carries the spawning pid |
| 8939c283 | docs | wp3c delta review r2 decisions |
| 8248c43b | wp1g | sync b9e38d1a → 56a69039 (cua-driver-rs 0.47.19): CHANGELOG entry, published installer version 0.47.18; GOAT keeps its versions (017) |
| eb04e2a0 | docs | wp1g re-fetch and unit record |
| fd7b782a | wp1f/CI review | `ToolRegistry::isolate_desktop` doc names its limit: calls dispatched through the registry only (test-only item) |
| 9f79bc0f, 222d484a | docs | wp1f/CI review decisions; issue table and final receipt at eb04e2a0 |
| bfef37d8 | final review of 83aeb256/1f2060af | the landed-line hint reads the field before Return (a failed Return post may still have been sent); GNOME parked-float doc wording |
| d46baf86 | docs | final review of 83aeb256/1f2060af decisions |
| ff24221c | review of bfef37d8 | the landed-line message leads with "do not repeat type_text" and gives the hint's read-first Return advice (was focus-only) |
| a2c73b30 | docs | bfef37d8 review decisions |
| f3107f37 | wp1g review | install.ps1's network-failure fallback names `mcp-config --client <name>` (was `connect`); a test guards both installers' fallbacks |
| 07d81e6c | docs | wp1g review decisions |
| a3bd993b | ff24221c review round 2 | both landed-line instructions (do not repeat; read first, then Return) lead the message, inside run_steps' 300-character step message |
| 286bc5bb | docs | ff24221c round-2 decisions |
| c7cce744 | ff6658cc review | GOAT.md lists which APIDisabled refusals are typed (press_key's menu-chord refusal stays untyped: menu validation may have posted Cmd+F20); Linux fd test marker is `spawned-by-<pid>` |
| 7cac46e0 | docs | ff6658cc review decisions |
| 743260c6 | test fix | libei `a_text_stops_when_its_tool_call_is_over` (upstream, unchanged) abandoned the call after a fixed 50 ms settle and failed when the typing thread had sent nothing yet (seen under the receipt's load and 1 in 20 idle runs); it now waits on the server log for the first part (red with a scratch 200 ms typing delay, then 30/30, portal-input 1230 / 15) |

### Re-fetch before the last D (2026-11-12)

origin/main `40c6f6eb` (unchanged); agent-desktop main `54855b62` (unchanged); trycua/cua main `465aab0b` (0.47.15), 10 commits after
`e51f5740` → unit wp1c (013).
Second re-fetch (~05:40 UTC): origin/main and agent-desktop unchanged; trycua/cua main `4cf88bae` (0.47.16 + cua-sdk 0.25.0), 8 commits
after `465aab0b` → unit wp1d (014).
Third re-fetch (~08:35 UTC): origin/main `40c6f6eb` and agent-desktop main `54855b62` unchanged; trycua/cua main `4f2aa028` (0.47.18 +
cua-sdk 0.25.2), 16 commits after `4cf88bae` → unit wp1e (015): `invoke_menu` `delivery_mode` contract (ac720702), hyprland requalify
channel (377b0fee), release entries, vendored cua-telemetry version. The final evidence report moves to a closing phase after wp1b, wp1c
and wp1e.
Final re-fetch (~09:50 UTC): origin/main `40c6f6eb` and agent-desktop `54855b62` unchanged; trycua/cua main `b9e38d1a`, 4 commits after
`4f2aa028`, one in scope (b9e38d1a, #6498) → unit wp1f (016), folded into the wp6 cycle as task wp6/t2.
Re-fetch after ff6658cc (~11:45 UTC): origin/main `40c6f6eb` and agent-desktop `54855b62` unchanged; trycua/cua main `56a69039`
(cua-driver-rs 0.47.19), 2 commits after `b9e38d1a` in `libs/cua-driver`, release bookkeeping only → unit wp1g (017), task wp6/t3.

### Follow-ups recorded, not done in this lane

- #287 round-2 review P3 (rebutted, residual): the macOS parked-wake seam (charge idle time before applying commands) has no direct
  test; the X11 loop has one via `apply_messages_after_wake`. A test needs that drain extracted from the AppKit loop.
  The final review adds the same for the parked repin wiring (`next_repin_pinned`, `last_repin` at `cursor/overlay.rs:982`,
  `:1090`): only `parked_wait` and `parked_repin_due` are tested.
- Linux GTK3 hit test (upstream 7dbc78d1, taken byte-identical in 133b2ef9; final review of the CI/wp1d slice): the `SiblingStacking`
  first-match correction checks only the application's toolkit, so it may apply inside an embedded toolkit's subtree (WebKitGTK web
  process) at `atspi/native.rs:5359`; and its extra AT-SPI round-trips share the 1500 ms `INPUT_QUERY_BUDGET`, where a timeout yields
  `PointOwnership::Unknown` and the caller presses unguarded (`atspi/native/hit_test.rs:309`, fail-open predates this lane). Smallest
  fixes: skip the substitution when parent and hit node name different buses; reserve a budget slice for the stacking pass.
- `rust/VERSION` (0.38.0-goat.1) lags the workspace version (0.40.1-goat.1), identically on origin/main; nothing reads it.
- `run_steps` copies a step's `hint`/`next_call` into its report only for a failed step (`batch_tools.rs:1849-1868`), so a non-error
  step that carries advice (type_text's landed line without its Return) reaches a batched caller only through the 300-character
  message; a3bd993b puts both instructions in its first 220 characters. Copying hints for ok steps changes run_steps output for every
  tool.
- wp1f review P3 (upstream b9e38d1a, taken as merged): `wayland/text_keys.rs:90` opens the literal `libxkbcommon.so.0`, so a future
  static or bundled libxkbcommon build would refuse typing as "needs libxkbcommon 1.0" although the symbols are present; no such
  artifact ships. And about 20 unit tests (`niri/keyboard.rs:816` onward, `libei.rs:3527` onward) unwrap `xkb1()`, so a
  `platform-linux --lib` job on a libxkbcommon 0.10 host (Ubuntu 20.04) would panic rather than exercise the refusal; no such lane
  exists (CI and the receipt container have 1.x).
- wp1c round-2 review P3 (rebutted, residual): the `TypeTextDelivery` → `line_without_return` mapping in `type_text` `invoke` is two
  adjacent arms with no direct test; both ends are pinned (`submit_step`, `completed_text_result`), and pinning the arms needs a
  delivery-enum or AX-rung restructure.
- CI builds no `platform-linux` tests with `portal-input`, so the libei and portal tests (59 more than the default set) ran nowhere until
  57e9cae1; adding `--features portal-input` to a Linux unit job would keep them compiling (workflow change left to the maintainers).
- Linux element tokens are not bound to the process instance (Linux publishes and resolves with `NotChecked`); Windows and macOS are.
  Needs `publish_*_with_instance` + `resolve_for_instance` in platform-linux (020).
- `browser/prepare.rs:1134`: a `profile_root()` error after profile creation skips cleanup (inherited byte-identical from upstream,
  reachable only by concurrent env mutation); no composition test for GOAT containment + upstream breaker (wp1b review A).
- #287: the GNOME helper's own 33 ms repaint timer runs regardless of parking (its pose parks); a `cursor.float` off→on toggle sends
  the overlay no command, so a cursor that parked meanwhile stays upright until its next move or action (documented).
- #288: the fast-path caches are bounded and shared across sessions, so sessions can evict each other's entries (a miss, no leak);
  `forget_window` still has no production caller (pre-existing).
- #272 case 3: a sheet/popover/menu element whose `AXWindow` is a same-pid child of the admitted window is refused by
  `bind_exact_with` (tools/actionability/semantic.rs:85-88); admitting it needs its own safety argument.
- Flake observed once: `window_change_detector::tests::a_change_read_after_the_deadline_still_releases_the_lease` (arrived with the wp1
  sync, absent on origin/main) failed at `assert!(lock(&TAIL).is_none())` (window_change_detector.rs:1583) in one full parallel
  `platform-macos --lib` run on 1ec667ea's tree; it passed in the receipt, 5/5 alone and in the next full run (2022 passed). It sleeps on
  wall-clock deadlines and reads process-global tail state.
- macOS `type_text` single-line submit: the focus write before Return is disclosed in the answer but not added to the mutation chain
  (wp1c review P3, accepted in part).
- Flake observed once: one `cua-driver-core --lib browser` test failed (328 passed, 1 failed) in the first wp1b close-out receipt on
  d5e23ca6; that receipt kept only summary lines, so the test is unnamed. Six standalone reruns and three reruns beside a concurrent
  `platform-macos --lib` passed 329/0. Candidates are the wall-clock asserts in `browser/cdp_ws.rs:1576`, `:1602` (500 ms) and
  `browser/v2_tests.rs:4186`, `:4387`, `:4461`. Receipts now keep a full log per step (`/tmp/goat-cu-sync/receipt-lib.zsh`).
- Receipt helper fix: the wp2–wp4 and first wp1b close-out scripts printed each step's own exit correctly but gated on the status of the
  `echo`, so a failing step did not stop them. wp2–wp4 printed exit 0 and `ok` for every step, so their results stand; the first wp1b
  receipt is superseded by its re-run with the fixed helper (failures=0).
- #299 (Linux, pre-existing): other blocking reads still run on the async runtime — `list_windows` `retain_listed`
- Linux `invoke_menu` (pre-existing at origin/main): a sent AT-SPI `do_action` that returns a D-Bus error is reported as the
  `menu_path_unavailable` refusal (tools/invoke_menu.rs:38-43, atspi/native.rs `doAction failed`); it may have acted, so it should read
  unverifiable (the #279 rule, macOS-only in this lane).
  (`/proc/<pid>/status`) and the unlisted-window note (`/proc/<pid>/environ`), `get_window_state` `is_process_live` and
  `wayland::list_windows_dispatch` (#299 review 4). Windows label reads use the full `process_fingerprint` (bounded cost, accepted).
- Pre-existing warnings, same call graph on origin/main: `cua-driver-core` built without default features (`batch_tools/locate.rs`
  `LocatorRegistry::Borrowed`, `Locator::new`); Windows target `uia/mod.rs:1205` `walk_cached_bounded`, `input/inject.rs:314/318/363`.

### Issue dispositions (path:line at the final head)

| Issue | Disposition | Evidence |
|---|---|---|
| #299 | FIXED (macOS, Linux, Windows) | 020 §A/§B, review rounds 1–3; failing-first `a_token_from_an_earlier_start_of_the_pid_is_refused`, `a_synchronous_resolve_reads_live_evidence`, `a_pid_that_restarts_during_a_listing_gets_no_label`, Linux/Windows `a_label_needs_two_agreeing_start_time_reads` |
| #288 | FIXED | 020 §C + review rounds 1–3; failing-first `another_sessions_snapshot_is_not_visible` (runtime red store.rs:348), delta red tool.rs:6531 |
| #287 | FIXED (option 2, bounded float then park; GNOME helper follows) | 030 §B #287; failing-first cursor_pose/render_map/pose_stream tests; round 2 e354f982 (macOS parked loop waits on the idle deadline instead of frame ticks; a snap, re-show or theme reselect restarts a parked float) and 1f2060af (parked pinned cursor keeps its ~1 s repin; SetMotion/ApplyMotion restart a parked float), each red first |
| #286 | FIXED | 030 §A; failing-first `a_relocated_background_point_outside_the_frame_is_refused` |
| #283 | FIXED (#283.2 dead hand_back); #283.1 documented (GOAT.md), live smoke NOT RUN | 030 wp3 B/C |
| #281 | FIXED (#281.2 message, #281.3 dispatcher name + site log); #281.1 documented (GOAT.md) | 030 wp3 B/C; failing-first `a_selection_target_that_is_not_settable_says_so` |
| #279 | FIXED (#279.1 focus retry; #279.2 one meaning for perform errors in click, press_key, right_click, double_click, element_pointer AXOpen — legacy helper deleted, 98632052; extended to scroll's native scroll-bar presses and press_key's semantic AXConfirm/AXCancel, 0f0d4b70; the proven-not-delivered refusals typed NOT_DELIVERED so run_steps counts them as sending nothing, ff6658cc) | 030 wp3 B/C; failing-first `a_transient_focus_write_error_keeps_polling_until_the_deadline`, `perform_error_after_dispatch_is_unverifiable_unless_nothing_was_sent`, 98632052's 8 red tests, 0f0d4b70's 4, ff6658cc's 3 (incl. `a_batch_step_refused_as_not_delivered_counts_as_not_delivered`) |
| #272 | FIXED for the clipping cases (1, 5) in the macOS adapter (semantic visibility = window state + own frame); case 3 (sheet/popover/menu whose AXWindow differs) still refused by the exact binding — follow-up | 040 wp4 (corrected after the wp3/wp4 review); failing-first `semantic_press_ignores_ancestor_clipping`; live smoke on third-party apps NOT RUN |

### Final receipt (kache doctor exit 0), /tmp/goat-cu-sync/receipt-final.sh on eb04e2a0 (earlier run on 9bd0e96c also 24/24)

All 24 steps exit 0 (failures=0): check-host; core-lib 1825 passed / 3 ignored; core-int 93; contract-lib 91; overlay-lib 157; macos-lib
2031 passed / 48 ignored; windows-host 207; linux-host 110; linux-target and windows-target type checks; clippy-macos (-D warnings);
clippy-linux (-D warnings, cross target); windows-bins and linux-bins (cua-driver, cursor-theme-cli, cua-driver-uia, cua-driver-sdk,
portal-input); cargo deny; build-driver; e2e-check; check_protocol.py (protocol-compatible in every GOAT launch form; daemon 0.8.0 =
0.8.0; descriptions reworded for get_guide, set_config, type_text, not protocol); proto_delta problems 0; output delta adds only
process_instance on get_window_state, list_apps.apps[], list_windows.windows[], removes nothing; JS helper syntax, helper paint and
helper safety harnesses. Logs: /tmp/goat-cu-sync/receipt-final/.

### CI after the receipt

- PR #300 at 5c760643 (run 43183852970, Linux `cua-driver-core --lib`): `batch_tools::tests::pauses_between_steps_but_not_after_the_last`
  failed at tests.rs:1400 with "2. click ERROR (time_budget): step exceeded its execution budget" (1825 passed). The test runs on a paused
  tokio clock and replaces only step 1's deadline with a trigger (`deadline_override.take()`, batch_tools.rs:1037); step 2 keeps a real
  budget timer. A click takes the process-wide `desktop_action_coordinator()` and `desktop_lane()` (tool.rs:148, :174), which other tests in
  the binary hold; while step 2 waited for them the runtime idled and the paused clock jumped to the budget. Red (scratch hook, not
  committed): another thread holding the process-wide lane between the steps reproduced the exact CI message in 0.04 s. Fix b0c7a85a: test-only
  `ToolRegistry::isolate_desktop` (own coordinator and lane, as tool.rs's lane test does) and `Harness::isolated()`; the same scratch hold
  then passes. `cua-driver-core --lib`: macOS 3x 1823 passed / 3 ignored (kache), Linux container 2x 1826 passed / 3 ignored; clippy
  -D warnings (platform-linux, cua-driver-core) clean. It is the only `start_paused` test in batch_tools.
- Found by a local x86_64 cross-check at 7c5cfcd1 (`cargo check --target x86_64-unknown-linux-gnu -p cua-driver -p cursor-theme-cli
  -p cua-driver-sdk -p platform-linux --features cua-driver/portal-input --all-targets`, exit 101): `wayland/libei.rs`'s test module held
  GOAT's `Emitted` and upstream's `use local_eis::{record, Emitted}` (from 87e2e869, already on origin/main), so `platform-linux` tests
  did not compile with `portal-input` (11 errors). No CI job builds them with that feature (cua-driver's default features are empty),
  which is why main stays green. 57e9cae1 imports the upstream one as `EisEmitted`. Linux container: red 11 compile errors →
  `cargo test --locked -p platform-linux --lib --features portal-input` 1229 passed / 15 ignored, clippy with the feature -D warnings
  clean; the cross-check then exits 0 (Windows target, Linux x86_64 bins + platform-linux, clippy Linux x86_64 and macOS).
- PR #300 at b54560e5 (run 43156456005, `PR Linux schema contract and platform unit`):
  `wayland::mutter_display::tests::a_socket_that_never_accepts_fails_at_once_and_leaves_nothing_open` failed 81 vs 79 at
  mutter_display.rs:523 with 1167 other tests passing. The socket path closes its descriptor on every error (`connect_bounded` owns
  it as `OwnedFd` straight after `socket(2)`); the test counted `/proc/self/fd` for the whole process while the other tests in the
  binary open and close their own descriptors on other threads. Red (scratch, not committed): a sibling thread holding a socket pair
  inside the window gives 8 vs 6, the same +2. Fix: the test re-runs itself alone in a child test process (`--exact` plus a marker
  variable) and requires the child's ` 1 passed;`, so a filter that matches nothing fails. Linux (rust:1.97.1 aarch64 container,
  plain cargo; Xvfb installed for the two X11 tests): named test green; the scratch sibling held across the child run green; a scratch
  typo in the child filter red; full `platform-linux --lib` 3× 1168 passed / 15 ignored; clippy `-p platform-linux -p cua-driver-core
  --all-targets -D warnings` exit 0.

### Protocol record (check_protocol.py on 068c9a00, darwin-universal baseline)

`python3 goat/compat/check_protocol.py --candidate rust/target/debug/cua-driver --candidate-version 0.40.1-goat.1 --baseline-key
darwin-universal` exit 0: "protocol-compatible with the baseline in every GOAT launch form"; daemon contract 0.8.0 = 0.8.0. This lane adds,
in both the embedded-proxy and direct-embedded forms, `invoke_menu` optional argument `delivery_mode` (wp1e) and the output-only
`process_instance` on `get_window_state`, `list_apps.apps[]`, `list_windows.windows[]` (#299; the output delta script reports nothing
removed). The other "added optional argument(s)" lines compare against that older frozen baseline and predate this lane:
`/tmp/goat-cu-sync/proto_delta.py` against the 0.40.1-goat.1 release binary (this lane's base release) on 068c9a00 reports 65 = 65 tools in
both forms, added [], removed [], the one optional argument `invoke_menu.delivery_mode`, descriptions reworded for `set_config` and
`type_text` (not protocol), daemon 0.8.0 = 0.8.0, problems 0; the same script found no added optional argument after wp1 (010), and the
check_protocol output differs from the 9bd0e96c receipt only by the two `invoke_menu` lines. No allowed-changes entry is needed: each
change is provably additive. `contract/manifest.json` and both UniFFI binding sets pass their generators' `--check` (wp1e receipt).

### Per-phase receipts (kache doctor exit 0 in each; `cxc receipt test`)

| Phase | Head | Result |
|---|---|---|
| wp2 | 019c3858 | failures=0: core fast_path 57, snapshot_store 74, read_session 1; macOS process_identity 27, list_windows 10, list_apps 5, get_window_state 88, fast_path_guard 1, lease_wiring 22; host linux process_instances 3, windows process_instance 10 |
| wp3 | cd815162 | failures=0: platform-macos --lib 2026 passed / 48 ignored; cursor-overlay --lib 157; clippy -p platform-macos --all-targets -D warnings |
| wp4 | 5c760643 | failures=0: core actionability 26, post_action_wait 6, session 186; macOS actionability 55, element_pointer 30, get_window_state 88 |
| wp1b | d5e23ca6 | failures=0 (fixed helper): core browser 329, cursor-overlay 157, e2e standalone_browser_behavior 3, macOS security_prompt 10, browser 104, Linux container platform-linux 1168 |
| wp1c | d0fb0350 | failures=0: macOS type_text 60, type_text_chars 4, e2e check, Linux container platform-linux 1168 |
| wp1e | 068c9a00 | failures=0 (15 steps): contract 91, contract-gen --check, uniffi --check, contract_parity, schema_consistency, protocol_schema_test 2, check_protocol, macOS invoke_menu 33, windows host 207, Linux container 1168, requalify new tests 2, python compat 2 |
| whole head | eb04e2a0 | `/tmp/goat-cu-sync/receipt-final.sh` 24 steps, failures=0: kache doctor, host `cargo check --all-targets` (platform-macos, core, cursor-overlay, cua-driver, sdk, contract), core lib + integration, contract lib, cursor-overlay lib, platform-macos lib, platform-windows host, Linux x86_64 and Windows MSVC target checks (libs and bins), clippy -D warnings macOS and Linux x86_64, cargo deny, `cargo build -p cua-driver`, platform-linux host lib, e2e `--tests` check, check_protocol, protocol and output deltas, Wayland helper node checks. Linux container at the same content: cursor-overlay 157, platform-linux --lib --features portal-input 1229 / 15 ignored; default features 1170 / 15 and clippy at ff6658cc (no Rust change after it but fd7b782a's test-only doc) |
| whole head | 7cac46e0 | the same 24 steps, failures=0. Linux container: cursor-overlay 157, cua-driver-core --lib 1826 / 3, platform-linux --lib 1171 / 15 (with and without a leaked marker) and clippy; portal-input 1229 / 1 failed under the receipt's concurrent load (`a_text_stops_when_its_tool_call_is_over`, a fixed-settle test from upstream), 1230 / 0 rerun idle → fixed in 743260c6 (portal-input 1230 / 15, default 1171 / 15, clippy both, 30/30 standalone) |

Linux runs are in a rust:1.97.1 aarch64 container (plain cargo, Xvfb installed); CI covers x86_64.

### Review state for the exact head

Earlier rounds per unit are in 004. The whole-branch final review of 12e47ec6 failed in provider compaction; the commits without a review
of their final form are under narrow fresh reviews: `cu-delta-review-019c3858-s24` (0f0d4b70, 019c3858), `cu-final-287-e354f982-s24`,
`cu-final-wp1c-1ec667ea-s24`, `cu-final-ci-wp1d-s24` (d0e90760, 2a726372, 133b2ef9), `cu-wp1e-review-068c9a00-s24` (b5aa4734, 068c9a00).
Docs-only commits are not reviewed separately.

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
| 7761e270 | docs | receipt at 7cac46e0, libei fix |
| 7d048914 | final review before merge | type_text message "Return not confirmed" (a failed post may have gone out); libei wait counts text parts only; GOAT.md names both chord refusal codes |
| c2067c65 | wp1h | sync 56a69039 → 7dc3cc43 (018): macOS cursor on every display and action marks retired, with GOAT's #287 parked loop and no-guessed-scale rule; disabled Linux/Windows controls indexed and refused; Computer History deprecated (GOAT's start guide too); permissions direct-capture status; installer 0.47.19; vendored cua-telemetry 0.26.0 |
| 9b9b29af | wp1h | upstream's Windows-only disabled-row test sets GOAT's `checked`/`expanded` (Windows-target check 101 → 0) |
| af2d3826 | docs | wp1h record (018) |
| 08c5ca6e | wp1h review B | Windows `element_is_enabled` joins COM before reading `IsEnabled`, so the disabled-control refusal cannot fail open on a fresh blocking-pool thread |
| ed282703 | wp1h review A | macOS AX click drops the retired `ShowFocusRect` send (upstream's removal the merge had not re-applied) |
| 79766176 | docs | wp1h reviews, follow-ups, receipt at ed282703 |
| 190a160e | wp1i | sync 7dc3cc43 → 7f2252fb (019): `page` keeps only `get_text`/`query_dom` (page actions, their switch, the Windows bookmark bridge and GOAT's `page_dispatch.rs` removed; `goat/compat` records the non-additive `page` schema); `replay_trajectory`, `install_ffmpeg`, `get_accessibility_tree` deprecated with a notice and a `tools/list` marker; Perception frozen; installer download retry; releases 0.47.20/0.47.21 and cua-sdk 0.27.0 (GOAT versions kept, vendored cua-telemetry 0.27.0); macOS pixel restore no longer tracks `page` |
| c678bba8 | docs | wp1i record, receipt at 190a160e |
| c8a91fc0 | wp1j | sync 7f2252fb → d17e70f5 (019b): a ref-targeted trusted `browser_click` reports its bounded target readback (confirmed / unverifiable); release 0.47.22 CHANGELOG (GOAT versions kept) |
| ffc428c2 | docs | wp1j record, receipt at c8a91fc0 |
| bc02a591 | wp1k | sync d17e70f5 → 52306ab4 (019c): upstream's accepted-browser-defaults test, which reads GOAT's three adapters and passes |
| aeb89b8c | docs | wp1k record, receipt at bc02a591 |
| a2539a20 | wp1k review | the Linux/Windows reasons for keeping the attach-background default say the attach fronts the browser while it opens the setup page, then restores focus |
| d789e198 | docs | wp1j and wp1k review outcomes |
| 7cab649b | wp1i review | macOS `type_text` browser-tab guidance names `get_browser_state` + `browser_type` instead of `page`'s removed actions; the removed-page-action test asserts both refusals |
| b3a6a07e | docs | wp1i and wp1k review outcomes, receipt at 7cab649b |
| af4e914b | wp1l | sync 52306ab4 → 228b5a4d (019d): release 0.47.23 CHANGELOG entry; GOAT versions kept |
| 35755746 | docs | wp1l re-fetch and commit list |
| 99de9504 | wp1m | sync 228b5a4d → 01596ffc (019e): upstream's test that every native-input tool publishes the `ActionResult` (passes on GOAT's surface, macOS and Linux) and the contract's audit table |
| 1f1d4edd | final review of 7cab649b | the untrusted web read-back result leads with the read-first rule inside run_steps' 300-character step message; Safari's address-field refusal reads with `page` and acts through element actions; `same_pid_keyboard_ambiguity` names `get_browser_state` and the `browser_*` tools |
| 5e172dbe | wp1n | sync 01596ffc → fd4505cf (019f): installer pin 0.47.22 (upstream's published version) |
| 491904b1 | final review round 2 | the unverified mark carries the condition on typing again ("type again only if the text is missing"), so no cut can keep the write without it |
| 03af2302 | docs | the contract doc names the `select_text`/`menu_bar_extras` carve-out (wp1m review); wp1m, wp1n and final review records |
| 27f04c31 | docs | final receipt at 03af2302 and last re-fetch |
| abc581b7 | CI fix | two `window_state_view` tests no longer share the process-wide snapshot store: the "both" case moves to an unused window id, and a test build keeps 4096 snapshots in total (production 32, per-window 8) so parallel reads cannot evict a test's own (goat-ci run 43301885942) |
| 34f79cb5 | wp1o | sync fd4505cf → 2fc7f3c7 (019g): release 0.47.24 CHANGELOG entry; GOAT versions kept |
| 52f2b794 | wp1p | sync 2fc7f3c7 → 2319d456 (019h): installer pin 0.47.23 (upstream's published version) |
| 902b6e30 | wp1m review | the carve-out sentence names only the two tools' acting results (discovery and refusal payloads carry no `verified`); the capability-derived test gap recorded as a follow-up |

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
Re-fetch before the last D (~16:20 UTC): origin/main `40c6f6eb` and agent-desktop `54855b62` unchanged; trycua/cua main `7dc3cc43`,
7 commits after `56a69039` in `libs/cua-driver` (54 files) → unit wp1h (018), task wp6/t4.
Re-fetch after 79766176 (2026-11-12 ~18:00 UTC, repeated ~18:15 UTC after the GOAT restart): origin/main `40c6f6eb` and agent-desktop
`54855b62` unchanged; trycua/cua main `7f2252fb`, 11 commits after `7dc3cc43`, 7 of them in `libs/cua-driver` (45 files) → unit
wp1i (019), task wp6/t5.
Re-fetch after c678bba8 (~18:45 UTC): origin/main `40c6f6eb` and agent-desktop `54855b62` unchanged; trycua/cua main `d17e70f5`, 2 commits
after `7f2252fb`, both in `libs/cua-driver` (12 files) → unit wp1j (019b), task wp6/t6.
Re-fetch after ffc428c2 (~19:00 UTC): origin/main `40c6f6eb` and agent-desktop `54855b62` unchanged; trycua/cua main `52306ab4`, 1 commit
after `d17e70f5` (core `browser/platform.rs` only) → unit wp1k (019c), task wp6/t7.
Re-fetch after b3a6a07e (~20:10 UTC): origin/main `40c6f6eb` and agent-desktop `54855b62` unchanged; trycua/cua main `228b5a4d`, the 0.47.23
release (CHANGELOG and version files only) → unit wp1l (019d), task wp6/t8.
Re-fetch after 35755746 (~19:55 UTC): trycua/cua main `01596ffc` (#7507, protocol_schema_test and the contract doc) → unit wp1m (019e), task wp6/t9.
Re-fetch after 1f1d4edd (~20:15 UTC): trycua/cua main `fd4505cf` (#7509, published installer 0.47.22) → unit wp1n (019f), task wp6/t10.
origin/main `40c6f6eb` and agent-desktop `54855b62` unchanged at each.
Last re-fetch before the final D (2026-11-12 20:30 UTC, after the receipt at 03af2302): trycua/cua main `fd4505cf` (no commit after wp1n),
origin/main `40c6f6eb`, agent-desktop `54855b62` — no new in-scope delta. The remote lane branch reads `99de9504` (pushed by the
coordinator), an ancestor of the local head.
Re-fetch after abc581b7 (~20:50 UTC): trycua/cua main `2fc7f3c7` (release 0.47.24) → unit wp1o (019g). Re-fetch after 34f79cb5
(~21:10 UTC): trycua/cua main `2319d456` (published installer 0.47.23) → unit wp1p (019h).
Last re-fetch (2026-11-12 21:40 UTC, after 902b6e30): trycua/cua main `2319d456` (no commit after wp1p), origin/main `40c6f6eb`,
agent-desktop `54855b62` — no new in-scope delta. The remote lane branch reads `34f79cb5` (pushed by the coordinator), an ancestor of
the local head.

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
- wp1h review B: GOAT's `press_key`/`hotkey` token paths do not refuse a disabled target on any platform (macOS reads `AXEnabled`
  only for menu chords; Windows `tools/press_key.rs:275-281`, `tools/hotkey.rs:396`; Linux `tools/press_key.rs:146`). Disabled controls
  are indexed on Linux and Windows since wp1h, as on macOS, so a key sent by token can reach one. Smallest fix: call each platform's
  `refuse_if_disabled` after resolving the token, before focus. And both platforms' `refuse_if_disabled` let a tool act when the live
  state cannot be read (upstream's design; a destroyed element is caught by the stale-token checks).
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
- wp1i, for GOAT: `get_accessibility_tree` is deprecated upstream (removal in the first release after 26 Nov 2026) and GOAT still
  allows it (goat-si/goat `packages/shared/src/cuaDriverManifest.ts:28` at origin/dev c77c9745). It works and now carries a notice
  naming `list_windows`/`list_apps`/`get_window_state`; GOAT must drop it from `CUA_DRIVER_ALLOWED_TOOLS` before goat-cu syncs the
  removal, or `check_protocol.py`'s `validate()` fails the candidate.
- Perception worker sandbox (pre-existing, unchanged since 1feaeffb): with `TMPDIR=/tmp` (a symlink to `/private/tmp`) six
  `perception_client` tests fail with `WorkerCrashed` / "early eof"; the default `TMPDIR` and `/private/tmp` pass 35/35. The worker's
  temp working directory is handed to the sandbox unresolved (`perception_client.rs:642-659`), so a symlinked `TMPDIR` likely breaks
  production parses too. GOAT does not ship Perception, and upstream froze it (a967d132).
- Receipts after the GOAT restart run with `TMPDIR=/private/tmp`: this thread's 85-character `TMPDIR` pushes five `cua-driver` socket
  test paths past SUN_LEN (424/424 with a short `TMPDIR`), and `/tmp` trips the Perception sandbox above.
- wp1j review (upstream d17e70f5, for the lane that adopts `browser_*`): the trusted click's `DOM.resolveNode` keeps the CDP call's 20 s
  timeout while the tab's mutation lock is held (`browser/tools.rs:1783`; `dom_event` has the same shape at `:1594`); the click
  readback reads the target's `value` in-process on a tab the secret fence may cover (`:73`, compared and dropped, never published);
  the ref-targeted trusted dialog branch (`:1943`) has no test.
- wp1k review: `every_platform_declares_the_browser_platform_defaults_it_accepts` never reads the reasons. Its source parser would
  miss a second `impl BrowserPlatform` block (the test then fails falsely) or count a `#[cfg]`-gated override (the check becomes
  vacuous); neither shape exists in GOAT's adapters today (019c). GOAT patch to keep on the next sync: the two Linux/Windows reason
  strings in `browser/platform.rs`'s `ACCEPTED_DEFAULTS` (a2539a20).
- wp1i review: `page` keeps `read_only: false`, `idempotent: false`, `open_world: true` (`cua-driver-core/src/page.rs:106-109`,
  identical upstream), so a page read is recorded into trajectories, pushes a PiP frame and is batch-scoped as a mutation; GOAT never calls
  `page`. And the `deprecated` marker sits at the top level of each `tools/list` entry (`tool.rs:414-419`, forwarded by `proxy.rs`
  and `sdk_adapter.rs`), outside the MCP `Tool` shape: GOAT's readers (`check_protocol.py`, `AcpMcpStdioBridge.ts`,
  `McpNativeSession.swift`) tolerate it; a strict third-party validator might not.
- wp1m review: GOAT's macOS-only `select_text` and `menu_bar_extras` are desktop input (`authorization.rs:292-293`) but advertise no
  native-input capability (no arm in `default_capabilities_for`, `tool.rs:956`), so `every_native_input_tool_publishes_the_action_result`
  skips them, and they return legacy `effect`/`verified` payloads (`select_text.rs:396-398`, `menu_bar_extras.rs:415-421`) instead of
  the `ActionResult`. Follow-up: give both their capabilities and the action-result projection. The contract doc names the carve-out
  (GOAT patch to keep on the next sync). Also stale: `tool.rs:990-999` says `type_text_chars` is registered on Linux; no roster
  registers it.
- wp1m morph review (34f79cb5): `every_native_input_tool_publishes_the_action_result` is capability-derived, so a future native-input tool
  that advertises no capability and is not an action-result tool passes unseen. Smallest fix: a GOAT test with a per-platform expected
  set of native-input tool names, `select_text` and `menu_bar_extras` listed as the current exceptions.
- Final review rounds 2-3 (informational): an embedded web view after pixel focus reads "do not repeat it" next to the mark's "type
  again only if the text is missing" (the px rung is terminal); the non-web branch says "read the field" where the field could not be
  read; with a field title over ~27 characters the browser-tab step's px clause falls past run_steps' 300-character cut (the read and
  its condition always survive, and the cut ends in "...").

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

Issue evidence at the final code head 03af2302 (failing-first tests, run by every whole-head receipt): #299
`platform-macos/src/process_identity.rs:366`, `:421`, `:469`, `platform-linux/src/process_instances.rs:78`,
`platform-windows/src/process_instance.rs:348`; #288 `cua-driver-core/src/fast_path/store.rs:334`; #286
`platform-macos/src/tools/click.rs:5647`; #281 `click.rs:5586`; #279 `hotkey.rs:897`, `click.rs:8003`, `tools/mod.rs:2955`; #272
`tools/actionability.rs:1022`; #287 `platform-macos/src/cursor/overlay.rs:1984`, `cursor-overlay/src/render_state.rs:3765`.

All 24 steps exit 0 (failures=0): check-host; core-lib 1825 passed / 3 ignored; core-int 93; contract-lib 91; overlay-lib 157; macos-lib
2031 passed / 48 ignored; windows-host 207; linux-host 110; linux-target and windows-target type checks; clippy-macos (-D warnings);
clippy-linux (-D warnings, cross target); windows-bins and linux-bins (cua-driver, cursor-theme-cli, cua-driver-uia, cua-driver-sdk,
portal-input); cargo deny; build-driver; e2e-check; check_protocol.py (protocol-compatible in every GOAT launch form; daemon 0.8.0 =
0.8.0; descriptions reworded for get_guide, set_config, type_text, not protocol); proto_delta problems 0; output delta adds only
process_instance on get_window_state, list_apps.apps[], list_windows.windows[], removes nothing; JS helper syntax, helper paint and
helper safety harnesses. Logs: /tmp/goat-cu-sync/receipt-final/.

### CI after the receipt

- PR #300 at 27f04c31 (run 43301885942, `PR Linux schema contract and platform unit`, `cua-driver-core --lib`):
  `window_state_view::tests::an_elements_diff_counts_only_the_elements_it_returns` failed at window_state_view.rs:2641 with
  `since_status` "diff" instead of "no_change" (1830 passed). Its "both" case read window 7021, which
  `since_returns_diff_then_falls_back` also reads, and `since:"latest"` names a window's most recent read in the process-wide snapshot
  store, so the other test's read landing between this test's second and third reads made "latest" the other test's snapshot. Red
  (scratch, not committed): replaying that read between the two reads gave the CI failure exactly (left "diff", right "no_change",
  "both"; `/tmp/goat-cu-sync/wsv-red.log`). Running the module alone then showed a second shared-store failure:
  `since_with_elements_returns_every_element_and_passes_the_typed_check` read "unknown_snapshot" (`wsv-module.log`, 45 passed / 1
  failed) because the store keeps 32 snapshots across all windows and parallel tests evicted the one it had just stored; storing 32 other
  windows' snapshots between its reads reproduced it (`wsv2-red.log`). Fix abc581b7: the "both" case uses window 7030, which no other
  test uses, and a test build keeps 4096 snapshots in total (`#[cfg(test)]`; production 32; the per-window cap of 8 unchanged; no test
  exercises the total cap). No assertion changed. Each scratch passes with its fix in place. macOS: `window_state_view` 46 passed in
  30 of 30 runs, `cua-driver-core --lib` 2× 1828 passed / 3 ignored. Linux container at 902b6e30 (no Rust change after abc581b7):
  both named tests passed (`--exact`), `window_state_view` 46 passed in 30 of 30 runs, `cua-driver-core --lib` 1831 passed / 3 ignored
  and `clippy -p cua-driver-core --all-targets -D warnings` exit 0 (`/tmp/goat-cu-sync/wsv-focused-902b6e30.log`). goat-ci on 34f79cb5 (which contains abc581b7):
  `PR Linux schema contract and platform unit`, `PR macOS schema contract`, `PR Windows schema contract and platform unit`, Clippy macOS
  and Linux, `deny` and the Linux and Windows arm64 builds green.
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
check_protocol output differs from the 9bd0e96c receipt only by the two `invoke_menu` lines. Up to wp1h no allowed-changes entry was
needed. wp1i (upstream b13430e0) removes `page`'s page actions, which `check_protocol.py` reports as `tools page: input schema changed` in
both launch forms (exit 1 at 190a160e before the entry). `goat/compat/allowed-changes.json` records it with the reason: GOAT's manifest
excludes `page`, so GOAT never calls it, and the actions were already refused unless a daemon operator set
`CUA_DRIVER_ENABLE_LEGACY_PAGE_MUTATIONS`. With the entry, check_protocol exits 0 and proto_delta reports problems 0 (65 = 65 tools,
`invoke_menu.delivery_mode` the only added argument). The deprecated tools stay listed with unchanged input schemas (description prefix
and a `deprecated` entry only). `contract/manifest.json` and both UniFFI binding sets pass their generators' `--check` (wp1e and wp1i).

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
| wp1h | c2067c65 / 9b9b29af | receipt 22/24 at c2067c65 (windows-target, windows-bins: E0063 in upstream's Windows-only test) → both 0 at 9b9b29af; focused suites and the Linux container in 018 |
| whole head | ed282703 | 24 steps, failures=0: core-lib 1825 / 3, core-int 93, contract-lib 91, overlay-lib 158, macos-lib 2038 / 48, windows-host 207, linux-host 110, Linux and Windows target checks (libs and bins), clippy -D warnings macOS and Linux x86_64, deny, build-driver, e2e check, check_protocol (compatible in every launch form; daemon 0.8.0 = 0.8.0), protocol delta problems 0 (only `invoke_menu.delivery_mode`; set_config/type_text reworded), output delta adds only `process_instance`, helper tests. Linux container at c2067c65 (no Linux code after it): platform-linux 1173 / 15, portal-input 1232 / 15, cursor-overlay 158, core 1826 / 3, clippy both feature sets |
| wp1i | 190a160e | failing first `page_actions_never_hold_the_restore_wait` (red 101, then green); focused suites, cross checks and the Linux container in 019 |
| whole head | 190a160e | `TMPDIR=/private/tmp` (see follow-ups): 24 steps, failures=0: core-lib 1822 / 3 (+2 isolated child runs), core-int 93, contract-lib 91, overlay-lib 158, macos-lib 2036 / 48, windows-host 200, linux-host 110, Linux and Windows target checks (libs and bins), clippy -D warnings macOS and Linux x86_64, deny, build-driver, e2e check, check_protocol (compatible in every launch form with the `page` entry), protocol delta problems 0, output delta adds only `process_instance`, helper tests. Linux container at 190a160e content: platform-linux 1173 / 15, portal-input 1232 / 15, cursor-overlay 158, core 1825 / 3, clippy both feature sets. A first run with `TMPDIR=/tmp` failed core-lib on six Perception sandbox tests (environment, see follow-ups) and was stopped |
| wp1j | c8a91fc0 | upstream's new trusted-click tests red against the parent's `tools.rs` (4 of 5, exit 101), `browser::v2_tests` 120 with the merge; Linux container core 1830 / 3, clippy core + platform-linux clean |
| whole head | c8a91fc0 | `TMPDIR=/private/tmp`: the same 24 steps, failures=0: core-lib 1827 / 3 (+2 isolated child runs), core-int 93, contract-lib 91, overlay-lib 158, macos-lib 2036 / 48, windows-host 200, linux-host 110, target checks, clippy macOS and Linux x86_64, deny, build-driver, e2e check, check_protocol compatible, protocol delta problems 0, output delta adds only `process_instance`, helper tests |
| wp1k | bc02a591 | `browser::platform` 4 passed (new test included); Linux container core 1831 / 3 |
| whole head | bc02a591 | `TMPDIR=/private/tmp`: the same 24 steps, failures=0: core-lib 1828 / 3 (+2 isolated child runs), core-int 93, contract-lib 91, overlay-lib 158, macos-lib 2036 / 48, windows-host 200, linux-host 110, target checks, clippy macOS and Linux x86_64, deny, build-driver, e2e check, check_protocol compatible, protocol delta problems 0, output delta adds only `process_instance`, helper tests |
| review fixes | a2539a20, 7cab649b | `browser::platform` 4 passed; `browser_tab_guidance_names_the_typed_browser_tools` red (exit 101, type_text.rs:5065) then `tools::type_text` 62 / 1; `protocol_schema_test` 2 / 1 ignored after a probe run recorded both refusal texts |
| whole head | 7cab649b | `TMPDIR=/private/tmp`: the same 24 steps, failures=0: core-lib 1828 / 3 (+2 isolated child runs), core-int 93, contract-lib 91, overlay-lib 158, macos-lib 2037 / 48, windows-host 200, linux-host 110, Linux and Windows target checks (libs and bins), clippy -D warnings macOS and Linux x86_64, deny, build-driver, e2e check, check_protocol compatible in every launch form (get_guide reworded, not protocol), protocol delta problems 0, output delta adds only `process_instance`, helper tests. No Linux code after 190a160e beyond core (Linux container core 1831 / 3 at bc02a591) |
| wp1m | 99de9504 | `protocol_schema_test` 3 / 1 ignored (macOS), 5 passed (Linux container) |
| final review fixes | 1f1d4edd | red first: `an_untrusted_web_readback_step_message_keeps_its_instruction` (type_text.rs:5111), set_value.rs:2799, background_input.rs:494; then platform-macos type_text + set_value 199 / 1, core background_input 21; Linux container core 1831 / 3, clippy core + platform-linux clean |
| whole head | 1f1d4edd | `TMPDIR=/private/tmp`: the same 24 steps, failures=0: core-lib 1828 / 3, core-int 93, contract-lib 91, overlay-lib 158, macos-lib 2038 / 48, windows-host 200, linux-host 110, target checks, clippy macOS and Linux x86_64, deny, build-driver, e2e check, check_protocol compatible, protocol delta problems 0, output delta adds only `process_instance`, helper tests |
| wp1n | 5e172dbe | `bash -n`, connect-step 15, install version fallback 40 / 7 skipped (upstream layout) |
| final review round 2 fix | 491904b1 | the sweep over detail lengths 0-300 red at type_text.rs:5118, then tools::type_text 63 / 1; platform-macos --lib 2038 / 48; clippy -p platform-macos --all-targets -D warnings clean. Only this file changed after the 1f1d4edd receipt |
| whole head (final) | 03af2302 | `TMPDIR=/private/tmp`, kache doctor 0: the same 24 steps, failures=0: core-lib 1828 / 3 (+2 isolated child runs), core-int 93, contract-lib 91, overlay-lib 158, macos-lib 2038 / 48, windows-host 200, linux-host 110, Linux x86_64 and Windows MSVC target checks (libs and bins), clippy -D warnings macOS and Linux x86_64, cargo deny (bans, licenses, sources ok), build-driver, e2e check, check_protocol compatible in every launch form (daemon 0.8.0 = 0.8.0), protocol delta problems 0, output delta removes nothing, Wayland helper checks. After it, the only Rust change is abc581b7 (one test module) |
| CI fix | abc581b7 | red first (scratch): the CI failure and "unknown_snapshot" each reproduced exactly; then `window_state_view` 30/30 runs and core-lib 2× 1828 / 3 (macOS), the Linux container receipt at 902b6e30 and goat-ci on 34f79cb5 in "CI after the receipt" |
| wp1o | 34f79cb5 | CHANGELOG top 40 lines identical to 2fc7f3c7; the 8 version files byte-identical to the parent (019g) |
| wp1p | 52f2b794 | `bash -n`, connect-step 15, install version fallback 40 / 7 skipped (upstream layout) (019h) |

Linux runs are in a rust:1.97.1 aarch64 container (plain cargo, Xvfb installed); CI covers x86_64.

### Review state for the exact head

Earlier rounds per unit are in 004. The whole-branch final review of 12e47ec6 failed in provider compaction; the commits without a review
of their final form are under narrow fresh reviews: `cu-delta-review-019c3858-s24` (0f0d4b70, 019c3858), `cu-final-287-e354f982-s24`,
`cu-final-wp1c-1ec667ea-s24`, `cu-final-ci-wp1d-s24` (d0e90760, 2a726372, 133b2ef9), `cu-wp1e-review-068c9a00-s24` (b5aa4734, 068c9a00).
After wp1h: wp1i 190a160e NEAR-PASS → 7cab649b; wp1j c8a91fc0 PASS; wp1k bc02a591 NEAR-PASS → a2539a20, round 2 PASS; fresh final review
of 7cab649b NEAR-PASS → 1f1d4edd (af4e914b PASS), round 2 PASS (also wp1n 5e172dbe) with lows → 491904b1, round 3 PASS at 491904b1;
wp1m 99de9504 NEAR-PASS, its findings documentation only → 03af2302. Every code commit's final form has a PASS or a reviewed fix; the
commits after 491904b1 change docs and devlog only. Docs-only commits are not reviewed separately.

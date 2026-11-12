# 010 — wp1: Sync libs/cua-driver 566a4ce6 → c0d3b919 keeping every GOAT patch

Class C4 (cross-platform merge, security-relevant). Stale check at wp1 P: re-run the 001 classification against the
then-current HEAD; any new goat-cu commit on `origin/main` is merged first.

wp1 P stale check (2026-11-11 ~18:40 UTC): `origin/main` still `40c6f6eb`; trycua/cua `main` still `c0d3b919`; HEAD `e3dd8120`
differs from `40c6f6eb` only by `devlog/`, so 001's classification stands. Previous D (wp0): "roadmap locked; next wp1 P
re-verifies 010 then executes B1–B4" — unchanged direction. Architect dispatch `cu-wp1-architect-261112` failed with 429 (004).
Host targets installed: `x86_64-unknown-linux-gnu`, `x86_64-pc-windows-msvc` (cross `cargo check` attempted in C; C toolchain
availability decides RUN/NOT RUN). `cargo-deny` present.

## B1 — Synthetic upstream commits and 3-way apply (no remote, no clone)

```sh
OLD=/tmp/goat-cu-upstream/566a4ce655905c0d28065f4f534e221e92916e24/libs/cua-driver
NEW=/tmp/goat-cu-upstream/c0d3b919fc4621b494353e5a594cd36cd4e125f0/libs/cua-driver
mkdir -p /tmp/goat-cu-sync
GD=$(git rev-parse --absolute-git-dir)          # linked-worktree gitdir; objects go to the shared store
GIT_INDEX_FILE=/tmp/goat-cu-sync/idx-old git -C "$OLD" --git-dir="$GD" --work-tree="$OLD" add -A -f .
T_OLD=$(GIT_INDEX_FILE=/tmp/goat-cu-sync/idx-old git --git-dir="$GD" write-tree)
C_OLD=$(git commit-tree "$T_OLD" -m "trycua/cua 566a4ce655905c0d28065f4f534e221e92916e24 libs/cua-driver (synthetic sync base)")
GIT_INDEX_FILE=/tmp/goat-cu-sync/idx-new git -C "$NEW" --git-dir="$GD" --work-tree="$NEW" add -A -f .
T_NEW=$(GIT_INDEX_FILE=/tmp/goat-cu-sync/idx-new git --git-dir="$GD" write-tree)
C_NEW=$(git commit-tree "$T_NEW" -p "$C_OLD" -m "trycua/cua c0d3b919fc4621b494353e5a594cd36cd4e125f0 libs/cua-driver (synthetic sync target)")
git cherry-pick --no-commit "$C_NEW"      # 3-way: base T_OLD, ours HEAD, theirs T_NEW
```

Objects only; no ref is created. Files only goat-cu has (`GOAT.md`, `LICENSE.md`, `vendor/`, `goat/`, `.github/`) are untouched
because they exist in neither synthetic tree. Gate before resolving: `git diff --cached --name-only | wc -l` + unmerged count
must match 001 (298 upstream paths; 71 conflicted ± files whose merge-ort result differs from `git merge-file`).

## B2 — Resolution policy (every conflict hunk)

| Group | Files | Rule |
|---|---|---|
| V Version/metadata | `rust/VERSION`, `rust/Cargo.toml` workspace `version`, `python/pyproject.toml`, `python/src/cua_driver/__init__.py`, `typescript/package.json`, `typescript/package-lock.json` root version, `rust/Skills/cua-driver/SKILL.md` version line, `README.md` version mentions | KEEP-GOAT values (`0.40.1-goat.1` workspace; `0.38.0`/`0.38.0-goat.1` where GOAT has them); take every other upstream line |
| V CHANGELOG | `rust/CHANGELOG.md` | upstream 0.41.0 … 0.47.12 entries above GOAT's existing entries |
| V Cargo.lock | `rust/Cargo.lock` | take upstream's lock, then let a kache cargo command rewrite workspace member versions; keep GOAT-only deps if cargo re-adds them; `cargo deny` afterwards |
| C Core | `cua-driver-core/src/{recording.rs, lib.rs, tool.rs, protocol.rs, batch_tools.rs, browser/*}`, `cua-driver-sdk/src/{embedded.rs, runtime.rs}`, `cua-driver/Cargo.toml` | COMBINE. Keep: GOAT desktop lease + worker holds with upstream `DesktopLane` (release reverse order); session-scoped snapshot lookups; GOAT private-directory recording writes after upstream `redact_values`; GOAT `BrowserRefusal` detail. Adopt: `security_prompt`, `input_pacing`, `type_pacing` modules, recording restart finalization (#6794), batch nudge (#6921) |
| M macOS | the 31 `platform-macos` files in 001 | COMBINE. Keep: actionability gates (#266, #269), write-outcome classification (#259, #267, #275, #277), exact-field/item/hit-target gates (#278, #280, #282), process-start stamps, key-window hand-back ordering, background-only delivery, exact-target window lease, scroll validation (#297), AXPress-less checked pointer click (#298), GOAT window-local stamp fallback when the OS version is unreadable. Adopt: security-prompt refusal (#7338) ahead of any send, focus arbiter (#6803) as the activation-undo owner, batched AX reads (#6878) feeding GOAT's walk evidence, enabled re-read after activation (#6702/#7041), middle-click routing (#7074), macOS 27 drag stamp (#7372), AXWindows error naming (#7192), keyboard-layout exact text (#6827/#6939) |
| L Linux | the 18 `platform-linux` files | COMBINE. Keep: `spawn_text_write_until` fencing, `gated_mutator` + `element_write_unacknowledged`, single-EIS-region drag refusal, process-instance ids. Adopt the rest |
| W Windows | the 13 `platform-windows` files | COMBINE. Keep: process-instance-bound publish (U15), guarded cursor restore, partial-modifier accounting, "effect uncertain, do not resend" pixel timeout, UIA non-S_OK = uncertain (#247). Adopt upstream keyboard/typing-turn changes |
| T Tests/fixtures | `tests/fixtures/apps/macos/appkit/main.swift`, e2e tests | union of both sides' cases |

Every clean-merged modified-both file in the 5 safety-critical core files (`session.rs`, `tool.rs`, `authorization.rs`,
`window_target.rs`, `health_report.rs`) is read once after the merge for the invariants above, even with no markers.

## B3 — vendor/ refresh from c0d3b919

```sh
S=/tmp/goat-cu-upstream/c0d3b919fc4621b494353e5a594cd36cd4e125f0/libs
diff -rq "$S/cua/crates/cua-telemetry" vendor/cua/crates/cua-telemetry      # list changes first
cp -R "$S/cua/crates/cua-telemetry/." vendor/cua/crates/cua-telemetry/      # then git rm any file upstream removed
cp "$S/cua/crates/cua-teleport/src/ux/targets.json" vendor/cua/crates/cua-teleport/src/ux/targets.json
cp "$S/images/sandbox-images.json" vendor/images/sandbox-images.json
# re-apply vendor/cua/crates/cua-telemetry/Cargo.toml workspace values from "$S/cua/Cargo.toml"; keep cua-connections dev-dep out
```

`vendor/README.md`:

```diff
-Source commit: trycua/cua `566a4ce655905c0d28065f4f534e221e92916e24` (main, 2026-11-06). Files are
+Source commit: trycua/cua `c0d3b919fc4621b494353e5a594cd36cd4e125f0` (main, 2026-11-11). Files are
```

`GOAT.md` §Syncing upstream step 1 names the restart commit; it stays. The newest sync record is the wp1 commit message
(no PR in this lane), which names both upstream SHAs and carries the preservation table.
GOAT.md step 3 node-runtime lock: `uniffi-bindgen-react-native` is `0.31.0-3` at 566a4ce6, c0d3b919, 9e7256bf and in the merged
`typescript/package.json:55`, so `scripts/node-runtime/Cargo.lock` is not regenerated.

## B4 — Protocol record

```sh
cargo build -p cua-driver                                   # kache wrapper via ~/.cargo/config.toml
python3 goat/compat/check_protocol.py --candidate rust/target/debug/cua-driver \
  --candidate-version 0.40.1-goat.1 --baseline-key darwin-universal
```

Any non-additive change it reports gets an `allowed-changes.json` entry with reason + GOAT impact, or the merge is changed
to keep GOAT's schema. Additions are listed in the D record.

## C — Verification (fresh, focused, kache)

| Command | Reads the change? |
|---|---|
| `kache doctor` | wrapper health |
| `cargo check -p cua-driver-core -p platform-macos -p cursor-overlay -p cua-driver --all-targets` (rust/) | yes: every merged Rust file on macOS |
| `cargo test -p cua-driver-core --lib` | core merged files |
| `cargo test -p platform-macos --lib` | macOS merged files (no live UI; tests that need a live app are `#[ignore]`) |
| `cargo test -p cursor-overlay --lib` | overlay (wp3 baseline) |
| `cargo deny --locked --exclude-dev check licenses bans sources` | lock/vendor |
| `(cd goat/compat && python3 -m unittest -v test_check_protocol)` (as goat-ci.yml) + B4 | protocol |
| `cargo check --target x86_64-unknown-linux-gnu -p platform-linux` / `x86_64-pc-windows-msvc -p platform-windows` | only if the targets and C toolchains exist on this host; otherwise NOT RUN (CI covers) |

## Preservation table

Filled during B from 001's 71-row table: path | hunks | rule (KEEP-GOAT / TAKE-UPSTREAM / COMBINE) | invariant kept | upstream PRs adopted.

### macOS, core, CLI and metadata (lead-resolved)

| Path | Rule | Invariant kept / reason | Upstream |
|---|---|---|---|
| README.md | KEEP-GOAT | GOAT fork README structure; upstream's Cua Spaces install section not added |  |
| python/pyproject.toml, python/src/cua_driver/__init__.py, rust/VERSION, rust/Cargo.toml (version), typescript/package.json, typescript/package-lock.json, rust/Skills/cua-driver/SKILL.md (version) | KEEP-GOAT | GOAT release versions (0.40.1-goat.1 workspace; 0.38.0 packages) | release bumps 0.41.0–0.47.12 |
| rust/Cargo.toml [profile.release] | KEEP-GOAT | GOAT build-override strip=none; upstream strip+fat LTO (#7316, PyPI size) not adopted: release profile is out of this lane | #7316 not adopted |
| rust/crates/cua-driver/Cargo.toml | KEEP-GOAT | no cua-agent-setup/toml_edit: upstream `connect` (#6763/#7165/#7190) not adopted — needs unvendored cua-agent-setup + cua-home and edits third-party agent configs; connect.rs, its test, CLI enum/parser/manifest/examples removed | connect not adopted |
| rust/Cargo.lock | TAKE-UPSTREAM then cargo | upstream lock, workspace members rewritten to 0.40.1-goat.1 by cargo; cua-agent-setup/cua-home drop out |  |
| rust/Skills/cua-driver/{SKILL,WORKFLOW,MACOS}.md | COMBINE | GOAT text + upstream batch-nudge wording (two calls, ready run_steps) and Calc notes | #6921 |
| cua-driver-core/src/lib.rs | COMBINE | GOAT input_resource, tree_boundary + upstream input_pacing, type_pacing | #6884, #6889 |
| cua-driver-core/src/tool.rs | COMBINE | GOAT registry_disposition(read_only) + upstream batch_nudge annotate(nudge_args) | #6921 |
| cua-driver-core/src/protocol.rs, batch_tools.rs | KEEP-GOAT | GOAT's condensed instructions / run_steps description (wording only) |  |
| cua-driver-core/src/browser/prepare.rs | COMBINE | GOAT holds_browser_attachment helpers + upstream async is_driver_owned_pid_for_session with process fingerprint (pid-reuse safe) | upstream pid-reuse fix |
| cua-driver-core/src/browser/tools.rs, secret_fill.rs | COMBINE | GOAT snapshot_mode; browser_resource_ownership now async (.await at GOAT sites incl. secret_fill #285) |  |
| cua-driver-core/src/browser/v2_tests.rs | COMBINE | both test sets |  |
| cua-driver-core/src/recording.rs | COMBINE | upstream Recorder/Live state (#6794 restart finalizes previous) + GOAT PrivateDir in Live; final session.json via verified PrivateDir; failed-restart test now asserts the live recording stays consistent | #6794 |
| cua-driver-sdk/src/embedded.rs, runtime.rs | COMBINE | GOAT FOREGROUND_IDLE_THRESHOLD_ENV + upstream input pacing envs; GOAT CompatibilityContext + upstream admission state | #6889, #7037 |
| platform-macos/src/window_change_detector.rs | COMBINE | upstream focus_arbiter owns detached lease/epoch (#6803); GOAT Tail before-state keyed by arbiter id for superseded-change reporting; GOAT end_tail = clear tail + arbiter end_detached + focus_guard detached-hold release; GOAT 120 ms report window; GOAT opened_surfaces + upstream add_window_identity | #6803, #6928 |
| platform-macos/src/key_focus_guard.rs, tools/bring_to_front.rs, tools/launch_app.rs, input/skylight.rs | COMBINE | GOAT end_tail + key hand-back wait before activation; upstream focus_arbiter::note_intentional/UndoWatch; GOAT KeyFocus u32 own-id semantics kept; upstream blocker hint in refusals | #6803, #7192/CUA-3763 |
| platform-macos/src/input/mouse.rs | COMBINE | GOAT gesture engine, pre-built click pairs, target check; upstream dispatch_mouse_event, input pacing, security-prompt refusal on HID primitives; GOAT's background gesture stamped `point - origin` on every OS. #7372 adopted: `background_stamp` stamps window-local only when `window_location_is_window_local()` (the click route's gate; GOAT keeps window-local when the version is unreadable), the screen point before macOS 27. Failing-first `background_drag_stamps_window_local_points_only_where_window_server_reads_them` (mouse.rs:3294 left (100,100) right (270,294)) then pass | #7338, #6889, #7372 |
| platform-macos/src/input/keyboard.rs | COMBINE | GOAT GlobalModifierIo, desktop guarded typing; upstream post_hid_key/hid_sink seam (via hid_history), prompt refusal on global typing | #7338 |
| platform-macos/src/tools/{press_key,hotkey,type_text,scroll,drag,right_click,click}.rs | COMBINE | GOAT key_hold routes, not_editable guards, write-outcome results; upstream security-prompt refusal added before every HID/global route (refuse_frontmost_keys / PromptInput) and mapped with security_prompt::tool_result | #7338 |
| platform-macos/src/tools/click.rs (AX path) | KEEP-GOAT + prompt | GOAT actionability battery, selection gates, checked pointer press; upstream web-focus wait (#6889) and ax_click_structured not ported (perf/shape); middle-click routing (#7074) merged | #7074; #6889 not ported |
| platform-macos/src/tools/double_click.rs | COMBINE | GOAT dispatch_open + actionability; upstream foreground assist and AXEnabled re-read after activation | #6947, #7041 |
| platform-macos/src/tools/invoke_menu.rs | KEEP-GOAT | GOAT worker owns lease and key hold until hand-back (same goal as upstream #7039 PressAdmission, which is not adopted; its dead code removed) | #7039 covered |
| platform-macos/src/ax/exact_target.rs, ax/bindings.rs, tools/mod.rs, document_edit.rs | COMBINE | upstream AxWindowId (#6758) kept as element_window_ids/owning_window_ids/focused_window_ids_of_pid for upstream call sites; GOAT gates keep u32 (root for elements, own id for the focused window); GOAT gather_background_facts kept (sheet parent, recorded/context menus, sheet-aware minimized state) with upstream's per-window `ax_mapped_window_rows` replacing the full layer-0 enumeration. Equivalent for the count: `count_competing_keyboard_destinations` only counts same-pid, visible, AX-mapped, non-minimized rows; the sheet parent is still dropped. The merged upstream source test `snapshot_dispatch_invariants.rs` failed at :23 before this and names GOAT's `window_id` argument after it; upstream AXWindows error API (#7192) added beside GOAT's evidence readers | #6758 partial, #6882, #7192 |
| platform-macos/src/ax/tree.rs, ax/window_scope.rs | KEEP-GOAT | GOAT tree_source walk with evidence/completeness (GOAT already has collapsed menus and batched attrs); upstream ax/batch.rs removed as unused; AxUnresolved gains ax_windows_error (None on GOAT walk) | #6878 not adopted (GOAT batching) |
| platform-macos/src/capture.rs | COMBINE | GOAT native-capture gate (abandoned workers tolerated, stall marker); upstream captureImage(in:) main-display fn kept | #6881 |
| platform-macos/src/windows.rs | COMBINE | GOAT enumeration completeness (entry_readable) + upstream WindowDescription/describe_window single-window lookup | #6882 |
| platform-macos/src/apps/mod.rs | COMBINE | GOAT kernel running states + upstream installed-app scan cache | perf |
| platform-macos/src/browser/platform.rs (test) | TAKE-UPSTREAM | posix_spawn fixture with only named listeners (supersedes GOAT pre_exec fd closing) | #7048 |
| platform-macos/src/ax/window_scope.rs | COMBINE | GOAT SheetOfWindow + upstream ax_windows_error field | #7192 |
| cua-driver-core/src/security_prompt.rs (+ platform-macos/src/security_prompt.rs test) | GOAT patch on an upstream module | `is_consent_alert` folds U+2019 and ASCII apostrophes (`same_label`). macOS 27.2 (26B5091g) TCC and NetworkExtension tables hold only "Don’t Allow", so upstream's live test `the_system_consent_labels_include_allow_and_dont_allow` failed here; folding only widens refusal (fail-safe). Live test now asserts the tables loaded ("Allow" present) and that both renderings classify as consent | #7338 follow-up |

### Identifier audit of the merged Rust files (/tmp/goat-cu-sync/idscan.py)

Every merged `.rs` outside platform-linux (169 files): names GOAT added (HEAD minus 566a4ce6) missing now, and names upstream added
(c0d3b919 minus 566a4ce6) missing now. Windows' 10 conflict files: nothing lost either side. Each hit and its disposition:

| File | Missing name(s) | Disposition |
|---|---|---|
| core recording.rs | GOAT test `failed_restart_does_not_leave_old_recording_enabled` | replaced by `failed_restart_keeps_the_live_recording_consistent` (upstream restart model, row above) |
| cua-driver main.rs | upstream `connect` | not adopted (row above) |
| macOS ax/exact_target.rs | upstream sheet targeting (`ax_target_window`, `windows_with_sheets`, `attached_sheets`, 2 tests) | GOAT sheet handling kept (`SheetParentOnce`, `sheet_parent_window_id`, `sheet_enumeration_tests`) |
| macOS ax/mod.rs, ax/tree.rs | upstream `batch`, identity-attr batching (#6878) | GOAT tree walk with evidence kept (row above) |
| macOS capture.rs | upstream `NativeCaptureState` ticket + `late_timeout_cannot_mark_a_newer_permit_stalled` | covered: GOAT per-permit slot CAS (`abandon`, capture.rs:659) and `a_worker_that_finished_is_never_counted_as_abandoned` |
| macOS input/skylight.rs | upstream `sheet_focused` tests (window with a focused sheet is "already key") | GOAT own-id KeyFocus kept: keys addressed to the parent of a focused sheet are refused ("no keys were sent"), fail-closed; keys to the sheet's own id are confirmed (`a_focused_sheet_avoids_reactivation_for_the_sheet_and_its_window`) |
| macOS input/mouse.rs | upstream `macos_27_background_drag_stamps_window_local_points` | #7372 adopted through `background_stamp` + GOAT test (row above) |
| macOS tools/click.rs | upstream web-focus wait (#6889), `ax_click_structured` + `ax_click_result_ignores_the_previous_actions_late_window` (#6928) | not ported: GOAT `finish_ax_click` (click.rs:5739) reports earlier and immediate windows as `opened_surfaces` and then never claims `suspected_noop` (no pixel-retry hint), the conservative "do not resend" side of GOAT's result contract, unchanged from HEAD |
| macOS tools/invoke_menu.rs | upstream `PressAdmission` (#7039) | covered by GOAT worker-owned lease (row above) |
| macOS tools/type_text.rs | upstream `RETYPE_TAIL_DELAY_MS`, `UNCHANGED_GIVE_UP` (#6884) | batching adopted; tail retype and the 500 ms zero-progress give-up not adopted: a retype resends while the first send's effect is uncertain, and an early "0 delivered" can be wrong when batched keys land late, inviting a duplicate; GOAT keeps the 2 s drain (`DELIVERY_DRAIN_TIMEOUT`) |
| macOS window_change_detector.rs | upstream removed `Tail`, `TAIL`, `end_tail`; GOAT kept them | GOAT tail reporting (row above) |
Line-level follow-up (/tmp/goat-cu-sync/linescan.py: every line GOAT added over 566a4ce6 that no longer appears verbatim): 103 lines in
18 files, all accounted for. Reflowed or re-wrapped calls (`hotkey.rs` key_hold routes now inside a security-prompt refusal
block, counts 8 = 8; `account_focus_receipt` 3 = 3 in press_key, 8 = 8 in type_text); async `browser_resource_ownership(..).await`;
`recording.rs` private_dir moved into upstream `Live` (all writes still through `PrivateDir`, recording.rs:854-951, 1145-1173);
`HOST_OWNED_RUNTIME` kept (capture.rs:1533, fully qualified type); HID posts now through upstream `dispatch_mouse_event`
(mouse.rs:68) and `post_hid_key` (keyboard.rs:823), both tagging and recording in `hid_history`; HEAD's `post_global_key` tap
parameter had no non-HID caller (`git grep CGEventTapLocation::Session HEAD` empty), so HID-only loses nothing; `key_hold`
BareGlobal still records through `post_hid` (key_hold.rs:158); `decide_window_scope` gained the AXWindows-error argument;
`window_change_detector` tail read now keyed by the focus arbiter (`unread_by_tail`, :374/:466); exact_target (this lane's
#6882 change); browser/platform.rs test fixture (upstream #7048, row above).

### Independent review dispositions (wp1)

Reviews: `cu-wp1-review-a-s24-261112` (core) NEAR-PASS, `cu-wp1-review-b-s24-261112` (macOS input) NEAR-PASS (codex /
system24/deepseek-v4.1-flash / high, read-only). Each finding was checked against HEAD and upstream before acting.

| # | Finding | Disposition | Evidence |
|---|---|---|---|
| B1 P2 | batched default without upstream's tail retype | FIXED. GOAT's window path kept its HEAD default (30 ms; 0 for a terminal, type_text.rs:350-355); only `scope:"desktop"` took upstream's `DEFAULT_DELAY_MS = 0`, where there is no read-back. `DEFAULT_DELAY_MS = 30` (HEAD's desktop default), window path reads it, `delay_ms` description matches. Retype still not adopted (it resends after an unconfirmed delivery) | failing-first `the_default_delay_paces_each_character` (left 0, right 30) → pass; `tools::type_text` 57 passed |
| B2 P3 | `UNCHANGED_GIVE_UP` absent | NOT ADOPTED (unchanged decision, identifier audit above): an early "0 delivered" can be wrong when keys land late | — |
| B3 P3 | detached watch skips `release_detached` when its read returns after the deadline | FIXED (pre-merge bug: HEAD window_change_detector.rs:756-759 had the same combined return). Superseded tail still returns at once; own tail past deadline breaks to the release path | failing-first `a_change_read_after_the_deadline_still_releases_the_lease` (panicked :1575, lease held) → pass; `window_change_detector` 30 passed, `focus_arbiter` 2 passed |
| B4 P3 | web-focus wait is a 60 ms watch, not upstream's 800 ms `wait_for_web_focus` | NOT PORTED, unchanged from HEAD (HEAD click.rs:6970 already used `watch_for_change` bounded by `CLICK_OBSERVATION`); upstream's wait re-sends focus requests. Stale "(800 ms)" doc on `needs_web_focus_settle` corrected | doc only |
| A1 P2 | removed `connect` still advertised | FIXED: scripts/post-install-hints.txt restored to HEAD (its only change was the connect step); cli.rs bare-invocation hint and comment, skills.rs doc, _install-rust.sh fallback line back to `mcp-config --client <name>`; the installer's own connect step stays guarded by `driver_has_command connect` (skipped on GOAT's binary). Upstream installer test rewritten to GOAT's hints (`test_post_install_hints_name_mcp_config_not_connect`) | `uvx --from pytest pytest -q scripts/tests/test_install_connect_step.py`: 14 passed (GOAT CI does not run scripts/tests) |
| A2 P2 | `commit()` sets `settled` on every result, skipping the Drop rollback | NOT CHANGED (upstream design, identical in c0d3b919 prepare.rs:250-267). In both cited error paths the platform's pending handle is already gone (`commit_pending` removes it first; the missing-handle error is that path), so HEAD's Drop backstop would also have found nothing to undo (`abort_pending` → missing handle) apart from re-running the consent-surface dismiss passes after a panicked commit. Residual recorded | platform-macos/src/browser/setup_ui.rs:1377-1398, platform.rs:1421-1426, 1477-1505 |
| A3 P3 | unattested launch stored without fingerprint | NOT CHANGED (upstream prepare.rs:962): fails closed (never proven driver-owned); the later refusal names consumer-profile rather than the launch. Residual | — |
| A4 P3 | finalize loses the session.json error in state | FIXED: `finalize` records a refused/failed final write in `last_error` (the GOAT `PrivateDir::verify` refusal is the new case) | failing-first `a_refused_final_session_write_is_reported_in_state` (last_error None at :48) → pass; recording_private_followups 4, recording_restart 3, core `recording` 62 passed |
| A5 P3 | nudge example not valid JSON | NOT CHANGED: upstream wording (#6921); no GOAT invariant | — |
| A6 P3 | state worker entry stranded by a panic outside `catch_unwind` | NOT CHANGED: upstream code (recording.rs `STATE_CAPTURE_WORKERS`, new in c0d3b919); residual | — |

### Linux and Windows

Resolved by children `cu-wp1-linux-s24-261112` and `cu-wp1-windows-s24-261112` (004); per-hunk decisions appended when they report.

#### Windows (child `cu-wp1-windows-s24-261112`, completed 2026-11-11 ~20:05 UTC; lead-verified and staged)

| Path | Rule | Invariant kept / reason |
|---|---|---|
| input/inject.rs | COMBINE | GOAT `noactivate_registry_tests` and `iconic_sentinel_tests` kept; `pen_drag` release through GOAT `inject_pointer` (own-input recording) plus upstream's infallible `DestroySyntheticPointerDevice` |
| lib.rs | KEEP-GOAT + additive | GOAT modules (process_instance, process_termination, uia_mutation, uia_root, walk_completeness, value_append, secure_field) + upstream `typing_turn` |
| input/keyboard.rs | COMBINE | upstream `TypingTurn` marking and `type_pacing` batching (`plan_chunks`/`batch_events`) with GOAT `batch_edges` release tracking, `send_input_releasing` + `key_batch_failure` (partial-modifier accounting), and the producers `UnitFailure::ForegroundLost/Short/Stalled` (keyboard.rs:830/855/875; "inserted, may still appear, nothing retried"); GOAT "UIPI or a locked or secure input desktop" wording kept over upstream's |
| input/keyboard.rs `foreground_input_error`/`dispatch_state` | new combination | upstream envelope (`foreground_restored: false`) + GOAT write-outcome classification (inserted-but-unacknowledged → unverifiable, never retried) |
| tools/{hotkey,press_key,type_text}.rs | COMBINE | FocusUncertain → `is_foreground_envelope` → `foreground_input_error` → `keyboard_dispatch_error` (hotkey.rs:139/422, press_key.rs:102/345/375, type_text.rs:133/380) |
| tools/click.rs | KEEP-GOAT | `acquire_uia_pattern`, `RungOutcome`/`MutationRoute` (element already `ManuallyDrop`) |
| uia/mod.rs | COMBINE | upstream `ensure_com` short-circuit (STA-tolerant `com_init_result`) + GOAT `client::new_uia_client()`, `budget.record_skip`, `UiaReadCtx` failure logging; upstream's 2 COM-init tests added |
| uia/scroll.rs | COMBINE | GOAT `ScrollWrite`/`scroll_preflight`/`MutationStop` + upstream `# Safety` docs (moved onto the unsafe fns) |
| uia/windows_enum.rs | KEEP-GOAT + upstream fn | explicit `UiaDeadlineError::WorkerDisconnected` arm over `.unwrap_or_default()`; upstream `point_shape_chain`; duplicate merge copy removed |

Lead verification: name scan nothing lost; line scan 7 GOAT lines, all the per-unit → per-batch rewrite and import reflow.
The child's Windows-target `cargo check` stopped in C build scripts (rquickjs-sys, zstd-sys: no Windows SDK on this host). The lead
type-checked anyway with stand-in C compiler/archiver for the target (`/tmp/goat-cu-sync/fakecc/fake-{cc,ar}.sh` write empty
objects; `cargo check` never links; rquickjs-sys copies its prebuilt x86_64-pc-windows-msvc bindings), separate
`CARGO_TARGET_DIR=/tmp/goat-cu-sync/wincheck-target`:

| Command (rust/) | Exit | Result |
|---|---|---|
| `CC_/CXX_/AR_x86_64_pc_windows_msvc=<fake> cargo check --target x86_64-pc-windows-msvc -p platform-windows --all-targets` | 101 | E0282 at input/keyboard.rs:388 (`failure.error.as_ref()` ambiguous in the child's `dispatch_state`); unused imports `CoCreateInstance`, `CLSCTX_INPROC_SERVER`, `CUIAutomation` in uia/mod.rs (test-only use) |
| same, after: `inner: &anyhow::Error = &failure.error` / `error`; the three imports moved into `uia_is_usable_on_a_thread_already_in_an_sta` | 0 | 5 warnings, all present at HEAD on this target: `walk_cached_bounded` dead (HEAD uia/mod.rs:1179), 3 style-guard helpers test-only (HEAD inject.rs:314-363), `let mut command` (HEAD prepare.rs:894, `mut` needed only on macOS/Linux) |
| `cargo check -p platform-windows --tests` (host) | 0 | — |
| `cargo test -p platform-windows --lib` (host) | 0 | 203 passed (typing_turn 2, keyboard_dispatch producers 3) |

NOT RUN for Windows: linking and any Windows runtime/test (`input/keyboard.rs` tests are cfg(windows)); `cargo clippy` on the target.

Linux-target type check of platform-linux (stand-in C tools as for Windows, plus a stand-in `pkg-config` with
`PKG_CONFIG_ALLOW_CROSS=1`; `CARGO_TARGET_DIR=/tmp/goat-cu-sync/linuxcheck-target`):

| Command (rust/) | Exit | Result |
|---|---|---|
| `cargo check --target x86_64-unknown-linux-gnu -p platform-linux --all-targets` (child's libei.rs) | 101 | syntax: duplicated `fn wait_for_reply` signature (libei.rs:197-198; child removed it after a steer) |
| same | 101 | syntax: stray `}` after `activate_option_token_gated` (atspi/mod.rs:732); lead removed it |
| same | 0 | 1 warning: unused `len` in `write_through_editable_proxies` (atspi/native.rs). Upstream added `editable_insert_length_for` (toolkit-aware InsertText length); GOAT already computes that length per toolkit in `text_write_policy::text_units` (Gtk bytes, Qt UTF-16, unknown non-ASCII refused, text_write_policy.rs:153). Lead dropped the unused parameter and kept upstream's early refusal (nothing written either way; the indexed route re-walks once, the focused route reports `Refused` instead of `InsertUnavailable`) |
| same, after | 0 | no warnings (future-incompat note for third-party `nom` only), /tmp/goat-cu-sync/linuxcheck.log |

NOT RUN for Linux: linking, unit tests (platform-linux is Linux-only), live X11/Wayland/libei runs.

Cross-target type checks of the lead-resolved crates (same stand-in C tools, separate target dirs under /tmp/goat-cu-sync):

| Command (rust/) | Exit | Result |
|---|---|---|
| `cargo check --target x86_64-unknown-linux-gnu -p cua-driver-core -p cursor-overlay --all-targets` | 0 | no warnings (/tmp/goat-cu-sync/linuxcheck-core.log) |
| `cargo check --target x86_64-pc-windows-msvc -p cua-driver-core -p cursor-overlay --all-targets` | 0 | only the HEAD-era `let mut command` warning (/tmp/goat-cu-sync/wincheck-core.log) |

Lead audit of child-resolved files (name scan + line scan, as for macOS/core). Correction: the earlier "nothing lost" lines for
Linux files scanned in groups were invalid (an unsplit zsh variable passed all paths as one argument, so the scanners read
nothing). Re-run per file at ~20:30 UTC after the Linux resolution landed:
- Name scan, all 12 Linux conflict files: nothing GOAT-added or upstream-added lost; libei.rs keeps upstream-removed `SHIFT_L`,
  `XKB_EVDEV_OFFSET` (used by GOAT's keymap reader, libei.rs:1281-1292); wayland/mod.rs keeps `wayland_input_enabled` (alias of
  `is_wayland`, used by type_text_chars.rs).
- Line scan: 26 GOAT lines in 6 files, all accounted for. atspi/mod.rs: `activate_option_token_gated` now returns upstream's
  `native::OptionPick`; set_value.rs maps NoMatch/NoOptions to GOAT `no_such_option`, retries an unreadable popup only while
  waiting, and reports an `OptionPickSent` pick as possibly set; every `gated_route(gate, ..)` kept, including the XTest
  `Escape` (set_value.rs:454). launch_app.rs test and hyprland.rs `ScaledOutput` alias: formatting/type alias only.
  libei.rs: `wait_for_reply_timeout` takes upstream's `REPLY_WAIT`/`NOT_READY`; pointer commands still route through GOAT
  `routing_devices` + `crate::libei_route` (`plan_drag` refuses a cross-region drag before any event, libei.rs:895-922);
  delayed typing keeps GOAT `Cmd::KeyboardText` → `perform_keycodes` (refuses if `ei_text` appeared); upstream's `TextTicket`
  / unacknowledged text pipeline present.
- Linux tools/type_text.rs: 12 GOAT lines differ only by upstream's pacing signature; every `gated_route(&key_gate, ..)` around
  XTest/background/tty/Tk text routes kept (14 sites, same as HEAD); `wayland_input_enabled()` → `is_wayland()` is the same
  predicate (wayland/mod.rs:150 returns `is_wayland()`); budgets only grow (`text_typing_budget` = GOAT niri-aware
  `typing_budget` + libei line pacing, tools/impl_.rs:1339, plus `paced_extra`).
- Windows (mid-edit, 19:15): input/keyboard.rs `UnitFailure::Short`/`Stalled` per-unit mapping and the `keyboard_dispatch_error`
  arm in tools/hotkey.rs / tools/type_text.rs were missing; child steered (004). Re-audit on its report.


## C receipts on the merged tree before the Linux/Windows resolution lands (kache wrapper; `kache doctor` all checks passed)

| Command (in `rust/`) | Exit | Result | Log |
|---|---|---|---|
| `cargo test -p platform-macos --lib` (before the two fixes below) | 101 | 1997 passed, 1 failed (`security_prompt::tests::the_system_consent_labels_include_allow_and_dont_allow`), 48 ignored | /tmp/goat-cu-sync/macos-lib-r3.log |
| `cargo test -p cua-driver-core --lib security_prompt::tests` (new test, before fix) | 101 | `a_consent_label_matches_whichever_apostrophe_the_prompt_draws` FAILED at security_prompt.rs:277 | — |
| same, after fix | 0 | 6 passed | — |
| `cargo test -p platform-macos --lib security_prompt::tests` after fix | 0 | 4 passed, 1 ignored (live desktop) | — |
| `cargo test -p cua-driver-core --lib` | 0 | 1811 passed, 0 failed, 3 ignored | /tmp/goat-cu-sync/core-lib-r3.log |
| `cargo test -p cursor-overlay --lib` | 0 | 153 passed | /tmp/goat-cu-sync/overlay-lib-r3.log |
| `cargo test -p cua-driver-core --test '*'` (before exact_target fix) | 101 | `snapshot_dispatch_invariants::background_window_sources_delegate_without_space_queries_or_refiltering` panicked at :23 | /tmp/goat-cu-sync/core-int-r3.log |
| `cargo test -p cua-driver-core --no-fail-fast --test '*'` after fix | 0 | 24 test binaries, 92 passed, 0 failed | /tmp/goat-cu-sync/core-int-r4.log |
| `cargo test -p platform-macos --lib ax::exact_target` | 0 | 20 passed | — |
| `cargo test -p platform-macos --lib` (after both fixes) | 0 | 1998 passed, 0 failed, 48 ignored | /tmp/goat-cu-sync/macos-lib-r4.log |
| `cargo check -p platform-macos -p cua-driver-core -p cursor-overlay -p cua-driver -p cua-driver-sdk -p cua-driver-contract --all-targets` | 0 | no warnings, no errors | /tmp/goat-cu-sync/check-r4.log |
| `cargo test -p platform-macos --lib input::mouse::tests::background_drag` (new test, before the gate) | 101 | `background_drag_stamps_window_local_points_only_where_window_server_reads_them` FAILED at mouse.rs:3294 | — |
| `cargo test -p platform-macos --lib input::mouse` / `input::gesture` / `tools::drag` after the gate | 0 | 35 / 18 / 11 passed | — |
| `cargo test -p platform-macos --lib` after the drag gate | 0 | 1999 passed, 0 failed, 48 ignored | /tmp/goat-cu-sync/macos-lib-r5.log |

Still to run after Linux/Windows land: protocol check (B4), `cargo deny`, cross checks, and a final repeat of the rows above.

### B4 protocol record (macOS build of the merged tree; Linux/Windows sources do not enter it)

| Command | Exit | Result | Log |
|---|---|---|---|
| `cargo build -p cua-driver` | 0 | debug binary | /tmp/goat-cu-sync/build-r4.log |
| `python3 goat/compat/check_protocol.py --candidate rust/target/debug/cua-driver --candidate-version 0.40.1-goat.1 --baseline-key darwin-universal` | 0 | "protocol-compatible with the baseline in every GOAT launch form"; vs the frozen baseline only optional-argument additions and tool additions that predate this lane (62→65: browser_fill_secret, menu_bar_extras, select_text); `get_guide` argument descriptions reworded (not protocol); daemon contract 0.8.0 = 0.8.0 | /tmp/goat-cu-sync/protocol-r4.log |
| `(cd goat/compat && python3 -m unittest -v test_check_protocol)` | 0 | 23 tests OK (first try from the repo root exited 1 on `import check_protocol`; CI runs it from goat/compat) | /tmp/goat-cu-sync/protocol-unit-r4.log |
| `python3 /tmp/goat-cu-sync/proto_delta.py` vs the 0.40.1-goat.1 darwin release binary (SHA256SUMS verified earlier) | 0 | both launch forms 65 = 65 tools, added [], removed [], no added optional argument, `type_text` description reworded only, daemon 0.8.0 = 0.8.0, problems 0 | /tmp/goat-cu-sync/protocol-delta-r4.log |

No non-additive change → no `goat/compat/allowed-changes.json` entry for wp1.

`cargo deny --locked --exclude-dev check licenses bans sources` (rust/, current lock with cargo-rewritten member versions and
cua-agent-setup/cua-home dropped): exit 0, "bans ok, licenses ok, sources ok" (/tmp/goat-cu-sync/deny-r4.log).

## Final C receipts on the complete merged tree (2026-11-11 20:38 UTC; 0 unmerged paths; logs /tmp/goat-cu-sync/final-r1/)

| Command (rust/ unless noted) | Exit | Result |
|---|---|---|
| `kache doctor` | 0 | all checks passed |
| `cargo check -p platform-macos -p cua-driver-core -p cursor-overlay -p cua-driver -p cua-driver-sdk -p cua-driver-contract --all-targets` | 0 | 0 warnings |
| `cargo test -p cua-driver-core --lib` | 0 | 1811 passed, 0 failed, 3 ignored |
| `cargo test -p cua-driver-core --no-fail-fast --test '*'` | 0 | 24 binaries, 92 passed, 0 failed |
| `cargo test -p cursor-overlay --lib` | 0 | 153 passed |
| `cargo test -p platform-macos --lib` | 0 | 1999 passed, 0 failed, 48 ignored |
| `cargo test -p platform-windows --lib` (host) | 0 | 203 passed |
| `cargo deny --locked --exclude-dev check licenses bans sources` | 0 | bans ok, licenses ok, sources ok |
| `cargo build -p cua-driver` | 0 | — |
| (repo) `python3 goat/compat/check_protocol.py --candidate rust/target/debug/cua-driver --candidate-version 0.40.1-goat.1 --baseline-key darwin-universal` | 0 | compatible in every GOAT launch form; daemon 0.8.0 = 0.8.0 |
| (repo) `python3 /tmp/goat-cu-sync/proto_delta.py` vs 0.40.1-goat.1 release | 0 | 65 = 65 tools, added/removed none, `type_text` wording only, problems 0 |
| (goat/compat) `python3 -m unittest -v test_check_protocol` | 0 | 23 OK |
| Linux / Windows type checks (stand-in C tools; see above) | 0 | platform-linux, platform-windows, core, cursor-overlay |

## D — wp1 closed (2026-11-11 20:55 UTC)

| Ref | SHA |
|---|---|
| base goat-cu `origin/main` (fetched at D) | `40c6f6ebf824bccbbbfe1d68cbd17a2085701167` (unchanged) |
| head (local commit, not pushed) | `1feaeffba34f79cd9ab263e456156ff61937a09c` |
| upstream synced | trycua/cua `566a4ce655905c0d28065f4f534e221e92916e24` → `c0d3b919fc4621b494353e5a594cd36cd4e125f0` |
| upstream `main` at D | `e51f57407cac353702d166f24ac776875bbd0b02` (wp1b target; 9e7256bf..e51f5740 is release metadata only) |
| agent-desktop `main` | `54855b6225aec00da4ac65e2d0362fd41cd9fcb3` (unchanged) |

C receipt: `cxc receipt test -- /tmp/goat-cu-sync/receipt-wp1.sh` exit 0, failures=0
(`.codexclaw/evidence/01a1cbb2-fe9c-7e60-ba5c-bbe64a2a2346/test-receipt.json`, checkEpoch c-20261111205332-9563b4): kache doctor;
broad macOS check; core lib 1811; core int 93 (24 binaries); cursor-overlay 153; platform-macos lib 2000 (48 ignored);
platform-windows host 203; cargo deny; cargo build; check_protocol.py; proto_delta vs 0.40.1-goat.1 (65 = 65, problems 0).
Linux and Windows target type checks: exit 0 (stand-in C tools).

Unresolved / not applicable: upstream `connect`, release-profile strip/LTO, macOS type_text tail retype and 500 ms give-up,
web-focus request retries (reasons above); review A2/A3/A5/A6 residuals (above).
NOT RUN: Linux/Windows linking and tests, clippy for those targets, live macOS fixture runs (no GOAT-owned fixture run in wp1).
Next: wp1b (011), then wp2/wp3 per 012.

## Pre-D stale check (2026-11-11 ~18:50 UTC)

`git fetch origin main`: `origin/main` still `40c6f6ebf824bccbbbfe1d68cbd17a2085701167`. vercel-labs/agent-desktop `main` still
`54855b6225aec00da4ac65e2d0362fd41cd9fcb3`. trycua/cua `main` moved to `9e7256bfbb5ded8dc800484e43846b2df938af6d` (8 commits,
cua-driver-rs 0.47.13; 51 files differ under `libs/cua-driver`; vendored telemetry, teleport targets, sandbox images and
`libs/cua/Cargo.toml` are byte-identical). Added as goal unit **wp1b** (011), applied as a second synthetic 3-way step on top of the
committed wp1 merge so each step keeps its own preservation table.

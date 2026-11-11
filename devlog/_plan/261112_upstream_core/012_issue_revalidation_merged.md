# 012 — Issue re-validation on the merged tree (wp1 working tree, before the wp1 commit)

Source: research child `cu-wp24-revalidate-s24-261112` (codex / system24/deepseek-v4.1-flash / high), read-only, completed
2026-11-11 ~20:35 UTC. Line numbers are on the merged working tree (HEAD `5b07f0b4` + uncommitted 566a4ce6 → c0d3b919 merge).
Lead spot-checks (fresh): click.rs:6966-6968 inverted use confirmed (IllegalArgument / InvalidUIElement →
`StopUncertain(Reason::Unknown)` at ax/write_outcome.rs:56 → "Sent … it may have acted" while `unresolved_perform_error`
(click.rs:6346-6365) bails for the same errors); fast_path/store.rs and wire.rs byte-identical to upstream (`cmp`);
evaluate.rs:130-145 gates `visible` and `offscreen` Unknown as transient; semantic.rs:150 `Scope::WindowInternal`;
`a_visible_resting_default_cursor_parks_until_idle_fade` last present before 87e2e869 (#290). Line numbers move again when wp1b
lands; each wp re-reads its sites at P.

Disposition summary (all 8 still present after the sync; upstream changed none of them except #287, which upstream introduced):

| Issue | Status on merged tree | Plan change |
|---|---|---|
| #299 | present; 10 bare `resolve(pid, &args)` sites; store already has `*_with_instance` + `resolve_for_instance` | 020 §B: wiring only; add `refresh_screenshot_for_session` (get_window_state.rs:937) |
| #288 | present; store/wire byte-identical to upstream | 020 §C unchanged |
| #287 | present (upstream behavior) | 030 §B: add the parking assertion in render_map.rs / render_state.rs (the old GOAT test was deleted in #290) |
| #286 | present; refusal click.rs:2525-2537, unchecked relocation :2593-2598 | 030 §A line refs only |
| #283 | present; dead `hand_back` at click.rs:6819 only (live one at :4583) | 030 §C scope the cleanup |
| #281 | present; `selection_not_settable` absent; `_site` click.rs:3429 | 030 §C line refs |
| #279 | (1) present, hotkey.rs:152-163; (2) real defect at click.rs:6966-6968 (inverted predicate use), pre-merge | 030 §C: fix that branch, keep the helper name |
| #272 | present; offscreen Unknown at WindowInternal also burns the budget | 040 §A: decide unknown-offscreen for semantic delivery |

## Child report (verbatim)

## Re-validation on the merged tree — HEAD `5b07f0b4` + uncommitted upstream merge (cua-driver `566a4ce6 → c0d3b919`)

All paths below are under the worktree root; `src/` means `rust/crates/platform-macos/src/`. Upstream comparison tree: `/tmp/goat-cu-upstream/c0d3b919…/libs/cua-driver`. One tooling caveat up front: `src/apps/mod.rs` contains raw NUL bytes (an upstream test fixture at `src/apps/mod.rs:1324` writes `\x00` literally), so `rg`/`diff` report it as binary — use `rg -a` / `diff -a`.

### #299 — process instance beside each app pid

**Source facts.** The defect is present. `process_identity.rs` is still a GOAT-only file with the supplier disconnected: `src/process_identity.rs:23-28` ("U33B will connect this supplier…" plus `#[allow(dead_code)]` on `process_fingerprint`). The kernel start-stamp reader is available at `src/ax/enablement.rs:71-75` (`process_start_stamp`) and `:89-98` (`process_start_stamp_with`, `pbi_start_tvsec`/`pbi_start_tvusec`).

Output shape is unchanged: `AppInfo` at `src/apps/mod.rs:24-46` still has `name, pid, bundle_id, running, active, launch_path, kind, last_used` and no instance field. `app_record_json` at `src/tools/list_apps.rs:69-85` emits exactly those keys. `window_record_json` at `src/tools/list_windows.rs:192-210` emits `pid` at `:195`. `tree_output_fields` at `src/tools/get_window_state.rs:1617-1649` emits `pid` at `:1633` and no instance field.

Every macOS snapshot-token publication and resolution site:

| Kind | Site |
|---|---|
| Publish (screenshot-only refresh, keeps id/tokens) | `src/tools/get_window_state.rs:937-943` → `refresh_screenshot_for_session` |
| Publish (tree + screenshot) | `src/tools/get_window_state.rs:956-963` → `publish_sized_for_session` |
| `publish_for_session` | `src/tools/mod.rs:2473` only, inside `#[cfg(test)] mod pixel_focus_readback_tests` (`:2449`) — not production |
| `Snapshots::publish` | tests only: `src/tools/get_window_state.rs:3284`, `src/tools/element_pointer_tests.rs:771` |
| Resolve | `src/tools/click.rs:1540`, `scroll.rs:233`, `set_value.rs:133`, `double_click.rs:132`, `type_text.rs:332`, `press_key.rs:736`, `hotkey.rs:334`, `select_text.rs:356`, `type_text_chars.rs:228`, `right_click.rs:316` — ten call sites, all bare `resolve(pid, &args)`, none carrying process evidence |

`rg 'resolve_for_instance|_with_instance' src/` returns nothing, so the macOS crate has not adopted the protected APIs. The Windows binding pattern still exists in full: `rust/crates/platform-windows/src/process_instance.rs:146-157` (`publish_window_snapshot`), `:159-181` (+basis), `:183-204` (capture+basis), `:206-214` (`resolve_with_evidence`), plus `:130-134` probe capture and `:137-142` timeout capture. The merged core already carries the primitives it wraps: `snapshot_store.rs:579` `publish_for_session_with_instance`, `:602` `publish_sized_for_session_with_instance`, `:1313` `resolve_for_instance`, with `resolve` at `:1302` delegating `InstanceEvidence::NotChecked`, and the refusal codes at `:44-69` (`process_instance_unverified`, `process_instance_mismatch`).

**Upstream change.** None relevant. Upstream has no `process_identity.rs`, no `process_instance` field in `apps/mod.rs`, `list_apps.rs`, or `list_windows.rs`, and no `_with_instance`/`resolve_for_instance` in its `snapshot_store.rs` (only `publish_sized_for_session` at `:487`). `diff -a` of `apps/mod.rs` shows the merge brought enumeration changes only — `path_of_pid` made `pub(crate)`, the `LSBackgroundOnly`/`LSUIElement` bundle split, kernel running-state merge — plus GOAT-side scan caching (`scan_installed_apps_cached`, `INSTALLED_CACHE_TTL`) that came from HEAD, not upstream. `list_apps.rs` differences are the tool description rewrite and the retained HEAD inventory-failure handling. None of it adds process identity.

**Plan status.** 020 §A/§B still correct, with three corrections (below): the `refresh_screenshot_for_session` site is missing from §B, the store APIs §B says to add already exist, and 020 §A should name `list_windows.rs:192` / `get_window_state.rs:1617` explicitly since both moved.

**Analysis.** The residual risk the plan targets is intact: a token minted for a pid that exited and was reused still resolves on macOS, and `refresh_screenshot_for_session` (`snapshot_store.rs:786-812`) mutates the retained snapshot in place without touching any instance field, so a screenshot-only read preserves stale identity along with the tokens.

### #288 — fast-path store is daemon-wide

**Source facts.** Present and untouched. `fast_path/store.rs` keys snapshots by `u32` id (`:58` `put_snapshot`) and images/windows by `(i32, u64)` pid/window (`:38-46` state, `:125` `put_window`, `:140` `put_image`). None of the accessors take a session: `element` `:70`, `stats` `:82`, `latest_stats` `:89`, `element_at` `:101`, `window` `:135`, `image` `:157`, `forget_window` `:162`. `global()` at `:171-174`. Readers: `fast_path/wire.rs:175` (`latest_stats`), `:213` (`element`). macOS publication/reads: `src/fast_path_guard.rs:77` (`put_snapshot`), `:108` (`put_image`), `:134` (`put_window`), `:395` (`element`), `:481-493` (`image`/`window`), `:504` (`latest_stats`), `:510` (`element_at`). Mode still defaults off: `fast_path/mod.rs:127-142` with `ENV_VAR = "CUA_DRIVER_FAST_PATH"` at `:128` and `unwrap_or(FastPathMode::Off)`.

**Upstream change.** None. `cmp` of both `store.rs` and `wire.rs` against upstream is byte-identical, and `git show HEAD:` diffed against the working tree is empty for both.

**Plan status.** 020 §C is accurate as written; every cited line still matches (`:77`, `:108`, `:134`, `:395`, `:510`, `wire.rs`'s two readers). Only the `delta_target` line span drifted (`wire.rs:199-215` → `:196-215`, `element` at `:213`).

### #287 — resting cursor float never parks

**Source facts.** Present. `has_resting_motion` at `rust/crates/cursor-overlay/src/render_state.rs:377-402`, with the float branch returning `true` unconditionally at `:390-392` (`options.float && default_theme`). `needs_frame_tick` at `:524-531`, including `has_resting_motion()` at `:529`. `rest_amplitude` at `cursor_pose.rs:93-97`: eases out, holds at 0 for `FLOAT_HOLD_SECS`, eases back to 1 and stays there. Constants: `FLOAT_SECS = 4.0` at `cursor_pose.rs:24`, `FLOAT_HOLD_SECS = 0.5` at `:37`, `FLOAT_EASE_IN_SECS = 0.8` at `:39`, `FLOAT_EASE_OUT_SECS = 0.2` at `:41`. `float_hold_secs` field at `render_state.rs:133`, initialized to `FLOAT_HOLD_SECS + FLOAT_EASE_IN_SECS` at `:241-242`, advanced at `:776`, reset on arrival at `:784`, reset by `settle_float` at `:451-460`, reset by a rest cue at `:994`.

**Upstream change.** Upstream introduced the always-ticking behavior. `cursor_pose.rs` and `render_state.rs` are byte-identical to upstream; the only `cursor-overlay/src` difference is `render_map.rs`, where the merge added a looping-theme setup to `resting_motion_stops_for_reduced_motion_hidden_and_faded_cursors`.

**Plan status.** 030 §B's behavioral fix still applies and its API references (`FloatOffset::ZERO`, `settle_float`, `float_amplitude_at` at `:423-433`) are valid, but its test instruction is wrong. `a_visible_resting_default_cursor_parks_until_idle_fade` does not exist anywhere in the tree — `git log -S` shows commit `87e2e869` (#290, the previous upstream sync) deleted it. The current replacements are `render_map.rs:729-752` `a_visible_resting_cursor_keeps_ticking_for_its_resting_motion` (asserts ticking and `assert_ne!` between resting frames) and `render_state.rs:3417-3431` `the_resting_cursor_floats_unless_float_is_off` (asserts `has_resting_motion()` true at rest). Both sit inside the proposed window (`hold` = 0 or 1.3 s, window ≈ 9.3 s), so they keep passing as the plan claims — but the new parking assertion must be added in `render_map.rs`/`render_state.rs`, not "restored". One positive finding for the plan: `pose_stream.rs:265-271` (`RestFloat` carrying `hold_secs`) and `:274-286` (`pose_look`) inherit the fix through `rest_amplitude` itself, so streaming surfaces park without a second change.

### #286 — pixel relocation after the frame-bounds check

**Source facts.** Present. The background frame-bounds refusal is `src/tools/click.rs:2525-2537` inside the block at `:2517-2540`; `frame` is in scope from `window_points::frame_or_resolve` (`src/tools/window_points.rs:410-418`) and the refusal uses `frame.bounds` at `:2531-2536`. `fast_path_guard::guard_pixel` is called at `:2579-2602`, and the relocated point is added unchecked at `:2593-2598` (`win_local_x + dx, win_local_y + dy`). `guard_pixel` has exactly one caller (`click.rs:2579`); `double_click.rs:305` delegates pixel clicks through `ClickTool`, and `right_click.rs` does its own frame check at `:521-530` but never calls `guard_pixel`.

**Upstream change.** No fix. Upstream has the same sequence at `click.rs:1131-1154` (bounds check) and `:1194-1215` (relocation), and it never gained GOAT's helper form or the re-check.

**Plan status.** 030 §A remains correct; only line numbers moved (`:2470-2489` → `:2517-2540`, `:2527-2554` → `:2576-2605`). The snippet's `super::invalid_arguments` matches the current call at `:2529`. The `grep at P` for `double_click`/`right_click` resolves to "not needed".

### #283 — native text focus gate follow-ups

**Source facts.** Both halves are present.

New refusals (intended fail-closed): `focus_capability` at `src/tools/click.rs:3576-3586` returns errors for `Known(false)` → `"AXFocused is not settable"` (`:3582`), `Absent` (`:3583`), and `Unknown(reason)` (`:3584`); it is wired at `:3639` (`focus_capability_refusal(NativeReader.settable(ptr, "AXFocused"))` inside `prepare_native_focus`), `:4832` (hit-focus), `:4888`, and `:5362`. Disabled `AXPress` refuses through `crate::input::ax_actions::ensure_ax_action_enabled` (`src/tools/click.rs:3635-3637`, `:4073-4077`), implemented at `src/input/ax_actions.rs:558-565` with `ensure_ax_enabled` at `:528-533` and `disabled_action_refusal` at `:536-551` (foreground message vs `background_unavailable`). Unknown AXPress capability refuses at `:4065-4070`. Selection requires readable settable `AXSelected` at `:4425-4433`. Tests covering the refusal paths: `native_focus_unknown_capability_never_falls_through` at `:5357-5380`, `hit_focus_unknown_capability_never_falls_through` at `:5147-5172`, `native_focus_refusal_preserves_prior_input` at `:5382`, plus `:8132-8160`. `rust/CHANGELOG.md` has no GOAT entry — the file now opens at `0.47.12` (upstream) and the only GOAT block is `0.37.0-goat.3` at `:507`.

Dead `hand_back`: `src/tools/click.rs:6819` `let hand_back: Option<String> = None;` with no-op uses at `:6824`, `:6844`, `:6867`. Upstream's `click.rs` contains no `hand_back` at all. **But** a live `hand_back` with real scope text exists at `:4583-4587`, used meaningfully at `:4589`, `:4602`, `:4624` — so the cleanup must target only the `:6819` declaration.

**Upstream change.** Upstream's `focus_element` swallowed AX errors (`ax_actions.rs:295-304` upstream), so the merged strict version is GOAT's. Upstream also lacks `security_prompt.rs` (GOAT-only, `src/security_prompt.rs`, called at `click.rs:41`, `:1522`, `:1919`, `:2372`, `:3342`) and lacks `focus_capability` entirely.

**Plan status.** 030 §C rows for #283 are valid, with one precision fix: the removal must be scoped to `:6819` and must not disturb `:4583`.

**Analysis.** The refusal wording is now duplicated in three surfaces (native focus capability, hit focus capability, selection settability) and none of it is in the CHANGELOG, so item 1 remains genuinely open. Live smoke on the AppKit fixture is still NOT RUN.

### #281 — click selection gate follow-ups

**Source facts.** Present. `SelectionSite` at `src/tools/click.rs:3421-3426` (`First, PostPress, NativeFocus, HitFocus`); `dispatch_post_press_selection` declares `_site: SelectionSite` unused at `:3428-3429`; the wrapper at `:3499` passes it through to `run_post_press_selection`, whose only consumer is `run_live_selection_write`'s `matches!(site, SelectionSite::First)` at `:4457`. Call sites: `:2020`, `:3529`, `:4263`, `:4695`, `:4734`, `:5316`, `:5427`, `:5470`, `:5550`, `:5585`.

Unreadable action list already refuses: `SelectionDiscovery::Unknown(reason)` → `post_press_failure(… "selection target is unknown: {reason}")` at `:4231-4236`; `Absent` at `:4225-4230`; the discovery walk's unknown reason is set in `src/input/ax_actions.rs:342-349`.

Readable-but-not-settable is conflated: the write preconditions at `:4425-4433` require `NativeReader.settable(target_ptr, "AXSelected")` to be `Known(true)`; `recheck_selection_then_write` (`:4178-4183`) then returns `None`, which projects to the `(Some(None), _)` arm at `:4369-4375` and the message `"selection target changed or became unavailable"` at `:4373`. `rg 'selection_not_settable'` finds nothing in the crate.

**Upstream change.** Upstream has selection text but no `_site`/`SelectionSite`, no settable precondition at that position, and no `selection_not_settable`; the ambiguity is GOAT-side.

**Plan status.** 030 §C rows #281.1–#281.3 remain correct; line numbers moved (`:4312-4317` → `:4369-4375`, `:3372` → `:3429`, call sites `:1974`/`:3472`/`:4206` → `:2020`/`:3526-3529`/`:4263`).

### #279 — hotkey focus fails fast, and the "may have acted" predicate

**Source facts.** Retry issue present: `src/tools/hotkey.rs:152-163` is still `loop { focus_element(element_ptr)?; if is_element_focused(...) … }` with `HOTKEY_FOCUS_TIMEOUT = 200 ms` at `:134` and `HOTKEY_FOCUS_POLL_INTERVAL = 10 ms` at `:135`; callers `:457`, `:730`, `:749`. `focus_element` is strict at `src/input/ax_actions.rs:826-843`, returning `FocusWriteError { error, outcome }` on any non-success.

The predicate itself matches its name: `perform_error_proves_nothing_was_sent` at `src/tools/click.rs:6271-6281` returns true only for `kAXErrorIllegalArgument`, `kAXErrorInvalidUIElement`, `kAXErrorAPIDisabled`. Negated uses are correct (`double_click.rs:350`, `right_click.rs:1623`, `press_key.rs:1902`, `:2210`), and `unresolved_perform_error` uses it correctly at `:6351-6353` (bail for nothing-sent, otherwise "may have acted").

**One real defect at the reviewer's site.** `src/tools/click.rs:6966-6968` uses the predicate positively and returns `"⚠️ Sent {ax_action} … the app returned AX error {err}; it may have acted."` — i.e. when the error proves nothing was sent, it tells the caller the action was sent. It is reachable: `native_write_outcome` (`src/ax/write_outcome.rs:19-58`) maps `IllegalArgument`/`InvalidUIElement` to `_ => StopUncertain(Reason::Unknown)` at `:56`, and the preceding guards only intercept `StopNotDelivered` (`:6930-6938`) and `FallThrough` (`:6957-6964`), so those two codes land on `:6966`. The identical inversion exists at HEAD (`HEAD:click.rs:6907-6908`), so it predates the merge.

**Plan status.** 030 §C #279.1 stands; its cited `ax_actions.rs:801-818` is stale (now `:826-843`). #279.2 should be kept and re-pointed: the found defect is the inverted use at `click.rs:6966-6968`, so the fix is to route that branch through `unresolved_perform_error` (or negate it), not to rename the helper. That is a substantive addition to the plan.

### #272 — AXPress actionability refuses on unreadable visibility

**Source facts.** Present. `src/tools/actionability.rs:272` still maps `AxPress | ScrollButton | NativeMenu => (Intent::Click, Policy::headless())`. Core `rust/crates/cua-driver-core/src/actionability/evaluate.rs:113-129` computes `semantic_pointer`, `semantic_nonpointer`, `without_pointer`, and `geometry_needed`, but `:130-131` still calls `gate(&mut checks, "visible", &facts.visible, false)` unconditionally; `gate` at `:60-84` turns `Unknown` into `CheckStatus::Transient { unknown: true }` (`:77-83`), and the offscreen gate at `:132-145` independently transitions on `Known(true)` and `Unknown` (`:139-144`). `requirements.rs:50` makes `Click` require `(visible=true, stable=true, enabled=true, …, receives_events=true)`.

macOS adapter scope is `Scope::WindowInternal` for semantic delivery (`src/tools/actionability/semantic.rs:150`), and at that scope `visible` derives from `hidden`/`minimized`/`clipping` (`src/tools/actionability/scope.rs:331-343`, `unknown("clipping or ancestry unavailable")`) while `offscreen` is clipping-only (`:345-349`, `unknown("clipping or ancestry unavailable")`).

**Upstream change.** None — upstream has no `actionability` module at all (`rust/crates/cua-driver-core/src/actionability/` absent), and `git show HEAD:` of `evaluate.rs` matches the working tree at `:130-131`.

**Plan status.** 040 §A's core snippet is still the right shape and its `:130-131` reference is exact, but the plan's guard `if !matches!(facts.offscreen, Evidence::Known(true))` is insufficient: because semantic delivery runs at `WindowInternal`, `offscreen` is `Unknown` whenever clipping is unreadable, and the independent gate at `:139-144` then still waits out the 5 s budget. The rule needs a decision for unknown offscreen in semantic delivery (known hidden/offscreen must still refuse). Its agent-desktop references (`gates.rs`, `requirements.rs`) are external sources, not files in this tree.

## Plan corrections

1. **020 §B must add `refresh_screenshot_for_session`** (`get_window_state.rs:937`, store `snapshot_store.rs:786-812`). It preserves the snapshot id and tokens, so a screenshot-only read after a process restart keeps a stale instance; the plan currently covers only `publish_sized_for_session` (`:956`).
2. **020 §B: the store APIs already exist.** `publish_for_session_with_instance` (`:579`), `publish_sized_for_session_with_instance` (`:602`), `resolve_for_instance` (`:1313`), refusals `:44-69`. The work is wiring in `platform-macos`, not new store code — mirroring `platform-windows/src/process_instance.rs:146-214`.
3. **020 §B: ten resolve sites, enumerated with current lines** — `click.rs:1540`, `scroll.rs:233`, `set_value.rs:133`, `double_click.rs:132`, `type_text.rs:332`, `press_key.rs:736`, `hotkey.rs:334`, `select_text.rs:356`, `type_text_chars.rs:228`, `right_click.rs:316`.
4. **020 §A: relocated sites** — `app_record_json` `list_apps.rs:69`; `window_record_json` `list_windows.rs:192` (pid `:195`); `tree_output_fields` `get_window_state.rs:1617` (pid `:1633`). Use `rg -a` on `apps/mod.rs` (raw NULs at `:1324`).
5. **030 §B: replace the test instruction.** `a_visible_resting_default_cursor_parks_until_idle_fade` was deleted by `87e2e869` (#290). Add the parking assertion to `render_map.rs:729-752` / `render_state.rs:3417-3431`; both existing tests sit inside the window and keep passing. Note that `rest_amplitude` alone carries the fix to `pose_stream.rs:265-271`.
6. **030 §A: refresh line refs** — check `click.rs:2517-2540` (refusal `:2525-2537`), relocation `:2576-2605` (add at `:2593-2598`).
7. **030 §C #279.1: refs moved** — `focus_element` is `ax_actions.rs:826-843` (not `:801-818`); `focus_hotkey_element` `hotkey.rs:152-163`.
8. **030 §C #279.2 is real, and points at a different defect than stated.** The predicate is used correctly at `click.rs:6351`; the defect is the inverted positive use at `click.rs:6966-6968` (returns "Sent … it may have acted" when the error proves nothing was sent), reachable via `IllegalArgument`/`InvalidUIElement` → `StopUncertain(Reason::Unknown)` (`ax/write_outcome.rs:56`). Fix that branch; do not rename the helper.
9. **030 §C #283.2: scope the cleanup** to `click.rs:6819` (uses `:6824`, `:6844`, `:6867`) and leave the live `hand_back` at `:4583`/`:4589`/`:4602`/`:4624` alone.
10. **030 §C #281: relocate refs** — projection `click.rs:4369-4375`, settability `:4425-4433`, `_site` `:3429`, call sites `:2020`/`:3526-3529`/`:4263`. `selection_not_settable` is still absent.
11. **040 §A: incomplete rule.** Because `semantic.rs:150` uses `Scope::WindowInternal`, `offscreen` is `Unknown` when clipping is unreadable and the independent gate at `evaluate.rs:139-144` still burns the budget. The plan must decide unknown-offscreen treatment for semantic delivery (known hidden/offscreen keeps refusing) — its current `Known(true)` guard alone does not fix #272.
12. **040 §A external-reference note.** `gates.rs`/`requirements.rs` are agent-desktop sources, not files in this worktree or upstream; keep them labeled as external.

Unchanged and still accurate: 002's dispositions for #288 (byte-identical files), #286 (upstream has the same gap), #287 (upstream introduced the behavior), #283/#281/#279 refusal inventory, and #299's dead-supplier diagnosis.

**NOT RUN:** so true. No cargo, no builds, no live smoke on the AppKit fixture, no Linux/Windows review (other children own those paths) — prohibited by the read-only source-research scope. No files were edited; only `rg`/`sed`/`cmp`/`diff`/`git show`/`git diff`/`git log` were used.

Note on process: the `$goat-discipline` skill directory is not present in this environment (`~/.goat/userdata/skills` holds only `browser`, `computer-use`, `screen-code`), so I applied the discipline rules from the operator guidance block in context; nothing about the read-only, evidence-with-path:line, or `NOT RUN` requirements changed.

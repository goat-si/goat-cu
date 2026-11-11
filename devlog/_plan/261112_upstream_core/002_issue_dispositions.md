# 002 — Issue re-validation on base 40c6f6eb (pre-sync)

Each row is re-checked on the synced tree in the owning work phase's P (stale check); line numbers below are at `40c6f6eb`.
"Upstream c0d3b919" says whether the target upstream touches the same code.

## #299 — process instance with each app pid (feature, protocol-additive)

- Claim: `list_apps`, `list_windows`, `get_window_state` name apps by pid only; GOAT cannot tell pid reuse (goat-si/goat#2347).
- Confirmed. `rust/crates/platform-macos/src/tools/list_apps.rs:69-84` (`app_record_json`) emits pid, name, bundle_id, active,
  running, launch_path, kind, last_used. `AppInfo` (`rust/crates/platform-macos/src/apps/mod.rs:24-46`) has no instance field.
  `list_windows.rs:195` emits `pid` only. The kernel start stamp reader already exists:
  `rust/crates/platform-macos/src/ax/enablement.rs:69-96` (`process_start_stamp`, `pbi_start_tvsec/usec`), and a
  double-read fingerprint `rust/crates/platform-macos/src/process_identity.rs:25-49` (marked dead code, "U33B will connect").
- Upstream c0d3b919: no instance field added (apps/mod.rs changes are an installed-app scan cache and `*_without_spaces`).
- Related safety gap found while checking: macOS snapshots are published without a process instance
  (`tools/mod.rs:2337` `publish_for_session`, `get_window_state.rs:956` `publish_sized_for_session`) and resolved with
  `resolve(pid, &args)` (click.rs:1535, scroll.rs:230, set_value.rs:133, double_click.rs:132, type_text.rs:326, press_key.rs:727,
  hotkey.rs:327, select_text.rs:356, type_text_chars.rs:222, right_click.rs:316), while Windows binds them
  (`platform-windows/src/process_instance.rs:144-213`, U15 #260). A token minted for a process that exited and whose pid was
  reused resolves on macOS. Tracked as AD-G2 (003) and fixed in wp2 with #299.
- Plan: wp2 (020 §A, §B). Additive optional output field `process_instance`; protocol record in wp2 D.

## #288 — fast_path target store is daemon-wide

- Claim: `fast_path::store::global()` lookups are not session-scoped.
- Confirmed. `rust/crates/cua-driver-core/src/fast_path/store.rs:37-46` keys snapshots by `u32` id and images/windows by
  `(pid, window_id)`; `global()` at :170-174. Lookups: `wire.rs:175` (`latest_stats`), `wire.rs:199-215` (`delta_target` resolves
  any token's pid/window), `platform-macos/src/fast_path_guard.rs:395` (`element`), :510 (`element_at`).
  Publication: fast_path_guard.rs:77, :108, :134. Mode is off by default; GOAT does not enable it.
- Upstream c0d3b919: `fast_path/` unchanged in the range (byte-identical).
- Plan: wp2 (020 §C). Record the owning session on every entry; lookups take the caller's `_session_id` and miss across sessions.

## #287 — cursor.float replaced GOAT idle parking

- Claim: the default cursor keeps ticking at rest because `has_resting_motion()` is true while `cursor.float` is on.
- Confirmed. `rust/crates/cursor-overlay/src/render_state.rs:377-402` returns true for float + default theme until the idle
  fade; `needs_frame_tick` (:524-531) therefore stays true at rest. Float amplitude at rest: `cursor_pose::rest_amplitude`
  (`cursor_pose.rs:90-97`) eases out, holds, eases back to 1 and never parks.
- Upstream c0d3b919: cursor-overlay changed only `theme_artifact.rs` (+1); behavior unchanged.
- Plan: wp3 (030 §B), issue option 2 inside the driver: after a bounded rest window the float eases to amplitude 0 (upright, no
  mid-tilt freeze) and `has_resting_motion()` turns false, so the overlay parks until the idle fade. Option 1 (host sets
  `cursor.float:false`) is a GOAT app change and out of scope.

## #286 — fast-path pixel relocation after the background frame-bounds check

- Confirmed. `rust/crates/platform-macos/src/tools/click.rs:2470-2489` refuses a background window-local point outside
  `frame.bounds`; `click.rs:2527-2554` then applies `relocate: Some((dx, dy))` from `fast_path_guard::guard_pixel` without a
  re-check. Results report the original point.
- Upstream c0d3b919: upstream click.rs keeps the same relocation (:1194-1215) and has no GOAT bounds check.
- Plan: wp3 (030 §A). Extract the bounds rule into a helper, re-run it on the relocated point for background delivery.

## #283 — native text focus gate follow-ups (#280)

1. New refusals for non-settable AXFocused / disabled AXPress: intended fail-closed behavior; needs a live smoke and a
   changelog note. Live smoke runs only on the GOAT-owned AppKit fixture; otherwise NOT RUN with reason.
2. Dead `hand_back`: confirmed at `click.rs:6760` (`let hand_back: Option<String> = None;`, used at :6765, :6785, :6808 only
   to format nothing). Cleanup (C1): remove the variable and its no-op formatting.
- Plan: wp3 (030 §C).

## #281 — click selection gate follow-ups (#278)

1. Unreadable action list now refuses selection: intended fail-closed; record in CHANGELOG.
2. Readable-but-not-settable AXSelected reports "selection target changed or became unavailable" (`click.rs:4312-4317`
   via `post_press_failure`). Confirmed: the `None` branch at :4312 covers both "target vanished" and "not settable".
   Fix: distinct message and structured code `selection_not_settable` (protocol-visible error code; additive).
3. `_site` unused: confirmed `click.rs:3372` in `dispatch_post_press_selection`; callers pass `SelectionSite::First` (:1974)
   and `PostPress` (:3472, :4206). Wire it into the refusal naming or drop the parameter (C1).
- Plan: wp3 (030 §C).

## #279 — focus_element hard-fail makes hotkey focus fail fast (#277)

1. Confirmed. `rust/crates/platform-macos/src/tools/hotkey.rs:151-163` `focus_hotkey_element` calls
   `focus_element(element_ptr)?` (`input/ax_actions.rs:801-818`), which now returns `Err` on any AXError, so the poll loop
   exits on the first transient error. An `AXFocused=true` write is idempotent, so retrying until `HOTKEY_FOCUS_TIMEOUT` is
   safe; only errors that prove the element or API is gone (`kAXErrorInvalidUIElement`, `kAXErrorAPIDisabled`,
   `kAXErrorIllegalArgument`) should stop early.
2. `perform_error_proves_nothing_was_sent` (`click.rs:6214-6225`) name vs. use: callers negate it to mean "may have acted"
   (double_click.rs:329, right_click.rs:1619, press_key.rs:1889). The predicate matches its name; re-check the reviewer's
   cited site on the synced tree and either rename or close as not reproducible with path:line evidence.
- Plan: wp3 (030 §C).

## #272 — AXPress actionability gate refuses when visibility is unreadable

- Confirmed. `rust/crates/platform-macos/src/tools/actionability.rs:272` maps AXPress to `(Intent::Click, Policy::headless())`.
  Core `rust/crates/cua-driver-core/src/actionability/evaluate.rs:113-114` computes `semantic_pointer`, but
  :131 `gate(&mut checks, "visible", &facts.visible, false)` still requires `visible == Known(true)`; `Unknown` is a transient
  check that waits out the budget and refuses (issue's 5 s refusal).
- Agent-desktop `54855b62` does not require pointer visibility for strictly headless semantic delivery:
  `crates/core/src/actionability/requirements.rs:59-97` (`requires_geometry`), :99-116 (`delivered_without_pointer`),
  `gates.rs:11-66` (`visibility`: only explicit hidden/offscreen fails; bounds-present or visible-without-bounds pass).
- Plan: wp4 (040 §A, AD-G1).

## Neighbouring open issues touching the same code (not in this lane's set)

#276 (select_text secure-field code), #268 (old folding function on Windows/macOS), #264 (set_value re-minimize message),
#262 (locator malformed_completeness), #258 (actionability adapter hard-codes visibility/hit-test unknown — overlaps #272),
#256 (since-diff states-only changes), #249, #246, #242, #231 (live process evidence into resolve — overlaps AD-G2),
#228. Only #258 and #231 are folded where they share the exact code (040 §A, 020 §B); the rest stay open.

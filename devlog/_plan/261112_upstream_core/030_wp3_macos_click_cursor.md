# 030 — wp3: macOS click ordering, cursor rest parking, gate follow-ups (#286, #287, #279, #281, #283)

> Re-validated on the merged tree in 012: §A line refs moved; §B's "restore" test was deleted in #290, add the parking assertion instead; §C #279.2 is a real defect at click.rs:6966-6968 (inverted predicate use, pre-merge), #283.2 cleanup is click.rs:6819 only.

Stale check at wp3 P: `tools/click.rs` (20 trial conflict hunks) and `tools/hotkey.rs` move in wp1; re-locate every block.

## §A — #286: re-check a relocated background point (click ordering)

Before (40c6f6eb `click.rs:2470-2489`, `:2527-2554`): bounds rule inline on the translated point; relocation applied later
without a re-check.

- MODIFY `click.rs`: extract

```rust
/// A background window-local point must lie inside the live frame (the #2237 wrong-surface class).
fn background_point_outside_frame(lx: f64, ly: f64, bounds: &CGRectLike) -> bool {
    lx < 0.0 || ly < 0.0 || lx > bounds.width || ly > bounds.height
}
```

  use it at the existing check, keep `frame` in scope, and after `Outcome::Proceed { relocate: Some((dx, dy)) }` with
  `!delivery_mode.is_foreground()`:

```rust
if background_point_outside_frame(win_local_x + dx, win_local_y + dy, &frame.bounds) {
    return super::invalid_arguments(format!(
        "click: fast-path relocation moved the window-local point to ({:.1}, {:.1}) pt, outside window {wid}'s \
         {:.0}×{:.0} pt frame; background delivery refused. Nothing was sent.", ...));
}
```

  and report the delivered (relocated) point in the result alongside the requested one.
- Same pattern in `double_click.rs` / `right_click.rs` if they call `guard_pixel` (grep at P).
- Test (failing first): `click.rs` tests — `a_relocated_background_point_outside_the_frame_is_refused` against a pure helper
  `apply_relocation(point, relocate, bounds, foreground) -> Result<(f64, f64), String>` whose first version mirrors today's
  unchecked add (test fails), then gains the check.

## §B — #287: park the resting default cursor after a bounded float

- MODIFY `rust/crates/cursor-overlay/src/cursor_pose.rs`: add
  `pub const FLOAT_REST_SECS: f64 = 2.0 * FLOAT_SECS;` (float plays this long at full amplitude after easing in) and make
  `rest_amplitude(hold, from)` ease back out to exactly 0 over `FLOAT_EASE_IN_SECS` once
  `hold > FLOAT_HOLD_SECS + FLOAT_EASE_IN_SECS + FLOAT_REST_SECS`; add `pub fn rest_parked(hold: f64) -> bool`.
- MODIFY `render_state.rs` `has_resting_motion`: the float branch returns
  `!crate::cursor_pose::rest_parked(self.float_hold_secs)`; the parked pose is `FloatOffset::ZERO` (upright), so no mid-tilt
  freeze. Any new action already resets the hold via `settle_float`.
- Tests (failing first): `cursor_pose.rs` — `the_rest_amplitude_parks_at_zero_after_the_rest_window`;
  `render_state.rs` — restore GOAT's `a_visible_resting_default_cursor_parks_until_idle_fade` (resting default cursor with
  `idle_hide_ms = 20_000`: `has_resting_motion()` true inside the window, false and `float_offset() == ZERO` after it,
  `needs_frame_tick()` false until the idle fade). Upstream's `the_resting_cursor_floats_unless_float_is_off` keeps passing
  (it sits inside the window).
- Platform tests asserting cursor-float ticking at rest (Linux overlay placement, Wayland cadence, Windows overlay; see #290
  follow-up `712985ea`) are re-read; they cover the first seconds of rest and should hold. NOT RUN on this host.

## §C — gate follow-ups

| Item | Change | Test |
|---|---|---|
| #279.1 hotkey focus fails fast | `hotkey.rs` `focus_hotkey_element`: on `Err(FocusWriteError { error, .. })` stop only for `kAXErrorInvalidUIElement`, `kAXErrorAPIDisabled`, `kAXErrorIllegalArgument`; otherwise keep polling to `HOTKEY_FOCUS_TIMEOUT`. Extract `fn focus_retry_allowed(err: AXError) -> bool` | `hotkey.rs` tests: `focus_retry_allowed` table (cannot-complete → true; invalid element → false) |
| #279.2 helper name | re-check cited use on the synced tree; rename only if a caller uses it with the opposite meaning, else close with path:line | — |
| #281.2 not-settable message | `click.rs` selection worker: return a distinct outcome when AXSelected is readable but not settable; map to `post_press_failure(.., "selection target exposes AXSelected but it is not settable")` with structured `code: "selection_not_settable"` | `click.rs` tests: projection of that outcome carries the new code |
| #281.3 `_site` | drop the unused parameter (C1) or use it in the refusal name | compile |
| #281.1, #283.1 fail-closed refusals | CHANGELOG "GOAT" note; live smoke on the GOAT AppKit fixture if a fixture build is available, else NOT RUN | — |
| #283.2 dead `hand_back` | remove `let hand_back: Option<String> = None` and its no-op formatting in the web path (C1) | compile |

## Verification

`cargo test -p platform-macos --lib click:: hotkey::`, `cargo test -p cursor-overlay --lib`, `cargo check -p platform-macos --all-targets`.

## wp3 B/C evidence — §A and §C (lead), on HEAD ef7a3646

**#279.2 — corrected diagnosis.** 012 called `click.rs` "Sent … it may have acted" for a "proves nothing was sent" error an inverted
predicate. History says otherwise: dd5fc6db (#277) added that branch on purpose — its PR body: "IllegalArgument after a state change
stays uncertain and never claims a safe retry" — and the core write-outcome rule (`ax/write_outcome.rs` `native_write_outcome`)
classes a Perform's IllegalArgument/InvalidUIElement as `StopUncertain(Unknown)`. The real defect was the contradiction the issue names:
the same helper meant "never reached the app" in `unresolved_perform_error` (whose `bail!` branch was then unreachable from production
while two tests asserted it) and "may have acted" at the press site. Fix: `perform_outcome(err)` (click.rs:6049) wraps the core rule;
`unresolved_perform_error` (:6526) fails only on `StopNotDelivered` (accessibility disabled) and reports every other code unverifiable;
the press site's helper branch is removed, so production behaviour is unchanged and the helper keeps one meaning (its doc now names its
remaining users). `post_press_should_select` uses the same `perform_outcome`.
Follow-up (not one of the 8 issues, not changed here): press_key.rs:1903/:2211, right_click.rs:1624 and double_click.rs:351 still use
`perform_error_proves_nothing_was_sent` to choose a fallback after IllegalArgument/InvalidUIElement; under #277's rule that fallback can
follow an action that already ran. Needs its own failing-first change per tool.

**#279.1** hotkey.rs: `focus_with_retry` (:184) with `focus_retry_allowed` (:168) — a FocusWriteError other than InvalidUIElement,
APIDisabled or IllegalArgument keeps polling to `HOTKEY_FOCUS_TIMEOUT` (rewriting AXFocused=true is idempotent and the hotkey still waits
for the focus read); at the deadline the last write error is returned. `focus_hotkey_element` (:152) passes the live closures.

**#286** click.rs: `point_outside_frame` (:3435) shared by the requested-point check and `relocated_point` (:3443); the frame size is kept
from `frame_or_resolve` and the fast-path relocation (:2595) is refused in background when the moved window-local point leaves the frame
("… relocated the window-local point to (x, y) pt, outside window …; background delivery refused. Nothing was sent."). Foreground keeps
the moved point. `guard_pixel` has one caller; right_click/double_click do not relocate (012).

**#281.2** click.rs: `SelectionUnavailable { Changed, NotSettable }` (:4227); the live re-check returns `NotSettable` when AXSelected reads
but `settable` is Known(false) (:4499); the projection maps it to "selection target exposes AXSelected but it is not settable". No new
protocol code (message only). **#281.3**: `dispatch_post_press_selection` → `dispatch_gated_write` (:3472; it gates first-selection,
post-press, native-focus and hit-focus writes) and `site` is logged when a gate refuses (:3507). **#283.2**: dead `hand_back` in the
text-input focus path removed (`focus_step()?` :7003); the live hand-back (:4642-4683) is untouched. **#281.1 / #283.1**: fail-closed
refusals confirmed intended (#278, #280) and documented in GOAT.md "macOS click and focus refusals"; live smoke NOT RUN (no GOAT AppKit
fixture build in this lane).

| Command (rust/) | Exit | Result |
|---|---|---|
| `cargo test -p platform-macos --lib -- a_transient_focus_write_error_keeps_polling_until_the_deadline a_selection_target_that_is_not_settable_says_so a_relocated_background_point_outside_the_frame_is_refused perform_error_after_dispatch_is_unverifiable_unless_nothing_was_sent` (helpers mirroring today's behaviour) | 101 | 4 failed: click.rs:5623 relocation `left == right`; hotkey.rs:893 `-25204: Err(AXSetAttribute(AXFocused) returned -25204 …)`; click.rs:7918 `-25201 must not fail`; click.rs:5597 "…selection target changed or became unavailable" (/tmp/goat-cu-sync/wp3-red.log) |
| same + `an_ax_error_that_proves_nothing_was_sent_stays_a_plain_failure post_press_changed_target_after_gate_never_writes`, after the fixes | 0 | 6 passed |
| `cargo check -p platform-macos --all-targets` | 0 | no warnings |
| `cargo test -p platform-macos --lib -- tools::click:: tools::hotkey::` | 0 | 125 passed, 2 ignored |
| `cargo test -p platform-macos --lib lease_wiring` | 0 | 22 passed |
| `rustfmt --edition 2021 --check` click.rs / hotkey.rs vs the same on HEAD | — | identical remaining diffs (pre-existing only) |

## wp3 B/C evidence — §B #287 (child `cu-wp3-287-s24-261112`, lead-verified), on HEAD 09484534

Issue option 2 (change the rest curve), bounded as planned: `FLOAT_REST_SECS = 2 × FLOAT_SECS` (cursor_pose.rs:42). `rest_amplitude`
(:101) multiplies the old envelope by `1 − smoothstep((hold − REST_PARK_SECS) / FLOAT_EASE_IN_SECS)` (`REST_PARK_SECS` :94 = hold + ease-in +
rest), so a settled default cursor floats ≈8 s at full amplitude and eases to exactly 0; `rest_parked` (:112). `has_resting_motion`
(render_state.rs:403) returns `!rest_parked(float_hold_secs)`, so `needs_frame_tick` drops to false until the idle fade is due; every
motion restart zeroes `float_hold_secs` (settle_float, arrival, mid-move SnapTo) and the 3 600 s clamp stays parked. Parked pose is
`FloatOffset::ZERO` (upright, no mid-tilt freeze). Host-level `cursor.float:false` still parks at once (issue option 1, GOAT app side).
Lead addition: the GNOME Shell helper paints its own resting float from the look JSON (wayland-helper/winrects@cua/extension.js
`restAmplitude`), so it would keep floating after the Rust cursor parked. The look's float object now carries `park` (pose_stream.rs:187,
`FloatLook.rest_secs` :99) and the helper honours it (extension.js:111-123, :233-234); a payload without `park` (older driver) never parks,
an older helper ignores the extra field. The helper's 33 ms repaint timer is unchanged (pre-existing; it does not follow the float).

| Command (rust/) | Exit | Result |
|---|---|---|
| child: `cargo test -p cursor-overlay --lib` with `rest_parked` stubbed false | 101 | 154 passed, 2 failed: cursor_pose.rs:395 "parked upright" (1.0 vs 0.0), render_map.rs:810 "a parked cursor rests still" |
| lead: `cargo test -p cursor-overlay --lib -- the_json_carries_every_sample_and_the_look` before `park` | 101 | pose_stream.rs:719 "the look carries park" |
| lead: `cargo test -p cursor-overlay --lib` after | 0 | 156 passed (incl. `the_rest_amplitude_parks_at_zero_after_the_rest_window`, `a_visible_resting_default_cursor_parks_until_idle_fade`; upstream's `the_resting_cursor_floats_unless_float_is_off` and `a_visible_resting_cursor_keeps_ticking_for_its_resting_motion` unchanged) |
| child: `cargo test -p platform-macos --lib cursor` | 0 | 30 passed, 2 ignored |
| child: `cargo check -p cursor-overlay -p platform-macos --all-targets`; `cargo check -p platform-linux -p platform-windows --all-targets` (host) | 0 / 0 | — |
| `node --check` on a copy of extension.js | 0 | parses |

Pre-existing (not this lane): building cua-driver-core for cursor-overlay alone warns `LocatorRegistry::Borrowed` / `Locator::new` unused
(batch_tools/locate.rs:957/:977, test-only callers); identical on origin/main 40c6f6eb and in the wp1 receipt's overlay log.
NOT RUN: Linux/Windows overlay suites and the GNOME helper (no Linux/GNOME host); manual overlay observation.

## wp3 close-out on HEAD 019c3858

Paths are under `rust/crates/`; line numbers are at 019c3858. #272 is wp4 (040).

| Issue | Disposition | Code | Failing-first test |
|---|---|---|---|
| #286 | FIXED | `platform-macos/src/tools/click.rs:3439` `point_outside_frame` and `:3447` `relocated_point`, shared by the requested-point check and the fast-path relocation; refused before the cursor moves | `click.rs:5647` `a_relocated_background_point_outside_the_frame_is_refused` |
| #287 | FIXED (option 2: bounded float, then park) | `cursor-overlay/src/cursor_pose.rs:94` `REST_PARK_SECS`, `:101` `rest_amplitude`, `:112` `rest_parked`; `render_state.rs:386` `has_resting_motion`; macOS overlay parks without frame ticks and wakes at `idle_fade_wait` (`platform-macos/src/cursor/overlay.rs:773`) | cursor_pose.rs / render_map.rs / pose_stream.rs tests above; e354f982's macOS parking test |
| #279 | FIXED | focus retry: `platform-macos/src/tools/hotkey.rs:168` `focus_retry_allowed`, `:184` `focus_with_retry`. One meaning for AX perform errors: `click.rs:6167` `perform_outcome`; `press_key.rs:1923` `decide_chord`; `right_click.rs:1612` and `input/ax_actions.rs:526` `ActionNotDelivered` → `tools/mod.rs:107` `not_delivered_refusal`; `scroll.rs:1485`–`:1508` (a scroll-bar press that may have acted ends the call); `input/ax_actions.rs:641` `perform_ax_action` classified. Pixel fallbacks post only at `element_pointer.rs:254` `checked_point` (`right_click.rs:1751`, `double_click.rs:475`) | `hotkey.rs:897`, `click.rs:6132` `perform_outcome_keeps_one_meaning_per_error`, 98632052's 8 and 0f0d4b70's 4 red tests |
| #281 | FIXED (#281.2, #281.3); #281.1 documented in GOAT.md | `click.rs:4231` `SelectionUnavailable` | `click.rs:5586` `a_selection_target_that_is_not_settable_says_so` |
| #283 | FIXED (#283.2 dead `hand_back` removed, 393c49e2); #283.1 documented in GOAT.md; live smoke NOT RUN | — | — |

Review state: wp3/wp4 review of 393c49e2 FAIL (2 P2) → 98632052; wp3c review of 98632052 NEAR-PASS (2 P2, 1 P3) → 0f0d4b70; #287
round 1 NEAR-PASS → e354f982 (round 2 stalled at the GOAT restart; its scope is in `cu-final-review-12e47ec6-s24`). The delta review of
0f0d4b70 and 019c3858 (`cu-delta-review-019c3858-s24`) is pending; wp3 closes after it returns.

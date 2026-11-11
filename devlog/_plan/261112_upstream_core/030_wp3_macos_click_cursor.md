# 030 — wp3: macOS click ordering, cursor rest parking, gate follow-ups (#286, #287, #279, #281, #283)

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

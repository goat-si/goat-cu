# 040 — wp4: agent-desktop core gaps (AD-G1 → #272, AD-G8)

> Re-validated on the merged tree in 012: §A's guard is incomplete — at `Scope::WindowInternal` (semantic.rs:150) `offscreen` is Unknown when clipping is unreadable and evaluate.rs:139-144 still waits out the budget; decide unknown-offscreen for strictly semantic delivery (known hidden/offscreen keeps refusing).

Stale check at wp4 P: re-read `cua-driver-core/src/actionability/evaluate.rs` and the macOS adapter after wp1–wp3.

## §A — AD-G1 / #272 (+#258 overlap): headless semantic delivery does not need pointer visibility

Agent-desktop rule (`crates/core/src/actionability/gates.rs:11-66`, `requirements.rs:59-125`): for strictly headless semantic
delivery (an advertised Press / ShowMenu, or the action's own capability), explicit hidden/offscreen state still fails, but
missing pointer visibility or geometry does not wait or refuse.

- MODIFY `rust/crates/cua-driver-core/src/actionability/evaluate.rs` visible gate (40c6f6eb :130-131):

```rust
if requirements.visible {
    let semantic_delivery = without_pointer && (semantic_pointer || semantic_nonpointer);
    match (&facts.visible, semantic_delivery) {
        // Pointer-free semantic delivery: only a proven-hidden target fails; an unreadable
        // pointer visibility (zero-size ancestor, deep tree, window mismatch) does not block it.
        (Evidence::Unknown(reason), true) if !matches!(facts.offscreen, Evidence::Known(true)) => push(
            &mut checks, "visible", CheckStatus::Pass,
            format!("semantic delivery; pointer visibility not required ({reason})"),
        ),
        _ => gate(&mut checks, "visible", &facts.visible, false),
    }
    ...
```

  (Known(false) still transient-refuses; `offscreen` Known(true)/Unknown handling below stays.)
- Test (failing first): `cua-driver-core/src/actionability/tests.rs` —
  `headless_press_with_unknown_visibility_is_actionable`: facts with `affordances: Known([Press])`, `visible: Unknown`,
  `offscreen: Known(false)`, bounds known, enabled true, stable true → `Intent::Click, Policy::headless()` verdict actionable;
  companion `headless_press_with_hidden_target_still_waits` (`visible: Known(false)`) stays non-actionable;
  `background_window` pointer click with `visible: Unknown` stays non-actionable.
- macOS: no adapter change needed if the scope reader already reports `offscreen`; if it reports `offscreen: Unknown` for
  the #272 cases, the test matrix gains that row and the rule is decided at wp4 P with the reviewer.

## §B — AD-G8: settle-budget tests on the virtual clock

Read `cua-driver-core/src/post_action_wait.rs` tests. If any budget test sleeps on the wall clock, convert it to
`#[tokio::test(start_paused = true)]` (dev-dependency `tokio` `test-util` already present). Otherwise record COVERED.

## §C — Re-confirm COVERED / NOT APPLICABLE rows of 003 on the synced tree with path:line.

## Verification

`cargo test -p cua-driver-core --lib actionability post_action_wait`, `cargo test -p platform-macos --lib actionability`.

## wp4 P decision and B/C evidence (lead), on HEAD 393c49e2

**§A #272 / AD-G1 — decided at P: fix in the macOS adapter, not in core.** Core cannot tell clipping-derived evidence from
explicit state: `NativeScope::visible` (scope.rs) is window hidden/minimized *then* ancestor clipping, and `offscreen` at
`Scope::WindowInternal` is clipping only, so a zero-size or scrolled-out ancestor gives *known* `visible=false`/`offscreen=true`
(issue cases 1 and 5) and the 012 guard on Unknown alone would not fix them. Agent-desktop's `gates.rs::visibility` (54855b62) fails only
on live hidden/offscreen state and the target's own zero-size bounds, never on ancestor clipping, for delivery without a pointer.
Change: `ScopeReader::window_visible` (actionability.rs:54; native scope.rs:331, which `visible` now reuses) reads only the window's
explicit visibility; `Adapter::facts_with_hits` (actionability.rs:156) keeps clipping `visible`/`offscreen` for the pointer battery and,
for `semantic_facts`, uses `semantic_visibility` (:194): window hidden/minimized → false, window state unreadable → unknown, own frame
empty → false, own frame absent/unreadable/invalid → unknown, else true; `offscreen` is `Absent` (macOS has no live offscreen state).
Kept: hidden app / minimized window / unreadable window state / empty or unreadable own frame still wait and refuse; stability, enabled,
affordance and token/exact-target checks unchanged; pointer clicks unchanged (clipping + hit). The clipping walk no longer proves that the
target sits in the admitted window; the exact binding does: every semantic route goes through `bind_exact_with`
(tools/actionability/semantic.rs:71-89), which requires the target's PID, its `AXWindow`, and that window's id to equal the admitted
`window_id`.
Correction (wp3/wp4 review, finding 6): an earlier version of this paragraph said semantic delivery relies on the token's snapshot binding
and that this covers issue case 3 (sheets, popovers, menus whose window differs). It does not: such an element reads a different
`AXWindow` (or none), so `bind_exact_with` returns unknown and the call still waits out its budget and refuses. #272 is fixed for the
clipping cases (1 and 5); case 3 stays refused and is a follow-up (admitting a same-pid child window of the admitted window needs its own
safety argument). Commit 68771fdb's message repeats the earlier claim; this record supersedes it.
Issue's live smoke (Safari/Chrome page button, Electron, sheet/popover, scrolled-out row, AppKit button): NOT RUN — those are not
GOAT-owned fixtures.
Test adjustment: `failed_action_names_block_custom_action` (actionability.rs:666) gives its semantic loop a target frame; it passed only
because FakeScope claimed clipping visibility for a frameless target, which the native clipping read never did (scope.rs `clipping`
returns unknown "target frame unavailable").

**§B AD-G8 — NOT APPLICABLE as a test-only port.** `post_action_wait.rs` has no time-based tests (:113-149, argument parsing and task
scope). GOAT's post-action settle is the window-change detector's own std::thread with wall-clock deadlines
(window_change_detector.rs:760, :796); its tests poll with 2 s ceilings, one asserts an upper bound `elapsed < 100ms` (:1360). A virtual
clock needs clock injection into that production thread, beyond agent-desktop #642's test-only scope. Residual: that upper bound can
flake on a saturated host.

**§C re-confirmed on the synced tree.** AD-G4 COVERED: get_window_state.rs:1720-1721 sets `observation_complete:false` and
`observation_incomplete_reasons` (tests :3516, :3547). AD-G5 COVERED by wp1: session.rs:864-883 releases labels when the owner
transport closes (#7083), session_tools.rs:526 ends after the in-flight action. AD-G6 / AD-G7 NOT APPLICABLE: no app-terminated wait
and no on-disk ref store in rust/crates (`rg wait_event|app_terminated|refs_store` empty). AD-G9 → #287 (wp3 §B), AD-G10 → #286 (fixed,
393c49e2).

| Command (rust/) | Exit | Result |
|---|---|---|
| `cargo test -p platform-macos --lib -- semantic_press_ignores_ancestor_clipping` (semantic facts still from clipping) | 101 | actionability.rs:1004 "zero-size or scrolled-out ancestor": `Transient { reason: "visible is false or absent" }` vs `Pass` |
| `cargo test -p platform-macos --lib -- actionability tools::click::` after the change, before the test adjustment | 101 | 1 failed: `failed_action_names_block_custom_action` "visible: target frame unavailable" (frameless semantic target) |
| same, after the adjustment | 0 | 164 passed, 1 ignored |
| `cargo check -p platform-macos --all-targets` | 0 | no warnings |
| rustfmt --check actionability.rs / scope.rs / click.rs vs HEAD | — | identical remaining diffs |

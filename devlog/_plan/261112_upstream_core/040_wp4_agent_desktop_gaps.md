# 040 — wp4: agent-desktop core gaps (AD-G1 → #272, AD-G8)

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

# 018 — wp1h: upstream delta 56a69039 → 7dc3cc43 (multi-display cursor, disabled controls, history deprecation)

Found at the re-fetch before the last D (2026-11-12 ~16:20 UTC): trycua/cua main `7dc3cc43`, 7 commits after `56a69039` (wp1g, 017)
that touch `libs/cua-driver`, 54 files; goat-cu origin/main `40c6f6eb` and agent-desktop `54855b62` unchanged. Patches and per-file
base/upstream copies: `/tmp/goat-cu-upstream/wp1h/`, `/tmp/goat-cu-upstream/56a69039…`, `/tmp/goat-cu-upstream/7dc3cc43…`.
Sync commit c2067c65, follow-up 9b9b29af.

| Commit | Scope | Disposition |
|---|---|---|
| 8fb8d67e (#7458) | macOS cursor rendered on every display (per-display surfaces, new `cursor/display_layout.rs`); every action mark retired (`cursor.marks` accepted and ignored, macOS focus box and GNOME helper action cues removed) | IN, combined with GOAT #287 (below) |
| 84197407 (#7469) | Linux and Windows index disabled controls with `enabled: false`; click, right_click, double_click, set_value, type_text refuse a disabled token as `element_disabled`; GTK3 harness case | IN, combined (below) |
| 0b6922ba (#7478) | Computer History deprecated: no agent-facing pointer to `history_status`/`history_query`, tools marked Deprecated, `history enable` removal notice | IN; GOAT's own start-guide line also loses its history sentence |
| 8dba7a5d (#7480) | `permissions status` prints when direct capture was last verified | IN (clean) |
| 7c553ee8 (#7462) | example `requirements.txt` pin after PyPI pruning | IN (clean) |
| 1cf2bc95 (#7460) | published installer version 0.47.19 | IN (GOAT's 0.47.18 equalled upstream's base); GOAT's `install.ps1` mcp-config fallback kept |
| 7dc3cc43 (#7484) | cua-sdk 0.26.0 | IN: lock `cua-telemetry` 0.26.0, vendored crate version and fork comment, `vendor/README.md` source commit 7dc3cc43; vendored sources, tests, `targets.json`, `sandbox-images.json` byte-identical to 7dc3cc43. GOAT's lock has no `cua-agent-setup`/`cua-home`, so their bumps do not apply |

No cua-driver-rs release in the range, so `rust/CHANGELOG.md` is unchanged; GOAT versions kept (`rust/VERSION` 0.38.0-goat.1,
workspace 0.40.1-goat.1, SKILL.md 0.38.0, no 0.47 line in the lock).

Method as 015-017: `git merge-file` per file (`/tmp/goat-cu-sync/wp1h-trial.sh`): 42 clean, 11 with conflicts, resolved with
`/tmp/goat-cu-sync/resolve.py` (per-conflict goat / upstream / combined text).

### Combinations (GOAT patch kept with the upstream change)

- `platform-macos/src/cursor/overlay.rs` (9 conflicts): rebuilt from upstream's file with GOAT's 15 hunks against the base re-applied:
  the glide latency span; `needs_frame_tick` without the idle-hide countdown term; `ParkedWait` / `parked_wait` / `parked_repin_due`;
  the parked wait, parked time charged to idle clocks before commands, the next idle-fade deadline, and the parked repin. Upstream's idle
  loop now wakes every 250 ms to re-read the display layout, so a new `loop_wait` caps every parked receive at
  `DISPLAY_CHECK_INTERVAL` (GOAT's `parked_wait` stays the pure cursor decision). The repin reasserts every surface, and also runs on a
  surface rebuild while parked (upstream only checked `refit` inside the active-loop gate). The focus rect is gone with upstream.
- `display_layout.rs` (new, upstream): GOAT's `get_backing_scale` returns `Option` and GOAT never paints at a guessed scale, so
  `known_scale` refuses a reading with an unreadable display scale (`Err(UNKNOWN_SCALE)`) instead of upstream's 1.0: at start no
  overlay, later the loop keeps its surfaces (upstream's own `Err` handling at both call sites). GOAT's main-screen
  `initial_overlay_scale` / `geometry_from_frame` went with upstream's `main_display_geometry`; their `get_screen_size` test is
  replaced by `an_unreadable_display_scale_refuses_the_reading`.
- `docs/cursor-themes.md`: the `cursor.marks` row is dropped, GOAT's parking wording for `cursor.float` kept. macOS
  `set_config.rs`: GOAT's shorter description with "cursor.marks is retired and ignored". macOS `click.rs`: the merge misaligned
  GOAT's test region against upstream's removed `ShowFocusRect` send; GOAT's side kept (the send itself merged cleanly).
- `cua-driver-core/src/protocol.rs`: GOAT's short instructions kept; GOAT had moved the history advice into the `get_guide` start topic
  (`rust/Skills/cua-driver/SKILL.md` line 53), which upstream's deletion did not reach, so that sentence and its dangling link are
  removed and `agents_are_not_sent_to_deprecated_history` replaces the old history-order test.
- Linux `tools/{click,set_value,type_text}.rs`: GOAT's imports plus `refuse_if_disabled`; `type_text` refuses a disabled token before
  GOAT's password lookup. `atspi/native.rs`: GOAT's secure-value redaction with upstream's `indexed_row_attrs` (`enabled=false`).
  `atspi/snapshot.rs`: upstream's `snapshot_hit_test_never_picks_a_disabled_control` ported to GOAT's `build_snapshot` /
  `hit_test(elements, ..)` API (upstream's `update_snapshot` / `forget_window` do not exist here).
- Windows `uia/mod.rs` (3 conflicts): GOAT's ctx-based reader kept; `collect_pattern_evidence` no longer skips a disabled control's
  actions (value probe and read-failure accounting unchanged); `is_actionable` no longer requires enablement; the tree row keeps GOAT's
  `toggle=` / `expanded=` with upstream's `enabled=false`. 9b9b29af: upstream's Windows-only row test sets GOAT's `checked` /
  `expanded` fields.
- `rust/Cargo.lock`: GOAT's side for the two conflicts (packages GOAT's lock does not carry).
- `lease_wiring_tests/source_exemptions.rs`: `run_appkit`'s render-thread spawn re-pinned (7844733029246040032 →
  7198220637949120086); still the overlay renderer, no application input.

### Receipts

| Command | Exit | Result |
|---|---|---|
| `cargo test -p cua-driver-core --lib agents_are_not_sent_to_deprecated_history` before the SKILL.md edit | 101 | panicked at protocol.rs:635 (start guide names `history_status`) |
| same after | 0 | agent_instruction_tests 5 passed; guide tests 15 passed |
| `cargo test -p platform-windows --lib disabled_node_value_pattern_failure…` with the old helper | 101 | panicked at walk_completeness_tests.rs:455 (no actions for a disabled node) |
| `cargo test -p platform-windows --lib` after | 0 | 207 passed |
| `cargo test -p platform-macos --lib` | 0 | 2038 passed / 48 ignored (first run 2 failed: the lease-wiring inventory before the re-pin) |
| `cargo test -p cursor-overlay --lib` / `cua-driver-core --lib` | 0 / 0 | 158 / 1823 passed, 3 ignored |
| `TMPDIR=/tmp cargo test -p cua-driver --bin cua-driver` | 0 | 424 passed; with the default 85-character TMPDIR 5 socket tests fail on SUN_LEN (environment) |
| `cargo run -p cua-driver-contract --bin cua-contract-gen -- all --check`; `node scripts/generate-uniffi-bindings.mjs --check` | 0 / 0 | manifest and UniFFI bindings up to date |
| `node scripts/tests/test_wayland_helper_cursor_paint.mjs`, `…_safety.mjs` | 0 / 0 | |
| Linux container: `platform-linux --lib`, `--features portal-input`, clippy `-D warnings` (platform-linux, core, cursor-overlay; and portal-input) | 0 | 1173 / 15, 1232 / 15, clean (first run: E0425/E0061 in upstream's snapshot test, ported) |
| Linux container: `cursor-overlay --lib`, `cua-driver-core --lib` | 0 | 158, 1826 / 3 |
| `/tmp/goat-cu-sync/receipt-final.sh` at c2067c65 | 2 | 22/24: `windows-target` and `windows-bins` failed (E0063 in upstream's Windows-only row test) |
| the same two steps at 9b9b29af | 0 / 0 | only the recorded pre-existing warnings (uia/mod.rs:1205, input/inject.rs:314/318/363) |

NOT RUN: live multi-display macOS overlay, live GNOME Shell helper, the GTK3 e2e harness (incl. upstream's disabled-control case),
Windows runtime, PowerShell installer tests.

Reviews: A (macOS cursor) PASS; B (the rest) NEAR-PASS. Follow-ups from them: 08c5ca6e (Windows `element_is_enabled` joins COM
before reading, so the disabled refusal cannot fail open), ed282703 (the AX click's retired `ShowFocusRect` send, which the merge
had left in place). Whole-head receipt at ed282703: 24 steps, failures=0 (050).

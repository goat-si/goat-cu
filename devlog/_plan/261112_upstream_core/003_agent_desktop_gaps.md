# 003 — Agent-desktop 54855b62 core gap list

Scope of comparison (lane brief): process identity / pid reuse, session-scoped stores, cursor idle, click ordering,
read outcome / completeness / deadline, post-action settle. Agent-desktop paths are relative to
`/tmp/goat-cu-agent-desktop/54855b6225aec00da4ac65e2d0362fd41cd9fcb3/`; goat-cu paths are at `40c6f6eb`.
Agent-desktop is a CLI with a per-session on-disk ref store; goat-cu is an MCP daemon with an in-memory, session-filtered
`SnapshotStore`. File-store mechanics (ref-store locks, prune-by-copy) have no goat-cu counterpart.

| Id | Agent-desktop | goat-cu | Status | Unit |
|---|---|---|---|---|
| AD-G1 | Headless semantic delivery needs no pointer geometry/visibility proof: `crates/core/src/actionability/requirements.rs:59-125`, `gates.rs:11-66`, `evaluate.rs:140-190` (#376) | `cua-driver-core/src/actionability/evaluate.rs:113-170` gates `visible` for every visible-requiring intent; semantic AXPress with unreadable ancestry waits and refuses (#272, #258) | GAP | wp4 040 §A |
| AD-G2 | Refs are re-identified through the element's own address and process (#441, #442) | Windows binds snapshots to the process instance (U15 #260, `platform-windows/src/process_instance.rs`); macOS publishes and resolves without it (`platform-macos/src/process_identity.rs:24-28` dead code "U33B will connect"; publish at `tools/mod.rs:2337`, `get_window_state.rs:956`; ten `resolve(pid, &args)` sites) | GAP (pid reuse) | wp2 020 §B |
| AD-G3 | App inventory carries process identity from libproc (#573 `crates/macos/src/system/process/process_apps.rs`) | `apps/mod.rs:79-100` already lists pids via `proc_listallpids`; no instance in answers (#299) | GAP (output) | wp2 020 §A |
| AD-G4 | Partial snapshot still persisted after the observation deadline (#372 `crates/core/src/snapshot/mod.rs:29-35`) | macOS get_window_state publishes truncated/time-budget reads with `observation_complete:false` and `observation_incomplete_reasons` (`get_window_state.rs:1652`, tests :3499-3520) | COVERED | — |
| AD-G5 | Session scope acquired before trace state; session manifest RMW under one lock (#602 `context/mod.rs`, `session/liveness.rs`) | in-memory session registry; upstream #7037 (end defers behind admitted call) and #7083 (labels released on transport close) arrive with wp1 | COVERED by wp1 | wp1 |
| AD-G6 | wait_event returns the first match that is confirmed, not the first match (#602 `commands/wait_event.rs:109-121`) | goat-cu has no app-terminated event wait; `verify_state` predicates already require accepted observations within the deadline (`expectation.rs`) | NOT APPLICABLE | — |
| AD-G7 | Ref prune deletes only byte-identical trace copies (#602 `refs/refs_store_prune.rs`) | no on-disk ref store | NOT APPLICABLE | — |
| AD-G8 | Post-action settle budget tests on the virtual clock (#642, test-only) | `cua-driver-core/src/post_action_wait.rs` tests: re-check in wp4 P that they use `tokio::time::pause`/`start_paused` and no wall sleeps | VERIFY | wp4 040 §B |
| AD-G9 | Cursor overlay renders outside caller dir, above menus (#381, #511) | GOAT overlay is in-process; z-order handled by `cursor-overlay/src/z_order.rs`; idle parking regression is #287 | #287 only | wp3 030 §B |
| AD-G10 | Click ordering: activation/focus, verified pointer target, then dispatch (`CONCEPTS.md:234`) | macOS pixel click: frame-bounds check → exact-target lease → fast-path relocation (unchecked, #286) → Chromium select check → glide → dispatch | GAP (#286) | wp3 030 §A |

COVERED / NOT APPLICABLE rows are re-confirmed with fresh path:line on the synced tree in wp4 D.

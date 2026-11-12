# 001 — Upstream delta inventory (566a4ce6 → c0d3b919)

Research document. No implementation diffs here; wp1's design is 010.

## How it was computed (reproducible)

```sh
cd /tmp/goat-cu-upstream
git diff --no-index --name-status 566a4ce6…/libs/cua-driver c0d3b919…/libs/cua-driver   # 298 entries, no renames/deletes
# per file: compare the worktree file with the old and new upstream copies (cmp -s)
# per modified-both file: git merge-file -p -L ours -L base -L theirs <goat> <old> <new>  (exit = conflict hunks)
```

Outputs kept outside the repo: `/tmp/goat-cu-research/delta-class.tsv`, `trial-conflicts.tsv`, `trial-merge-1/`,
`preservation-mech.tsv`, `upstream-driver-commits.tsv` (182 upstream commits touching `libs/cua-driver` after 2026-11-06T15:17:08Z).

## Classes

| Class | Files | Meaning |
|---|---|---|
| added-new | 46 | new upstream files, absent in goat-cu |
| modified-clean | 126 | goat-cu file equals old upstream; take upstream as is |
| modified-both, merges clean | 55 | 3-way merge resolves without markers; still reviewed for GOAT invariants (session.rs, tool.rs, authorization.rs, window_target.rs, health_report.rs among them) |
| modified-both, conflicts | 71 | 301 conflict hunks in the trial merge |
| deleted | 0 | — |

Upstream did not change `contract/` or `compat-fixtures/` in the range. Upstream docs changed: `docs/action-icon-catalog.md`,
`docs/action-result-contract.md`, `docs/action-support.md`, `docs/test-harnesses-guide.md`; added `docs/kwin-window-capture.md`.

## Conflicted files (trial merge) with GOAT and upstream attribution

Columns: hunks; GOAT PRs since #290 (`87e2e869..40c6f6eb`); last GOAT PRs touching the file; upstream PRs touching it after 566a4ce6.

| Path | Hunks | GOAT since #290 | GOAT history (newest first) | Upstream PRs |
|---|---|---|---|---|
| `rust/crates/platform-macos/src/ax/tree.rs` | 21 | #297,#291 | #297,#291,#290,#273,#270,#265,#261,#251,#248,#216,#214 | #7192,#6893,#6878,#6758 |
| `rust/crates/platform-macos/src/tools/click.rs` | 20 | #298,#292 | #298,#292,#290,#282,#280,#278,#277,#269,#266,#216 | #7338,#7074,#7041,#6928,#6909,#6889,#6803,#6758,#6702 |
| `rust/crates/platform-macos/src/tools/type_text.rs` | 18 | — | #290,#282,#280,#267,#217 | #7338,#7041,#6884,#6702 |
| `rust/crates/platform-linux/src/tools/type_text.rs` | 16 | — | #290 | #7164,#6982,#6827,#6892,#6768 |
| `rust/crates/cua-driver-core/src/recording.rs` | 15 | — | #290 | #6839,#6807,#6794 |
| `rust/Cargo.lock` | 15 | #293 | #293,#290,#284,#263,#221,#222 | #7381,#7377,#7364,#7363,#7359,#7344,#7334,#7332,#7331,#73... |
| `rust/crates/platform-macos/src/capture.rs` | 13 | — | #290 | #6882,#6881,#6809 |
| `rust/crates/platform-macos/src/input/mouse.rs` | 10 | — | #290 | #7372,#7338,#6889 |
| `rust/crates/platform-macos/src/input/keyboard.rs` | 10 | — | #290 | #7338,#6939,#6884 |
| `rust/crates/platform-linux/src/wayland/libei.rs` | 10 | — | #290 | #6940,#6827 |
| `rust/crates/platform-macos/src/window_change_detector.rs` | 9 | #292 | #292,#290,#263 | #6928,#6803 |
| `rust/crates/platform-macos/src/ax/bindings.rs` | 9 | — | #290,#265,#244 | #7192,#6878,#6758 |
| `rust/crates/platform-macos/src/ax/exact_target.rs` | 8 | — | #290 | #6882,#6758 |
| `rust/crates/platform-macos/src/tools/hotkey.rs` | 7 | — | #290,#282,#215,#217 | #7338 |
| `rust/crates/platform-windows/src/input/keyboard.rs` | 6 | — | #290,#255,#213 | #6892,#6769,#6779 |
| `rust/crates/platform-macos/src/windows.rs` | 6 | — | #290 | #6882 |
| `rust/crates/platform-macos/src/tools/press_key.rs` | 6 | #292 | #292,#290,#282,#263,#215 | #7338,#6928 |
| `rust/crates/platform-macos/src/tools/invoke_menu.rs` | 6 | — | #290,#263 | #7338,#7039,#6953,#6758 |
| `rust/crates/platform-macos/src/input/skylight.rs` | 6 | — | #290,#263,#217 | #6937,#6882,#6803,#6758 |
| `rust/crates/platform-linux/src/tools/set_value.rs` | 6 | — | #290 | #7345,#7254,#6982,#6892,#6768 |
| `rust/crates/platform-windows/src/uia/mod.rs` | 5 | — | #290,#247,#240,#224 | #6769,#6812 |
| `rust/crates/platform-macos/src/tools/drag.rs` | 5 | — | #290 | #7372,#7338,#6803 |
| `rust/crates/platform-linux/src/wayland/mod.rs` | 5 | — | #290 | #6982,#6827,#6890,#6820,#6821 |
| `rust/crates/platform-linux/src/atspi/native.rs` | 5 | — | #290,#227 | #7345,#7265,#7157,#6887,#6848,#6851,#6829 |
| `rust/crates/platform-macos/src/tools/launch_app.rs` | 3 | #295 | #295,#290,#263 | #6803 |
| `rust/crates/platform-macos/src/tools/double_click.rs` | 3 | — | #290,#269,#266 | #7041,#6702 |
| `rust/crates/platform-macos/src/ax/window_scope.rs` | 3 | — | #290 | #7192,#6758 |
| `typescript/package-lock.json` | 2 | — | — | #7381,#7377,#7364,#7363,#7344,#7334,#7331,#7312,#7296,#72... |
| `rust/crates/platform-windows/src/uia/windows_enum.rs` | 2 | — | #290,#255,#247 | #6769 |
| `rust/crates/platform-windows/src/input/inject.rs` | 2 | — | #290 | #6769 |
| `rust/crates/platform-macos/src/tools/scroll.rs` | 2 | #297 | #297,#290 | #7338 |
| `rust/crates/platform-macos/src/tools/right_click.rs` | 2 | — | #290,#269,#263,#266 | #7338 |
| `rust/crates/platform-macos/src/tools/mod.rs` | 2 | — | #290,#285,#282,#263,#254,#216,#214,#217 | #7338,#6794,#6758,#6702 |
| `rust/crates/platform-macos/src/tools/get_window_state.rs` | 2 | #291 | #291,#290,#273,#263,#265,#251,#233,#216 | #7192,#6893,#6879,#6882 |
| `rust/crates/platform-linux/src/atspi/mod.rs` | 2 | — | #290,#227 | #7345,#7157,#7080,#6822 |
| `rust/crates/cua-driver-core/src/lib.rs` | 2 | — | #290,#218 | #7338,#6892,#6889 |
| `rust/Skills/cua-driver/SKILL.md` | 2 | — | #290 | #7381,#7377,#7364,#7363,#7344,#7334,#7331,#7312,#7296,#72... |
| `rust/Cargo.toml` | 2 | #293 | #293,#284,#222,#211 | #7381,#7377,#7364,#7363,#7344,#7316,#7334,#7331,#7312,#72... |
| `typescript/package.json` | 1 | — | — | #7381,#7377,#7364,#7363,#7344,#7334,#7331,#7312,#7296,#72... |
| `rust/crates/platform-windows/src/uia/scroll.rs` | 1 | — | #290,#247 | #6769 |
| `rust/crates/platform-windows/src/tools/type_text.rs` | 1 | — | #290 | #6892,#6769,#6779 |
| `rust/crates/platform-windows/src/tools/press_key.rs` | 1 | — | #290 | #6769,#6779 |
| `rust/crates/platform-windows/src/tools/hotkey.rs` | 1 | — | #290 | #6779 |
| `rust/crates/platform-windows/src/tools/click.rs` | 1 | — | #290 | #6769 |
| `rust/crates/platform-windows/src/lib.rs` | 1 | — | #290,#260,#255,#240,#224,#213 | #6779 |
| `rust/crates/platform-macos/src/tools/bring_to_front.rs` | 1 | — | #290 | #7260,#6980,#6803,#6758 |
| `rust/crates/platform-macos/src/key_focus_guard.rs` | 1 | — | #290,#263 | #6803 |
| `rust/crates/platform-macos/src/browser/platform.rs` | 1 | — | #290,#263,#217 | #7048,#6803 |
| `rust/crates/platform-macos/src/ax/mod.rs` | 1 | — | #290,#261,#254,#251,#244,#216 | #6878 |
| `rust/crates/platform-macos/src/apps/mod.rs` | 1 | — | #290,#233,#217 | #6879,#6882,#6883,#6803 |
| `rust/crates/platform-linux/src/wayland/region_route.rs` | 1 | — | #290 | #6821 |
| `rust/crates/platform-linux/src/wayland/hyprland.rs` | 1 | — | #290 | #7305,#6982,#6821,#6822,#6695 |
| `rust/crates/platform-linux/src/tools/type_text_chars.rs` | 1 | — | #290 | #6982,#6892 |
| `rust/crates/platform-linux/src/tools/launch_app.rs` | 1 | — | #290 | #6695 |
| `rust/crates/platform-linux/src/tools/get_screen_size.rs` | 1 | — | #290 | #6821 |
| `rust/crates/platform-linux/src/tools/click.rs` | 1 | — | #290 | #7265,#7164,#7157,#6982,#6820,#6768 |
| `rust/crates/cua-driver/Cargo.toml` | 1 | — | — | #7190,#7165 |
| `rust/crates/cua-driver-sdk/src/runtime.rs` | 1 | — | #290,#285 | #6802,#6794 |
| `rust/crates/cua-driver-sdk/src/embedded.rs` | 1 | — | #290 | #6889,#6769 |
| `rust/crates/cua-driver-core/src/tool.rs` | 1 | — | #290,#285,#245,#241,#226,#221 | #7338,#7083,#6921,#6885,#6738 |
| `rust/crates/cua-driver-core/src/protocol.rs` | 1 | — | #290 | #6921 |
| `rust/crates/cua-driver-core/src/browser/v2_tests.rs` | 1 | — | #290,#285 | #6808 |
| `rust/crates/cua-driver-core/src/browser/tools.rs` | 1 | — | #290,#285 | #6769,#6738 |
| `rust/crates/cua-driver-core/src/browser/prepare.rs` | 1 | — | #290 | #6808,#6738 |
| `rust/crates/cua-driver-core/src/batch_tools.rs` | 1 | — | #290,#252,#241,#238,#221 | #6921 |
| `rust/VERSION` | 1 | — | — | #7381,#7377,#7364,#7363,#7344,#7334,#7331,#7312,#7296,#72... |
| `rust/Skills/cua-driver/WORKFLOW.md` | 1 | — | #290 | #6921 |
| `rust/Skills/cua-driver/MACOS.md` | 1 | — | #290,#252 | #6937 |
| `python/src/cua_driver/__init__.py` | 1 | — | #290,#226 | #7381,#7377,#7364,#7363,#7344,#7334,#7331,#7312,#7296,#72... |
| `python/pyproject.toml` | 1 | — | — | #7381,#7377,#7364,#7363,#7344,#7334,#7331,#7312,#7296,#72... |
| `README.md` | 1 | — | #290,#212 | #7147 |

The final per-file resolution (KEEP-GOAT / TAKE-UPSTREAM / COMBINE, with the invariant kept) is recorded in 010 §Preservation
table during wp1 B and copied into the wp1 commit message.

## Upstream behavior/security changes GOAT must adopt (core, contract, macOS)

| Upstream | Change | GOAT interaction |
|---|---|---|
| #7338 `e13f4321` | refuse input to macOS security and consent prompts (`security_prompt.rs` core + macOS) | new refusal on every input route; must sit before GOAT's lease/actionability gates send anything |
| #7039 `02e4aa0b` | invoke_menu worker keeps its pid lease and the desktop lane until it returns | GOAT holds a cross-process desktop lease and worker holds (#263); combine, release in reverse order |
| #7037 `bd7a99d7` | session end defers behind a call admitted after it decided | session.rs (clean merge); GOAT session-scoped snapshot filtering must survive |
| #7083 `a42bebad` | release session labels when their owner transport closes | session.rs; keeps GOAT's runtime_session_key scoping |
| #6803 `14ae7176` | one macOS focus arbiter decides every activation undo (`focus_arbiter.rs`) | GOAT focus_steal/key-window hand-back ordering (#158) must route through the arbiter without reordering hand-back after keys |
| #7041/#6947/#6702 | wait for / re-read AXEnabled after foreground activation | GOAT foreground-aware disabled refusal (#290 decision) |
| #7074 `4c5793fa` | element-token middle clicks routed to the target window | click.rs pointer route; GOAT exact-target window lease |
| #7372 `a00a44bb` | stamp window-local point on macOS 27 background drags | drag.rs; GOAT fallback when OS version unreadable (#290 decision) |
| #7260 `7ab7fe8b` | bring_to_front tells live-but-unreported from exited apps | bring_to_front.rs |
| #7192 `0365501c` | name the AXWindows read error in ax_window_unresolved | ax/bindings.rs, tree.rs |
| #6878 `6054a728` | read each AX node in one batched round trip (`ax/batch.rs`) | tree.rs walk; GOAT walk evidence/completeness (#261, #265, #270, #273) must keep failed reads as evidence |
| #6827/#6939 | exact text on the active keyboard layout or refuse; TIS thread affinity | type_text.rs, keyboard.rs, `input/input_source.rs`; GOAT key_hold routes |
| #6884/#6889 | batched type_text key events; web focus wait; mouse pacing knobs (`input_pacing.rs`, `type_pacing.rs`) | GOAT write-outcome classification of type_text (#267) |
| #6921 `7f81b8b2` | batch nudge after the second action | `batch_nudge.rs` (modified-clean) |
| #6928/#6893 | opened-window identity per action; search field identity | GOAT already ported as #292/#291; expect equivalent code, take upstream text where identical |
| #6743 | refuse MCP requests before initialize on HTTP and envelope transports | `mcp_http.rs`, `mcp_envelope.rs` |

Linux/Windows-only changes (KWin capture/layout, libei, MPX pool, X11 focus give-back, Windows keyboard/typing turn) merge under the
same policy but cannot be compiled on this macOS host; see 010 §Verification for what is NOT RUN.

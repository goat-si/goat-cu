# 019e — wp1m: upstream delta 228b5a4d → 01596ffc (every native input tool publishes the action result)

Found at the re-fetch after 35755746 (2026-11-12 ~19:55 UTC): trycua/cua main `01596ffc`, 1 commit after `228b5a4d` (wp1l, 019d), touching
`docs/action-result-contract.md` and `rust/crates/cua-driver/tests/protocol_schema_test.rs`; goat-cu origin/main `40c6f6eb` and agent-desktop
`54855b62` unchanged. Merge `/tmp/goat-cu-sync/wp1m-merge.sh` (base 228b5a4d), diff `/tmp/goat-cu-upstream/wp1i/01596ffc.diff`.

| Commit | Scope | Disposition |
|---|---|---|
| 01596ffc (#7507, fixes trycua/cua#2885) | `every_native_input_tool_publishes_the_action_result`: every tool in the live `tools/list` that advertises an `input.pointer.*`, `input.keyboard.*`, `menu.*` or `window.frame.set` capability must be an action-result tool advertising the `ActionResult` schema; the contract doc records the modality-ladder audit outcome | IN (clean: the doc equalled the base; the test file merged with GOAT's edits, including 7cab649b's page-refusal assertions) |

The test reads GOAT's own `tools/list`. Capabilities are assigned by tool name (`cua_driver_contract::tool_capabilities`, then
`default_capabilities_for`, `cua-driver-core/src/tool.rs:956`), so the check is the same on every platform: each named tool with a native-input
capability (click, double_click, right_click, drag, mouse_drag, parallel_mouse_drag, mouse_button_down/up, move_cursor, scroll, type_text,
type_text_chars, set_value, press_key, hotkey, invoke_menu, set_window_frame) is in `ACTION_RESULT_TOOLS`; GOAT's `select_text` and
`menu_bar_extras` advertise none of those capabilities.

### Receipts

| Command | Exit | Result |
|---|---|---|
| `cargo test -p cua-driver --test protocol_schema_test` (macOS) | 0 | 3 passed / 1 ignored, including the new test |
| Linux container (rust:1.97.1): `cargo test --locked -p cua-driver --test protocol_schema_test` | 0 | 5 passed, including the new test and `removed_legacy_page_actions_are_refused_even_with_the_old_opt_in` (7cab649b) |

NOT RUN: the Windows lane (`protocol_schema_test` runs there in goat-ci; no local Windows host).

Review `cu-review-wp1m-99de9504-s24` (steered past its bound): NEAR-PASS. It confirmed both files are the parent plus exactly upstream's
hunk (`patch -F0` reproduces the test file; the doc hashes match), counted the checked tools per roster (macOS 12, Linux 16, Windows 12,
all in `ACTION_RESULT_TOOLS`, so the `checked >= 6` floor holds) and found the audit table true for GOAT's tree. M1: the invariant is
met by GOAT's `select_text` and `menu_bar_extras` only because they advertise no native-input capability; both are desktop input
(`authorization.rs:292-293`) and still return legacy `effect`/`verified` payloads → named in `docs/action-result-contract.md` (a GOAT
patch to keep on the next sync) and recorded as a 050 follow-up. L1 (the line citation) and L2 (no Windows receipt) fixed here.

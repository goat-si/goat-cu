# 013 — wp1c: Sync libs/cua-driver e51f5740 → 465aab0b (found at the pre-D re-fetch)

Re-fetch on 2026-11-12 (before the last D): `git fetch origin --prune` → origin/main still `40c6f6ebf824bccbbbfe1d68cbd17a2085701167`;
`gh api repos/vercel-labs/agent-desktop/commits/main` → still `54855b6225aec00da4ac65e2d0362fd41cd9fcb3`;
`gh api repos/trycua/cua/commits/main` → `465aab0bc2909e6300d7e386986370b17654a24c` (2026-11-12T01:43:27Z, release cua-driver-rs 0.47.15),
10 commits after the wp1b target `e51f57407cac353702d166f24ac776875bbd0b02`. Per criterion c-6 this delta is unit wp1c.

## Upstream commits (trycua/cua, e51f5740..465aab0b) touching libs/cua-driver

| Commit | Change | Files under libs/cua-driver | GOAT disposition |
|---|---|---|---|
| 36351a66 #7401 | GTK3 invoke_menu foreground harness passes `delivery_mode: foreground` | `rust/crates/cua-driver-e2e/tests/harness_gtk3_test.rs` (+4/−2) | TAKE (clean 3-way merge; follows wp1b 267f4922, background invoke_menu never opens a menu) |
| 33102290 #7352 | macOS background type_text: a single-line field gets the line by AX and Return as its own key | `rust/crates/platform-macos/src/tools/type_text.rs` (+236/−3) | PORT onto GOAT's AX-insert rung (5 conflict blocks; see below) |
| 465aab0b #7402 | release 0.47.15 | version lines in `python/pyproject.toml`, `python/src/cua_driver/__init__.py`, `rust/Cargo.toml`, `rust/Cargo.lock` (15), `rust/VERSION`, `rust/Skills/cua-driver/SKILL.md`, `typescript/package.json`, `typescript/package-lock.json` (2); `rust/CHANGELOG.md` | versions KEEP-GOAT (every upstream hunk is only 0.47.14 → 0.47.15; no dependency or `uniffi-bindgen-react-native` pin change, so no lock regeneration); CHANGELOG TAKE (clean merge, 0.47.15 entry on top) |

The other 7 commits touch only `apps/`, `libs/cua/` crates that are not vendored, `libs/cua-spacesd`, CI and docs. Vendored inputs
(`libs/cua/crates/cua-telemetry`, `cua-teleport/src/ux/targets.json`, `libs/images/sandbox-images.json`, `libs/cua/Cargo.toml`) are not in
the compare, so the vendored files stay byte-identical; only `vendor/README.md`'s source commit moves to `465aab0b`.

Method: the changed files were fetched at `465aab0b` with `gh api -H 'Accept: application/vnd.github.raw' repos/trycua/cua/contents/...`
into `/tmp/goat-cu-upstream/465aab0b…/libs/cua-driver`; base files come from the existing `e51f5740` archive. Trial:
`git merge-file` per file into `/tmp/goat-cu-sync/wp1c-trial1` (script `/tmp/goat-cu-sync/wp1c-trial.sh`): conflicts only in the version
lines above and in `type_text.rs` (5 blocks); CHANGELOG and the harness test merge clean.

## 33102290 against GOAT

Upstream bug: Safari's address bar searched for "netflix.com netflix.com": an AX insert of "line\n" into a single-line field cannot read
back (the field drops the newline), and the key retype that followed doubled the text. GOAT does not have the doubling: every
`InsertRead::Readable` outcome of GOAT's AX rung returns (Partial / unconfirmed), so nothing is retyped. It also never submits, which is the
gap. GOAT already proves a whole insert (`ax_insert_progress`: bound target, read-back equals the selection replaced by the text, value
changed), already reads the selection (`selected_range`), and already sends a single Return as its own pid-routed key on the key rungs
(`press_key(pid, "return", &[])`, `single_terminal_return`). The port reuses those instead of upstream's `replace_utf16_range` /
`submit_read_back_matches` / duplicate `selected_range`; Return stays behind `BackgroundKeyboardPolicy::Allowed` and a confirmed keyboard
focus on the written element.

## Dispatch

`cu-wp1c-33102290-s24-261112` (implementation, write scope `platform-macos/src/tools/type_text.rs`), returned codex /
system24/deepseek-v4.1-flash / reasoningEffort=high.

## B — Port (child, lead-audited)

\`type_text.rs\`: \`single_line_submit_body\` (upstream, same roles and Return forms); GOAT \`SubmitPlan\` / \`SubmitWithhold\` /
\`submit_plan\` decide the Return: only for \`TypedProgress::Complete\` from GOAT's bound read-back (\`ax_insert_progress\` on the body that was
written), \`BackgroundKeyboardPolicy::Allowed\`, and confirmed focus (\`focus_element\`, then \`element_has_exact_app_focus(pid, wid, ptr)\`
while the element is live). Return goes out as \`press_key(pid, "return", &[])\`. Withheld Returns say why, claim the body only when the insert
was proven, and \`NotASubmit\` leaves every existing GOAT outcome unchanged (\`InsertRead::Unsupported\` still escalates to keys with the
full text; web/Electron read-backs stay \`Unverifiable\`, so no Return). Not ported (GOAT already covers them): \`replace_utf16_range\`,
\`submit_read_back_matches\`, \`write_left_field_untouched\`, \`submit_write_unconfirmed\`, the duplicate \`selected_range\`.
Behaviour note: an unbound target (focused element, no element token or window) reads back \`Unverifiable\`, so the Return is withheld and
the answer says so; before the port the whole "line\n" was AX-inserted and reported unconfirmed.

Lead audit addition: a failed Return post after a proven insert used to propagate as an error (\`?\`), which a caller would retry and so
insert the line twice. \`submit_return_report\` reports it as the inserted line with an unverified Return instead.

## C — Verification (kache, kache doctor exit 0)

| Command | Exit | Result |
|---|---|---|
| \`cargo test -p platform-macos --lib tools::type_text\` (child, stubs) | 0* | red: \`single_line_fields_split_off_one_trailing_return\` (left None, right Some("netflix.com")), \`submit_return_needs_a_complete_readback_and_confirmed_focus\` (NotASubmit vs SendReturn) |
| \`cargo test -p platform-macos --lib a_failed_return_after_a_proven_insert\` (lead, stub) | 0* | red: panicked type_text.rs:4647 (Err case reported verified) |
| \`cargo test -p platform-macos --lib tools::type_text\` | 0 | 61 passed, 1 ignored |
| \`cargo test -p platform-macos --lib\` | 0 | 2019 passed, 48 ignored |
| \`cargo check -p platform-macos --all-targets\` | 0 | no warnings |
| \`cargo check -p cua-driver-e2e --tests\` | 0 | harness change compiles (the GTK3 harness needs a live Linux desktop: NOT RUN) |
| \`rustfmt --edition 2021 --check type_text.rs\` | 0 | clean (also formats this branch's earlier line 330) |

*the shell pipeline's exit; the cargo test run itself failed as intended.

NOT RUN: live Safari/AppKit submit (no GOAT-owned fixture with a single-line field was driven); the GTK3 e2e harness.


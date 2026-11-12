# 019b — wp1j: upstream delta 7f2252fb → d17e70f5 (trusted browser_click readback, 0.47.22)

Found at the re-fetch after c678bba8 (2026-11-12 ~18:45 UTC): trycua/cua main `d17e70f5`, 2 commits after `7f2252fb` (wp1i, 019),
both in `libs/cua-driver` (12 files); goat-cu origin/main `40c6f6eb` and agent-desktop `54855b62` unchanged. Base/upstream copies:
`/tmp/goat-cu-upstream/7f2252fb…`, `/tmp/goat-cu-upstream/d17e70f5…`; diff `/tmp/goat-cu-upstream/wp1i/d17e70f5.diff`.
Sync commit c8a91fc0.

| Commit | Scope | Disposition |
|---|---|---|
| 57edfe91 (#7503) | release cua-driver-rs 0.47.22 | `rust/CHANGELOG.md` entry IN; version fields KEEP-GOAT (every resolved version file is byte-identical to the parent) |
| d17e70f5 (#7499) | a ref-targeted `browser_click` on the `trusted` route gets the bounded target readback the `dom_event` route has: `DOM.resolveNode` before the press, `read_dom_click_state` before and after; a change reports `confirmed` with `value_readback` evidence, an unchanged target `unverifiable` with readback evidence, a missing readback or unresolvable node `unverifiable` without evidence; coordinate clicks unchanged. Contract doc and five fixture tests | IN (clean merge) |

Method as 015-019 (`/tmp/goat-cu-sync/wp1j-merge.sh`): 4 files clean, 8 version files with conflicts (GOAT's side). The line-multiset check
finds no GOAT line lost and no upstream line missing in `browser/tools.rs`, `browser/v2_tests.rs`, `rust/CHANGELOG.md` and
`docs/action-result-contract.md`. The readback reuses GOAT's existing `read_dom_click_state`, so its bounds and value handling are the
ones the `dom_event` route already had. GOAT's manifest excludes every `browser_*` tool, so GOAT does not call this path.
No protocol change (no schema touched); the installer pin is unchanged (0.47.22 is not published).

### Receipts

| Command | Exit | Result |
|---|---|---|
| upstream's new tests against the parent's `tools.rs`: `cargo test -p cua-driver-core --lib browser::v2_tests::trusted` | 101 | 4 of the 5 new tests fail (v2_tests.rs:4517, :4537, :4565, :4647); `trusted_coordinate_click_has_no_target_to_read_back` passes before and after (it asserts no readback) |
| `cargo test -p cua-driver-core --lib browser::v2_tests` with the merge | 0 | 120 passed |
| Linux container: `cua-driver-core --lib`; clippy `-D warnings` (core, platform-linux) | 0 / 0 | 1830 passed / 3 ignored; clean |
| `TMPDIR=/private/tmp zsh /tmp/goat-cu-sync/receipt-final.sh` at c8a91fc0 | 0 | 24 steps, failures=0: core-lib 1827 / 3, macos-lib 2036 / 48, windows-host 200, linux-host 110, check_protocol compatible, protocol delta problems 0 (050) |
NOT RUN: a live Chromium trusted click against a page.

Review `cu-review-wp1j-c8a91fc0-s24`: PASS. It replayed upstream's hunks onto the parent with `patch --fuzz=0` and got all four files
byte-identical; the readback stays on the bound frame session, adds no input, cannot report `confirmed` without a real before/after
difference, and publishes nothing it read. Two low notes, both upstream's design and shared with the `dom_event` route, recorded as
050 follow-ups (the `DOM.resolveNode` call keeps the connection's 20 s call timeout; the readback reads a field's value in-process on
a tab the secret fence may cover); the ref-targeted trusted dialog branch has no dedicated test.

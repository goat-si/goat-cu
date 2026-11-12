# 019c — wp1k: upstream delta d17e70f5 → 52306ab4 (accepted browser platform defaults)

Found at the re-fetch after ffc428c2 (2026-11-12 ~19:00 UTC): trycua/cua main `52306ab4`, 1 commit after `d17e70f5` (wp1j, 019b), touching
`libs/cua-driver/rust/crates/cua-driver-core/src/browser/platform.rs` only; goat-cu origin/main `40c6f6eb` and agent-desktop `54855b62`
unchanged. Diff `/tmp/goat-cu-upstream/wp1i/52306ab4.diff`, merge `/tmp/goat-cu-sync/wp1k-merge.sh`.

| Commit | Scope | Disposition |
|---|---|---|
| 52306ab4 (#7504, fixes trycua/cua#2884) | each platform adapter's accepted `BrowserPlatform` defaults are declared with a reason (`ACCEPTED_DEFAULTS`), and `every_platform_declares_the_browser_platform_defaults_it_accepts` reads the trait and the three adapter impls and fails when the declared and real sets differ; doc comment on the trait | IN (GOAT's file equalled the base; the merged file is byte-identical to 52306ab4) |

The test reads GOAT's own adapters (`platform-{linux,macos,windows}`), so it checks GOAT's overrides, not upstream's: it passes, meaning GOAT
leaves exactly the declared defaults in place on each platform. No protocol, version or vendor change.
Sync commit bc02a591.

### Receipts

| Command | Exit | Result |
|---|---|---|
| `cargo test -p cua-driver-core --lib browser::platform` | 0 | 4 passed, including the new test |
| Linux container: `cua-driver-core --lib` | 0 | 1831 passed / 3 ignored |
| `TMPDIR=/private/tmp zsh /tmp/goat-cu-sync/receipt-final.sh` at bc02a591 | 0 | 24 steps, failures=0: core-lib 1828 / 3, macos-lib 2036 / 48, windows-host 200, linux-host 110, check_protocol compatible, protocol delta problems 0 |
NOT RUN: none beyond the lane's standing NOT RUN (no runtime surface changed).

Review `cu-review-wp1k-bc02a591-s24`: NEAR-PASS. The merged file is byte-identical to 52306ab4, and its re-implementation of the parser
on GOAT's sources gives the same left-default sets (Linux 7, macOS 2, Windows 6), failing in both directions. Medium: upstream's reason for
Linux/Windows keeping `existing_profile_attach_background_limitation` ("does not bring the browser forward") is false for GOAT's adapters,
which front the browser while opening its setup page and then restore focus (Linux `browser_setup_ui.rs:511`, setup restore mode
`Restore` on every desktop; Windows `browser_setup_ui.rs:281/328/364`) → a2539a20 rewords both (text only; `browser::platform` 4 passed).
Low, recorded: the test never reads the reasons, and its source parser would miss a second impl block (a false failure) or count a
feature-gated override (vacuous); neither shape exists in GOAT's adapters today.

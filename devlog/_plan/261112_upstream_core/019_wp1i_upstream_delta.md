# 019 — wp1i: upstream delta 7dc3cc43 → 7f2252fb (page actions removed, three tools deprecated, Perception frozen)

Found at the re-fetch after 79766176 (2026-11-12 ~18:00 UTC, repeated after the GOAT restart): trycua/cua main `7f2252fb`, 11 commits after `7dc3cc43` (wp1h, 018), 7 of them
touching `libs/cua-driver` (45 files); goat-cu origin/main `40c6f6eb` and agent-desktop `54855b62` unchanged. Patches and per-file
base/upstream copies: `/tmp/goat-cu-upstream/wp1i/`, `/tmp/goat-cu-upstream/7dc3cc43…`, `/tmp/goat-cu-upstream/7f2252fb…`.
Sync commit 190a160e.

| Commit | Scope | Disposition |
|---|---|---|
| b13430e0 (#7483) | `replay_trajectory`, `install_ffmpeg`, `get_accessibility_tree` deprecated (new `deprecation.rs`: a `Deprecated:` notice on every response, a `deprecated` entry and `DEPRECATED` description prefix in `tools/list`, forwarded by the proxy and the SDK adapter; recording's ffmpeg hint names package managers). The `page` tool keeps `get_text`/`query_dom`; its page actions, their arguments and `CUA_DRIVER_ENABLE_LEGACY_PAGE_MUTATIONS` are removed with the Windows bookmark bridge, the Linux/Windows CDP evaluate paths and the macOS Apple Events preferences patcher | IN, combined (below); the non-additive `page` schema is recorded in `goat/compat/allowed-changes.json` |
| a967d132 (#7479) | Cua Perception frozen: README heading, skill notes | IN; the heading note goes on GOAT's `### Optional visual perception` |
| 271ea2c3 (#7486), ef24fe1b (#7493) | releases 0.47.20 and 0.47.21 | `rust/CHANGELOG.md` entries IN (top 40 lines identical to 7f2252fb); version fields KEEP-GOAT |
| f14e6cc2 (#7490) | installers retry release downloads for two minutes past a CDN outage | IN (clean); both installers byte-identical to 7f2252fb except GOAT's `mcp-config` hint line |
| 62677ab0 (#7492) | cua-sdk 0.27.0 | lock `cua-telemetry` 0.27.0; vendored crate version, fork comment and `vendor/README.md` source commit 7f2252fb. `libs/cua/Cargo.toml` changed only version lines, and no `cua-telemetry` source, `targets.json` or `sandbox-images.json` changed in the range |
| c5a3022e (#7498) | GTK3 disabled-control case starts its behavioral video | IN (clean) |
| afe3e212, 2959a5b6, 7a7dba09, 7f2252fb | Cua Spaces releases and bundle, release-queue CI | OUT (no `libs/cua-driver` files) |

The installer download pin stays 0.47.19: it is upstream's recorded published version at 7f2252fb
(`.github/release-state/cua-driver-rs-published-version`), and no 0.47.20 or 0.47.21 release exists yet. GOAT versions kept
(`rust/VERSION` 0.38.0-goat.1, workspace 0.40.1-goat.1, SKILL.md, Python and TypeScript 0.38.0).

Method as 015-018: `git merge-file` per file against the current head (`/tmp/goat-cu-sync/wp1i-merge2.sh`, identical output to the
first trial `wp1i-trial.nM7v`): 33 clean, 12 with conflicts, resolved with `/tmp/goat-cu-sync/wp1i-resolve.py`. Seven of the 12 are
version fields (GOAT's side); `rust/Cargo.lock`'s 17 conflicts are version lines plus upstream-only `cua-agent-setup`/`cua-home`
blocks (GOAT's side). A line-multiset check (`/tmp/goat-cu-sync/wp1i-preserve.py`) finds no GOAT line lost and no upstream line
missing in any file except the intended ones listed here.

### Combinations and decisions

- `page` loses its page actions. GOAT calls only the tools in `CUA_DRIVER_ALLOWED_TOOLS`, which excludes `page`
  (goat-si/goat `packages/shared/src/cuaDriverManifest.ts` at origin/dev c77c9745: `page: "deprecated browser compatibility tool"`),
  and the actions were already refused unless a daemon operator set the switch; a removed action now fails schema validation, or the
  tool's own "page action … was removed" refusal, before anything is sent. `check_protocol.py` reported exactly
  `tools page: input schema changed` in both launch forms; `allowed-changes.json` now records it with that reason (plan 010:79).
- Windows: upstream deletes `tools/page_bookmark.rs` (GOAT-modified by #255) and `page_title_marker.rs`. GOAT's
  `src/page_dispatch.rs` (#255: no CDP fallback after a sent bookmark Invoke, so `execute_javascript` never ran a script twice)
  served only that path, so it goes with it; with no script execution left its invariant holds trivially. #255's keyboard half
  (`keyboard_dispatch.rs`) is untouched. `tools/page.rs`'s conflict takes upstream's side (the bookmark/CDP methods removed).
- `cua-driver-core/src/page.rs`: upstream's read-only description replaces GOAT's shortened one, which described the removed actions.
- `cua-driver-core/src/tool.rs`: GOAT's two root-token refusal tests and upstream's `deprecated_tools_still_work_and_say_when_they_go`
  sit at the same spot; both kept. The deprecation notice is appended after GOAT's result projection and typed-output validation, and
  after the recording summary is taken, so a batch step's first line and the recorded text are unchanged.
- macOS `tools/pixel_restore.rs` (GOAT-only): the focus-restore fence tracked `page` setup, text input and targeted evaluation because
  they could reconnect or relaunch a browser. Those actions are gone, so `page` is never tracked; `MACOS.md`'s paragraph says so.
  Failing first: `page_actions_never_hold_the_restore_wait` (all `page` actions, the removed ones included, expect 0).
- `lease_wiring_tests/source_exemptions.rs`: the `.spawn` exemption for `BrowserJs::enable_javascript_apple_events` is removed with
  the function (the inventory test reported "expected one site, found 0").
- GOAT still allows `get_accessibility_tree` (`cuaDriverManifest.ts:28`). It keeps working until the first release after 26 Nov 2026,
  and each response now carries the deprecation notice naming `list_windows`/`list_apps`/`get_window_state`. GOAT has to drop it from
  its allow list before goat-cu syncs the removal: `check_protocol.py` `validate()` requires every GOAT tool in the candidate.

### Receipts

| Command | Exit | Result |
|---|---|---|
| `cargo test -p platform-macos --lib page_actions_never_hold_the_restore_wait` before the fix | 101 | `insert_text tracking`, left 1, right 0 (pixel_restore_debounce_tests.rs:280) |
| `cargo test -p platform-macos --lib -- pixel_restore lease_wiring` after the fix | 101 | 55 passed, 2 failed: the inventory still listed the removed `enable_javascript_apple_events` spawn |
| `cargo test -p platform-macos --lib` after removing that exemption | 0 | 2036 passed / 48 ignored |
| `cargo test -p cua-driver-core --lib`; `--test '*'` | 0 / 0 | 1822 passed / 3 ignored; every integration target passed |
| `cargo test -p cua-driver --tests` | 101 | 5 socket tests fail on SUN_LEN with this thread's 85-character TMPDIR; every other target passed |
| `TMPDIR=/tmp cargo test -p cua-driver --bin cua-driver` | 0 | 424 passed |
| `cargo test -p platform-windows --lib` | 0 | 200 passed (207 − the 3 `page_dispatch` and 4 `page_title_marker` tests) |
| `cargo check` host (macOS, core, overlay, driver, sdk, contract, all targets) | 0 | |
| `cargo check --target x86_64-pc-windows-msvc` (driver, theme cli, uia, sdk, platform-windows, core, overlay) | 0 | only the recorded warnings (uia/mod.rs:1205, input/inject.rs:314/318/363) |
| `cargo check --target x86_64-unknown-linux-gnu` (platform-linux, core, overlay, driver, sdk; portal-input) | 0 | only the `nom v1.2.4` future-incompat note |
| `cargo clippy -p platform-macos -p cua-driver-core -p cua-driver --all-targets -- -D warnings` | 0 | |
| `check_protocol.py` (darwin-universal) before / after the `page` entry | 1 / 0 | `tools page: input schema changed` ×2 / compatible in every launch form |
| `cua-contract-gen -- all --check`; `generate-uniffi-bindings.mjs --check` | 0 / 0 | |
| `pytest goat/compat/test_check_protocol.py`; `scripts/tests/test_install_connect_step.py` | 0 / 0 | 23 passed; 15 passed |
| `test_install_version_fallback.py` in upstream's layout; `bash -n scripts/_install-rust.sh` | 0 / 0 | 40 passed / 7 skipped (5 new with f14e6cc2) |
| Linux container (rust:1.97.1-trixie): `platform-linux --lib`, `--features portal-input` | 0 / 0 | 1173 / 15, 1232 / 15 |
| Linux container: clippy `-D warnings` (platform-linux, core, cursor-overlay; platform-linux with portal-input) | 0 / 0 | clean |
| Linux container: `cursor-overlay --lib`, `cua-driver-core --lib` | 0 / 0 | 158, 1825 / 3 |
| `TMPDIR=/private/tmp zsh /tmp/goat-cu-sync/receipt-final.sh` at 190a160e | 0 | 24 steps, failures=0 (050); a first run with `TMPDIR=/tmp` failed six `perception_client` sandbox tests (35/35 with the default or `/private/tmp`; code unchanged since 1feaeffb) and was stopped |

NOT RUN: Windows and Linux runtime (no live bookmark/CDP path remains to exercise), the GTK3 and web e2e harnesses, PowerShell
installer tests, a live GOAT session calling the deprecated `get_accessibility_tree`.

Review `cu-review-wp1i-190a160e-s24` (steered at ~19:40 UTC past its bound): NEAR-PASS. It confirmed completeness (42 modified, 1 added,
2 removed = 45), no GOAT line lost beyond the intended sites, `page` read-only end to end, the deprecation notice placement, and that GOAT's
three `tools/list` readers tolerate the `deprecated` key. Findings:

1. Medium-low, fixed in 7cab649b: macOS `type_text` (description, the web next-step note, the `page` escalation reason) still sent a
   browser-tab caller to `page`'s removed `execute_javascript` / `insert_text` / `type_keystrokes`; upstream's file has the same text.
   Now: check the field first, then `get_browser_state` + `browser_type` if the text did not land; the escalation tag stays `page`.
   `browser_tab_guidance_names_the_typed_browser_tools` red at type_text.rs:5065, then `tools::type_text` 62 passed / 1 ignored.
2. Low, fixed in 7cab649b: upstream's replacement test only checked `is_error`; it now asserts the default refusal names the
   unrestricted-mode launch gate and, past it, "was removed" (texts observed by a probe run first); `protocol_schema_test` 2 / 1 ignored.
3. Informational, 050 follow-up: `page` keeps `read_only: false` (as upstream), so page reads are still recorded and batch-scoped as
   mutations.
4. Informational, 050 follow-up: the `deprecated` key sits at the top level of each `tools/list` entry, outside the MCP `Tool` shape;
   GOAT's readers accept it, a strict third-party client might not.

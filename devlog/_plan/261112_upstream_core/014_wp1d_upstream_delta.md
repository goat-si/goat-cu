# 014 — wp1d: Sync libs/cua-driver 465aab0b → 4cf88bae (found at the second pre-D re-fetch)

Re-fetch on 2026-11-12 ~05:40 UTC: origin/main `40c6f6eb` (unchanged); vercel-labs/agent-desktop main `54855b62` (unchanged);
trycua/cua main `4cf88bae51fe7fd8f0518048de113eba1e42a452` (release cua-sdk 0.25.0), 8 commits after the wp1c target
`465aab0bc2909e6300d7e386986370b17654a24c`. Per criterion c-6 this delta is unit wp1d.

| Commit | Change | Files under libs/cua-driver | GOAT disposition |
|---|---|---|---|
| 7dbc78d1 #7400 | Linux hit test picks the GTK3 overlay drawn on top (`SiblingStacking`, `later_sibling_on_top`, at most 32 siblings, GTK ≤ 3 and GAIL only) | `platform-linux/src/atspi/native.rs` (+132/−1), `atspi/native/hit_test.rs` (+2/−1), GTK3 harness test (+72), `tests/fixtures/apps/linux/gtk3/main.py` (+16) | TAKE (clean 3-way merge). It makes point ownership and act-at-point stricter: an overlay drawn over a control owns the point, so GOAT no longer treats the covered control as hit |
| 5d7d729c #5178 | macOS SwiftUI occluder lifetime owned by the e2e test | `cua-driver-e2e/tests/bring_to_front_macos_test.rs` (+9/−52) | TAKE (clean, test-only) |
| 493b6f8d #7416 | release 0.47.16 | version lines (python, Cargo.toml, Cargo.lock 15, VERSION, SKILL.md, typescript) + CHANGELOG | versions KEEP-GOAT (only 0.47.15 → 0.47.16 lines); CHANGELOG TAKE |
| 4cf88bae #7403 | release cua-sdk 0.25.0 | `libs/cua/Cargo.toml` workspace version 0.24.2 → 0.25.0 (+ member path-dep versions) | vendor: `cua-telemetry/Cargo.toml` writes out the workspace values, so its `version` moves to 0.25.0 and `rust/Cargo.lock` follows (one line, via `cargo metadata --offline`; upstream's own lock records the same) |

Other commits (cua-spacesd, CI) are outside libs/cua-driver and the vendored inputs. The vendored sources (`cua-telemetry`, teleport
`targets.json`, `libs/images/sandbox-images.json`) are unchanged in the range; `vendor/README.md` and the fork comment in
`cua-telemetry/Cargo.toml` now name `4cf88bae` (the comment had still named c0d3b919, a pre-existing nit from wp1b review A).

Method as 013: changed files fetched at both commits with `gh api ... contents` into `/tmp/goat-cu-upstream/<sha>/libs/cua-driver`;
`git merge-file` trial `/tmp/goat-cu-sync/wp1d-trial1` (script `/tmp/goat-cu-sync/wp1d-trial.sh`): conflicts only in version lines.

## C — Verification (kache)

| Command (from rust/) | Exit | Result |
|---|---|---|
| `cargo test --locked -p platform-linux --lib` (host) | 0 | 110 passed (the new `SiblingStacking` test is Linux-only, compiled below) |
| `cargo clippy --locked --target x86_64-unknown-linux-gnu -p platform-linux -p cua-driver-core --all-targets -- -D warnings` | 0 | clean |
| `cargo check --locked --target x86_64-unknown-linux-gnu -p cua-driver -p cursor-theme-cli -p cua-driver-sdk -p cua-driver-e2e --features cua-driver/portal-input --all-targets` | 0 | clean |
| `cargo check --locked -p cua-driver-e2e --tests` (macOS host) | 0 | clean |
| `cargo deny --locked --exclude-dev check licenses bans sources` | 0 | ok |

NOT RUN: the GTK3 harness and macOS bring-to-front e2e tests (live desktops); Linux unit tests on a Linux host.

# 017 — wp1g: upstream delta b9e38d1a → 56a69039 (cua-driver-rs 0.47.19 release)

Found at the re-fetch after ff6658cc (2026-11-12 ~11:45 UTC): trycua/cua main `56a69039`, 2 commits after `b9e38d1a` (wp1f, 016) that
touch `libs/cua-driver`. Range `gh api repos/trycua/cua/compare/b9e38d1a...56a69039`; upstream files kept in `/tmp/goat-cu-upstream/wp1g/`.

| Commit | Scope | Disposition |
|---|---|---|
| 00f7352b (#7446) | published installer version 0.47.17 → 0.47.18 in `scripts/_install-rust.sh` and `scripts/install.ps1` | IN: GOAT's value equalled upstream's base (0.47.17), as in wp1e (015) |
| 56a69039 (#7448) | cua-driver-rs 0.47.19 release: CHANGELOG entry for b9e38d1a; workspace/VERSION/python/typescript/SKILL/lock versions 0.47.18 → 0.47.19 | IN: CHANGELOG entry. Versions stay GOAT's (`rust/VERSION` 0.38.0-goat.1, workspace 0.40.1-goat.1, lock, python, typescript, SKILL.md), as in wp1b–wp1f |
| (same range) | `apps/cua-spaces-macos`, `libs/cua-spacesd`, release-please state | OUT: not in `libs/cua-driver` |

No code moved, and the vendored inputs (`cua-telemetry`, `targets.json`, `sandbox-images.json`, `libs/cua/Cargo.toml`) are not in the
range, so `vendor/` keeps `4f2aa028`.

Checks: the top 60 lines of `rust/CHANGELOG.md` and the whole of `scripts/install.ps1` are byte-identical to 56a69039;
`scripts/_install-rust.sh` differs from 56a69039 only at line 1721, GOAT's `mcp-config --client <name>` hint for the removed `connect`
(wp1 review A), the same single difference it had from b9e38d1a. `bash -n scripts/_install-rust.sh` exit 0.

| Command | Exit | Result |
|---|---|---|
| `cargo test -p cua-driver --bin cua-driver release_source` (kache) | 0 | 17 passed (parses `CUA_DRIVER_RS_BAKED_VERSION`) |
| `pytest -q -rs libs/cua-driver/scripts/tests/test_install_version_fallback.py` in a copy of `scripts/` laid out as upstream's `libs/cua-driver/scripts` (the test resolves the scripts at `parents[4]/libs/cua-driver/scripts`, a path that does not exist in this repository, so from `scripts/tests/` all 35 fail on FileNotFoundError; no workflow here runs it) | 0 | 35 passed, 7 skipped (PowerShell not installed) |

NOT RUN: the PowerShell installer cases (no `pwsh` on this host).

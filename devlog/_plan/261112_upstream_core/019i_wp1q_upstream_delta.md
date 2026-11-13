# 019i — wp1q: upstream delta 2319d456 → ed9fd9a5 (published installer 0.47.24, cua-sdk 0.27.1)

Found at the re-fetch after the final review closed (2026-11-13 01:36 UTC), on branch `cu/261112-upstream-core-r3` (PR #300 was
squash-merged as `c93f8dbb`; see 050). trycua/cua main `ed9fd9a5`, 10 commits after `2319d456` (wp1p, 019h); two touch files goat-cu
carries. Merge `/tmp/goat-cu-sync/wp1q-merge.sh` (base 2319d456).

| Commit | Scope | Disposition |
|---|---|---|
| a689a915 (#7528) | published installer version 0.47.24: `scripts/_install-rust.sh:690`, `scripts/install.ps1:166`, `.github/release-state/cua-driver-rs-published-version` | IN: both installers merged clean, only the pin line moves (0.47.23 → 0.47.24); each still differs from ed9fd9a5 only by GOAT's `mcp-config --client <name>` hint line. The release-state file is outside `libs/cua-driver` and not carried |
| 19acc2e2 (#7524) | release cua-sdk 0.27.1: `libs/cua` workspace version and lock, driver lock entries for `cua-agent-setup`, `cua-home`, `cua-telemetry` | IN as a version: vendored `cua-telemetry` 0.27.0 → 0.27.1 (`vendor/cua/crates/cua-telemetry/Cargo.toml`), its `rust/Cargo.lock` entry, vendor README source commit. GOAT's lock has no `cua-agent-setup`/`cua-home` |
| bb965298, 13172caa, 775fa275, 58ec0113, 1fc65ba5, 97ab1566, 34461d7a, ed9fd9a5 | CI cache and toolchain pins, merge-queue lanes, Cua Spaces releases and fixes | OUT (no carried file) |

`cua-driver-rs-v0.47.24` is released (2026-11-12T21:52Z, 58 assets, prerelease-flagged like 0.47.18-0.47.23), and upstream's
published-version file reads 0.47.24. The `cua-telemetry` tree is `0fa62eaf` at 7f2252fb, 2319d456 and ed9fd9a5, and every vendored
`cua-telemetry` source file (`build.rs`, `src/`, `tests/`), `targets.json` and `sandbox-images.json` has the same blob as ed9fd9a5.
The vendored `Cargo.toml` is GOAT's written-out manifest (workspace fields expanded, the `cua-connections` dev-dependency left out; see
`vendor/README.md`), so only its version, its fork comment and the source commit move. No driver release in the range
(`libs/cua-driver/rust/CHANGELOG.md` unchanged). GOAT versions kept. The vendored crate is a path dependency, not a workspace member, so
its own tests do not build here; `cargo check --locked -p cua-telemetry` builds 0.27.1.

agent-desktop main `947b051f`, 2 commits after `54855b62`: a7765959 release 1.3.0 (CHANGELOG, versions) and 947b051f `rustix` 1.1.6
for RUSTSEC-2026-0486. Neither is a core gap from 003; the advisory is recorded as a follow-up in 050.

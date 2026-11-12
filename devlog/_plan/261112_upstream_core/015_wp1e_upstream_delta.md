# 015 — wp1e: upstream delta 4cf88bae → 4f2aa028 (cua-driver-rs 0.47.18)

Found at the third re-fetch before the last D (050, ~08:35 UTC). trycua/cua main `4f2aa028` is 16 commits after `4cf88bae` (synced by
wp1d, 014); `gh api repos/trycua/cua/compare/4cf88bae...4f2aa028` lists 21 changed files under `libs/cua-driver`. Patches:
`/tmp/goat-cu-upstream/wp1e/` (`ac720702.patch`, `377b0fee.patch`, `range.diff`).

## Commits

| Commit | Scope | Disposition |
|---|---|---|
| ac720702 | `invoke_menu` accepts optional `delivery_mode` (background\|foreground) in `InvokeMenuInput` with capability `input.delivery_mode`; Linux stops stripping it before parsing; manifest, Python/TS bindings, docs specs, protocol_schema_test | IN: port |
| 377b0fee | hyprland-plugin requalify: `[omarchy]` listed first in the omarchy-edge channel (`requalify/channels.py`, `test_requalify.py`) | IN: port |
| ee1f7f5f, 4f2aa028, 35a8de84 | cua-driver-rs 0.47.17 / 0.47.18 releases, published installer version | IN: CHANGELOG entries; GOAT keeps its versions (`rust/VERSION` 0.38.0-goat.1, workspace 0.40.1-goat.1), installer scripts and lock version lines stay GOAT's as in wp1b–wp1d |
| 902a2507, 94b6fdd4 | cua-sdk 0.25.1 / 0.25.2: `libs/cua/Cargo.toml` workspace version 0.25.0 → 0.25.2 | IN (vendor): `vendor/cua/crates/cua-telemetry/Cargo.toml` writes out the workspace version; no `cua-telemetry` source, `targets.json` or `sandbox-images.json` change in the range. Refresh the version and `vendor/README.md`'s source commit |
| aad8cc0f, f2201ac5, 42120ab5, 0c1891bf, 4a5f90ba, a6de5982, 6637de44, 2c048fd8, ebea2f86 | spaces, vmm, images | OUT: not in `libs/cua-driver` and not vendored |

## Plan (P)

1. Apply the `libs/cua-driver` part of the range as a 3-way merge onto the branch (as wp1c/wp1d), keeping GOAT versions.
2. GOAT safety: macOS and Windows `invoke_menu` accept `delivery_mode` and keep their behaviour. On macOS a background-only item still
   refuses with `background_unavailable` and names `bring_to_front` / a foreground hotkey (`platform-macos/src/tools/invoke_menu.rs:486`);
   `invoke_menu` never activates a background app. Linux keeps the foreground escalation it already had (`DeliveryMode::from_args`).
3. Failing first: the upstream contract test (`cua-driver-contract` `invoke_menu` capabilities + `delivery_mode` enum) fails on the
   current tree before the contract hunk lands.
4. Protocol: `goat/compat/check_protocol.py` against the darwin-universal baseline; the addition is an optional property on a closed
   schema plus a capability, so it should be provably additive; record the output (and an allowed-changes entry only if the checker asks).
5. Focused checks via kache: `cua-driver-contract --lib`, `platform-macos --lib invoke_menu`, `platform-windows --lib` (host),
   `cua-driver --test protocol_schema_test` (needs the driver binary), Python `test_requalify.py`; Linux `platform-linux --lib` in the
   container.
6. Independent review of the wp1e commit before the final D.

## B record

Trial merge `/tmp/goat-cu-sync/wp1e-trial1` (`/tmp/goat-cu-sync/wp1e-trial.sh`: `git merge-file` of each file, base 4cf88bae, upstream
4f2aa028). Taken from the trial: `cua-driver-contract` `desktop.rs`, `inputs.rs`, `lib.rs`; `cua-driver/tests/protocol_schema_test.rs`;
`platform-linux/src/tools/invoke_menu.rs`; `hyprland-plugin/requalify/{channels,test_requalify}.py`; `rust/CHANGELOG.md` (top identical to
4f2aa028); `scripts/_install-rust.sh`, `scripts/install.ps1` (published installer version 0.47.13 → 0.47.17: GOAT's value equalled
upstream's base, as in earlier syncs). Kept GOAT's: `rust/VERSION`, `rust/Cargo.toml`, the 0.47.x lines of `rust/Cargo.lock`,
`SKILL.md`, python and typescript package versions. Regenerated, not merged (GOAT's generated files differ structurally from upstream's):
`contract/manifest.json` with `cua-contract-gen -- all`, `python/src/cua_driver/_native_contract.py` and
`typescript/src/native/cua_driver_contract.ts` with `node scripts/generate-uniffi-bindings.mjs` (after `npm ci` in `typescript/` for the
pinned UBRN generator). The binding diffs are the `delivery_mode` field and its optional converter, plus one TypeScript doc comment on
`fast_path` that had drifted from its Rust source. Vendor: `cua-telemetry` version 0.25.2 and fork comment, `vendor/README.md` source
commit 4f2aa028, lock entry 0.25.2; vendored sources, `targets.json` and `sandbox-images.json` byte-identical to 4f2aa028.

Found on the way: `contract/manifest.json` was already stale before wp1e (`cua-contract-gen -- all --check` exit 1): the #299 work added
`process_instance` to three output schemas without regenerating it. Fixed separately in b5aa4734 (diff = the three properties; check exit 0).

| Command | Exit | Result |
|---|---|---|
| `cargo test -p cua-driver-contract --lib` with only upstream's `lib.rs` test hunk | 101 | `invoke_menu_requires_an_exact_nonempty_bounded_path`: capabilities lacked `input.delivery_mode` (red) |
| same after the merge | 0 | 91 passed |
| `pytest test_requalify.py -k` the two new omarchy tests, HEAD's `channels.py` / merged | 1 / 0 | 2 failed / 2 passed |
| `pytest test_requalify.py` (whole file) | 1 | 33 passed, 3 failed: `WorkflowContractTest` reads the monorepo's `.github/workflows/nightly-hyprland-plugin-requalify.yml`, absent in this repo's layout; same failures before the merge |
| `pytest python/tests/test_compatibility_contract.py` | 0 | 2 passed |
| `cargo check --locked -p cua-driver -p platform-macos -p platform-windows -p cua-driver-sdk --all-targets` | 0 | no warnings |
| `rustfmt --check` on the touched Rust files | 1 | only `cua-driver-contract/src/outputs.rs:978`, untouched here and identical on origin/main 40c6f6eb |

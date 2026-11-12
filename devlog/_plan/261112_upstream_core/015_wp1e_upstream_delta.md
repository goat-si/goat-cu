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

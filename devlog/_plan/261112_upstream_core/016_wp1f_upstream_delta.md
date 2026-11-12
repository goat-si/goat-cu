# 016 — wp1f: upstream delta 4f2aa028 → b9e38d1a (libxkbcommon 1.0 text functions at run time)

Found at the final re-fetch before the last D (~09:50 UTC): origin/main `40c6f6eb` and agent-desktop `54855b62` unchanged; trycua/cua
main `b9e38d1a`, 4 commits after `4f2aa028` (wp1e, 015), 7 files under `libs/cua-driver`. Patch: `/tmp/goat-cu-upstream/wp1f/`.

| Commit | Scope | Disposition |
|---|---|---|
| b9e38d1a (#6498) | Linux: resolve libxkbcommon's `V_1.0.0` functions (`xkb_utf32_to_keysym`, `xkb_keymap_key_get_mods_for_level`) at run time so the driver and `libcua_driver_sdk.so` load on libxkbcommon 0.10 (Ubuntu 20.04); typing routes that need them refuse with "typing needs libxkbcommon 1.0 or later …; nothing was typed" (on niri before any focus/layout query or keyboard bind); `wayland/dynamic_library.rs` shared with `libpipewire.rs`; `libloading` no longer optional | IN: port |
| 268ac5e7, 3fb7aefe, e3b90902 | Cua Spaces bundling and release | OUT: not in `libs/cua-driver` |

No cua-driver-rs release in the range, so CHANGELOG and versions are unchanged; the vendored inputs (`cua-telemetry`, `targets.json`,
`sandbox-images.json`, `libs/cua/Cargo.toml`) are not in the range, so `vendor/` keeps `4f2aa028`.

Method as 015: `git merge-file` of each file, base 4f2aa028, upstream b9e38d1a (`/tmp/goat-cu-sync/wp1f-trial.sh`): all seven clean,
including GOAT's `wayland/libei.rs`; `dynamic_library.rs` is new. `rust/Cargo.lock` already lists `libloading` for platform-linux, so
`--locked` passes without a lock change. GOAT safety: the refusal is raised before anything is sent, and GOAT's own libei routes compile
against the new `text_sym`/`level_mods`/`find_key` signatures; the readelf test below proves no call site still imports `V_1.0.0`.

| Command (Linux container, rust:1.97.1 aarch64, plain cargo) | Exit | Result |
|---|---|---|
| `cargo test --locked -p platform-linux --lib -- --exact wayland::text_keys::tests::the_crate_does_not_require_libxkbcommon_1_0` with only that test added to the old `text_keys.rs` | 101 | panicked at text_keys.rs:246: the test binary's libxkbcommon entry lists `Name: V_1.0.0` (red) |
| `cargo test --locked -p platform-linux --lib` after the merge | 0 | 1170 passed / 15 ignored, incl. `the_crate_does_not_require_libxkbcommon_1_0`, `a_missing_library_names_the_requirement`, `the_crate_does_not_link_libpipewire` |
| `cargo clippy --locked -p platform-linux -p cua-driver-core --all-targets -- -D warnings` | 0 | clean |

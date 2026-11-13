# 019h — wp1p: upstream delta 2fc7f3c7 → 2319d456 (published installer version 0.47.23)

Found at the re-fetch after 34f79cb5 (2026-11-12 ~21:10 UTC): trycua/cua main `2319d456`, 5 commits after `2fc7f3c7` (wp1o, 019g); one
touches `libs/cua-driver`. Merge `/tmp/goat-cu-sync/wp1p-merge.sh` (base 2fc7f3c7).

| Commit | Scope | Disposition |
|---|---|---|
| d2b06459 (#7520) | published installer version 0.47.23: `scripts/_install-rust.sh:690`, `scripts/install.ps1:166`, `.github/release-state/cua-driver-rs-published-version` | IN: both installers merged clean, only the pin line moves (0.47.22 → 0.47.23); each still differs from 2319d456 only by GOAT's `mcp-config --client <name>` hint line. The release-state file is outside `libs/cua-driver` and not carried (GOAT's `.github/` holds only `workflows/goat-ci.yml`) |
| 4a9ee679, 2aab2f60, 0266f05d, 2319d456 | Cua Spaces SDK, bundle and test fixes | OUT (no `libs/cua-driver` file) |

`cua-driver-rs-v0.47.23` is released (2026-11-12T20:25Z, 58 assets, prerelease-flagged like 0.47.18-0.47.22), and upstream's published-version
file reads 0.47.23.

| Command | Exit | Result |
|---|---|---|
| `bash -n scripts/_install-rust.sh` | 0 | |
| `python3.13 -m pytest -q scripts/tests/test_install_connect_step.py` | 0 | 15 passed |
| `test_install_version_fallback.py` in upstream's `libs/cua-driver/scripts` layout | 0 | 40 passed / 7 skipped |

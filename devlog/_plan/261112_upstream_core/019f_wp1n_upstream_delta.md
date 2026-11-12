# 019f — wp1n: upstream delta 01596ffc → fd4505cf (published installer version 0.47.22)

Found at the re-fetch after 1f1d4edd (2026-11-12 ~20:15 UTC): trycua/cua main `fd4505cf`, 1 commit after `01596ffc` (wp1m, 019e); goat-cu
origin/main `40c6f6eb` and agent-desktop `54855b62` unchanged. Merge `/tmp/goat-cu-sync/wp1n-merge.sh` (base 01596ffc).

| Commit | Scope | Disposition |
|---|---|---|
| fd4505cf (#7509) | published installer version 0.47.22: `scripts/_install-rust.sh:690`, `scripts/install.ps1:166`, and `.github/release-state/cua-driver-rs-published-version` (outside `libs/cua-driver`) | IN: both installers merged clean, only the pin line moves (0.47.19 → 0.47.22); each still differs from fd4505cf only by GOAT's `mcp-config --client <name>` hint line |

Pin policy as in 017: GOAT's 0.47.19 equalled upstream's base value, and `cua-driver-rs-v0.47.22` is released (2026-11-12T19:36Z, 58
assets, flagged prerelease exactly like 0.47.18 and 0.47.19), so the pin names an installable release. GOAT's own versions are unchanged.

| Command | Exit | Result |
|---|---|---|
| `bash -n scripts/_install-rust.sh` | 0 | |
| `python3.13 -m pytest -q scripts/tests/test_install_connect_step.py` | 0 | 15 passed |
| `test_install_version_fallback.py` in upstream's `libs/cua-driver/scripts` layout | 0 | 40 passed / 7 skipped |

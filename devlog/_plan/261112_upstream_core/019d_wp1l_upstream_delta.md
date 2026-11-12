# 019d — wp1l: upstream delta 52306ab4 → 228b5a4d (release 0.47.23)

Found at the re-fetch after b3a6a07e (2026-11-12 ~20:10 UTC): trycua/cua main `228b5a4d`, 1 commit after `52306ab4` (wp1k, 019c), the
cua-driver-rs 0.47.23 release (#7505), 9 `libs/cua-driver` files; goat-cu origin/main `40c6f6eb` and agent-desktop `54855b62` unchanged.
Merge `/tmp/goat-cu-sync/wp1l-merge.sh` (base 52306ab4).

| Commit | Scope | Disposition |
|---|---|---|
| 228b5a4d (#7505) | release cua-driver-rs 0.47.23: CHANGELOG entry (#7499 trusted browser_click readback, #7504 accepted browser defaults, both already synced in wp1j/wp1k) and version fields | CHANGELOG IN (top 40 lines identical to 228b5a4d); version fields KEEP-GOAT: all 8 conflicted files are version-only and resolve byte-identical to the parent |

The installer pin stays 0.47.19: upstream's `.github/release-state/cua-driver-rs-published-version` at 228b5a4d is still 0.47.19 and no
0.47.23 release exists. No code, protocol or vendor change, so the whole-head receipt at 7cab649b (24/24) covers this tree's code.

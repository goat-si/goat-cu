# 010 — wp1: Sync libs/cua-driver 566a4ce6 → c0d3b919 keeping every GOAT patch

Class C4 (cross-platform merge, security-relevant). Stale check at wp1 P: re-run the 001 classification against the
then-current HEAD; any new goat-cu commit on `origin/main` is merged first.

wp1 P stale check (2026-11-11 ~18:40 UTC): `origin/main` still `40c6f6eb`; trycua/cua `main` still `c0d3b919`; HEAD `e3dd8120`
differs from `40c6f6eb` only by `devlog/`, so 001's classification stands. Previous D (wp0): "roadmap locked; next wp1 P
re-verifies 010 then executes B1–B4" — unchanged direction. Architect dispatch `cu-wp1-architect-261112` failed with 429 (004).
Host targets installed: `x86_64-unknown-linux-gnu`, `x86_64-pc-windows-msvc` (cross `cargo check` attempted in C; C toolchain
availability decides RUN/NOT RUN). `cargo-deny` present.

## B1 — Synthetic upstream commits and 3-way apply (no remote, no clone)

```sh
OLD=/tmp/goat-cu-upstream/566a4ce655905c0d28065f4f534e221e92916e24/libs/cua-driver
NEW=/tmp/goat-cu-upstream/c0d3b919fc4621b494353e5a594cd36cd4e125f0/libs/cua-driver
mkdir -p /tmp/goat-cu-sync
GD=$(git rev-parse --absolute-git-dir)          # linked-worktree gitdir; objects go to the shared store
GIT_INDEX_FILE=/tmp/goat-cu-sync/idx-old git -C "$OLD" --git-dir="$GD" --work-tree="$OLD" add -A -f .
T_OLD=$(GIT_INDEX_FILE=/tmp/goat-cu-sync/idx-old git --git-dir="$GD" write-tree)
C_OLD=$(git commit-tree "$T_OLD" -m "trycua/cua 566a4ce655905c0d28065f4f534e221e92916e24 libs/cua-driver (synthetic sync base)")
GIT_INDEX_FILE=/tmp/goat-cu-sync/idx-new git -C "$NEW" --git-dir="$GD" --work-tree="$NEW" add -A -f .
T_NEW=$(GIT_INDEX_FILE=/tmp/goat-cu-sync/idx-new git --git-dir="$GD" write-tree)
C_NEW=$(git commit-tree "$T_NEW" -p "$C_OLD" -m "trycua/cua c0d3b919fc4621b494353e5a594cd36cd4e125f0 libs/cua-driver (synthetic sync target)")
git cherry-pick --no-commit "$C_NEW"      # 3-way: base T_OLD, ours HEAD, theirs T_NEW
```

Objects only; no ref is created. Files only goat-cu has (`GOAT.md`, `LICENSE.md`, `vendor/`, `goat/`, `.github/`) are untouched
because they exist in neither synthetic tree. Gate before resolving: `git diff --cached --name-only | wc -l` + unmerged count
must match 001 (298 upstream paths; 71 conflicted ± files whose merge-ort result differs from `git merge-file`).

## B2 — Resolution policy (every conflict hunk)

| Group | Files | Rule |
|---|---|---|
| V Version/metadata | `rust/VERSION`, `rust/Cargo.toml` workspace `version`, `python/pyproject.toml`, `python/src/cua_driver/__init__.py`, `typescript/package.json`, `typescript/package-lock.json` root version, `rust/Skills/cua-driver/SKILL.md` version line, `README.md` version mentions | KEEP-GOAT values (`0.40.1-goat.1` workspace; `0.38.0`/`0.38.0-goat.1` where GOAT has them); take every other upstream line |
| V CHANGELOG | `rust/CHANGELOG.md` | upstream 0.41.0 … 0.47.12 entries above GOAT's existing entries |
| V Cargo.lock | `rust/Cargo.lock` | take upstream's lock, then let a kache cargo command rewrite workspace member versions; keep GOAT-only deps if cargo re-adds them; `cargo deny` afterwards |
| C Core | `cua-driver-core/src/{recording.rs, lib.rs, tool.rs, protocol.rs, batch_tools.rs, browser/*}`, `cua-driver-sdk/src/{embedded.rs, runtime.rs}`, `cua-driver/Cargo.toml` | COMBINE. Keep: GOAT desktop lease + worker holds with upstream `DesktopLane` (release reverse order); session-scoped snapshot lookups; GOAT private-directory recording writes after upstream `redact_values`; GOAT `BrowserRefusal` detail. Adopt: `security_prompt`, `input_pacing`, `type_pacing` modules, recording restart finalization (#6794), batch nudge (#6921) |
| M macOS | the 31 `platform-macos` files in 001 | COMBINE. Keep: actionability gates (#266, #269), write-outcome classification (#259, #267, #275, #277), exact-field/item/hit-target gates (#278, #280, #282), process-start stamps, key-window hand-back ordering, background-only delivery, exact-target window lease, scroll validation (#297), AXPress-less checked pointer click (#298), GOAT window-local stamp fallback when the OS version is unreadable. Adopt: security-prompt refusal (#7338) ahead of any send, focus arbiter (#6803) as the activation-undo owner, batched AX reads (#6878) feeding GOAT's walk evidence, enabled re-read after activation (#6702/#7041), middle-click routing (#7074), macOS 27 drag stamp (#7372), AXWindows error naming (#7192), keyboard-layout exact text (#6827/#6939) |
| L Linux | the 18 `platform-linux` files | COMBINE. Keep: `spawn_text_write_until` fencing, `gated_mutator` + `element_write_unacknowledged`, single-EIS-region drag refusal, process-instance ids. Adopt the rest |
| W Windows | the 13 `platform-windows` files | COMBINE. Keep: process-instance-bound publish (U15), guarded cursor restore, partial-modifier accounting, "effect uncertain, do not resend" pixel timeout, UIA non-S_OK = uncertain (#247). Adopt upstream keyboard/typing-turn changes |
| T Tests/fixtures | `tests/fixtures/apps/macos/appkit/main.swift`, e2e tests | union of both sides' cases |

Every clean-merged modified-both file in the 5 safety-critical core files (`session.rs`, `tool.rs`, `authorization.rs`,
`window_target.rs`, `health_report.rs`) is read once after the merge for the invariants above, even with no markers.

## B3 — vendor/ refresh from c0d3b919

```sh
S=/tmp/goat-cu-upstream/c0d3b919fc4621b494353e5a594cd36cd4e125f0/libs
diff -rq "$S/cua/crates/cua-telemetry" vendor/cua/crates/cua-telemetry      # list changes first
cp -R "$S/cua/crates/cua-telemetry/." vendor/cua/crates/cua-telemetry/      # then git rm any file upstream removed
cp "$S/cua/crates/cua-teleport/src/ux/targets.json" vendor/cua/crates/cua-teleport/src/ux/targets.json
cp "$S/images/sandbox-images.json" vendor/images/sandbox-images.json
# re-apply vendor/cua/crates/cua-telemetry/Cargo.toml workspace values from "$S/cua/Cargo.toml"; keep cua-connections dev-dep out
```

`vendor/README.md`:

```diff
-Source commit: trycua/cua `566a4ce655905c0d28065f4f534e221e92916e24` (main, 2026-11-06). Files are
+Source commit: trycua/cua `c0d3b919fc4621b494353e5a594cd36cd4e125f0` (main, 2026-11-11). Files are
```

`GOAT.md` §Syncing upstream step 1 names the restart commit; it stays. The newest sync record is the wp1 commit message
(no PR in this lane), which names both upstream SHAs and carries the preservation table.

## B4 — Protocol record

```sh
cargo build -p cua-driver                                   # kache wrapper via ~/.cargo/config.toml
python3 goat/compat/check_protocol.py --candidate rust/target/debug/cua-driver \
  --candidate-version 0.40.1-goat.1 --baseline-key darwin-universal
```

Any non-additive change it reports gets an `allowed-changes.json` entry with reason + GOAT impact, or the merge is changed
to keep GOAT's schema. Additions are listed in the D record.

## C — Verification (fresh, focused, kache)

| Command | Reads the change? |
|---|---|
| `kache doctor` | wrapper health |
| `cargo check -p cua-driver-core -p platform-macos -p cursor-overlay -p cua-driver --all-targets` (rust/) | yes: every merged Rust file on macOS |
| `cargo test -p cua-driver-core --lib` | core merged files |
| `cargo test -p platform-macos --lib` | macOS merged files (no live UI; tests that need a live app are `#[ignore]`) |
| `cargo test -p cursor-overlay --lib` | overlay (wp3 baseline) |
| `cargo deny --locked --exclude-dev check licenses bans sources` | lock/vendor |
| `python3 -m unittest -v goat/compat/test_check_protocol` + B4 | protocol |
| `cargo check --target x86_64-unknown-linux-gnu -p platform-linux` / `x86_64-pc-windows-msvc -p platform-windows` | only if the targets and C toolchains exist on this host; otherwise NOT RUN (CI covers) |

## Preservation table

Filled during B from 001's 71-row table: path | hunks | rule (KEEP-GOAT / TAKE-UPSTREAM / COMBINE) | invariant kept | upstream PRs adopted.

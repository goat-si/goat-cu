# 261125 — Sync libs/cua-driver from trycua/cua ed9fd9a57 to cua-driver-rs-v0.62.0

Lane: branch `cu/261125-upstream-0.62` from goat-cu `main` `3889e119`. One PR to `main`. Coordinator: claude-opus-5-5 (GOAT);
implementation slices and reviews: codex / gpt-6.1-terra / reasoningEffort xhigh, one writer at a time on this checkout.
Not authorized: merge, tags, releases, GOAT version bumps (workspace stays `0.47.24-goat.1`, packages `0.38.0`).

Last synced upstream: `ed9fd9a5757242bb2461373d5dca606b8ef9b9cf` (goat-cu #301). Target: `cua-driver-rs-v0.62.0` =
`0ea979cb6cf299f92e5a72da8b96900b27e0f79e`. 356 upstream commits touch libs/cua-driver (485 files, +60160/-9542).

## Method

Synthetic commits of `libs/cua-driver` at each step boundary live only as objects in this repository's store
(`/private/tmp/goat-cu-sync62/mkchain.sh`, table in `/private/tmp/goat-cu-sync62/chain.txt`: tag, upstream sha, tree, synthetic
commit). `/private/tmp/goat-cu-sync62/apply-step.sh N` cherry-picks synthetic row N+1 onto HEAD without committing (three-way: base
= row N, ours = HEAD, theirs = row N+1) and lists unmerged paths. Files only goat-cu has (GOAT.md, LICENSE.md, vendor/, goat/,
.github/, devlog/, GOAT-only sources) are untouched by the pick. `/private/tmp/goat-cu-sync62/vendor.sh REV` refreshes vendor/.
The upstream clone `/private/tmp/trycua-cua` is read-only reference (`git -C /private/tmp/trycua-cua log/show/diff`).

| Step | Upstream range | Tags | Est. conflicted files vs main |
|---|---|---|---|
| S1 | ed9fd9a57 → b4a62a31c | 0.48.0 | 18 |
| S2 | b4a62a31c → 94bc14727 | 0.49.0–0.49.5 | 23 |
| S3 | 94bc14727 → 31185b10a | 0.49.6–0.49.12 | 19 |
| S4 | 31185b10a → ba69f28b4 | 0.50.0 | 24 |
| S5 | ba69f28b4 → 974d5f149 | 0.51.0–0.51.6 | 19 |
| S6 | 974d5f149 → 81d240c3e | 0.52.0–0.52.6 | 26 |
| S7 | 81d240c3e → b3af0299e | 0.52.7–0.52.13 | 27 |
| S8 | b3af0299e → 09dd7017f | 0.53.0–0.54.0 | 40 |
| S9 | 09dd7017f → d9caafdf1 | 0.55.0–0.55.1 | 37 |
| S10 | d9caafdf1 → 5c56ef26b | 0.55.2–0.55.5 | 18 |
| S11 | 5c56ef26b → 72f5415fe | 0.56.0–0.56.9 | 52 |
| S12 | 72f5415fe → b63bc9918 | 0.57.0–0.58.2 | 24 |
| S13 | b63bc9918 → 181227e2a | 0.59.0–0.59.5 | 28 |
| S14 | 181227e2a → 0ea979cb6 | 0.60.0–0.62.0 | 27 |

## Resolution policy (every conflict hunk, and every clean merge in a safety-critical file)

Read first: GOAT.md ("What differs from upstream", "macOS click and focus refusals", "Embedded process lifetime",
"Windows executable grants"), the bodies of goat-cu PR #300 and #301 (saved locally: `/private/tmp/goat-cu-sync62/pr300.md`, `pr301.md`; no GitHub access needed, read-only gh is allowed), and
`devlog/_plan/261112_upstream_core/010_wp1_upstream_sync.md` §B2 and its preservation tables (the last sync's per-file rules).

| Group | Rule |
|---|---|
| Versions | KEEP-GOAT: `rust/VERSION` `0.38.0-goat.1`, `rust/Cargo.toml` workspace `0.47.24-goat.1`, `python/pyproject.toml`, `python/src/cua_driver/__init__.py`, `typescript/package.json`, `typescript/package-lock.json` root, `rust/Skills/cua-driver/SKILL.md` version line, README version mentions at `0.38.0`. Take every other upstream line (new deps, new crates). |
| CHANGELOG | `rust/CHANGELOG.md`: upstream entries for the step's releases above GOAT's existing entries. |
| Installers | `scripts/_install-rust.sh`, `scripts/install.ps1`: take upstream (pins, mirror download, etc.), keep GOAT's `mcp-config --client <name>` hint line and GOAT's removal of `connect` (GOAT does not ship cua-agent-setup/`connect`). Run the installer tests when they change. |
| Cargo.lock | Take upstream's lock, then let a `cargo` command rewrite workspace member versions to GOAT's; keep GOAT-only deps; `cargo deny` at the end. Upstream crates that GOAT does not build (connect/cua-agent-setup/cua-home) stay out. |
| Core / macOS / Linux / Windows code | COMBINE. Keep every GOAT behavior: safety/refusal semantics (GOAT.md refusal list, #272 #277–#283 #286 #297 #298), element-token + `process_instance` binding and `process_instance_mismatch` (#299), session-scoped fast-path store (#288), cursor parking (#287), AX perform error semantics (#279: IllegalArgument/InvalidUIElement after a send are unverifiable and never followed by a second route), background-only input rules (no foregrounding, no global HID where GOAT refuses), exact-target window lease, embedded process lifetime/process groups, Windows executable identity grants, GOAT `PrivateDir` recording writes, GOAT manifest/session_manifest semantics. Adopt upstream where it does not weaken these. When upstream re-implements something GOAT already patched, keep the stronger behavior (usually GOAT's) and record it. |
| Tests / fixtures | Union of both sides' cases; a GOAT test is never deleted to make a merge compile — adapt it to the merged API and keep its assertion. |
| Removed upstream code GOAT patched | Take the removal only if GOAT's patch has no remaining purpose; record it. |

New tools, removed/renamed tools, schema changes, foreground/focus changes, network/telemetry, installers and permission
flows are listed per step for the PR's security/behavior review.

## Per-step procedure (the implementation slice)

1. `git status` clean (tracked); HEAD is the previous step's commit.
2. `/private/tmp/goat-cu-sync62/apply-step.sh N`. Read `git -C /private/tmp/trycua-cua log --oneline FROM..TO -- libs/cua-driver` and the
   CHANGELOG entries of the step's releases.
3. Resolve every unmerged path by the policy; then read every clean-merged file under `rust/crates/*/src` that GOAT changed
   relative to upstream (`git diff --name-only <row-N synthetic commit> HEAD` ∩ files the step touches) at the merged
   result for the invariants above. Check no GOAT line was silently lost: for each such file, compare GOAT's added lines
   (`git diff <row-N synthetic> HEAD -- f`, the `+` lines) against the merged file.
4. `vendor.sh <TO tag>` if the step's upstream changed vendored files; record.
5. Verify (kache wrapper is configured in ~/.cargo/config.toml; keep `CARGO_BUILD_JOBS=6`; shared, loaded host; never kill processes
   by name pattern): from `rust/`: `cargo check --locked -p cua-driver-core -p platform-macos -p cursor-overlay -p cua-driver --all-targets`;
   `cargo check --locked --target x86_64-unknown-linux-gnu -p platform-linux -p cua-driver-core --all-targets` and
   `cargo check --locked --target x86_64-pc-windows-msvc -p platform-windows --lib` when the step touches those crates (if a C
   build script cannot build for the target on this host, say so: NOT RUN, CI covers); `cargo test --locked -p <crate> --lib <module>`
   for every module the step changed in core and platform-macos; rustfmt on changed .rs files with `/private/tmp/goat-cu-sync62/fmtcheck.sh HEAD` style per-file stdin check (`fmtcheck.sh <base> <head>` after committing, or `git show :<f> | rustfmt --edition 2021 | cmp - <(git show :<f>)` for staged files (stdin `--check` always exits 0); plain `rustfmt --check <file>` also walks submodules and reports files the step did not change); `rg -n '^(<<<<<<<|=======|>>>>>>>)( |$)'` over changed files is empty;
   `git diff --cached --check`. If the protocol changed, `(cd goat/compat && python3 -m unittest test_check_protocol)`.
   Use `TMPDIR=/private/tmp` for cua-driver tests (Unix socket path length).
6. Write `devlog/_plan/261125_upstream_062/0NN_sN.md` (NN = 10+N): upstream commits/PRs in range (one line each, disposition
   IN / ADAPTED / NOT ADOPTED with reason), conflict table (path | rule | invariant kept | upstream adopted), security/behavior
   review notes (new/removed tools, schema changes, input delivery, focus/foreground, network/telemetry, installers,
   permissions), verification table (command | exit | counts), NOT RUN.
7. One commit for the step, code + record: subject `Sync libs/cua-driver from trycua/cua <from9> to <to9> (cua-driver-rs <tags>)`,
   body: the key decisions (KEEP-GOAT/COMBINE highlights, re-implementations where GOAT's stronger behavior was kept),
   verification counts, NOT RUN. Never `git push`; never amend or rewrite earlier commits.


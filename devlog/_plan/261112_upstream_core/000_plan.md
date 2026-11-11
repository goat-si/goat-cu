# 000 — Upstream core catch-up, issue sweep and agent-desktop gaps (roadmap)

Lane: `mac-cua-driver-upstream-core` (dispatch, authority.loop=true, push/openPr/merge=false).
Session: `01a1cbb2-fe9c-7e60-ba5c-bbe64a2a2346`. Goalplan: `.codexclaw/goalplans/hotl-loop-lane-mac-cua-driver-upstream-core-sess/`.
Worktree: `/Users/jun/.goat/desktop-nightly/worktrees/goat-cua-driver/cu-261112-upstream-core-5b1a1bd1acbbaf8d`, branch `cu/261112-upstream-core-r2`.

## Pinned SHAs (recorded 2026-11-11 ~17:45 UTC)

| Ref | SHA | Note |
|---|---|---|
| goat-cu `origin/main` (base, = HEAD at start) | `40c6f6ebf824bccbbbfe1d68cbd17a2085701167` | #298 merged; `git fetch origin --prune` reported no new refs |
| Last synced upstream (goat-cu #290, merge `87e2e869`) | trycua/cua `566a4ce655905c0d28065f4f534e221e92916e24` | 2026-11-06T15:17:07Z, cua-driver-rs 0.40.1 + 1 |
| Target upstream (trycua/cua `main`) | `c0d3b919fc4621b494353e5a594cd36cd4e125f0` | 2026-11-11T17:32:18Z, `rust/VERSION` = 0.47.12 |
| vercel-labs/agent-desktop `main` | `54855b6225aec00da4ac65e2d0362fd41cd9fcb3` | 2026-11-11T06:09:44Z (#642) |

Upstream is read from GitHub tarballs extracted under `/tmp/goat-cu-upstream/<sha>/libs/{cua-driver,cua,images}`
and `/tmp/goat-cu-agent-desktop/<sha>/` (outside the worktree). No upstream clone, remote or worktree is created.

## Merge state of earlier lanes

AD-CORE units merged into `main` before this lane: U19 #221, U20 #223, U22 #225, U21 #226 (+docs #243), U13b #230,
U02c #241, U18 #245, U13c #250, U23b #253, U20b #257, U15 #260. DRV-ISSUES fixes merged: #266, #267, #269, #270, #273,
#275, #277, #278, #280, #282, #291, #292, #294, #295, #297, #298. Their open follow-ups are the issue set in 002.

## Objective

1. Sync `libs/cua-driver` from `566a4ce6` to `c0d3b919` as one 3-way merge that keeps every GOAT patch (wp1).
2. Re-verify and fix or disposition goat-cu #299, #288, #287, #286, #283, #281, #279, #272 on the synced tree (wp2, wp3, wp4).
3. Port the applicable agent-desktop `54855b62` core gaps (wp4).
4. Re-fetch before the last D and turn any new in-scope delta into a unit (wp5).

## Constraints

- Local commits only. No push, PR, merge, release, tag, GOAT pin, GitHub issue/comment write.
- Scope: this worktree's Rust core/contract/platform crates, `goat/compat`, `vendor/`, `GOAT.md`, docs, tests, this devlog.
- Logic/state/protocol fixes: failing-first deterministic Rust regression in an existing test file, minimal fix, fresh focused test.
- Cargo through kache only (`kache doctor`: all checks passed on 2026-11-11). Focused `-p <crate>` commands; no repo-wide gate.
- macOS live checks only against GOAT-owned fixtures (`tests/fixtures/apps/macos/appkit`), never user apps or `~/.goat/userdata`.
- Sub-work dispatch target: GOAT `task.create` `{providerInstanceId:"codex", model:"hyds/deepseek-v4.1-flash", options:{reasoningEffort:"high"}}`.
  Each dispatch's returned selection and outcome is logged in 004.

## Dependency-ordered work-phase map

| Phase | Doc | Depends on | Delivers | Closes |
|---|---|---|---|---|
| wp0 | 000–004 | — | this roadmap | — |
| wp1 | 010 | wp0 | upstream merge, vendor refresh, preservation table, protocol record | c-1, c-3, c-5 (sync part) |
| wp2 | 020 | wp1 | `process_instance` on app/window answers (#299); session-scoped fast-path store (#288) | #299, #288 |
| wp3 | 030 | wp1 | relocated background point re-check (#286); cursor rest parking (#287); gate follow-ups #279 #281 #283 | #286 #287 #279 #281 #283 |
| wp4 | 040 | wp2, wp3 | agent-desktop semantic-delivery visibility parity (#272) and remaining AD gaps | #272, c-7 |
| wp5 | 050 | wp4 | re-fetch, delta unit, final report | c-6 |

wp2 and wp3 touch disjoint files (core `fast_path/store.rs`, `apps`/list tools vs. `tools/click.rs`, `cursor-overlay`, `hotkey.rs`)
and could run in either order; they are sequenced to keep one writer in the checkout.

## Criteria → evidence

| Criterion | Evidence location |
|---|---|
| c-1 SHA table + preservation table | this file; 001 (trial); wp1 commit message + `010` §Preservation table (final) |
| c-2 issue dispositions | 002 (plan) → each wp D record with path:line |
| c-3 protocol record | wp1/wp2 D: `check_protocol.py` output + `goat/compat/allowed-changes.json` diff |
| c-4 failing-first + passing receipts | each wp C: `cxc receipt test` receipts |
| c-5 kache cargo commands | each wp C/D: command + exit code |
| c-6 re-fetch | wp5 D |
| c-7 agent-desktop gap list | 003 → wp4 D |

## Source-of-truth sync target

`GOAT.md` (§Syncing upstream names the last synced commit) and `vendor/README.md` (Source commit). Both change in wp1.
No architecture INDEX doc exists in this repo; the contract docs (`docs/action-result-contract.md`, `docs/tool-output-format.md`)
are patched where a protocol-visible field is added (wp2).

## NOT RUN in wp0

Builds, tests, protocol check, live macOS runs. wp0 is docs-only.

## Roadmap lock (wp0 B)

Documents 000–005 and 010–050 form the locked roadmap. Audit: 005 (independent reviewer unavailable, 429; lead self-audit
NEAR-PASS, two citation/procedure blockers folded). Next cycle: wp1 P re-verifies 010 against the then-current
`origin/main` and upstream, then executes B1–B4.

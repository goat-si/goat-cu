# 004 — Dispatch log

Contract: every investigation, implementation and review slice goes through GOAT `task.create` with target
`{providerInstanceId:"codex", model:"hyds/deepseek-v4.1-flash", options:{reasoningEffort:"high"}}`; record the returned
provider/model/options; if the provider is unavailable, record the actual return and the lead continues.

| When (UTC) | Task | clientRequestId | Returned selection | Outcome |
|---|---|---|---|---|
| 2026-11-11 ~17:50 | R1 upstream preservation table | `cu-r1-preservation-261112` | codex / hyds/deepseek-v4.1-flash / reasoningEffort=high | failed: "exceeded retry limit, last status: 429 Too Many Requests" |
| 2026-11-11 ~17:50 | R2 agent-desktop gap list | `cu-r2-adgap-261112` | codex / hyds/deepseek-v4.1-flash / reasoningEffort=high | failed: "exceeded retry limit, last status: 429 Too Many Requests" |
| 2026-11-11 ~17:50 | R3 issue revalidation | `cu-r3-issues-261112` | codex / hyds/deepseek-v4.1-flash / reasoningEffort=high | failed: "exceeded retry limit, last status: 429 Too Many Requests" |
| 2026-11-11 ~18:30 | A1 roadmap audit (wp0), mode wait | `cu-a1-roadmap-audit-261112` | codex / hyds/deepseek-v4.1-flash / reasoningEffort=high | failed: "exceeded retry limit, last status: 429 Too Many Requests"; host CPU 98% |
| 2026-11-11 ~18:40 | wp1 architect proposal, mode wait | `cu-wp1-architect-261112` | codex / hyds/deepseek-v4.1-flash / reasoningEffort=high | failed: "exceeded retry limit, last status: 429 Too Many Requests" |
| 2026-11-11 ~18:42 | wp1 plan audit, mode wait | `cu-wp1-audit-261112` | codex / hyds/deepseek-v4.1-flash / reasoningEffort=high | failed: "exceeded retry limit, last status: 429 Too Many Requests" |

Coordinator decision (2026-11-11, during wp1 B): use `codex / system24/deepseek-v4.1-flash / reasoningEffort=high` (same
DeepSeek v4.1 Flash model; hyds rate-limited); stop hyds retries; no other substitute without a new decision.

| When (UTC) | Task | clientRequestId | Returned selection | Outcome |
|---|---|---|---|---|
| 2026-11-11 ~19:20 | wp1 Linux conflict resolution (12 platform-linux files) | `cu-wp1-linux-s24-261112` | codex / system24/deepseek-v4.1-flash / reasoningEffort=high | running at dispatch |
| 2026-11-11 ~19:20 | wp1 Windows conflict resolution (10 platform-windows files) | `cu-wp1-windows-s24-261112` | codex / system24/deepseek-v4.1-flash / reasoningEffort=high | running at dispatch |
| 2026-11-11 ~19:55 | wp1 sync independent review (macOS + core) | `cu-wp1-review-s24-261112` | codex / system24/deepseek-v4.1-flash / reasoningEffort=high | failed: "Error running remote compact task ... Provider error 400 ... Invalid 'input[76].name': string does not match pattern" |
| 2026-11-11 ~20:20 | wp1 review A (core, narrowed) | `cu-wp1-review-a-s24-261112` | codex / system24/deepseek-v4.1-flash / reasoningEffort=high | running at dispatch |
| 2026-11-11 ~20:20 | wp1 review B (macOS input, narrowed) | `cu-wp1-review-b-s24-261112` | codex / system24/deepseek-v4.1-flash / reasoningEffort=high | running at dispatch |
| 2026-11-11 18:57 | wp2–wp4 issue revalidation on the merged tree (read-only) | `cu-wp24-revalidate-s24-261112` | codex / system24/deepseek-v4.1-flash / reasoningEffort=high | running at dispatch |

The lead produced 001–003 directly after these failures.

Correction: the `~19:20`/`~20:20` times above were estimated without reading a clock. `clock.curr_time` read
2026-11-11 18:57:59 UTC at the revalidation dispatch, so those four dispatches happened between ~18:40 (wp1 P) and 18:57.

2026-11-11 19:15 UTC: steered `cu-wp1-windows-s24-261112` (thread.send, delivery "steered") with the line-audit findings: HEAD's
per-unit `UnitFailure::Short`/`Stalled` mapping in platform-windows input/keyboard.rs and the `keyboard_dispatch_error` arm in
tools/hotkey.rs and tools/type_text.rs were missing from its working copy mid-edit. Linux files without markers at that time
(atspi/mod.rs, get_screen_size.rs, launch_app.rs, set_value.rs, wayland/hyprland.rs, wayland/region_route.rs): name and line
audits clean.

2026-11-11 20:08 UTC: `cu-wp1-windows-s24-261112` completed (codex / system24/deepseek-v4.1-flash / high). Steered items 1-3 addressed.
Lead re-verification found one compile error and three unused imports in its result via a Windows-target type check (010); fixed and staged.

2026-11-11 20:35 UTC: `cu-wp24-revalidate-s24-261112` completed (codex / system24/deepseek-v4.1-flash / high); report kept in 012,
key claims spot-checked by the lead.

2026-11-11 ~20:40 UTC: wp1 reviews A/B and Linux resolution completed (all codex / system24/deepseek-v4.1-flash / high); dispositions in 010.
2026-11-11 ~21:00 UTC: wp2 §C #288 implementation dispatched — `cu-wp2-288-s24-261112`, returned codex / system24/deepseek-v4.1-flash /
reasoningEffort=high, status running. Lead implements §A/§B in parallel on disjoint lines.
2026-11-12: `cu-wp2-288-s24-261112` completed; lead audited the diff, re-ran its checks, added a runtime red check and re-pinned two
lease-wiring inventory fingerprints (020). `cu-wp2-review-ab-s24-261112` (#299 review, codex / system24/deepseek-v4.1-flash / high)
still running at this entry.
2026-11-12: wp2 #288 independent review dispatched — `cu-wp2-review-288-s24-261112`, role review, returned codex /
system24/deepseek-v4.1-flash / reasoningEffort=high, status running.
2026-11-12: wp2 committed e0851769 ahead of both wp2 reviews; both reviewers steered to read commit e0851769 (`git show`).
2026-11-12: wp3 #287 implementation dispatched — `cu-wp3-287-s24-261112` (write scope cursor-overlay cursor_pose.rs / render_state.rs /
render_map.rs), returned codex / system24/deepseek-v4.1-flash / reasoningEffort=high, status running.
2026-11-12: wp3/wp4 review dispatched — `cu-wp34-review-s24-261112` (commits 393c49e2, 68771fdb), role review, returned codex /
system24/deepseek-v4.1-flash / reasoningEffort=high, status running.
2026-11-12: wp2b #299 Linux/Windows implementation dispatched — `cu-wp2b-299-lw-s24-261112`, returned codex /
system24/deepseek-v4.1-flash / reasoningEffort=high, status running.
2026-11-12: wp1b review dispatched — `cu-wp1b-review-s24-261112` (commit ef7a3646), role review, returned codex /
system24/deepseek-v4.1-flash / reasoningEffort=high, status running.
2026-11-12: `cu-wp2-review-ab-s24-261112` (#299) completed NEAR-PASS, 4 findings accepted → 09484534; decisions recorded
(review_findings.record). Round 2 dispatched with continueFrom — `cu-wp2-review-ab-r2-s24-261112`, returned codex /
system24/deepseek-v4.1-flash / reasoningEffort=high.
2026-11-12: `cu-wp3-287-s24-261112` completed; lead verified (156 overlay tests), added the GNOME helper `park` field, committed
a5e2bbde. Fresh review dispatched — `cu-wp3-287-review-s24-261112`, returned codex / system24/deepseek-v4.1-flash / reasoningEffort=high.
2026-11-12: `cu-wp2-review-288-s24-261112` (#288) completed NEAR-PASS → 0d163dd1 (P2 accepted, P3 ×2 accepted, P3 rebutted); round 2
`cu-wp2-review-288-r2-s24-261112` (continueFrom) NEAR-PASS → fb4f50e0 (P2 accepted: session-agnostic window mapping replaced by the
caller's derived read session); round 3 `cu-wp2-review-288-r3-s24-261112` dispatched. All returned codex / system24/deepseek-v4.1-flash / high.
2026-11-12: `cu-wp2-review-ab-r2-s24-261112` (#299 round 2) PASS with 3 P3 (1 rebutted, 2 accepted). `cu-wp2b-299-lw-s24-261112`
completed; lead moved the listings' first pass before enumeration, committed 0654beae; round 3 `cu-wp2-review-ab-r3-s24-261112`
dispatched (continueFrom), returned codex / system24/deepseek-v4.1-flash / reasoningEffort=high. Decisions recorded with
review_findings.record for every round.
2026-11-12: `cu-wp1b-review-s24-261112` FAILED (actual return: "Error running remote compact task: Provider error 400 … Invalid
'input[124].name': string does not match pattern" — the same provider compaction failure as the first wp1 review); no findings were
recorded before it. Retried split in two narrower reviews — `cu-wp1b-review-a-s24-261112` (prepare.rs, lockfiles, docs) and
`cu-wp1b-review-b-s24-261112` (cursor release vs click ordering, Linux gates), both returned codex / system24/deepseek-v4.1-flash /
reasoningEffort=high.
2026-11-12: final receipt dry run (/tmp/goat-cu-sync/receipt-final.sh, not a recorded receipt) on ece9964a: failures=0.
2026-11-12: `cu-wp3-287-review-s24-261112` (#287) NEAR-PASS, 5 findings accepted → e354f982 (macOS parks without frame ticks via
an idle-fade deadline; a parked float restarts on a moving SnapTo, a re-show, a theme reselect). `cu-wp2-review-288-r3-s24-261112` (#288)
PASS with 5 P3 → fd7ef859 (derivation gated on the mode, mode-free wire test, two more session shapes). `cu-wp1b-review-a-s24-261112` PASS
with 3 P3 → 4fc4252b (Windows unused_mut); the other two recorded as follow-ups. Decisions recorded with review_findings.record.
2026-11-12: continuation rounds dispatched — `cu-wp3-287-review-r2-s24-261112`, `cu-wp2-review-ab-r4-s24-261112` (#299, e894eabd),
`cu-wp2-review-288-r4-s24-261112`; all returned codex / system24/deepseek-v4.1-flash / reasoningEffort=high.
2026-11-12: pre-D re-fetch found trycua/cua main at 465aab0b (0.47.15); unit wp1c added (013). Port of 33102290 dispatched —
`cu-wp1c-33102290-s24-261112`, returned codex / system24/deepseek-v4.1-flash / reasoningEffort=high.
2026-11-12: `cu-wp34-review-s24-261112` (393c49e2, 68771fdb) FAIL: P2 press_key/right_click/double_click still use the legacy
perform-error helper (IllegalArgument/InvalidUIElement read as "nothing sent", then re-press / key chord / pixel fallback); P2 pixel
right-click fallback runs without the pointer gate after 68771fdb's clipping-free semantic gate; P2 040's case-3 claim does not match
`bind_exact_with` (corrected in 040/050, case 3 is a follow-up); P3 APIDisabled refusal untested; P3 foreground double_click pixel fallback
unchecked. Fix dispatched — `cu-wp3c-perform-gate-s24-261112` (isolation worktree from 4fc4252b), returned codex /
system24/deepseek-v4.1-flash / reasoningEffort=high.
2026-11-12: `cu-wp1c-33102290-s24-261112` completed (type_text.rs +241/−2, 60 type_text tests, 2018 lib); lead audit added
`submit_return_report` (a failed Return after a proven insert no longer surfaces as a retryable error), committed 19fcc334. Review
dispatched — `cu-wp1c-review-s24-261112`, returned codex / system24/deepseek-v4.1-flash / reasoningEffort=high.
2026-11-12: `cu-wp2-review-ab-r4-s24-261112` (#299 round 4) NEAR-PASS, 3 P3 accepted → ad5f6193 (docs, 020) + follow-up recorded.
2026-11-12: `cu-wp2-review-288-r4-s24-261112` (#288 round 4) PASS, 1 P3 accepted → 39cabbe0.
2026-11-12: `cu-wp1b-review-b-s24-261112` FAILED (actual return: "exceeded retry limit, last status: 429 Too Many Requests";
run terminationCause quota after 2 h 26 m; it had named two forming findings but reported none). Re-dispatched narrower as
`cu-wp1b-review-b1-s24-261112` (cursor release_t) and `cu-wp1b-review-b2-s24-261112` (Linux X11 gates) with the partial notes.
2026-11-12: `cu-wp3c-perform-gate-s24-261112` FAILED (actual return: "exceeded retry limit, last status: 429 Too Many Requests"),
`cu-wp1b-review-b2-s24-261112` FAILED (same). wp3c left a partial uncommitted diff (click.rs perform_outcome shared + legacy helper
deleted + APIDisabled tests; press_key decide_chord mapped) in its worktree goat-5023485b-df387c95a2840a7c; the lead applied it to the lane
worktree and continues the slice (lane rule: record the actual return, the lead continues).
2026-11-12: lead completed wp3c → 98632052 (+ b724ece4 rustfmt of this branch's macOS files); red: 8 tests under the old meaning;
green platform-macos --lib 2023 passed / 48 ignored. wp3/wp4 review decisions recorded (6 accepted). Round 2 dispatched with continueFrom —
`cu-wp34-review-r2-s24-261112`; B2 retried — `cu-wp1b-review-b2-retry-s24-261112`; both returned codex / system24/deepseek-v4.1-flash /
reasoningEffort=high.
2026-11-12: `cu-wp1b-review-b2-retry-s24-261112` and `cu-wp34-review-r2-s24-261112` FAILED (actual return: "exceeded retry limit, last status:
429 Too Many Requests"). B2 reviewed lead-side (011 §C B2). wp3/wp4 round 2 to be re-dispatched after the running reviews finish.
2026-11-12: PR #300 pushed at 458ff8f6 under the then-current authority; goat-ci failed (Windows serve.rs, clippy) → d0e90760, 2a726372.
Delivery rule then changed: the coordinator pushes and manages the PR; this lane pushes nothing more (watch removed).
2026-11-12: second pre-D re-fetch → wp1d (133b2ef9). Final receipt on 9bd0e96c: failures=0.
2026-11-12 04:28 UTC: GOAT server restart cancelled the four running reviews; GOAT resumed them as run 2. `cu-wp1c-review-s24-261112` finished:
NEAR-PASS, 3 P2 + 3 P3 → 1ec667ea (5 accepted, 1 rebutted with ad5f6193 evidence; decisions recorded). `cu-wp3-287-review-r2-s24-261112`,
`cu-wp1b-review-b1-s24-261112` and `cu-wp3c-review-s24-261112` logged nothing after the restart notice (04:28) through 05:04; not duplicated.
Their scopes (e354f982, ef7a3646 cursor release, 98632052) are named in the fresh final review of the exact head.
2026-11-12: `cu-wp3c-review-s24-261112` finished: NEAR-PASS, 2 P2 + 1 P3 on 98632052 (scroll-bar AXPress then wheel, uncoded
right/double-click refusals, press_key AXConfirm/AXCancel classification) → 0f0d4b70; restoring the old behaviour gave 4 red tests,
then platform-macos --lib 2026 passed / 48 ignored and clippy -D warnings exit 0.
2026-11-12: goat-ci on PR #300 at b54560e5: Linux `mutter_display` descriptor-count test failed 81 vs 79 (parallel tests' descriptors
in a process-wide count) → test runs alone in a child test process (050 "CI after the receipt"). Left local for the coordinator to push.
2026-11-12: `cu-delta-review-019c3858-s24` dispatched (0f0d4b70 + 019c3858), codex / system24/deepseek-v4.1-flash / reasoningEffort=high.
2026-11-12 08:38 UTC: `cu-final-review-12e47ec6-s24` FAILED (actual return: "Error running remote compact task: Provider error 400 ...
Invalid 'input[124].name': string does not match pattern"), the third long review lost to compaction; no report. Stalled
`cu-wp3-287-review-r2-s24-261112` (no activity since the 04:28 restart) cancelled. Replaced by three narrow fresh final-round reviews of
the commits without a review of their final form: `cu-final-287-e354f982-s24`, `cu-final-wp1c-1ec667ea-s24`, `cu-final-ci-wp1d-s24`
(d0e90760, 2a726372, 133b2ef9); all returned codex / system24/deepseek-v4.1-flash / reasoningEffort=high.
2026-11-12 ~09:45 UTC: GOAT server restart cancelled all five (task.read: status cancelled), none had reported. Re-dispatched at b0c7a85a with
the same narrow briefs (`-r2` ids), plus `cu-review-wp1f-ci-b0c7a85a-s24` for 3be03b46 (wp1f) and b0c7a85a (CI test fix); all six returned
codex / system24/deepseek-v4.1-flash / reasoningEffort=high.
2026-11-12: `cu-review-libei-57e9cae1-s24` PASS (one INFO, the CI coverage follow-up, accepted). The pre-restart
`cu-final-wp1c-1ec667ea-s24` completed after all: NEAR-PASS, P2 (landed line still carried a foreground re-send escalation) accepted →
895e6ad2 (failing-first assertion red, then green; platform-macos --lib 2026 / 48, clippy clean), P3 mapping coverage rebutted, P3
char counts accepted as recorded residual; decisions recorded with review_findings.record. Its `-r2` duplicate cancelled; fresh final
review of 895e6ad2 dispatched as `cu-final-wp1c-895e6ad2-s24` (codex / system24/deepseek-v4.1-flash / high).
2026-11-12: the pre-restart `cu-final-ci-wp1d-s24` also completed: PASS (bindings move pure, re-pin = FNV-1a of the moved file, serve.rs
restores exactly what Windows calls, 133b2ef9 equals upstream with versions kept, vendored crate = 4cf88bae). Three P3/INFO accepted as
recorded follow-ups (GTK3 stacking scope and budget fail-open, upstream/pre-existing; rust/VERSION lag); `-r2` duplicate cancelled.
2026-11-12: the pre-restart `cu-wp1e-review-068c9a00-s24` completed: b5aa4734 PASS, 068c9a00 NEAR-PASS. P2 (batch foreground fallback
would retry macOS invoke_menu) rebutted: a run_steps step tool must be in BATCHABLE_TOOLS (batch_tools.rs:2602-2621, :36-47), which
lacks invoke_menu, and the click menu-bar reroute keeps `tool == "click"` (:1757-1772), so the gate never asks whether invoke_menu
accepts delivery_mode. P3 (015 implied the upstream docs specs were taken) accepted, 015 corrected. `-r2` duplicate cancelled.
2026-11-12: `cu-final-wp1c-895e6ad2-s24` NEAR-PASS: seam hint still said "typing again" (accepted → 83aeb256, red via
recovery_hint::annotate then green), delivery mapping unpinned (accepted → completed_delivery + table test, red on a scratch flip),
counts residual (accepted). The pre-restart `cu-final-287-e354f982-s24` completed: NEAR-PASS, 5 P3: GNOME doc, parked repin,
motion restart, doc residue accepted → 1f2060af (red first: set-motion case; repin test compile); macOS seam test rebutted as
residual. `-r2` duplicate cancelled. Decisions recorded with review_findings.record.
2026-11-12: `cu-delta-review-019c3858-s24-r2`: 0f0d4b70 NEAR-PASS, 019c3858 PASS. P2 accepted: the coded undelivered refusals carried
code/effect/dispatched but no typed disposition, and `ToolResult::proves_no_input` reads only that, so a run_steps batch whose only
step sent nothing was typed DeliveredUnverified / retry Unsafe → ff6658cc (`not_delivered_refusal` and scroll's non-fronted refusal
typed NOT_DELIVERED; 3 red without it, including a batch-level run_steps test through the real constructor, then green;
platform-macos --lib 2031 / 48, clippy -D warnings clean). P3 GOAT.md APIDisabled wording and P3 missing batch test accepted in the
same commit. P3 Linux env marker accepted: the marker carries the spawning pid; in the container, with the marker leaked as 1 and a
scratch typo in the child filter, the old check passed (no child) and the new one failed (child ran 0 tests); final code green with
and without the leaked marker, platform-linux --lib 1170 / 15, clippy clean. The pre-restart original `cu-delta-review-019c3858-s24`
had resumed as a second run; cancelled as a duplicate. Decisions recorded with review_findings.record. Still running after the
11:2x UTC GOAT restart: `cu-review-wp1f-ci-b0c7a85a-s24`, `cu-final-83aeb256-1f2060af-s24`.
2026-11-12: fresh reviews dispatched for the new commits: `cu-review-ff6658cc-s24` (typed NOT_DELIVERED refusals, Linux fd marker) and
`cu-review-wp1g-8248c43b-s24` (wp1g sync); both returned codex / system24/deepseek-v4.1-flash / reasoningEffort=high.
2026-11-12: `cu-review-wp1f-ci-b0c7a85a-s24`: 3be03b46 PASS (each file equals a git merge-file of GOAT base, 4f2aa028, b9e38d1a; every
V_1.0.0 use goes through the resolved table and refuses before sending), b0c7a85a PASS (diagnosis reproduced from the code, test-only).
P3-1 (type_text_chars drops the structured refusal) rebutted: Linux does not register TypeTextCharsTool (`tools/impl_.rs:5196-5198`),
and `renamed_tool` routes type_text_chars calls to type_text (`cua-driver-core/src/tool.rs:883`, `:2091`), whose errors go through
`text_route_error_result`. P3-2 (static/bundled libxkbcommon) and P3-3 (unit tests on a 0.10 host) accepted as recorded follow-ups
(050). P3-4 (isolate_desktop doc) accepted → fd7b782a (core check and clippy --all-targets -D warnings exit 0). Decisions recorded with
review_findings.record. The detached whole-head receipt started last turn was killed at turn end (nohup did not survive); rerun inside
the turn at eb04e2a0: 24 steps, failures=0 (050).

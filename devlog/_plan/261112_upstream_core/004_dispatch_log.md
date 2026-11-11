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

The lead produced 001–003 directly after these failures.

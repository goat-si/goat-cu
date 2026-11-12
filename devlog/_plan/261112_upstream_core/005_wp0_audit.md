# 005 — wp0 audit record

## Independent reviewer

GOAT `task.create` `cu-a1-roadmap-audit-261112`, role review, target codex / hyds/deepseek-v4.1-flash / reasoningEffort=high
(returned selection matched), mode wait: `status: failed`, summary "exceeded retry limit, last status: 429 Too Many Requests".
Host pressure reported by GOAT at that call: CPU 98% (elevated). No independent review exists for wp0.

## Lead self-audit (not independent)

Verdict: NEAR-PASS after folding the items below.

1. FOLDED — citation drift: list_apps `:66-85` → `:69-84`; click.rs bounds `:2462-2481` → `:2470-2489`, relocation
   `:2527-2556` → `:2527-2554`; evaluate.rs visible gate `:127` → `:131`; wire.rs delta_target `:205-214` → `:199-215`.
2. FOLDED — 010 B1: `git --work-tree=<archive> add .` from the repo cwd rejects `.` as outside the work tree; the procedure now
   runs `git -C <archive> --git-dir=<abs gitdir> --work-tree=<archive>` with an absolute temp index.
3. CHECKED — AD-G1 rule cannot admit a pointer click on unknown visibility: `without_pointer` requires `Policy::headless()`
   and `semantic_pointer` requires `!background_window && !headed` (`evaluate.rs:113-126`); the #298 checked pointer route and
   pixel clicks use other policies.
4. CHECKED — order: wp1 must precede wp2–wp4 because 9 of the cited files conflict in the sync (click.rs, hotkey.rs,
   get_window_state.rs, tools/mod.rs, apps/mod.rs …); wp2/wp3 file sets are disjoint; wp4 depends on wp3's click.rs.
5. RESIDUAL — #287 option 2 changes the fork's visible rest behavior; it is the issue's own option 2 and stays inside the
   driver. #283.1/#281.1 live smoke depends on a GOAT fixture build on this host.
6. RESIDUAL — Linux/Windows halves of wp1 and the Linux/Windows half of #299 cannot be compiled here.

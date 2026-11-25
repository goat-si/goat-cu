# 261125 — GOAT fix step after S14 (plan)

Head at start: c71144fa (S14). Ledger: 090_findings.md (committed with this file). Every fix: failing-first test (red, then green) unless the item is docs/record-only; one writer at a time on this checkout; one commit per slice with subject "fix(<area>): <summary> (sync 0.62 fix slice Fn)"; record in 03N_fN.md (N = slice). Ledger rows stay as they are; each slice record lists the rows it closes with the commit and test names.

## Required audits (read-only, at c71144fa; their confirmed findings join the slice that owns the area)

- OPEN-1 no-resend / NOT_DELIVERED producers (incl. S10-P1, S13-R2-N1).
- OPEN-2 password exposure across all output paths (Windows, macOS AXSecureTextField, Linux password role).

## Slices (in order)

| Slice | Area | Ledger rows |
|---|---|---|
| F1 | macOS input: blocking Highs + no-resend | S9-R1-H1, S9-R2-H1, S12-R1-M1, S13-R2-M1, S9-R1-M1, S3-R1-M1 |
| F2 | security prompts + passwords (mandatory, GOAT pins this release) | S12-R2-M1 (+S12-R2-M1-DIR), S8-R2-H1, OPEN-2 results, S12-R2-N1 (GOAT.md) |
| F3 | macOS drag/menu/cursor/popup/activation | S4-B1, S4-R1-M1, S4-R1-M2, S4-R2-M1, S4-R2-L1, S12-R2-L1, S13-R1-M1, S3-M1, S3-M2, S5-P3, S3-P2 |
| F4 | core sessions, snapshots, batch, daemon identity, SDK | S5-R2-M1 (+S6-R1-N1, S6-R2-M1), S5-R1-M1, S5-R1-M2, S5-R1-L1, S5-R2-L1, S1-M1, S1-M2, S1-R2-M1, S10-R1-M1, S3-P1, S11-R1-N1, S11-R2-M1, S6-R2-M2, S5-P2, S4-P1, OPEN-1 results (core) |
| F5 | browser consent, Linux setup, Windows | S2-H1, S2-R2-H1, S2-R2-M1, S7-R1-H1, S7-R1-M1, S4-P2, S7-R2-M1, S11-R1-M1, S5-P1, S1-R2-L1, S14-P1 (X11 if contained), OPEN-1 results (platform) |
| F6 | tests, docs, records, protocol notes | S6-R2-M3, S8-R2-M1, S8-R2-M2, S9-R2-M1, S13-R2-L1, S10-R1-L1, S12-N1 (CHANGELOG), S1-R2-L2, S2-L1 and every record correction named in the ledger, S2-R3-N1 / S5-N1 consumer check (goat-si/goat read-only), S9-N1 advisories vs main |

Not fixed here (PR follow-up list, with path:line from the ledger): S6-P1, S7-P1, S14-P1 remainder, S9-R1-D1 (maintainer decision), S13-R1-N1.

## After the slices

Full verification at the final head (cargo check --locked host set; clippy -D warnings for platform-macos, cua-driver-core, cua-driver and cursor-overlay on macOS; lib tests for changed crates; protocol, contract, installer tests; cargo deny licenses bans sources; Linux container run of cua-driver-core if docker is available), fresh Terra final review of the PR head, push, PR, goat-ci slow_tests dispatch.

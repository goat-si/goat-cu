# F6 PR notes

## Consumer check (S2-R3-N1 / S5-N1)

Read-only `git grep` on goat-si/goat checkout commit `531b9679021e714420af476b8e84b932a625cad2`, origin verified, supplemented by GitHub code search (default branch; search indexing and tokenization apply). No repository writes. Exact searches for `detail.candidates`, `drag_session`, `enter_pressed`, `ui_tree_revision`, and `custom_actions` returned no paths. Literal `ax_timeout` appears as get_window_state degradation fixtures in `apps/server/src/screenCode/DesktopObserveDepth.test.ts:363`, `DesktopObserveLatency.test.ts:108`, and `DesktopObservedEffect.test.ts:328`, not the removed Linux click escalation. `escalation` appears in screenCode refusal fixtures `apps/server/src/screenCode/DesktopDriverRefusals.test.ts:657,711` for screen unlock and in `DesktopInputPlan.ts:169` as a comment on foreground key routing; no identified consumer of the removed Linux click field. GitHub search `z_index` returned `apps/web/src/components/preview/sessions/NativeBrowserPicture.tsx`, `apps/desktop/src/preview/PickPreload.ts`, `apps/web/src/components/preview/ThreadPreviewMiniPlayer.tsx`, `apps/web/src/components/preview/previewMiniPlayerLayout.ts`, and `apps/server/src/screenCode/DesktopTargetHost.linuxBind.test.ts` (the latter has a numeric fixture at line 21). Hits are not evidence that nullable driver z_index is consumed. Broader `candidates` matches are common unrelated names; no exact detail.candidates consumer was identified. These searches do not establish compatibility for dynamic or differently spelled consumers.

## RUSTSEC comparison (S9-N1)

Each crate and exact version from `/private/tmp/goat-cu-sync62/s9-deny-full.log` compared with `git show 3889e119:rust/Cargo.lock`. Informational presence only; not a vulnerability assessment.

| Advisory | Crate | Version | Present in main lock |
|---|---|---|---|
| RUSTSEC-2026-0190 | anyhow | 1.0.102 | yes |
| RUSTSEC-2026-0374 | basic-toml | 0.1.10 | yes |
| RUSTSEC-2026-0543 | chrono | 0.4.45 | yes |
| RUSTSEC-2026-0555 | chrono-tz | 0.10.4 | yes |
| RUSTSEC-2026-0385 | ctutils | 0.4.2 | yes |
| RUSTSEC-2026-0186 | memmap2 | 0.9.10 | yes |
| RUSTSEC-2026-0194 | quick-xml | 0.39.4 | yes |
| RUSTSEC-2026-0195 | quick-xml | 0.39.4 | yes |
| RUSTSEC-2026-0486 | rustix | 1.1.4 | yes |
| RUSTSEC-2026-0346 | rustls | 0.23.45 | yes |
| RUSTSEC-2026-0347 | rustls | 0.23.45 | yes |
| RUSTSEC-2026-0348 | rustls | 0.23.45 | yes |
| RUSTSEC-2026-0349 | rustls | 0.23.45 | yes |
| RUSTSEC-2026-0386 | tokio | 1.52.1 | yes |
| RUSTSEC-2026-0192 | ttf-parser | 0.21.1 | yes |
| RUSTSEC-2024-0449 | unsafe-libyaml | 0.2.11 | yes |
| RUSTSEC-2026-0592 | url | 2.5.8 | yes |
| RUSTSEC-2026-0371 | zbus_names | 4.3.2 | yes |
| RUSTSEC-2026-0367 | zip | 8.6.0 | yes |
| RUSTSEC-2026-0368 | zip | 8.6.0 | yes |
| RUSTSEC-2026-0369 | zip | 8.6.0 | yes |
| RUSTSEC-2026-0429 | zip | 8.6.0 | yes |
| RUSTSEC-2026-0545 | zip | 8.6.0 | yes |
| RUSTSEC-2026-0546 | zip | 8.6.0 | yes |
| RUSTSEC-2026-0547 | zip | 8.6.0 | yes |
| RUSTSEC-2026-0548 | zip | 8.6.0 | yes |
| RUSTSEC-2026-0391 | zstd | 0.13.3 | yes |
| RUSTSEC-2026-0398 | zstd-safe | 7.2.4 | yes |
| RUSTSEC-2026-0403 | zstd-safe | 7.2.4 | yes |
| RUSTSEC-2026-0410 | zstd-sys | 2.0.16+zstd.1.5.7 | yes |
| RUSTSEC-2026-0370 | zvariant | 5.12.0 | yes |
| RUSTSEC-2026-0372 | zvariant_derive | 5.12.0 | yes |

# Status: Performance and minimum spec

Owner thread: "Measure performance for minimum specs". Covers: PF.

Only the owner thread edits this file. One row per package: update it in the same pull request as the package. Logs are in `handover/packages/`. Index and standing rules: `handover/START-HERE.md`. Stage order: `handover/design/SSE4-coming-together.md`.

| Package | State | Log |
|---|---|---|
| PF performance readout and minimum spec (`design/SSE4-performance-and-minimum-spec.md`, 22 Nov): one recorder and chunk list (`perf-chunks.json`) for a readout switched on in More, a bench on `/harness/`, and a minimum spec table built from reports | PF1 the readout (`packages/PF1.md`), PF1b players, PF2 the bench on `/harness/` and `tests/harness/bench.mjs --against` (ND3b's 5 % timing check), PF3 `docs/minimum-spec.md` from `docs/perf-reports/` (`packages/PF2.md`) done; reports from AC-ProArt and the Pixel 11 Pro in; opening Voyages measured again after V8 (in-system `plan_voyage` 630 ms, then about 450 ms with the hardware square root, then 259 ms after PR 245: within the 300 ms budget); next: an app readout report from the phone | packages/PF2.md |

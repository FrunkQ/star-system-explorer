---
name: sse4-citations-and-fudges
description: Citation check track and the fudge register: where they live, Alex's 19 Nov rules, state at 19 Nov evening, and how web sources can be reached from the cloud container
metadata:
  type: project
  modified: 2026-11-19T22:22:31.140Z
---
- Thread "Verify physics citations" (started 19 Nov 15:15). Registers in the repo: `docs/citations.md` (index with per-area counts; status words verified/corrected/unverified/textbook) with one file per area under `docs/citations/`, and `docs/fudges.md` (one table per area plus "Candidates to move into flows"). Golden changes and reasons per area: `handover/packages/CITE.md`.
- Alex's rules (19 Nov, in handover/START-HERE.md): real physics over simplification where practical, any fudge known, documented and with a very good reason; do everything in the physics chain blocks, Big Blocks only for what can't be a flow, each with a UI element that visualises it.
- State 19 Nov 22:25: first pass of all 7 areas merged to main (PRs 22, 29, 30, 32, 33, 34-37). About 540 rows checked, about 65 fixed, about a third still unverified (fetch limits). Physics changed: isotope shares and Al-26 heat, Zeng radius unit, planet disc brightness 2/3, Sizes tab star data, methane lifetime and H escape, star multiplicity, giant tide factor (was inverted; Venus now engulfed), pebble isolation diffusion term, FMBK merger ratios, Holmberg 0.40, Hawking 1.14e67 yr (photons+gravitons), Kopparapu 2014 runaway limit by mass.
- Still open: unverified rows; HPT00 appendix A; Z-dependent core-luminosity constant; Barkana-Loeb cooling mass; Juric, Maoz; nucleus sources (checked, fixes in kernel PR 28; add rows to docs/citations/galaxy-cosmology.md once PR 28 is on main); Yuan and Narayan 2014 L ~ mdot^2 passage unread.
- CI speed-up merged 21 Nov (PR 39, Alex confirmed "yes to speed up - test periodically - the odd break is fine in alpha"): one run per PR, superseded PR runs cancelled, Rust and npm caches, docs-only PRs skip build jobs, UI-only PRs skip the three-OS Rust matrix; every job still runs on each merge to main and nightly at 03:17 UTC.
- Network: curl to doi.org, crossref, arxiv, ADS, Wikipedia is blocked (403). WebSearch then WebFetch on result URLs works; budget about 100 fetches/hour shared.

Related: [[sse4-build-conventions]]

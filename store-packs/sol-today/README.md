# Sol today (store pack, track RS)

This pack is published on Explorers and is **not** built into SSE4. Nothing under `store-packs/` is loaded by the kernel or the app; the onboarding tour fetches this pack from the store to teach loading a pack (Alex, 23 Nov 2026).

- `catalogue.json` is generated: `python3 -I scripts/spaceflight/import_gcat.py <gcat tsv dir> store-packs/sol-today`. Do not edit it by hand.
- `stations.json` is authored.
- Data licence: GCAT data is CC BY 4.0 and the credit line in `pack.json` must travel with the pack. Our own data and code are Apache-2.0.
- Design and packages: `handover/design/SSE4-real-spaceflight-feasibility.md`, `handover/status/spaceflight.md`.

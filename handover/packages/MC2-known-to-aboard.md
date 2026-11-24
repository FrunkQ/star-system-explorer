# MC2: Known to on entries aboard

Track MC (ship manifest and crew). Owner thread "Build ship manifest and crew roster". Builds on MC1 and on SV1 (kernel, actors covered) and SV2 (the shared eye).

## What it does

- Each person and lot in the Aboard panel carries the shared `KnownTo` eye from chart-grammar, shown on the Build and Run desks only (the same `eyesShow` rule as the Ships and stations card). A row not known to the table is drawn in pencil.
- A bar under the Crew/Manifest switch offers "Show the crew to the table" (or "the manifest"): one `set_known` with scope `{list: [...]}` for every entry on that side not yet known, so one Undo (`undo_known`) takes it all back.
- New entries are as known as their place (Known to design question 6): under the Kestrel player-data ladder a new actor reads as "heard of" (rung long-range), not GM only. The aboard.* kept kinds are tier "scanned", so they reach the table once an entry is known.
- `set_known` and `undo_known` are sent at the campaign clock (`campaignNow`), because `known_to` reads at the campaign clock. Sending at the sky's time (`ws.now_yr`) put the change after "now" when the sky ran ahead, so it did not show. The Ships and stations card and the body card still send at `ws.now_yr`; their owners were told.

## Checks

- `node tests/ui/manifest.e2e.mjs` (by hand, not in CI's list): Build desk, three entries start not known, show all makes them known, Undo returns them to where they started.
- `npm run ui:check`, `npm run ui:test` (459 pass), `node tests/ui/setup.e2e.mjs`.
- No Rust change; no golden change.

## Screens

/mnt/project-files/sse4/handover-screens/MC2-screens/ (crew-known-desk.png shows the eyes after show all).

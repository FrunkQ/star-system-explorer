# Run-through: the party's ship from voyage to landing (blocker 4, gap 9)

Landing run-through for next week (gap list: `/mnt/project-files/sse4/plans/landing-run-through-gaps.md`). Travel thread, 23 Nov 2026.

## What a GM can do
1. Ships and stations: on a ship whose template names a nav ship (`construct.nav_ship`, every Traveller ship) press **Make this the party's ship**. The list marks it "party".
2. Voyages starts where the party's ship is, picks it in the Ship picker as "<name> (the party's ship)", and **Fly <name> there** moves it to the destination.
3. "<name> is at Mars. **Land here**" opens Land on that world (`#chart/world/<sys>/<world>~surface`).
4. With no party ship, picking a world on the chart (or a moon of it) makes that world the voyage's start (gap 9).
5. Voyages' names sit in up to four rows under the orbit line and never overlap; a name with no room shows on hover; the start and goal always get theirs (gap 9). After an arrival the goal moves to the next world on, so the chart stays drawn.

## How
- `packages/ui-contract/src/party.ts`: `PartyShip {ship, name, nav, at, voyage?}`, `navShipOf`, `partyLander`, `voyageStart`. `Workspace.party` and `setParty`, saved with the campaign beside `overlay_anchors` (older saves have none).
- `labelLanes` in `voyage.ts` (greedy lanes, start and goal first).
- Kernel: `plan_descent` takes `lander: {ship: <id or nav ship>}`, landed on its own numbers by `sse_nav::entry::ship_as_lander` (the same hull, shield, aero, gear and engine conversion `ship_at_world`'s entry check flies). No protocol shape change, no golden change.

## Seams agreed
- Renderer (Land): reads `ws.party`; when `party.at` is the world on screen it offers the party's ship first and sends `partyLander(party)`. Players replay the descent from the same inputs (`LandingShow`), so no telemetry is streamed. The site deep link (`site=lat,lon`) comes from the renderer.
- Chart room: adds `party` as its own key on the players' host state beside `shown`; this thread then calls `share.host.setState({ party })`. "Fly there" never moves players' screens; the GM's "Send this view" does.

## Limits
- Where the ship is lives on the workspace (`at`), not yet in the record; it moves to the construct's flight log (V8 flight log) later.
- Only ships whose template names a nav ship can be the party's ship.

## Checks
- `cargo test -p sse-kernel plan_descent` (new `plan_descent_lands_the_partys_own_ship`).
- `tests/ui/party.test.ts` (3), `tests/ui/party.e2e.mjs` in chromium (added to `ui:e2e`); `voyage.e2e.mjs` still passes.
Screens: /mnt/project-files/sse4/handover-screens/RT-party-ship-screens/ (not in the repo).

## Part 2: a game's own task beside the dangers (X3h, Traveller thread's ask; Alex approved, GM screens only)
- At world card (under a voyage): each danger shows `gameDifficulty(ladder, {p_wrong, role})`, e.g. "Piloting, Formidable 14+".
- Route console: beside the difficulty word, `gameDifficulty(ladder, {word})`.
- Only on the GM's seat, only when a stacked pack gives a ladder; nothing rolled. `party.e2e.mjs` checks every danger carries one with the Traveller pack on.
- Seen in passing, next: the GM-seat At world card names the world by its address ("Fit it for u:…/body:3"); ship_at_world should carry the body's name.

## Part 3: the party's ship goes to the players (24 Nov)
- App.svelte sends `ws.party` to every players' screen through the chart room's `party` host-state key (PR 344) whenever it changes, and null when there is none. It never moves their view: "Send this view" does.
- `p2p.test.ts`'s round trip now carries a typed `PartyShip` with its last voyage.
- Next (chart room or renderer): the players' screens show where the ship is.

## Part 4 (24 Nov): world names on the At world card

The kernel's `ship_at_world` names a NAV body by its address (adopt's
PlanetSpec carries no drawn name), so the GM's At world card read "At
u:kestrel/sys:sol/body:3" and "Fit it for u:…". Voyages now passes the
chart's own name into `AtWorld` (`name`), used when the report's name is
empty or an address. party.e2e checks the heading. The kernel-side fix
(PlanetSpec names from the names layer) is left to the kernel thread; this
covers every screen that shows the card.


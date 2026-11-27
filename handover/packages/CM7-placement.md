# CM7: designer placement by mount (compound ships)

Track CM. Design: `handover/design/SSE4-compound-ships-design.md` section 8.2 and 13 (on PR 679 at the time of writing), accepted by Alex 27 Nov 2026 00:43 UTC, with his rulings in section 14A winning. Built 27 Nov 2026 by the thread "Build the snap-together ship builder". Renderer only; no golden moves.

## What it is

Each construct of a compound is still laid out on its own (`designShip`, unchanged for a lone ship); `packages/renderer/src/compound-layout.ts` places those layouts by their mounts and sizes bays from what they hold.

- **Reading the kernel's facts** (CM1, PR 682): `mountFromFacts` (`mount.on`, `.place`, `.port`, `.bay`, `.bond`, `.lines`, `.station.along_m`, `.clock_deg`, `.stand_off_m`; `docked.to` reads as a latch on the hull, as the kernel reads it), `portsFromFacts` (`port.<id>.place`, `.size_m`, `.lines`, and `.station.*` when a later package writes it), `baysFromFacts` (`part.<id>.bay_length_m`, `bay_radius_m`, `bay_door_m`). Only names are shared with CM1; nothing here calls the kernel, so CM7 does not wait on PR 682 to merge.
- **The five places**: front (stern at the carrier's nose), behind (nose at the carrier's stern), hull (at `along_m` from the nose and `clock_deg` round, clock 0 up and 90 towards +y, belly touching the carrier's surface found along that clock line), outside (as the hull plus `stand_off_m`, with a tether drawn), inside (in its bay; several in one bay sit up to `bay_per_layer` round each layer, layers along the bay). A construct on the hull or outside is turned so its own up faces away from the carrier; `turn_deg` (the builder's turn, a recipe value for CM14, never a kernel fact) turns it further.
- **Bays** (`SystemKind` `bay`): `sizeBays` sizes each bay from what it holds (largest held construct's length and reach, times `bay_clearance`, arranged round and along), and the kernel's typed sizes win. `DesignShip.bays` lays them as bodies straight after the hold, or where a style's `order` puts `bay`. A carrier with things inside and no bay part gets one, and a note says so. `designCompound` designs carried constructs first, so a carrier's bay follows from them (capabilities drive form).
- **Ports**: rings on the hull (`PortMark`), front at the nose, behind at the stern, hull ports at their station or spread round the middle (with a note), drawn as ellipses in the plans (`shipViews`) and as tubes in the model (`compoundMesh`), from the same `ringPoints`.
- **Turned panels**: `Piece.roll_deg` turns a panel's plane about its root line, so a construct's wings and radiators turn with it on a hull. `shipViews` and `shipMesh` draw it; pieces without it draw exactly as before.
- **Fit notes, warn never refuse**: longer or wider than its room in a bay, a bay door too small to leave by, strap-ons with fixed bonds and drives not balanced round the axis (a latch takes no thrust, design 6.4), something in front of a carrier whose drive is at its nose, two constructs that overlap, a hull mount or port with no station, a carried construct whose carrier is not in the picture.
- Rules are pack data, authored, picture only, in the fudge register: base-ships `ship_design.compound` (`bay_clearance` 1.15, `bay_per_layer` 4, `port_ring_share` 0.12, `tether_share` 0.02).

## What it does not do yet

- **No screen shows it.** The 2D picture for compounds (outlines per construct, joints, call-outs per construct, the sequence strip), the Carried fold and the object menu actions are CM8; the builder is CM14. The call-outs `shipViews` draws today are per system, so two constructs' tanks share one label.
- In 3D a bay is a closed body, so what is inside is hidden; the 2D picture inks it through the hull (CM8) and the builder's cutaway is CM14.
- Where a port sits on the hull is the designer's choice until the kernel writes `port.<id>.station.*`.
- Stations (the station designer's docks as ports, modules as constructs) are CM12.
- The compound's plume and `needs` are the stern-most stage's; per-stage figures are the staging block (CM5) and the sequence (CM6).

## Checks

- `packages/renderer/test/compound-layout.test.ts` (8 tests, in `npm run ui:test`): end-to-end stacking and the root kept; placing never resizes (each construct's body volume unchanged); a bay sized from what it holds and holding it, typed sizes winning, with notes for too long, too wide and a small door; six in one bay with no clash; hull contact along the clock line, turn, centring, balance (and a latch never counting), overlap; outside stand-off and tether; facts read back; port rings on their circle, drawn in plans and model; a turned wing reaching along z in the model.
- Gates seen red: with the belly left out of the hull distance, the behind offset at 0, a bay not widened for its ring, the balance check switched off, and the turn left off, the tests fail (1, 1, 1, 1 and 2 failures); restored, all pass.
- Existing ship tests unchanged and passing (`ship-design`, `ship-mesh`, `ship-abilities`).

## Pictures

Project files `sse4/handover-screens/CM7-screens/`: `stack.png` (a capsule root with an upper stage behind it holding a lander in a bay sized from it, a core stage behind that, and two fixed boosters on the core's hull) and `carrier.png` (six probes in a hangar bay, four round a layer; a shuttle at a hull port; a pod on a 30 m tether), drawn by `shipViews` from `designCompound`. Stand-in craft resized from the base-nav ships; not real vehicles.

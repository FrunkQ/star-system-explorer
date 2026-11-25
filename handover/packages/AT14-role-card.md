# AT14 (role card half): the Interface a role wears

Thread "Build Run desk playbook and players' side", 25 Nov 2026. Atlas and reports design (`handover/design/SSE4-atlas-and-reports-design.md`, 11A.4, accepted 24 Nov 20:14); seam agreed with "Build star atlas and editions" on 25 Nov.

## What
- `Seat.kit?: string`: the interface kit a role wears, a KitDoc id ("sse.kit.hauler", or a pack's own), read with the Atlas's `resolveKit`. Absent: today's screens.
- `Seat.palette?: string`: one of the kit's palettes; absent is the kit's first, its default.
- Both travel with the seat plan like `brand`: saved with the campaign and carried to players' screens with the seats.
- `SeatChange.kit` and `.palette` (`null` takes one away). Choosing another kit forgets the palette. `checkSeats` checks both are ids, and that no palette is set without a kit.
- The role card: "Interface" (Plain, or a starter kit: Hauler, Corporate, Deck, The Guide), and under it the kit's palettes as swatches drawn in their own ground, ink and accent. The old "Look and sensors" line now says only Sensors.

## Part 2: kits on players' screens (agreed with the Atlas thread)
- On a players' screen, a layout chosen for the role on the Switchboard (`screens[0]`) wins. Otherwise the kit's layout for the situation applies (`kitScreen(seat.kit, situation)`, PR 549), and failing both, the Bridge.
- Each part gets the kit's control set as `ctx.control_set` (`controlSetOf(kit)`), beside `ctx.controls`. Absent means the plain set.
- Built: ui-contract `layouts.ts` `situationOf(shown)` (held: holding; a landing; a send about one thing: world; a system; wider: sector) and `screenFor` (a send's own screen, then the role's chosen layout, then `kitScreen(kit, situation)`, then the Bridge). The players' page follows `seat.kit` as it changes, reloads when the situation changes the kit's layout, and passes `controlSet` through ScreenRuntime to every part. A kit layout naming a part this screen cannot draw yet (the players' Atlas part, AT15 part 2) falls back to the Bridge instead of a screen of errors. The palette on players' screens is AT15 part 2's.
- Tests: layouts.test.ts (situations and precedence).

## Checks
- `packages/ui-contract/test/seats.test.ts`: kit and palette set, kept, forgotten on another kit, removed; checkSeats lines. `ui:check`, `ui:test`.
- `table.e2e`: on the Sensors role card, choose Hauler; its first palette is the default; choose Amber; Corporate starts at its own default. p2p and switchboard e2e pass.

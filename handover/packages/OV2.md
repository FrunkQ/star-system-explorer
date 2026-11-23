# OV2: territory map layers, the contract

Territory overlays design (`handover/design/SSE4-territory-overlays-design.md`), accepted in full by Alex on 23 Nov 2026 ("yes to all"). OV2 is the contract only. The kernel's OV3 gives holdings, and OV4 draws them in the Atlas look.

## What it adds to `ui-contract/src/overlay.ts`

- **`OverlaySpec.areas`** (`AreaSpec`, design 4.1) holds:
  - `by`, one of: a relation, a tag group, a choice, a fact (optionally cut into `bands`), `knowledge: "charted"`, or a `reach` (from a place, a radius given or read from a fact, an optional rate);
  - `answers`, `level` (`"auto"` or a depth), and `fill` (`web`, `between_stars` as a reach or `"grid"`, `in_system` as hill or bands, `on_surface` as regions or outlines);
  - `weigh_by`, `look` and `tier`.
- **`frame` is now optional.** A map layer of areas is about places, so every view draws it in its own frame. A frameless overlay without `areas` is dropped as before.
- **`areaProblems(a)`** says in words what is wrong with a layer as written, for a GM, a pack maker or Orac. **`areaSpecsOf(packs)`** returns the layers that read cleanly, with the higher pack winning.
- **New layer kind `areas`.** Items are `{ key, ink, status, strength?, rings, label?, label_turn?, divided?, place? }`, and `as` is the stacked role. `areaRole(i)` gives fill, then edges, then marks (design 4.6).
- **Statuses:** holds, claims, influences.
- **Map palette.** `OverlayInk` gains `{ slot: 1..8, pattern? }`, with `MAP_SLOTS`, `MAP_PATTERNS` and `mapSlotToken(n)` (`--sse-map-n`; the themes define the colours in OV4).
  - `assignSlots(answers, touching)`: neighbours never share a slot, and a preference is kept when it is free.
  - Slots are reused by answers that do not touch.
  - It is deterministic by key order, so every device colours a map alike.
- **New frames:** `body-latlon` (x is longitude east and y latitude north, in degrees) and `web-mpc`.
- **`Holding`**: the shape `holdings_in` and `who_holds` return, agreed with the kernel thread for OV3.
  - Fields: `place`, `layer`, `key`, `status`, `strength?`, `extent?` (reach, band, or an outline in a frame), `by?`, `since_s?`, `until_s?`, `source?`, `via` (held, inherited, rolled-up or rule), `divided?` and `why`.
  - The kernel sends no geometry. Views turn holdings into rings, except an outline, which travels as stored.
- **Constants:** `AREA_REACH_DEFAULT` (4 pc, ruling 6), `AREA_DIVIDED_BELOW` (0.6, ruling 5) and `AREA_LOOKS`.

## Drawing for now

- `chart-grammar/Overlays.svelte` draws an `areas` layer as outlines: solid where held, dashed where claimed, dotted where influenced. The stroke is the slot colour, or plain ink where a theme has no map palette yet.
- Plain inks and slot inks now work on every layer kind.
- The Atlas look comes in OV4.

## Checks

- `ui-contract/test/overlay.test.ts`: OV1's three tests pass unchanged, plus four new ones:
  - a layer round-trips through the stack;
  - every source reads, and mistakes are worded;
  - palette slots behave as described above;
  - stacked roles come out as fill, edges, marks.
- `ui:check` is clean. `ui:test` passes apart from minerva's local-only "sifter hits".

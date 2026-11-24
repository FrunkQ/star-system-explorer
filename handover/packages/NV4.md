# NV4: Find by tag and light it up (V3 gap 41)

Status: done 24 Nov 2026 (thread "Build object selector and menus"). Accepted by Alex on 24 Nov (second V3 audit, item 41).

## What it does
- **Find by tag**: typing "#" and a tag in the picker finds what carries it (NV2 part 1, `matchesQuery`). "#ocean" finds Tarn, tagged `class:planet/ocean`.
- **Light up**: with a "#tag" search showing hits, **Light up all N on the chart** marks every one of them wherever it is drawn, with a glow in the accent. A chip on the place line ("#ocean ×") says what is lit and turns it off; so does pressing the button again.
- **One general rule, no view edits**: anything that stands for an object carries `data-address` (the system chart's bodies today, and any view that adopts it later), so a style keyed on the lit addresses marks them. The picker's own rows are left out.
- Tags stay labels only: lighting up reads them and changes nothing; the state hash does not move.

## Where
`chart-grammar/src/Picker.svelte` (`lit`, `onlight`, the button); `shell/src/nav/PlaceLine.svelte` (the lit state, the style, the chip).

## Checks
`tests/ui/object-menu.e2e.mjs`: "#ocean" lights Tarn on the Kestrel chart (computed filter), the chip turns it off.

Screenshot: project files `sse4/handover-screens/NV4-screens/`.

## Not yet
- The map: lighting up the systems that hold a tag on the galaxy chart (needs the campaign's systems drawn there with `data-address`; galaxy charts thread, with item 17).
- A colour per tag (V3 let a tag carry a colour): symbology already has tag colours (V12); a lit tag could use its colour.

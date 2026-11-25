# AT1: the Atlas's page model

Track AT (atlas design `handover/design/SSE4-atlas-and-reports-design.md`, sections 4.3, 5, 10, 12). Built 24 Nov 2026.

## What it is
`packages/ui-contract/src/atlas/`:
- `edition.ts`: `sse.edition/1` documents, `resolveEdition(id, docs)` applying `extends` (refuses an unknown id, a
  missing parent, a loop), `checkEditionShape` (loader check 1; checks 2 to 6 are AT3). An edition with no `extends`
  starts from the Survey atlas.
- `editions/survey.edition.json`: the Survey atlas, the default, with the shipped templates for sector, system, star,
  world, moon, station and other (design 4.3), its section and column words and its gap words.
- `catalogue.ts`: the fact catalogue (which fields an edition may name) and how each numeric field is said: a figure
  kind and what it is about, handed to `say()` in `figures.ts`. `showFact` gives known, band or gap.
- `page.ts`: `contents(scene)` (the book in reading order; only subjects in the scene; a subject whose parent the
  audience does not know sits under its nearest known ancestor), `chapterOf`, `buildPage(scene, address, edition,
  {vocab, depth})` giving blocks `title`, `prose`, `facts`, `table`, `plate`, `story`.

## Rules it keeps
- Pure: no clock, no randomness, no reads beyond its inputs. The same scene, edition and options give the same JSON.
- **Gaps from the template, never from the value.** `sse-observe` writes a placeholder only for a fact that exists
  (`crates/sse-observe/src/scene.rs`, the `Placeholder` branch iterates the state's own facts). So the Atlas never
  reads a placeholder: a template line the audience lacks is a gap whether the scene carries a placeholder or nothing,
  and the test strips every placeholder and gets byte-identical pages, in plain and bar gap styles.
- On the GM's own scene (`audience: "gm"`) a line with nothing behind it does not apply and is left out.
- No figure is formatted in the Atlas: numbers go through `say()`; bands are said with the low end on the high end's
  rung. Counts (`settlements.count`) are whole numbers with no unit. An uncatalogued number would fall back to the
  vocabulary's reading; the test holds that no shipped template names one.

## Known limits (for later packages)
- Until the kernel's SV1 field states reach the scene, a player cannot be told "does not apply" from "not known": a
  world's air lines are gaps on an airless moon too. The moon template is kept short for that reason.
- Plates are a block naming the subject and its children; drawing them is AT2.
- Letterhead, margin notes, sources, stamps: AT6, AT7, AT8.

## Checks
`node --test packages/ui-contract/test/atlas.test.ts` (10 tests): contents and chapters; determinism; a world page's
figures and gaps; a seat's blocks hold nothing its scene lacks; placeholder and absence read the same; the GM's page
leaves out lines that do not apply; a heard-of subject; the system page; editions extend, override words without
changing figures, and are refused with reasons; every template field is catalogued.

## Integration points
None outside the track yet. AT2 draws these blocks.

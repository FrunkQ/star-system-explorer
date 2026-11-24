# NM6 part 4: the class gallery (Classes page in Reference)

Status: done, 24 Nov 2026 (thread "Build names gallery and life board"). V3 gaps item 35; Alex's ruling: the gallery
lives in Reference, uses SSE4's own imagery, and a class is a label read off results, never a gate.

## What it does

- Reference, group "Names and classes", page **Classes** (`views/reference/src/Classes.svelte`, pure parts in
  `ui-contract/src/classes.ts`).
- Reads every class atlas on the stack (`provides.other` with `schemes`, `axes`, `classes`: C1 and C2's format). A
  later pack's class or scheme of the same id replaces the earlier one.
- With no atlas on the stack, it shows the shipped atlases from the screen's `classes_offer` addresses
  (`packs/base-atlas.json, packs/sse3-classic.json`, copied by `scripts/build-shell.sh`), each apart, SSE4's own first.
  On the Build desk it offers "Add <atlas>" (stack_packs, as the Names page does for names).
- Each scheme is listed by its question ("What kind of object is this?"). Its base classes are tiles, each under its
  parent ("a kind of gas giant", dashed); modifiers ("Marks that add to a class") are chips.
- A tile's picture is the mark the charts draw for that class under the screen's symbology, so it always matches the
  chart. The shipped symbologies bind SSE 3 classic ids, so SSE4 atlas classes draw as plain marks for now.
- A class opens to a card: plain name, physics name (Science), story name, the summary, "A world is X when" with each
  band in words and units and the fringe it fades out over (Science), what it needs beyond numbers, the scheme's gate
  (Science), worlds like this, near misses (Science), kinds of it, ids and pack (Engine). Escape closes and gives
  focus back.
- Find a class: every word typed must appear in the name, id, summary or example worlds; a plural finds its singular.
- Reading writes nothing (e2e checks the state hash).

## Checks

- `ui-contract/test/classes.test.ts`: schemes, tree, bands in words, numbers, SSE 3 classic, find, later pack wins.
- `tests/ui/reference.e2e.mjs`: the Classes section (desk and phone): offered atlases, four schemes, tiles with
  marks, the ammonia clouds card, find "rings", near misses on shirt sleeve, SSE 3 classic tiles, no sideways scroll.
- Screens: project files `sse4/handover-screens/NM6-screens/classes-desk.png`, `classes-phone.png`.

## Next

- The still and turning pictures: the kernel's `class_sample {class}` (kernel status, queue item 6) gives a world mid
  band; the card then shows its globe (still) and the renderer's turning globe in True.
- Marks for SSE4 atlas classes: a symbology binding per atlas id (Symbols screen data), owner of the symbologies.
- Gas giant colours keep V3's rainbow range from cloud chemistry (ND2) once the renderer draws giants' clouds.

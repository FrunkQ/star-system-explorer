# ED0: pairs on the Solar System (part 1, the Orbit box)

Status: part 1 done 25 Nov 2026 (thread "Build planet and star editing"), after kernel BC1 and K13 (PR 468: barycentre always, the big moons and Charon in base-sol). Design: `handover/design/SSE4-editing-design.md` section 10A ("Editing a pair: two orbits, and the centre stays hidden").

## Part 1: a pair's two orbits in the Orbit box

- `pairOf(facts, address)` in `packages/ui-contract/src/box.ts` reads the pair off the kernel's facts. A body's heaviest satellite makes a pair with it when their common centre (a × m / (M + m)) lies outside the heavier one's surface, a real boundary rather than a typed threshold (decision 2). The same pair comes back from either member. Results: Pluto and Charon are a pair (about 2,100 km out, 1.8 Pluto radii). Earth and the Moon are not (the centre is inside the Earth). Jupiter and the Sun just are.
- In the Orbit box, on either member's card, a line says where the hidden centre is: "Pluto and Charon go round a point between them 2,710 km from Pluto's centre, 2.3 times its radius: outside Pluto, so they are drawn as a pair." The centre is words, never a thing to pick (decision 3).
- **Round each other**: on the heavier member's card (Pluto), a strip for the lighter one's orbit (Charon's size). It writes to Charon, as one change with one Undo, and the card's "What followed" is still Pluto's.
- **Together round X**: on the lighter member's card (Charon), a strip for the heavier one's orbit round its host (Pluto round the Sun). A top-level pair (the Sun with Jupiter) has no second part.
- `Handles.commit(h, si, how, shown?, at?)`: `at` writes a pair part to the other member.

## Still to do in ED0

- The chart drawing each pair round its centre, with a faint cross (renderer thread), and "Seen from: the centre".
- Pairs in the object selector as a bracketed tree, with rows that drag to re-parent (selector thread).
- The checked list of structural edits (add a wide star, add a close star, promote Jupiter, remove Charon, remove Alpha Centauri B, drag a world from A to B, cut and paste the Pluto pair) as UI e2e against the three starter systems.
- The walk step "Open Pluto: watch both circle the point between them".

Checks: `packages/ui-contract/test/box.test.ts` (Pluto and Charon from either card, not Nix; Earth and the Moon not; the Sun and Jupiter just). `tests/ui/editing.e2e.mjs` (Pluto's box says the centre is outside Pluto; Round each other typed to 25,000 km writes it, and one Undo). Screen in project files `sse4/handover-screens/ED7-screens/pair-pluto-desk.png`.

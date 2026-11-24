# NV2: The selector, part 2 (V3 gap 37, with the find of gap 41)

Status: part 1 done 24 Nov 2026 (thread "Build object selector and menus"). The selector's first cut is the chart room's (PR 399: `ui-contract/src/picker.ts`, `chart-grammar/src/Picker.svelte`); from here it is this track's.

## Part 1 (this pull request): no shell files touched, while the chart room splits App.svelte
- **Find by tag.** Typing "#" and a tag searches tags instead of names (`matchesQuery`). It matches the start of a tag or of any part after ":" or "/", ignoring case: "#ocean" finds Tarn (tagged `class:planet/ocean`), "#size:small" finds the small worlds. `subjectItems` carries each subject's tags. Tags stay labels only.
- **Search across the campaign.** `elsewhereRows` and the Picker's `elsewhere` prop: while typing, places elsewhere in the campaign are listed after this level's hits under "Elsewhere in the campaign", each saying where it is ("in Sol"), this level's own left out, at most 50. The shell does not pass them yet: that wiring is part 2, in `shell/src/nav/` once the split lands.
- **The object menu on the selector's rows.** Each pickable row carries `data-address`, so a right-click or a held press on it opens the same menu as on the chart (NV1). Rows kept only as context do not.

Checks: `packages/ui-contract/test/picker.test.ts` (tags, elsewhere, subject tags); `tests/ui/object-menu.e2e.mjs` (finding "#ocean" in the picker on Kestrel; the menu from a picker row); `picker.e2e` and `place.e2e` still pass.

## Part 2 (next, after the split): in `shell/src/nav/`
- Pass `elsewhere`: the campaign's systems, and the bodies of systems already read this session.
- A row's › makes it the current place; a plain press selects it on the chart without leaving the level.
- Every "which one?" opens the selector: Voyages' destination, Add here's host, Send this view, Measure, Copy to.

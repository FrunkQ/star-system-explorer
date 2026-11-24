# Status: Editing planets, stars and systems (track ED)

Owner thread: "Build planet and star editing". Covers: ED0 to ED13 and ED0b, from `handover/design/SSE4-editing-design.md` (accepted by Alex 24 Nov 2026 15:21; the package list is its section 15). Kernel asks K1 to K17 are the kernel thread's ("Build kernel resolve and rules"), ordered with this thread on 24 Nov: K13 (pairs) first, then K1 sweep, K2 and K3, K17, K15, K16, K4, K12, K5, K7 and K8, K9, K10, K11 and K14.

Only the owner thread edits this file. Logs are in `handover/packages/`. Index and standing rules: `handover/START-HERE.md`.

| Package | State | Log |
|---|---|---|
| ED1a part 1 the units engine: `say` and `readFigure` in `ui-contract/src/figures.ts` over the "How figures are said" table (`figures-said.json`), every factor from the sheet's Units table; readings a click steps through, remembered per kind and scale class (`withChoice`, `chosenReading`) | done | packages/ED1a.md |
| ED1a part 2 every existing figure onto `say`; click a unit on screen; choices saved with the campaign; "How figures are said" on Reference; the editing-grammar check that fails a screen formatting its own numbers | next | |
| ED0 the Solar System with pairs (Pluto and Charon, Earth and Moon, the Sun's wobble) and the checked list of structural edits; family tree in the ghost's panel and the selector | waits on kernel BC1 and the rest of K13 | |
| ED1 the handle sheet, the handles row and the strip on the card, exact answers on pause | queued (K2, K3) | |
| ED2 the sweep behind the strip: tipping points, pencil figures, "what followed" | queued (K1) | |
| ED3 dragging on the chart: orbit size, shape, readout, magnetism, cancel | queued | |
| ED4 to ED13, ED10a, ED13a | queued, in the design's order | |
| ED0b the Solar System experiment for real, against `packs/base-sol`, log in project files | after ED1 to ED13 | |

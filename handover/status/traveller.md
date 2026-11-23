# Status: Traveller pack

Owner thread: "Build Traveller pack". Covers: X3a to X3d, TG8, TG9, TG11, TG13, MP16.

Only the owner thread edits this file. One row per package: update it in the same pull request as the package. Logs are in `handover/packages/`. Index and standing rules: `handover/START-HERE.md`. Stage order: `handover/design/SSE4-coming-together.md`.

| Package | State | Log |
|---|---|---|
| X3a: code sheets (`sse-codes`) and the Traveller pack's world codes (UWP, PBG, trade codes, zones, bases) from classification rules; units, Imperial calendar (`ui-contract/calendar.ts`), words; kernel op `codes`; world data sheet view (`sse.view.world-codes`) | done (Traveller pack track); lens, reports and sifters next | packages/X3a.md, packages/X3a-sheet.md |
| X3b part 1: a whole system from a coded world (pins to a formation request, codes win, impossible values marked with reasons, make physically sound, counts searched) | done (Traveller pack track); kernel one-shot op and moons next | packages/X3b.md |
| X3c: hex map view (`sse.view.hex-map`); sector formats (`sse-sector-io`); import through the code sheet (kernel op `import_sector`, preview or authored events); Travellermap fetch through the site Worker (`/tmap/*`) | done (Traveller pack track); GM screens wait on the screen hook | packages/X3c.md, packages/X3c-io.md |
| X3d parts 1 to 3: gas mixes and what to wear outside; sensors, range bands and the knowledge ladder as triggers and grants (to each ship's own crew); stock ships, stations and sites as J7 templates | done (Traveller pack track); stock ships (J7) and jump-era physics (J6) next | packages/X3d.md |

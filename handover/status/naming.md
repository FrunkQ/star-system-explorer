# Status: Naming

Owner: the thread that owns track NM. Covers: NM.

Only the owner thread edits this file. One row per package: update it in the same pull request as the package. Logs are in `handover/packages/`. Index and standing rules: `handover/START-HERE.md`. Stage order: `handover/design/SSE4-coming-together.md`.

| Package | State | Log |
|---|---|---|
| NM0 and NM3a naming (`design/SSE4-naming-design.md`): sources and licences checked (`docs/citations/naming.md`), clash odds; `packs/base-names` from `scripts/build-base-names.py` with the Catalogue scheme in both forms (Roman numerals by orbit, letters from b), IAU descriptor terms for surface features, and the avoid list | done (V3 naming row written 22 Nov); NM3b and NM3d below | packages/NM0.md, packages/NM3a.md |
| NM3b and NM3d naming: the English-sounding scheme (place-name elements with their meanings on the card, craters after long-dead British astronomers and explorers) and the two invented languages Vesh and Oluen read from their name sheets in `packs/base-names/sheets/` | done | packages/NM3b.md, packages/NM3d.md |
| NM3c naming: Many nations, ten real languages from NorthEuraLex and WOLD (CC BY 4.0, `scripts/fetch-name-sources.py` into `packs/base-names/sources/`), romanised by stated rules, rows left out on review with reasons, mixed by system in three presets (even, by speakers, space agencies) | done; first pass, no speaker review yet | packages/NM3c.md |
| NM naming (`design/SSE4-naming-design.md`, accepted for building 22 Nov): name schemes as pack data (List, Sounds, Number, Pattern, Mix makers), keyed draws on their own stream, names that follow a renamed parent, sibling and avoid checks, text safety; then name layers, settings, the base-names pack and the call sites | NM1, the engine (crate `sse-names`, kernel op `name_sample`, `packages/NM1.md`), and NM2, names on real worlds (layers and settings as facts, `read_names`, `set_naming`, `rename`, `fix_names`, seats see the layers their view discloses, `packages/NM2.md`), done; next NM4's call sites (after NM3a) and NM5; NM3a to NM3d go to the data sheet thread, NM3d's two languages from the design thread | design section 7 |

# FT1: base-tech pack and strength for weight

Track FT (future tech), design `handover/design/SSE4-future-tech-design.md` (accepted 23 Nov 2026, Alex "yes to all"). Owner: the master data sheet thread.

## Part 0: citation check of the design note (23 Nov)

The note's figures and sources were checked before anything ships from it. Results are in the note's section 14 (source table with DOIs and read status, figure table). Corrections made in the note: McKendree 1996 (not 1995), nanotube habitat about 1,000 km; lunar elevator cites Pearson, Levin, Oldson and Wykes 2005 (NIAC); the Mars elevator is computed, not cited; the radiator figure states its assumption (two faces at emissivity 0.9); spin comfort is Globus and Hall 2017 (4 rpm, 6 trained); the Shkadov thrust is L(1 − cos θ)/2c. Still open: Sánchez and Scheeres 2014's 25 Pa (not read, do not ship a number from it).

## Part 1: strength for weight on the master sheet (this PR)

Strength for weight (limit 7) is tensile strength over density, read through Look up, so the sheet carries both for the materials the ladder names. Four mixtures, by mass, empty recipe:

| Row | tensile_strength | density | strength for weight | Source, status |
|---|---|---|---|---|
| steel-wire | 2.0 GPa | 7,850 kg/m³ | 0.25 MJ/kg | textbook, from memory: marked unverified on the sheet |
| aramid | 3.0 GPa | 1,440 | 2.1 MJ/kg | DuPont 2017 Kevlar technical guide Table II-1 (Kevlar 49 yarn), read; a resin strand reaches 3.6 GPa |
| pbo-fibre | 5.8 GPa | 1,560 | 3.7 MJ/kg | Fiber-Line Zylon datasheet (HM: 42.0 g/d, specific gravity 1.56), read; 5.8 GPa is worked out and matches Toyobo |
| carbon-nanotube | 63 GPa | 1,340 | 47 MJ/kg | Yu et al. 2000, citation confirmed, abstract not read; density as usually quoted, not read |

No goldens change: nothing reads the new rows yet. `sse_core::sheet::mixture_density` gains the four densities (generated).


Atomic Rockets (projectrho.com, pointed out by Alex 23 Nov) is used only as a map to primary sources; nothing is cited from it or copied.

## Part 2: the base-tech pack (next)

`packs/base-tech`: the ten limits, the figures each holds back with the field a block reads, cited marks (built, allowed, ceiling), the neutral ladder and its two size factors, the real advances. Built by a script that reads the sheet (SHEET_FED in the MP22 audit), `--check` in check.sh.

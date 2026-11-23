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

## Part 2: the base-tech pack

`packs/base-tech`, built by `scripts/build-base-tech.py` (SHEET_FED in the MP22 audit; `--check` in check.sh; manifest test in sse-pack). Under `provides.other.tech`:

- **limits**: the ten of design section 4, each with its Science line, what it shapes, what lies past its ceiling, and its figures. Limit 6 (light speed) has no figure: FTL stays a per-setting rule (voyages design).
- **figures** (17): id, limit, unit, `better` (the way an advance pushes it), `reads` (the field a sizing block reads today, or "FT2" where the kernel adds the reader) and cited **marks**: built, allowed, ceiling. A ceiling always names its law. Fiction has a figure with a ceiling at zero where no partial version exists: heat sent nowhere, inertia screened, weight without spin, light hidden.
- **ladder**: the neutral ladder, levels 0 (muscle and wind) to 6 (deep); Sol today is level 4. Ten times the size per level short; half per level beyond, for at most two levels (Alex, question 4).
- **advances** (12, all real or not yet built, none fictional): what each sets, its home level, its part, and the mark it earns, worked out by the builder from the figure's marks (past a ceiling fictional, within the best built value built, otherwise not yet built).

Worked out from the sheet, never typed: the exhaust ceiling and photon thrust from c; exhaust speeds from Isp times g0; hydrogen-oxygen energy from water's heat of formation and atomic weights; D-He3 energy from e and m_u; strength for weight from the four materials of part 1; annihilation from c squared.

Sources read for the marks (23 Nov): L3Harris RL10 sheet (465.5 s); NASEM 2021 Table 2.1 (Pewee about 875 s); Williams et al. 2005 (D-He3, 35,435 s, 8.62 kW/kg; partly read); Adams et al. 2003 (77,000 s); Ball 2019 (3.7 MeV per nucleon); OpenStax University Physics 3 (8.21e13 J/kg); Pattarini et al. 2020 (seated head-to-foot 4.91 m/s2 past 2 minutes); Globus and Hall 2017; ISS ATCS overview (ammonia 2.8 C); NASA-STD-3001 V1 rev C (600 mSv); OCHMO-TB-020 (20 g/cm2); Koontz 2016 (40 to 50 g/cm2); Lynch et al. 2023 (0.89, 2.79 and 2.39 kg a day); NASA 2023 (98% water); Lubin 2016 (3.3 nN/W).

Not read, marked so: NASA-STD-3001 V2 acceleration tables (chest-to-back limits), the BVAD tables, Santarius and Logan 1998's full text (behind the 1,500 K radiator and 0.1 waste share carried from base-ships), the ideal nanotube shell (Peng 2008, the 77 MJ/kg bond ceiling), steel wire.

Next (FT2, kernel): advances on a civilisation, inherited and dated; needs computed from parts; the size rule; the availability reading; fictional by ceiling.

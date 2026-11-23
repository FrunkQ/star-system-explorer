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
- **ladder**: the neutral ladder, levels 0 (muscle and wind) to 6 (deep); Sol today is level 4. (Part 1 had ten times the size per level short for everything; part 3 replaces it.)
- **advances** (12, all real or not yet built, none fictional): what each sets, its home level, its part, and the mark it earns, worked out by the builder from the figure's marks (past a ceiling fictional, within the best built value built, otherwise not yet built).

Worked out from the sheet, never typed: the exhaust ceiling and photon thrust from c; exhaust speeds from Isp times g0; hydrogen-oxygen energy from water's heat of formation and atomic weights; D-He3 energy from e and m_u; strength for weight from the four materials of part 1; annihilation from c squared.

Sources read for the marks (23 Nov): L3Harris RL10 sheet (465.5 s); NASEM 2021 Table 2.1 (Pewee about 875 s); Williams et al. 2005 (D-He3, 35,435 s, 8.62 kW/kg; partly read); Adams et al. 2003 (77,000 s); Ball 2019 (3.7 MeV per nucleon); OpenStax University Physics 3 (8.21e13 J/kg); Pattarini et al. 2020 (seated head-to-foot 4.91 m/s2 past 2 minutes); Globus and Hall 2017; ISS ATCS overview (ammonia 2.8 C); NASA-STD-3001 V1 rev C (600 mSv); OCHMO-TB-020 (20 g/cm2); Koontz 2016 (40 to 50 g/cm2); Lynch et al. 2023 (0.89, 2.79 and 2.39 kg a day); NASA 2023 (98% water); Lubin 2016 (3.3 nN/W).

Not read, marked so: NASA-STD-3001 V2 acceleration tables (chest-to-back limits), the BVAD tables, Santarius and Logan 1998's full text (behind the 1,500 K radiator and 0.1 waste share carried from base-ships), the ideal nanotube shell (Peng 2008, the 77 MJ/kg bond ceiling), steel wire.

## Part 3: the size rule by part family (design 5.2, accepted by Alex 23 Nov 19:20)

Alex: the ten times per level was a rule of thumb to improve on. Immaturity now acts on the advance's own figure and the part's size falls out of the sizing blocks. Fields, under `provides.other.tech`:

- `families[]`: `id`, `name`, `about`, `src`, and `maturity` with `rule`:
  - `worse_by` with `factor`: the figure is divided by `factor` per level short (multiplied where the figure's `better` is "lower"). Engines and power plants 7, bulk materials 5, energy stores 3.
  - `toward` with `share`: one level short, the figure is `share` (2/3) of the way from the civilisation's older figure for the same job to the mature one. Near-ceiling figures.
  - `same`: plain mass.
  - `floor`: devices with a least size; the advance states it.
  - `size` with `size_per_level_short` 10, `shrink_per_level_beyond` 0.5, `shrink_levels` 2: fictional and unknown devices only; the figure stays and the part's mass and volume scale.
- `advances[].family`: the family id. `advances[].fixed`: figures physics fixes, which maturity never moves (D-He3's energy per kg, photon thrust per watt, spin comfort, a chemical drive's zero waste heat).
- `ladder.improve_per_level_beyond` 1.5: past the home level the figure gets half as much better again per level until it meets its next cited allowed or ceiling mark (replaces "half the size for two levels" for real advances).

The old `ladder.size_per_level_short`, `shrink_per_level_beyond` and `shrink_levels` stay for now, because FT2's first reader (PR 304) reads them; the kernel removes them when its reader moves to the families.

A prototype is never worse than the best older advance the civilisation holds for the same job (design 5.2); that floor is the kernel's reading, not pack data. The fission reactor now sets power per kg (SNAP-10A, about 1.2 W/kg, DOE) rather than fuel energy, since fission energy per kg never matures; SP-100's design (about 33 W/kg, JPL) is an allowed mark.

Next (FT2, kernel): advances on a civilisation, inherited and dated; needs computed from parts; the size rule; the availability reading; fictional by ceiling.

## Part 4: keeping the books (design section 15, accepted by Alex 23 Nov 20:14)

Fields added under `provides.other.tech`:

- `balances[]` (design 15.1): `id`, `name`, `relation` (in figure ids and the design's own quantities: thrust_n, mdot_kg_s, jet_w, plant_w, waste_w, stored_w, loads_w, q_w, t_hot_k, t_cold_k, pump_w), `figures`, `law`, `catches`, `src`. Five: energy-pays-exhaust (with a `near_c` relativistic form), jet-power, energy-in-out, momentum, heat-downhill. A check that does not close is fictional by arithmetic, like a figure past its ceiling.
- Marks may carry `for` (the one fuel or case a computed ceiling holds for) and `computed_from` (`balance` plus the figure values it used). `exhaust_km_s` now has a ceiling for hydrogen-oxygen of 5.18 km/s, √(2 × 13.4 MJ/kg): water's heat of formation less its latent heat of vaporisation, from the sheet (the lower heating value; the design's 13.3 was rounded from another table). RL10B-2 reaches 88% of it. A mark with `for` never makes another fuel's advance fictional.
- `waste_heat_k` (new figure, limit heat): electronics 350 K and quarters 300 K (authored, design 15), fusion drive structure 1500 K. Every advance with waste_per_jet above zero must set it (validated); the fusion torch does.
- `fictional_where[]`: light, heat, momentum, energy. Any fictional advance (none in the base pack) must give all four in `advances[].where`, "nowhere" allowed (validated).
- Figures are stored as quantities with units; the builder refuses a unit of "x", "factor" or "times".
- Family `src` is now "fudge-register": the maturity defaults are rules of thumb from a small sample, rowed in docs/fudges.md (Future tech); their examples stay cited in `examples_src` "ft-design-5.2".
- `real_craft[]` (design 15.6): thirteen real craft held by hand until a real spaceflight pack supplies them, each with `quantities` (SI, with units; converted from the source's units by the builder), `tests` (the balance or sizing it checks) and `src`: ISS heat rejection, the three Saturn V stages and the stack, Falcon 9, MMRTG, MHW-RTG, GPHS-RTG, Juno's arrays, Shuttle entry, the Apollo heat shield, the ISS's volumes. Sources read 23 Nov (NASA, MSFC, Rocketdyne, JPL, SpaceX; no secondary summaries). Gaps, said on the rows: F-1 vacuum and J-2 sea-level Isp, Falcon 9 stage masses, whether the ISS radiators radiate from both faces (planform 475 m² worked out from the unit sizes).

Citation check of section 15's own figures (design note 14.2): 5.2 km/s confirmed (5.18 from the sheet); 121 m² for 100 kW at 300 K confirmed (one face, emissivity 1).

Next (FT2, kernel): read the balances on a design card; the real-hardware test over `real_craft`.

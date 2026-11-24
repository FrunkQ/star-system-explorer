# RS2: real parts (track RS)

Owner: "Build real spaceflight history pack". Design: `handover/design/SSE4-real-spaceflight-feasibility.md` sections 4 and 5. Field names: `handover/design/SSE4-atomic-rockets-plan.md` section 11.3, confirmed by the travel thread.

## Part 1 (24 Nov 2026): launch vehicles, stages and engines from GCAT

- `scripts/spaceflight/build_parts.py` writes `store-packs/sol-today/parts.json` from GCAT's engines, stages and launch-vehicle stage tables (CC BY 4.0). It covers:
  - 5 launch vehicles: Saturn V, Space Shuttle, Falcon 9 FT5, Titan IIIE and the A-4;
  - 13 stages;
  - 17 engines, including those on the craft themselves: Apollo's SPS, the Shuttle's OMS, Voyager's kick motor, Cassini's R-4D-11, Juno's Leros 1b and Dawn's NSTAR-30.
- Exhaust speed is GCAT's Isp times g0, with g0 read from the master data sheet. No constant is typed in the script.
- Propellants are named by sheet substance ids. RP-1 is on the sheet as a mixture (`rp-1`, PR 348). Two solids are not yet: `pban-ap-al` (the Shuttle SRB's propellant) and `htpb-ap-al` (GCAT's generic "Solid"); the data sheet thread is adding both under those ids.
- What GCAT does not give is listed per engine under `to_read`:
  - engine mass;
  - sea-level thrust;
  - whether its single thrust and Isp are sea-level or vacuum figures (its F-1 row is near the sea-level value and its J-2 row the vacuum one);
  - for NSTAR, its electric input, thrust and efficiency.

  Engines GCAT lacks entirely are listed per craft: the LM descent and ascent engines, Curiosity's MLE, SNAP-10A's ion unit and reactor, JWST's thrusters and Zvezda's KTDU-80. Nothing is guessed.
- GCAT books the Shuttle's SSMEs on its ET stage. The construct record will put them on the orbiter.

Next (part 2): read the `to_read` figures from primary sources: NASA SP and TM documents, press kits and fact sheets for engines, power sources, heat shields and instruments. Also read the future-tech test figures for each of the twelve craft.

## Part 2, first batch (24 Nov 2026): craft from NSSDCA

- `store-packs/sol-today/craft.json` has 7 craft: Voyager 2, Apollo 11's Columbia and Eagle, Dawn, Juno, Curiosity and JWST. Each figure is quoted from that craft's NSSDCA page in `as_written`, in the field names of RK 11.3.
- Figures the future-tech test asked for:
  - Voyager's RTGs: 470 W at launch and 335 W in 1997, so the output halves in about 39.7 years, faster than plutonium-238 alone because the thermocouples age.
  - Dawn's arrays: 10.3 kW at 1 au from 38.2 m², and 1.3 kW at 3 au at end of life.
  - Dawn's NSTAR: 92 mN at 2.6 kW input and 3,200 s, a jet power efficiency of 55.5% (the future-tech thread expected about 60%).
  - Juno's arrays: 18 kW at Earth and 400 W at Jupiter.
  - Curiosity's MMRTG: 125 W.
  - The LM's stage masses, and its engines at about 45 kN and 15 kN.
  - Apollo's CM heat shield thickness and parachutes.
  - JWST's 6.6 m aperture and roughly 10 by 22 m sunshield.
- Cassini, SNAP-10A and ISS (their NSSDCA pages gave errors) and STS-1, Falcon 9 and the V-2 went to the second batch, below.

## Part 2, second batch (24 Nov 2026): craft from their own sources

- `craft.json` now holds 13 craft. The six new ones each name their source in `src` and `src_note`; the register rows are in `docs/citations/spaceflight.md`.
  - Cassini-Huygens, from JPL's 1997 launch press kit: 5,712 kg at launch with 3,132 kg of propellant; two 445 N main engines on MMH and NTO; three RTGs giving 885 W, 675 W planned at Saturn and 633 W at the end of the mission; the four flybys and the 94-minute Saturn orbit insertion.
  - SNAP-10A, from the 1964 Atomics International design summary: 39.5 kW thermal and 525 W electric, so about 1.3% net. The flight figures (43 days, 530 to 600 W, 5.8 m² radiator, the cesium ion engine at 400 W and 8.5 mN) are from a secondary account and marked `secondary`.
  - ISS radiators, from NASA's ATCS overview: 70 kW from six 23.3 by 3.4 m radiators on ammonia, and four 14 kW photovoltaic radiators. This is the future-tech thread's radiator test.
  - STS-1, from its press kit: the five OMS burns (50.3, 41.8, 11.1, 11.4 m/s and the 91.1 m/s deorbit burn), ascent events, entry interface and landing. These are the RS3 log's big burns.
  - Falcon 9, from the 2015 user's guide: nine Merlin 1D at 756 kN each, MVac at 934 kN, landing legs and grid fins. SpaceX does not publish Isp or masses there.
  - V-2, from the USAF museum and designation-systems.net, both secondary: 249 kN and 130 kg/s of propellant, so an exhaust speed of about 1.91 km/s (derived).
- Still to read: Shuttle hardware (orbiter masses, SSME, SRB, OMS, fuel cells, radiators, tiles and peak heating), Cassini's Isp and burn delta-v, ISS power, mass, life support and shielding, Falcon 9 Isp and masses, a primary V-2 report, Apollo CM and Curiosity entry heating, and the engine masses in parts.json. Next is RS3, the flight logs, which uses the STS-1 and Cassini burns read here.

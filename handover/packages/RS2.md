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

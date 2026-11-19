# P1 addition: heat-producing elements and reference links (18 Nov 2026)

Asked by Alex: heat-producing elements (K-40, U, Th, Al-26) shape long-term geology and climate; build them as
real, explained, cited blocks people can see; and link references to the papers.

## What was built
- `crates/sse-atoms/src/heat.rs`, six atoms (37 in the library now):
  - `sse.heat.radiogenic_heating`: W/kg of silicate at a body's age. Today's bulk silicate Earth (U 20.3 ppb,
    Th 79.5 ppb, K 240 ppm; McDonough and Sun 1995) wound back 4.567 Gyr per isotope, scaled by
    `inheritance.k40_rel` (K) and `inheritance.r_process_rel` (U, Th), decayed to `body.age`. Isotope heat and
    half-lives from Turcotte and Schubert 2014 table 4.2. Earth today 4.91e-12 W/kg.
  - `sse.heat.mantle_mass` (M(1 − f_core)), `sse.heat.radiogenic_power` (Earth 19.8 TW; KamLAND 2011 about 20),
    `sse.heat.surface_heat_flow` (Earth 0.039 W/m² radiogenic).
  - `sse.heat.active_geology_lifetime`: age where radiogenic heat flow falls to `q_crit` (default 0.019 W/m²,
    half of Earth's today; a calibration, editable), by 80 bisection steps on 0 to 100 Gyr. Earth 10.3 Gyr,
    Mars 3.9 (ends about 0.7 Gyr ago), Moon 1.9, Earth with twice the r-process 19.4 (Unterborn et al. 2015).
    Radiogenic only: primordial heat, tides and lid style are left out and said so in the explain text.
  - `sse.heat.al26_planetesimal_heating`: Al-26 energy left after a planetesimal forms (adiabatic bound,
    H0 = 0.00865 × 5.25e-5 × 0.3583 W/kg, τ = 1.034 Myr; 5.31 MJ/kg at CAI time) and the share of water kept,
    log-linear between 0.6 and 1.8 MJ/kg down to 0.1 (params). Grimm and McSween 1993, Lichtenberg et al. 2019.
- Flows `packs/base-physics/flows/planet-heat.json` (hook `planet.heat`; Earth, Mars, Moon, thorium-rich Earth)
  and `planetesimal-water.json` (hook `planetesimal.water`; early iron parent body dry, late chondrite wet,
  no-Al-26 star wet). `pack.json` and the kernel fixture copy re-blessed.
- New kinds described in `KIND_DESCRIPTIONS`; dims `SPECIFIC_POWER`, `SPECIFIC_ENERGY` in `common::dims`.

## Reference links
- `sse_flow::Reference { text, doi?, url? }` replaces `String` in `AtomDecl.references` and
  `Explanation.references`. JSON is `{ "text", "doi"?, "url"? }`, the shape the UI track's "Why?" panel reads
  (PR 2, 723e983). A bare string still deserialises. Atom sources write `"<text>. doi:<doi>"` or
  `"<text> https://..."`; `Reference::parse` splits them and gives a DOI its `https://doi.org/` link.
- Every existing atom backfilled. DOIs are given where known; journal papers whose DOI was not certain get an
  ADS link built from the bibcode instead. This container cannot reach doi.org, so the links are unverified
  here: a spot check from a browser is worth doing.
- Test `every_atom_is_cited_explained_and_versioned` now needs at least one linked reference per atom, with
  one listed exception (`sse.ocean.brine_freezing_point`, textbook sources only). DOI and link shapes are checked.

## Goldens
- `tests/golden/contract.jsonl` re-blessed on purpose: the base-physics stack hash and manifest ids move
  (two new flows), and `references` in atom and explanation responses are now objects. No formation golden moved.

## Not yet
- Formation does not publish `body.age`, `interior.core_mass_fraction` or the inheritance rels to a world's
  facts, so the heat flow is not yet run per world in a pipeline; F7 or the integration step wires it.
- F3/F4 water does not yet read Al-26 (`planetesimal.water_kept_fraction`); same step.

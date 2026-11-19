# CITE: citation and constant check (track started 19 Nov 2026)

Goal: every reference, published relation and quoted constant checked against its source; registers in
`docs/citations.md` (per area under `docs/citations/`) and `docs/fudges.md`. Thread "Verify physics citations".

Method: sources read through web search and fetch (curl to doi.org, crossref, arXiv and ADS is blocked in the
cloud container). A shared fetch limit of about 100 pages an hour cut the first pass short; rows not read are
marked unverified, never guessed. A second pass follows.

## Log

### Interiors and heat (19 Nov)
Physics changed, goldens re-blessed on purpose:
- K-40 share of potassium 1.17e-4 → 1.19e-4 (the old value was the atom fraction; the code needs mass fraction;
  Ruedas 2017 table 1, consistent with Turcotte and Schubert's 3.48e-9 W/kg for K).
- U-238 and U-235 shares 0.9928/0.0072 → 0.9929/0.00711 (mass fractions, same source).
- Al-26 decay heat 0.355 → 0.3583 W/kg (Ruedas 2017 table 2); CAI-time energy 5.26 → 5.31 MJ/kg (test updated).
- Zeng et al. 2016 rocky radius and its inverse use the paper's R⊕ = 6.371e6 m, not the equatorial 6.3781e6 m
  (atoms `sse.interior.rocky_radius`, `core_mass_fraction`, and the sse-place shoreline cut).
- Atom versions to 1.0.1: `sse.heat.radiogenic_heating`, `radiogenic_power` (text), `active_geology_lifetime`,
  `al26_planetesimal_heating`, `sse.interior.rocky_radius`, `core_mass_fraction`.
- Goldens moved: formation-pipeline (heat power, geology-end ages), surface, surface-impacts, surface-plates,
  surface-regimes, surface-wear (envelopes follow the heat), time-generated (ages), place (Zeng radius),
  contract.jsonl (formation_ran lines). Bulk silicate Earth heat +0.3%, rocky radii −0.11%.

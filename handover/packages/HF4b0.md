# HF4b.0: the melt design's numbers and blocks

Track HF (heat flow), owner thread "Fix low heat flow in worlds". Design: `handover/design/SSE4-melt-and-heat-design.md` (accepted by Alex 26 Nov 2026), sections 8.1 and 8.2. Status: `handover/status/heat-flow.md`.

## What landed

- **Sheet rows** (`scripts/melt_rows.py`, called by `scripts/build-base-reference.py`): `mantle-rock` gains `phases` (dry solidus fit, liquidus fit, deep liquidus, latent heat of fusion, the solidus rise of depleted rock), `rigidity`, `creep` (reference viscosity, temperature and activation energy, dry factor, melt weakening, Andrade exponent), `melt` (lock-up share, grain size, permeability law) and `thermal` (conductivity, diffusivity, heat capacity, expansivity); `basalt` gains solid and liquid density and liquid viscosity. Where the thermal history already typed a value, the row holds the same value.
- **Fourteen formula blocks** in the new `packs/base-physics/formulas/melt.json`, written by `scripts/build-melt-blocks.py` (worked examples computed there from the same equation, and run again by the loader):
  - melt: `sse.melt.fraction`, `supply`, `in_place`, `retained`, `softening`, `processed`, `erupted_share`, `heat_carried`;
  - lid: `sse.heat.lid_base_temperature`, `lid_growth`, `lid_conduction_with_burial`, `boundary_layer_speed`;
  - tides: `sse.tides.andrade_response` (HF4a's Maxwell block is its creep-off case, checked to 1e-12), `sse.tides.host_power_ceiling` (Io with Jupiter's k2/Q: 1.2e14 W).
- Rock numbers are block inputs, fed from the sheet rows by the packages that read them. A block's own parameter is only a setting the design names (a_rh 2.24, the melt-fraction exponent 1, the Andrade switch, the rigidity floor 0.05, the erupted share 0.2).
- Tests (sse-atoms): `andrade_response_is_maxwell_with_creep_off` (also absolute: a stiff Io at 1e19 Pa s has Maxwell Im k2 below 1e-5 and Andrade above 1e-4), `lid_conduction_without_burial_is_plain_conduction` (and the advected heat balance with burial), `melt_heat_split_adds_up`. The worked examples are the first gate: any change to a formula without its example goes red at load, and did while the Andrade equation was over the 64-operation limit.
- Atom count 183 to 197 (P1 log). Fudge register: the erupted share, the turnover length, the linear rigidity law.

## Not in this package, and why

- The two Rust blocks of 8.2 (`sse.melt.zone`, the 48-point sum, and `sse.tides.layered_response`) land with the packages that read them: HF4b.2 and HF4c.2.
- Moving `CoreSettings`' typed rock values (`k_mantle`, `cp_mantle`, `eta_ref`, the solidus fit, and the rest) to sheet references in the surface pack: the design lets that move to HF4b.1, which rewrites the march's reads anyway. HF4b.0 therefore changes no result: no core golden, reach lines only.
- The reference-object rows of 8.1 (Jupiter's and Saturn's k2/Q, measured Love numbers, measured heat loss) go in with HF4c.1, their first reader.

## Goldens

`tests/golden/contract-reach.txt`: reach lines for the new blocks only. No line of `contract.jsonl` or `surface.json` changes.

## Unverified

Every row and block source was written from memory on 26 and 27 Nov 2026 while the paper search was out (until 1 Dec 2026). All rows carry `"check": "unverified"`; the sources are on `handover/citations/master.jsonl` as unchecked.

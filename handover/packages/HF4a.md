# HF4a: one tidal response block

Thread "Fix low heat flow in worlds", 26 Nov 2026. Block `sse.tides.maxwell_response` in `packs/base-physics/formulas/tides.json`.

A world's own tidal response, worked out from its interior, so that no world shares another's k2/Q. It is the homogeneous incompressible Maxwell body (Henning, O'Connell and Sasselov 2009, from memory, **unverified**):

- A = 19 μ / (2 ρ g R), X = ω η / μ, D = 1 + (1 + A)² X²
- Im k2 = (3/2) A X / D (the k2/Q the eccentricity heating law takes)
- Re k2 = (3/2) (1 + (1 + A) X²) / D

Inputs: rigidity, viscosity, mean density, surface gravity, radius, and the tidal frequency (the mean motion for a locked moon). New kinds: `interior.rigidity`, `interior.viscosity`, `tides.frequency`, `tides.im_k2`, `tides.re_k2`.

Nothing reads the block yet. HF4c switches `sse.tides.eccentricity_heating` from its k2/Q constant to this block's Im k2, and replaces the surface classifier's own copy of the heating law (k2 0.3, Q 100 for every world) with a run of the cited block. The small-moon shape thread ("Build the asteroid and comet designer") reads the same block for Mimas, Enceladus and Miranda.

What the block cannot do alone: Io's measured dissipation needs a partly molten layer (low rigidity and viscosity), which the thermal history cannot yet hold (HF4b). A one-layer body has no ocean under its shell either (Enceladus, Europa).

Test `maxwell_response_limits`: the elastic limit, the fluid limit and the peak, plus Io's solid k2 of about 0.03 as an absolute check. Seen to go red with the A dropped from Im k2.

Goldens: one line added to `tests/golden/contract-reach.txt` (the new block in the reach report). Atom count 176 to 177.

# MP24: where each element goes (element inventory, part 1)

Accepted by Alex 23 Nov 2026 ("build them as stated, use blocks"); matter pack design sections 4.1 and 5. Owner: the master data sheet thread. MP7 (each star's elements) is the kernel thread's; MP6 (each body's inventory) and MP8 (deposits) are proven core's.

## What the sheet gains

On every element row that the sources cover (built by `scripts/element_affinity.py` from `handover/packages/MP-affinity-data/`):

| Key | What | Source | Rows |
|---|---|---|---|
| `condensation.tc50_k` | 50% condensation temperature from a gas of solar make-up at 1e-4 bar | Lodders 2003 Table 8 | 81 |
| `condensation.host_phase` | the host phase, as Lodders writes it | Lodders 2003 | 83 |
| `condensation.rides_in` | the formation material it rides in: formation-metal, -rock, -carbon, -water, or "gas" | worked out from the host (rule in the script; carbon follows chondrites' organics, not equilibrium methane ice; sulfides count with the metal, as formation does) | 83 |
| `ci_mass_fraction` | mass share in CI chondrites | McDonough and Sun 1995 | 76 |
| `earth_mass_fraction.core` | Earth's core | McDonough 2003 Table 4 | 38 |
| `earth_mass_fraction.silicate` | silicate Earth (pyrolite) | McDonough and Sun 1995 | 76 |
| `earth_mass_fraction.crust` | bulk continental crust | Rudnick and Gao 2003 | 73 |

The sheet stores cited concentrations only. How an element splits between core, mantle, crust, sea and air in another world is worked out by a block from these, weighted by how far that world separated (design section 5, step 4). No shares are typed.

Also new: `substances[h2o].phases.latent_fusion` (333.55 kJ/kg, CRC, unverified), so the cryoscopic constant R·T²·M/ΔH is worked out (1.86 K kg/mol); the constant `P_rot_sun` (25.38 days, NSSDCA, unverified). `sse_core::sheet` gains `element_ci_mass_fraction` and `latent_fusion` for the kernel's last three ledger lines.

## Follow-up for MP6 (proven core's ask, 23 Nov)

- `ci_mass_fraction` for H (1.97%) and O (45.9%) from Palme and O'Neill 2014 (one copy read): neither copy of McDonough and Sun 1995 carries them, so rock's oxygen can be normalised. Those two rows cite palme-oneill2014.
- `condensation.oxide` on the rock-forming elements (Si, Ti, Al, Cr, Fe, Mn, Mg, Ca, Na, K, P, Ni): the oxide each takes in rock, by the convention of rock analyses, as a substance id. Twelve oxide substances join the sheet with their formulas (sio2, tio2, al2o3, cr2o3, feo, mno, mgo, cao, na2o, k2o, p2o5, nio). The check refuses an oxide id with no substance row. A block binds rock's oxygen by stoichiometry, so water's oxygen stays with the water material.

## Honesty

Every value is marked unverified: the publishers' pages could not be read from this sandbox, so each table was taken from machine-readable reproductions and cross-checked against a second copy where one exists (the README in the data folder lists every URL read and every flag). Fixes made on reading: Earth's core cobalt printed "0.25 ppm" is 0.25 wt% (2500 ppm); core titanium left out (unit unsure); CI nitrogen and phosphorus differ by 1000 between the two copies and the per-cent-level values are kept. Lodders 2021 CI values were not found readable, so McDonough and Sun 1995 stands in.

Partition shares as single values are already in the fudge register's honest limits (design section 5).

Not in this PR: `origin.scatter_dex` (shape agreed with the kernel: `{at_feh: [-2, 0], dex: [x, y], src}`, missing reads as zero) waits for its own cited wave (Cayrel 2004, Bensby 2014).

No goldens change: nothing reads the new rows yet.

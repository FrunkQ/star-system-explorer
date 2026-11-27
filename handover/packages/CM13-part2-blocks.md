# CM13 part 2: tank and dish physics as kernel blocks (compound ships)

Track CM. Design: `handover/design/SSE4-compound-ships-design.md` sections 8A.4 and 8B.2 (accepted by Alex 27 Nov 2026 00:43 UTC; 14A wins). Built 27 Nov 2026 by the thread "Build the snap-together ship builder". Part 1 (the part format and the base palette) is its own PR.

## What it is

Four cited Equation blocks in `packs/base-physics/formulas/ship.json` (family `sse.ship`), each with a worked example the kernel checks, so the builder and the part editor size tanks and dishes by the physics rather than by numbers typed on a part:

- `sse.ship.tank_wall_mass`: m = φ p V ρ / σ, the thin-wall pressure vessel (Young and Budynas 2002, Roark's, chapter 13); φ is 1.5 for a sphere, 2 for a long cylinder. 1 m³ at 3 bar in aluminium (2810 kg/m³, 250 MPa allowed) as a sphere: 5.06 kg. Size does not enter, so stretching a tank raises its wall mass in proportion to what it holds.
- `sse.ship.split_surface`: S = A N^(1/3), the skin a load gains when split into N like tanks (Galileo 1638, first day). Eight tanks have twice the surface of one; insulation and boil-off follow the surface.
- `sse.ship.dish_gain`: G = η (π D / λ)² (Larson and Wertz 1999, SMAD, chapter 13). A 3.7 m dish at 3.57 cm, η 0.55: 47.7 dBi, close to a real outer-planet probe's high-gain dish.
- `sse.ship.link_rate`: R = P G_t G_r (λ / 4π d)² / (k_B T_s E_b/N₀), the Friis link (SMAD chapter 13), k_B from the sheet. 20 W from that dish to a 70 m class dish (gain 3e6) at 1e12 m, 25 K, E_b/N₀ 2.5: about 33 kbit/s.

Eighteen `ship.*` quantity kinds described in `sse-atoms` `KIND_DESCRIPTIONS`. All four follow a changed constant (`reach.rs`): they are closed forms, and the link reads k_B through the handle; `the_link_rate_follows_a_changed_boltzmann_constant` checks a doubled k_B halves the rate. The formula reader refuses a typed constant (seen: writing k_B as a number is refused at load), so the follow cannot be faked.

## Golden

Changes on purpose, in the core lane (coordinator's order): `tests/golden/contract-reach.txt` gains four `follows` lines, one per block; `packs/base-physics/pack.json` regenerated (`SSE_BLESS=1 cargo test -p sse-atoms`). `contract.jsonl` does not move: the contract runs on the frozen fixture `tests/fixtures/packs/base-physics.json`, which is left alone. No world's values change.

## What it does not do yet

- No op reads them yet: the builder (CM14) asks them for each tank and dish it shows, and the palette's generic tanks and dish name them under `to_read` once part 1 is on main.
- Boil-off itself (heat leak through the insulation, then latent heat) and insulation mass per area are not blocks yet; the split block gives the surface they scale with.
- Book addresses are Open Library ISBN pages, not checked from this container (no outbound access); the three new papers are in `handover/citations/papers/`, unchecked.

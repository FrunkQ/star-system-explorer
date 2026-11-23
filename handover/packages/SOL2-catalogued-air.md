# SOL2-chain: catalogued worlds bring their own air (proven core thread, 23 Nov)

**Why.** The test drive found Sol's Earth resolving at 255 K, "episodic lid", no
liquid: `bodies_from_facts` (sse-kernel `adopt.rs`) only knew V3's
`body.surface_pressure` and a pack-default albedo, and base-sol carries no air.

**What changed.** A world built from facts now uses, in order:

- its own `body.albedo` (Bond) for the equilibrium temperature, else the
  surface pack's `albedo_default`;
- `planet.surface_temperature` (K, the climate atoms' name) when catalogued,
  else equilibrium plus the pack greenhouse (only with oceans), as before;
- `atmosphere.surface_pressure` (Pa, the air atoms' name), else V3's
  `body.surface_pressure`.

Measured beats estimated; nothing is typed. A measured mean belongs to the
world it was measured on: typed by the GM it stands as typed; catalogued, it
is kept as a ratio to the equilibrium temperature of the world as catalogued
(the grey factor (1 + 3/4 tau)^(1/4), read from `catalogued_facts`, the fold of
catalogued events only). So a world the GM moves or re-whitens keeps its air's
warming but answers to its new sunlight: the stage 1 walk moves Mars to 1 AU
and sees it warm from 214 K to 264 K. The data sheet thread adds the
NSSDCA air, temperature and water facts to `packs/base-sol` under these keys.

**Tests.** `crates/sse-kernel/src/sol_tests.rs` enters base-sol the way the
start sheet's door does (one catalogued batch):

- base-sol alone (its NSSDCA air, temperature and water, PR 231): Earth
  288 K, mobile lid, dynamo on; Venus 737 K, not a mobile lid, no dynamo;
  Mars 214 K, stagnant lid, no dynamo;
- the Moon (no measured mean) sits at its own-albedo equilibrium temperature;
- typed temperature and air win over the estimate;
- Mars moved to 1 AU warms by sqrt(a_then / a_now); a typed temperature then
  stands as typed.

No golden changed (the fixtures carry no albedo or air on fact-built worlds).

**Also found.** base-sol prints `body.obliquity` in degrees while kernel facts
are radians; the chart room's door now converts by unit (PR 225).

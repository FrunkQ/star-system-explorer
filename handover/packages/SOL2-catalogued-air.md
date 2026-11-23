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

Measured beats estimated; nothing is typed. The data sheet thread adds the
NSSDCA air, temperature and water facts to `packs/base-sol` under these keys.

**Tests.** `crates/sse-kernel/src/sol_tests.rs` enters base-sol the way the
start sheet's door does (one catalogued batch):

- base-sol alone (its NSSDCA air, temperature and water, PR 231): Earth
  288 K, mobile lid, dynamo on; Venus 737 K, not a mobile lid, no dynamo;
  Mars 214 K, stagnant lid, no dynamo;
- the Moon (no measured mean) sits at its own-albedo equilibrium temperature;
- typed temperature and air win over the estimate.

No golden changed (the fixtures carry no albedo or air on fact-built worlds).

**Also found.** base-sol prints `body.obliquity` in degrees while kernel facts
are radians; the chart room's door now converts by unit (PR 225).

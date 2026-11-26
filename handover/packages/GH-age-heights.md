# GH age heights: stars' scale height by age

Not started. Written up 26 Nov 2026 by the thread "Fix Local Arm dust physics" for a fresh agent (Alex asked
for young stars at their true height; the coordinator split it out of the Local Arm PR).

## The fault

Every disc star sits in one vertical profile whatever its age: `GalaxyEnvelope::systems_pc3` (and `local`)
put a ring's stars in a thin disc (300 pc) and a thick disc (900 pc, 12% of them) by fixed shares. The star
draw, the count tree and the glow all read that, and the class mix of a column (`TreeWalk::class_mix`, the
glow's `light_points`) depends on radius only. So a 20 Myr O star is as likely at 600 pc as a 10 Gyr K dwarf.

Measured: OB stars sit about 45 to 50 pc from the plane, A stars about 100, old thin-disc dwarfs about 300, the
thick disc 900 and old (from memory, to verify).

## The physics is already in the code, unused for counts

The draw gives each star a vertical oscillation from its age: sigma_z = 25 km/s x (age / 10 Gyr)^0.35, at least
5 km/s (`StarParams::sigma_*`, Holmberg et al. 2009), in a harmonic potential nu = sqrt(4 pi G rho_0) with
rho_0 = 0.1 (`midplane_density`). An ensemble of those orbits is a Gaussian of width sigma_z / nu: about 41 pc
when young, 340 pc at 10 Gyr. One general mechanism: the counts should use the same relation.

## What it needs

1. The envelope's stellar vertical profile as a sum over the ring's age bins, each at its own height
   (sigma_z(age) / nu), the thick disc as the oldest stars (the draw's `birth()` already calls stars born before
   3 Gyr "thick disc"). Keep each ring's total; cache per ring and step (it is on the hot path).
2. The count tree: a class's expected count in a node is the sum over age bins of its share times that bin's
   vertical profile, so `class_mix` and `expected` become height-aware (the four-point Gauss-Legendre in height
   already exists in `expected`). Births drawn in a cell weighted by the vertical profile at its height.
3. The glow: the light mix per (radius bin, height bin), or per age bin weighted at each step.
4. The "drawn stars plus glow is the same at any limit" test must still pass: it is the check that the draw and
   the glow agree.

## What to expect

Offline from traced sight lines (26 Nov, Local Arm thread): young stars at their height brighten V's midplane,
so the glow test's V lane falls (to about 0 at 54 pc). That is not a reason to skip it; it says the remaining
gap in that test is dust (the chemistry retune, CC3) and the bulge's size, not the stars' heights.

Every drawn star's position statistics move: core lane, one at a time, goldens re-blessed with this reason.

# BD1 (follow-up, not started): young brown dwarfs come out far too hot

Found 26 Nov 2026 by the star classifier (PR 642): very young brown dwarfs are classed A to K. Written up by the
thread "Fix how giant stars age" (package GE1), which left it out of its own change on purpose.

## Cause (read from `crates/sse-formation/src/lifeline.rs`, `Phase::BrownDwarf`)
- Luminosity is Burrows et al. 2001 eq. 3, L = 4 × 10⁻⁵ (t / Gyr)^−1.3 (M / 0.05)^2.64 L☉, with the age floored
  at 1 Myr. At 1 Myr that is about 0.3 L☉ for 0.05 M☉.
- The radius is fixed at 0.1 R☉ at every age. A brown dwarf a few Myr old is still contracting and is several
  times larger (from memory: about 0.5 to 1 R☉ at 1 Myr, falling to about 0.1 R☉ only after a few hundred Myr).
- So T = T☉ (L / R²)^¼ comes out near 14,000 K at 1 Myr, where real ones are about 2,500 to 3,000 K (spectral
  type M6 to M9 when young). The luminosity is roughly right; the radius is the fault.
- Eq. 3 is also a fit for the cooling phase; it is not meant for the first few Myr (deuterium burning holds young
  brown dwarfs up; `body.rs` already knows the deuterium phase for its light source).

## The fix to build
- Radius from contraction: a brown dwarf radius against age and mass (Burrows et al. 2001 figure 3, or the
  Kelvin-Helmholtz contraction of a polytrope n = 1.5 with the degenerate floor of Chabrier and Baraffe 2000),
  so T follows from L and R instead of a fixed R.
- Check young temperatures against observed young brown dwarfs (M6 to M9 at 1 to 10 Myr) and old ones (L, T, Y),
  as an absolute gate that is seen red with the fixed radius first.
- Same rule for the life line's pre-main-sequence, which is also drawn at fixed ZAMS L and R (fudge F-S6).
- Moves formation, time and core contract goldens (any system with a young brown dwarf); core lane.

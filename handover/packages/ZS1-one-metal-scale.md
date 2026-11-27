# ZS1: one metal scale, so the model Sun is the Sun (27 Nov 2026)

Thread "Fix how giant stars age", after GE1. Reported by the climate thread: the model Sun on its life line (1 M☉,
Z = 0.0142) shone 1.134 L☉ at 4,567 Myr against the measured 1.0.

## Cause
Two solar metal scales met at the Hurley fits:
- HPT00's fits are in log(Z/0.02): the Pols et al. 1998 models they fit put the Sun at Z = 0.02 on the older
  Anders and Grevesse 1989 mix. Formation knew this and kept its Z on the fits' scale (`z_of_feh` = 0.02 × 10^[Fe/H];
  fudge F-S13 said so).
- The kernel's catalogued side used measured Z (`adopt::DEFAULT_Z` = 0.0142, Asplund et al. 2009), as did
  `giant_field`'s default, and passed it straight to `Fits::new`. The fits read 0.0142 as [Fe/H] −0.15, a metal-poor
  star, which burns brighter and dies sooner: 1.134 L☉ at 4,567 Myr, a main sequence of 10.0 Gyr instead of 11.1.
- The fact `star.metallicity_z` mixed the scales too: generated systems wrote the fits' 0.02 for a solar star,
  and the kernel's readers (`[Fe/H]` for the Explore metals change, planet composition's default) took it as measured,
  so a solar generated system read as [Fe/H] +0.15 there.

## Change (cause, not scaling)
- Every Z in SSE4 is measured: `hurley::Z_SUN` = 0.0142 (Asplund et al. 2009 protosolar, from memory).
- `Fits::new(z)` converts at its own door, `fit_z(z) = z × 0.02 / Z_SUN` (the same [Fe/H]); `Fits` keeps `z`
  (measured) and `z_fit`. White dwarf cooling's Z^0.4 reads the fits' scale too.
- `z_of_feh` = Z_SUN × 10^[Fe/H]; `Z_FITS` (the fits' range, 1e-4 to 0.03 on their scale) is given as measured,
  7.1e-5 to 0.0213; `DEFAULT_Z` and `giant_field`'s default are `Z_SUN`.
- Inputs that meant "solar" on the old fits' scale say 0.0142 now: the Solar System fixture
  (`tests/fixtures/time/solar-system.json`), the core contract's `look_ahead` case for the Sun, and the tests that
  checked HPT00 and Tout 1996 numbers at "Z = 0.02". Those still check the same paper numbers.

## After
- The model Sun at 4,567 Myr: 0.952 L☉, 0.983 R☉, 5,749 K (was 1.134, 1.030, 5,869). The remaining 5 percent is
  HPT00's own fit at its solar Z (their ZAMS Sun is 0.698 L☉), not tuned.
- Generated systems' stars are unchanged (the fits see the same Z): formation and time goldens move only in hashes and
  the stored Z. The Solar System's ageing (`time.json`) moves only its manifest and chronicle head; every event and
  the run digest are unchanged.
- Catalogued stars at measured Z now age as their [Fe/H] says.

## Goldens (re-blessed on purpose)
formation, formation-pipeline, formation-accrete-dole, time-generated, time (manifest only), traveller-codes. Kernel
contract (core and areas): listed in the PR once main compiles again (PR 688).

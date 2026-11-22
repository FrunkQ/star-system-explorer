# Y2, light side: star magnitudes through dust

Night sky design (`design/SSE4-night-sky-design.md`) sections 4.3 and 11, stage Y2. The kernel thread owns the
draw (`sky_from_here`: which stars, directions, light time); this package is the light track's share: band
fluxes and dust extinction, so the draw can turn a star and a sight line into magnitudes and colours.

## What was built

- `packs/base-light`: `passbands`, the Johnson-Cousins U, B, V, R, I and 2MASS J, H, Ks bands with each
  band's effective wavelength and AB minus Vega offset (Blanton and Roweis 2007, Table 1). Pack data, so a
  setting can add bands (Gaia G, SDSS) without code. `LightTable::passband(id)`.
- `crates/sse-atoms/src/extinction.rs`: atom `sse.light.dust_extinction`, the Cardelli, Clayton and Mathis
  1989 curve A_λ/A_V for any R_V (infrared, optical, ultraviolet with the 217.5 nm bump, far ultraviolet).
  Atom count 77 to 78.
- `crates/sse-light/src/photometry.rs`:
  - `photometry_of(f_λ, sight, bands)`: the one path. Flux density at each band's effective wavelength gives
    the AB magnitude (3631 Jy zero point, Oke and Gunn 1983), the band offset gives Vega, and A_V times the
    CCM ratio adds the dust. Colours U−B, B−V, V−R, V−I, J−Ks and V−Ks with their dust excess.
  - `star_photometry(photosphere, sight, bands)`: a blackbody photosphere from temperature and radius or
    luminosity. A few Planck evaluations a star, cheap enough for a sky draw.
  - `target_photometry(target, a_v, r_v, bands, table)`: a resolved target's full light description
    (every component, Doppler shifted), so a point in the sky and its system's card agree.
- Kernel op `read_photometry { stars, bands? }` (Trivial): each star is a photosphere plus a sight line
  (`distance_m`, `a_v`, `r_v`). Error `bad_photometry`. Two contract cases (the Sun at 10 pc, a reddened
  red dwarf; and a star with neither radius nor luminosity). The contract golden gains them on purpose;
  the stack hash moved with the new atom.

## Checks (`crates/sse-light/src/tests.rs`, `crates/sse-atoms/src/tests.rs`)

- The Sun as a 5772 K blackbody at 10 pc: M_V 4.85 against 4.78 and B−V 0.65 against 0.65 (Blanton and
  Roweis 2007; Ramírez et al. 2012). From luminosity, the same magnitudes.
- One magnitude of dust in V dims V by 1.00 and gives E(B−V) = 0.33, which is A_V/R_V; Ks loses under 0.15.
- CCM89 ratios: A_B/A_V 1.32, A_K/A_V 0.114, the 217.5 nm bump above its sides.
- A hotter star is bluer (10,000 K reads B−V 0.14; the stand-in, see below).

## Stand-ins (docs/fudges.md row "sse_light::photometry")

Bands read at one wavelength, not integrated through filter curves; fill stars as blackbodies. Hot stars come
out about 0.1 to 0.15 too red in B−V.

## For the kernel thread (Y2 draw)

Call `sse_light::photometry::star_photometry` per kept star with A_V from the patch dust model along the
sight line. A class's visibility radius can use the same function: the distance at which the class's
brightest member reaches the limiting magnitude, with A_V growing along the way.

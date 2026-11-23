# GH2: orbits and the four processes after infall

Track GH (`design/SSE4-galaxy-histories-design.md`, section 7.3 and the three cuts of section 10), built 23 Nov
2026 by the kernel thread, after GH1 (assembly tree) and DM4 (halo structure and rotation curve).

## What it does (`crates/sse-cosmos/src/infall.rs`, fudge F46; F39 narrowed)
- **Host** (`infall::Host`): the host's potential through time, one radial table per stored slice (every 0.25 Gyr
  from the first infall to the time read), each from DM4's `Structure` with the host galaxy's state at that time.
  The table holds the pull, the potential, dark matter and hot gas density, the speed spread and the enclosed
  mass with its slope, on one fixed log grid (0.05 to 2,000 kpc, 96 points). The potential is read by cubic
  Hermite in ln r with the pull as its slope, so the force is exactly the potential's gradient and a body's
  energy holds in a still host.
- **Infall** (`infall::infall`): each progenitor enters on the host's r200 at its infall time, in a keyed
  direction, at 0.9 to 1.4 of the host's virial speed with a radial share of 0.3 to 1 (Jiang, Cole, Sawala and
  Frenk 2015). It carries its own NFW profile (Dutton and Macciò 2014 concentration at infall) and, at Chart
  level, its own galaxy's stars, gas and disc scale at infall (for gas stripping). The draws use the counters
  Y4 used for the old orbit and place (`draw_base + 2` to `+ 6`), so the tree and every member are unchanged.
- **The four processes** (`infall::follow`), every body, every step, no gates:
  - sinking: Chandrasekhar drag, ln Λ = ln(1 + M/m), σ = v_c/√2, speed factor from a fixed erf table;
  - tidal stripping: the mass outside the tidal radius r_t = r (m(<r_t)/((2 − d ln M/d ln r) M))^(1/3) goes at
    α = 0.55 per the satellite's dynamical time (SatGen), sped by heat; stars stay until a tenth of the dark
    matter is left (Peñarrubia et al. 2008), then fall in proportion (F46);
  - gas stripping: ram pressure ρ_hot v² against 2πG Σ_tot Σ_gas at the satellite's disc scale (Gunn and Gott);
    when it wins, star formation stops. This replaces the fixed 3 Gyr delay;
  - shock heating: each pass through the disc inside five scale lengths adds ΔE = 2 g_m² r_h² / (3 v_z²) against
    the satellite's binding; heat speeds stripping and at 1 the satellite is torn apart.
- **Labels** read off the result: `satellite`, `being torn apart` (with `stream`), `stream`, `shell` (closest over
  farthest approach under 0.1), `mixed remains` (torn apart over 5 Gyr ago), `merged` (inside 0.01 r200). The
  thresholds are pack settings (`SkyParams.satellites.orbits`).
- **Track**: each body's position at every host slice from infall on, so satellites can play forward and back on
  the ruler without recomputing.

## The three cuts (design section 10), each inside its condition
1. A body is stepped only from its infall.
2. Its step is sized once at infall from its own closest approach (pericentre distance over speed there, 25 steps
   near pericentre, kept between 10⁻⁴ and 0.01 Gyr), from its initial conditions and the host at that time. It is
   a function of the body alone. Test `a_path_is_the_same_on_rerun_and_whatever_time_it_is_read_to`: the same
   step and the same track whether read to half-way or to now.
3. One table per host per slice, built once. Test `the_host_table_matches_the_direct_formulas`: the pull and
   density within 2 % of `Structure` from 0.3 kpc to r200 (stated tolerance), the potential's slope equals the
   pull, and the drag factor table within 2 × 10⁻⁴ of erf.

## Checks
- Energy: in a still host with no processes, leapfrog energy drifts under 10⁻⁴ over 13 Gyr (bound: 10⁻³).
- Four median 10¹² M☉ hosts, Chart level, 253 members: 196 survive to now (about 49 a host above ψ 10⁻³); of 14
  over 3 % of the host, 10 reached the centre or were torn apart; 124 of 145 below ψ 3 × 10⁻³ survive; 177 that
  came close lost dark matter; 16 are torn apart or losing stars. Every small survivor has stopped forming stars
  (Wetzel, Tollerud and Weisz 2015); 2 of the 12 survivors over ψ 0.01 still form stars. Most small dwarfs lose
  their gas soon after infall: G3's low-mass galaxies hold only 10⁵ to 10⁷ M☉ of gas on a 0.2 to 0.4 kpc disc,
  and the host's hot gas at r200 is already about 10³ M☉ kpc⁻³ (4 × 10⁻⁵ cm⁻³).
- The kernel's sky test: satellites now lie up to their farthest approach (allowed to 1.5 r200, the splashback
  radius), and a quenched satellite forms no stars.
- Identical on rerun (test above); no hash maps, pinned maths only, no fused multiply-add.

## Measured
- Survey level (2,140 bodies of one 10¹² M☉ host, orbits only): 5.1 million steps, 0.80 s natively on the cloud
  runner (0.16 µs a step). GH0 measured the first-written orbits at about 8 s (28 million steps at 0.3 µs). The
  1 s laptop target is met natively; the WebAssembly figure needs the harness bench on the same machine.
- Chart level in the kernel's sky op: the host galaxy, the slices, about 57 member galaxies (as before) and their
  orbits. Test `survey_orbits_timed` (ignored by default) prints the Survey figure.

## Kernel (`crates/sse-kernel/src/galaxies.rs`)
- `satellites_seen` follows every member: merged ones are counted, torn-apart ones are counted by label under
  `remains`, survivors are placed where their orbit has them, their light cut to the stars still bound, with
  an `orbit` entry (labels, dark and stars kept, pericentres, closest and farthest approach, disc passes, heat,
  when quenched, velocity).
- `SatelliteParams` keeps only `orbits`; the Boylan-Kolchin merger times, Einasto placement and fixed quench
  delay are gone (no pack set them).

## Goldens (re-blessed on purpose)
The kernel contract: the responses that list our galaxy's satellites (the sky). Each satellite moves to where its
orbit has it, gains `orbit`, and `remains` joins the walk; the merged count changes with the processes (11 to 10
in the sky request). No other golden changes.

## Next
GH3 (progenitors feed the host: stars, gas, holes; merger responses with dated marks). Citations to check for the
citations thread: Jiang, Cole, Sawala and Frenk 2015 (MNRAS 448, 1674), Dutton and Macciò 2014 (doi:10.1093/mnras/stu742).

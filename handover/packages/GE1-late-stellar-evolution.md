# GE1 (giant evolution): late stellar evolution, giants that stay inside real stars (26 Nov 2026)

Thread "Fix how giant stars age". Asked by the coordinator after two agents reported faults: `describe_star` gave a
25 M☉ star at 7.5 Myr 6,090 R☉ at 2,650 K; the kernel caps every giant's convective speed at Mach 1; the Formation
screen's look-ahead would turn red dwarfs into giants.

## The sweep (before)
Life line (`crates/sse-formation/src/lifeline.rs`) read over 0.1 to 100 M☉, every age from the end of the main
sequence to the end, at Z = 0.02 and 0.001:
- **Massive stars brightened ten times** through the giant phases (`l_tip = 10 × l_bgb` for M > M_HeF). The radius
  law then put a 25 M☉ star at 2 × 10⁶ L☉ and 7,230 R☉ at 2,575 K at its end, 60 M☉ at 19,000 R☉, 100 M☉ at
  31,600 R☉ and 2,420 K. That one line was the cause of the reported 6,090 R☉.
- **No Humphreys-Davidson limit.** Stars of 40 to 100 M☉ were drawn as cool hypergiants at 10⁶ to 10⁷ L☉, where
  no star is seen.
- **Red dwarfs became red giants** with a 2,500 L☉ tip (0.1 M☉ to 292 R☉) although stars below about 0.25 M☉ never
  become giants, and stars below about 0.47 M☉ cannot grow the helium flash core that sets that tip.
- **Intermediate masses (2 to 8 M☉) were too faint at the end**: 10 × L_TMS gave a 3 M☉ star a 2,000 L☉ tip and a
  2 M☉ star 375 L☉, where the asymptotic giant branch reaches about 10⁴.

## What changed (cause, not clamp)
- The giant tip now follows the core the star ends with:
  - degenerate helium core (M ≤ M_HeF): the 2,500 L☉ red giant tip as before, or D·Mc⁶ (HPT00 eq. 37, D from the
    zpars at the star's Z) at the star's whole mass when it is smaller than the flash core;
  - non-degenerate core ending as a white dwarf: the AGB tip from Paczyński 1970, L = 59250 (Mc − 0.522), at the
    white dwarf's mass from the existing Cummings et al. 2018 relation;
  - massive stars: 2 × L_TMS (Geneva tracks, from memory).
- Stars below 0.25 M☉ (Laughlin, Bodenheimer and Adams 1997) go straight from the main sequence to a helium white
  dwarf of their whole mass. Their `t_bgb = t_end = t_tms`.
- A giant passing the Humphreys-Davidson limit (6.3 × 10⁵ L☉, the value the night sky already uses) is stripped:
  new `Phase::WolfRayet` (`"wolf-rayet"`, label "Wolf-Rayet star") from new `LifeLine::t_strip` to `t_end`, drawn
  as a zero-age helium star of the same luminosity (HPT00 eqs. 77-78). A star already past the limit when its main
  sequence ends never crosses to the red. `describe_star` lists the new span and leaves out spans a star skips.
- The ageing engine (`sse-time` epoch) tells a `stripped` event; body light reads the new phase as fusion.

## After (Z = 0.02, largest size over the whole life)
| Initial mass | Tip L☉ | Largest R☉ | at K | End |
|---|---|---|---|---|
| 0.15 | none | main sequence only | | helium white dwarf 0.15 |
| 0.3 | 171 | 36 | 3,497 | white dwarf 0.30 |
| 1 | 2,500 | 162 | 3,202 | white dwarf 0.57 (unchanged) |
| 3 | 13,200 | 400 | 3,093 | white dwarf 0.74 |
| 5 | 28,700 | 574 | 3,136 | white dwarf 1.01 |
| 15 | 1.2 × 10⁵ | 1,042 | 3,309 | neutron star |
| 20 | 2.5 × 10⁵ | 1,611 | 3,204 | neutron star |
| 25 | 4.1 × 10⁵ | 2,169 | 3,145 | black hole |
| 40 | 1.1 × 10⁶ | 2,454 (before stripping) | 3,282 | Wolf-Rayet, then black hole |
| 60, 100 | | 60, 135 (blue, never red) | 25,000 to 21,000 | Wolf-Rayet at 130,000 to 150,000 K, then black hole |

The reported cases: 25 M☉ at 7.5 Myr is now 2,060 R☉ at 3,171 K (was 6,090 at 2,650); 20 M☉ at 9 Myr 1,124 R☉.

## What is still rough (fudge rows F-S8, F-S32 to F-S35)
- Red supergiants run about 300 K cool and a quarter large: the HPT00 giant-branch radius is used above the masses it
  is fitted to (F-S34). Not tuned. The proper fix is HPT00's core helium burning and AGB radius.
- The full HPT00 giant phases (L_HeI, L_BAGB, t_He, AGB core growth) are the real replacement for F-S7 and F-S8.
  The coefficient tables are already in `hurley/coeffs.rs`, but the port lacks SSE's `zcnsts` layout of the gbp
  coefficients and `zfuncs`. The cloud session could not fetch them (a package download was refused by the session's
  safety check, and the published source pages may only be quoted in short pieces). With the SSE source in the
  project files the port is a straightforward next package.
- No winds in the life line: masses are initial masses until the remnant; stripping is instant at the limit.
- Low-mass stars still stop at the red giant tip (no horizontal branch or AGB): the Sun's largest radius stays
  0.85 au (T1). Left alone here because the T1 core clock and the Solar System golden read it.
- Two copies of the Humphreys-Davidson limit (here and sse-cosmos `cool_limit_lsun`); Davies et al. 2018 put the red
  supergiant ceiling near log L 5.5. Moving both together is a separate decision.

## The Mach 1 cap (question 2)
The cap in `sse-atoms/src/convection.rs` is physics, not a clamp hiding the radius fault: supersonic surface flows
shock and lose their energy, and 3D models of red supergiants show near-sonic surface flows. Correctly sized giants
hit it too (an 18 M☉, 800 R☉ Betelgeuse would be Mach 4.8 uncapped). The weak part is the speed law's fixed
photospheric opacity, which takes ordinary red giants (Arcturus: Mach 1.5 uncapped) to the cap earlier than real
ones probably are. Logged as F-S35 with the replacement (velocities from a 3D model atmosphere grid). Not changed:
it is the star surface block's law, and swapping it moves every star's churn.

## Checks
- New test `every_life_line_stays_inside_real_stars` (sse-formation): 61 masses from 0.1 to 100 M☉, two metallicities,
  400 ages each: radius under 3,000 R☉, temperature above 2,900 K, nothing cooler than 8,000 K brighter than the
  limit, Wolf-Rayet stars hot and small; plus the reported cases. Seen red with the old tenfold brightening.
- Goldens re-blessed on purpose: `formation.json`, `formation-pipeline.json` (every system hash: the life line has a
  new field and red dwarf ends moved), `time-generated.json` (red dwarfs' end ages, now the end of the main
  sequence). Solar System ageing (`time.json`) unchanged. Core contract: see the PR.
- Figures above marked "from memory" are unverified: paper search is out until 1 Dec 2026.

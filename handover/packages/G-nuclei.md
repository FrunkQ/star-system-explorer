# G nuclei: galactic nuclei (black holes, quasars, mergers, tidal disruptions)

Status: done (19 Nov 2026). Kernel track. Design: the galaxy design (`handover/design/SSE4-galaxy-design.md`), the nucleus parts of sections 4 and 9, and the agreed plan: "quasar luminosity and spectrum, seed channels and black-hole mergers are one G package after J4 to J7, publishing the nucleus as a body at the halo centre".

## What was built
**`sse_cosmos::nucleus`.** `nucleus(cosmology, galaxy params, nucleus params, halo, galaxy, edits, seed)` reads G3's galaxy (its black-hole mass history and the bursts G3 fed it) and turns the hole into a nucleus with a light history. Nothing in G3 changes: the hole's mass is G3's, so the quench feedback and every G3 golden stay as they were.

| Part | What it does | Physics |
|---|---|---|
| Seed | `direct_collapse` (1e5 Msun at z = 10, only in haloes past the atomic-cooling 16.5 km/s, with a keyed 10% chance) or `pop3_remnant` (100 Msun when the galaxy starts). The time to reach G3's seed mass at the Eddington rate is reported. | Volonteri 2010; Bromm and Loeb 2003 |
| Episodes | Each G3 burst feed is grown at the Eddington limit, so it lasts `t_Sal ln(1 + feed / m0)`; the peak is the Eddington luminosity of the grown hole. Inside an episode the history holds back the part of the feed not yet swallowed. | Salpeter 1964; t_Sal = 0.45 Gyr x eps / (1 - eps), 50 Myr at eps = 0.1 |
| Light | L = eps Mdot c^2 above 1% Eddington; below it the flow is radiatively inefficient and L falls as lambda^2 (continuous at the join). Eddington ratio, and the thin disc's peak temperature (about 1e5 K at 1e8 Msun, hotter for smaller holes). | Shakura and Sunyaev 1973; Narayan and Yi 1995 |
| Class | `quasar` (L >= 1e38 W), `seyfert` (>= 1e35 W and lambda >= 1e-3), `low_luminosity` (lambda >= 1e-5), `dormant`. | conventional cuts |
| Seen from | `seen_from(nucleus, params, point, direction)`: within 10 degrees of a radio-loud jet a `blazar`, within the 45 degree torus opening `type_1`, otherwise `type_2`. The jet axis is a keyed isotropic direction; radio loudness is keyed, 10% at 1e8 Msun, rising as M^1.6, capped at 50%. | Antonucci 1993; Urry and Padovani 1995; Best et al. 2005 |
| Mergers | Every major halo merger brings a hole of ratio x the host's; they coalesce 1 Gyr later (or are still a pair today). The chirp mass is given for gravitational-wave work. | sketch delay |
| Tidal disruptions | Rate 2.9e-5 per year at 1e8 Msun, scaling as M^-0.404, none above the Hills mass 1.1e8 Msun (Sun-like stars go in whole). `tidal_disruptions(nucleus, params, seed, from, to)` draws them per 10 kyr bin, so any window gives the same events as the bins it covers. | Stone and Metzger 2016; Hills 1975 |
| Body | The nucleus is `<halo>/nucleus:0`. `facts(nucleus, point)` gives `body.mass`, `body.radius` (Schwarzschild), `body.compact = black_hole`, `star.luminosity` and the `nucleus.*` facts, for the chronicle and the one-body classifier. | |

**Pack.** `packs/base-cosmology` gains `cosmology.nucleus` (all 20 settings) and `sources.nucleus`. `CosmologyPack.nucleus` reads it; a test checks the pack holds the defaults.

## Results (median haloes, seed g3)
| Halo | Hole today | Today | Brightest | Quasar phase |
|---|---|---|---|---|
| 1e11 | 2.4e6 | low luminosity, lambda 2e-4 | 1.2e37 W | none |
| 1e12 (Milky-Way-like) | 2.9e7 | low luminosity, lambda 6e-4 | 9.7e37 W | none |
| 1e13 | 3.2e7 | dormant | 2.4e38 W at 2.2 Gyr | yes, z about 3 |
| 1e13.5 | 3.9e7 | dormant | 1.6e38 W at 1.3 Gyr | yes, z about 5 |

- No median nucleus is a quasar today; the quasar era is the young universe, as observed (quasar density peaks at z 2 to 3).
- The Milky-Way-like nucleus disrupts a star every 20 000 years or so (Sgr A* estimates: 1e-4 to 1e-5 per year).

## Checks
- `nucleus_tests::black_holes_quasars_and_quiet_nuclei`: seven halo masses; Eddington ratio never above 1; no quasars today; group haloes had a quasar phase before 6 Gyr; Milky-Way-like hole between 1e6 and 1e8 with a TDE rate between 1e-5 and 1e-3 per year; no TDEs above the Hills mass.
- `nucleus_tests::light_views_and_events`: the Eddington luminosity of 1e8 Msun (1.26e39 W); disc peak temperatures; the lambda^2 regime; the Salpeter time; the unified-model views along and across the axis; chirp masses; TDE windows add up and match the rate within 5 sigma; body facts and address; determinism.
- `nucleus_tests::pack_holds_the_defaults_and_golden`.
- New golden `tests/golden/nucleus.json` (new on purpose), also a harness engine (`nucleus`), so the browsers check it too.

## Fudges and limits (for `docs/fudges.md` once it is on main)
- **G3's holes were too small in big galaxies; fixed in the same PR** (section "G3 change" below).
- The merger hole is ratio x the host's hole and coalesces after a fixed 1 Gyr: real delays run from 0.1 Gyr to never, from dynamical friction, stellar hardening and gas.
- The direct-collapse chance (10%) and its single redshift are a sketch; real direct collapse needs a nearby Lyman-Werner source and metal-free gas.
- Radio loudness is keyed once for the whole life; real jets switch on and off with the accretion state.
- Class cuts are conventions, not physics. The thin disc ignores spin (efficiency is one number) and the corona and the X-ray spectrum.
- References were written from memory and need checking with the citations thread.

## Next
- The galaxy charts thread draws the nucleus at the halo centre in `galaxy_chart`.
- G6's quiet-nucleus edit already stops G3's feed; a "light a quasar" edit would be an ordinary starburst edit.
- Quasar spectra by band for track O's sensors. Bright quasars here peak near 1e38 W, at the faint end of observed quasars (1e38 to 1e41 W), because G3's burst feeds are small at z 2 to 3: a later G3 package could feed holes from the cold gas inflow too.

## G3 change: holes arrive with ex-situ stars
G3 grew holes only from its own bursts and star formation, so in group and cluster haloes the hole was 1e-4 to 1e-5 of the bulge, against 1e-3 to 5e-3 observed (Kormendy and Ho 2013). Merging galaxies bring their holes, which sink and coalesce with the host's. G3 now adds `exsitu_black_hole` (1e-3, Reines and Volonteri 2015 for early types) times every ex-situ star it adds (major-merger stars and the smooth satellite term).

| Halo | Hole before | Hole after | Hole / bulge after |
|---|---|---|---|
| 1e12 | 2.9e7 | 3.2e7 | 0.6% |
| 1e13 | 3.2e7 | 8.7e7 | 0.13% |
| 1e13.5 | 3.9e7 | 2.3e8 | 0.11% |
| 1e14 | 2.6e7 | 6.2e8 | 0.10% |

- Radio-mode feedback reads the hole, so big haloes quench a little harder: the group galaxy's SFR today falls from 0.028 to 0.011 Msun/yr and its stars by 1.4%; the Milky-Way-like galaxy's stars by 2.2% (3.01e10 to 2.94e10) and SFR from 2.64 to 2.51 Msun/yr. The dwarf barely moves. The G3 tuning tests all still pass, so `quench_black_hole` was left alone.
- A quiet-nucleus edit still gains the arriving holes (they are not accretion); G6's test now checks exactly that.
- New check in `nucleus_tests`: hole/bulge between 5e-4 and 1e-2 in haloes of 1e13 and up.
- **Golden changes (on purpose):** `galaxy.json`, `envelope.json`, `stars.json`, `galaxy-events.json` and `nucleus.json`, all from the larger holes and the slightly stronger quench.

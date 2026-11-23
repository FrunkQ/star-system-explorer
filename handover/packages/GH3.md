# GH3: progenitors feed the host; merger responses and marks

Track GH (`design/SSE4-galaxy-histories-design.md`, sections 7.5 and 9.4), built 23 Nov 2026 by the kernel thread. G3's two stand-ins for what other galaxies bring (ex-situ stars as ratio squared of the host's, and the smooth ex-situ term in big haloes) are replaced, in a galaxy that is looked at, by what its own assembly tree delivers.

## What it does (`crates/sse-cosmos/src/feed.rs`, fudge F47)
- `feed::members` gives every Chart-level member of the tree (GH1) its own G3 galaxy and its GH2 orbit in the host's potential. `satellites_seen` now calls it too, so the sky's satellites and the feed share one code path (GH2's output is unchanged).
- `feed::arrivals` reads, off each orbit, what reaches the host and when:
  - **Infall**: the halo's growth already counted the member's baryons as smooth inflow, so the part already in its galaxy is taken back from the host's inflow (budget closure). Fed or not, a host ends with about the same stars (test).
  - **Halo**: stars lost at each slice of its track (GH2's `kept`, new) join the stellar halo with the member's mean star metals.
  - **Hot**: when its star formation stops (gas stripping), its gas joins the host's hot reservoir and cools back in through G3's return.
  - **Merge**: what reaches the centre (bound mass, kept stars, any gas left, its black hole).
  - **Pass**: each closest approach and each disc crossing while bound (GH2's `passes`, new).
- `galaxy::evolve_feeding` runs G3 with the feed. A merger answers by its ratios: bulge growth = thin disc × μ × (1 − gas share) (Hopkins et al. 2009); its stars settle in the bulge; its gas and metals join (fresh gas); disc heating into a thick disc; G3's burst; the two holes add. A disc pass heats the disc a little; a closest approach writes marks. Inked mergers (G6) keep G3's rule; a removed G2 merger takes its pin out of the tree.
- **Marks** (design 9.4): `Mark { kind, caused_by, began_gyr, strength 0..1, fades_gyr, shape }`, `strength_at(t)`. Kinds: warp, ripples, tilt, thick_disc (for good), driven_arms, ring, burst_knots, lopsided, gas_trail, warm_side, fresh_gas. Strength is x / (x + half) of the kind's driving ratio, so there are no gates; marks under 0.01 are not listed. Settings in `ResponseParams` (pack key `sky.satellites.responses`).
- `Galaxy.assembled` (skipped when unfed, so unfed galaxies serialise as before): stars not born here, stellar halo and its metals, thick disc, the biggest contributor and its share, members merged and torn apart, marks.

## Where it is used
- The kernel's `fed_galaxy` (memo of the last four, keyed by everything a galaxy is made from) feeds the galaxy chart (any halo, with edits) and the home galaxy behind `with_world` (star cells, the sky, systems) and `reconstruct_origin`.
- The galaxy chart gains `assembly` and a model line.
- Other galaxies in the sky and satellites' own galaxies stay on G3's envelope (recursion one level; a fed galaxy is about 0.13 s natively). The sky's galaxy rows now say `about: true`; the chart of the same galaxy agrees to a few per cent (test).

## A G3 fix on the way
G3's merger and authored bursts took gas into stars but left its metals behind, so every burst raised the gas metal fraction (by 30 % in G6's inked merger). The burst now takes its share of the metals. This moves G3's and G6's goldens on purpose.

## Checks (`crates/sse-cosmos/src/gh3_tests.rs`)
- Stars not born here come mostly from a few big progenitors (median share of the biggest > 0.25 over 16 Milky-Way-mass hosts) and their metals rise with the biggest one's stellar mass (Spearman ρ > 0.4; D'Souza and Bell 2018, doi:10.1038/s41550-018-0533-x; Deason, Mao and Wechsler 2016, doi:10.3847/0004-637X/821/1/5).
- The share of stars not born here rises with mass: about 1 to 10 % at 10¹¹ M☉ haloes, 2 to 40 % at 10¹², above 40 % at 10¹³ (Rodriguez-Gomez et al. 2016, doi:10.1093/mnras/stv2786). Measured means about 0.03, 0.1 and 0.75.
- A Gaia-Sausage-like meal (10¹¹ M☉, gas-rich, 10 Gyr ago) leaves a thick disc of 5 to 60 % of today's thin disc and a dip in the gas metals at the meal (two families of disc stars), with thick_disc, tilt, lopsided, burst_knots and fresh_gas marks; every mark fades by its own time, the thick disc lasts.
- Feeding is repeatable and closes its budget.

## Measured (default parameters, 4 median hosts each)
| Halo | Stars fed / unfed | Not born here | Thick / thin disc | Merged, torn apart of members | Marks |
|---|---|---|---|---|---|
| 10¹¹ | 0.6 to 1.0 | 0.1 to 12 % | 0.3 to 0.9 | 11 to 24, 5 to 10 of 50 to 58 | 750 to 1,140 |
| 10¹² | 0.75 to 1.2 | 2 to 38 % | 0.2 to 0.9 | 9 to 18, 3 to 8 of 50 to 56 | 850 to 1,900 |
| 10¹³ | 0.75 to 1.3 | 69 to 84 % | (no thin disc left) | 8 to 24, none torn of 48 to 55 | 630 to 1,100 |

Time: 0.10 to 0.17 s natively a host (about 50 members at a few ms each). Not yet timed in WebAssembly; the memo keeps it to once a session.

## Known limits (on F47)
- Stellar haloes come out at 10⁶ to 3 × 10⁸ M☉ against the Milky Way's 1.4 × 10⁹: stars leave a member only after nine tenths of its dark matter (F46), and the big ones sink and merge first, so their stars go to the bulge. GH4 places debris by orbit.
- The second, lower-metal family after a meal lasts about a tenth of a Gyr, since a meal's gas arrives at once; the observed one lasts Gyr (Spitoni et al. 2019) because gas keeps falling in.
- Bursts at each close pass are off: G3's burst share is set for whole mergers and at every pass it tripled small hosts' stars.
- Members' orbits run in the unfed host's potential (one pass).

## Goldens re-blessed on purpose
- `tests/golden/galaxy.json` (G3) and `galaxy-events.json` (G6's inked merger): the burst metals fix above (G6's z_gas change at its merger falls from 0.30 to 0.03).
- `tests/golden/contract.jsonl`: the galaxy chart (now fed, with `assembly`), and every op that reads the home galaxy: the star cell (expected systems in the Sun's cell 92.8, now 101.4), the sky and its glow (dust a tenth higher), a promoted star's and system ages' event ids, and the hashes that follow.
- The cosmos crate's own star, sky and envelope goldens are built on unfed scenario galaxies and do not move.

## Next
GH4 (components in the envelope: streams, shells, ancient patches; debris placed by orbit).

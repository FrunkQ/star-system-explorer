# K-a and K-b: jets for the giants' curls, the wind over each sea

Asked by the visual polish design (`handover/design/SSE4-visual-polish-design.md`, section 12, "Asks of other threads"), accepted by Alex on 24 Nov 2026 ("yes to all"). Owner: "Build shared climate and weather".

## K-a: jets by latitude

The `weather` op, `want: "climate"`, on a world whose regime is `banded-giant`, now also returns `jets`:

| Key | What |
|---|---|
| `lat_deg` | band centres, south to north (the S6 band edges' midpoints) |
| `jet_speed_m_s` | eastward wind at the band centre (+ eastward), bands plus the polar jet |
| `jet_shear_m_s` | the band's north edge wind minus its south edge wind, m/s per band |
| `eddy_speed_m_s` | the local eddy speed the Rhines scale reads (one value per giant kind for now) |
| `rhines_scale_m` | width of one jet, π sqrt(2U/β), β = 2Ω cos φ / a, U the eddy speed |
| `deformation_radius_m` | N H / f |
| `source` | the line above in words |

The names are the ones the detail ladder builder asked for (coordinator, 24 Nov 19:21).

### The correction (citation check, correction 3)

`SSE4-visual-polish-citation-check.md` section 6: π sqrt(2U/β) is the width of one jet, not the spacing between eastward jets, and it fits Jupiter and Saturn only with U the local eddy speed (10 to 40 m/s on Jupiter), not the jet speed. The "Verify every cited paper" list (PR 471) scored Rhines 1975 "says something else" for the same reason.

Before: prograde jets = round(sqrt(aΩ/U)) with U = 150 m/s (gas) or 400 m/s (ice), which is one prograde jet per Rhines width with the jet speed. Jupiter got 9.

After: prograde jets = round(½ sqrt(aΩ/U_eddy)), since eastward jets sit two jet widths apart; U_eddy = 17 m/s on gas giants (inside 10 to 40), giving Jupiter 13 (6 to 7 a hemisphere observed) and Saturn 12. Ice giants have no measured eddy speed; 100 m/s keeps Uranus and Neptune at 3. Both values are in the fudge register. The jet speed scales (150 and 400 m/s) stay for the band winds and the polar jet; the polar jet's latitude keeps the jet speed (it puts Saturn's hexagon jet at 78.6°; the eddy speed would move it to about 84°).

The comment in `crates/sse-climate/src/circulation.rs` on the terrestrial Rhines cells ("jets sit π/k_β apart") now says one cell is π/k_β wide; that count was already of widths, so Earth's three cells do not change.

Pack sources corrected in `base-weather` (`weather_giants.bands`, with Gastine et al. 2014 added to the references), `base-surface` (`regime_features.clouds`, which gains the same eddy pair) and `base-climate` (`climate.circulation`). Citations in `docs/citations.md`.

## K-b: wind over each sea

Every weather field cell (`want: "field"`, and the cells inside reports) on a world with a surface and air carries `wind10_m_s`: the mean 10 m wind speed over the band's storms and calms, from the one climate's 10 m wind (W-wind10). It is absent (skipped) on giants and airless worlds, so their goldens do not change. The globe already had each sea band's `wind_m_s` (survey `climate.sea.bands`, PR 418) and the site report its `data.sea.wind_m_s`.

## Goldens

Changed on purpose: the giant layouts and every golden that carries a giant's jet count or band edges (weather-giants, surface cloud deck), and the weather field goldens for worlds with a surface (`wind10_m_s` added). Listed with the PR.

## Tests

- `jet_profile_reads_the_eddy_speed_and_matches_the_bands`: every fixture's profile has one entry per band; near 30° the Rhines width is within a factor of 2 of the band width; Jupiter's width at 30° is 7,000 to 10,000 km.
- `jupiter_ring_saturn_vortex_with_hexagon_ice_giants_no_polygon`: Jupiter 13 jets; ice giants 3.
- `band_edges_match_the_s6_cloud_deck`: the restated formula and the surface pack's eddy pair agree with the weather pack's.

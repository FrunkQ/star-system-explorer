# MF9: aurorae (part 1: where they glow)

Track MF (magnetic fields), design `handover/design/SSE4-magnetic-fields-design.md` section 6.5. Light track.
Each driver of aurorae is computed on every world and kept where it lands inside the bubble; nothing asks
"is this a giant?".

## What was built

- `crates/sse-atoms/src/aurora.rs`, two cited atoms (count 78 to 80):
  - `sse.aurora.footprint`: the magnetic latitude where the dipole line crossing the magnetic equator at r
    reaches the surface, cos²λ = R/r (Kivelson and Russell 1995; oval latitudes Feldstein and Starkov 1967).
  - `sse.aurora.hill_distance`: where plasma loaded at Ṁ stops corotating,
    ρ_H = R (2π Σ_P* B² R² / Ṁ)^¼ (Hill 1979; the form and factor checked in Nichols 2011, eq. 24).
- `sse-field`: `read` returns `aurora`:
  - a **wind** oval at the footprint of the line through the bubble's nose, with its range from a gusty to
    a quiet wind (the settings' gust range);
  - a **spin** oval at the Hill distance, when the body gives `plasma_source_kg_s` and
    `pedersen_conductance_s` and that distance lies inside the bubble;
  - a spot for each of `moons` whose orbit lies inside the bubble;
  - each oval's range in spin latitude for `magnetic_tilt_deg` (an oval at magnetic colatitude c round a
    pole tilted t spans spin colatitudes |t − c| to t + c);
  - `diffuse: true` for a world whose air, not a field, holds the wind off.

## Checks (crates/sse-field/src/tests.rs, crates/sse-atoms/src/tests.rs)

- Earth: wind oval 71.6° (the oval lies at 67° to 75°), lower in a gusty wind, higher in a quiet one.
- Jupiter with Io's tonne a second and Σ* 0.1 S: spin oval mapping to 27 R_J (Hill 2001's 30 at Σ* 0.14),
  inside the wind oval; Io's spot 65.7°; a moon beyond the bubble gets no spot. The Hill check at Σ* 0.14
  gives 30 R_J.
- Uranus, tilt 59°: the oval reaches below 30° spin latitude, far from the spin poles.
- Venus: diffuse glow.

## Stand-ins (docs/fudges.md row "sse.aurora.*")

No open-flux budget (the wind oval is the nose line's footprint), dipole geometry for both ovals, no power,
no colours, no storm events yet.

## Part 2a (24 Nov): every line names its matter

- Each line in base-light's light table gains `matter`, the element or substance that gives it off,
  by its id on the master data sheet (lower-case symbols for elements: `o`, `h`; substances as they
  are: `n2`, `co`). The annihilation line, TiO and CH carry none until the sheet has rows for them.
- `sse_light::table::LineDef.matter`; test `every_line_names_its_matter_on_the_sheet`. The data sheet
  thread adds a sheet check on the same ids and a "Light it gives off" section on the cards.

## Part 2b (24 Nov): how aurorae shine (VP-L2)

- Four Equations, `packs/base-physics/formulas/glow.json` (atoms 134 to 138):
  - `sse.glow.electron_range`: how much air a falling electron gets through (Rees 1989);
  - `sse.glow.wind_power`: 1 % of the wind's power through the bubble's cross-section comes down (Akasofu 1981; the share is F59);
  - `sse.glow.spin_power`: Ṁ (Ω ρ_H)², spinning Io's gas up to the corotation limit (Hill 2001);
  - `sse.glow.line_survival`: 1 / (1 + τ k n), a line's light against collisions (Kalogerakis et al. 2009).
- base-light lines gain `aurora {lifetime_s, quench_m3_s, from [{matter, yield}], source}` on `oi-5577`, `oi-6300`, `n2p`,
  `ha`, `hb` and `lya`. A line shines when a gas of the air names it in `from`: no line asks what kind of world it is.
- `sse_light::colour::colour_of_lines`: the colour of light that is all in lines.
- `sse-field` (`glow.rs`): `Body` gains `spin_period_s` and `upper_air {gases, profile | base_pressure_pa, scale_height_m,
  gravity_m_s2, source}`; `Settings.aurora` gives each driver's electron energy (8 keV wind, 50 keV spin).
  Each oval gains `edges_deg`, `width_deg` (full width) and, flattened in, `power_w`, `power_by_hemisphere`,
  `electron_ev`, `stops_at_m`, `emissions [{line, matter, wavelength_m, power_w, share, peak_height_m, thickness_m,
  colour}]` and the blended `colour` (the light table's `Colour`: chromaticity, linear sRGB with the top channel at one,
  hex). `peak_height_m` is the middle of the layer (half the line's light above, half below); `thickness_m` runs from
  the tenth to the ninetieth part. The diffuse glow on an airy world with no field gets `diffuse_glow` of the same shape.
- The Solar System fixture (`sol-field.json`) gives every planet its spin, Earth the US Standard Atmosphere 1976 profile
  with N2, O2 and O, and the giants their 1 bar pressure, scale height, gravity and makeup (NSSDCA).
- `read_system`'s field read passes the world's spin; its upper air waits on the atmosphere stage (said as a stand-in).

Checks (`aurorae_shine_in_their_air_s_own_lines`, `aurora_lines_have_colours_and_sheet_parents`):

| | Got | Measured |
|---|---|---|
| Earth, power a hemisphere | 37 GW | 5 to 100 GW (NOAA hemispheric power) |
| Earth, oval width | 8.8° (61.6° to 70.3°) | the oval lies at 67° to 75° in quiet times |
| Earth, green 557.7 nm | 110 km, 25 km thick | 100 to 115 km |
| Earth, red 630.0 nm | 203 km, 216 km thick | 150 to 350 km |
| Earth, N2+ 391.4 nm | 110 km | 100 to 120 km |
| Earth, mix | #85FF00, green | green |
| Jupiter, main oval | 1.2e14 W, 2.5° wide, electrons stop 316 km above 1 bar | of order 1e14 W (Hill 2001); visible emission about 245 km above 1 bar (Vasavada et al. 1999) |
| Jupiter, mix | #FF4D9A, pink, H-alpha, H-beta and Lyman-alpha | Saturn pink with a strong H-alpha (Dyudina et al. 2016); no colour reported for Jupiter |

Golden: `contract.jsonl` changes on purpose. Every `read_field` oval gains its width and edges, the wind's oval its
power, and the light table's lines their `aurora` data.

## Next (part 2c and on)

- Moon spots' power and lead angle (Alfvén wing power, Saur et al. 2013).
- Gas shares by height (Earth's red brightens where oxygen takes over above 200 km), and the 427.8 nm N2+ line.
- The kernel passes each world's upper air from the atmosphere stage (`upper_air`), so generated worlds get colours.
- Storm response as dated events from O10's ejections (the bubble shrinks, the oval moves equatorward).
- Patchy aurorae over crustal fields once MF10 lands.

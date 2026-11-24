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

## Next (part 2)

- Power: the wind oval's as a share of the wind's power through the bubble, the spin oval's from the
  corotation currents.
- Colours from the air's own emission lines in the light table, in the output shape agreed with the
  aurora drawing thread on 24 Nov (handover/status/light.md, Next item 2, VP-L2).
- Storm response as dated events from O10's ejections (the bubble shrinks, the oval moves equatorward).
- Patchy aurorae over crustal fields once MF10 lands.

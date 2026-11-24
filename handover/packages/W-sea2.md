# W-sea2 to W-sea4: the sea's colour, waves by liquid, whitecaps

Alex, 24 Nov 2026: seas should be coloured by their liquid, waves should follow the liquid's properties, and storms should froth. It must stay cheap in a browser: the kernel hands over a few numbers per region, and the shaders draw them. Owner: "Build shared climate and weather". The drawing side is built by "Build surface time-lapse" (PR 476) and reads these keys with no change on its side.

## Where the numbers are

- **Site report:** the weather op's `report` returns them in `data.sea`.
- **World card:** the survey returns them in `climate.sea`, with per-band waves under `waves.bands[]`.

All keys are additive.

### W-sea2: colour (`sea.rs` `optics`)

These are read from the data sheet: `substances.<liquid>.optical.absorption_liquid`, `scattering_liquid` (Morel and Prieur 1977, data sheet PR 475) and `refractive_index_liquid`.

| Key | What |
|---|---|
| `bands_nm` | 650, 550 and 450 nm |
| `absorb_per_m` | absorption, per band |
| `backscatter_per_m` | ½ b, beyond 550 nm by the sheet's λ^−4.3 law |
| `refractive_index` | at 550 nm |
| `deep_rgb` | 0.33 b_b/(a + b_b) per band |

- For pure water this gives about [0.0003, 0.0045, 0.06]: a deep blue (test `a_water_sea_is_blue_from_its_own_absorption`).
- A liquid the sheet cannot colour (methane and ethane today) gets a `note` that names what is missing, and no `deep_rgb`.

### W-sea3: waves by the liquid (`liquid`, `drive`, `trains`)

- **Liquid properties:** density, viscosity and surface tension come from the sheet's `phases.saturated_liquid` table at the sea's temperature.
- **`drive`:** the wind's scaling for this air over this liquid, sqrt((ρ_air/ρ_liquid)/0.0012). Earth's growth law reads the scaled wind. This is a stand-in, in the fudge register.
- **`trains`:** up to five per site, and five per band on the card. Each is {wavelength_m, amplitude_m, period_s, dir_deg}, with dir_deg the heading travelled (east 0, anticlockwise).
  - They are sampled from the Pierson-Moskowitz spectrum, with H_s = 4 sqrt(m₀).
  - Each uses ω² = gk + (σ/ρ)k³.
- **`ripple_cutoff_m`:** 2π sqrt(σ/(ρg)). It is 1.7 cm for water on Earth and 3.3 cm for methane on Titan.
- **`min_phase_speed_m_s`:** (4gσ/ρ)^¼.
- **`superfluid` and `liquid_note`:** set below the sheet's `phases.lambda_point_k`. Helium's row does not have it yet; it has been asked of the data sheet thread.

### W-sea4: froth

- **`whitecap_fraction`:** W = 5e-5 (U − 4.47)³ (Stramska and Petelski 2003), on the driven wind. The card gives the year's mean over each band's wind distribution.
- **`foam_albedo`:** 0.22 (Koepke 1984).
- **`streak_dir_deg`:** the wind's heading.

## Alex's acceptance test: a rogue planet with a helium sea

The chain, link by link, and who owns each:

1. **Data sheet** ("Build the master matter data sheet"): helium needs `lambda_point_k`, a critical point and a vapour curve. Its `saturated_liquid` table is there (2.18 to 4.9 K). Asked 24 Nov.
2. **Climate volatiles** (this thread): `sse-climate` `volatile_species` needs a triple point, and helium has none at its own vapour pressure. A liquid-vapour-only path is needed. Next on this thread's list.
3. **Surface cover's sea liquid** (`sse-surface` `cover.rs`, owner of S7): it picks "hydrocarbon" below 200 K and water otherwise. It needs to pick the liquid by phase, from the sheet's substances at the surface temperature and pressure.
4. **The world itself:** a rogue planet's surface stays near 2 K only with a very small internal heat flux, about 1e-6 W/m² (σT⁴ at 2 K). The editor must let the GM set that.

Once 1 to 3 land, the sea code above already gives helium its ripples (σ about 3e-4 N/m, so cut-off waves of about 0.9 cm under 1 g), waves and superfluid line with no special model. Add the end-to-end test then.

## Tests

- `optics_tests`: water is blue; methane says what is missing.
- `liquid_tests`:
  - water's and methane's properties come off the sheet;
  - the capillary cut-offs are right;
  - the trains follow the dispersion and the spectrum;
  - whitecaps start above 4.47 m/s and reach about 0.85% at 10 m/s.

## Goldens

The survey and weather-report sea lines gain the new keys. Earth's waves shift slightly with `drive`, about 1 for Earth's air over water.

# IG2: icy moons' regimes and young shells, and Titan's measured air

Proven core, 26 Nov 2026. The follow-up IG1 left: Europa and Ganymede too dark, Titan treated as airless. Io's sulphur is not part of it (it waits on the sulphur optics rows on the data sheet).

## The causes found

1. **The regime rules never saw the moons' ice.** A catalogued moon carries only its mass and radius, so its `water_fraction` was 0. IG1 read an ice share off the density for ice ground only; the regime rules, rounding by gravity and plumes still read the typed share. So every icy moon in Sol was a rock "stagnant-lid", Europa included, and its renewal (S4's `resurface` entry for `ice-shell`) never ran.
2. **Tile ages ignored the renewal.** Even on a renewed regime, an atlas tile's surface age was the time since the regime began (Europa's whole 4567 Myr), although the crater field already thins craters by the renewal. The dark lag and SW1 weathering read that age.
3. **Titan had no measured air** in base-sol, so it was airless at 82 K.

## What changed

- **One door for "how icy is it"**: `sse_surface::ice_ground::water_share(b)`, the typed makeup or, below 0.1 Earth masses, the density reading (Showman and Malhotra 1999, the `sse.surface.ice_share_from_density` block). The regime quantities, `round_ratio`, plumes and ice ground all read it. The density reading is used only when nothing set the makeup (`Body::water_from_density`, set by adoption when no `makeup.*` fact is typed; formation's share wins, even 0) and only from 5% up (`DENSITY_ICE_MIN_SHARE`: the Moon reads 2% and would otherwise get an ocean and plumes). Both in the fudge register. The bulk composition and cover inputs still read the typed share only (the density reading would give the Moon 2% water).
- **Renewal age** (`atlas::renewal_mean_age_myr`): a world's ground is no older than the mean age its regimes' renewal leaves, ∫ S da over its life with S = `CraterField::renewal_kept` (the same survival S4's craters use). A world with no renewal keeps its regime age. In the fudge register (one mean, not a patchwork).
- **Ice ground is the crust**: it replaces rock land (basalt, granite, regolith) only; a regime's own deposits stay (Titan's organic dunes). An ice regime whose open ice sublimes away (Ceres) is left rock, on the sketch and on worn plates both.
- **Under air the regime's albedo stands**, ice included: haze fallout onto Titan's ground is not modelled (fudge register).
- **Titan in base-sol** (`scripts/build-base-sol.py`, `MOON_SURFACE`): 93.65 K and 1467 hPa at the Huygens site (Fulchignoni et al. 2005, Nature 438, 785, abstract read 26 Nov 2026: verified); air nitrogen with 5.65% methane (Niemann et al. 2005, Nature 438, 779; the abstract confirms nitrogen and methane, the share is the data sheet's and is marked unverified).

## Results (real Solar System, Orbit level 0, all six faces)

| World | Regime before, after | Ground age, Myr | Mean tile albedo before, after | Measured geometric |
|---|---|---|---|---|
| Europa | stagnant-lid, ice-shell | 4567, 123 | 0.32, 0.90 | 0.68 |
| Ganymede | stagnant-lid, cold-ice-lid | 4567, 4567 | 0.27, 0.27 | 0.44 |
| Callisto | stagnant-lid, cold-ice-lid | 4567 | 0.24, 0.24 | 0.19 |
| Titan | stagnant-lid (airless, 82 K), hydrocarbon (1.467 bar, 93.65 K) | | 0.23, 0.30 | 0.22 (with haze) |
| Ceres | (rock), cold-ice-lid with rock ground | 4567 | 0.14, 0.14 | 0.09 |
| Enceladus | small-body, cold-ice-lid | 4567 | 0.28, 0.28 | 1.0 |
| Venus | episodic-lid | 4567, 363 | | |
| Moon, Mercury | unchanged | | 0.13, 0.13 | 0.12, 0.14 |

Europa's crater counts give a surface tens of millions of years old (Zahnle et al. 2003, recalled, not re-read); 123 Myr is the base-surface pack's renewal (60 Myr at 0.06 W/m²) at Europa's lower heat flow. Venus's 363 Myr sits inside the usual 300 to 750 Myr from crater counts (recalled, unverified).

## Caveats (recorded, not fixed)

- **Europa now overshoots** (0.90 against 0.68): its ice is water's authored condensed colour (0.95), and its non-ice matter (hydrated salts, sulphur from Io's plasma darkening the trailing side) is not counted. The fix is a measured frost reflectance row on the data sheet, then Io's sulphur reaching Europa; asked of the data sheet owner through the coordinator, not added here.
- **Ganymede stays dark**: its bright grooved terrain (about two thirds of it, about 2 Gyr old) was made in a past active episode, usually put down to tidal heating while passing through a resonance. Sol's Ganymede has no `tidal_history`, and its density reading (29% ice against about 45% from interior models, IG1's fudge row) makes its shell thicker still.
- **Enceladus** is a cold lid at 0.0055 W/m²: its real tidal heat (south polar terrain, about 10 GW) is not reaching it, so no young ground and no plumes.
- Tile ages on a renewed world are one mean everywhere (fudge register).

## Golden

Re-blessed on purpose: the sse-surface crate goldens (`surface-impacts.json`, `surface-regimes.json`) move because renewed worlds (the Venus and Europa fixtures) get younger tile ages. The core contract golden and the area files do not move: the contract does not load base-sol, and formation-built worlds keep their own water share.

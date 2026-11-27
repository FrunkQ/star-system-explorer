//! The rule-based weather block (`sse.weather.rules`): fills the contract from
//! the circulation, the condensation level and keyed draws. W1 draws the
//! zonal climate (belts by latitude and season); later packages add the
//! formations placed by rule and the drawn storm systems.

use crate::aerosols::{self, AerosolSettings, Sources};
use crate::circulation::{self, Circulation};
use crate::conditions::Conditions;
use crate::contract::*;
use crate::formations::{self, FormationSettings, Terrain};
use crate::giants::{self, GiantSettings};
use crate::settings::WeatherSettings;
use sse_core::math as m;
use sse_surface::atlas::{direction, TileKey};

pub const BLOCK_ID: &str = "sse.weather.rules";
pub const BLOCK_VERSION: &str = "1";

pub struct RuleBlock {
    pub settings: WeatherSettings,
    pub formations: FormationSettings,
    pub giants: GiantSettings,
    pub aerosols: AerosolSettings,
    /// Volcanoes, eruptions, impacts, fires and industry on the world asked
    /// about (W4); empty unless the caller gives them.
    pub sources: Sources,
}

pub fn lat_lon(d: [f64; 3]) -> (f64, f64) {
    (
        m::asin(d[2].clamp(-1.0, 1.0)) * 180.0 / m::PI,
        m::atan2(d[1], d[0]) * 180.0 / m::PI,
    )
}

/// Centre directions of an `n` × `n` grid over a tile, row order.
pub fn cell_dirs(key: TileKey, n: u32) -> Vec<[f64; 3]> {
    let span = 2.0 / (1u64 << key.level) as f64;
    let mut out = Vec::with_capacity((n * n) as usize);
    for j in 0..n {
        for i in 0..n {
            let u = -1.0 + span * (key.x as f64 + (i as f64 + 0.5) / n as f64);
            let v = -1.0 + span * (key.y as f64 + (j as f64 + 0.5) / n as f64);
            out.push(direction(key.face, u, v));
        }
    }
    out
}

/// The direction `x_km` east and `y_km` north of a centre on a sphere of
/// radius `a_m`, on the azimuthal equidistant plane round it: the point the
/// great circle reaches after √(x² + y²) along the bearing atan2(x, y). Its
/// east and north are the centre's; at a pole, north is the direction of
/// longitude `lon_deg` + 180° (so east is longitude `lon_deg` + 90°).
pub fn patch_dir(a_m: f64, lat_deg: f64, lon_deg: f64, x_km: f64, y_km: f64) -> [f64; 3] {
    let (p, l) = (lat_deg * m::PI / 180.0, lon_deg * m::PI / 180.0);
    let c = [m::cos(p) * m::cos(l), m::cos(p) * m::sin(l), m::sin(p)];
    let e = [-m::sin(l), m::cos(l), 0.0];
    let nn = [-m::sin(p) * m::cos(l), -m::sin(p) * m::sin(l), m::cos(p)];
    let d = m::hypot(x_km, y_km) * 1000.0 / a_m;
    if d <= 0.0 {
        return c;
    }
    let (sx, sy) = (x_km * 1000.0 / a_m / d, y_km * 1000.0 / a_m / d);
    let (cd, sd) = (m::cos(d), m::sin(d));
    [0, 1, 2].map(|k| cd * c[k] + sd * (sx * e[k] + sy * nn[k]))
}

/// Fraction of the year since northern spring.
pub fn year_phase(c: &Conditions, date: Date) -> f64 {
    let f = date.day / c.year_days.max(1e-9);
    f - m::floor(f)
}

fn empty_layer() -> Layer {
    Layer {
        cloud_cover: 0.0,
        cloud_type: "none".into(),
        condensate: String::new(),
        optical_depth: 0.0,
        wind_m_s: [0.0, 0.0],
        turbulence: 0.0,
        particle_radius_um: 0.0,
        base_m: 0.0,
        top_m: 0.0,
    }
}

/// Everything about a world the block works out once per call.
pub struct Prepared {
    pub circ: Circulation,
    /// The one climate (W7a) and its season at this date.
    pub core: sse_climate::Climate,
    pub season: sse_climate::ebm::Season,
    pub climate: ClimateAverages,
    pub cond: Option<circulation::Condensation>,
    pub itcz_deg: f64,
    pub phase: f64,
}

impl RuleBlock {
    pub fn new(
        settings: WeatherSettings,
        formations: FormationSettings,
        giants: GiantSettings,
        aerosols: AerosolSettings,
    ) -> Self {
        RuleBlock {
            settings,
            formations,
            giants,
            aerosols,
            sources: Sources::default(),
        }
    }

    /// The same block reading these smoke, ash and smog sources.
    pub fn with_sources(mut self, sources: Sources) -> Self {
        self.sources = sources;
        self
    }

    fn is_giant(p: &Prepared) -> bool {
        p.circ.regime == "banded-giant"
    }

    pub fn from_packs(stack: &[sse_pack::PackManifest]) -> Result<Self, String> {
        Ok(RuleBlock::new(
            WeatherSettings::from_packs(stack)?,
            FormationSettings::from_packs(stack)?,
            GiantSettings::from_packs(stack)?,
            AerosolSettings::from_packs(stack)?,
        ))
    }

    /// Drawn systems, with pins honoured: a pinned system replaces the drawn
    /// one of the same id and is kept for its recorded life.
    pub fn systems_at(
        &self,
        c: &Conditions,
        p: &Prepared,
        date: Date,
        pins: &[Pin],
    ) -> Vec<WeatherSystem> {
        let drawn = if Self::is_giant(p) {
            giants::giant_systems(&self.giants, c, date)
        } else {
            formations::systems(&self.formations, &self.settings, c, p, date)
        };
        let mut drawn = drawn;
        drawn.extend(aerosols::plumes(
            &self.aerosols,
            &self.settings,
            c,
            date,
            &self.sources,
        ));
        let mut out: Vec<WeatherSystem> = drawn
            .into_iter()
            .filter(|s| !pins.iter().any(|q| q.id == s.id))
            .collect();
        for q in pins {
            if q.system.born <= date.day && date.day < q.system.ends {
                let mut s = q.system.clone();
                if s.evidence != "authored" {
                    s.evidence = "pinned".into();
                }
                out.push(s);
            }
        }
        out.sort_by(|a, b| a.id.cmp(&b.id));
        out
    }

    /// One cell with every formation and system at a place.
    #[allow(clippy::too_many_arguments)]
    pub fn cell_at(
        &self,
        c: &Conditions,
        p: &Prepared,
        date: Date,
        systems: &[WeatherSystem],
        terrain: Option<&dyn Terrain>,
        lat: f64,
        lon: f64,
    ) -> Cell {
        if Self::is_giant(p) {
            let gp = giants::prepare(&self.giants, c, date);
            let mut cell = giants::giant_cell(&self.giants, c, &gp, lat, lon);
            giants::set_heights(
                &self.giants,
                &self.settings.climate,
                c,
                &gp.layout,
                &mut cell,
            );
            return cell;
        }
        let mut cell = self.zonal_cell(c, p, lat, lon);
        crate::synoptic::scatter(
            &self.formations,
            &self.settings,
            c,
            p,
            date,
            systems,
            &mut cell,
        );
        formations::imprint(
            &self.formations,
            &self.settings,
            c,
            p,
            date,
            systems,
            terrain,
            &mut cell,
        );
        let sea = terrain.map(|t| t.ground(formations::dir(lat, lon)).1);
        aerosols::imprint_aerosols(
            &self.aerosols,
            &self.settings,
            c,
            date,
            &self.sources,
            sea,
            &mut cell,
        );
        cell
    }

    /// The field over a tile, reading the ground when it is known.
    pub fn field_with(
        &self,
        c: &Conditions,
        date: Date,
        key: TileKey,
        n: u32,
        pins: &[Pin],
        terrain: Option<&dyn Terrain>,
    ) -> FieldTile {
        let p = self.prepare(c, date);
        if Self::is_giant(&p) {
            let mut tile = giants::giant_tile(&self.giants, c, date, key, n);
            let l = giants::layout(&self.giants, c);
            for cell in &mut tile.cells {
                giants::set_heights(&self.giants, &self.settings.climate, c, &l, cell);
            }
            return tile;
        }
        let systems = self.systems_at(c, &p, date, pins);
        let cells = cell_dirs(key, n)
            .into_iter()
            .map(|d| {
                let (lat, lon) = lat_lon(d);
                self.cell_at(c, &p, date, &systems, terrain, lat, lon)
            })
            .collect();
        FieldTile { key, n, cells }
    }

    /// What moves a giant's cloud from `date` over `span_days` (WS2, `want: "flow"`); `None` for a world that is
    /// not a giant.
    pub fn flow(
        &self,
        c: &Conditions,
        date: Date,
        span_days: f64,
    ) -> Option<crate::eddies::FlowTable> {
        let p = self.prepare(c, date);
        Self::is_giant(&p).then(|| crate::eddies::flow_table(&self.giants, c, date.day, span_days))
    }

    /// The field over a patch round a place (`FieldPatch`), cell by cell as
    /// the tiles are: a closer look at a storm or a pole.
    #[allow(clippy::too_many_arguments)]
    pub fn patch(
        &self,
        c: &Conditions,
        date: Date,
        lat_deg: f64,
        lon_deg: f64,
        half_km: f64,
        n: u32,
        pins: &[Pin],
    ) -> FieldPatch {
        let p = self.prepare(c, date);
        let dirs: Vec<[f64; 3]> = (0..n)
            .flat_map(|j| (0..n).map(move |i| (i, j)))
            .map(|(i, j)| {
                let at = |k: u32| half_km * (-1.0 + (2.0 * k as f64 + 1.0) / n as f64);
                patch_dir(c.radius_m, lat_deg, lon_deg, at(i), at(j))
            })
            .collect();
        let cells = if Self::is_giant(&p) {
            let gp = giants::prepare(&self.giants, c, date);
            let l = giants::layout(&self.giants, c);
            dirs.into_iter()
                .map(|d| {
                    let (lat, lon) = lat_lon(d);
                    let mut cell = giants::giant_cell(&self.giants, c, &gp, lat, lon);
                    giants::set_heights(&self.giants, &self.settings.climate, c, &l, &mut cell);
                    cell
                })
                .collect()
        } else {
            let systems = self.systems_at(c, &p, date, pins);
            dirs.into_iter()
                .map(|d| {
                    let (lat, lon) = lat_lon(d);
                    self.cell_at(c, &p, date, &systems, None, lat, lon)
                })
                .collect()
        };
        FieldPatch {
            lat_deg,
            lon_deg,
            half_km,
            n,
            cells,
        }
    }

    /// A plain weather report for one place and date.
    pub fn report(
        &self,
        c: &Conditions,
        date: Date,
        lat: f64,
        lon: f64,
        pins: &[Pin],
        terrain: Option<&dyn Terrain>,
    ) -> Report {
        let p = self.prepare(c, date);
        let systems = self.systems_at(c, &p, date, pins);
        let cell = self.cell_at(c, &p, date, &systems, terrain, lat, lon);
        formations::report(
            &self.formations,
            &cell,
            c,
            &systems,
            date.day,
            self.provenance(),
        )
    }

    pub fn prepare(&self, c: &Conditions, date: Date) -> Prepared {
        let s = &self.settings;
        let core = circulation::core(s, c);
        let circ = core.circulation.clone();
        let climate = circulation::averages(s, c, &core);
        let cond = circulation::condensation(s, c);
        let phase = year_phase(c, date);
        let itcz = circulation::itcz_deg(s, c, &circ, phase);
        let season = core.season_at(phase).clone();
        Prepared {
            circ,
            season,
            core,
            climate,
            cond,
            itcz_deg: itcz,
            phase,
        }
    }

    /// The zonal (belted) weather at one place: the climate drawing W1 gives.
    pub fn zonal_cell(&self, c: &Conditions, p: &Prepared, lat: f64, lon: f64) -> Cell {
        let s = &self.settings;
        let mut layers: Vec<Layer> = (0..LAYERS.len()).map(|_| empty_layer()).collect();
        // Rain and cloud from the one climate's season at this latitude.
        let sea = &p.season;
        let at = |v: &[f64]| p.core.lerp(v, lat);
        let peak = sea.precip.iter().cloned().fold(0.0, f64::max);
        let rel = if peak > 0.0 {
            (at(&sea.precip) / peak).clamp(0.0, 1.0)
        } else {
            0.0
        };
        let u = if p.circ.regime == "airless" {
            0.0
        } else {
            circulation::zonal_wind(s, &p.circ, lat)
        };
        let bands = crate::hooks::level_bounds(c, s);
        for (k, l) in layers.iter_mut().enumerate() {
            // The descent column's height bands, so the two agree (W-heights).
            l.base_m = bands[k].0 * 1000.0;
            l.top_m = bands[k].1 * 1000.0;
            // Winds strengthen with height toward the jets.
            l.wind_m_s = [u * (1.0 + 1.5 * k as f64), 0.0];
        }
        // kg m⁻² s⁻¹ of liquid to mm a day.
        let mut precip =
            at(&sea.precip).max(0.0) * sse_core::sheet::DAY / p.core.liquid_kg_m3 * 1000.0;
        let mut kind = "none".to_string();
        if let Some(cd) = &p.cond {
            let deck = circulation::deck_cover(s, c);
            let cover = at(&sea.cover).clamp(0.0, 1.0);
            let tau = 2.0 + 18.0 * rel;
            // Rising air: deep towers; sinking air: low cloud only. The towers grow in
            // over a span of rising, so a belt's edge is a gradient, not a seam.
            let deep = if p.climate.precip_mean_m_yr > 0.0 && p.circ.regime != "thin-air" {
                ((rel - 0.45) / 0.3).clamp(0.0, 1.0)
            } else {
                0.0
            };
            if p.circ.regime == "thin-air" {
                layers[1] = layer(cover, "cirrus", &cd.substance, 0.3, &layers[1]);
            } else {
                let low_kind = if deep > 0.5 {
                    "cumulus"
                } else {
                    "stratocumulus"
                };
                let low_tau = tau * (0.3 * deep + 0.4 * (1.0 - deep));
                layers[0] = layer(
                    cover * (0.6 * deep + 0.8 * (1.0 - deep)),
                    low_kind,
                    &cd.substance,
                    low_tau,
                    &layers[0],
                );
                if deep > 0.0 {
                    layers[1] = layer(
                        cover * 0.5 * deep,
                        "cumulonimbus",
                        &cd.substance,
                        tau,
                        &layers[1],
                    );
                }
                layers[2] = layer(
                    cover * (0.5 * deep + 0.3 * (1.0 - deep)),
                    "cirrus",
                    &cd.substance,
                    0.5 + 0.5 * deep,
                    &layers[2],
                );
            }
            if deck > 0.0 {
                let pr = s
                    .photochemical
                    .iter()
                    .find(|x| x.cloud_type == "deck")
                    .expect("deck");
                let li = pr.layer.min(LAYERS.len() - 1);
                layers[li] = layer(deck, "deck", &pr.makes, pr.optical_depth, &layers[li]);
            }
        }
        if precip > 0.0 {
            let liquid = c.liquid.clone().unwrap_or_default();
            kind = if liquid == "water" {
                if at(&sea.t_k) < sse_core::sheet::T0C {
                    "snow".into()
                } else {
                    "rain".into()
                }
            } else {
                format!("{liquid} rain")
            };
        } else {
            precip = 0.0;
        }
        let aerosols = haze(s, c);
        let _ = lon;
        let wind10 = if c.solid_surface && p.circ.regime != "airless" {
            p.core.speed10_at(lat)
        } else {
            0.0
        };
        Cell {
            lat_deg: lat,
            lon_deg: lon,
            layers,
            precip_kind: kind,
            precip_mm_day: precip,
            temp_offset_k: 0.0,
            lightning_km2_yr: 0.0,
            aerosols,
            wind10_m_s: wind10,
        }
    }
}

pub(crate) fn layer(cover: f64, kind: &str, cond: &str, tau: f64, old: &Layer) -> Layer {
    Layer {
        cloud_cover: cover.clamp(0.0, 1.0),
        cloud_type: kind.into(),
        condensate: cond.into(),
        optical_depth: tau,
        wind_m_s: old.wind_m_s,
        turbulence: old.turbulence,
        particle_radius_um: 0.0,
        base_m: old.base_m,
        top_m: old.top_m,
    }
}

/// Photochemical hazes (Titan's orange smog) that cover the whole world.
pub fn haze(s: &WeatherSettings, c: &Conditions) -> Vec<Aerosol> {
    haze_from(&s.photochemical, c)
}

/// The hazes of a list of photochemical rules that hold under `c` (W4 adds
/// its own list, with outgassed gas counted in, through this).
pub fn haze_from(list: &[crate::settings::Photochemical], c: &Conditions) -> Vec<Aerosol> {
    list.iter()
        .filter(|p| {
            p.cloud_type == "haze"
                && c.flux_w_m2 >= p.min_flux_w_m2
                && c.pressure_bar >= p.min_pressure_bar
                && p.needs
                    .iter()
                    .all(|(g, x)| c.gases.get(g).copied().unwrap_or(0.0) >= *x)
        })
        .map(|p| Aerosol {
            kind: "haze".into(),
            optical_depth: p.optical_depth,
            colour: p.colour,
            particle_radius_um: p.particle_radius_um,
        })
        .collect()
}

impl WeatherBlock for RuleBlock {
    fn id(&self) -> BlockId {
        BlockId {
            id: BLOCK_ID.into(),
            version: BLOCK_VERSION.into(),
            level: "rule".into(),
        }
    }

    fn climate(&self, c: &Conditions) -> ClimateAverages {
        circulation::climate(&self.settings, c)
    }

    fn field(&self, c: &Conditions, date: Date, key: TileKey, n: u32, pins: &[Pin]) -> FieldTile {
        self.field_with(c, date, key, n, pins, None)
    }

    fn systems(&self, c: &Conditions, date: Date, pins: &[Pin]) -> Vec<WeatherSystem> {
        let p = self.prepare(c, date);
        self.systems_at(c, &p, date, pins)
    }

    fn provenance(&self) -> Provenance {
        Provenance {
            block: self.id(),
            evidence: "drawn".into(),
        }
    }
}

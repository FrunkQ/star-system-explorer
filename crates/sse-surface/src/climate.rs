//! The one climate on the ground (W7a). A body's climate conditions are
//! built here, once, and every surface stage that needs weather (S3 wear's
//! rain and glaciers, S7 cover's temperature, wetness and winds) reads the
//! same [`WorldClimate`]. The weather blocks build their conditions with the
//! same function, so the rain that cuts the valleys is the rain on the
//! weather map.

use crate::cover::Air;
use crate::{Body, Climate as Source, SurfaceEnvelope, SurfaceSettings};
use serde::{Deserialize, Serialize};
use sse_climate::{Conditions, Extra};
use sse_core::math as m;

const R_EARTH_M: f64 = sse_core::sheet::R_EARTH_MEAN;
const G: f64 = sse_core::sheet::G;
const M_EARTH_KG: f64 = sse_core::sheet::M_EARTH;

/// Climate conditions for a body: the facts it has, the inputs given (spin,
/// tilt and so on) and, for anything still missing, the climate pack's
/// defaults, listed in the second value.
pub fn conditions(
    s: &SurfaceSettings,
    b: &Body,
    env: &SurfaceEnvelope,
    x: &Extra,
) -> (Conditions, Vec<String>) {
    let d = &s.climate.defaults;
    let mut assumed = vec![];
    let r = b.radius_earth * R_EARTH_M;
    let g = G * b.mass_earth * M_EARTH_KG / (r * r);
    let (a_au, host) = b
        .orbit
        .as_ref()
        .map(|o| (o.a_au, o.host_msun))
        .unwrap_or((1.0, 1.0));
    let year_days = x.year_days.unwrap_or_else(|| {
        assumed.push("year from the orbit (Kepler)".into());
        365.256_363 * m::sqrt(a_au * a_au * a_au / host.max(1e-12))
    });
    let locked = env.locked;
    let src = b.spin_source.as_deref().unwrap_or("authored");
    let rotation_hours = x.rotation_hours.unwrap_or_else(|| {
        if locked {
            year_days * 24.0
        } else if let Some(h) = b.spin_hours {
            if src != "authored" {
                assumed.push(format!("spin {h:.2} h ({src})"));
            }
            h
        } else {
            assumed.push(format!(
                "spin {} h (not yet published for this body)",
                d.rotation_hours
            ));
            d.rotation_hours
        }
    });
    let obliquity_deg = x.obliquity_deg.unwrap_or_else(|| match b.obliquity_deg {
        Some(o) => {
            if src != "authored" {
                assumed.push(format!("tilt {o:.1}° ({src})"));
            }
            o
        }
        None => {
            assumed.push(format!("tilt {}° (not yet published)", d.obliquity_deg));
            d.obliquity_deg
        }
    });
    let flux = x.flux_w_m2.unwrap_or_else(|| {
        assumed.push("sunlight from a sun-like star at the orbit".into());
        sse_core::sheet::S_SUN / (a_au * a_au)
    });
    let gases = x.gases.clone().unwrap_or_else(|| {
        if b.deep_envelope && !d.giant_gases.is_empty() {
            assumed.push(format!(
                "air of the protosolar mix ({})",
                d.giant_gases_source
            ));
            d.giant_gases.clone()
        } else {
            assumed.push("air of the default mix".into());
            d.gases.clone()
        }
    });
    let t = env.t_surface_now_k;
    let liquid = x.liquid.clone().or_else(|| {
        (b.oceans > 0.0 && t > sse_core::sheet::T0C && t < 373.0).then(|| "water".to_string())
    });
    // Earth's 1 ocean covers 71 % of its surface; a fuller ocean drowns more.
    let ocean_fraction = x.ocean_fraction.unwrap_or(if liquid.is_some() {
        (0.71 * m::pow(b.oceans.max(0.0), 0.25)).min(1.0)
    } else {
        0.0
    });
    let air = Air::guess(&s.cover, env, b.pressure_bar, b.oceans > 0.0);
    let albedo = x.albedo.unwrap_or(match &b.climate {
        Source::Star { albedo, .. } => *albedo,
        Source::Fixed { .. } => s.albedo_default,
    });
    let wet = liquid.is_some();
    let c = Conditions {
        address: b.address.clone(),
        label: b.label.clone(),
        radius_m: r,
        gravity_m_s2: g,
        solid_surface: !b.deep_envelope,
        rotation_hours,
        star_locked: locked
            && b.address
                .segments()
                .last()
                .is_some_and(|s| s.kind() == "body"),
        obliquity_deg,
        year_days,
        eccentricity: b.orbit.as_ref().map(|o| o.e).unwrap_or(0.0),
        perihelion_lon_deg: x.perihelion_lon_deg.unwrap_or(0.0),
        flux_w_m2: flux,
        albedo,
        t_surface_k: t,
        pressure_bar: if b.deep_envelope { 1.0 } else { b.pressure_bar },
        mean_molar_kg: x.mean_molar_kg.unwrap_or(air.molar_kg),
        gases,
        liquid,
        ocean_fraction,
        internal_flux_w_m2: x.internal_flux_w_m2.unwrap_or(0.0),
        ground_ice_kg_m2: Default::default(),
        land_by_lat: if wet {
            x.land_by_lat.clone().unwrap_or_default()
        } else {
            Vec::new()
        },
        coasts_by_lat: if wet {
            x.coasts_by_lat.clone().unwrap_or_default()
        } else {
            Vec::new()
        },
    };
    (c, assumed)
}

/// The one climate a surveyed world's ground reads: the Orbit-level energy
/// balance plus the lapse rate of its air, so a temperature can be read at
/// any height.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct WorldClimate {
    pub climate: sse_climate::Climate,
    /// Cooling with height above the datum, K per km.
    pub lapse_k_km: f64,
    /// Facts the climate assumed because the body does not publish them.
    pub assumed: Vec<String>,
    /// W9 part 2: the ice-age response to the orbit's secular history, when
    /// the world has one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ice_ages: Option<sse_climate::forcing::IceAgeModel>,
}

impl WorldClimate {
    /// Mean temperature at latitude (radians) and height above the datum (m).
    pub fn t_at(&self, lat: f64, h_m: f64) -> f64 {
        self.climate.t_at(lat * 180.0 / m::PI) - self.lapse_k_km * h_m.max(0.0) / 1000.0
    }

    /// Rain and snow reaching the ground at a latitude (radians), m a year.
    pub fn precip_m_yr(&self, lat: f64) -> f64 {
        self.climate.precip_at(lat * 180.0 / m::PI)
    }

    /// W9: the water a place at `lat_deg` and `height_m` above the datum
    /// receives (rain, snow, melt, evaporation, runoff, ice balance).
    pub fn water_at(
        &self,
        cs: &sse_climate::ClimateSettings,
        lat_deg: f64,
        height_m: f64,
    ) -> sse_climate::water::WaterSupply {
        sse_climate::water::water_at(cs, &self.climate, self.lapse_k_km, lat_deg, height_m)
    }

    /// Rain relative to the wettest band, 0 to 1, at a latitude (radians).
    pub fn wetness(&self, lat: f64) -> f64 {
        self.climate.wetness_at(lat * 180.0 / m::PI)
    }

    /// Whether the prevailing surface wind blows from the east (radians).
    pub fn easterly(&self, lat: f64) -> bool {
        self.climate.circulation.easterly(lat * 180.0 / m::PI)
    }
}

/// W7c part 2: land and coasts in `strips` equal strips of latitude, south
/// to north, read off a height map on `grid` (`elevation_m` above sea
/// level; land is above zero).
///
/// - Land: the share of the strip's cells above sea level (the cube grid's
///   cells are near enough equal in area).
/// - Coasts: the mean number a line round the strip crosses. Each land-sea
///   pair of neighbours holds a piece of coast about a cell wide, at right
///   angles to the step between them; its north-south extent is the cell
///   size times the step's eastward share, and the crossings a strip's
///   lines make sum those extents over the strip's width (Cauchy-Crofton).
pub fn land_strips(
    grid: &crate::plates::Grid,
    elevation_m: &[f64],
    strips: usize,
) -> (Vec<f64>, Vec<f64>) {
    let k = strips.max(1);
    let n = grid.len().min(elevation_m.len());
    let strip_of = |d: [f64; 3]| {
        let lat = m::asin(d[2].clamp(-1.0, 1.0));
        (((lat / m::PI + 0.5) * k as f64) as usize).min(k - 1)
    };
    let mut cells = vec![0.0; k];
    let mut land = vec![0.0; k];
    let mut extent = vec![0.0; k];
    // Cell size on the unit sphere.
    let size = m::sqrt(4.0 * m::PI / n.max(1) as f64);
    for c in 0..n {
        let d = grid.dirs[c];
        let i = strip_of(d);
        cells[i] += 1.0;
        let up = elevation_m[c] > 0.0;
        if up {
            land[i] += 1.0;
        }
        for b in grid.neighbours(c) {
            // Each pair once.
            if b <= c || b >= n || (elevation_m[b] > 0.0) == up {
                continue;
            }
            let e = grid.dirs[b];
            let step = [e[0] - d[0], e[1] - d[1], e[2] - d[2]];
            let len = m::sqrt(step[0] * step[0] + step[1] * step[1] + step[2] * step[2]).max(1e-12);
            let rho = m::sqrt(d[0] * d[0] + d[1] * d[1]).max(1e-12);
            let east = (-d[1] * step[0] + d[0] * step[1]) / (rho * len);
            extent[i] += size * east.abs();
        }
    }
    let width = m::PI / k as f64;
    let frac = (0..k)
        .map(|i| {
            if cells[i] > 0.0 {
                land[i] / cells[i]
            } else {
                0.0
            }
        })
        .collect();
    let coasts = extent.iter().map(|x| x / width).collect();
    (frac, coasts)
}

/// W7c part 2: `x` with land and coasts by strip from a height map, and the
/// ocean fraction they imply, so the climate's sea is the map's sea.
pub fn with_land(
    x: &Extra,
    grid: &crate::plates::Grid,
    elevation_m: &[f64],
    strips: usize,
) -> Extra {
    let (land, coasts) = land_strips(grid, elevation_m, strips);
    let n = grid.len().min(elevation_m.len()).max(1);
    let sea = elevation_m.iter().take(n).filter(|h| **h <= 0.0).count() as f64 / n as f64;
    let mut y = x.clone();
    y.land_by_lat = Some(land);
    y.coasts_by_lat = Some(coasts);
    y.ocean_fraction = Some(sea);
    y
}

/// W7c part 2a: `x` with the world's land and coasts by latitude, read off
/// the Orbit-level (S16 guide) plate run's heights before wear, with sea
/// level for `oceans` Earth oceans. Every detail level, the survey and the
/// weather op read the same strips, so they keep one climate. `orbit_run`
/// may hand over an Orbit-level history already run (no second run). A dry
/// world, or one without a surface, gets `x` back unchanged.
pub fn land_extra(
    s: &SurfaceSettings,
    env: &SurfaceEnvelope,
    seed: &sse_core::draw::Seed,
    epoch_gyr: f64,
    oceans: f64,
    x: &Extra,
    orbit_run: Option<&crate::plates::PlateHistory>,
) -> Extra {
    if oceans <= 0.0 || env.kind == "cloud-deck" {
        return x.clone();
    }
    let os = crate::detail::orbit_settings(s);
    let own;
    let h = match orbit_run {
        Some(h) => h,
        None => {
            own = crate::plates::history(&os, env, seed, epoch_gyr);
            &own
        }
    };
    let raw = crate::wear::base_heights(&os, env, seed, h);
    match crate::plates::sea_level(&raw, env.reference_radius_m, oceans) {
        Some(sl) => {
            let elev: Vec<f64> = raw.iter().map(|z| z - sl).collect();
            let grid = crate::plates::grid_for(h);
            with_land(x, &grid, &elev, s.climate.bands as usize)
        }
        None => x.clone(),
    }
}

/// The climate of `b` at `epoch_myr`, with the given extra facts.
pub fn world_climate(
    s: &SurfaceSettings,
    b: &Body,
    env: &SurfaceEnvelope,
    x: &Extra,
    epoch_myr: f64,
) -> WorldClimate {
    let (c, assumed) = conditions(s, b, env, x);
    let air = Air::guess(&s.cover, env, b.pressure_bar, b.oceans > 0.0);
    WorldClimate {
        climate: sse_climate::orbit(&s.climate, &c, epoch_myr),
        lapse_k_km: air.lapse_k_km(&s.cover, env.gravity_m_s2.max(1e-6)),
        assumed,
        ice_ages: None,
    }
}

/// The climate of the epochs when the world had surface water, which is the
/// rain that wear reads. A world wet now with the same water is simply the
/// present climate. A world wet only in the past (early Mars) gets its water
/// back: liquid water at the history's volume and pressure, and a mean
/// temperature of at least `wet_floor_k`, because a sea that rains needs one
/// (a registered fudge until the atmosphere history publishes past
/// temperatures).
pub fn wet_climate(
    s: &SurfaceSettings,
    b: &Body,
    env: &SurfaceEnvelope,
    water: &crate::wear::WaterHistory,
    x: &Extra,
    epoch_myr: f64,
) -> WorldClimate {
    let wet_now =
        b.oceans > 0.0 && b.oceans == water.oceans && b.pressure_bar == water.pressure_bar;
    if wet_now || water.oceans <= 0.0 {
        return world_climate(s, b, env, x, epoch_myr);
    }
    let mut b2 = b.clone();
    b2.oceans = water.oceans;
    b2.pressure_bar = water.pressure_bar;
    let mut e2 = env.clone();
    e2.t_surface_now_k = env.t_surface_now_k.max(s.wear.wet_floor_k);
    let mut x2 = x.clone();
    if x2.liquid.is_none() {
        x2.liquid = Some("water".into());
    }
    let mut w = world_climate(s, &b2, &e2, &x2, epoch_myr);
    w.assumed.push(format!(
        "while wet: {} oceans under {} bar, at least {} K",
        water.oceans, water.pressure_bar, s.wear.wet_floor_k
    ));
    w
}

/// W9 part 2: the ice-age model of a world's climate under an orbit track:
/// the spin axis precessing from the body's tilt and perihelion now, with the
/// star's and moons' torques on its bulge.
pub fn ice_ages(
    s: &SurfaceSettings,
    b: &Body,
    w: &WorldClimate,
    track: &sse_climate::forcing::OrbitTrack,
    moons: &[sse_climate::forcing::MoonPull],
    epoch_myr: f64,
) -> sse_climate::forcing::IceAgeModel {
    use sse_climate::forcing as fc;
    let cs = &s.climate;
    let c = &w.climate.conditions;
    let (a_au, host) = b
        .orbit
        .as_ref()
        .map(|o| (o.a_au, o.host_msun))
        .unwrap_or((1.0, 1.0));
    let kb = fc::precession_parts(cs, c, fc::star_kg(host), a_au, moons);
    let f = &cs.forcing;
    let spin = fc::spin_history(
        track,
        c.obliquity_deg,
        c.perihelion_lon_deg,
        kb,
        f.step_yr,
        f.sample_yr,
    );
    fc::ice_age_model(cs, &w.climate, epoch_myr, spin, &track.source)
}

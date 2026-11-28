//! A giant's eddies, from the heat it sends out (track WS, package WS2; design
//! `handover/design/SSE4-stirred-clouds-design.md` sections 3.2 and 3.3).
//!
//! - **How fast**: the heat flowing through the weather layer feeds its eddies at
//!   ε = η F g / p₀ (`sse.weather.eddy_power`), and an eddy of width L runs at
//!   C (ε L)^(1/3) (`sse.weather.cascade_speed`, Kraichnan's cascade).
//! - **What sizes**: from the deformation radius up to the band's width, each
//!   doubling of size covering the pack's share of the band.
//! - **What fate**: the band's shear holds an eddy turning its way as a steady
//!   ellipse and tears one turning against it past Moore and Saffman's limit, in
//!   one shear time (`sse.weather.torn_by_shear`).
//! - **When**: drawn per window, keyed by the window's index, the band, the size
//!   and the world, as the storm families are, so any date gives the same eddies.
//!
//! Eddies are too many and too brief to name: they are not in the systems list
//! ("special bodies tracked; gravel is a sum"), only in the flow table.

use crate::conditions::Conditions;
use crate::giants::{
    deformation_m, draws, spin_sign, wrap, zonal_vorticity, zonal_wind, GiantLayout, GiantSettings,
    DEG,
};
use serde::{Deserialize, Serialize};
use sse_core::math as m;

const DAY_S: f64 = sse_core::sheet::DAY;
/// A bar in pascals (a unit).
const BAR_PA: f64 = 1e5;

/// What the shear does to a vortex (`sse.weather.torn_by_shear`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Shear {
    /// The steady shape's length along the wind over its width.
    pub aspect: f64,
    pub torn: bool,
    /// How long a torn vortex lasts, s (0 when it holds).
    pub life_s: f64,
}

/// Moore and Saffman's elliptical vortex in a band's shear, through the block in force. `zeta` is the band's
/// vorticity and `omega` the vortex's core vorticity, both in the same sense convention.
pub fn torn_by_shear(zeta: f64, omega: f64) -> Shear {
    if omega == 0.0 || !omega.is_finite() || !zeta.is_finite() {
        return Shear {
            aspect: 1.0,
            torn: false,
            life_s: 0.0,
        };
    }
    let v = sse_physics::run("sse.weather.torn_by_shear", &[zeta, omega])
        .expect("sse.weather.torn_by_shear takes two finite vorticities");
    Shear {
        aspect: v[0],
        torn: v[1] > 0.5,
        life_s: v[2],
    }
}

/// Heat the air sends out, W/m²: from inside plus the sunlight it absorbs, averaged over the sphere.
pub fn heat_out_w_m2(c: &Conditions) -> f64 {
    c.internal_flux_w_m2.max(0.0) + c.absorbed_w_m2().max(0.0)
}

/// Power into the eddies per kilogram of the weather layer, W/kg (`sse.weather.eddy_power`).
pub fn eddy_power(gs: &GiantSettings, c: &Conditions) -> f64 {
    let p0 = gs.stirring.weather_layer_bar * BAR_PA;
    sse_physics::run(
        "sse.weather.eddy_power",
        &[heat_out_w_m2(c), c.gravity_m_s2, p0],
    )
    .map(|v| v[0])
    .unwrap_or(0.0)
}

/// An eddy's speed at its edge, m/s, for its width in metres (`sse.weather.cascade_speed`).
pub fn eddy_speed(c: &Conditions, eps: f64, width_m: f64) -> f64 {
    if eps <= 0.0 || width_m <= 0.0 {
        return 0.0;
    }
    sse_physics::run(
        "sse.weather.cascade_speed",
        &[eps, width_m, c.t_surface_k, c.mean_molar_kg],
    )
    .map(|v| v[0])
    .unwrap_or(0.0)
}

/// One band of a hemisphere: band `k` (0 at the equator) spans sin(latitude) from k/jets to (k+1)/jets, so every
/// band holds the same area.
#[derive(Clone, Copy, Debug)]
pub struct Band {
    pub north: bool,
    pub k: u32,
    pub lat_lo_deg: f64,
    pub lat_hi_deg: f64,
}

impl Band {
    /// Its width north to south, m.
    pub fn width_m(&self, a_m: f64) -> f64 {
        (self.lat_hi_deg - self.lat_lo_deg).abs() / DEG * a_m
    }
    /// Its area, m².
    pub fn area_m2(&self, a_m: f64, jets: u32) -> f64 {
        2.0 * m::PI * a_m * a_m / jets.max(1) as f64
    }
    /// The latitude at its middle (by area), degrees, signed.
    pub fn mid_deg(&self) -> f64 {
        let s = (m::sin(self.lat_lo_deg / DEG) + m::sin(self.lat_hi_deg / DEG)) / 2.0;
        m::asin(s) * DEG
    }
}

/// Every band, north and south, equator first.
pub fn bands(l: &GiantLayout) -> Vec<Band> {
    let j = l.jets.max(1);
    let mut out = vec![];
    for north in [true, false] {
        let h = if north { 1.0 } else { -1.0 };
        for k in 0..j {
            let lo = m::asin(k as f64 / j as f64) * DEG;
            let hi = m::asin(((k + 1) as f64 / j as f64).min(1.0)) * DEG;
            out.push(Band {
                north,
                k,
                lat_lo_deg: h * lo,
                lat_hi_deg: h * hi,
            });
        }
    }
    out
}

/// The eddies' sizes in one band: one class per doubling from the deformation radius up to the band's width
/// (one class at the band's width when the deformation radius is wider), m, each [smallest, largest).
pub fn size_classes(gs: &GiantSettings, c: &Conditions, b: &Band) -> Vec<[f64; 2]> {
    let wide = b.width_m(c.radius_m);
    let ld = deformation_m(gs, c, b.mid_deg());
    if ld.is_nan() || wide.is_nan() || ld <= 0.0 || wide <= 0.0 {
        return vec![];
    }
    if ld >= wide {
        return vec![[wide, wide]];
    }
    let mut out = vec![];
    let mut lo = ld;
    while lo < wide && out.len() < 12 {
        let hi = (2.0 * lo).min(wide);
        out.push([lo, hi]);
        lo = hi;
    }
    out
}

/// One eddy (design 3.3). Its centre at a date is `lon_deg` at `born`, drifting with `drift_m_s`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Eddy {
    pub lat_deg: f64,
    /// Longitude when it is born, degrees.
    pub lon_deg: f64,
    /// Width, km (as a storm's `size_km`: the vortex's radius is half of it).
    pub size_km: f64,
    /// Peak wind of its Gaussian vortex, m/s (at its full strength).
    pub strength: f64,
    /// +1 cyclonic, −1 anticyclonic.
    pub sense: f64,
    /// Its steady shape in the band's shear, east-west over north-south.
    pub aspect: f64,
    /// Turning against the shear past the steady limit: it is drawn out and gone in one shear time.
    pub torn: bool,
    pub born: f64,
    pub ends: f64,
    /// The zonal wind at its centre, m/s eastward, which carries it.
    pub drift_m_s: f64,
}

impl Eddy {
    /// Where its centre is on a day, degrees longitude.
    pub fn lon_at(&self, a_m: f64, day: f64) -> f64 {
        let dl = self.drift_m_s * (day - self.born) * DAY_S
            / (a_m * m::cos(self.lat_deg / DEG).max(1e-6))
            * DEG;
        wrap(self.lon_deg + dl)
    }
    /// Its strength on a day: rising and falling as sin² over its life, so nothing pops.
    pub fn strength_at(&self, day: f64) -> f64 {
        if day < self.born || day >= self.ends || self.ends <= self.born {
            return 0.0;
        }
        let x = m::sin(m::PI * (day - self.born) / (self.ends - self.born));
        self.strength * x * x
    }
}

/// The eddies' bookkeeping for one world: the power the heat feeds them and their speed by size.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EddyFacts {
    /// Heat the air sends out, W/m².
    pub heat_out_w_m2: f64,
    /// Power into the eddies, W/kg.
    pub eddy_power_w_kg: f64,
}

pub fn facts(gs: &GiantSettings, c: &Conditions) -> EddyFacts {
    EddyFacts {
        heat_out_w_m2: heat_out_w_m2(c),
        eddy_power_w_kg: eddy_power(gs, c),
    }
}

/// The counter for (band, size class, window, slot) in the eddy stream.
fn ectr(band: u64, class: u64, window: i64, slot: u64) -> u64 {
    let w = (window.wrapping_add(1 << 35) as u64) & ((1u64 << 36) - 1);
    (band << 56) | ((class & 0xf) << 52) | (w << 16) | (slot & 0xffff)
}

/// Every eddy alive at some moment from `day0` to `day1` (days since the campaign epoch).
pub fn eddies_between(
    gs: &GiantSettings,
    c: &Conditions,
    l: &GiantLayout,
    day0: f64,
    day1: f64,
) -> Vec<Eddy> {
    let st = &gs.stirring;
    let eps = eddy_power(gs, c);
    let a = c.radius_m;
    let mut out = vec![];
    if eps <= 0.0 || st.eddy_area_share_per_octave <= 0.0 || st.eddy_life_turns <= 0.0 {
        return out;
    }
    let d = draws(c, "weather.giant_eddies.v1");
    let turn = spin_sign(l);
    for (bi, b) in bands(l).iter().enumerate() {
        let h = if b.north { 1.0 } else { -1.0 };
        for (ci, [lo, hi]) in size_classes(gs, c, b).into_iter().enumerate() {
            let mid = m::sqrt(lo * hi);
            let u_mid = eddy_speed(c, eps, mid);
            if u_mid <= 0.0 {
                continue;
            }
            // One turn of the class's middle eddy at its edge; a window is its life.
            let window = st.eddy_life_turns * m::PI * mid / u_mid / DAY_S;
            let alive =
                st.eddy_area_share_per_octave * b.area_m2(a, l.jets) / (m::PI * 0.25 * mid * mid);
            let first = m::floor(day0 / window) as i64 - 1;
            let last = m::floor(day1 / window) as i64;
            for wi in first..=last {
                // A whole number with the right mean: the count rounded up with the chance of its fraction.
                let n = (alive + d.unit(ectr(bi as u64, ci as u64, wi, 0))) as u64;
                for j in 0..n.min(0x1fff) {
                    let u = |k: u64| d.unit(ectr(bi as u64, ci as u64, wi, 1 + 8 * j + k));
                    let born = (wi as f64 + u(0)) * window;
                    if born > day1 || born + window < day0 {
                        continue;
                    }
                    let width = lo * m::exp(u(1) * m::ln(hi / lo));
                    let s_lo = m::sin(b.lat_lo_deg.abs() / DEG);
                    let s_hi = m::sin(b.lat_hi_deg.abs() / DEG);
                    let lat = h * m::asin(s_lo + (s_hi - s_lo) * u(2)) * DEG;
                    let lon0 = 360.0 * u(3) - 180.0;
                    let sense = if u(4) < 0.5 { 1.0 } else { -1.0 };
                    let strength = eddy_speed(c, eps, width);
                    let r = 0.5 * width;
                    let omega = sense * h * turn * 2.0 * m::exp(0.5) * strength / r;
                    let sh = torn_by_shear(zonal_vorticity(gs, l, a, lat), omega);
                    let life_d = if sh.torn {
                        (sh.life_s / DAY_S).min(window)
                    } else {
                        window
                    };
                    let ends = born + life_d;
                    if ends < day0 {
                        continue;
                    }
                    out.push(Eddy {
                        lat_deg: lat,
                        lon_deg: lon0,
                        size_km: width / 1000.0,
                        strength,
                        sense,
                        aspect: sh.aspect,
                        torn: sh.torn,
                        born,
                        ends,
                        drift_m_s: zonal_wind(gs, l, a, lat),
                    });
                }
            }
        }
    }
    out
}

/// A plume: a tall storm carrying heat from inside up to the cloud tops (WS3, design 3.4). Fresh bright cloud on
/// the top deck, half a deformation radius across, drifting with the wind.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Plume {
    pub lat_deg: f64,
    /// Longitude when it is born, degrees.
    pub lon_deg: f64,
    pub size_km: f64,
    pub born: f64,
    pub ends: f64,
    pub drift_m_s: f64,
}

impl Plume {
    pub fn lon_at(&self, a_m: f64, day: f64) -> f64 {
        let dl = self.drift_m_s * (day - self.born) * DAY_S
            / (a_m * m::cos(self.lat_deg / DEG).max(1e-6))
            * DEG;
        wrap(self.lon_deg + dl)
    }
    /// 0 to 1 over its life, rising and falling as sin².
    pub fn fade_at(&self, day: f64) -> f64 {
        if day < self.born || day >= self.ends || self.ends <= self.born {
            return 0.0;
        }
        let x = m::sin(m::PI * (day - self.born) / (self.ends - self.born));
        x * x
    }
}

/// Plumes alive at once (`sse.weather.plume_count`), and the deformation radius they are sized by (at 45°), m.
pub fn plume_count(gs: &GiantSettings, c: &Conditions) -> (f64, f64) {
    let ld = deformation_m(gs, c, 45.0);
    if c.internal_flux_w_m2 <= 0.0 || ld <= 0.0 {
        return (0.0, ld);
    }
    let n = sse_physics::run(
        "sse.weather.plume_count",
        &[c.internal_flux_w_m2, c.radius_m, ld],
    )
    .map(|v| v[0])
    .unwrap_or(0.0);
    (n, ld)
}

/// Every plume alive at some moment from `day0` to `day1`. Each lives one turn of an eddy its own size and rises
/// where its band's shear turns the way the spin does (cyclonic: where Jupiter's lightning is seen).
pub fn plumes_between(
    gs: &GiantSettings,
    c: &Conditions,
    l: &GiantLayout,
    day0: f64,
    day1: f64,
) -> Vec<Plume> {
    let (alive, ld) = plume_count(gs, c);
    let mut out = vec![];
    if alive <= 0.0 {
        return out;
    }
    let size = 0.5 * ld;
    let eps = eddy_power(gs, c);
    let u = eddy_speed(c, eps, size);
    if u <= 0.0 {
        return out;
    }
    let life = m::PI * size / u / DAY_S;
    let a = c.radius_m;
    let d = draws(c, "weather.giant_plumes.v1");
    let turn = spin_sign(l);
    let first = m::floor(day0 / life) as i64 - 1;
    let last = m::floor(day1 / life) as i64;
    for wi in first..=last {
        let n = (alive + d.unit(ectr(0, 0, wi, 0))) as u64;
        for j in 0..n.min(0x7ff) {
            let r = |k: u64| d.unit(ectr(0, 0, wi, 1 + 32 * j + k));
            let born = (wi as f64 + r(0)) * life;
            let ends = born + life;
            if born > day1 || ends < day0 {
                continue;
            }
            // Uniform by area until the shear is cyclonic there (up to eight tries).
            let Some(lat) = (0..8).find_map(|t| {
                let lat = m::asin(2.0 * r(2 + 2 * t) - 1.0) * DEG;
                let h = if lat >= 0.0 { 1.0 } else { -1.0 };
                (zonal_vorticity(gs, l, a, lat) * h * turn > 0.0).then_some(lat)
            }) else {
                continue;
            };
            out.push(Plume {
                lat_deg: lat,
                lon_deg: 360.0 * r(1) - 180.0,
                size_km: size / 1000.0,
                born,
                ends,
                drift_m_s: zonal_wind(gs, l, a, lat),
            });
        }
    }
    out
}

/// How long the top deck lasts before it is made anew, s. Two causes, added as rates
/// (`sse.weather.overturn_time`, WS3b, note SSE4-cloud-renewal-rising-air-note.md):
/// - its particles fall out (`sse.weather.particle_fall_time`): the pack's particle radius for the deck's
///   condensate (or the fallback), the data sheet's density of its solid, the air in the middle of the deck,
///   and the deck's thickness;
/// - air rising and sinking through it replaces it: the heat the plumes carry (`sse.weather.plume_count`),
///   carried as the latent heat of the deepest deck's condensate the climate knows, crossing the deck's own air mass.
pub fn renewal_s(
    gs: &GiantSettings,
    cs: &sse_climate::settings::ClimateSettings,
    c: &Conditions,
    l: &GiantLayout,
) -> Option<f64> {
    let fall = fall_out_s(gs, cs, c, l)?;
    // The deepest deck whose condensate the climate knows (an ice giant's ammonium hydrosulphide is not one).
    let Some(deep) = l.layers.iter().find_map(|d| {
        let name = d.trim_end_matches(" ice");
        cs.substances.iter().find(|s| s.name == name)
    }) else {
        return Some(fall);
    };
    let x = c
        .gases
        .get(&deep.gas)
        .copied()
        .filter(|x| *x > 0.0)
        .unwrap_or(gs.stirring.fallback_deep_mole_fraction);
    let dh = gs.clouds.heights.as_ref()?;
    let base = crate::giants::deck_base_bar(dh, cs, c, &l.layers)[2];
    sse_physics::run(
        "sse.weather.overturn_time",
        &[
            convected_flux(gs, c),
            deep.latent_j_kg,
            x,
            deep.molar_kg,
            c.mean_molar_kg,
            base * BAR_PA,
            dh.thickness_h,
            c.gravity_m_s2,
            fall,
        ],
    )
    .ok()
    .map(|v| v[1])
}

/// The heat a giant's plumes carry up, averaged over the sphere, W/m² (`sse.weather.plume_count`).
pub fn convected_flux(gs: &GiantSettings, c: &Conditions) -> f64 {
    let ld = deformation_m(gs, c, 45.0);
    if c.internal_flux_w_m2 <= 0.0 || ld <= 0.0 {
        return 0.0;
    }
    sse_physics::run(
        "sse.weather.plume_count",
        &[c.internal_flux_w_m2, c.radius_m, ld],
    )
    .map(|v| v[1])
    .unwrap_or(0.0)
}

/// How long the top deck's particles take to fall through it, s (`sse.weather.particle_fall_time`).
pub fn fall_out_s(
    gs: &GiantSettings,
    cs: &sse_climate::settings::ClimateSettings,
    c: &Conditions,
    l: &GiantLayout,
) -> Option<f64> {
    let cl = &gs.clouds;
    let deck = &l.layers[2];
    let r_um = cl
        .particle_radius_um
        .get(deck)
        .copied()
        .filter(|r| *r > 0.0)
        .unwrap_or(gs.stirring.fallback_particle_um);
    let name = deck.trim_end_matches(" ice");
    let gas = cs.substances.iter().find(|s| s.name == name)?.gas.clone();
    // The sheet in force, or the shipped one outside a call.
    static BASE: std::sync::OnceLock<sse_reference::Reference> = std::sync::OnceLock::new();
    let phys = sse_physics::now();
    let sheet: &sse_reference::Reference = match &phys {
        Some(p) => p.sheet(),
        None => BASE.get_or_init(sse_reference::Reference::base),
    };
    let rho = sheet
        .row("substances", &gas)?
        .get("phases")?
        .get("density_solid")?
        .get("v")?
        .as_f64()?;
    let dh = cl.heights.as_ref()?;
    let base = crate::giants::deck_base_bar(dh, cs, c, &l.layers)[2];
    let mid = base * m::exp(-0.5 * dh.thickness_h);
    let t = c.t_surface_k * m::pow(mid.max(dh.tropopause_bar), crate::giants::kappa(c));
    let dz = dh.thickness_h * c.scale_height_m();
    sse_physics::run(
        "sse.weather.particle_fall_time",
        &[
            r_um * 1e-6,
            rho,
            c.gravity_m_s2,
            t,
            c.mean_molar_kg,
            mid * BAR_PA,
            dz,
        ],
    )
    .ok()
    .map(|v| v[1])
}

/// The longest span one flow table covers, days.
pub const MAX_SPAN_DAYS: f64 = 60.0;
/// The jets' table step, degrees of latitude.
pub const JET_STEP_DEG: f64 = 0.25;

/// A pole's circumpolar jet, as the winds need it (WS1's stream function).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FlowPole {
    pub north: bool,
    /// Sides of its polygon, 0 for a circle.
    pub polygon_sides: u32,
    pub orient_lon_deg: f64,
}

/// Everything that moves a giant's cloud over a span of dates, as one table (design section 6, `want: "flow"`):
/// the jets by latitude, the polar jet's shape, every storm and every eddy alive in the span. The winds at a
/// place are the jets' (`giants::giant_cell`'s stream function) plus each vortex's Gaussian, written in its
/// ellipse.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FlowTable {
    pub day0: f64,
    pub day1: f64,
    pub radius_m: f64,
    /// Spins backwards: every wind and vortex is mirrored.
    pub retrograde: bool,
    /// Latitudes, degrees, south to north at `JET_STEP_DEG`, and the eastward wind there, m/s.
    pub jet_lat_deg: Vec<f64>,
    pub jet_u_m_s: Vec<f64>,
    pub polar_jet_lat_deg: f64,
    pub polar_jet_m_s: f64,
    /// The polar jet's width, km (its deformation radius).
    pub jet_deformation_km: f64,
    pub poles: Vec<FlowPole>,
    /// Storms alive on `day0`, as the systems read gives them (with their aspect).
    pub systems: Vec<crate::contract::WeatherSystem>,
    pub eddies: Vec<Eddy>,
    /// Plumes alive in the span (WS3).
    pub plumes: Vec<Plume>,
    /// How long cloud lasts before it falls out and is made anew, days (WS3); 0 where it cannot be worked out.
    pub renewal_days: f64,
    pub facts: EddyFacts,
    pub why: String,
}

/// The flow table from `day` over `span_days` (0 to `MAX_SPAN_DAYS`).
pub fn flow_table(
    gs: &GiantSettings,
    cs: &sse_climate::settings::ClimateSettings,
    c: &Conditions,
    day: f64,
    span_days: f64,
) -> FlowTable {
    let l = crate::giants::layout(gs, c);
    let a = c.radius_m;
    let span = span_days.clamp(0.0, MAX_SPAN_DAYS);
    let n = (180.0 / JET_STEP_DEG) as usize + 1;
    let jet_lat_deg: Vec<f64> = (0..n).map(|k| -90.0 + JET_STEP_DEG * k as f64).collect();
    let jet_u_m_s = jet_lat_deg
        .iter()
        .map(|&p| zonal_wind(gs, &l, a, p))
        .collect();
    let systems = crate::giants::giant_systems(gs, c, crate::contract::Date { day })
        .into_iter()
        .collect();
    let eddies = eddies_between(gs, c, &l, day, day + span);
    let plumes = plumes_between(gs, c, &l, day, day + span);
    let renewal_days = renewal_s(gs, cs, c, &l).map(|s| s / DAY_S).unwrap_or(0.0);
    let f = facts(gs, c);
    let why = format!(
        "{} eddies from {:.1} W/m² of heat ({:.2e} W/kg into the weather layer down to {} bar), sized from the deformation radius to the band's width; storms from the systems read; jets from the bands.",
        eddies.len(),
        f.heat_out_w_m2,
        f.eddy_power_w_kg,
        gs.stirring.weather_layer_bar
    );
    FlowTable {
        day0: day,
        day1: day + span,
        radius_m: a,
        retrograde: l.retrograde,
        jet_lat_deg,
        jet_u_m_s,
        polar_jet_lat_deg: l.polar_jet_lat_deg,
        polar_jet_m_s: spin_sign(&l) * l.polar_jet_m_s,
        jet_deformation_km: l.jet_deformation_km,
        poles: l
            .poles
            .iter()
            .map(|q| FlowPole {
                north: q.north,
                polygon_sides: q.polygon_sides,
                orient_lon_deg: q.orient_lon_deg,
            })
            .collect(),
        systems,
        eddies,
        plumes,
        renewal_days,
        facts: f,
        why,
    }
}

#[cfg(test)]
#[path = "eddies_tests.rs"]
mod tests;

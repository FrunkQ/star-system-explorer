//! S6: the other regimes' surface grammars (surface design sections 2.1 and 6).
//!
//! Each regime writes its own landforms, layered over the S0 regime sketch
//! the same way S4's craters are:
//! - ice shells: double ridges, bands and chaos terrain (Europa; Ganymede's
//!   older grooved bands);
//! - heat pipes: paterae and isolated block mountains (Io);
//! - episodic and plume lids: coronae, rift belts (chasmata), tessera
//!   plateaus, and dated global overturns (Venus);
//! - stagnant lids: contraction scarps from cooling, and rift systems beside
//!   the tallest S4 shield (Mercury, Mars);
//! - the hydrocarbon cycle: equatorial dune fields and polar lakes and seas,
//!   mostly in the north (Titan);
//! - carbon-rich worlds: tar seas (speculative, labelled);
//! - giants: a cloud deck with a jet count from the Rhines scale, winds,
//!   long-lived storms, cloud colour by temperature and cloud-top heights.
//!
//! Small bodies (lumpy shapes), ocean worlds (sea floor only) and lava worlds
//! (molten day side) are written by the S0 sketch itself; S6 checks them.
//!
//! Dates: on renewed surfaces (a regime with an S4 `resurface` entry) the
//! landforms are a steady state: their count is the density times
//! (1 − e^(−Z)), Z the integrated renewal, and each date is drawn so older
//! ones are thinned by later renewal, the same rule as S4's craters. So
//! Europa's ridges are all young and Ganymede's bands are old.
//!
//! Every landform is a smooth profile (C1, bounded curvature) across its
//! width, so the atlas band-limits it like a crater: it appears at a level
//! only when four samples cross its width.

use crate::impacts::{CraterField, Features};
use crate::sketch::dot;
use crate::{SurfaceEnvelope, SurfaceSettings};
use serde::{Deserialize, Serialize};
use sse_core::draw::{Draws, Seed, Stream};
use sse_core::math as m;
use std::collections::BTreeMap;

pub const REGIMES_VERSION: &str = "surface-regimes-v1";

/// One kind of landform a regime writes: how many, how big, how tall.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct LandformRule {
    pub kind: String,
    /// Per million km² at full activity (or a covered area fraction when
    /// `cover_fraction` is set).
    #[serde(default)]
    pub per_mkm2: f64,
    #[serde(default)]
    pub cover_fraction: Option<f64>,
    /// Size range (diameter, or length for linear landforms), km, log-uniform.
    pub size_km: [f64; 2],
    /// Width over length for linear landforms (absent: round).
    #[serde(default)]
    pub width_ratio: Option<f64>,
    /// Height range, m (negative for depressions), log-uniform in magnitude.
    pub height_m: [f64; 2],
    /// Height scales with the world's radius against this (contraction), km.
    #[serde(default)]
    pub radius_ref_km: Option<f64>,
    /// Latitude band the landform sits in, degrees (|lat| from, to).
    #[serde(default)]
    pub lat_deg: Option<[f64; 2]>,
    /// Share placed in the northern hemisphere (0.5: no preference).
    #[serde(default = "half")]
    pub north_share: f64,
    #[serde(default)]
    pub cover: Option<String>,
    #[serde(default)]
    pub material: Option<String>,
    /// Activity follows heat flow as (q / q_ref)^3 at the date (renewal-weighted
    /// dates), or the world's cooling (contraction), or nothing (uniform).
    #[serde(default)]
    pub driver: String,
    pub source: String,
}

fn half() -> f64 {
    0.5
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct CloudSettings {
    /// Zonal wind scale for the Rhines scale, m/s: gas giants and ice giants
    /// (below `ice_giant_mass_earth`).
    pub wind_gas_m_s: f64,
    pub wind_ice_m_s: f64,
    pub ice_giant_mass_earth: f64,
    pub storms_per_jet: f64,
    /// Cloud chemistry by temperature: (below K, zone albedo, belt albedo).
    pub chemistry: Vec<(f64, f64, f64, String)>,
    pub mean_molar_kg: f64,
    pub source: String,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct RegimeSettings {
    /// Landform rules by regime.
    pub landforms: BTreeMap<String, Vec<LandformRule>>,
    pub clouds: CloudSettings,
    /// Default rotation when a world's is not known, hours.
    pub rotation_hours_default: f64,
    pub max_per_kind: u32,
}

/// One landform: round (`axis` none) or a great-circle segment along `axis`.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Landform {
    pub id: String,
    pub kind: String,
    pub dir: [f64; 3],
    /// Unit tangent along a linear landform at its centre.
    #[serde(default)]
    pub axis: Option<[f64; 3]>,
    pub lat_deg: f64,
    pub lon_deg: f64,
    /// Diameter of a round landform, length of a linear one, km.
    pub size_km: f64,
    pub width_km: f64,
    pub height_m: f64,
    pub formed_myr: f64,
    #[serde(default)]
    pub cover: Option<String>,
    #[serde(default)]
    pub material: Option<String>,
    pub cause: String,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Storm {
    pub id: String,
    pub lat_deg: f64,
    pub lon_deg: f64,
    pub size_km: f64,
    /// Brighter (+) or darker (−) than its band.
    pub albedo_shift: f64,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Jet {
    /// Latitude of the jet's centre, degrees.
    pub lat_deg: f64,
    pub wind_m_s: f64,
}

/// A giant's cloud deck: zones and belts between alternating jets.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct CloudDeck {
    pub rotation_hours: f64,
    pub wind_scale_m_s: f64,
    /// Prograde jets pole to pole: sqrt(a Ω / U) (the Rhines scale).
    pub jets: u32,
    pub band_edges_deg: Vec<f64>,
    pub winds: Vec<Jet>,
    pub storms: Vec<Storm>,
    pub chemistry: String,
    pub zone_albedo: f64,
    pub belt_albedo: f64,
    pub scale_height_km: f64,
    pub cause: String,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Overturn {
    pub t_myr: f64,
    pub text: String,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct RegimeSurface {
    pub address: String,
    pub regime: String,
    pub epoch_myr: f64,
    pub landforms: Vec<Landform>,
    pub overturns: Vec<Overturn>,
    pub clouds: Option<CloudDeck>,
    /// What the S0 sketch itself writes for this regime, or what is not modelled.
    pub notes: Vec<String>,
    pub speculative: bool,
}

/// What the regime grammars read beyond the envelope.
#[derive(Clone, Debug, Default)]
pub struct RegimeInputs<'a> {
    pub rotation_hours: Option<f64>,
    pub mass_earth: f64,
    /// S4's features: rifts open beside the tallest shield.
    pub impacts: Option<&'a Features>,
    /// S4's crater field: later craters garden away fossil landforms.
    pub craters: Option<&'a CraterField>,
}

fn unit_dir(d: &Draws, i: u64) -> [f64; 3] {
    let v = [d.normal(i), d.normal(i + 1), d.normal(i + 2)];
    let l = m::sqrt(v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).max(1e-300);
    [v[0] / l, v[1] / l, v[2] / l]
}

fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn norm(a: [f64; 3]) -> [f64; 3] {
    let l = m::sqrt(a[0] * a[0] + a[1] * a[1] + a[2] * a[2]).max(1e-300);
    [a[0] / l, a[1] / l, a[2] / l]
}

fn lat_lon(v: [f64; 3]) -> (f64, f64) {
    (
        m::asin(v[2].clamp(-1.0, 1.0)).to_degrees(),
        m::atan2(v[1], v[0]).to_degrees(),
    )
}

fn from_lat_lon(lat_deg: f64, lon_deg: f64) -> [f64; 3] {
    let (la, lo) = (lat_deg.to_radians(), lon_deg.to_radians());
    [m::cos(la) * m::cos(lo), m::cos(la) * m::sin(lo), m::sin(la)]
}

fn poisson(d: &Draws, i: u64, lam: f64) -> u64 {
    if lam <= 0.0 {
        return 0;
    }
    if lam < 30.0 {
        let u = d.unit(i);
        let mut p = m::exp(-lam);
        let mut c = p;
        let mut k = 0u64;
        while u > c && k < 400 {
            k += 1;
            p *= lam / k as f64;
            c += p;
        }
        k
    } else {
        m::round(lam + m::sqrt(lam) * d.normal(i)).max(0.0) as u64
    }
}

fn log_uniform(lo: f64, hi: f64, u: f64) -> f64 {
    let (a, b) = (lo.abs().max(1e-12), hi.abs().max(1e-12));
    a * m::pow(b / a, u)
}

struct Clock<'a> {
    spans: Vec<(f64, f64, String, f64, f64)>,
    s: &'a SurfaceSettings,
    epoch: f64,
    t_from: f64,
    /// Integrated renewal at `CLOCK_STEPS` + 1 even dates from `t_from` to
    /// `epoch` (worked out once; every landform's date reads it).
    table: Vec<f64>,
}

/// Steps of the renewal integral from the surface's start to the epoch.
const CLOCK_STEPS: usize = 1024;

impl Clock<'_> {
    fn heat_at(&self, t: f64) -> f64 {
        match self.spans.iter().find(|x| t >= x.0 && t <= x.1) {
            Some(x) if x.1 > x.0 => x.3 + (x.4 - x.3) * (t - x.0) / (x.1 - x.0),
            Some(x) => x.3,
            None => 0.0,
        }
    }

    fn regime_at(&self, t: f64) -> &str {
        self.spans
            .iter()
            .find(|x| t >= x.0 && t <= x.1)
            .or(self.spans.last())
            .map(|x| x.2.as_str())
            .unwrap_or("")
    }

    /// Renewal rate at t (S4's rule), per Myr.
    fn rate(&self, tt: f64) -> f64 {
        match self.s.impacts.resurface.get(self.regime_at(tt)) {
            Some(r) => {
                let q = self.heat_at(tt);
                m::pow(
                    (q / r.q_ref_w_m2.max(1e-12)).max(0.0),
                    self.s.impacts.resurface_exponent,
                ) / r.myr.max(1e-9)
            }
            None => 0.0,
        }
    }

    fn tabulate(&mut self) {
        let dt = (self.epoch - self.t_from).max(0.0) / CLOCK_STEPS as f64;
        let mut z = 0.0;
        let mut table = Vec::with_capacity(CLOCK_STEPS + 1);
        table.push(0.0);
        for k in 0..CLOCK_STEPS {
            z += dt * self.rate(self.t_from + (k as f64 + 0.5) * dt);
            table.push(z);
        }
        self.table = table;
    }

    /// Integrated renewal between two dates, any regime (for fossils).
    fn renewal_between(&self, a: f64, b: f64) -> f64 {
        if b <= a {
            return 0.0;
        }
        let steps = m::ceil((b - a) / 25.0).max(1.0) as u32;
        let dt = (b - a) / steps as f64;
        (0..steps)
            .map(|k| dt * self.rate(a + (k as f64 + 0.5) * dt))
            .sum()
    }

    /// Integrated renewal from the surface's start to the epoch.
    fn z_total(&self) -> f64 {
        self.table.last().copied().unwrap_or(0.0)
    }

    /// A date for a landform: renewal-weighted (steady state) when the regime
    /// renews its surface, else by the driver.
    fn date(&self, driver: &str, ztot: f64, u: f64) -> f64 {
        let (lo, hi) = (self.t_from, self.epoch);
        if ztot > 0.0 && (driver == "heat" || driver == "heat-now") {
            let e = m::exp(-ztot);
            let target = ztot + m::ln((u * (1.0 - e) + e).max(1e-300));
            // Invert the tabulated integral, linear within a step.
            let k = self
                .table
                .partition_point(|z| *z < target)
                .clamp(1, CLOCK_STEPS);
            let (za, zb) = (self.table[k - 1], self.table[k]);
            let f = if zb > za {
                ((target - za) / (zb - za)).clamp(0.0, 1.0)
            } else {
                0.5
            };
            let dt = (hi - lo) / CLOCK_STEPS as f64;
            return lo + dt * ((k - 1) as f64 + f);
        }
        lo + (hi - lo) * u
    }
}

/// One regime's landforms over the clock's span (its start to its end).
#[allow(clippy::too_many_arguments)]
fn grow(
    s: &SurfaceSettings,
    d: &Draws,
    clock: &Clock,
    regime: &str,
    epoch_now: f64,
    radius_km: f64,
    area_mkm2: f64,
    base_hi: u64,
    id_prefix: &str,
    out: &mut Vec<Landform>,
) {
    let rs = &s.regime_features;
    let epoch = clock.epoch;
    let ztot = clock.z_total();
    let q_now = clock.heat_at(epoch);
    let q0 = clock.heat_at(clock.t_from);
    let rules = rs.landforms.get(regime).cloned().unwrap_or_default();
    for (ri, rule) in rules.iter().enumerate() {
        let base = base_hi | ((ri as u64 + 1) << 40);
        // How much of the landform the world holds now.
        let activity = match rule.driver.as_str() {
            "heat" if ztot > 0.0 => 1.0 - m::exp(-ztot),
            "heat" | "heat-now" => {
                let r = s.impacts.resurface.get(regime);
                let q_ref = r.map(|x| x.q_ref_w_m2).unwrap_or(0.05);
                m::pow((q_now / q_ref).max(0.0), 3.0).min(1.0)
            }
            "cooling" => ((q0 - q_now) / q0.max(1e-12)).clamp(0.0, 1.0),
            _ => 1.0,
        };
        if activity <= 0.0 {
            continue;
        }
        let r_scale = rule.radius_ref_km.map(|x| radius_km / x).unwrap_or(1.0);
        let band_area_frac = rule
            .lat_deg
            .map(|[a, b]| (m::sin(b.to_radians()) - m::sin(a.to_radians())).abs())
            .unwrap_or(1.0);
        let mean_area = |lo: f64, hi: f64| {
            // Mean of π (D/2)² over log-uniform D, km² (round landforms).
            m::PI / 4.0 * (hi * hi - lo * lo) / (2.0 * m::ln(hi / lo).max(1e-9))
        };
        let count = match rule.cover_fraction {
            Some(f) => {
                let a = f * activity * area_mkm2 * 1e6;
                poisson(d, base, a / mean_area(rule.size_km[0], rule.size_km[1]))
            }
            None => poisson(
                d,
                base,
                rule.per_mkm2 * activity * area_mkm2 * band_area_frac,
            ),
        }
        .min(rs.max_per_kind as u64);
        for j in 0..count {
            let k = |x: u64| base + 16 * (j + 1) + x;
            let size = log_uniform(rule.size_km[0], rule.size_km[1], d.unit(k(0)));
            let width = rule.width_ratio.map(|w| w * size).unwrap_or(size);
            let mag = log_uniform(rule.height_m[0], rule.height_m[1], d.unit(k(1))) * r_scale;
            let height = if rule.height_m[0] < 0.0 { -mag } else { mag };
            let dir = match rule.lat_deg {
                Some([a, b]) => {
                    let (sa, sb) = (m::sin(a.to_radians()), m::sin(b.to_radians()));
                    let z = sa + (sb - sa) * d.unit(k(2));
                    let lat = m::asin(z).to_degrees();
                    let lat = if d.unit(k(3)) < rule.north_share {
                        lat
                    } else {
                        -lat
                    };
                    from_lat_lon(lat, 360.0 * d.unit(k(4)) - 180.0)
                }
                None => unit_dir(d, k(5)),
            };
            let axis = rule.width_ratio.map(|_| {
                // Ice-shell lineae trend about 45° from east (tidal stress);
                // other linear landforms any way.
                let east = norm(cross([0.0, 0.0, 1.0], dir));
                let north = cross(dir, east);
                let az = if regime == "ice-shell" {
                    let sgn = if d.unit(k(6)) < 0.5 { 1.0 } else { -1.0 };
                    sgn * 45f64.to_radians() + 0.26 * d.normal(k(7))
                } else {
                    m::PI * d.unit(k(6))
                };
                norm([
                    m::cos(az) * east[0] + m::sin(az) * north[0],
                    m::cos(az) * east[1] + m::sin(az) * north[1],
                    m::cos(az) * east[2] + m::sin(az) * north[2],
                ])
            });
            let t = clock.date(&rule.driver, ztot, d.unit(k(8)));
            let (lat, lon) = lat_lon(dir);
            let cause = format!(
                "{} {:.2} Gyr ago: {} Heat flow then {:.3} W/m² under a {}.",
                rule.kind.replace('-', " "),
                (epoch_now - t) / 1000.0,
                rule.source,
                clock.heat_at(t),
                regime.replace('-', " ")
            );
            out.push(Landform {
                id: format!("{}{}:{}", id_prefix, rule.kind, j),
                kind: rule.kind.clone(),
                dir,
                axis,
                lat_deg: lat,
                lon_deg: lon,
                size_km: size,
                width_km: width,
                height_m: height,
                formed_myr: t,
                cover: rule.cover.clone(),
                material: rule.material.clone(),
                cause,
            });
        }
    }
}

/// The regime's landforms (and a giant's cloud deck) at `epoch_gyr`.
pub fn regime_surface(
    s: &SurfaceSettings,
    env: &SurfaceEnvelope,
    seed: &Seed,
    inputs: &RegimeInputs,
    epoch_gyr: f64,
) -> RegimeSurface {
    let rs = &s.regime_features;
    let d = Draws::new(
        *seed,
        env.address.clone(),
        Stream::new("surface.regimes.v1").expect("valid"),
    );
    let end = env.timeline.last().map(|x| x.to_gyr).unwrap_or(0.0);
    let epoch = epoch_gyr.clamp(0.0, end) * 1000.0;
    let spans: Vec<(f64, f64, String, f64, f64)> = env
        .timeline
        .iter()
        .map(|x| {
            (
                x.from_gyr * 1000.0,
                x.to_gyr * 1000.0,
                x.regime.clone(),
                x.heat_flow_w_m2[0],
                x.heat_flow_w_m2[1],
            )
        })
        .collect();
    let regime = spans
        .iter()
        .find(|x| epoch >= x.0 && epoch <= x.1)
        .or(spans.last())
        .map(|x| x.2.clone())
        .unwrap_or_else(|| env.regime_now.clone());
    let t_from = spans
        .iter()
        .filter(|x| x.0 <= epoch && x.2 != regime)
        .map(|x| x.1.min(epoch))
        .fold(0.0, f64::max);
    let mut clock = Clock {
        spans,
        s,
        epoch,
        t_from,
        table: vec![],
    };
    clock.tabulate();
    let def = s.regimes.get(&regime);
    let mut out = RegimeSurface {
        address: env.address.to_string(),
        regime: regime.clone(),
        epoch_myr: epoch,
        landforms: vec![],
        overturns: vec![],
        clouds: None,
        notes: vec![],
        speculative: def.is_some_and(|x| x.speculative),
    };
    let radius_km = env.reference_radius_m / 1000.0;
    let area_mkm2 = 4.0 * m::PI * radius_km * radius_km / 1e6;
    match def.map(|x| x.style.as_str()) {
        Some("lumpy") => out
            .notes
            .push("small body: the sketch's lumpy shape is the surface; S4 craters it".into()),
        Some("ocean") => out
            .notes
            .push("global ocean: only sea-floor relief, no land (the sketch)".into()),
        Some("smooth") => out
            .notes
            .push("lava: molten day side, crust on the night side (the sketch)".into()),
        Some("plates") => out.notes.push("plates: S2".into()),
        _ => {}
    }
    if def.is_some_and(|x| x.style == "bands") {
        out.clouds = Some(cloud_deck(s, env, &d, inputs, epoch));
        return out;
    }
    let ztot = clock.z_total();
    grow(
        s,
        &d,
        &clock,
        &regime,
        epoch,
        radius_km,
        area_mkm2,
        0,
        "",
        &mut out.landforms,
    );
    // Fossil landforms: what earlier regimes built (Ganymede's grooved
    // terrain from its resonance passage) stays unless a later regime
    // renews the surface over it.
    let mut past: Vec<(f64, f64, String)> = vec![];
    for x in clock
        .spans
        .iter()
        .filter(|x| x.1 <= t_from + 1e-9 && x.2 != regime)
    {
        match past.last_mut() {
            Some(p) if p.2 == x.2 && (p.1 - x.0).abs() < 1e-9 => p.1 = x.1,
            _ => past.push((x.0, x.1, x.2.clone())),
        }
    }
    for (pi, (a, b, r)) in past.iter().enumerate() {
        if !rs.landforms.contains_key(r) {
            continue;
        }
        let mut c = Clock {
            spans: clock.spans.clone(),
            s,
            epoch: *b,
            t_from: *a,
            table: vec![],
        };
        c.tabulate();
        let mut fossils = vec![];
        let tag = format!("{r}@{a:.0}:");
        grow(
            s,
            &d,
            &c,
            r,
            epoch,
            radius_km,
            area_mkm2,
            (pi as u64 + 1) << 52,
            &tag,
            &mut fossils,
        );
        for (j, mut l) in fossils.into_iter().enumerate() {
            // Survives the renewal of everything after it, and the craters
            // that came after it: a landform is erased like a crater of its
            // own width under saturation (Gault 1970; Hartmann 1984), so a
            // fossil from the heavy bombardment is cratered away and one
            // from a later, quieter era is kept.
            let z = clock.renewal_between(l.formed_myr, epoch);
            let kept = inputs
                .craters
                .map(|f| f.erasure_survival(l.width_km, l.formed_myr, epoch))
                .unwrap_or(1.0);
            if d.unit(((pi as u64 + 1) << 52) | (1 << 39) | j as u64) < m::exp(-z) * kept {
                l.cause = format!(
                    "{} Fossil: no later regime has renewed it, and the craters since have not erased it.",
                    l.cause
                );
                out.landforms.push(l);
            }
        }
    }
    // Rift systems open beside the tallest shield (Valles Marineris beside Tharsis).
    if regime == "stagnant-lid" {
        if let Some(sh) = inputs.impacts.and_then(|f| {
            f.shields
                .iter()
                .max_by(|a, b| a.height_m.total_cmp(&b.height_m))
        }) {
            if sh.height_m > 10_000.0 {
                let east = norm(cross([0.0, 0.0, 1.0], sh.dir));
                let off = 1.5 * sh.radius_km / radius_km;
                let c = norm([
                    sh.dir[0] + off * east[0],
                    sh.dir[1] + off * east[1],
                    sh.dir[2] + off * east[2],
                ]);
                let length = (0.6 * radius_km * (0.8 + 0.4 * d.unit(7_000))).min(4000.0);
                let (lat, lon) = lat_lon(c);
                out.landforms.push(Landform {
                    id: "rift:0".into(),
                    kind: "rift".into(),
                    dir: c,
                    axis: Some(norm(cross(c, cross(east, c)))),
                    lat_deg: lat,
                    lon_deg: lon,
                    size_km: length,
                    width_km: 0.04 * length,
                    height_m: -(3000.0 + 4000.0 * d.unit(7_001)) * (3.71 / env.gravity_m_s2.max(0.1)).min(2.0),
                    formed_myr: sh.from_myr + 0.25 * (sh.to_myr - sh.from_myr),
                    cover: None,
                    material: None,
                    cause: format!(
                        "A rift system {:.2} Gyr ago, split open by the load and uplift of {} beside it (as Valles Marineris beside Tharsis).",
                        (epoch - (sh.from_myr + 0.25 * (sh.to_myr - sh.from_myr))) / 1000.0,
                        sh.id
                    ),
                });
            }
        }
    }
    // Global overturns on an episodic lid: Poisson in the integrated renewal.
    if regime == "episodic-lid" && ztot > 0.0 {
        let n = poisson(&d, 8_000, ztot).min(64);
        let mut ts: Vec<f64> = (0..n)
            .map(|i| clock.date("heat", ztot, d.unit(8_001 + i)))
            .collect();
        ts.sort_by(|a, b| a.total_cmp(b));
        for t in ts {
            out.overturns.push(Overturn {
                t_myr: t,
                text: format!(
                    "Global overturn {:.2} Gyr ago: the lid founders and lava resurfaces the world (heat flow {:.3} W/m²).",
                    (epoch - t) / 1000.0,
                    clock.heat_at(t)
                ),
            });
        }
    }
    if out.speculative {
        out.notes
            .push(format!("{regime}: speculative; landforms labelled so"));
    }
    out
}

fn cloud_deck(
    s: &SurfaceSettings,
    env: &SurfaceEnvelope,
    d: &Draws,
    inputs: &RegimeInputs,
    _epoch: f64,
) -> CloudDeck {
    let c = &s.regime_features.clouds;
    let rot = inputs
        .rotation_hours
        .unwrap_or(s.regime_features.rotation_hours_default)
        .max(0.1);
    let omega = m::TAU / (rot * 3600.0);
    let a = env.reference_radius_m;
    let ice = inputs.mass_earth > 0.0 && inputs.mass_earth < c.ice_giant_mass_earth;
    let u = if ice { c.wind_ice_m_s } else { c.wind_gas_m_s };
    // The one jet count and band layout (sse-climate, W7a).
    let jets = sse_climate::circulation::giant_jets(a, omega, u);
    let nb = 2 * jets;
    let edges = sse_climate::circulation::giant_band_edges(jets);
    let winds: Vec<Jet> = (0..nb)
        .map(|i| {
            let lat = 0.5 * (edges[i as usize] + edges[i as usize + 1]);
            // Prograde at the equator, alternating, weakening poleward.
            // Bands counted from the equator, mirrored north and south.
            let k = if i >= jets { i - jets } else { jets - 1 - i };
            let sign = if k.is_multiple_of(2) { 1.0 } else { -0.5 };
            Jet {
                lat_deg: lat,
                wind_m_s: sign * u * m::cos(lat.to_radians()),
            }
        })
        .collect();
    let t = env.t_surface_now_k;
    let chem = c
        .chemistry
        .iter()
        .find(|x| t < x.0)
        .or(c.chemistry.last())
        .cloned()
        .unwrap_or((1e9, 0.5, 0.5, "unknown".into()));
    let (zone, belt) = if ice { (0.32, 0.28) } else { (chem.1, chem.2) };
    let ns = poisson(d, 9_000, c.storms_per_jet * jets as f64).min(64);
    let band_km = |i: usize| (edges[i + 1] - edges[i]).to_radians() * a / 1000.0;
    let storms = (0..ns)
        .map(|j| {
            let bi = (d.unit(9_001 + 4 * j) * nb as f64) as usize % nb as usize;
            let lat = edges[bi] + (edges[bi + 1] - edges[bi]) * (0.3 + 0.4 * d.unit(9_002 + 4 * j));
            Storm {
                id: format!("storm:{j}"),
                lat_deg: lat,
                lon_deg: 360.0 * d.unit(9_003 + 4 * j) - 180.0,
                size_km: band_km(bi) * (0.3 + 0.6 * d.unit(9_004 + 4 * j)),
                albedo_shift: if j % 3 == 0 { -0.12 } else { 0.1 },
            }
        })
        .collect();
    let h_km = 8.314 * t.max(10.0) / (c.mean_molar_kg * env.gravity_m_s2.max(0.1)) / 1000.0;
    CloudDeck {
        rotation_hours: rot,
        wind_scale_m_s: u,
        jets,
        band_edges_deg: edges,
        winds,
        storms,
        chemistry: chem.3.clone(),
        zone_albedo: zone,
        belt_albedo: belt,
        scale_height_km: h_km,
        cause: format!(
            "{jets} prograde jets from the Rhines scale: sqrt(a Ω / U) with a {:.0} km, a {rot:.1} h day and winds of {u:.0} m/s ({}); clouds of {} at {t:.0} K.",
            a / 1000.0,
            if ice { "an ice giant" } else { "a gas giant" },
            chem.3
        ),
    }
}

impl CloudDeck {
    /// Albedo and cloud-top height (m, zones high, belts low) at a direction.
    pub fn at(&self, v: [f64; 3], radius_km: f64) -> (f64, f64) {
        let nb = self.band_edges_deg.len() - 1;
        // Zones (w = 1) and belts (w = 0) alternate smoothly in sin(latitude);
        // the bands either side of the equator are zones, mirrored about it.
        let x = (v[2].abs().min(1.0) + 1.0) * 0.5 * nb as f64;
        let sign = if (self.jets as usize).is_multiple_of(2) {
            1.0
        } else {
            -1.0
        };
        let w = 0.5 + 0.5 * sign * m::sin(m::PI * x);
        let mut albedo = self.belt_albedo + (self.zone_albedo - self.belt_albedo) * w;
        for s in &self.storms {
            let c = from_lat_lon(s.lat_deg, s.lon_deg);
            let dist = m::acos(dot(c, v).clamp(-1.0, 1.0)) * radius_km;
            let r = dist / (0.5 * s.size_km).max(1e-9);
            albedo += s.albedo_shift * m::exp(-r * r);
        }
        let height = (w - 0.5) * self.scale_height_km * 1000.0;
        (albedo.clamp(0.0, 1.0), height)
    }
}

/// Height of a landform at a direction, m, and whether the point is inside it.
pub fn landform_height(l: &Landform, v: [f64; 3], radius_km: f64) -> (f64, bool) {
    let g = |z: f64| m::exp(-z * z);
    let hw = 0.5 * l.width_km.max(1e-9);
    match l.axis {
        Some(t) => {
            let n = cross(l.dir, t);
            let along = m::atan2(dot(v, t), dot(v, l.dir)) * radius_km;
            let across = m::asin(dot(v, n).clamp(-1.0, 1.0)) * radius_km;
            if dot(v, l.dir) < 0.0 {
                return (0.0, false);
            }
            let x = across / hw;
            if x.abs() > 3.0 {
                return (0.0, false);
            }
            let half_len = 0.5 * l.size_km;
            let over = (along.abs() - half_len).max(0.0) / hw;
            if over > 3.0 {
                return (0.0, false);
            }
            let taper = g(over);
            let h = l.height_m;
            let p = match l.kind.as_str() {
                "double-ridge" => h * (g((x - 0.5) / 0.33) + g((x + 0.5) / 0.33)),
                "band" | "groove-band" => h * g(x / 0.7),
                "rift" | "chasma" => {
                    h * g(x / 0.5) - 0.25 * h * (g((x - 1.0) / 0.5) + g((x + 1.0) / 0.5))
                }
                "scarp" => h * g(x / 0.6) * (1.0 + 0.6 * m::tanh(2.0 * x)),
                _ => h * g(x / 0.7),
            };
            (p * taper, x.abs() < 1.0 && over == 0.0)
        }
        None => {
            let ang = m::acos(dot(l.dir, v).clamp(-1.0, 1.0));
            let r = ang * radius_km / hw;
            if r >= 1.6 {
                return (0.0, false);
            }
            let q = (1.0 - r * r).max(0.0);
            let h = l.height_m;
            let p = match l.kind.as_str() {
                "corona" => h * (g((r - 0.8) / 0.2) - 0.4 * g(r / 0.45)),
                "dune-field" | "tar-sea" => 0.0,
                _ => {
                    if r < 1.0 {
                        h * q * q
                    } else {
                        0.0
                    }
                }
            };
            (p, r < 1.0)
        }
    }
}

/// Curvature constant per kind, in |height| / (half width)² (for tolerances).
pub fn landform_curvature(kind: &str) -> f64 {
    match kind {
        "double-ridge" => 20.0,
        "corona" => 52.0,
        "rift" | "chasma" => 10.0,
        "scarp" => 14.0,
        "dune-field" | "tar-sea" => 0.0,
        _ => 8.0,
    }
}

impl RegimeSurface {
    /// Why the ground at a place looks as it does (landforms covering it, newest first).
    pub fn explain(&self, lat_deg: f64, lon_deg: f64, radius_km: f64) -> Vec<String> {
        let v = from_lat_lon(lat_deg, lon_deg);
        let mut hits: Vec<(f64, String)> = self
            .landforms
            .iter()
            .filter(|l| landform_height(l, v, radius_km).1)
            .map(|l| (l.formed_myr, format!("{}: {}", l.id, l.cause)))
            .collect();
        hits.sort_by(|a, b| b.0.total_cmp(&a.0).then(a.1.cmp(&b.1)));
        let mut out: Vec<String> = hits.into_iter().map(|x| x.1).collect();
        if let Some(c) = &self.clouds {
            out.push(c.cause.clone());
        }
        out
    }

    /// Fraction of the surface a kind covers (sampled on a fixed grid).
    pub fn cover_fraction(&self, kind: &str, radius_km: f64, samples: u32) -> f64 {
        let n = samples.max(8);
        let (mut hit, mut tot) = (0.0, 0.0);
        for j in 0..n {
            let z = -1.0 + 2.0 * (j as f64 + 0.5) / n as f64;
            for i in 0..2 * n {
                let lon = 360.0 * (i as f64 + 0.5) / (2 * n) as f64 - 180.0;
                let v = from_lat_lon(m::asin(z).to_degrees(), lon);
                tot += 1.0;
                if self
                    .landforms
                    .iter()
                    .filter(|l| l.kind == kind)
                    .any(|l| landform_height(l, v, radius_km).1)
                {
                    hit += 1.0;
                }
            }
        }
        hit / tot
    }
}

impl CloudDeck {
    /// Narrowest band, km along the meridian (for tolerances).
    pub fn narrowest_band_km(&self, radius_km: f64) -> f64 {
        self.band_edges_deg
            .windows(2)
            .map(|w| (w[1] - w[0]).to_radians() * radius_km)
            .fold(f64::MAX, f64::min)
    }
}

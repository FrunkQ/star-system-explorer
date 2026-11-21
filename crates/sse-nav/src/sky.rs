//! The Window (V8, design handoff 6.8 "look out the window"): what a ship's
//! sensor picks up from where it is, at one time. Not a picture: for each
//! object, where it lies in the sky, how big it looks, how bright it is and
//! whether this sensor can see it at all, or make out its disc.
//!
//! Brightness is reflected starlight with a Lambert sphere's phase law and a
//! geometric albedo (Russell 1916 for the phase law; the Sun's V magnitude
//! from Willmer 2018). The star's brightness scales from its luminosity, so
//! a star much cooler or hotter than the Sun is shown with its bolometric
//! brightness, not its visual one, and says so.

use crate::kepler::{norm, sub, V3};
use crate::NavSystem;
use serde::{Deserialize, Serialize};
use sse_core::math as m;
use sse_core::quantity::consts::AU;

/// A sensor a ship (or a pair of eyes) looks through.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Sensor {
    pub name: String,
    /// Faintest apparent magnitude it picks up.
    pub limit_mag: f64,
    /// Smallest angle it separates (arcseconds): a disc smaller than this is a point.
    pub resolution_arcsec: f64,
    /// Field of view (degrees) the console opens with.
    pub field_deg: f64,
    /// What it sees: one band for now ("visible").
    pub band: String,
    pub why: String,
}

/// Pack settings for the Window (`provides.other.nav.window`).
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct WindowSettings {
    /// The Sun's apparent V magnitude at 1 au.
    pub sun_mag_1au: f64,
    /// Geometric albedo used for a body that has none.
    pub default_albedo: f64,
    pub sensors: std::collections::BTreeMap<String, Sensor>,
}

impl Default for WindowSettings {
    fn default() -> Self {
        serde_json::from_value(serde_json::json!({
            "sun_mag_1au": -26.74,
            "default_albedo": 0.3,
            "sensors": {
                "eye": { "name": "Eyes at a port", "limit_mag": 6.0, "resolution_arcsec": 60.0, "field_deg": 100.0, "band": "visible",
                         "why": "a dark-adapted eye: about magnitude 6, one arcminute" },
                "binoculars": { "name": "Binoculars (10x50)", "limit_mag": 9.5, "resolution_arcsec": 6.0, "field_deg": 6.0, "band": "visible",
                         "why": "50 mm aperture: about magnitude 9.5, a few arcseconds" },
                "telescope": { "name": "Ship's telescope (0.5 m)", "limit_mag": 18.0, "resolution_arcsec": 0.3, "field_deg": 0.5, "band": "visible",
                         "why": "0.5 m mirror, a short exposure: about magnitude 18, diffraction limit about 0.25 arcseconds" }
            }
        }))
        .expect("default window settings")
    }
}

/// Where the viewer is.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Viewpoint {
    /// Beside a body (just outside it, on its sunward side, so the body itself fills the view behind).
    Body(String),
    /// A point around the star (m, ecliptic frame).
    Point(V3),
}

/// One object in the window.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Seen {
    pub address: String,
    pub name: String,
    pub kind: String,
    /// Ecliptic longitude and latitude from the viewer (degrees).
    pub lon_deg: f64,
    pub lat_deg: f64,
    pub distance_au: f64,
    /// Apparent angular diameter (degrees).
    pub angular_diameter_deg: f64,
    pub magnitude: f64,
    /// Star-object-viewer angle (degrees) and the lit fraction of the disc.
    pub phase_deg: f64,
    pub lit_fraction: f64,
    /// Bright enough for this sensor.
    pub detected: bool,
    /// Big enough to show a disc.
    pub resolved: bool,
    /// Light travel time from it to the viewer (s).
    pub light_s: f64,
    pub why: String,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct WindowView {
    pub at_s: f64,
    pub viewer: V3,
    pub sensor: Sensor,
    pub sensor_id: String,
    /// The body the viewer is beside, if any (left out of `seen`).
    pub beside: Option<String>,
    pub seen: Vec<Seen>,
    /// How many bodies are too faint for this sensor.
    pub missed: u32,
    pub model: Vec<String>,
}

const C_M_S: f64 = 299_792_458.0;

fn lon_lat(d: V3) -> (f64, f64) {
    let r = norm(d).max(1e-30);
    (
        m::atan2(d[1], d[0]).to_degrees().rem_euclid(360.0),
        m::asin((d[2] / r).clamp(-1.0, 1.0)).to_degrees(),
    )
}

/// Lambert sphere phase law: reflected light at phase angle a over full phase.
pub fn lambert_phase(a: f64) -> f64 {
    (m::sin(a) + (m::PI - a) * m::cos(a)) / m::PI
}

/// What `sensor` sees from `viewpoint` at universe time `t_s`.
pub fn window(
    w: &WindowSettings,
    sys: &NavSystem,
    viewpoint: &Viewpoint,
    t_s: f64,
    sensor_id: &str,
) -> Result<WindowView, String> {
    let sensor = w.sensors.get(sensor_id).cloned().ok_or_else(|| {
        format!(
            "no sensor {sensor_id}; sensors: {}",
            w.sensors.keys().cloned().collect::<Vec<_>>().join(", ")
        )
    })?;
    let (viewer, beside) = match viewpoint {
        Viewpoint::Point(p) => (*p, None),
        Viewpoint::Body(id) => {
            let b = sys
                .body(id)
                .ok_or_else(|| format!("no body {id} in {}", sys.name))?;
            let (p, _) = sys.at(b, t_s);
            let r = norm(p).max(1.0);
            // Just outside the body on its sunward side (two radii from its centre).
            let k = 1.0 - 2.0 * b.radius_m / r;
            ([p[0] * k, p[1] * k, p[2] * k], Some(b.address.clone()))
        }
    };
    let res_deg = sensor.resolution_arcsec / 3600.0;
    let mut seen = vec![];
    let mut missed = 0u32;
    // The star.
    {
        let d = sub([0.0; 3], viewer);
        let dist = norm(d);
        let (lon, lat) = lon_lat(d);
        let ang = 2.0 * m::atan(sys.star_radius_m() / dist).to_degrees();
        let mag = w.sun_mag_1au - 2.5 * m::log10(sys.star_luminosity_lsun.max(1e-12))
            + 5.0 * m::log10(dist / AU);
        let detected = mag <= sensor.limit_mag;
        seen.push(Seen {
            address: "star".into(),
            name: "the star".into(),
            kind: "star".into(),
            lon_deg: lon,
            lat_deg: lat,
            distance_au: dist / AU,
            angular_diameter_deg: ang,
            magnitude: mag,
            phase_deg: 0.0,
            lit_fraction: 1.0,
            detected,
            resolved: ang >= res_deg,
            light_s: dist / C_M_S,
            why: format!(
                "{:.2} times the Sun's light, {:.2} au away",
                sys.star_luminosity_lsun,
                dist / AU
            ),
        });
    }
    for b in &sys.bodies {
        if beside.as_deref() == Some(b.address.as_str()) {
            continue;
        }
        let (p, _) = sys.at(b, t_s);
        let d = sub(p, viewer);
        let dist = norm(d).max(1.0);
        let r_star = norm(p).max(1.0);
        // Phase angle at the body, between the star and the viewer.
        let to_star = sub([0.0; 3], p);
        let to_viewer = sub(viewer, p);
        let cosa =
            (to_star[0] * to_viewer[0] + to_star[1] * to_viewer[1] + to_star[2] * to_viewer[2])
                / (r_star * dist);
        let a = m::acos(cosa.clamp(-1.0, 1.0));
        let phase = lambert_phase(a).max(1e-9);
        let albedo = w.default_albedo;
        let flux = sys.star_luminosity_lsun.max(1e-12)
            * albedo
            * (b.radius_m / dist)
            * (b.radius_m / dist)
            * phase
            / ((r_star / AU) * (r_star / AU));
        let mag = w.sun_mag_1au - 2.5 * m::log10(flux);
        let ang = 2.0 * m::atan(b.radius_m / dist).to_degrees();
        let (lon, lat) = lon_lat(d);
        let detected = mag <= sensor.limit_mag;
        if !detected {
            missed += 1;
        }
        let lit = (1.0 + m::cos(a)) / 2.0;
        seen.push(Seen {
            address: b.address.clone(),
            name: b.name.clone(),
            kind: "planet".into(),
            lon_deg: lon,
            lat_deg: lat,
            distance_au: dist / AU,
            angular_diameter_deg: ang,
            magnitude: mag,
            phase_deg: a.to_degrees(),
            lit_fraction: lit,
            detected,
            resolved: ang >= res_deg,
            light_s: dist / C_M_S,
            why: format!(
                "starlight reflected at albedo {albedo} from {:.2} au from the star, {:.0}% lit as seen from here",
                r_star / AU,
                lit * 100.0
            ),
        });
    }
    Ok(WindowView {
        at_s: t_s,
        viewer,
        sensor,
        sensor_id: sensor_id.into(),
        beside,
        seen,
        missed,
        model: vec![
            "positions from Kepler orbits around the star (the same orbits NAV flies)".into(),
            format!("brightness: reflected starlight off a Lambert sphere at albedo {} for every body (the chart has no albedos yet)", w.default_albedo),
            "the star's brightness scales with its luminosity (bolometric), not its visual band".into(),
            "background stars and moons are not drawn yet".into(),
        ],
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scenario::sol;

    #[test]
    fn from_earth_the_sun_and_planets_look_right() {
        let w = WindowSettings::default();
        let sys = sol();
        let v = window(&w, &sys, &Viewpoint::Body("Earth".into()), sys.t0_s, "eye").unwrap();
        let sun = &v.seen[0];
        assert!((sun.magnitude + 26.74).abs() < 0.1, "{}", sun.magnitude);
        assert!(
            (sun.angular_diameter_deg - 0.533).abs() < 0.02,
            "{}",
            sun.angular_diameter_deg
        );
        let jupiter = v.seen.iter().find(|s| s.name == "Jupiter").unwrap();
        // Jupiter runs from about -2.9 to -1.6 (with its real albedo near 0.5; 0.3 here).
        assert!(
            jupiter.magnitude > -3.5 && jupiter.magnitude < -0.5,
            "{}",
            jupiter.magnitude
        );
        assert!(
            jupiter.detected && !jupiter.resolved,
            "a bright point to the eye"
        );
        let tele = window(
            &w,
            &sys,
            &Viewpoint::Body("Earth".into()),
            sys.t0_s,
            "telescope",
        )
        .unwrap();
        assert!(
            tele.seen
                .iter()
                .find(|s| s.name == "Jupiter")
                .unwrap()
                .resolved
        );
        assert!(
            v.seen.iter().all(|s| s.name != "Earth"),
            "Earth is where we are"
        );
        assert!(window(
            &w,
            &sys,
            &Viewpoint::Body("Earth".into()),
            sys.t0_s,
            "x-ray"
        )
        .is_err());
    }

    #[test]
    fn full_phase_is_brightest() {
        assert!((lambert_phase(0.0) - 1.0).abs() < 1e-12);
        assert!(lambert_phase(m::PI / 2.0) < 0.33);
        assert!(lambert_phase(m::PI) < 1e-12);
    }
}

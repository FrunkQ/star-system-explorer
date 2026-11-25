//! SW1: space weathering of bare ground. On a world with no air to speak
//! of, the star's wind and micrometeorites slowly darken and redden soil
//! (nanophase iron; Hapke 2001, Pieters and Noble 2016). Each sample of bare
//! ground matures with the time it has lain open, the younger of its
//! surface age and the age of the youngest crater whose ejecta cover it, so
//! young craters stay bright against dark old ground. The sums are the
//! `sse.surface.space_weathering` and `sse.surface.weathered_reflectance`
//! blocks (`sse_atoms::weathering`).

use crate::{Body, SurfaceEnvelope};
use serde::{Deserialize, Serialize};
use sse_atoms::weathering as w;

/// Air at the ground at least this thick stops both the wind and the
/// micrometeorites, bar (10 Pa: a few metres of gas column stop dust grains
/// tens of micrometres across; Mars' 6 mbar does). A stand-in in the fudge
/// register.
pub const AIR_STOPS_BAR: f64 = 1.0e-4;

/// A host at least this massive is a star (solar masses), so an orbit about
/// it gives the world's distance from its star; below it the host is a
/// planet and the moon's distance comes from the planet.
pub const STAR_MSUN: f64 = 0.07;

/// How a world's bare ground weathers.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Weathering {
    /// False where air stops weathering: tiles then carry no maturity.
    pub active: bool,
    /// Time to come within 1/e of full maturity here, Myr.
    pub tau_myr: f64,
    pub wind_reaches_ground: bool,
    /// The star's wind flux against 1 AU.
    pub wind_flux_ratio: f64,
    /// Mature over fresh reflectance of the same soil at 650, 550 and
    /// 450 nm: a sample's reflectance is fresh × factor^maturity.
    pub mature_factor_rgb: [f64; 3],
    pub why: Vec<String>,
}

impl Weathering {
    /// Maturity, 0 to 1, of ground open for `age_myr`.
    pub fn maturity(&self, age_myr: f64) -> f64 {
        if !self.active {
            return 0.0;
        }
        w::maturity(
            age_myr * 1.0e6 * sse_core::sheet::YR_JULIAN,
            self.wind_flux_ratio,
            self.wind_reaches_ground,
            w::TAU_1AU_S,
            w::WIND_SHARE,
        )
    }

    /// Reflectance factors at 650, 550 and 450 nm at a maturity.
    pub fn factors(&self, maturity: f64) -> [f64; 3] {
        w::factors(maturity, self.mature_factor_rgb)
    }
}

/// How `b`'s bare ground weathers; None for a world with no ground.
/// `star_au` is the distance from the star (a moon's is its planet's); when
/// absent it is read from the world's climate or its own orbit about a star.
pub fn weathering(b: &Body, e: &SurfaceEnvelope, star_au: Option<f64>) -> Option<Weathering> {
    if b.deep_envelope || e.kind == "cloud-deck" {
        return None;
    }
    let mut why = vec![];
    let d_au = star_au.or(match &b.climate {
        crate::Climate::Star { a_au, .. } => Some(*a_au),
        crate::Climate::Fixed { .. } => b
            .orbit
            .as_ref()
            .filter(|o| o.host_msun >= STAR_MSUN)
            .map(|o| o.a_au),
    });
    let ratio = match d_au {
        Some(d) if d > 0.0 => 1.0 / (d * d),
        _ => 0.0,
    };
    let dynamo = e.core.as_ref().is_some_and(|c| c.dynamo_now);
    let open = !dynamo && ratio > 0.0;
    let active = b.pressure_bar < AIR_STOPS_BAR;
    if !active {
        why.push(format!(
            "its air ({:.3} bar at the ground) stops the star's wind and the micrometeorites, so bare ground does not weather",
            b.pressure_bar
        ));
    } else {
        why.push(if dynamo {
            "its own magnetic field holds the star's wind off the ground; micrometeorites alone weather it".to_string()
        } else if ratio > 0.0 {
            format!("with no air and no field of its own, the star's wind ({ratio:.2} times its flux at 1 AU) and micrometeorites both reach the ground")
        } else {
            "its distance from a star is not known here, so only micrometeorites are counted".to_string()
        });
    }
    let rate = w::WIND_SHARE * ratio * if open { 1.0 } else { 0.0 } + (1.0 - w::WIND_SHARE);
    let tau_myr = w::TAU_1AU_S / rate / (1.0e6 * sse_core::sheet::YR_JULIAN);
    if active {
        why.push(format!("soil matures over about {tau_myr:.0} Myr (Grier et al. 2001; Pieters and Noble 2016), darkening and reddening as nanophase iron builds up (Hapke 2001): young craters and their ejecta stay bright"));
    }
    Some(Weathering {
        active,
        tau_myr,
        wind_reaches_ground: open,
        wind_flux_ratio: ratio,
        mature_factor_rgb: w::MATURE_OVER_FRESH,
        why,
    })
}

#[cfg(test)]
mod tests {
    use crate::scenario::{fixtures, settings};

    fn of(name: &str, star_au: Option<f64>) -> Option<super::Weathering> {
        let b = fixtures()
            .into_iter()
            .find(|f| f.body.label == name)
            .expect("fixture")
            .body;
        let e = crate::envelope(&settings(), &b);
        super::weathering(&b, &e, star_au)
    }

    #[test]
    fn only_airless_ground_weathers_and_fresh_ground_is_unchanged() {
        let moon = of("Moon", Some(1.0)).expect("ground");
        assert!(moon.active && moon.wind_reaches_ground);
        assert_eq!(moon.maturity(0.0), 0.0);
        assert_eq!(moon.factors(0.0), [1.0, 1.0, 1.0]);
        // Older ground is more mature, and the wind closer in speeds it.
        assert!(moon.maturity(1000.0) > moon.maturity(100.0));
        let close = of("Moon", Some(0.4)).expect("ground");
        assert!(close.tau_myr < moon.tau_myr);
        let earth = of("Earth", Some(1.0)).expect("ground");
        assert!(!earth.active);
        assert_eq!(earth.maturity(4000.0), 0.0);
    }
}

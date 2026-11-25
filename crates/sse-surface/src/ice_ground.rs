//! IG1: ice ground. Whether a world's bare ground is ice: it has water (its
//! typed makeup, or, for a small world, its mean density read as rock and
//! ice mixed by volume), the ground is below ice's melting point, and open
//! ice lasts its age. On an airless world open ice sublimes and part of each
//! molecule's hops is lost to starlight or to space (Watson, Murray and
//! Brown 1961), so the warm Moon keeps none while Jupiter's moons keep all of
//! theirs. Under air the vapour stays in the air and comes down again. The
//! sums are the `sse.surface.ice_share_from_density` and
//! `sse.surface.open_ice_loss` blocks (`sse_atoms::ice_ground`).

use crate::weathering::{AIR_STOPS_BAR, STAR_MSUN};
use crate::{Body, SurfaceEnvelope, SurfaceSettings};
use serde::{Deserialize, Serialize};
use sse_atoms::ice_ground as ig;
use sse_core::math as m;
use sse_core::sheet::{mixture_density, G, M_EARTH, R_EARTH_MEAN, T0C, YR_JULIAN};

/// Read a world's ice from its density only below this mass, Earth masses:
/// above it self-compression makes rock look denser than it is at low
/// pressure, so a low density no longer means ice. A stand-in in the fudge
/// register.
pub const DENSITY_ICE_BELOW_MEARTH: f64 = 0.1;

/// Whether a world's bare ground is ice, and why.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct IceGround {
    pub ice: bool,
    /// Share of the mass that is ice (typed, or from the density).
    pub ice_share: f64,
    /// Thickness that ice would make spread over the surface, km.
    pub shell_km: f64,
    /// Depth of open ice lost over the world's age, km (0 under air).
    pub lost_km: f64,
    pub why: Vec<String>,
}

/// `b`'s ice ground; None for a world with no ground.
pub fn ice_ground(
    s: &SurfaceSettings,
    b: &Body,
    e: &SurfaceEnvelope,
    star_au: Option<f64>,
) -> Option<IceGround> {
    if b.deep_envelope || e.kind == "cloud-deck" {
        return None;
    }
    let mass = b.mass_earth * M_EARTH;
    let r = b.radius_earth * R_EARTH_MEAN;
    if mass <= 0.0 || r <= 0.0 {
        return None;
    }
    let mut why = vec![];
    let rho = mass / (4.0 / 3.0 * m::PI * r * r * r);
    let (ri, rr) = (mixture_density::SURFACE_ICE, mixture_density::SURFACE_ROCK);
    let share = if b.water_fraction > 0.0 {
        why.push(format!(
            "its makeup is {:.0}% ice and water",
            100.0 * b.water_fraction
        ));
        b.water_fraction
    } else if b.mass_earth < DENSITY_ICE_BELOW_MEARTH {
        let w = ig::ice_share(rho, rr, ri);
        if w > 0.0 {
            why.push(format!("its mean density of {rho:.0} kg/m³ is lighter than rock: about {:.0}% of it is ice (Showman and Malhotra 1999)", 100.0 * w));
        }
        w
    } else {
        0.0
    };
    let shell_m = share * mass / (ri * 4.0 * m::PI * r * r);
    let t = e.t_surface_now_k;
    let mut lost_m = 0.0;
    let ice = if share <= 0.0 {
        false
    } else if t >= T0C {
        why.push(format!(
            "its ground is above ice's melting point ({t:.0} K), so its water is not ice"
        ));
        false
    } else if b.pressure_bar >= AIR_STOPS_BAR {
        why.push(format!(
            "under its air ({:.3} bar) the ice's vapour stays in the air and comes down again",
            b.pressure_bar
        ));
        true
    } else {
        let d_au = star_au
            .or(b
                .orbit
                .as_ref()
                .filter(|o| o.host_msun >= STAR_MSUN)
                .map(|o| o.a_au))
            .unwrap_or(0.0);
        let water = s.climate.substances.iter().find(|x| x.gas == "h2o");
        let p = water.map_or(0.0, |w| sse_climate::ebm::p_sat(w, t));
        let mw = water.map_or(sse_core::sheet::molar_mass::H2O, |w| w.molar_kg);
        let g = G * mass / (r * r);
        let ve = m::sqrt(2.0 * G * mass / r);
        let age_s = b.age_gyr.max(0.0) * 1.0e9 * YR_JULIAN;
        lost_m = ig::open_loss_m(p, t, mw, g, ve, d_au, age_s, ri, ig::PHOTO_RATE_1AU);
        let keeps = lost_m < shell_m;
        why.push(if keeps {
            format!("at {t:.0} K open ice sublimes so slowly that only about {lost_m:.0} m of its {:.0} km is lost to starlight and to space over its age, so its ground is ice", shell_m / 1000.0)
        } else {
            format!("at {t:.0} K open ice sublimes away: over its age it would lose about {:.0} km against the {:.1} km it has, so its ground is rock (ice only in cold traps)", lost_m / 1000.0, shell_m / 1000.0)
        });
        keeps
    };
    Some(IceGround {
        ice,
        ice_share: share,
        shell_km: shell_m / 1000.0,
        lost_km: lost_m / 1000.0,
        why,
    })
}

/// Fresh reflectance at 550 nm of the bare ground materials whose optics the
/// master data sheet carries, read with its Look up: basalt from immature
/// mare soil, regolith from mature highland soil over SW1's mature-over-fresh
/// factor, ice from water's condensed colour, and the dark lag on old ice
/// from mature mare soil.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GroundOptics {
    pub basalt: f64,
    pub regolith: f64,
    pub ice: f64,
    pub dust: f64,
}

fn table_at(row: &serde_json::Map<String, serde_json::Value>, nm: f64) -> Option<f64> {
    let t = row.get("optical")?.get("reflectance")?;
    let l: Vec<f64> = t
        .get("lambda_nm")?
        .as_array()?
        .iter()
        .filter_map(|x| x.as_f64())
        .collect();
    let r: Vec<f64> = t
        .get("r")?
        .as_array()?
        .iter()
        .filter_map(|x| x.as_f64())
        .collect();
    if l.is_empty() || l.len() != r.len() {
        return None;
    }
    if nm <= l[0] {
        return Some(r[0]);
    }
    for i in 0..l.len() - 1 {
        if nm <= l[i + 1] {
            let f = (nm - l[i]) / (l[i + 1] - l[i]);
            return Some(r[i] + (r[i + 1] - r[i]) * f);
        }
    }
    r.last().copied()
}

/// The ground optics from the shipped data sheet (read once).
pub fn ground_optics() -> GroundOptics {
    static ONCE: std::sync::OnceLock<GroundOptics> = std::sync::OnceLock::new();
    *ONCE.get_or_init(|| {
        let sheet = sse_reference::Reference::base();
        let mix = |id: &str| sheet.row("mixtures", id).and_then(|r| table_at(r, 550.0));
        let ice = sheet.row("substances", "h2o").and_then(|r| {
            r.get("optical")?
                .get("condensed_colour_rgb")?
                .get("v")?
                .get(1)?
                .as_f64()
        });
        let k = sse_atoms::weathering::MATURE_OVER_FRESH[1];
        GroundOptics {
            basalt: mix("lunar-soil-immature-mare").unwrap_or(0.14),
            regolith: mix("lunar-soil-mature-highland").map_or(0.25, |x| x / k),
            ice: ice.unwrap_or(0.95),
            dust: mix("lunar-soil-mature-mare").unwrap_or(0.057),
        }
    })
}

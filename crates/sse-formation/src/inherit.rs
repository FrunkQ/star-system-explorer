//! Stage 1, inheritance: what a patch hands a newborn system (design 3, row 1;
//! galaxy design 8.4).
//!
//! `PatchEnvelope` is the inlet of the formation suite: the distributions of a
//! patch at a time. G0 supplies a default Milky-Way-like one, G3 to G5 real
//! ones; until G0 lands, `tests/fixtures/formation/solar-neighbourhood.json`
//! stands in. Draws are keyed by the system's own address, so a system
//! resolves without its neighbours (H6.1).

use serde::{Deserialize, Serialize};
use sse_core::math as m;
use sse_core::{Address, Seed};
use sse_core::{Draws, Stream};
use sse_resolve::{Distribution, Sample};

pub const FORMAT: &str = "sse.formation-envelope/1";

/// Distributions of a patch, each cited in the fixture.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct PatchEnvelope {
    pub format: String,
    pub name: String,
    /// Galactic time "now" for this patch, Gyr since the origin.
    pub now_gyr: f64,
    /// [Fe/H], dex.
    pub feh: Distribution,
    /// Age of a system at `now_gyr`, Gyr.
    pub age_gyr: Distribution,
    /// C/O number ratio.
    pub c_o: Distribution,
    /// Mg/Si number ratio.
    pub mg_si: Distribution,
    /// Initial 26Al/27Al relative to the Solar System's 5.2e-5.
    pub al26_rel: Distribution,
    /// Members of the birth cluster.
    pub cluster_members: Distribution,
    /// Far-ultraviolet field at the disc, in Habing units (G0).
    pub fuv_g0: Distribution,
    /// Citations, one line each.
    #[serde(default)]
    pub references: Vec<String>,
}

/// What one system inherited (stage 1 output).
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Inheritance {
    pub feh: f64,
    /// Measured metal mass fraction, Z = Z_sun x 10^[Fe/H] (Z_sun = 0.0142,
    /// Asplund et al. 2009); the fits convert it to their own scale.
    pub z: f64,
    pub age_gyr: f64,
    pub born_gyr: f64,
    pub c_o: f64,
    pub mg_si: f64,
    pub al26_rel: f64,
    pub cluster_members: f64,
    pub fuv_g0: f64,
    /// [alpha/Fe], dex (0 from the stand-in envelope).
    #[serde(default)]
    pub alpha_fe: f64,
    /// r-process and potassium-40 relative to the Sun's at birth (1 from the stand-in).
    #[serde(default = "one")]
    pub r_process_rel: f64,
    #[serde(default = "one")]
    pub k40_rel: f64,
    /// Total stellar mass of the cloud core when the patch fixed it (G0), solar masses.
    #[serde(default)]
    pub system_mass: Option<f64>,
    #[serde(default)]
    pub population: Option<String>,
}

fn one() -> f64 {
    1.0
}

/// Mean stellar mass of a Kroupa 2001 cluster on 0.1 to 100 solar masses, used
/// to turn a cluster mass into a member count.
pub const MEAN_STAR_MASS: f64 = 0.6;

/// Measured metal mass fraction from [Fe/H], scaled from the Sun's
/// (`hurley::Z_SUN`). Formation and `describe_star` both use it.
pub fn z_of_feh(feh: f64) -> f64 {
    crate::hurley::Z_SUN * m::pow(10.0, feh)
}

impl Inheritance {
    /// Stage 1 from a system the galaxy suite resolved (G0's `SystemBirth`).
    /// `now_gyr` is the universe time the history is read up to.
    pub fn from_birth(b: &sse_cosmos::SystemBirth, now_gyr: f64) -> Self {
        Inheritance {
            feh: b.feh,
            z: z_of_feh(b.feh),
            age_gyr: b.age_gyr(now_gyr),
            born_gyr: b.birth_gyr,
            c_o: b.c_o,
            mg_si: b.mg_si,
            al26_rel: b.al26_rel,
            cluster_members: m::round(b.cluster_mass_msun / MEAN_STAR_MASS).max(1.0),
            fuv_g0: b.cluster_fuv_g0,
            alpha_fe: b.alpha_fe,
            r_process_rel: b.r_process_rel,
            k40_rel: b.k40_rel,
            system_mass: Some(b.mass_msun),
            population: Some(b.population.clone()),
        }
    }
}

/// Values the caller fixes instead of drawing (a GM's pins, the Solar System).
#[derive(Clone, PartialEq, Debug, Default, Serialize, Deserialize)]
pub struct InheritancePins {
    pub feh: Option<f64>,
    pub age_gyr: Option<f64>,
    pub c_o: Option<f64>,
    pub mg_si: Option<f64>,
    pub al26_rel: Option<f64>,
    pub cluster_members: Option<f64>,
    pub fuv_g0: Option<f64>,
}

pub fn num(s: Sample) -> f64 {
    match s {
        Sample::Num(x) => x,
        Sample::Int(i) => i as f64,
        Sample::Cat(_) => f64::NAN,
    }
}

impl PatchEnvelope {
    pub fn from_json(s: &str) -> Result<Self, String> {
        let e: PatchEnvelope = serde_json::from_str(s).map_err(|e| e.to_string())?;
        if e.format != FORMAT {
            return Err(format!("format is {:?}, expected {FORMAT}", e.format));
        }
        for (name, d) in e.fields() {
            d.validate().map_err(|x| format!("{name}: {}", x.0))?;
        }
        Ok(e)
    }

    fn fields(&self) -> [(&'static str, &Distribution); 7] {
        [
            ("feh", &self.feh),
            ("age_gyr", &self.age_gyr),
            ("c_o", &self.c_o),
            ("mg_si", &self.mg_si),
            ("al26_rel", &self.al26_rel),
            ("cluster_members", &self.cluster_members),
            ("fuv_g0", &self.fuv_g0),
        ]
    }

    /// Draw (or take pinned) inheritance for the system at `system`.
    pub fn inherit(&self, seed: &Seed, system: &Address, pins: &InheritancePins) -> Inheritance {
        let d = Draws::new(
            *seed,
            system.clone(),
            Stream::new("formation.inherit.v1").expect("valid stream"),
        );
        let pick = |i: u64, dist: &Distribution, pin: Option<f64>| {
            pin.unwrap_or_else(|| num(dist.quantile(d.unit(i))))
        };
        let feh = pick(0, &self.feh, pins.feh);
        let age = pick(1, &self.age_gyr, pins.age_gyr);
        Inheritance {
            feh,
            z: z_of_feh(feh),
            age_gyr: age,
            born_gyr: self.now_gyr - age,
            c_o: pick(2, &self.c_o, pins.c_o),
            mg_si: pick(3, &self.mg_si, pins.mg_si),
            al26_rel: pick(4, &self.al26_rel, pins.al26_rel),
            cluster_members: m::round(pick(5, &self.cluster_members, pins.cluster_members)),
            fuv_g0: pick(6, &self.fuv_g0, pins.fuv_g0),
            alpha_fe: 0.0,
            r_process_rel: 1.0,
            k40_rel: 1.0,
            system_mass: None,
            population: None,
        }
    }
}

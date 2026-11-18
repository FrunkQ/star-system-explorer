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
    /// Metal mass fraction on the fits' scale: Z = 0.02 x 10^[Fe/H] (HPT00 calibrate
    /// the Sun as Z = 0.02, so this keeps the Sun solar in the fits).
    pub z: f64,
    pub age_gyr: f64,
    pub born_gyr: f64,
    pub c_o: f64,
    pub mg_si: f64,
    pub al26_rel: f64,
    pub cluster_members: f64,
    pub fuv_g0: f64,
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
            z: 0.02 * m::pow(10.0, feh),
            age_gyr: age,
            born_gyr: self.now_gyr - age,
            c_o: pick(2, &self.c_o, pins.c_o),
            mg_si: pick(3, &self.mg_si, pins.mg_si),
            al26_rel: pick(4, &self.al26_rel, pins.al26_rel),
            cluster_members: m::round(pick(5, &self.cluster_members, pins.cluster_members)),
            fuv_g0: pick(6, &self.fuv_g0, pins.fuv_g0),
        }
    }
}

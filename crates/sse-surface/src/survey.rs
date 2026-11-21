//! The surveyed surface of one world, every stage in order: plates (S2),
//! wearing down (S3), impacts and volcanism (S4), regime landforms (S6), then
//! cover, deposits, the built layer and GM edits (S7) and the feature
//! register. A giant gets its cloud deck only. This is what the kernel's
//! `survey_surface` op serves; tests and goldens build the same chain.

use crate::atlas::Atlas;
use crate::climate::{wet_climate, WorldClimate};
use crate::cover::{aerostat_band, layers, AerostatBand, Air, CoverInputs, Reshape};
use crate::impacts::{impacts, ImpactHistory, ImpactInputs};
use crate::named::{register, AuthoredName, Register, Rename};
use crate::regimes::{regime_surface, RegimeInputs};
use crate::wear::{wear_world, WaterHistory};
use crate::{envelope, Body, SurfaceEnvelope, SurfaceSettings};
use serde::{Deserialize, Serialize};
use sse_core::draw::Seed;
use sse_core::math as m;

pub const SURVEY_VERSION: &str = "surface-survey-v1";

/// What the survey needs beyond the body: the GM's and the campaign's inputs.
#[derive(Clone, PartialEq, Debug, Default, Serialize, Deserialize)]
pub struct SurveyOptions {
    /// Day length, hours; a locked world's orbital period when not given.
    #[serde(default)]
    pub rotation_hours: Option<f64>,
    #[serde(default)]
    pub obliquity_deg: Option<f64>,
    /// Biosphere productivity relative to Earth's (0: lifeless).
    #[serde(default)]
    pub life: f64,
    /// Built kinds to place by rule; empty for an unsettled world.
    #[serde(default)]
    pub settle: Vec<String>,
    #[serde(default)]
    pub reshape: Vec<Reshape>,
    #[serde(default)]
    pub names: Vec<AuthoredName>,
    #[serde(default)]
    pub renames: Vec<Rename>,
    /// Cloud-level wind for the aerostat band, m/s (the pack's default when absent).
    #[serde(default)]
    pub wind_m_s: Option<f64>,
}

impl SurveyOptions {
    /// The climate facts these options give: spin (signed) and tilt.
    pub fn extra(&self, rotation_hours: Option<f64>) -> sse_climate::Extra {
        sse_climate::Extra {
            rotation_hours,
            obliquity_deg: self.obliquity_deg,
            ..Default::default()
        }
    }
}

/// One surveyed world.
#[derive(Clone, Debug)]
pub struct Survey {
    pub envelope: SurfaceEnvelope,
    pub atlas: Atlas,
    /// None for a giant (no ground to name).
    pub register: Option<Register>,
    pub aerostat: Result<AerostatBand, String>,
    /// The one climate the ground was worn and covered under (None for a giant).
    pub climate: Option<WorldClimate>,
    /// The stages that ran, in order.
    pub stages: Vec<String>,
}

/// The day length the climate reads: given, else a locked world's orbital period.
pub fn day_hours(env: &SurfaceEnvelope, body: &Body, given: Option<f64>) -> Option<f64> {
    // A regime reads the length of the day, not its sense.
    given.or(body.spin_hours.map(f64::abs)).or_else(|| {
        let o = body.orbit.as_ref()?;
        env.locked.then(|| {
            let years = m::sqrt(o.a_au * o.a_au * o.a_au / o.host_msun.max(1e-12));
            years * 8766.0
        })
    })
}

/// Survey `body` at `epoch_gyr` (Gyr since it formed).
pub fn survey_world(
    s: &SurfaceSettings,
    body: &Body,
    seed: &Seed,
    epoch_gyr: f64,
    opts: &SurveyOptions,
) -> Result<Survey, String> {
    chain(s, body, seed, epoch_gyr, opts, true)
}

/// The Orbit detail level of `body` (S16): every stage but the feature
/// register, on the coarse grid of `detail::orbit_settings`. Its plate run
/// is the one a survey refines.
pub fn orbit_world(
    s: &SurfaceSettings,
    body: &Body,
    seed: &Seed,
    epoch_gyr: f64,
    opts: &SurveyOptions,
) -> Result<Survey, String> {
    let o = crate::detail::orbit_settings(s);
    let mut sv = chain(&o, body, seed, epoch_gyr, opts, false)?;
    if sv.atlas.has_plates() {
        sv.atlas.address = sv
            .envelope
            .address
            .child_key("shell", &format!("orbit@{}", sv.atlas.epoch_myr))
            .expect("valid shell key");
    }
    Ok(sv)
}

fn chain(
    s: &SurfaceSettings,
    body: &Body,
    seed: &Seed,
    epoch_gyr: f64,
    opts: &SurveyOptions,
    named: bool,
) -> Result<Survey, String> {
    let e = envelope(s, body);
    let rotation = day_hours(&e, body, opts.rotation_hours);
    let oceans = body.oceans > 0.0;
    let air = Air::guess(&s.cover, &e, body.pressure_bar, oceans);
    let aerostat = aerostat_band(&s.cover, &e, &air, opts.wind_m_s);
    let regime_in = |f, c| RegimeInputs {
        rotation_hours: rotation,
        mass_earth: body.mass_earth,
        impacts: f,
        craters: c,
    };
    if e.kind == "cloud-deck" {
        let rs = regime_surface(s, &e, seed, &regime_in(None, None), epoch_gyr);
        let atlas = Atlas::new(s, &e, seed, epoch_gyr, vec![]).with_regime(rs);
        return Ok(Survey {
            envelope: e,
            atlas,
            register: None,
            aerostat,
            climate: None,
            stages: vec!["envelope".into(), "cloud-deck".into()],
        });
    }
    let h = crate::plates::history(s, &e, seed, epoch_gyr);
    let water = WaterHistory::steady(body.oceans, body.pressure_bar, epoch_gyr);
    // The one climate (W7a): wear's rain and cover's temperature read it.
    let climate = wet_climate(
        s,
        body,
        &e,
        &water,
        // The climate locks the spin itself from the year (as the weather does).
        &opts.extra(opts.rotation_hours),
        epoch_gyr * 1000.0,
    );
    let w = wear_world(s, &e, seed, &h, &water, &climate, body.magma_end_myr);
    let atlas = Atlas::from_worn(s, &e, seed, &h, &w, vec![]);
    let hist = ImpactHistory::lunar();
    let ii = ImpactInputs {
        history: &hist,
        water: &water,
        pressure_bar: body.pressure_bar,
        plates: Some(&h),
    };
    let (mut feat, field) = impacts(s, &e, seed, &ii, epoch_gyr)?;
    let mut rs = regime_surface(
        s,
        &e,
        seed,
        &regime_in(Some(&feat), Some(&field)),
        epoch_gyr,
    );
    if !named {
        // Orbit detail keeps only features at least two coarse cells across
        // (S16: the largest craters and a few big landforms); smaller ones
        // are left to the texture, never given a cause they have not had.
        let cell_km =
            e.reference_radius_m * (m::PI / 2.0) / s.plates.cells_per_face as f64 / 1000.0;
        let min_km = 2.0 * cell_km;
        feat.craters.retain(|c| c.d_km >= min_km);
        rs.landforms.retain(|l| l.size_km >= min_km);
    }
    let atlas = atlas.with_impacts(feat, field).with_regime(rs);
    let inp = CoverInputs {
        air,
        water: oceans || body.water_fraction > 0.0,
        climate: &climate,
        life: opts.life,
        plates: Some(&h),
        reshape: opts.reshape.clone(),
        settle: opts.settle.clone(),
    };
    let ly = layers(s, &e, seed, &atlas, &inp);
    let atlas = atlas.with_layers(ly);
    if !named {
        let stages = ["envelope", "plates", "wear", "impacts", "regime", "cover"];
        return Ok(Survey {
            envelope: e,
            atlas,
            register: None,
            aerostat,
            climate: Some(climate),
            stages: stages.iter().map(|x| x.to_string()).collect(),
        });
    }
    let reg = register(
        &s.features,
        &e,
        seed,
        &atlas,
        &opts.names,
        opts.renames.clone(),
    )?;
    let stages = [
        "envelope", "plates", "wear", "impacts", "regime", "cover", "register",
    ];
    Ok(Survey {
        envelope: e,
        atlas,
        register: Some(reg),
        aerostat,
        climate: Some(climate),
        stages: stages.iter().map(|x| x.to_string()).collect(),
    })
}

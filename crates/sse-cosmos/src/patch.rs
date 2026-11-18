//! P, stage G0: the default patch envelope (galaxy design 7 and 12).
//!
//! A patch is a box at a galactocentric radius and height, at a universe
//! time. Its envelope is computed, never stored unless edited. Three tiers:
//!
//! ```text
//! patch            facts: local conditions and hazards; budget: birth mass
//!   cohort:k       one population born in one time span (halo, thick disc,
//!                  five thin-disc ages); share fixed by the model
//!     sys:i        a system's inheritance: birth time, [Fe/H], [alpha/Fe],
//!                  C/O, Mg/Si, r-process and K-40 and Al-26 relative to the
//!                  Sun's, birth cluster mass and its ultraviolet glare
//! ```
//!
//! Cohorts carry the age-metallicity relation: each cohort's [Fe/H] mean is
//! the ISM curve at its birth time, so old systems are metal-poor and
//! alpha-rich without correlated draws. Stars, multiplicity and life lines
//! are the physics suite's (formation stage F0); this hands over the cloud
//! core and its ingredients (formation design 3, stage 1).
//!
//! Not yet: spiral arms, bar, bulge, local texture, events still in effect,
//! green regions (G4) and frames and promotion (G5).

use crate::model::{CohortSpec, Geometry, PatchModel};
use serde::{Deserialize, Serialize};
use sse_chronicle::UniverseTime;
use sse_core::cbor::Value;
use sse_core::{math as m, Address, Dimension};
use sse_resolve::{
    Budget, CountSpec, Distribution, Envelope, FieldSpec, PartitionSpec, Resolved, Weights,
};
use std::collections::BTreeMap;

pub const GYR_S: f64 = 3.15576e16;
pub const MSUN_KG: f64 = 1.988_41e30;
/// One minor unit of the birth-mass budget: a thousandth of a solar mass.
pub const MASS_QUANTUM_KG: f64 = MSUN_KG * 1e-3;
/// km/s to pc/Myr.
pub const KMS_PC_MYR: f64 = 1.022_712;

/// Where and when a patch is.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Site {
    /// Galactocentric radius in the disc plane.
    pub r_kpc: f64,
    /// Height above the midplane.
    pub z_pc: f64,
    /// Edge of the (cubic) patch.
    pub size_pc: f64,
    pub epoch: UniverseTime,
}

impl Site {
    /// The Sun's neighbourhood now: a 16 pc leaf at 8.2 kpc (galaxy design 7).
    pub fn solar(model: &PatchModel) -> Self {
        Site {
            r_kpc: model.sun_radius_kpc,
            z_pc: 20.0,
            size_pc: 16.0,
            epoch: gyr_time(model.now_gyr),
        }
    }
    pub fn volume_pc3(&self) -> f64 {
        self.size_pc * self.size_pc * self.size_pc
    }
    pub fn epoch_gyr(&self) -> f64 {
        self.epoch.s as f64 / GYR_S
    }
}

/// Round to 11 significant digits, so a value written to JSON and read back
/// is bit-identical: serde_json's default float parser is exact only for
/// short mantissas (Clinger's fast path), and envelope fixtures must resolve
/// exactly as the code-built envelope does.
pub fn tidy(x: f64) -> f64 {
    if x == 0.0 || !x.is_finite() {
        return x;
    }
    format!("{x:.10e}").parse().expect("formatted float parses")
}

pub fn gyr_time(gyr: f64) -> UniverseTime {
    UniverseTime::seconds(m::round(gyr * GYR_S) as i64)
}

/// One cohort as it stands in a patch: its span (cut at the epoch), its
/// number density and the means its systems draw around.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct CohortState {
    pub index: usize,
    pub population: String,
    pub from_gyr: f64,
    pub to_gyr: f64,
    pub density_pc3: f64,
    pub feh_mean: f64,
    pub feh_sd: f64,
    pub alpha_fe_mean: f64,
    pub r_sigma_dex: f64,
    /// Velocity dispersion of its stars now (km/s).
    pub sigma_kms: f64,
}

/// What a patch knows at its epoch (galaxy design 7: conditions and hazards).
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Conditions {
    pub model: String,
    pub site: Site,
    /// Systems per cubic parsec, by population.
    pub density_pc3: BTreeMap<String, f64>,
    pub systems_per_pc3: f64,
    pub expected_systems: f64,
    /// Present ISM.
    pub feh_ism: f64,
    pub alpha_fe_ism: f64,
    /// Star formation per unit volume (solar masses per Gyr per pc^3).
    pub sfr_msun_gyr_pc3: f64,
    /// Supernova rates per Gyr per pc^3.
    pub cc_sn_gyr_pc3: f64,
    pub ia_sn_gyr_pc3: f64,
    /// Expected supernovae within the sterilising distance of a system, per Gyr.
    pub sterilising_sn_per_gyr: f64,
    /// Stellar passages within `encounter_pc` of a system, per Myr.
    pub encounters_per_myr: f64,
    /// Time since the ISM here passed the rocky-planet floor ([Fe/H]), or
    /// `None` if it has not yet.
    pub rocky_metals_for_gyr: Option<f64>,
    /// What died here before, per cubic parsec: supernovae so far by kind.
    pub cc_sn_so_far_pc3: f64,
    pub ia_sn_so_far_pc3: f64,
    pub mean_birth_mass_msun: f64,
    pub cc_sn_per_msun: f64,
    pub cohorts: Vec<CohortState>,
}

fn pop_density(model: &PatchModel, name: &str, r_kpc: f64, z_pc: f64) -> f64 {
    let p = model
        .populations
        .iter()
        .find(|p| p.name == name)
        .expect("validated");
    let r0 = model.sun_radius_kpc;
    match p.geometry {
        Geometry::Disc => {
            p.density_pc3
                * m::exp(-(r_kpc - r0) / p.scale_length_kpc)
                * m::exp(-z_pc.abs() / p.scale_height_pc)
        }
        Geometry::Halo => {
            let z_kpc = z_pc / 1000.0 / p.flattening;
            let r = m::sqrt(r_kpc * r_kpc + z_kpc * z_kpc).max(0.5);
            p.density_pc3 * m::pow(r0 / r, p.power)
        }
    }
}

/// ISM [Fe/H] at universe time `t_gyr` and radius `r_kpc`: rises as
/// log10(1 - exp(-t/tau)), normalised to the present gradient.
pub fn feh_ism(model: &PatchModel, r_kpc: f64, t_gyr: f64) -> f64 {
    let e = &model.enrichment;
    let f = |t: f64| 1.0 - m::exp(-(t - model.galaxy_start_gyr).max(0.0) / e.tau_gyr);
    let now = f(model.now_gyr);
    let here = f(t_gyr);
    let base = e.feh_now_at_sun + e.gradient_dex_per_kpc * (r_kpc - model.sun_radius_kpc);
    if here <= 0.0 {
        return e.floor;
    }
    (base + m::log10(here / now)).max(e.floor)
}

/// When the ISM at `r_kpc` reaches `feh` (inverse of `feh_ism`), if ever.
pub fn feh_ism_reaches(model: &PatchModel, r_kpc: f64, feh: f64) -> Option<f64> {
    let e = &model.enrichment;
    let base = e.feh_now_at_sun + e.gradient_dex_per_kpc * (r_kpc - model.sun_radius_kpc);
    let now = 1.0 - m::exp(-(model.now_gyr - model.galaxy_start_gyr) / e.tau_gyr);
    let target = now * m::pow(10.0, feh - base);
    if target >= 1.0 {
        return None;
    }
    Some(model.galaxy_start_gyr - e.tau_gyr * m::ln(1.0 - target))
}

/// ISM [alpha/Fe] at universe time `t_gyr`: a plateau until the type Ia
/// knee, then down (in log time) to today's value.
pub fn alpha_fe_ism(model: &PatchModel, t_gyr: f64) -> f64 {
    let a = &model.alpha_fe;
    let since = (t_gyr - model.galaxy_start_gyr).max(1e-6);
    let span = model.now_gyr - model.galaxy_start_gyr;
    let x = (m::log10(since / a.knee_gyr) / m::log10(span / a.knee_gyr)).clamp(0.0, 1.0);
    a.plateau - (a.plateau - a.now) * x
}

fn sigma_kms(model: &PatchModel, c: &CohortSpec, age_gyr: f64) -> f64 {
    let v = &model.velocity;
    if c.population == "halo" {
        v.halo_kms
    } else {
        v.sigma_at_10gyr_kms * m::pow((age_gyr / 10.0).max(0.01), v.power)
    }
}

/// Conditions at a site (closed form, nothing drawn).
pub fn conditions(model: &PatchModel, site: &Site) -> Conditions {
    let t = site.epoch_gyr();
    let r = site.r_kpc;
    let imf = &model.imf;
    let mean_mass = imf.mean().expect("power-law IMFs have a mean");
    let cc_frac = 1.0 - imf.cdf(model.cc_sn_min_mass).expect("numeric");
    let cc_per_msun = cc_frac / mean_mass;

    // Cohorts: population shares, cut at the epoch.
    let mut cohorts = Vec::new();
    let mut density_pc3: BTreeMap<String, f64> = BTreeMap::new();
    for (i, c) in model.cohorts.iter().enumerate() {
        if c.from_gyr >= t {
            continue;
        }
        let total_share: f64 = model
            .cohorts
            .iter()
            .filter(|o| o.population == c.population)
            .map(|o| o.share)
            .sum();
        let to = c.to_gyr.min(t);
        let formed = (to - c.from_gyr) / (c.to_gyr - c.from_gyr);
        let n = pop_density(model, &c.population, r, site.z_pc) * c.share / total_share * formed;
        let mid = (c.from_gyr + to) / 2.0;
        let feh = c
            .feh_fixed
            .unwrap_or_else(|| feh_ism(model, r, mid) + c.feh_offset);
        let alpha = c.alpha_fixed.unwrap_or_else(|| alpha_fe_ism(model, mid));
        *density_pc3.entry(c.population.clone()).or_default() += n;
        cohorts.push(CohortState {
            index: i,
            population: c.population.clone(),
            from_gyr: c.from_gyr,
            to_gyr: to,
            density_pc3: tidy(n),
            feh_mean: tidy(feh),
            feh_sd: c.feh_sd,
            alpha_fe_mean: tidy(alpha),
            r_sigma_dex: c.r_sigma_dex,
            sigma_kms: tidy(sigma_kms(model, c, t - mid)),
        });
    }
    let systems: f64 = cohorts.iter().map(|c| c.density_pc3).sum();

    // Star formation now: an exponential disc normalised to the galaxy's
    // rate, spread over the young stars' scale height. Zero once the last
    // cohort has ended (a model with no current cohort forms no stars).
    let s = &model.sfr;
    let h = s.scale_length_kpc * 1000.0;
    let sigma0 = s.galaxy_msun_per_yr * 1e9 / (m::TAU * h * h); // Msun/Gyr/pc^2 at the centre
    let forming = model
        .cohorts
        .iter()
        .any(|c| c.from_gyr <= t && t <= c.to_gyr && c.population == "thin");
    let sfr = if forming {
        sigma0 * m::exp(-r / s.scale_length_kpc) / (2.0 * s.young_scale_height_pc)
            * m::exp(-site.z_pc.abs() / s.young_scale_height_pc)
    } else {
        0.0
    };
    let cc = sfr * cc_per_msun;

    // Type Ia: each cohort's birth mass times the 1/t delay-time
    // distribution, averaged over its span of ages.
    let ia = &model.ia;
    let norm = ia.per_msun / m::ln((model.now_gyr - model.galaxy_start_gyr) / ia.min_delay_gyr);
    let mut ia_rate = 0.0;
    let mut ia_so_far = 0.0;
    let mut cc_so_far = 0.0;
    for c in &cohorts {
        let mass = c.density_pc3 * mean_mass;
        let young = (t - c.to_gyr).max(ia.min_delay_gyr);
        let old = (t - c.from_gyr).max(young + 1e-9);
        ia_rate += mass * norm * m::ln(old / young) / (old - young);
        // Expected so far: integral of the DTD over each age (midpoint).
        let age = t - (c.from_gyr + c.to_gyr) / 2.0;
        if age > ia.min_delay_gyr {
            ia_so_far += mass * norm * m::ln(age / ia.min_delay_gyr);
        }
        cc_so_far += mass * cc_per_msun;
    }

    let d = model.sterilising_pc;
    let sterilising = (cc + ia_rate) * 4.0 / 3.0 * m::PI * d * d * d;

    // Encounters: n * pi b^2 * v_rel, v_rel from each cohort's dispersion
    // against the local thin-disc mean.
    let thin: Vec<&CohortState> = cohorts.iter().filter(|c| c.population == "thin").collect();
    let thin_n: f64 = thin.iter().map(|c| c.density_pc3).sum();
    let sigma_ref = if thin_n > 0.0 {
        thin.iter()
            .map(|c| c.sigma_kms * c.density_pc3)
            .sum::<f64>()
            / thin_n
    } else {
        model.velocity.sigma_at_10gyr_kms
    };
    let b = model.encounter_pc;
    let encounters: f64 = cohorts
        .iter()
        .map(|c| {
            let v = m::sqrt(c.sigma_kms * c.sigma_kms + sigma_ref * sigma_ref) * KMS_PC_MYR;
            c.density_pc3 * m::PI * b * b * v
        })
        .sum();

    let rocky =
        feh_ism_reaches(model, r, model.rocky_feh).and_then(|at| (at < t).then_some(t - at));

    Conditions {
        model: model.id.clone(),
        site: site.clone(),
        density_pc3: density_pc3.into_iter().map(|(k, v)| (k, tidy(v))).collect(),
        systems_per_pc3: tidy(systems),
        expected_systems: tidy(systems * site.volume_pc3()),
        feh_ism: tidy(feh_ism(model, r, t)),
        alpha_fe_ism: tidy(alpha_fe_ism(model, t)),
        sfr_msun_gyr_pc3: tidy(sfr),
        cc_sn_gyr_pc3: tidy(cc),
        ia_sn_gyr_pc3: tidy(ia_rate),
        sterilising_sn_per_gyr: tidy(sterilising),
        encounters_per_myr: tidy(encounters),
        rocky_metals_for_gyr: rocky.map(tidy),
        cc_sn_so_far_pc3: tidy(cc_so_far),
        ia_sn_so_far_pc3: tidy(ia_so_far),
        mean_birth_mass_msun: tidy(mean_mass),
        cc_sn_per_msun: tidy(cc_per_msun),
        cohorts,
    }
}

fn mass_dim() -> Dimension {
    Dimension([0, 1, 0, 0, 0, 0, 0])
}

fn f(x: f64) -> Value {
    Value::Float(x)
}

/// The patch envelope: facts are the conditions; children are its cohorts,
/// with the birth-mass budget split in proportion to their expected systems.
pub fn patch_envelope(model: &PatchModel, address: Address, site: &Site) -> (Envelope, Conditions) {
    let c = conditions(model, site);
    let total_units = m::round(c.expected_systems * c.mean_birth_mass_msun * 1000.0) as u64;
    let mut budgets = BTreeMap::new();
    budgets.insert(
        "birth_mass".to_string(),
        Budget {
            amount: total_units,
            quantum: MASS_QUANTUM_KG,
            dim: mass_dim(),
            group: "mass".into(),
        },
    );
    let mut facts = BTreeMap::new();
    let v = site.volume_pc3();
    for (k, x) in [
        ("patch.r_kpc", site.r_kpc),
        ("patch.z_pc", site.z_pc),
        ("patch.size_pc", site.size_pc),
        ("patch.systems_per_pc3", c.systems_per_pc3),
        ("patch.feh_ism", c.feh_ism),
        ("patch.alpha_fe_ism", c.alpha_fe_ism),
        ("patch.sfr_msun_gyr_pc3", c.sfr_msun_gyr_pc3),
        ("patch.cc_sn_gyr_pc3", c.cc_sn_gyr_pc3),
        ("patch.ia_sn_gyr_pc3", c.ia_sn_gyr_pc3),
        ("patch.sterilising_sn_per_gyr", c.sterilising_sn_per_gyr),
        ("patch.encounters_per_myr", c.encounters_per_myr),
    ] {
        facts.insert(k.to_string(), f(x));
    }
    if let Some(r) = c.rocky_metals_for_gyr {
        facts.insert("patch.rocky_metals_for_gyr".into(), f(r));
    }
    facts.insert("patch.model".into(), Value::Text(model.id.clone()));
    let mut legacies = BTreeMap::new();
    legacies.insert("supernovae_cc".into(), f(tidy(c.cc_sn_so_far_pc3 * v)));
    legacies.insert("supernovae_ia".into(), f(tidy(c.ia_sn_so_far_pc3 * v)));
    let mut uncertainty = BTreeMap::new();
    uncertainty.insert("patch.feh_ism".into(), 0.1);
    uncertainty.insert("patch.encounters_per_myr".into(), 0.3);
    uncertainty.insert("patch.sterilising_sn_per_gyr".into(), 0.5);
    let weights: Vec<f64> = c.cohorts.iter().map(|k| k.density_pc3).collect();
    let env = Envelope {
        address,
        tier: "patch".into(),
        epoch: site.epoch,
        child_kind: "cohort".into(),
        budgets,
        count: CountSpec::Fixed {
            n: weights.len() as u64,
        },
        partition: PartitionSpec::Flat {
            weights: Weights::Given { weights },
        },
        fields: BTreeMap::new(),
        life: None,
        legacies,
        pins: vec![],
        pins_count_toward: true,
        uncertainty,
        facts,
        child: None,
    };
    (env, c)
}

fn draw(dist: Distribution) -> FieldSpec {
    FieldSpec::Draw { dist, version: 1 }
}
fn normal(mean: f64, sd: f64, lo: f64, hi: f64) -> Distribution {
    Distribution::Normal {
        mean,
        sd,
        lo: Some(lo),
        hi: Some(hi),
    }
}
fn lognormal(median: f64, sigma_dex: f64) -> Distribution {
    Distribution::LogNormal { median, sigma_dex }
}

/// A cohort's own envelope: its systems, drawn from the IMF until the
/// cohort's birth mass is used, and their inheritance as draws around the cohort's means.
/// `cohort` is the resolved cohort (its claimed budget is what its systems
/// share).
pub fn cohort_envelope(
    model: &PatchModel,
    cond: &Conditions,
    cohort: &Resolved,
) -> Option<Envelope> {
    let k = cohort.address.last().key().parse::<usize>().ok()?;
    let st = cond.cohorts.get(k)?;
    let spec = &model.cohorts[st.index];
    let claimed = match cohort.facts.get("budget.birth_mass") {
        Some(Value::Int(v)) => *v as u64,
        _ => return None,
    };
    let mut budgets = BTreeMap::new();
    budgets.insert(
        "birth_mass".to_string(),
        Budget {
            amount: claimed,
            quantum: MASS_QUANTUM_KG,
            dim: mass_dim(),
            group: "mass".into(),
        },
    );
    let mut fields = BTreeMap::new();
    fields.insert(
        "birth_time_s".to_string(),
        draw(Distribution::Uniform {
            lo: tidy(st.from_gyr * GYR_S),
            hi: tidy(st.to_gyr * GYR_S),
        }),
    );
    fields.insert(
        "feh".into(),
        draw(normal(st.feh_mean, st.feh_sd, -4.0, 0.7)),
    );
    fields.insert(
        "alpha_fe".into(),
        draw(normal(st.alpha_fe_mean, 0.05, -0.2, 0.6)),
    );
    let co = tidy(model.c_o.solar * m::pow(10.0, model.c_o.dex_per_feh * st.feh_mean));
    fields.insert("c_o".into(), draw(normal(co, model.c_o.sd, 0.1, 1.5)));
    fields.insert(
        "mg_si".into(),
        draw(normal(model.mg_si.mean, model.mg_si.sd, 0.5, 2.0)),
    );
    fields.insert("r_process_rel".into(), draw(lognormal(1.0, st.r_sigma_dex)));
    fields.insert("k40_rel".into(), draw(lognormal(1.0, model.k40.sigma_dex)));
    fields.insert(
        "al26_rel".into(),
        draw(lognormal(model.al26.median, model.al26.sigma_dex)),
    );
    fields.insert(
        "cluster_mass_msun".into(),
        draw(Distribution::PowerLaw {
            alpha: model.cluster.alpha,
            lo: model.cluster.lo_msun,
            hi: model.cluster.hi_msun,
        }),
    );
    fields.insert(
        "cluster_fuv_g0".into(),
        draw(lognormal(model.fuv.median_g0, model.fuv.sigma_dex)),
    );
    fields.insert(
        "population".into(),
        FieldSpec::Inherit {
            field: "cohort.population".into(),
        },
    );
    let mut facts = BTreeMap::new();
    facts.insert(
        "cohort.population".into(),
        Value::Text(spec.population.clone()),
    );
    facts.insert("cohort.from_gyr".into(), f(st.from_gyr));
    facts.insert("cohort.to_gyr".into(), f(st.to_gyr));
    facts.insert("cohort.feh_mean".into(), f(st.feh_mean));
    facts.insert("cohort.alpha_fe_mean".into(), f(st.alpha_fe_mean));
    facts.insert("cohort.sigma_kms".into(), f(st.sigma_kms));
    let mut uncertainty = BTreeMap::new();
    uncertainty.insert("feh".into(), st.feh_sd);
    uncertainty.insert("al26_rel".into(), model.al26.sigma_dex);
    Some(Envelope {
        address: cohort.address.clone(),
        tier: "cohort".into(),
        epoch: cond.site.epoch,
        child_kind: "sys".into(),
        budgets,
        // Stars are drawn from the IMF until the cohort's birth mass is used
        // (each system's mass is its own draw), so the count follows the
        // budget: on average density x volume.
        count: CountSpec::Fill {
            budget: "birth_mass".into(),
            dist: model.imf.clone(),
            unit: 1000.0,
        },
        partition: PartitionSpec::default(),
        fields,
        life: None,
        legacies: BTreeMap::new(),
        pins: vec![],
        pins_count_toward: true,
        uncertainty,
        facts,
        child: None,
    })
}

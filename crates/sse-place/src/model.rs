//! The forward world model placement searches through: causes (what can be
//! set: orbit, mass, ingredients) run through the base-physics flows to the
//! results the atlas classifies (temperatures, air, oceans).
//!
//! The star, climate and bulk numbers come from the real flows
//! (`sse.base.star-main-sequence`, `sse.base.planet-climate`,
//! `sse.base.planet-bulk`). The links between them that no flow publishes
//! yet (greenhouse depth from pressure, air kept or lost, where water is
//! liquid, surface dose) are pack settings in `packs/base-place`, labelled
//! rough, so a later surface or atmosphere flow replaces them.

use serde::{Deserialize, Serialize};
use sse_core::cbor::Value;
use sse_core::math as m;
use sse_core::quantity::consts::{AU, G, GM_EARTH, GM_SUN, L_SUN, R_EARTH};
use sse_core::{Dimension as D, Quantity};
use sse_flow::{Context, Flow, InputValue, SourceLabel};
use sse_pack::PackManifest;
use std::collections::BTreeMap;

pub const M_EARTH: f64 = GM_EARTH / G;
pub const M_SUN: f64 = GM_SUN / G;
const GYR_S: f64 = 3.15576e16;
const I_EARTH: f64 = L_SUN / (4.0 * m::PI * AU * AU);

/// Pack settings (`provides.other.placement`).
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Settings {
    pub candidates: u32,
    pub refine_steps: u32,
    pub keep: u32,
    /// A fit at or above this counts as full.
    pub full_fit: f64,
    /// Infrared optical depth = k * (pressure in bar)^n.
    pub optical_depth_k: f64,
    pub optical_depth_n: f64,
    /// Cosmic shoreline: air is kept while I/I_Earth <= (v_esc / v0)^4.
    pub shoreline_v0_m_s: f64,
    /// Surface dose = free * exp(-column / attenuation) + ground, mSv/yr.
    pub dose_free_msv_yr: f64,
    pub dose_attenuation_g_cm2: f64,
    pub dose_ground_msv_yr: f64,
    /// Orbits closer than this many mutual Hill radii to a neighbour are unstable.
    pub mutual_hill: f64,
    pub slot_au: (f64, f64),
    /// Prior ranges for causes the class leaves free.
    pub priors: BTreeMap<String, (f64, f64)>,
    #[serde(default)]
    pub sources: BTreeMap<String, String>,
}

impl Settings {
    pub fn from_packs(stack: &[PackManifest]) -> Result<Self, String> {
        let v = stack
            .iter()
            .rev()
            .find_map(|p| p.provides.other.get("placement"))
            .ok_or("no pack provides placement settings")?;
        serde_json::to_value(v)
            .and_then(serde_json::from_value)
            .map_err(|e| format!("placement settings: {e}"))
    }
    pub fn prior(&self, k: &str) -> (f64, f64) {
        self.priors.get(k).copied().unwrap_or((0.0, 1.0))
    }
}

/// What can be set. SI units in the field names' sense.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Causes {
    pub a_au: f64,
    pub ecc: f64,
    pub mass_me: f64,
    pub metal: f64,
    pub rock: f64,
    pub carbon: f64,
    pub ice: f64,
    pub gas: f64,
    pub albedo: f64,
    /// Volatile inventory, as the surface pressure it would give if kept.
    pub pressure_bar: f64,
    pub air_main: String,
    pub pp_o2_bar: f64,
    pub pp_co2_bar: f64,
    /// Surface liquid the inventory would give, as a cover fraction.
    pub liquid_inventory: f64,
    /// `water` or `methane`.
    pub liquid_kind: String,
    pub subsurface_ocean: bool,
}

/// The system a body is placed into.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct System {
    pub name: String,
    pub star_mass_msun: f64,
    pub age_gyr: f64,
    #[serde(default)]
    pub bodies: Vec<Neighbour>,
}
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Neighbour {
    pub name: String,
    pub a_au: f64,
    pub mass_me: f64,
}

/// The base-physics flows, compiled once.
pub struct Model {
    pub settings: Settings,
    atoms: sse_flow::AtomRegistry,
    kinds: BTreeMap<String, sse_core::QuantityKind>,
    units: sse_core::Units,
    star: Flow,
    climate: Flow,
    bulk: Flow,
}

fn q(v: f64, d: D) -> InputValue {
    InputValue {
        q: Quantity::si(v, d),
        source: SourceLabel::Authored,
    }
}

impl Model {
    pub fn new(physics: &PackManifest, settings: Settings) -> Result<Self, String> {
        let stack = sse_pack::resolve(std::slice::from_ref(physics), "0.1.0")
            .map_err(|e| format!("{e:?}"))?;
        let atoms = sse_atoms::registry();
        let cx = Context {
            atoms: &atoms,
            kinds: &stack.kinds,
            units: &stack.units,
        };
        let get = |id: &str| -> Result<Flow, String> {
            let f = stack.flows.get(id).ok_or(format!("flow {id} missing"))?;
            Flow::compile(f.doc.clone(), &cx).map_err(|e| format!("{id}: {e:?}"))
        };
        let (star, climate, bulk) = (
            get("sse.base.star-main-sequence")?,
            get("sse.base.planet-climate")?,
            get("sse.base.planet-bulk")?,
        );
        Ok(Model {
            settings,
            atoms,
            kinds: stack.kinds,
            units: stack.units,
            star,
            climate,
            bulk,
        })
    }
    fn cx(&self) -> Context<'_> {
        Context {
            atoms: &self.atoms,
            kinds: &self.kinds,
            units: &self.units,
        }
    }

    /// Luminosity of the star at the system's age, W.
    pub fn luminosity_w(&self, sys: &System) -> Result<f64, String> {
        let ins: BTreeMap<String, InputValue> = [
            ("mass".to_string(), q(sys.star_mass_msun * M_SUN, D::MASS)),
            ("age".to_string(), q(sys.age_gyr * GYR_S, D::TIME)),
        ]
        .into_iter()
        .collect();
        let ev = self.star.eval(&self.cx(), &ins).map_err(|e| e.message)?;
        ev.published("star.luminosity_at_age")
            .map(|x| x.value)
            .ok_or("no luminosity".into())
    }

    /// Infrared optical depth from surface pressure (rough, pack law).
    pub fn optical_depth(&self, p_bar: f64) -> f64 {
        if p_bar <= 0.0 {
            0.0
        } else {
            self.settings.optical_depth_k * m::pow(p_bar, self.settings.optical_depth_n)
        }
    }

    /// Radius in Earth radii from mass and make-up.
    pub fn radius_re(&self, c: &Causes, flux_e: f64, age_gyr: f64) -> f64 {
        if c.gas >= 0.5 {
            sse_atoms::planet::ck_radius_earth(c.mass_me)
        } else {
            let core = c.mass_me * (1.0 - c.gas);
            let cmf = if c.metal + c.rock > 0.0 {
                c.metal / (c.metal + c.rock + c.carbon)
            } else {
                0.0
            };
            let solid = (1.07 - 0.21 * cmf) * m::pow(core, 1.0 / 3.7) * (1.0 + 0.5 * c.ice);
            if c.gas > 0.0 {
                let env = 2.06
                    * m::pow(core, -0.21)
                    * m::pow(c.gas / 0.05, 0.59)
                    * m::pow(flux_e.max(1e-6), 0.044)
                    * m::pow(age_gyr.max(0.01) / 5.0, -0.18);
                solid + env
            } else {
                solid
            }
        }
    }

    /// Run the causes forward. Returns the facts the atlas reads (V4 field
    /// names, SI units), or why the flows refused them.
    pub fn forward(
        &self,
        sys: &System,
        lum_w: f64,
        c: &Causes,
    ) -> Result<BTreeMap<String, Value>, String> {
        let s = &self.settings;
        let flux_e = lum_w / (4.0 * m::PI * (c.a_au * AU) * (c.a_au * AU)) / I_EARTH;
        let r_re = self.radius_re(c, flux_e, sys.age_gyr);
        let mass = c.mass_me * M_EARTH;
        let radius = r_re * R_EARTH;
        let ins: BTreeMap<String, InputValue> = [
            ("mass".to_string(), q(mass, D::MASS)),
            ("radius".to_string(), q(radius, D::LENGTH)),
        ]
        .into_iter()
        .collect();
        let bulk = self.bulk.eval(&self.cx(), &ins).map_err(|e| e.message)?;
        let pick = |e: &sse_flow::Evaluation, k: &str| e.published(k).map(|x| x.value);
        let rho = pick(&bulk, "body.bulk_density").ok_or("no density")?;
        let g = pick(&bulk, "body.surface_gravity").ok_or("no gravity")?;
        let v_esc = pick(&bulk, "body.escape_velocity").ok_or("no escape velocity")?;
        let solid = c.gas < 0.005;
        // Air kept or lost (cosmic shoreline), only for bodies with a surface.
        let kept = solid && flux_e <= m::pow(v_esc / s.shoreline_v0_m_s, 4.0);
        let p_bar = if solid && kept { c.pressure_bar } else { 0.0 };
        let tau = self.optical_depth(p_bar);
        let ins: BTreeMap<String, InputValue> = [
            ("luminosity".to_string(), q(lum_w, D::POWER)),
            ("distance".to_string(), q(c.a_au * AU, D::LENGTH)),
            ("albedo".to_string(), q(c.albedo, D::NONE)),
            ("optical_depth".to_string(), q(tau, D::NONE)),
        ]
        .into_iter()
        .collect();
        let clim = self.climate.eval(&self.cx(), &ins).map_err(|e| e.message)?;
        let teq = pick(&clim, "planet.equilibrium_temperature").ok_or("no teq")?;
        let tsurf = pick(&clim, "planet.surface_temperature").ok_or("no tsurf")?;

        let mut f: BTreeMap<String, Value> = BTreeMap::new();
        put(&mut f, "body.mass", mass);
        put(&mut f, "body.radius", radius);
        put(&mut f, "body.bulk_density", rho);
        put(&mut f, "body.surface_gravity", g);
        put(&mut f, "body.escape_velocity", v_esc);
        put(&mut f, "orbit.semi_major_axis", c.a_au * AU);
        put(&mut f, "orbit.eccentricity", c.ecc);
        put(&mut f, "planet.insolation", flux_e * I_EARTH);
        put(&mut f, "planet.equilibrium_temperature", teq);
        put(&mut f, "planet.bond_albedo", c.albedo);
        put(&mut f, "system.age", sys.age_gyr * GYR_S);
        for (k, v) in [
            ("metal", c.metal),
            ("rock", c.rock),
            ("carbon", c.carbon),
            ("ice", c.ice),
            ("gas", c.gas),
        ] {
            put(&mut f, &format!("makeup.{k}"), v);
        }
        f.insert("orbit.host_kind".into(), Value::Text("star".into()));
        if !solid {
            return Ok(f);
        }
        let p_pa = p_bar * 1e5;
        let has_air = p_bar > 0.0;
        let frac = |pp: f64| {
            if c.pressure_bar > 0.0 {
                pp / c.pressure_bar
            } else {
                0.0
            }
        };
        put(&mut f, "planet.surface_temperature", tsurf);
        put(&mut f, "atmosphere.surface_pressure", p_pa);
        put(&mut f, "atmosphere.infrared_optical_depth", tau);
        put(&mut f, "atmosphere.pp.o2", frac(c.pp_o2_bar) * p_bar);
        put(&mut f, "atmosphere.pp.co2", frac(c.pp_co2_bar) * p_bar);
        f.insert(
            "atmosphere.main".into(),
            Value::Text(if has_air {
                c.air_main.clone()
            } else {
                "none".into()
            }),
        );
        // Surface dose: cosmic rays through the air column, plus the ground.
        let column_g_cm2 = if g > 0.0 { p_pa / g / 10.0 } else { 0.0 };
        put(
            &mut f,
            "surface.radiation_dose",
            s.dose_free_msv_yr * m::exp(-column_g_cm2 / s.dose_attenuation_g_cm2)
                + s.dose_ground_msv_yr,
        );
        // Where the surface liquid is liquid.
        let (liquid, ice) = if c.liquid_kind == "methane" {
            // Methane-ethane mixtures stay liquid from about 90 to 190 K at Titan-like pressures.
            let ok = (90.0..=190.0).contains(&tsurf) && p_bar >= 0.1;
            (if ok { c.liquid_inventory } else { 0.0 }, 0.0)
        } else {
            let boil = 1.0 / (1.0 / 373.15 - 8.314 * m::ln(p_bar.max(1e-12) / 1.01325) / 40660.0);
            if p_pa >= 611.657 && tsurf >= 273.15 && tsurf <= boil {
                (c.liquid_inventory, 0.0)
            } else if tsurf < 273.15 {
                (0.0, c.liquid_inventory)
            } else {
                (0.0, 0.0)
            }
        };
        // Ice-rich bodies below freezing are ice to the surface.
        let ice = if c.ice >= 0.15 && tsurf < 273.15 {
            1.0
        } else {
            ice
        };
        put(&mut f, "surface.liquid_fraction", liquid);
        put(&mut f, "surface.ice_fraction", ice);
        f.insert(
            "surface.liquid".into(),
            Value::Text(if liquid > 0.0 {
                c.liquid_kind.clone()
            } else {
                "none".into()
            }),
        );
        f.insert(
            "interior.subsurface_ocean".into(),
            Value::Bool(c.subsurface_ocean && ice >= 0.95),
        );
        Ok(f)
    }
}

fn put(f: &mut BTreeMap<String, Value>, k: &str, v: f64) {
    f.insert(k.to_string(), Value::Float(v));
}

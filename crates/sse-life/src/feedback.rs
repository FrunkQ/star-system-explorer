//! Box H (life design section 8, stage L2): life's fluxes and the planet's
//! answer at each time boundary.
//!
//! The life suite only reports fluxes (`history::StepFluxes`); a physics
//! boundary decides the consequences. `AirBox` is the stand-in physics side
//! until the planet layer's atmosphere and interior models exist: an oxygen
//! reservoir fed by organic carbon burial and hydrogen escape and drained by
//! the planet's reducing gases and by oxidative weathering, plus steady-state
//! methane and a CO2 drawdown index from land life. Oxygenation is then a
//! balance, not a date: oxygen builds only when burial and escape outpace the
//! reducing sinks (Catling and Claire 2005; Catling, Zahnle and McKay 2001).
//! Nothing here assumes life stabilises its world.
//!
//! Every O2 movement is booked in a ledger, so the exchanged material is
//! conserved at the boundary to rounding.

use crate::budget::BudgetParams;
use crate::history::{self, Boundary, HistoryParams, LifeHistory, StepFluxes, WorldTimeline};
use serde::{Deserialize, Serialize};
use sse_core::math as m;
use sse_core::Seed;
use std::collections::BTreeMap;

const YR_PER_MYR: f64 = 1.0e6;
const G_PER_MOL_C: f64 = 12.011;

/// Settings (`provides.other.life_feedback`), per world.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct AirParams {
    /// Moles of air (Earth: 1.8e20 for 1 bar).
    pub air_mol: f64,
    /// Surface area relative to Earth's (escape scales with it).
    pub area_rel: f64,
    /// Fraction of oxygenic production buried as organic carbon (net O2).
    pub burial_fraction: f64,
    /// Reducing gases from volcanoes and metamorphism, mol O2-equivalent a
    /// year, at `reductant_ref_myr`; e-folding time of their decline as the
    /// gases grow less reducing (negative: rising); a stand-in, see the pack sources.
    pub reductant_mol_yr: f64,
    pub reductant_ref_myr: f64,
    pub reductant_tau_myr: f64,
    /// Oxidative weathering at the present atmospheric level of O2, mol a
    /// year; it scales with sqrt(O2 / PAL).
    pub weathering_pal_mol_yr: f64,
    pub pal_mol: f64,
    /// Methane: the share of all production that decays through methanogens
    /// in oxic and anoxic worlds; lifetimes in years; the O2 level between.
    pub decay_methane_oxic: f64,
    pub decay_methane_anoxic: f64,
    pub ch4_life_oxic_yr: f64,
    pub ch4_life_anoxic_yr: f64,
    pub o2_switch_fraction: f64,
    /// Diffusion-limited hydrogen escape, mol O2-equivalent a year per unit
    /// CH4 mixing ratio for an Earth-sized world: 2.5e13 f_T(H2) H2 cm^-2 s^-1 with
    /// f_T = 2 f_CH4 (Catling et al. 2001; Catling and Claire 2005 eq. 4) is 1.34e16 mol H2
    /// a year, and two H2 lost leave one O2.
    pub escape_mol_yr_per_f: f64,
    /// Land life multiplies silicate weathering by this (Berner 1997).
    pub land_weathering_factor: f64,
    /// E3 competing sinks (evolution design section 7). Fire: COPSE's
    /// ignition index `clamp(slope * r - offset, 0, max)` from the O2 mixing
    /// ratio r, and the land-burial factor `k / (k - 1 + ignition)`, 1 at
    /// today's air. `fire_k` 0 switches fire off (a what-if).
    pub fire_k: f64,
    pub ignition_slope: f64,
    pub ignition_offset: f64,
    pub ignition_max: f64,
    /// Photorespiration: land production falls as `1 + a (1 - r / r_pal)`
    /// (COPSE's 1.5 - 0.5 O2/O2_0 is a = 0.5), floored at 0. 0 switches it off.
    pub photorespiration: f64,
    /// Aerobic decomposers: above today's air the burial share falls with O2
    /// as `(1 + h) / (1 + h r / r_pal)` (1 at and below today's level, whose
    /// burial efficiency the base fraction already holds); `h` 0 switches them off.
    pub decomposer_h: f64,
    /// E4: the soil box. When set, land weathering is its derived factor
    /// instead of `land_weathering_factor`, and soil carbon joins the ledger.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub soil: Option<crate::soil::SoilParams>,
    pub sources: Vec<String>,
}

impl Default for AirParams {
    fn default() -> Self {
        AirParams {
            air_mol: 1.8e20,
            area_rel: 1.0,
            burial_fraction: 0.001,
            reductant_mol_yr: 3.0e12,
            reductant_ref_myr: 4570.0,
            reductant_tau_myr: 2000.0,
            weathering_pal_mol_yr: 6.5e12,
            pal_mol: 3.78e19,
            decay_methane_oxic: 0.0025,
            decay_methane_anoxic: 0.5,
            ch4_life_oxic_yr: 9.1,
            ch4_life_anoxic_yr: 1000.0,
            o2_switch_fraction: 1.0e-5,
            escape_mol_yr_per_f: 6.7e15,
            land_weathering_factor: 4.0,
            fire_k: 3.0,
            ignition_slope: 48.0,
            ignition_offset: 9.08,
            ignition_max: 5.0,
            photorespiration: 0.5,
            decomposer_h: 1.0,
            soil: None,
            sources: vec![],
        }
    }
}

/// O2 moved in one step, moles. `change = burial + escape - reductants -
/// weathering` exactly, up to rounding.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize, Default)]
pub struct Ledger {
    pub burial: f64,
    pub escape: f64,
    pub reductants: f64,
    pub weathering: f64,
    pub change: f64,
    /// Reducing gas the oxygen could not meet (it stays in ocean and crust).
    pub reductant_unmet: f64,
    /// Escape beyond the reducing gases: it oxidises the crust, not the air.
    pub escape_to_crust: f64,
    /// Organic carbon buried, moles (future fossil fuels and shales).
    pub carbon_buried: f64,
    /// Of `burial`, the part from land production (the rest is marine).
    #[serde(default)]
    pub burial_land: f64,
    /// Burial that did not happen because of fire, photorespiration and
    /// aerobic decomposers, against today's-air factors (information only:
    /// the carbon went back to the air as CO2, so no O2 moved).
    #[serde(default)]
    pub burial_suppressed: f64,
    /// Organic carbon put into soil this step, moles (E4): its O2 stays in
    /// the air. Negative when soil loses carbon and takes the O2 back.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub soil_store: f64,
}

fn is_zero(x: &f64) -> bool {
    *x == 0.0
}

impl Ledger {
    pub fn residual(&self) -> f64 {
        self.change
            - (self.burial + self.soil_store + self.escape - self.reductants - self.weathering)
    }
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct AirPoint {
    pub at_myr: f64,
    pub o2_fraction: f64,
    pub ch4_ppm: f64,
    pub co2_rel: f64,
    pub reductant_mol_yr: f64,
    pub burial_mol_yr: f64,
    pub escape_mol_yr: f64,
    /// The fire factor on land burial this step (1 at today's air).
    #[serde(default = "one")]
    pub fire_factor: f64,
}

fn one() -> f64 {
    1.0
}

/// The stand-in physics side: one air reservoir.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct AirBox {
    pub p: AirParams,
    pub o2_mol: f64,
    pub ch4_mol: f64,
    pub co2_rel: f64,
    pub totals: Ledger,
    pub series: Vec<AirPoint>,
    /// Largest |residual| over all steps, moles.
    pub worst_residual: f64,
    /// E4: the soil and its record, when the soil box is on.
    #[serde(default, skip_serializing_if = "std::collections::BTreeMap::is_empty")]
    pub soil: BTreeMap<String, crate::soil::Soil>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub soil_series: Vec<crate::soil::SoilPoint>,
}

impl AirBox {
    pub fn new(p: AirParams) -> Self {
        AirBox {
            p,
            o2_mol: 0.0,
            ch4_mol: 0.0,
            co2_rel: 1.0,
            totals: Ledger::default(),
            series: vec![],
            worst_residual: 0.0,
            soil: BTreeMap::new(),
            soil_series: vec![],
        }
    }

    pub fn o2_fraction(&self) -> f64 {
        self.o2_mol / self.p.air_mol
    }

    /// COPSE's fire ignition index at an O2 mixing ratio.
    pub fn ignition(&self, r: f64) -> f64 {
        let p = &self.p;
        (p.ignition_slope * r - p.ignition_offset).clamp(0.0, p.ignition_max)
    }

    /// Land, marine burial factors at O2 fraction `f` (all 1 at today's air
    /// when fire, photorespiration and decomposers are calibrated), and the fire factor.
    pub fn burial_factors(&self, f: f64) -> (f64, f64, f64) {
        let p = &self.p;
        let r_pal = p.pal_mol / p.air_mol;
        let rel = if r_pal > 0.0 { f / r_pal } else { 0.0 };
        let fire = if p.fire_k > 0.0 {
            p.fire_k / (p.fire_k - 1.0 + self.ignition(f))
        } else {
            1.0
        };
        let photo = (1.0 + p.photorespiration * (1.0 - rel)).max(0.0);
        // Calibrated on today's burial efficiency, which already reflects
        // anoxic bottom waters: decomposers only cut burial further above it.
        let decomp = if rel > 1.0 {
            (1.0 + p.decomposer_h) / (1.0 + p.decomposer_h * rel)
        } else {
            1.0
        };
        (fire * photo * decomp, decomp, fire)
    }

    fn reductants_at(&self, t_myr: f64) -> f64 {
        self.p.reductant_mol_yr
            * m::exp((self.p.reductant_ref_myr - t_myr) / self.p.reductant_tau_myr)
    }
}

/// One step's outcome for a given end-of-step O2 guess.
struct Trial {
    o2_mol: f64,
    ch4_mol: f64,
    ledger: Ledger,
    demand: f64,
}

impl AirBox {
    /// Run one step with the oxic or anoxic chemistry set by `f_guess`, the
    /// O2 fraction assumed over the step.
    fn trial(&self, fl: &StepFluxes, f_guess: f64, soil_store: f64) -> Trial {
        let p = &self.p;
        let dt = fl.dt_myr * YR_PER_MYR;
        let anoxic_weight = 1.0 / (1.0 + f_guess / p.o2_switch_fraction);
        // Methane at steady state with this step's production (lifetimes are
        // far shorter than a step), lost to photochemistry and to escape.
        let decay =
            p.decay_methane_oxic + (p.decay_methane_anoxic - p.decay_methane_oxic) * anoxic_weight;
        let life_yr =
            p.ch4_life_oxic_yr + (p.ch4_life_anoxic_yr - p.ch4_life_oxic_yr) * anoxic_weight;
        let methanogen = fl.by_metabolism.get("methanogen").copied().unwrap_or(0.0);
        let ch4_flux = (methanogen + decay * fl.production_gc_yr) / G_PER_MOL_C;
        // Escape rate per mole of CH4 in the air, a year: diffusion-limited,
        // and only through an anoxic upper atmosphere.
        let k_esc = p.escape_mol_yr_per_f * p.area_rel * anoxic_weight / p.air_mol;
        let ch4_mol = ch4_flux / (1.0 / life_yr + k_esc);
        // O2 sources and sinks over the step, moles.
        // Burial competes with fire (land), photorespiration (land) and
        // aerobic decomposers (both): every factor falls as O2 rises, so the
        // source O2 depends on weakens as O2 builds (E3).
        let oxygenic = fl.by_metabolism.get("oxygenic").copied().unwrap_or(0.0);
        let land = fl
            .land_by_metabolism
            .get("oxygenic")
            .copied()
            .unwrap_or(0.0)
            .min(oxygenic);
        let (f_land, f_sea, _) = self.burial_factors(f_guess);
        let base = p.burial_fraction / G_PER_MOL_C * dt;
        let burial_land = base * land * f_land;
        let burial = burial_land + base * (oxygenic - land) * f_sea;
        let suppressed = (base * oxygenic - burial).max(0.0);
        // Hydrogen lost to space oxidises the planet: it cancels reducing
        // gases but never makes free O2 beyond them (the rest oxidises the crust).
        let demand = self.reductants_at(fl.at_myr) * dt;
        let escape_all = k_esc * ch4_mol * dt;
        let escape = escape_all.min(demand);
        // Soil can only take back the O2 there is (E4).
        let soil_store = soil_store.max(-(self.o2_mol + burial + escape));
        let avail = self.o2_mol + burial + escape + soil_store;
        let mut l = Ledger {
            soil_store,
            burial,
            escape,
            escape_to_crust: escape_all - escape,
            carbon_buried: burial,
            burial_land,
            burial_suppressed: suppressed,
            ..Default::default()
        };
        let o2_mol = if avail <= demand {
            l.reductants = avail;
            l.reductant_unmet = demand - avail;
            0.0
        } else {
            // x^2 + b x = c with x = sqrt(O2_new): weathering ~ sqrt(O2), implicit.
            let c = avail - demand;
            let b = p.weathering_pal_mol_yr * dt / m::sqrt(p.pal_mol);
            let x = 0.5 * (m::sqrt(b * b + 4.0 * c) - b);
            l.reductants = demand;
            l.weathering = c - x * x;
            x * x
        };
        l.change = o2_mol - self.o2_mol;
        Trial {
            o2_mol,
            ch4_mol,
            ledger: l,
            demand,
        }
    }
}

/// Bisection steps for the step's oxic or anoxic state (fixed, so runs repeat).
const STATE_ITERATIONS: usize = 60;

impl Boundary for AirBox {
    fn step(&mut self, fl: &StepFluxes) -> BTreeMap<String, f64> {
        let dt = fl.dt_myr * YR_PER_MYR;
        // The chemistry over a step depends on the O2 it ends with: more O2
        // means less methane and less escape, so less O2. That makes the end
        // state a unique fixed point; find it by bisection instead of letting
        // an explicit switch flip between oxic and anoxic every step.
        let air = self.p.air_mol;
        // E4: the soil for this step, from last step's land production and
        // this step's start oxygen (explicit in O2, so the bisection below
        // sees a fixed store).
        let new_soil: Option<BTreeMap<String, crate::soil::Soil>> =
            self.p.soil.as_ref().map(|sp| {
                let r_pal = self.p.pal_mol / self.p.air_mol;
                let o2_rel = if r_pal > 0.0 {
                    self.o2_fraction() / r_pal
                } else {
                    0.0
                };
                fl.land
                    .iter()
                    .map(|r| {
                        let prev = self.soil.get(&r.key).cloned().unwrap_or_default();
                        let s = sp.step(
                            &prev,
                            &crate::soil::SoilStep {
                                dt_myr: fl.dt_myr,
                                land_area_m2: r.area_m2,
                                temperature_k: r.temperature_k.unwrap_or(sp.t_ref_k),
                                land_npp_gc_yr: r.npp_gc_yr,
                                o2_rel,
                            },
                        );
                        (r.key.clone(), s)
                    })
                    .collect()
            });
        let held: f64 = self.soil.values().map(|s| s.carbon_mol).sum();
        let store = new_soil.as_ref().map_or(0.0, |s| {
            s.values().map(|x| x.carbon_mol).sum::<f64>() - held
        });
        let hi0 = self.trial(fl, 0.0, store).o2_mol / air;
        let (mut lo, mut hi) = (0.0_f64, hi0);
        for _ in 0..STATE_ITERATIONS {
            let mid = 0.5 * (lo + hi);
            if self.trial(fl, mid, store).o2_mol / air > mid {
                lo = mid;
            } else {
                hi = mid;
            }
        }
        let t = self.trial(fl, 0.5 * (lo + hi), store);
        let l = t.ledger;
        self.o2_mol = t.o2_mol;
        self.ch4_mol = t.ch4_mol;
        self.worst_residual = self.worst_residual.max(l.residual().abs());
        for (tot, v) in [
            (&mut self.totals.burial, l.burial),
            (&mut self.totals.escape, l.escape),
            (&mut self.totals.reductants, l.reductants),
            (&mut self.totals.weathering, l.weathering),
            (&mut self.totals.change, l.change),
            (&mut self.totals.reductant_unmet, l.reductant_unmet),
            (&mut self.totals.escape_to_crust, l.escape_to_crust),
            (&mut self.totals.carbon_buried, l.carbon_buried),
            (&mut self.totals.burial_land, l.burial_land),
            (&mut self.totals.burial_suppressed, l.burial_suppressed),
            (&mut self.totals.soil_store, l.soil_store),
        ] {
            *tot += v;
        }
        // E4: the soil keeps the carbon the air could give it.
        if let (Some(mut regions), Some(sp)) = (new_soil, self.p.soil.clone()) {
            // When the air could not give all the carbon back (soil losing
            // carbon with too little O2), every region keeps the same share
            // of what it was going to lose.
            let want = store;
            if want != l.soil_store && want < 0.0 {
                let keep = 1.0 - l.soil_store / want;
                for (k, s) in regions.iter_mut() {
                    let before = self.soil.get(k).map_or(0.0, |x| x.carbon_mol);
                    if s.carbon_mol < before {
                        s.carbon_mol += (before - s.carbon_mol) * keep;
                    }
                }
            }
            for (k, s) in regions.iter_mut() {
                if let Some(r) = fl.land.iter().find(|r| &r.key == k) {
                    if r.area_m2 > 0.0 {
                        s.organic_c_kg_m2 = s.carbon_mol * 12.011 / 1000.0 / r.area_m2;
                    }
                }
            }
            let all = crate::soil::Soil::area_mean(&regions, &fl.land);
            self.co2_rel = 1.0 / all.weathering_factor;
            self.soil_series.push(crate::soil::SoilPoint {
                at_myr: fl.at_myr + fl.dt_myr,
                soil: all,
                delta_t_k: sp.temperature_change_k(self.co2_rel),
            });
            self.soil = regions;
        }
        // Land life speeds silicate weathering and draws CO2 down.
        self.co2_rel = if self.p.soil.is_some() {
            self.co2_rel
        } else if fl.on_land {
            1.0 / self.p.land_weathering_factor
        } else {
            1.0
        };
        let f = self.o2_fraction();
        let f_ch4 = self.ch4_mol / air;
        self.series.push(AirPoint {
            at_myr: fl.at_myr + fl.dt_myr,
            o2_fraction: f,
            ch4_ppm: f_ch4 * 1.0e6,
            co2_rel: self.co2_rel,
            reductant_mol_yr: t.demand / dt.max(1.0),
            burial_mol_yr: l.burial / dt.max(1.0),
            escape_mol_yr: l.escape / dt.max(1.0),
            fire_factor: self.burial_factors(f).2,
        });
        [
            ("o2_fraction".to_string(), f),
            ("ch4_ppm".to_string(), f_ch4 * 1.0e6),
            ("co2_rel".to_string(), self.co2_rel),
        ]
        .into_iter()
        .collect()
    }
}

/// Everything a coupled run is made from: run it again from this and the
/// result is identical.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct CoupledManifest {
    pub format: String,
    pub world: WorldTimeline,
    pub budget: BudgetParams,
    pub history: HistoryParams,
    pub air: AirParams,
    pub seed: String,
}

pub const MANIFEST_FORMAT: &str = "sse.life-coupled/1";

/// A world counts as oxygenated once O2 holds above 1% of the air for at
/// least 200 Myr (or to the end of the run); brief whiffs do not count.
pub const OXYGENATED_FRACTION: f64 = 0.01;
pub const SUSTAINED_MYR: f64 = 200.0;

/// Start of the first stretch at or above `level` lasting `hold` Myr, or
/// lasting to the end of the series.
pub fn sustained_rise(series: &[AirPoint], level: f64, hold: f64) -> Option<f64> {
    let mut start: Option<f64> = None;
    for p in series {
        if p.o2_fraction >= level {
            let s = *start.get_or_insert(p.at_myr);
            if p.at_myr - s >= hold {
                return Some(s);
            }
        } else {
            start = None;
        }
    }
    start
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct CoupledRun {
    pub history: LifeHistory,
    pub air: AirBox,
    /// When O2 first held above 1% of the air (`sustained_rise`), Myr; none: never.
    pub oxygenated_myr: Option<f64>,
    pub manifest_hash: String,
}

/// Run life and the air box together. The world timeline's own
/// `o2_fraction` facts are overridden by the box at every step.
pub fn coupled(man: &CoupledManifest) -> Result<CoupledRun, String> {
    if man.format != MANIFEST_FORMAT {
        return Err(format!("manifest format must be {MANIFEST_FORMAT}"));
    }
    let mut air = AirBox::new(man.air.clone());
    let h = history::history_with(
        &man.world,
        &man.budget,
        &man.history,
        Seed::from_text(&man.seed),
        Some(&mut air),
    )?;
    let oxygenated_myr = sustained_rise(&air.series, OXYGENATED_FRACTION, SUSTAINED_MYR);
    Ok(CoupledRun {
        history: h,
        air,
        oxygenated_myr,
        manifest_hash: sse_core::ContentHash::of(man)
            .map_err(|e| format!("{e:?}"))?
            .hex(),
    })
}

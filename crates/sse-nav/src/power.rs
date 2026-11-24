//! The power ledger (RK9, Atomic Rockets plan section 11): every part of a
//! construct is a converter on four buses, electric (W), heat (W at a
//! temperature), propellant and jet. A power source puts watts on the
//! electric bus by its law (constant, sunlight, decay, fuel); loads and
//! electric engines take them; every loss ends as heat at its part's
//! `reject_k`. Nothing here forbids anything: where the books do not close
//! (an engine whose jet carries more power than it is given) the shortfall is
//! a row from "Nowhere", for the GM to fix or keep as stated fiction.
//!
//! The ledger is rows of (from, to, watts, kind), the shape the general Flow
//! view draws (plan section 16), and a few readings: thrust, power to spare,
//! heat to reject by temperature, and what came from nowhere.

use crate::G0;
use serde::{Deserialize, Serialize};
use sse_core::math as m;
use std::collections::BTreeMap;

/// Seconds in a Julian year, for half-lives given in years.
const YEAR_S: f64 = sse_core::sheet::YR_JULIAN;

/// How a source's output changes (plan 11.3).
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize, Default)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Law {
    /// A reactor: the same output at any time and place.
    #[default]
    Constant,
    /// A solar array: its watts at a reference flux, scaled by the flux where it is.
    Sunlight { ref_flux_w_m2: f64 },
    /// A radioisotope unit: halves every `half_life_y` from `since_s` (universe seconds).
    Decay { half_life_y: f64, since_s: i64 },
    /// A fuel cell: runs while its fuel lasts (the propellant bus; not drawn down here).
    Fuel { fuel: String },
}

/// A power source on a construct.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct PowerSource {
    #[serde(default)]
    pub id: String,
    /// An advance id ("solar-array", "fission-reactor", "radioisotope",
    /// "fuel-cell"), or the builder's own words when none matches.
    #[serde(default)]
    pub kind: String,
    /// Electric output under its law's reference conditions, W.
    pub electric_w: f64,
    /// Heat given off making it, W (none: not stated, so not checked).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub thermal_w: Option<f64>,
    /// Temperature its heat leaves at, K.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reject_k: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mass_t: Option<f64>,
    #[serde(default)]
    pub law: Law,
}

/// Where an engine's energy comes from (plan 11.2).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EnergyFrom {
    /// The propellant carries its own energy (chemical, nuclear thermal, fusion, antimatter).
    Fuel,
    /// Drawn from the electric bus (ion, Hall, plasma, arcjet, mass driver).
    Electric,
    /// Sent by another construct (laser thermal, sails).
    Beam,
    /// Taken from the star where the ship is (solar sail, solar thermal).
    Sunlight,
}

/// An engine design (a pack's, or the builder's own).
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct EngineDef {
    #[serde(default)]
    pub name: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub fuels: Vec<String>,
    /// Thrust at full power, N.
    pub thrust_n: f64,
    pub exhaust_km_s: f64,
    pub energy_from: EnergyFrom,
    /// Electric draw at full thrust, W (electric engines).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub electric_w: Option<f64>,
    /// Jet power over energy in (none: taken as 1, so the check is the most generous).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub efficiency: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reject_k: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mass_t: Option<f64>,
    /// Gears: exhaust speeds it can run at, km/s, trading thrust for economy at fixed power.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exhaust_range_km_s: Option<[f64; 2]>,
}

impl EngineDef {
    /// Jet power at full thrust: half of thrust times exhaust speed, W.
    pub fn jet_w(&self) -> f64 {
        0.5 * self.thrust_n * self.exhaust_km_s * 1000.0
    }
    fn eff(&self) -> f64 {
        self.efficiency.unwrap_or(1.0).clamp(1e-6, 1.0)
    }
}

/// An engine fitted to a construct: a pack's engine by id, or one written in place.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum EngineRef {
    Id(String),
    Own(Box<EngineDef>),
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct EngineMount {
    pub engine: EngineRef,
    #[serde(default = "one")]
    pub count: u32,
}
fn one() -> u32 {
    1
}

/// Something that uses electricity and ends as heat (quarters, sensors, radios, lasers).
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Load {
    pub name: String,
    pub electric_w: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reject_k: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mass_t: Option<f64>,
    /// Off: listed, drawing nothing.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub off: bool,
}

/// One band of the Flow view: `amount_w` watts from `from` to `to`.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Flow {
    pub from: String,
    pub to: String,
    pub amount_w: f64,
    /// "electric", "heat", "jet" or "fuel".
    pub kind: String,
}

/// An engine's share of the ledger.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct EngineReading {
    pub name: String,
    pub count: u32,
    pub energy_from: EnergyFrom,
    /// Thrust it makes here, now, with the power it gets, N (all of them).
    pub thrust_n: f64,
    pub exhaust_km_s: f64,
    /// Electric power it gets, W (all of them).
    pub electric_w: f64,
    /// Jet power, W (all of them).
    pub jet_w: f64,
    /// Jet power its stated figures need beyond what it is given, W (0 when the books close).
    pub nowhere_w: f64,
}

/// The ledger and its readings.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Ledger {
    pub flows: Vec<Flow>,
    /// Electric power made, W.
    pub made_w: f64,
    /// Electric power the loads and engines would take at full, W.
    pub wanted_w: f64,
    /// Made less taken, W (negative: short, and the engines throttle).
    pub spare_w: f64,
    pub engines: Vec<EngineReading>,
    pub thrust_n: f64,
    /// Heat to reject, W, by the temperature it leaves at (K, rounded).
    pub heat_by_k: BTreeMap<u32, f64>,
    /// Power the books cannot account for, W.
    pub nowhere_w: f64,
    pub why: Vec<String>,
}

/// Where and when the ledger is read.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Conditions {
    /// Starlight where the construct is, W/m².
    pub flux_w_m2: f64,
    /// Universe seconds.
    pub t_s: i64,
}

/// Default temperature heat leaves at when a part gives none, K (a cool radiator).
const DEFAULT_REJECT_K: f64 = 300.0;

/// A source's electric output here and now, W.
pub fn source_w(s: &PowerSource, c: Conditions) -> f64 {
    match &s.law {
        Law::Constant | Law::Fuel { .. } => s.electric_w,
        Law::Sunlight { ref_flux_w_m2 } => s.electric_w * c.flux_w_m2 / ref_flux_w_m2.max(1e-9),
        Law::Decay {
            half_life_y,
            since_s,
        } => {
            let years = (c.t_s - since_s).max(0) as f64 / YEAR_S;
            s.electric_w * m::pow(0.5, years / half_life_y.max(1e-9))
        }
    }
}

fn heat(map: &mut BTreeMap<u32, f64>, k: Option<f64>, w: f64) {
    if w > 0.0 {
        *map.entry(k.unwrap_or(DEFAULT_REJECT_K).round().max(0.0) as u32)
            .or_insert(0.0) += w;
    }
}

/// Resolve an engine reference against the pack's engines.
pub fn engine_of<'a>(
    r: &'a EngineRef,
    defs: &'a BTreeMap<String, EngineDef>,
) -> Option<&'a EngineDef> {
    match r {
        EngineRef::Id(id) => defs.get(id),
        EngineRef::Own(d) => Some(d),
    }
}

/// The ledger for a construct's sources, engines and loads, here and now.
/// Loads are served first; electric engines share what is left in proportion
/// to their draw, and their thrust follows the power they get:
/// F = 2 η P / v_e.
pub fn ledger(
    sources: &[PowerSource],
    mounts: &[EngineMount],
    loads: &[Load],
    defs: &BTreeMap<String, EngineDef>,
    c: Conditions,
) -> Ledger {
    let mut flows = vec![];
    let mut heat_by_k = BTreeMap::new();
    let mut why = vec![];
    let mut made = 0.0;
    for s in sources {
        let w = source_w(s, c);
        made += w;
        let name = if s.id.is_empty() {
            s.kind.clone()
        } else {
            s.id.clone()
        };
        flows.push(Flow {
            from: name.clone(),
            to: "electric".into(),
            amount_w: w,
            kind: "electric".into(),
        });
        if let Some(t) = s.thermal_w {
            // The plant's own loss, scaled with its output like the electricity.
            let th = t * w / s.electric_w.max(1e-12);
            heat(&mut heat_by_k, s.reject_k, th);
            flows.push(Flow {
                from: name,
                to: "radiators".into(),
                amount_w: th,
                kind: "heat".into(),
            });
        }
        match &s.law {
            Law::Sunlight { ref_flux_w_m2 } => why.push(format!(
                "{}: {:.0} W at {:.0} W/m² of starlight, {:.0} W here ({:.0} W/m²)",
                s.kind, s.electric_w, ref_flux_w_m2, w, c.flux_w_m2
            )),
            Law::Decay { half_life_y, .. } => why.push(format!(
                "{}: {:.0} W when made, {:.0} W now (half-life {:.1} years)",
                s.kind, s.electric_w, w, half_life_y
            )),
            _ => {}
        }
    }
    // Loads first.
    let load_w: f64 = loads.iter().filter(|l| !l.off).map(|l| l.electric_w).sum();
    let mut left = made;
    for l in loads.iter().filter(|l| !l.off) {
        let got = l.electric_w.min(left.max(0.0));
        left -= got;
        flows.push(Flow {
            from: "electric".into(),
            to: l.name.clone(),
            amount_w: got,
            kind: "electric".into(),
        });
        heat(&mut heat_by_k, l.reject_k, got);
        flows.push(Flow {
            from: l.name.clone(),
            to: "radiators".into(),
            amount_w: got,
            kind: "heat".into(),
        });
    }
    // Electric engines share the rest.
    let resolved: Vec<(&EngineMount, Option<&EngineDef>)> = mounts
        .iter()
        .map(|mt| (mt, engine_of(&mt.engine, defs)))
        .collect();
    let electric_want: f64 = resolved
        .iter()
        .filter_map(|(mt, d)| {
            d.filter(|d| d.energy_from == EnergyFrom::Electric)
                .map(|d| d.electric_w.unwrap_or(d.jet_w() / d.eff()) * mt.count as f64)
        })
        .sum();
    let share = if electric_want > 0.0 {
        (left.max(0.0) / electric_want).min(1.0)
    } else {
        1.0
    };
    let mut engines = vec![];
    let mut thrust = 0.0;
    let mut nowhere = 0.0;
    for (mt, d) in resolved {
        let Some(d) = d else {
            if let EngineRef::Id(id) = &mt.engine {
                why.push(format!(
                    "engine {id} is not in any stacked pack, so it gives no thrust"
                ));
            }
            continue;
        };
        let n = mt.count as f64;
        let (got_w, jet_w, short_w, f) = match d.energy_from {
            EnergyFrom::Electric => {
                let draw = d.electric_w.unwrap_or(d.jet_w() / d.eff());
                let got = draw * share * n;
                // The books: the stated jet power against what the stated draw can give.
                let short = (d.jet_w() - d.eff() * draw).max(0.0) * n;
                // Thrust follows the power it gets, as its stated figures have it at full draw.
                let f = d.thrust_n * share * n;
                (got, d.jet_w() * share * n, short * share, f)
            }
            _ => (0.0, d.jet_w() * n, 0.0, d.thrust_n * n),
        };
        let name = if d.name.is_empty() {
            "engine".to_string()
        } else {
            d.name.clone()
        };
        if got_w > 0.0 {
            flows.push(Flow {
                from: "electric".into(),
                to: name.clone(),
                amount_w: got_w,
                kind: "electric".into(),
            });
        }
        if d.energy_from == EnergyFrom::Fuel {
            flows.push(Flow {
                from: "propellant".into(),
                to: name.clone(),
                amount_w: jet_w / d.eff(),
                kind: "fuel".into(),
            });
        }
        if short_w > 0.0 {
            flows.push(Flow {
                from: "Nowhere".into(),
                to: name.clone(),
                amount_w: short_w,
                kind: "electric".into(),
            });
            why.push(format!(
                "{name}: its jet carries {:.2} MW but it is given {:.2} MW{}, so {:.2} MW comes from nowhere",
                d.jet_w() * n / 1e6,
                d.electric_w.unwrap_or(0.0) * n / 1e6,
                d.efficiency.map_or(String::new(), |e| format!(" at {:.0}% efficiency", e * 100.0)),
                short_w / 1e6
            ));
        }
        flows.push(Flow {
            from: name.clone(),
            to: "jet".into(),
            amount_w: jet_w,
            kind: "jet".into(),
        });
        let loss = match d.energy_from {
            EnergyFrom::Electric => (got_w - (jet_w - short_w)).max(0.0),
            _ => jet_w * (1.0 / d.eff() - 1.0),
        };
        heat(&mut heat_by_k, d.reject_k, loss);
        if loss > 0.0 {
            flows.push(Flow {
                from: name.clone(),
                to: "radiators".into(),
                amount_w: loss,
                kind: "heat".into(),
            });
        }
        if share < 1.0 && d.energy_from == EnergyFrom::Electric {
            why.push(format!(
                "{name}: given {:.0}% of its power, so {:.0}% of its thrust",
                share * 100.0,
                share * 100.0
            ));
        }
        thrust += f;
        nowhere += short_w;
        engines.push(EngineReading {
            name,
            count: mt.count,
            energy_from: d.energy_from,
            thrust_n: f,
            exhaust_km_s: d.exhaust_km_s,
            electric_w: got_w,
            jet_w,
            nowhere_w: short_w,
        });
    }
    let wanted = load_w + electric_want;
    Ledger {
        flows,
        made_w: made,
        wanted_w: wanted,
        spare_w: made - wanted,
        engines,
        thrust_n: thrust,
        heat_by_k,
        nowhere_w: nowhere,
        why,
    }
}

/// Thrust from power for an electric engine: F = 2 η P / v_e (plan 11.2), N.
pub fn thrust_from_power(power_w: f64, efficiency: f64, exhaust_km_s: f64) -> f64 {
    2.0 * efficiency * power_w / (exhaust_km_s * 1000.0).max(1e-9)
}

/// The ship's readings from its fitted engines: acceleration (g) at a mass
/// in tonnes, and the thrust-weighted exhaust speed (km/s).
pub fn readings(l: &Ledger, mass_t: f64) -> (f64, f64) {
    let a = l.thrust_n / (mass_t * 1000.0).max(1e-9) / G0;
    let ve = if l.thrust_n > 0.0 {
        l.engines
            .iter()
            .map(|e| e.thrust_n * e.exhaust_km_s)
            .sum::<f64>()
            / l.thrust_n
    } else {
        0.0
    };
    (a, ve)
}

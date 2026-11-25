//! ATMOS, advice on a world's air (Alex, 23 Nov: mantle redox sets the
//! outgassed mix; a "the science expects" line beside hand-set air; a
//! "doesn't go together" note as advice, never a refusal; a "Suggest an
//! atmosphere" button; UV photochemistry later). Editing design 11A.
//!
//! - **Expected.** The mantle's oxygen fugacity against iron-wüstite
//!   (`sse.interior.redox_iw`, from formation's `interior.redox_iw` when the
//!   world has a formation run, else from its core mass fraction alone)
//!   splits the hydrogen and carbon a magma ocean gives off into H2, H2O,
//!   CO, CO2 and CH4 (`sse.atmosphere.outgassed_mix`). That is the air the
//!   rock makes: escape, rain and weathering after it are not run here
//!   (fudge F-ATMOS), so a hand-set air that has aged far from it is told,
//!   not refused.
//! - **Differs.** Each gas whose hand-set share is more than `differs_dex`
//!   from the expected one, or that the rock does not give off at all.
//! - **Clashes.** E8's available Gibbs energy of the hand-set air
//!   (`sse_life::disequilibrium`), credited to each reaction: a pair that
//!   reacts away, with how long it lasts where a lifetime is known (Earth's
//!   sunlight; photochemistry for other stars is a later package). One line
//!   of advice each: "oxygen with methane lasts only while something keeps
//!   making them".
//! - **Suggest.** One `set_authored_batch` that writes the expected mix and
//!   pressure, for the "Suggest an atmosphere" button. Nothing is written by
//!   this op.

use crate::protocol::Response;
use crate::{err, json_to_value, Kernel};
use sse_core::cbor::Value;
use sse_flow::Atom;
use std::collections::BTreeMap;

type Ports = BTreeMap<String, f64>;

/// The five gases the outgassing block gives, in its port order.
pub const OUTGASSED: [&str; 5] = ["h2", "h2o", "co", "co2", "ch4"];

/// Settings (defaults; a pack may override any field later).
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct AdviceParams {
    /// A hand-set share this far (log10) from the expected one is listed.
    pub differs_dex: f64,
    /// Shares below this count as none.
    pub trace: f64,
    /// Without a formation ledger: carbon per hydrogen by mass in what a
    /// mantle holds, the bulk silicate Earth's (Hirschmann and Dasgupta
    /// 2009, Chemical Geology 262, 4: C/H 0.99 ± 0.42, doi:10.1016/j.chemgeo.2009.02.008).
    pub carbon_per_hydrogen: f64,
    /// How long a pair lasts in Earth's sunlit air, by reaction, years,
    /// with the source. A reaction not listed has no quick sink without
    /// life or a catalyst.
    pub lifetimes: BTreeMap<String, (f64, String)>,
    /// Diffusion-limited hydrogen escape on Earth, H2 molecules per cm² per
    /// second per unit mixing ratio (Hunten 1973, doi:10.1175/1520-0469(1973)030<1481:TEOLGF>2.0.CO;2;
    /// the 2.5e13 of Catling and Kasting 2017), scaled by gravity over
    /// standard gravity (the sheet's g0); the column is the air's own weight
    /// over its mean molar mass (the sheet's molar masses).
    pub escape_flux: f64,
    /// Hydrogen that leaves faster than this is left out of the suggestion.
    pub escape_keep_s: f64,
}

impl Default for AdviceParams {
    fn default() -> Self {
        let l = |yr: f64, src: &str| (yr, src.to_string());
        AdviceParams {
            differs_dex: 1.0,
            trace: 1.0e-9,
            carbon_per_hydrogen: 0.99,
            escape_flux: 2.5e13,
            escape_keep_s: 1.0e8 * crate::clock::JULIAN_YEAR_S,
            lifetimes: [
                (
                    "methane-oxidation".to_string(),
                    l(
                        9.1,
                        "methane, 9.1 ± 0.9 yr in today's air: Prather, Holmes and Hsu 2012, doi:10.1029/2012GL051440",
                    ),
                ),
                (
                    "knallgas".to_string(),
                    l(
                        2.0,
                        "hydrogen, about 2 yr in today's air (soil uptake and OH): Novelli et al. 1999, doi:10.1029/1999JD900788",
                    ),
                ),
                (
                    "co-oxidation".to_string(),
                    l(
                        0.17,
                        "carbon monoxide, about two months in today's air (OH): Novelli, Masarie and Lang 1998, doi:10.1029/98JD01366",
                    ),
                ),
            ]
            .into_iter()
            .collect(),
        }
    }
}

fn num(v: &Value) -> Option<f64> {
    match v {
        Value::Float(f) => Some(*f),
        Value::Int(i) => Some(*i as f64),
        _ => None,
    }
}

fn get(row: &BTreeMap<String, Value>, k: &str) -> Option<f64> {
    row.get(k).and_then(num)
}

/// Run one formula block by id: its inputs and outputs by port.
fn run(id: &str, inputs: &[f64]) -> Result<(Ports, Ports), String> {
    let a = sse_atoms::formulas::loaded()
        .iter()
        .find(|a| a.decl().id == id)
        .ok_or_else(|| format!("{id} is not loaded"))?;
    let d = a.decl();
    let p: Vec<f64> = d.params.iter().map(|p| p.default).collect();
    let out = a.eval(inputs, &p).map_err(|e| e.0)?;
    let ins = d
        .inputs
        .iter()
        .zip(inputs)
        .map(|(p, v)| (p.name.clone(), *v))
        .collect();
    let outs = d
        .outputs
        .iter()
        .zip(out)
        .map(|(p, v)| (p.name.clone(), v))
        .collect();
    Ok((ins, outs))
}

fn block(id: &str, ins: Ports, outs: Ports) -> serde_json::Value {
    serde_json::json!({ "block": id, "inputs": ins, "results": outs })
}

fn gas_name(g: &str) -> &str {
    match g {
        "h2" => "hydrogen",
        "h2o" => "water",
        "co" => "carbon monoxide",
        "co2" => "carbon dioxide",
        "ch4" => "methane",
        "o2" => "oxygen",
        "n2" => "nitrogen",
        "ar" => "argon",
        "he" => "helium",
        "nh3" => "ammonia",
        "so2" => "sulphur dioxide",
        "h2s" => "hydrogen sulphide",
        _ => g,
    }
}

/// Mean molar mass of a mix, kg/mol, over the gases the sheet has (nitrogen's
/// when none is known).
fn mean_molar_mass(x: &BTreeMap<String, f64>) -> f64 {
    use sse_core::sheet::molar_mass as mm;
    let m = |g: &str| match g {
        "h2" => Some(mm::H2),
        "h2o" => Some(mm::H2O),
        "co" => Some(mm::CO),
        "co2" => Some(mm::CO2),
        "ch4" => Some(mm::CH4),
        "nh3" => Some(mm::NH3),
        "n2" => Some(mm::N2),
        "o2" => Some(mm::O2),
        "ar" => Some(mm::AR),
        "he" => Some(mm::HE),
        "ne" => Some(mm::NE),
        "kr" => Some(mm::KR),
        "xe" => Some(mm::XE),
        "so2" => Some(mm::SO2),
        "h2s" => Some(mm::H2S),
        _ => None,
    };
    let (mut w, mut n) = (0.0, 0.0);
    for (g, f) in x {
        if let Some(mg) = m(g) {
            w += f * mg;
            n += f;
        }
    }
    if n > 0.0 {
        w / n
    } else {
        mm::N2
    }
}

fn years(yr: f64) -> String {
    if yr < 1.0 {
        format!("about {:.0} days", yr * 365.25)
    } else if yr < 100.0 {
        format!("about {yr:.0} years")
    } else {
        format!("about {yr:.0e} years")
    }
}

/// The advice for one body from its facts. `address` names it for the
/// suggested batch; `table` is the stack's life chemistry (E0).
pub fn advise(
    address: &str,
    row: &BTreeMap<String, Value>,
    table: &sse_life::chemistry::ThermoTable,
    dp: &sse_life::disequilibrium::DiseqParams,
    p: &AdviceParams,
) -> Result<serde_json::Value, Response> {
    use sse_core::sheet::{atomic_weight as aw, molar_mass as mm};
    let mut blocks = vec![];
    let mut notes: Vec<String> = vec![];
    let bad = |e: String| err("block_failed", e);
    // The mantle's redox: formation's figure, else the core alone.
    let (redox, redox_from) = match get(row, "interior.redox_iw") {
        Some(r) => (Some(r), "formation"),
        None => match get(row, "interior.core_mass_fraction") {
            Some(cmf) => {
                let (ins, outs) = run(
                    "sse.interior.redox_iw",
                    &[cmf, 0.0, aw::FE / 1000.0, mm::FEO, aw::C / 1000.0],
                )
                .map_err(bad)?;
                let r = outs.get("redox").copied();
                blocks.push(block("sse.interior.redox_iw", ins, outs));
                notes.push(
                    "The mantle's redox is read from the core alone: this world has no formation run, so the carbon that smelts iron is not known.".into(),
                );
                (r, "core")
            }
            None => (None, "unknown"),
        },
    };
    // What the rock gives off, and its weight.
    let mass = get(row, "body.mass");
    let radius = get(row, "body.radius");
    let water = get(row, "early.surface_water");
    let co2_pa = get(row, "early.co2_partial_pressure");
    let set_pa =
        get(row, "atmosphere.surface_pressure").or_else(|| get(row, "body.surface_pressure"));
    let (h_mol, c_mol, gas_pa) = match (water, co2_pa, mass, radius) {
        (Some(w), Some(c), Some(m), Some(r)) if r > 0.0 => {
            let area = 4.0 * sse_core::math::PI * r * r;
            let g = sse_core::sheet::G * m / (r * r);
            let co2_kg = c * area / g.max(1e-30);
            (w / mm::H2O, co2_kg / mm::CO2, Some(c + w * g / area))
        }
        _ => {
            notes.push(
                "How much hydrogen and carbon came out is not known (no formation run): the mix uses the bulk silicate Earth's carbon per hydrogen, and the expected pressure is left open.".into(),
            );
            // One mole of hydrogen as H2, carbon by the mantle's ratio.
            let c = p.carbon_per_hydrogen * (2.0 * aw::H / 1000.0) / (aw::C / 1000.0);
            (1.0, c, None)
        }
    };
    let mut expected: BTreeMap<String, f64> = BTreeMap::new();
    if let Some(r) = redox {
        let at_pa = gas_pa.or(set_pa).unwrap_or(1.0e5);
        let (ins, outs) =
            run("sse.atmosphere.outgassed_mix", &[r, h_mol, c_mol, at_pa]).map_err(bad)?;
        for g in OUTGASSED {
            if let Some(x) = outs.get(g) {
                if *x > p.trace {
                    expected.insert(g.to_string(), *x);
                }
            }
        }
        blocks.push(block("sse.atmosphere.outgassed_mix", ins, outs));
    } else {
        notes.push("Without the mantle's redox or the core's share, nothing is expected: set either to see what the rock gives off.".into());
    }
    // The hand-set air.
    let set: BTreeMap<String, f64> = row
        .iter()
        .filter_map(|(k, v)| {
            let g = k.strip_prefix("atmosphere.composition.")?;
            num(v).filter(|x| *x > 0.0).map(|x| (g.to_string(), x))
        })
        .collect();
    let t_k = get(row, "planet.surface_temperature")
        .or_else(|| get(row, "surface.temperature"))
        .unwrap_or(288.15);
    let seas = get(row, "body.surface_water_oceans").is_some_and(|x| x > 0.0)
        || matches!(row.get("body.oceans"), Some(Value::Bool(true)));
    let gravity = match (mass, radius) {
        (Some(m), Some(r)) if r > 0.0 => Some(sse_core::sheet::G * m / (r * r)),
        _ => None,
    };
    // How the hand-set air differs: the mantle's redox sets how much of the
    // hydrogen and carbon comes out reduced (H2 to H2O, CO to CO2), and some
    // gases do not come from the magma ocean at all.
    let mut differs = vec![];
    if !expected.is_empty() && !set.is_empty() {
        for (red, ox, what) in [("h2", "h2o", "hydrogen"), ("co", "co2", "carbon")] {
            let (er, eo) = (
                expected.get(red).copied().unwrap_or(0.0),
                expected.get(ox).copied().unwrap_or(0.0),
            );
            let (sr, so) = (
                set.get(red).copied().unwrap_or(0.0),
                set.get(ox).copied().unwrap_or(0.0),
            );
            if er + eo <= p.trace || sr + so <= p.trace || (red == "h2" && so <= p.trace) {
                continue;
            }
            let e_share = er / (er + eo);
            let s_share = sr / (sr + so);
            let lo = sse_core::math::pow(10.0, -p.differs_dex);
            let why = if s_share < e_share * lo {
                format!(
                    "more oxidised than the mantle's gas: {:.1} % of its {what} comes out as {}, here {:.2} %; something has oxidised it since (hydrogen escape, weathering, life)",
                    e_share * 100.0,
                    gas_name(red),
                    s_share * 100.0
                )
            } else if (1.0 - s_share) < (1.0 - e_share) * lo {
                format!(
                    "more reduced than the mantle's gas: {:.1} % of its {what} comes out as {}, here {:.2} %; something reduces it (a hydrogen store, life, a story)",
                    e_share * 100.0,
                    gas_name(red),
                    s_share * 100.0
                )
            } else {
                continue;
            };
            differs.push(serde_json::json!({
                "gas": red, "set": s_share, "expected": e_share, "of": [red, ox], "why": why
            }));
        }
        for (g, s) in set.iter().filter(|(g, _)| !OUTGASSED.contains(&g.as_str())) {
            let why = if g == "n2" {
                "nitrogen comes out of the rock too, but the outgassing block does not count it yet"
                    .to_string()
            } else {
                format!(
                    "{} does not come from the magma ocean: it needs another source (life, light, an older store, or a story)",
                    gas_name(g)
                )
            };
            differs.push(serde_json::json!({ "gas": g, "set": s, "expected": 0.0, "why": why }));
        }
    }
    // Hydrogen escapes at the diffusion limit (Hunten 1973): about
    // `escape_flux` H2 molecules per cm² per second per unit mixing ratio on
    // Earth, scaling with gravity (the flux is b f / H_air); the air's
    // column over that is how long it lasts.
    let mut escapes = vec![];
    let pa_for = set_pa.or(gas_pa);
    if let (Some(g), Some(pa)) = (gravity, pa_for) {
        let air = if set.is_empty() { &expected } else { &set };
        let mu = mean_molar_mass(air);
        let column = pa * sse_core::sheet::N_A / (mu * g) * 1.0e-4;
        let flux = p.escape_flux * g / sse_core::sheet::G0;
        let yr = column / flux / crate::clock::JULIAN_YEAR_S;
        escapes.push(serde_json::json!({
            "gas": "h2",
            "time_s": yr * crate::clock::JULIAN_YEAR_S,
            "why": format!("hydrogen leaves to space in {} at the diffusion limit, whatever its share (Hunten 1973)", years(yr)),
        }));
    }
    let h2_escapes = escapes
        .iter()
        .any(|e| e["time_s"].as_f64().is_some_and(|t| t < p.escape_keep_s));
    // What does not go together (E8).
    let clashes_of = |air: &BTreeMap<String, f64>,
                      blocks: Option<&mut Vec<serde_json::Value>>,
                      notes: Option<&mut Vec<String>>|
     -> Result<(Vec<serde_json::Value>, Option<f64>), Response> {
        let known: BTreeMap<String, f64> = air
            .iter()
            .filter(|(g, _)| *g != &dp.water && table.species(g).is_some())
            .map(|(g, x)| (g.clone(), *x))
            .collect();
        let left_out: Vec<&str> = air
            .keys()
            .filter(|g| *g != &dp.water && table.species(g).is_none())
            .map(|g| g.as_str())
            .collect();
        if let (Some(n), false) = (notes, left_out.is_empty()) {
            n.push(format!(
                "Left out of the chemistry (no formation energy on the stack): {}.",
                left_out.join(", ")
            ));
        }
        if known.len() < 2 {
            return Ok((vec![], None));
        }
        let p_bar = pa_for.unwrap_or(1.0e5) / 1.0e5;
        let d = sse_life::disequilibrium::disequilibrium(table, dp, &known, t_k, p_bar, seas)
            .map_err(bad)?;
        let mut out = vec![];
        for (rid, e) in &d.by_reaction {
            if *e < dp.signal_j_mol * 0.1 {
                continue;
            }
            let terms: Vec<(String, f64)> = table
                .reaction(rid)
                .map(|r| r.terms.clone())
                .or_else(|| dp.extra.iter().find(|x| &x.0 == rid).map(|x| x.1.clone()))
                .unwrap_or_default();
            let pair: Vec<String> = terms
                .iter()
                .filter(|(s, nu)| *nu < 0.0 && known.contains_key(s))
                .map(|(s, _)| s.clone())
                .collect();
            let names: Vec<&str> = pair.iter().map(|g| gas_name(g)).collect();
            let (time_s, why) = match p.lifetimes.get(rid) {
                Some((yr, src)) => (
                    Some(yr * crate::clock::JULIAN_YEAR_S),
                    format!(
                        "{} together last {} in sunlight like Earth's, only while something keeps making them ({src})",
                        names.join(" with "),
                        years(*yr)
                    ),
                ),
                None => (
                    None,
                    format!(
                        "{} would react ({e:.2} J per mol of air), but slowly without life or a catalyst",
                        names.join(" with ")
                    ),
                ),
            };
            out.push(serde_json::json!({
                "pair": pair, "sink": rid, "j_mol": e, "time_s": time_s, "why": why,
            }));
        }
        if let Some(b) = blocks {
            b.push(serde_json::json!({
                "block": "sse.life.disequilibrium",
                "inputs": { "temperature": t_k, "pressure_bar": p_bar, "liquid_water": seas },
                "results": { "available_j_mol": d.available_j_mol },
            }));
        }
        Ok((out, Some(d.available_j_mol)))
    };
    let (clashes, available) = clashes_of(&set, Some(&mut blocks), Some(&mut notes))?;
    // "Suggest an atmosphere": what the rock gives off, less the water the
    // seas take and the hydrogen that escapes; the hand-set gases the block
    // does not cover are kept unless they clash with it; shares sum to one.
    let mut edits = vec![];
    if !expected.is_empty() {
        let mut rock: BTreeMap<String, f64> = expected.clone();
        if seas {
            rock.remove("h2o");
        }
        if h2_escapes {
            rock.remove("h2");
        }
        let rock_sum: f64 = rock.values().sum();
        let mut kept: BTreeMap<String, f64> = set
            .iter()
            .filter(|(g, _)| !OUTGASSED.contains(&g.as_str()))
            .map(|(g, x)| (g.clone(), *x))
            .collect();
        let mut suggestion = BTreeMap::new();
        for _ in 0..4 {
            let kept_sum: f64 = kept.values().sum::<f64>().min(1.0);
            let room = if kept_sum > 0.0 { 1.0 - kept_sum } else { 1.0 };
            suggestion = kept.clone();
            if rock_sum > 0.0 {
                for (g, x) in &rock {
                    suggestion.insert(g.clone(), x / rock_sum * room);
                }
            }
            let (c, _) = clashes_of(&suggestion, None, None)?;
            let clashing: Vec<String> = c
                .iter()
                .flat_map(|x| x["pair"].as_array().cloned().unwrap_or_default())
                .filter_map(|g| g.as_str().map(String::from))
                .filter(|g| kept.contains_key(g))
                .collect();
            if clashing.is_empty() {
                break;
            }
            for g in clashing {
                kept.remove(&g);
            }
        }
        let total: f64 = suggestion.values().sum();
        for g in set.keys().filter(|g| !suggestion.contains_key(*g)) {
            edits.push(serde_json::json!({ "address": address, "field": format!("atmosphere.composition.{g}"), "value": 0.0 }));
        }
        for (g, x) in &suggestion {
            let v = if total > 0.0 { x / total } else { 0.0 };
            edits.push(serde_json::json!({ "address": address, "field": format!("atmosphere.composition.{g}"), "value": v }));
        }
        if seas && expected.contains_key("h2o") {
            notes.push("The suggestion leaves water out of the air: the seas hold it.".into());
        }
        if h2_escapes && expected.contains_key("h2") {
            notes.push(
                "The suggestion leaves hydrogen out: it escapes to space quickly here.".into(),
            );
        }
        // The pressure only when the formation ledger gives it and the seas
        // do not take the water (then its weight is not in the air).
        if let Some(pa) = gas_pa {
            let pa = if seas { co2_pa.unwrap_or(pa) } else { pa };
            edits.push(serde_json::json!({ "address": address, "field": "atmosphere.surface_pressure", "value": pa }));
        }
    }
    let says = match (expected.is_empty(), clashes.is_empty(), differs.is_empty()) {
        (true, _, _) => "Nothing is expected yet: this world's mantle is not known.".to_string(),
        (false, true, true) => "The air goes together and is close to what the rock gives off.".to_string(),
        (false, true, false) => "The air goes together, but it is not what the rock gives off: something else made or changed it.".to_string(),
        (false, false, _) => "Some of these gases do not go together: they last only while something keeps making them.".to_string(),
    };
    Ok(serde_json::json!({
        "body": address,
        "says": says,
        "redox_iw": redox,
        "redox_from": redox_from,
        "expected": expected,
        "pressure_pa": gas_pa,
        "set": set,
        "set_pressure_pa": set_pa,
        "differs": differs,
        "clashes": clashes,
        "escapes": escapes,
        "available_j_mol": available,
        "suggest": if edits.is_empty() { serde_json::Value::Null } else { serde_json::json!({ "op": "set_authored_batch", "edits": edits }) },
        "notes": notes,
        "blocks": blocks,
    }))
}

impl Kernel {
    /// `atmosphere_advice`: the body's facts at `at_s`, with `facts` laid over.
    pub(crate) fn atmosphere_advice(
        &mut self,
        body: &str,
        at_s: Option<i64>,
        facts: Option<&serde_json::Value>,
    ) -> Result<Response, Response> {
        let a: sse_core::Address = body
            .parse()
            .map_err(|e| err("bad_address", format!("{e}")))?;
        let u = self.universe()?;
        if !u.root.contains(&a) {
            return Err(err(
                "bad_address",
                format!("{a} is not in universe {}", u.root),
            ));
        }
        let st = u.chronicle.state_at(sse_chronicle::UniverseTime::seconds(
            at_s.unwrap_or(i64::MAX),
        ));
        let mut row = st.facts.get(&a).cloned().unwrap_or_default();
        if let Some(extra) = facts {
            let obj = extra
                .as_object()
                .ok_or_else(|| err("bad_value", "facts: an object of field to value"))?;
            for (k, v) in obj {
                row.insert(
                    k.clone(),
                    json_to_value(v).map_err(|e| err("bad_value", format!("{k}: {e}")))?,
                );
            }
        }
        let table =
            match &self.observe.chemistry {
                Some(t) => t.clone(),
                None => sse_life::chemistry::ThermoTable::from_packs(&[
                    sse_life::scenario::base_life_pack(),
                ])
                .map_err(|e| err("bad_pack", e))?,
            };
        let advice = advise(
            body,
            &row,
            &table,
            &sse_life::disequilibrium::DiseqParams::default(),
            &AdviceParams::default(),
        )?;
        Ok(Response::AtmosphereAdvised { advice })
    }
}

//! Reverse placement (classification design section 7, stage C3): "put
//! one of those here".
//!
//! Almost every axis a class cares about is a result, so placing a class is
//! a search over causes:
//! 1. **Quick bounds**: the orbits whose flux can give the class's
//!    temperatures, intersected with the requested scope, the orbits stable
//!    among the bodies already there, and the orbits where a body of the
//!    class's mass keeps its air. Empty bounds give "not here" with the reason.
//! 2. **Search**: a fixed number of candidates drawn with keyed draws from the
//!    class's typical values inside its bounds, run forward through the
//!    flows ([`model`]), scored with the same fit as recognition, then refined
//!    a fixed number of steps. Fixed counts and keyed draws give the same
//!    answer on every device.
//! 3. **Cross-references**: the `needs` of every class the result would carry,
//!    in every scheme, are settled by the suite that owns them: `life:` needs
//!    by the life suite over the body's habitat graph, `authored:` needs only
//!    by an authored intervention.
//! 4. **One of four outcomes**: placed, closest here (with what stands in
//!    the way), needs a helping hand (with the interventions), or forced (the
//!    missing values written as authored-impossible, marked).
//!
//! [`preview`] runs the same for every base class of a scheme: the picker's
//! available, available with help, or not possible here.
#![forbid(unsafe_code)]

pub mod model;
pub mod scenario;

pub use model::{Causes, Model, Neighbour, Settings, System};

use serde::{Deserialize, Serialize};
use sse_chronicle::{Chronicle, Event, EventBody, Evidence, Payload, Publisher, UniverseTime};
use sse_core::cbor::Value;
use sse_core::draw::{Draws, Stream};
use sse_core::math as m;
use sse_core::quantity::consts::AU;
use sse_core::{Address, Seed};
use sse_life::{HabitatGraph, LifeQuery, LifeSuite};
use sse_sift::classify::{Atlas, Bound, Scale};
use sse_sift::predicate::Subject;
use std::collections::{BTreeMap, BTreeSet};

#[cfg(test)]
mod tests;

/// A placement request.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Request {
    pub scheme: String,
    pub class: String,
    /// Orbit range to place in, au; the whole system when absent.
    #[serde(default)]
    pub scope_au: Option<(f64, f64)>,
    /// The GM insists: missing values are written as authored-impossible.
    #[serde(default)]
    pub force: bool,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Outcome {
    Placed,
    ClosestHere,
    NeedsHelp,
    Forced,
}

/// How each written fact was made (H8.6 honesty marks).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Mark {
    Authored,
    Derived,
    AuthoredImpossible,
}

/// The working behind the quick bounds.
#[derive(Clone, PartialEq, Debug, Default, Serialize, Deserialize)]
pub struct Bounds {
    /// Orbit intervals left after every bound, au.
    pub a_au: Vec<(f64, f64)>,
    /// One plain line per bound applied.
    pub notes: Vec<String>,
    /// Why the bounds are empty, when they are.
    pub empty_because: Option<String>,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Placement {
    pub scheme: String,
    pub class: String,
    pub outcome: Outcome,
    pub bounds: Bounds,
    pub causes: Option<Causes>,
    pub facts: BTreeMap<String, Value>,
    pub marks: BTreeMap<String, Mark>,
    /// Fit of the requested class, and whether recognition gives it back.
    pub fit: f64,
    pub recognised: bool,
    /// What the result is called in the requested scheme.
    pub label: Option<String>,
    pub modifiers: Vec<String>,
    /// What stands in the way (closest here) or what was overridden (forced).
    pub blocking: Vec<String>,
    /// Authored interventions it needs (needs a helping hand, forced).
    pub interventions: Vec<String>,
    /// Candidates run through the flows.
    pub tried: u32,
}

/// Picker entry.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Availability {
    Available,
    WithHelp,
    NotHere,
}

/// Bounds of a class and its ancestors (region and gate) by fact field, in
/// the fact's own units.
#[derive(Clone, PartialEq, Debug)]
enum FB {
    Num(f64, f64),
    Cat(BTreeSet<String>),
    Flag(bool),
}

fn field_bounds(atlas: &Atlas, class: &str) -> BTreeMap<String, FB> {
    let mut out: BTreeMap<String, FB> = BTreeMap::new();
    let mut chain = Vec::new();
    let mut c = atlas.classes.iter().find(|k| k.id == class);
    while let Some(k) = c {
        chain.push(k);
        c = k
            .parent
            .as_ref()
            .and_then(|p| atlas.classes.iter().find(|x| &x.id == p));
    }
    for b in chain.iter().flat_map(|k| k.region.iter().chain(&k.gate)) {
        let (axis, fb) = match b {
            Bound::Numeric { axis, core, .. } => (axis, FB::Num(core.0, core.1)),
            Bound::AnyOf { axis, any_of } => (axis, FB::Cat(any_of.iter().cloned().collect())),
            Bound::Flag { axis, is } => (axis, FB::Flag(*is)),
        };
        let Some(ax) = atlas.axes.get(axis) else {
            continue;
        };
        let k = ax.divide_by.unwrap_or(1.0);
        let fb = match fb {
            FB::Num(lo, hi) => FB::Num(lo * k, hi * k),
            x => x,
        };
        let merged = match (out.remove(&ax.field), fb) {
            (None, x) => x,
            (Some(FB::Num(a, b)), FB::Num(c, d)) => FB::Num(a.max(c), b.min(d)),
            (Some(FB::Cat(x)), FB::Cat(y)) => FB::Cat(x.intersection(&y).cloned().collect()),
            (Some(_), x) => x,
        };
        out.insert(ax.field.clone(), merged);
    }
    out
}

fn num(b: &BTreeMap<String, FB>, f: &str) -> Option<(f64, f64)> {
    match b.get(f) {
        Some(FB::Num(a, c)) => Some((*a, *c)),
        _ => None,
    }
}

fn intersect(iv: &[(f64, f64)], lo: f64, hi: f64) -> Vec<(f64, f64)> {
    iv.iter()
        .filter_map(|&(a, b)| {
            let (x, y) = (a.max(lo), b.min(hi));
            (y > x).then_some((x, y))
        })
        .collect()
}
fn remove(iv: &[(f64, f64)], lo: f64, hi: f64) -> Vec<(f64, f64)> {
    let mut out = Vec::new();
    for &(a, b) in iv {
        if hi <= a || lo >= b {
            out.push((a, b));
            continue;
        }
        if lo > a {
            out.push((a, lo));
        }
        if hi < b {
            out.push((hi, b));
        }
    }
    out
}
fn fmt_au(x: f64) -> String {
    if x >= 10.0 {
        format!("{x:.0}")
    } else if x >= 0.1 {
        format!("{x:.2}")
    } else {
        format!("{x:.3}")
    }
}
fn show(iv: &[(f64, f64)]) -> String {
    iv.iter()
        .map(|(a, b)| format!("{} to {} au", fmt_au(*a), fmt_au(*b)))
        .collect::<Vec<_>>()
        .join(", ")
}

/// Everything placement reads: the atlas, the flows and the life suite.
pub struct Placer<'a> {
    pub atlas: &'a Atlas,
    pub model: &'a Model,
    pub life: &'a dyn LifeSuite,
    pub life_rules: &'a [sse_life::HabitatRule],
}

const T_GREY: f64 = 0.75;

/// A scored candidate: (recognised, fit, index, causes, facts).
type Cand = (bool, f64, u32, Causes, BTreeMap<String, Value>);

impl Placer<'_> {
    fn quick_bounds(
        &self,
        sys: &System,
        lum: f64,
        req: &Request,
        b: &BTreeMap<String, FB>,
    ) -> Bounds {
        let s = &self.model.settings;
        let mut out = Bounds {
            a_au: vec![s.slot_au],
            ..Default::default()
        };
        let lum_sun = lum / sse_core::quantity::consts::L_SUN;
        // Temperatures to an orbit band, through teq = (L(1-A)/(16 pi sigma a^2))^(1/4)
        // and the grey greenhouse tsurf = teq (1 + 0.75 tau)^(1/4).
        let p = num(b, "atmosphere.surface_pressure").map(|(a, c)| (a / 1e5, c / 1e5));
        let pp = s.prior("pressure_bar");
        let (plo, phi) = p.map_or(pp, |(a, c)| (a.max(0.0), c.min(pp.1).max(a)));
        let (tau_lo, tau_hi) = (self.model.optical_depth(plo), self.model.optical_depth(phi));
        let mut teq = (0.0_f64, f64::INFINITY);
        if let Some((lo, hi)) = num(b, "planet.equilibrium_temperature") {
            teq = (lo, hi);
        }
        if let Some((lo, hi)) = num(b, "planet.surface_temperature") {
            teq = (
                teq.0.max(lo / m::pow(1.0 + T_GREY * tau_hi, 0.25)),
                teq.1.min(hi / m::pow(1.0 + T_GREY * tau_lo, 0.25)),
            );
        }
        if teq.0 > 0.0 || teq.1.is_finite() {
            let (alo, ahi) = s.prior("albedo");
            let sigma = sse_core::quantity::consts::STEFAN_BOLTZMANN;
            let a_of = |t: f64, alb: f64| {
                m::sqrt(lum * (1.0 - alb) / (16.0 * m::PI * sigma * t * t * t * t)) / AU
            };
            let inner = if teq.1.is_finite() {
                a_of(teq.1, ahi)
            } else {
                0.0
            };
            let outer = if teq.0 > 0.0 {
                a_of(teq.0, alo)
            } else {
                f64::INFINITY
            };
            out.a_au = intersect(&out.a_au, inner, outer);
            out.notes.push(format!(
                "temperatures of {:.0} to {} K need an orbit from {} to {} au around this star ({:.3} solar luminosities)",
                teq.0,
                if teq.1.is_finite() { format!("{:.0}", teq.1) } else { "any".into() },
                fmt_au(inner),
                if outer.is_finite() { fmt_au(outer) } else { "any distance".into() },
                lum_sun
            ));
            if out.a_au.is_empty() {
                out.empty_because = Some(format!(
                    "no orbit in {} to {} au gives the class's temperatures around this star",
                    fmt_au(s.slot_au.0),
                    fmt_au(s.slot_au.1)
                ));
                return out;
            }
        }
        if let Some((lo, hi)) = req.scope_au {
            let before = out.a_au.clone();
            out.a_au = intersect(&out.a_au, lo, hi);
            out.notes
                .push(format!("scope {} to {} au", fmt_au(lo), fmt_au(hi)));
            if out.a_au.is_empty() {
                out.empty_because = Some(format!(
                    "the class needs an orbit at {}, outside the requested {} to {} au",
                    show(&before),
                    fmt_au(lo),
                    fmt_au(hi)
                ));
                return out;
            }
        }
        // Stable slots among the bodies already there.
        let mass_hi = num(b, "body.mass").map_or(s.prior("mass_me").1, |(_, h)| h / model::M_EARTH);
        for nb in &sys.bodies {
            let h = m::cbrt(
                (mass_hi.min(1e5) + nb.mass_me) * model::M_EARTH
                    / (3.0 * sys.star_mass_msun * model::M_SUN),
            );
            let (lo, hi) = (
                (nb.a_au * (1.0 - s.mutual_hill * h)).max(0.0),
                nb.a_au * (1.0 + s.mutual_hill * h),
            );
            let before = out.a_au.clone();
            out.a_au = remove(&out.a_au, lo, hi);
            if out.a_au != before {
                out.notes.push(format!(
                    "{} at {} au clears {} to {} au ({} mutual Hill radii)",
                    nb.name,
                    fmt_au(nb.a_au),
                    fmt_au(lo),
                    fmt_au(hi),
                    s.mutual_hill
                ));
            }
            if out.a_au.is_empty() {
                out.empty_because = Some(format!(
                    "every orbit that could work is too close to {} to be stable",
                    nb.name
                ));
                return out;
            }
        }
        // Air: the class needs some, so the body must hold it against its star.
        if let Some((plo_pa, _)) = num(b, "atmosphere.surface_pressure") {
            if plo_pa > 0.0 {
                let me = mass_hi.min(
                    s.prior("mass_me")
                        .1
                        .max(num(b, "body.mass").map_or(0.0, |x| x.1 / model::M_EARTH)),
                );
                let r = (1.07 - 0.21 * 0.33)
                    * m::pow(me, 1.0 / 3.7)
                    * sse_core::quantity::consts::R_EARTH;
                let v = m::sqrt(2.0 * sse_core::quantity::consts::G * me * model::M_EARTH / r);
                let i_max = m::pow(v / s.shoreline_v0_m_s, 4.0);
                let a_min = m::sqrt(lum_sun / i_max);
                let before = out.a_au.clone();
                out.a_au = intersect(&out.a_au, a_min, f64::INFINITY);
                out.notes.push(format!(
                    "a body of up to {me:.2} Earth masses keeps air only beyond {} au here",
                    fmt_au(a_min)
                ));
                if out.a_au.is_empty() {
                    out.empty_because = Some(format!(
                        "a body of the class's mass loses its air at {}: the star strips it",
                        show(&before)
                    ));
                }
            }
        }
        out
    }

    fn draw_causes(
        &self,
        d: &Draws,
        base: u64,
        b: &BTreeMap<String, FB>,
        a_iv: &[(f64, f64)],
        typical: &BTreeMap<String, (f64, f64)>,
    ) -> Causes {
        let s = &self.model.settings;
        let mut k = 0u64;
        let mut u = || {
            k += 1;
            d.unit(base + k)
        };
        let lin = |x: f64, (lo, hi): (f64, f64)| lo + (hi - lo) * x;
        let log = |x: f64, (lo, hi): (f64, f64)| {
            let (lo, hi) = (lo.max(1e-30), hi.max(lo.max(1e-30)));
            m::exp(m::ln(lo) + (m::ln(hi) - m::ln(lo)) * x)
        };
        // Class bound narrowed by its typical value, else the prior.
        let range = |field: &str, prior: (f64, f64), scale: f64| -> (f64, f64) {
            let mut r = num(b, field)
                .map(|(a, c)| (a / scale, c / scale))
                .unwrap_or(prior);
            if r.1.is_infinite() || r.1 > 1e29 {
                r.1 = prior.1.max(r.0);
            }
            let both = (r.0.max(prior.0), r.1.min(prior.1));
            if both.1 >= both.0 && num(b, field).is_none() {
                r = both;
            }
            if let Some(&(c, sp)) = typical.get(field) {
                let t = ((c - sp) / scale, (c + sp) / scale);
                let n = (r.0.max(t.0), r.1.min(t.1));
                if n.1 >= n.0 {
                    r = n;
                }
            }
            r
        };
        // Orbit: log-uniform over the allowed intervals.
        let total: f64 = a_iv.iter().map(|(a, c)| m::ln(*c) - m::ln(*a)).sum();
        let mut x = u() * total;
        let mut a_au = a_iv.first().map_or(1.0, |v| v.0);
        for &(lo, hi) in a_iv {
            let w = m::ln(hi) - m::ln(lo);
            if x <= w {
                a_au = m::exp(m::ln(lo) + x);
                break;
            }
            x -= w;
            a_au = hi;
        }
        let mass_me = log(u(), range("body.mass", s.prior("mass_me"), model::M_EARTH));
        let metal = lin(u(), range("makeup.metal", s.prior("metal"), 1.0));
        let ice = lin(u(), range("makeup.ice", s.prior("ice"), 1.0));
        let gas = lin(u(), range("makeup.gas", s.prior("gas"), 1.0));
        let carbon = lin(u(), range("makeup.carbon", s.prior("carbon"), 1.0));
        let mut fr = [metal, ice, gas, carbon];
        let sum: f64 = fr.iter().sum();
        if sum > 1.0 {
            fr.iter_mut().for_each(|f| *f /= sum);
        }
        let rock = (1.0 - fr.iter().sum::<f64>()).max(0.0);
        let pressure_bar = log(
            u(),
            range("atmosphere.surface_pressure", s.prior("pressure_bar"), 1e5),
        );
        let air_main = match b.get("atmosphere.main") {
            Some(FB::Cat(c)) if !c.is_empty() => {
                let v: Vec<&String> = c.iter().filter(|x| *x != "none").collect();
                if v.is_empty() {
                    "none".to_string()
                } else {
                    v[(u() * v.len() as f64).min(v.len() as f64 - 1.0) as usize].clone()
                }
            }
            _ => if u() < 0.5 { "n2" } else { "co2" }.to_string(),
        };
        let pp_o2_bar = match num(b, "atmosphere.pp.o2") {
            Some(r) => lin(u(), (r.0, r.1.min(pressure_bar))),
            None => 0.0,
        }
        .min(pressure_bar);
        let pp_co2_bar = match num(b, "atmosphere.pp.co2") {
            Some(r) => lin(u(), (r.0, r.1.min(pressure_bar - pp_o2_bar).max(r.0))),
            None if air_main == "co2" => 0.95 * pressure_bar,
            None => log(u(), (4e-4 * pressure_bar, 4e-3 * pressure_bar)),
        }
        .min(pressure_bar - pp_o2_bar)
        .max(0.0);
        let liquid_kind = match b.get("surface.liquid") {
            Some(FB::Cat(c)) if c.contains("methane") || c.contains("ethane") => "methane",
            _ => "water",
        }
        .to_string();
        // The inventory is the cover it gives: as ice when the class wants ice.
        let cover = match num(b, "surface.ice_fraction") {
            Some(r) if r.0 > 0.0 => Some(r),
            _ => num(b, "surface.liquid_fraction"),
        };
        let liquid_inventory = lin(u(), cover.map_or(s.prior("water"), |r| (r.0, r.1.min(1.0))));
        let subsurface_ocean = matches!(b.get("interior.subsurface_ocean"), Some(FB::Flag(true)));
        let ecc = lin(u(), range("orbit.eccentricity", s.prior("ecc"), 1.0));
        let albedo = lin(u(), s.prior("albedo"));
        Causes {
            a_au,
            ecc,
            mass_me,
            metal: fr[0],
            rock,
            carbon: fr[3],
            ice: fr[1],
            gas: fr[2],
            albedo,
            pressure_bar,
            air_main,
            pp_o2_bar,
            pp_co2_bar,
            liquid_inventory,
            liquid_kind,
            subsurface_ocean,
        }
    }

    /// Nudge every numeric cause by up to `scale` of its range, keyed.
    fn perturb(&self, c: &Causes, d: &Draws, base: u64, scale: f64, a_iv: &[(f64, f64)]) -> Causes {
        let mut k = 0u64;
        let mut u = || {
            k += 1;
            2.0 * d.unit(base + k) - 1.0
        };
        let mut n = c.clone();
        let lo_a = a_iv.first().map_or(c.a_au, |v| v.0);
        let hi_a = a_iv.last().map_or(c.a_au, |v| v.1);
        n.a_au = (c.a_au * m::exp(scale * u())).clamp(lo_a, hi_a);
        n.mass_me = c.mass_me * m::exp(scale * u());
        n.albedo = (c.albedo + 0.5 * scale * u()).clamp(0.0, 0.95);
        n.pressure_bar = c.pressure_bar * m::exp(scale * u());
        let f = (n.pressure_bar / c.pressure_bar.max(1e-30)).max(0.0);
        n.pp_o2_bar *= f;
        n.pp_co2_bar *= f;
        n.liquid_inventory = (c.liquid_inventory + 0.5 * scale * u()).clamp(0.0, 1.0);
        n
    }

    fn score(&self, req: &Request, facts: &BTreeMap<String, Value>) -> (bool, f64) {
        let s = Subject::new(facts.clone());
        let fit = self.atlas.fit(&req.class, &s).map_or(0.0, |f| f.fit);
        let l = self.atlas.label(&req.scheme, &s, None);
        let rec = l.modifiers.contains(&req.class)
            || l.base.as_deref().is_some_and(|b| {
                let mut c = Some(b.to_string());
                let mut ok = false;
                while let Some(x) = c {
                    if x == req.class {
                        ok = true;
                        break;
                    }
                    c = self
                        .atlas
                        .classes
                        .iter()
                        .find(|k| k.id == x)
                        .and_then(|k| k.parent.clone());
                }
                ok
            });
        (rec, fit)
    }

    /// Every `needs` of every class the facts carry, in every scheme.
    fn needs(&self, facts: &BTreeMap<String, Value>) -> Vec<(String, String)> {
        let s = Subject::new(facts.clone());
        let mut out = Vec::new();
        for sc in self.atlas.schemes.keys() {
            let l = self.atlas.label(sc, &s, None);
            let mut ids: Vec<String> = l.modifiers.clone();
            let mut c = l.base.clone();
            while let Some(x) = c {
                c = self
                    .atlas
                    .classes
                    .iter()
                    .find(|k| k.id == x)
                    .and_then(|k| k.parent.clone());
                ids.push(x);
            }
            for id in ids {
                if let Some(k) = self.atlas.classes.iter().find(|k| k.id == id) {
                    for n in &k.needs {
                        if !out.iter().any(|(_, m)| m == n) {
                            out.push((id.clone(), n.clone()));
                        }
                    }
                }
            }
        }
        out
    }

    /// A small world stack for the result, for the life suite's habitat graph.
    fn stack_of(
        &self,
        address: &Address,
        facts: &BTreeMap<String, Value>,
    ) -> sse_world::WorldStack {
        let f = |k: &str| match facts.get(k) {
            Some(Value::Float(x)) => *x,
            Some(Value::Int(i)) => *i as f64,
            _ => 0.0,
        };
        let r = f("body.radius");
        let (t, p) = (
            f("planet.surface_temperature"),
            f("atmosphere.surface_pressure"),
        );
        let liquid = f("surface.liquid_fraction");
        let props = |extra: &[(&str, f64)]| -> BTreeMap<String, f64> {
            let mut b: BTreeMap<String, f64> = [
                ("temperature_k".to_string(), t),
                ("pressure_pa".to_string(), p),
            ]
            .into_iter()
            .collect();
            b.extend(extra.iter().map(|(k, v)| (k.to_string(), *v)));
            b
        };
        let mut shells = vec![
            sse_world::Shell {
                key: "crust".into(),
                kind: "crust".into(),
                name: "Crust".into(),
                top_m: 0.0,
                bottom_m: -30000.0,
                within: None,
                properties: props(&[("pore_water_fraction", 0.01)]),
                provides: vec![],
            },
            sse_world::Shell {
                key: "mantle".into(),
                kind: "mantle".into(),
                name: "Mantle".into(),
                top_m: -30000.0,
                bottom_m: -r * 0.5,
                within: None,
                properties: BTreeMap::new(),
                provides: vec!["heat".into()],
            },
        ];
        if p > 0.0 {
            shells.insert(
                0,
                sse_world::Shell {
                    key: "atmosphere".into(),
                    kind: "atmosphere".into(),
                    name: "Atmosphere".into(),
                    top_m: 100000.0,
                    bottom_m: 0.0,
                    within: None,
                    properties: BTreeMap::new(),
                    provides: vec![],
                },
            );
        }
        let mut regions = vec![sse_world::Region {
            key: "land".into(),
            kind: "land".into(),
            name: "Land".into(),
            on: "crust".into(),
            extent: sse_world::Extent::Unmapped,
            top_m: None,
            bottom_m: None,
            properties: props(&[("area_fraction", 1.0 - liquid)]),
            provides: vec![],
        }];
        if liquid > 0.0 {
            regions.push(sse_world::Region {
                key: "basins".into(),
                kind: "ocean_basin".into(),
                name: "Basins".into(),
                on: "crust".into(),
                extent: sse_world::Extent::Unmapped,
                top_m: None,
                bottom_m: None,
                properties: [("area_fraction".to_string(), liquid)]
                    .into_iter()
                    .collect(),
                provides: vec![],
            });
            shells.push(sse_world::Shell {
                key: "ocean".into(),
                kind: "ocean".into(),
                name: "Ocean".into(),
                top_m: 0.0,
                bottom_m: -3000.0,
                within: Some("basins".into()),
                properties: props(&[]),
                provides: vec![],
            });
        }
        sse_world::WorldStack {
            format: sse_world::FORMAT.into(),
            body: address.to_string(),
            name: "placement".into(),
            radius_m: r.max(1.0),
            shells,
            regions,
            features: vec![],
        }
    }

    /// Place one body of the requested class into the system.
    pub fn place(
        &self,
        seed: &Seed,
        address: &Address,
        sys: &System,
        req: &Request,
    ) -> Result<Placement, String> {
        if self
            .atlas
            .classes
            .iter()
            .all(|c| c.id != req.class || c.scheme != req.scheme)
        {
            return Err(format!("no class {} in scheme {}", req.class, req.scheme));
        }
        let s = &self.model.settings;
        let lum = self.model.luminosity_w(sys)?;
        let b = field_bounds(self.atlas, &req.class);
        let bounds = self.quick_bounds(sys, lum, req, &b);
        // With empty bounds the search still runs over the scope (or the
        // whole system), to say what is closest and to support forcing.
        let a_iv = if bounds.a_au.is_empty() {
            vec![req.scope_au.unwrap_or(s.slot_au)]
        } else {
            bounds.a_au.clone()
        };
        let mut typical = BTreeMap::new();
        let mut c = self.atlas.classes.iter().find(|k| k.id == req.class);
        while let Some(k) = c {
            for t in &k.typical {
                if let Some(ax) = self.atlas.axes.get(&t.axis) {
                    let d = ax.divide_by.unwrap_or(1.0);
                    typical
                        .entry(ax.field.clone())
                        .or_insert((t.centre * d, t.spread * d));
                }
            }
            c = k
                .parent
                .as_ref()
                .and_then(|p| self.atlas.classes.iter().find(|x| &x.id == p));
        }
        let stream = Stream::new("place.search.v1").map_err(|e| e.to_string())?;
        let d = Draws::new(*seed, address.clone(), stream);
        let mut tried = 0u32;
        let mut pool: Vec<Cand> = Vec::new();
        for i in 0..s.candidates {
            let cz = self.draw_causes(&d, u64::from(i) * 64, &b, &a_iv, &typical);
            tried += 1;
            if let Ok(f) = self.model.forward(sys, lum, &cz) {
                let (rec, fit) = self.score(req, &f);
                pool.push((rec, fit, i, cz, f));
            }
        }
        let rank = |x: &Cand, y: &Cand| y.0.cmp(&x.0).then(y.1.total_cmp(&x.1)).then(x.2.cmp(&y.2));
        pool.sort_by(rank);
        pool.truncate(s.keep as usize);
        for step in 0..s.refine_steps {
            let scale = 0.3 * m::pow(0.85, f64::from(step));
            #[allow(clippy::needless_range_loop)]
            for j in 0..pool.len() {
                let base =
                    (u64::from(s.candidates) + u64::from(step) * u64::from(s.keep) + j as u64) * 64;
                let cz = self.perturb(&pool[j].3, &d, base, scale, &a_iv);
                tried += 1;
                if let Ok(f) = self.model.forward(sys, lum, &cz) {
                    let (rec, fit) = self.score(req, &f);
                    let cand = (rec, fit, pool[j].2, cz, f);
                    if rank(&cand, &pool[j]) == std::cmp::Ordering::Less {
                        pool[j] = cand;
                    }
                }
            }
        }
        pool.sort_by(rank);
        let Some((rec, fit, _, causes, mut facts)) = pool.into_iter().next() else {
            return Err("no candidate could be run through the flows".into());
        };
        let mut marks: BTreeMap<String, Mark> =
            facts.keys().map(|k| (k.clone(), Mark::Derived)).collect();
        for k in [
            "body.mass",
            "orbit.semi_major_axis",
            "orbit.eccentricity",
            "planet.bond_albedo",
            "makeup.metal",
            "makeup.rock",
            "makeup.carbon",
            "makeup.ice",
            "makeup.gas",
            "system.age",
            "orbit.host_kind",
        ] {
            if facts.contains_key(k) {
                marks.insert(k.into(), Mark::Authored);
            }
        }
        let mut blocking = Vec::new();
        if let Some(why) = &bounds.empty_because {
            blocking.push(why.clone());
        }
        let subject = Subject::new(facts.clone());
        if let Some(cf) = self.atlas.fit(&req.class, &subject) {
            for a in cf.axes.iter().filter(|a| a.fit < s.full_fit) {
                if let Some(ax) = self.atlas.axes.get(&a.axis) {
                    blocking.push(format!(
                        "{} is {} here; the class needs it in its core{}",
                        ax.label,
                        match &a.value {
                            Some(Value::Float(x)) => format!("{x:.3}"),
                            Some(Value::Text(t)) => t.clone(),
                            Some(Value::Bool(b)) => b.to_string(),
                            Some(v) => format!("{v:?}"),
                            None => "unknown".into(),
                        },
                        a.change_to_core
                            .map_or(String::new(), |c| format!(" (change by {c:.3})"))
                    ));
                }
            }
        }
        // Outside the quick bounds nothing counts as placed, however well it scores.
        let full = rec && fit >= s.full_fit && bounds.empty_because.is_none();
        let mut interventions = Vec::new();
        let mut outcome = if full {
            Outcome::Placed
        } else {
            Outcome::ClosestHere
        };
        if req.force && outcome == Outcome::ClosestHere {
            // Write the class's core values over every result that misses it.
            outcome = Outcome::Forced;
            let mut chain = Vec::new();
            let mut c = self.atlas.classes.iter().find(|k| k.id == req.class);
            while let Some(k) = c {
                chain.push(k);
                c = k
                    .parent
                    .as_ref()
                    .and_then(|p| self.atlas.classes.iter().find(|x| &x.id == p));
            }
            for k in chain {
                for bd in k.region.iter().chain(&k.gate) {
                    let subject = Subject::new(facts.clone());
                    let Some(cf) = self.atlas.fit(&req.class, &subject) else {
                        continue;
                    };
                    let (axis, v) = match bd {
                        Bound::Numeric { axis, core, .. } => {
                            let Some(ax) = self.atlas.axes.get(axis) else {
                                continue;
                            };
                            let mid = if ax.scale == Scale::Log && core.0 > 0.0 {
                                m::sqrt(core.0 * core.1.min(1e30))
                            } else {
                                0.5 * (core.0 + core.1.min(core.0.abs().max(1.0) * 1e6))
                            };
                            (axis, Value::Float(mid * ax.divide_by.unwrap_or(1.0)))
                        }
                        Bound::AnyOf { axis, any_of } => (axis, Value::Text(any_of[0].clone())),
                        Bound::Flag { axis, is } => (axis, Value::Bool(*is)),
                    };
                    if cf
                        .axes
                        .iter()
                        .any(|a| &a.axis == axis && a.fit < s.full_fit)
                    {
                        let field = self.atlas.axes[axis].field.clone();
                        facts.insert(field.clone(), v);
                        marks.insert(field, Mark::AuthoredImpossible);
                    }
                }
            }
        }
        // Cross-references, settled by the suite that owns each.
        if full || outcome == Outcome::Forced {
            for (class, need) in self.needs(&facts) {
                if let Some(cap) = need.strip_prefix("life:") {
                    let graph =
                        HabitatGraph::derive(&self.stack_of(address, &facts), self.life_rules)
                            .map_err(|e| e.join("; "))?;
                    let q = LifeQuery {
                        address: address.clone(),
                        epoch: UniverseTime::years((sys.age_gyr * 1e9) as i64),
                        capabilities: vec![cap.to_string()],
                    };
                    let ans = self.life.query_feasible(&q, &graph.profiles());
                    match ans.answer {
                        Some(true) => {}
                        Some(false) => {
                            blocking.push(format!("{class} needs {cap}: {}", ans.reason));
                            if outcome != Outcome::Forced {
                                outcome = Outcome::ClosestHere;
                            }
                        }
                        None => interventions.push(format!(
                            "seed {cap} life ({class} needs it; the life suite says: {})",
                            ans.reason
                        )),
                    }
                } else if let Some(what) = need.strip_prefix("authored:") {
                    interventions.push(format!("authored {what} ({class} needs it)"));
                }
            }
            if outcome == Outcome::Placed && !interventions.is_empty() {
                outcome = Outcome::NeedsHelp;
            }
        }
        let fs = Subject::new(facts.clone());
        let l = self.atlas.label(&req.scheme, &fs, None);
        let (rec2, fit2) = self.score(req, &facts);
        Ok(Placement {
            scheme: req.scheme.clone(),
            class: req.class.clone(),
            outcome,
            bounds,
            causes: Some(causes),
            facts,
            marks,
            fit: if outcome == Outcome::Forced {
                fit2
            } else {
                fit
            },
            recognised: if outcome == Outcome::Forced {
                rec2
            } else {
                rec
            },
            label: l.base,
            modifiers: l.modifiers,
            blocking,
            interventions,
            tried,
        })
    }

    /// The picker: every base class of a scheme, available, with help, or
    /// not possible here, with the reason.
    pub fn preview(
        &self,
        seed: &Seed,
        address: &Address,
        sys: &System,
        scheme: &str,
        scope_au: Option<(f64, f64)>,
    ) -> Vec<(String, Availability, String)> {
        let mut out = Vec::new();
        for c in self
            .atlas
            .classes
            .iter()
            .filter(|c| c.scheme == scheme && c.kind == sse_sift::classify::ClassKind::Base)
        {
            let req = Request {
                scheme: scheme.into(),
                class: c.id.clone(),
                scope_au,
                force: false,
            };
            match self.place(seed, address, sys, &req) {
                Ok(p) => {
                    let (a, why) = match p.outcome {
                        Outcome::Placed => (Availability::Available, String::new()),
                        Outcome::NeedsHelp => (Availability::WithHelp, p.interventions.join("; ")),
                        _ => (
                            Availability::NotHere,
                            p.blocking.first().cloned().unwrap_or_default(),
                        ),
                    };
                    out.push((c.id.clone(), a, why));
                }
                Err(e) => out.push((c.id.clone(), Availability::NotHere, e)),
            }
        }
        out
    }
}

/// Record a placement in the chronicle: the causes as authored, the results
/// as derived from them, forced values as authored-impossible, and each
/// intervention as an authored event. Returns the event ids in order.
pub fn commit(
    ch: &mut Chronicle,
    address: &Address,
    at: UniverseTime,
    p: &Placement,
) -> Result<Vec<sse_core::ContentHash>, String> {
    let mut ids = Vec::new();
    let by = |mk: Mark| -> BTreeMap<Address, BTreeMap<String, Value>> {
        let f: BTreeMap<String, Value> = p
            .facts
            .iter()
            .filter(|(k, _)| p.marks.get(*k) == Some(&mk))
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        if f.is_empty() {
            BTreeMap::new()
        } else {
            [(address.clone(), f)].into_iter().collect()
        }
    };
    let data = serde_json::to_value(serde_json::json!({
        "scheme": p.scheme, "class": p.class, "outcome": p.outcome, "fit": p.fit, "tried": p.tried,
    }))
    .ok()
    .and_then(|v| serde_json::from_value::<Value>(v).ok());
    for (kind, mk, ev) in [
        ("place.authored", Mark::Authored, Evidence::Authored),
        ("place.derive", Mark::Derived, Evidence::Derived),
        (
            "place.forced",
            Mark::AuthoredImpossible,
            Evidence::AuthoredImpossible,
        ),
    ] {
        let set = by(mk);
        if set.is_empty() {
            continue;
        }
        let e = Event::new(EventBody {
            kind: kind.into(),
            subject: vec![address.clone()],
            occurred_at: at,
            activated_at: at,
            recognised_at: ch.next_stamp(),
            causes: ids.clone(),
            payload: Payload {
                set,
                data: data.clone(),
                ..Default::default()
            },
            evidence: ev,
            publisher: Publisher::new("sse.place", "place", "1"),
        })
        .map_err(|e| format!("{e:?}"))?;
        ids.push(e.id);
        ch.append(e).map_err(|e| format!("{e:?}"))?;
    }
    for i in &p.interventions {
        let e = Event::new(EventBody {
            kind: "place.intervention".into(),
            subject: vec![address.clone()],
            occurred_at: at,
            activated_at: at,
            recognised_at: ch.next_stamp(),
            causes: ids.first().cloned().into_iter().collect(),
            payload: Payload {
                data: Some(Value::Text(i.clone())),
                ..Default::default()
            },
            evidence: Evidence::Authored,
            publisher: Publisher::new("sse.place", "place", "1"),
        })
        .map_err(|e| format!("{e:?}"))?;
        ids.push(e.id);
        ch.append(e).map_err(|e| format!("{e:?}"))?;
    }
    Ok(ids)
}

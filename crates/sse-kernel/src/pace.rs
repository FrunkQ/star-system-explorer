//! TP4: process declarations and the pace ladder (time design
//! `handover/design/SSE4-time-scales-design.md` sections 6, 9 and 10).
//!
//! Every process that depends on time declares, as pack data beside its
//! block (`provides.other.processes` in base-time), its shape (line,
//! stream or march, and no fourth), its own time `tau`, its anchor, the
//! marks it can publish, for a march its grid rule, and the kernel ops that
//! read it. `tau` is physics, never a typed number: a block of the library
//! run through the block door, an Equation carried in the declaration, the
//! own time of the process that drives it, or (for the campaign's record,
//! which has no pace in physics) the spacing of its own dated events. Each
//! declaration is worked out at its reference subjects, whose inputs are
//! rows of the data sheet in force.
//!
//! The ladder is the list of rungs ordered by own time, from the fastest.
//! It is what the clock (TP7) reads to know the watchable pace of what is
//! on screen. The physics never slows: nothing here changes a number any
//! other op works out.

use crate::err;
use crate::protocol::Response;
use serde::Deserialize;
use serde_json::{json, Value as J};
use std::collections::BTreeMap;

/// The settings key the declarations live under, each pack's
/// `{about, list}`; a pack higher in the stack replaces a declaration by
/// its id or adds one.
pub(crate) const KEY: &str = "processes";

/// The three shapes of design section 6.
const SHAPES: &[&str] = &["line", "stream", "march"];

/// The named anchors of design section 4, plus the axis's own zero (the
/// origin, ruling 12A.1). A date is the one axis read through one of these.
pub(crate) const ANCHORS: &[&str] = &[
    "time.origin",
    "time.present_s",
    "campaign.now_s",
    "system.birth_s",
    "time.j2000_s",
    "calendar.epoch_s",
    "orbit.epoch_s",
];

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Settings {
    #[serde(default)]
    #[allow(dead_code)]
    about: String,
    list: Vec<J>,
}

/// Where a process's own time comes from.
#[derive(Deserialize, Clone)]
#[serde(deny_unknown_fields)]
struct Tau {
    /// A block of the library, by id, and the output port that is the time.
    #[serde(default)]
    block: Option<String>,
    #[serde(default)]
    output: Option<String>,
    /// An Equation (`sse.math.equation` config) whose one output is a time.
    #[serde(default)]
    equation: Option<J>,
    /// The process that drives this one (section 7, follow): its own time.
    #[serde(default)]
    driver: Option<String>,
    /// No pace in physics: the spacing of its own dated events, and why.
    #[serde(default)]
    marks: Option<String>,
}

#[derive(Deserialize, Clone)]
#[serde(deny_unknown_fields)]
struct Subject {
    says: String,
    /// Each input (a block's port, or an Equation's letter) as a sheet row
    /// `sheet/<id>`.
    #[serde(default)]
    inputs: BTreeMap<String, String>,
}

/// One declaration (design section 6, "Every process declares").
#[derive(Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub(crate) struct Decl {
    pub id: String,
    says: String,
    shape: String,
    anchor: String,
    tau: Tau,
    #[serde(default)]
    subjects: Vec<Subject>,
    #[serde(default)]
    marks: Vec<String>,
    #[serde(default)]
    grid: Option<String>,
    pub ops: Vec<String>,
    how: String,
}

/// The declarations in force, bottom pack first, a later one replacing an
/// earlier of the same id; and those that could not be read, with why.
pub(crate) fn declarations() -> (Vec<(String, Decl)>, Vec<J>) {
    let physics = sse_physics::in_force();
    let mut by_id: BTreeMap<String, (String, Decl)> = BTreeMap::new();
    let mut order: Vec<String> = Vec::new();
    let mut refused = Vec::new();
    for (pack, v) in physics.settings_all(KEY) {
        let s: Settings = match sse_core::cbor::from_value(v) {
            Ok(s) => s,
            Err(e) => {
                refused.push(json!({"pack": pack, "why": format!("processes: {e}")}));
                continue;
            }
        };
        for d in s.list {
            let id = d.get("id").and_then(J::as_str).unwrap_or("").to_string();
            match serde_json::from_value::<Decl>(d) {
                Ok(d) => {
                    if !by_id.contains_key(&d.id) {
                        order.push(d.id.clone());
                    }
                    by_id.insert(d.id.clone(), (pack.clone(), d));
                }
                Err(e) => refused.push(json!({"pack": pack, "id": id, "why": e.to_string()})),
            }
        }
    }
    let list = order
        .into_iter()
        .filter_map(|id| by_id.remove(&id))
        .collect();
    (list, refused)
}

/// What is wrong with a declaration's form, if anything.
fn form_fault(d: &Decl, ids: &[&str]) -> Option<String> {
    if !SHAPES.contains(&d.shape.as_str()) {
        return Some(format!(
            "shape {:?} is not one of line, stream or march (section 6: no fourth)",
            d.shape
        ));
    }
    if !ANCHORS.contains(&d.anchor.as_str()) {
        return Some(format!(
            "anchor {:?} is not a named anchor ({})",
            d.anchor,
            ANCHORS.join(", ")
        ));
    }
    if d.shape == "march" && d.grid.as_deref().is_none_or(|g| g.trim().is_empty()) {
        return Some("a march gives its grid rule (section 6, the march rule)".into());
    }
    if d.shape != "march" && d.grid.is_some() {
        return Some(format!(
            "a {} has no grid; only a march is stepped",
            d.shape
        ));
    }
    let t = &d.tau;
    let kinds = [
        t.block.is_some(),
        t.equation.is_some(),
        t.driver.is_some(),
        t.marks.is_some(),
    ];
    if kinds.iter().filter(|k| **k).count() != 1 {
        return Some("tau is one of block, equation, driver or marks".into());
    }
    if t.block.is_some() != t.output.is_some() {
        return Some("a block's tau names its block and the output that is the time".into());
    }
    if let Some(dr) = &t.driver {
        if dr == &d.id || !ids.contains(&dr.as_str()) {
            return Some(format!("its driver {dr:?} is not another declared process"));
        }
    }
    if (t.block.is_some() || t.equation.is_some()) && d.subjects.is_empty() {
        return Some("a tau worked out from physics names at least one subject".into());
    }
    if d.ops.is_empty() {
        return Some("names no op that reads it".into());
    }
    if let Some(o) = d.ops.iter().find(|o| !is_op(o)) {
        return Some(format!("names op {o:?}, which the kernel does not have"));
    }
    None
}

/// Whether `name` is a request op the kernel takes (asking for it is never
/// "unknown variant", the test `the_op_lists_are_every_op` reads it so too).
pub(crate) fn is_op(name: &str) -> bool {
    match serde_json::from_value::<crate::protocol::Request>(json!({ "op": name })) {
        Ok(_) => true,
        Err(e) => !e.to_string().contains("unknown variant"),
    }
}

/// A subject's inputs, in `names` order, each from the sheet in force.
fn inputs_of(s: &Subject, names: &[String]) -> Result<Vec<f64>, String> {
    if let Some(k) = s.inputs.keys().find(|k| !names.contains(k)) {
        return Err(format!(
            "{k} is not one of its inputs ({})",
            names.join(", ")
        ));
    }
    names
        .iter()
        .map(|n| {
            let row = s
                .inputs
                .get(n)
                .ok_or_else(|| format!("{n} is not given for {}", s.says))?;
            let id = row
                .strip_prefix("sheet/")
                .ok_or_else(|| format!("{n} is {row:?}: an input is a sheet row, sheet/<id>"))?;
            sse_physics::constant(id).ok_or_else(|| {
                format!("{n} reads sheet/{id}, which the stack's data sheet does not hold (load base-reference)")
            })
        })
        .collect()
}

/// The own time of one subject, in seconds.
fn tau_at(t: &Tau, s: &Subject) -> Result<f64, String> {
    let secs = if let (Some(id), Some(out)) = (&t.block, &t.output) {
        let b = sse_physics::block(id).ok_or_else(|| format!("block {id} is not loaded"))?;
        let d = b.decl();
        let (k, port) = d
            .output(out)
            .ok_or_else(|| format!("block {id} has no output {out:?}"))?;
        if port.dim != sse_core::Dimension::TIME {
            return Err(format!("block {id}'s output {out} is not a time"));
        }
        let names: Vec<String> = d.inputs.iter().map(|p| p.name.clone()).collect();
        let x = inputs_of(s, &names)?;
        sse_physics::run(id, &x)?[k]
    } else if let Some(cfg) = &t.equation {
        let p = sse_formula::Parsed::new(Some(cfg))?;
        if p.out_dims.len() != 1 || p.out_dims[0] != sse_core::Dimension::TIME {
            return Err("its Equation has one output, a time".into());
        }
        let x = inputs_of(s, &p.inputs)?;
        p.run_numbers(&x)?[0]
    } else {
        return Err("no physics to work out".into());
    };
    if secs.is_finite() && secs > 0.0 {
        Ok(secs)
    } else {
        Err(format!("worked out {secs} s, not a time"))
    }
}

/// Where the rung's own time came from, for the reply.
fn tau_from(t: &Tau) -> J {
    if let Some(b) = &t.block {
        json!({"block": b, "output": t.output})
    } else if let Some(e) = &t.equation {
        json!({"equation": e.get("name"), "formula": e.get("formula"), "source": e.get("source")})
    } else if let Some(d) = &t.driver {
        json!({"driver": d})
    } else {
        json!({"marks": t.marks})
    }
}

/// `pace_ladder`: every declared process, one rung per subject, ordered by
/// own time (fastest first; those paced by their own marks last).
pub(crate) fn ladder() -> Result<Response, Response> {
    let (decls, mut refused) = declarations();
    let ids: Vec<&str> = decls.iter().map(|(_, d)| d.id.as_str()).collect();
    // Each process's own times, worked out once; a driver's are read below.
    let mut own: BTreeMap<String, Result<Vec<(String, f64)>, String>> = BTreeMap::new();
    let mut ok: Vec<&(String, Decl)> = Vec::new();
    for pd in &decls {
        let d = &pd.1;
        if let Some(f) = form_fault(d, &ids) {
            refused.push(json!({"pack": pd.0, "id": d.id, "why": f}));
            continue;
        }
        ok.push(pd);
        if d.tau.driver.is_none() && d.tau.marks.is_none() {
            let r: Result<Vec<_>, String> = d
                .subjects
                .iter()
                .map(|s| {
                    tau_at(&d.tau, s)
                        .map(|t| (s.says.clone(), t))
                        .map_err(|e| format!("{}: {e}", s.says))
                })
                .collect();
            own.insert(d.id.clone(), r);
        }
    }
    let mut rungs: Vec<(Option<f64>, String, J)> = Vec::new();
    for (pack, d) in ok {
        let base = |subject: Option<&str>, tau: Option<f64>, why: Option<String>| {
            json!({
                "id": d.id, "says": d.says, "shape": d.shape, "anchor": d.anchor,
                "subject": subject, "tau_s": tau, "tau_from": tau_from(&d.tau), "why": why,
                "marks": d.marks, "grid": d.grid, "ops": d.ops, "how": d.how, "pack": pack,
            })
        };
        let times = match (&d.tau.driver, &d.tau.marks) {
            (Some(dr), _) => match own.get(dr) {
                Some(Ok(v)) => Ok(v.clone()),
                Some(Err(e)) => Err(format!("its driver {dr}: {e}")),
                None => Err(format!(
                    "its driver {dr} has no own time of its own to follow"
                )),
            },
            (None, Some(_)) => {
                rungs.push((None, d.id.clone(), base(None, None, None)));
                continue;
            }
            _ => own.get(&d.id).cloned().unwrap_or(Ok(vec![])),
        };
        match times {
            Ok(v) => {
                for (s, t) in v {
                    rungs.push((Some(t), d.id.clone(), base(Some(&s), Some(t), None)));
                }
            }
            Err(e) => rungs.push((None, d.id.clone(), base(None, None, Some(e)))),
        }
    }
    rungs.sort_by(|a, b| match (a.0, b.0) {
        (Some(x), Some(y)) => x.total_cmp(&y).then_with(|| a.1.cmp(&b.1)),
        (Some(_), None) => std::cmp::Ordering::Less,
        (None, Some(_)) => std::cmp::Ordering::Greater,
        (None, None) => a.1.cmp(&b.1),
    });
    if decls.is_empty() && refused.is_empty() {
        return Err(err(
            "no_processes",
            "the stack declares no processes (load base-time: provides.other.processes)",
        ));
    }
    Ok(Response::PaceLadder {
        ladder: json!({
            "processes": decls.len(),
            "rungs": rungs.into_iter().map(|r| r.2).collect::<Vec<_>>(),
            "refused": refused,
        }),
    })
}

//! ND6 (new-physics demo 2.5): each pack's own sources, fudge list and
//! reproduction report, worked out by this kernel when the stack loads.
//! The fudge list holds the pack's declared fudges and, found by the kernel,
//! every Model number of its flows and boxes graded `fitted` or `assumed`.
//! A reproduction check outside its stated tolerance refuses the stack,
//! with the published number and the pack's side by side.

use serde::Serialize;
use sse_flow::flow::NodeDoc;
use sse_flow::{Context, Flow};
use sse_pack::trust::{Fudge, Source};
use sse_pack::PackManifest;
use std::collections::BTreeMap;

/// One entry of a pack's fudge list.
#[derive(Clone, Debug, Serialize)]
pub struct FudgeRow {
    pub id: String,
    pub what: String,
    pub stands_in_for: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub replace_with: Option<String>,
    pub flows: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    /// `declared` (the pack's own list) or `model_number` (found by the kernel).
    pub from: &'static str,
    /// For a Model number: its value, unit and grade.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub number: Option<serde_json::Value>,
}

/// One published number beside the pack's.
#[derive(Clone, Debug, Serialize)]
pub struct Row {
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unit: Option<String>,
    pub published: f64,
    /// What the pack's flow gives, in the same unit; none when it could not run.
    pub got: Option<f64>,
    /// The pack's stated closeness, relative.
    pub tolerance: f64,
    /// How far off, relative to the published number.
    pub off: Option<f64>,
    pub pass: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct CheckReport {
    pub name: String,
    pub source: String,
    #[serde(rename = "where", skip_serializing_if = "Option::is_none")]
    pub at: Option<String>,
    pub flow: String,
    pub rows: Vec<Row>,
    pub pass: bool,
}

#[derive(Clone, Debug, Serialize)]
pub struct PackReport {
    pub pack: String,
    pub version: String,
    pub licence: String,
    pub sources: Vec<Source>,
    pub fudges: Vec<FudgeRow>,
    pub reproduction: Vec<CheckReport>,
    /// Every check within its tolerance (true when there are none).
    pub reproduced: bool,
}

fn declared(f: &Fudge) -> FudgeRow {
    FudgeRow {
        id: f.id.clone(),
        what: f.what.clone(),
        stands_in_for: f.stands_in_for.clone(),
        replace_with: f.replace_with.clone(),
        flows: f.flows.clone(),
        source: f.source.clone(),
        from: "declared",
        number: None,
    }
}

/// Fitted and assumed Model numbers in these nodes (equation design 3).
fn model_numbers(owner: &str, nodes: &[NodeDoc], out: &mut Vec<FudgeRow>) {
    for n in nodes {
        if n.atom.as_deref() != Some("sse.model.number") {
            continue;
        }
        let Some(c) = &n.config else { continue };
        let grade = c.get("grade").and_then(|g| g.as_str()).unwrap_or("");
        if grade != "fitted" && grade != "assumed" {
            continue;
        }
        let name = c.get("name").and_then(|g| g.as_str()).unwrap_or(&n.id);
        let src = c.get("source");
        let cite = src
            .and_then(|s| {
                s.get("doi")
                    .and_then(|d| d.as_str())
                    .map(|d| format!("doi:{d}"))
            })
            .or_else(|| src.and_then(|s| s.get("url").and_then(|d| d.as_str()).map(String::from)));
        out.push(FudgeRow {
            id: format!("{owner}/{}", n.id),
            what: format!("{name}: a {grade} number of the model"),
            stands_in_for: if grade == "fitted" {
                "a value fitted to data, not worked out from nature".into()
            } else {
                "an assumed value, not worked out from nature or fitted to data".into()
            },
            replace_with: None,
            flows: vec![owner.to_string()],
            source: cite,
            from: "model_number",
            number: Some(serde_json::json!({
                "value": c.get("value"),
                "unit": c.get("unit"),
                "grade": grade,
                "source": src,
            })),
        });
    }
}

/// Every pack's report, and the problems that refuse the stack.
pub fn reports(
    packs: &[PackManifest],
    flows: &BTreeMap<String, Flow>,
    cx: &Context,
) -> (Vec<PackReport>, Vec<String>) {
    let mut out = vec![];
    let mut problems = vec![];
    for p in packs {
        let mut fudges: Vec<FudgeRow> = p.fudges.iter().map(declared).collect();
        for f in &p.provides.flows {
            model_numbers(&f.id, &f.nodes, &mut fudges);
        }
        for b in &p.provides.boxes {
            model_numbers(&b.id, &b.nodes, &mut fudges);
        }
        let mut reproduction = vec![];
        for c in &p.reproduction {
            let mut rows = vec![];
            let run = match flows.get(&c.flow) {
                None => Err(vec![format!("the stack has no flow {}", c.flow)]),
                Some(f) => f.run_case(cx, &c.inputs),
            };
            for (kind, e) in &c.expect {
                let mut row = Row {
                    kind: kind.clone(),
                    unit: e.unit.clone(),
                    published: e.value,
                    got: None,
                    tolerance: e.tolerance,
                    off: None,
                    pass: false,
                    error: None,
                };
                let got = match &run {
                    Err(es) => Err(es.join("; ")),
                    Ok(ev) => match ev.published(kind) {
                        None => Err(format!("{} does not publish {kind}", c.flow)),
                        Some(q) => match &e.unit {
                            Some(u) => cx.units.to(q, u).map_err(|x| x.to_string()),
                            None => Ok(q.value),
                        },
                    },
                };
                match got {
                    Ok(g) => {
                        let off = if e.value == 0.0 {
                            (g - e.value).abs()
                        } else {
                            ((g - e.value) / e.value).abs()
                        };
                        row.got = Some(g);
                        row.off = Some(off);
                        row.pass = off <= e.tolerance;
                    }
                    Err(m) => row.error = Some(m),
                }
                if !row.pass {
                    let unit = e
                        .unit
                        .as_deref()
                        .map(|u| format!(" {u}"))
                        .unwrap_or_default();
                    problems.push(match (row.got, &row.error) {
                        (Some(g), _) => format!(
                            "{}: reproduction check {:?}: {kind}: {} gives {g}{unit}, {} gives {}{unit}, {:.3} % off, the pack allows {:.3} %",
                            p.id,
                            c.name,
                            p.id,
                            c.source,
                            e.value,
                            100.0 * row.off.unwrap_or(0.0),
                            100.0 * e.tolerance
                        ),
                        (None, m) => format!(
                            "{}: reproduction check {:?}: {kind}: could not run ({}); {} gives {}{unit}",
                            p.id,
                            c.name,
                            m.as_deref().unwrap_or(""),
                            c.source,
                            e.value
                        ),
                    });
                }
                rows.push(row);
            }
            reproduction.push(CheckReport {
                name: c.name.clone(),
                source: c.source.clone(),
                at: c.at.clone(),
                flow: c.flow.clone(),
                pass: rows.iter().all(|r| r.pass),
                rows,
            });
        }
        out.push(PackReport {
            pack: p.id.clone(),
            version: p.version.clone(),
            licence: p.licence.clone(),
            sources: p.sources.clone(),
            fudges,
            reproduced: reproduction.iter().all(|c| c.pass),
            reproduction,
        });
    }
    (out, problems)
}

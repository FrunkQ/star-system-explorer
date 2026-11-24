//! FT2, technology as how far a civilisation has pushed each limit (future
//! tech design 5 to 8): the stack's ladder, figures and advances
//! (`provides.other.tech`, `packs/base-tech`), a civilisation's level and
//! exceptions from the record (`sse_record::tech`), and the availability
//! reading of a construct's needs, which never says no.
//!
//! - **Needs from parts**: a design lists its parts, each with the figures
//!   it leans on and the value it needs; nothing keeps a "requires" list.
//! - **The size rule**: a part whose advance is n levels above its builder
//!   is `size_per_level_short`ⁿ times the size (a prototype); one m levels
//!   below shrinks by `shrink_per_level_beyond`^min(m, `shrink_levels`).
//! - **Fictional by ceiling**: a need past a figure's ceiling mark is
//!   fictional by arithmetic, and the reading names the law it breaks.

use serde::Serialize;
use std::collections::BTreeMap;

#[derive(Clone, PartialEq, Debug, Serialize)]
pub struct Mark {
    pub mark: String,
    pub v: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub law: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub what: Option<String>,
    /// A ceiling that holds only for one fuel or material (`"for"` in the
    /// pack): it is shown beside that thing, never applied to the figure.
    #[serde(rename = "for", skip_serializing_if = "Option::is_none")]
    pub for_: Option<String>,
}

#[derive(Clone, PartialEq, Debug, Serialize)]
pub struct Figure {
    pub id: String,
    pub plain: String,
    pub unit: String,
    /// True when a bigger number is better.
    pub higher: bool,
    pub marks: Vec<Mark>,
}

impl Figure {
    /// Does `have` meet `need` on this figure?
    pub fn meets(&self, have: f64, need: f64) -> bool {
        if self.higher {
            have >= need
        } else {
            have <= need
        }
    }
    /// The ceiling a need passes, if any. A ceiling `for` one fuel or
    /// material (hydrogen-oxygen's exhaust, tungsten melting) is not the
    /// figure's own and is skipped.
    pub fn past_ceiling(&self, need: f64) -> Option<&Mark> {
        self.marks
            .iter()
            .filter(|m| m.mark == "ceiling" && m.for_.is_none())
            .find(|m| !self.meets(m.v, need))
    }
}

#[derive(Clone, PartialEq, Debug, Serialize)]
pub struct Advance {
    pub id: String,
    pub name: String,
    pub level: i64,
    pub sets: BTreeMap<String, f64>,
    pub part: String,
    pub mark: String,
    pub pack: String,
}

#[derive(Clone, PartialEq, Debug, Serialize)]
pub struct Level {
    pub n: i64,
    pub id: String,
    pub name: String,
}

#[derive(Clone, PartialEq, Debug, Serialize)]
pub struct Ladder {
    pub id: String,
    pub name: String,
    pub size_per_level_short: f64,
    pub shrink_per_level_beyond: f64,
    pub shrink_levels: i64,
    pub levels: Vec<Level>,
    pub pack: String,
}

/// x multiplied by itself n times (n ≥ 0), exactly as written out.
fn times(x: f64, n: i64) -> f64 {
    (0..n).fold(1.0, |a, _| a * x)
}

impl Ladder {
    /// The size factor of a part whose advance sits `short` levels above its
    /// builder (negative: below).
    pub fn size_factor(&self, short: i64) -> f64 {
        if short > 0 {
            times(self.size_per_level_short, short)
        } else {
            let beyond = (-short).min(self.shrink_levels).max(0);
            times(self.shrink_per_level_beyond, beyond)
        }
    }
}

/// The stack's technology: one ladder (the top pack's), every figure and
/// every advance by id, later packs replacing earlier ones.
#[derive(Clone, PartialEq, Debug, Default)]
pub struct Tech {
    pub ladder: Option<Ladder>,
    pub figures: BTreeMap<String, Figure>,
    pub advances: BTreeMap<String, Advance>,
}

fn num(v: &serde_json::Value) -> Option<f64> {
    v.as_f64().filter(|x| x.is_finite())
}

impl Tech {
    pub fn from_packs(packs: &[(String, serde_json::Value)]) -> Result<Tech, Vec<String>> {
        let mut t = Tech::default();
        let mut problems = vec![];
        for (pack, v) in packs {
            for f in v["figures"].as_array().into_iter().flatten() {
                let id = f["id"].as_str().unwrap_or("").to_string();
                let marks = f["marks"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .filter_map(|m| {
                        Some(Mark {
                            mark: m["mark"].as_str()?.to_string(),
                            v: num(&m["v"])?,
                            law: m["law"].as_str().map(str::to_string),
                            what: m["what"].as_str().map(str::to_string),
                            for_: m["for"].as_str().map(str::to_string),
                        })
                    })
                    .collect::<Vec<_>>();
                let higher = match f["better"].as_str() {
                    Some("higher") => true,
                    Some("lower") => false,
                    _ => {
                        problems.push(format!("{pack}: figure {id:?}: better is higher or lower"));
                        continue;
                    }
                };
                if marks.iter().any(|m| m.mark == "ceiling" && m.law.is_none()) {
                    problems.push(format!("{pack}: figure {id:?}: a ceiling names its law"));
                }
                t.figures.insert(
                    id.clone(),
                    Figure {
                        plain: f["plain"].as_str().unwrap_or(&id).to_string(),
                        unit: f["unit"].as_str().unwrap_or("1").to_string(),
                        id,
                        higher,
                        marks,
                    },
                );
            }
            let l = &v["ladder"];
            if l.is_object() {
                let levels = l["levels"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .filter_map(|x| {
                        Some(Level {
                            n: x["n"].as_i64()?,
                            id: x["id"].as_str()?.to_string(),
                            name: x["name"].as_str().unwrap_or("").to_string(),
                        })
                    })
                    .collect::<Vec<_>>();
                let size = num(&l["size_per_level_short"]).unwrap_or(10.0);
                let shrink = num(&l["shrink_per_level_beyond"]).unwrap_or(1.0);
                if size < 1.0 || !(0.0..=1.0).contains(&shrink) || levels.is_empty() {
                    problems.push(format!(
                        "{pack}: ladder: levels, size_per_level_short of 1 or more, shrink_per_level_beyond of 0 to 1"
                    ));
                }
                t.ladder = Some(Ladder {
                    id: l["id"].as_str().unwrap_or("ladder").to_string(),
                    name: l["name"].as_str().unwrap_or("").to_string(),
                    size_per_level_short: size,
                    shrink_per_level_beyond: shrink,
                    shrink_levels: l["shrink_levels"].as_i64().unwrap_or(0),
                    levels,
                    pack: pack.clone(),
                });
            }
            for a in v["advances"].as_array().into_iter().flatten() {
                let id = a["id"].as_str().unwrap_or("").to_string();
                let sets: BTreeMap<String, f64> = a["sets"]
                    .as_object()
                    .into_iter()
                    .flatten()
                    .filter_map(|(k, x)| Some((k.clone(), num(x)?)))
                    .collect();
                let Some(level) = a["level"].as_i64() else {
                    problems.push(format!("{pack}: advance {id:?}: no level"));
                    continue;
                };
                t.advances.insert(
                    id.clone(),
                    Advance {
                        name: a["name"].as_str().unwrap_or(&id).to_string(),
                        id,
                        level,
                        sets,
                        part: a["part"].as_str().unwrap_or("").to_string(),
                        mark: a["mark"].as_str().unwrap_or("").to_string(),
                        pack: pack.clone(),
                    },
                );
            }
        }
        for a in t.advances.values() {
            for f in a.sets.keys() {
                if !t.figures.contains_key(f) {
                    problems.push(format!(
                        "{}: advance {:?} sets unknown figure {f:?}",
                        a.pack, a.id
                    ));
                }
            }
        }
        if problems.is_empty() {
            Ok(t)
        } else {
            Err(problems)
        }
    }

    /// Does a civilisation hold an advance? At or below its level unless
    /// never taken up, or gained early.
    pub fn holds(&self, of: &sse_record::tech::TechOf, a: &Advance) -> bool {
        if of.gained.contains_key(&a.id) {
            return true;
        }
        if of.lost.contains_key(&a.id) {
            return false;
        }
        of.level.as_ref().is_some_and(|(l, _)| a.level <= *l)
    }
}

/// One part of a design and the figures it leans on.
#[derive(Clone, PartialEq, Debug, serde::Deserialize)]
pub struct PartNeed {
    pub part: String,
    pub needs: BTreeMap<String, f64>,
}

fn fmt_x(x: f64) -> String {
    if x >= 100.0 {
        format!("{:.0}", x)
    } else if x >= 10.0 {
        format!("{:.1}", x)
    } else {
        format!("{:.2}", x)
    }
}

/// The availability reading of a design through a civilisation (design 7):
/// `can build`, `short` (with the prototype's size) or `past known`, per
/// need and overall, with the advances it leans on and their inks.
pub fn reading(
    tech: &Tech,
    of: &sse_record::tech::TechOf,
    parts: &[PartNeed],
) -> Result<serde_json::Value, String> {
    let ladder = tech
        .ladder
        .as_ref()
        .ok_or("no tech ladder in the pack stack (load packs/base-tech or a setting's own)")?;
    let level = of.level.as_ref().map(|(l, _)| *l);
    let mut rows = vec![];
    let mut leans: BTreeMap<String, serde_json::Value> = BTreeMap::new();
    let mut worst = 0; // 0 can build, 1 short, 2 past known
    for p in parts {
        for (fid, &need) in &p.needs {
            let f = tech
                .figures
                .get(fid)
                .ok_or_else(|| format!("part {:?}: no figure {fid:?} on the stack", p.part))?;
            if !need.is_finite() {
                return Err(format!("part {:?}: {fid} must be a finite number", p.part));
            }
            let fiction = f.past_ceiling(need).map(|m| {
                format!(
                    "past the ceiling of {} {} {}: breaks {}",
                    fmt_x(m.v),
                    f.unit,
                    f.plain,
                    m.law.as_deref().unwrap_or("a law")
                )
            });
            // Advances on the ladder that meet the need, lowest level first.
            let mut meets: Vec<&Advance> = tech
                .advances
                .values()
                .filter(|a| a.sets.get(fid).is_some_and(|&v| f.meets(v, need)))
                .collect();
            // An advance made for this kind of part first (a "drive" need is
            // not met by an "electric drive", which also needs a power plant),
            // then the lowest level, then by id.
            meets.sort_by(|a, b| {
                (a.part != p.part)
                    .cmp(&(b.part != p.part))
                    .then(a.level.cmp(&b.level))
                    .then(a.id.cmp(&b.id))
            });
            let held = meets.iter().find(|a| tech.holds(of, a));
            let mut row = serde_json::json!({
                "part": p.part, "figure": fid, "plain": f.plain, "unit": f.unit, "need": need,
            });
            if let Some(w) = &fiction {
                row["fictional"] = serde_json::json!(w);
            }
            let (reading, by, factor) = match (held, meets.first()) {
                (Some(a), _) => {
                    let beyond = level.map_or(0, |l| a.level - l);
                    ("can build", Some(*a), ladder.size_factor(beyond.min(0)))
                }
                (None, Some(a)) => {
                    let short = level.map_or(a.level + 1, |l| (a.level - l).max(1));
                    row["levels_short"] = serde_json::json!(short);
                    worst = worst.max(1);
                    ("short", Some(*a), ladder.size_factor(short))
                }
                (None, None) => {
                    // Their best on this figure, for "needs N times".
                    let best = tech
                        .advances
                        .values()
                        .filter(|a| tech.holds(of, a))
                        .filter_map(|a| a.sets.get(fid).copied())
                        .fold(None, |b: Option<f64>, v| match b {
                            None => Some(v),
                            Some(x) => Some(if f.meets(v, x) { v } else { x }),
                        });
                    if let Some(b) = best.filter(|b| *b != 0.0) {
                        let times = if f.higher { need / b } else { b / need };
                        row["times_their_best"] = serde_json::json!(times);
                        row["words"] = serde_json::json!(format!(
                            "needs {} times their best {} ({} {})",
                            fmt_x(times),
                            f.plain,
                            fmt_x(b),
                            f.unit
                        ));
                    } else {
                        row["words"] = serde_json::json!(format!(
                            "past anything known on this setting's ladder for {}",
                            f.plain
                        ));
                    }
                    worst = 2;
                    ("past known", None, 1.0)
                }
            };
            row["reading"] = serde_json::json!(reading);
            row["size_factor"] = serde_json::json!(factor);
            if let Some(a) = by {
                row["advance"] = serde_json::json!(a.id);
                row["home_level"] = serde_json::json!(a.level);
                if reading == "short" {
                    row["words"] = serde_json::json!(format!(
                        "{} short: {} (a prototype {} times the size)",
                        if row["levels_short"] == 1 {
                            "one level".to_string()
                        } else {
                            format!("{} levels", row["levels_short"])
                        },
                        a.name,
                        fmt_x(factor)
                    ));
                }
                let ink = if fiction.is_some() {
                    "fictional"
                } else {
                    a.mark.as_str()
                };
                leans.entry(a.id.clone()).or_insert_with(|| {
                    serde_json::json!({"advance": a.id, "name": a.name, "ink": ink, "level": a.level, "pack": a.pack})
                });
            }
            rows.push(row);
        }
    }
    // A fictional need makes its advance's chip fictional too.
    for r in &rows {
        if let (Some(a), Some(_)) = (r["advance"].as_str(), r.get("fictional")) {
            if let Some(c) = leans.get_mut(a) {
                c["ink"] = serde_json::json!("fictional");
            }
        }
    }
    let mut part_sizes: BTreeMap<String, f64> = BTreeMap::new();
    for r in &rows {
        let p = r["part"].as_str().unwrap_or("").to_string();
        let f = r["size_factor"].as_f64().unwrap_or(1.0);
        let e = part_sizes.entry(p).or_insert(1.0);
        *e = e.max(f);
    }
    let overall = ["can build", "short", "past known"][worst];
    Ok(serde_json::json!({
        "reading": overall,
        "level": level,
        "ladder": ladder.id,
        "needs": rows,
        "part_sizes": part_sizes,
        "leans_on": leans.into_values().collect::<Vec<_>>(),
        "fictional": rows.iter().any(|r| r.get("fictional").is_some()),
    }))
}

//! FT2, technology as how far a civilisation has pushed each limit (future
//! tech design 5 to 8): the stack's ladder, figures and advances
//! (`provides.other.tech`, `packs/base-tech`), a civilisation's level and
//! exceptions from the record (`sse_record::tech`), and the availability
//! reading of a construct's needs, which never says no.
//!
//! - **Needs from parts**: a design lists its parts, each with the figures
//!   it leans on and the value it needs; nothing keeps a "requires" list.
//! - **Maturity acts on the figure** (design 5.2): an advance n levels above
//!   its builder gives a first-generation figure by its family's rule
//!   (`worse_by` a factor per level, `toward` a share of the way from their
//!   older figure, `same`, `floor`), never worse than the best older advance
//!   they hold for the same figure. Past its home level a figure improves by
//!   `improve_per_level_beyond` per level until its next cited allowed or
//!   ceiling mark. Figures an advance lists as `fixed` never move. The part's
//!   size follows from the figure (first order: inversely); only the
//!   `fictional` family keeps the flat rule of thumb, `size_per_level_short`ⁿ
//!   short and `shrink_per_level_beyond`^min(m, `shrink_levels`) beyond.
//! - **Fictional by ceiling**: a need past a figure's ceiling mark is
//!   fictional by arithmetic, and the reading names the law it breaks. A mark
//!   `for` one fuel counts only for an advance or a design of that fuel.

use serde::Serialize;
use std::collections::BTreeMap;

#[derive(Clone, PartialEq, Debug, Serialize)]
pub struct Mark {
    pub mark: String,
    pub v: f64,
    /// A mark for one fuel only (an advance's or design's `fuel`).
    #[serde(rename = "for", skip_serializing_if = "Option::is_none")]
    pub for_fuel: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub law: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub what: Option<String>,
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
    /// The ceiling a need passes, if any, for a part of fuel `fuel`.
    pub fn past_ceiling(&self, need: f64, fuel: Option<&str>) -> Option<&Mark> {
        self.marks
            .iter()
            .filter(|m| m.mark == "ceiling" && applies(m, fuel))
            .find(|m| !self.meets(m.v, need))
    }
    /// The next cited allowed or ceiling mark strictly better than `v`.
    pub fn next_mark(&self, v: f64, fuel: Option<&str>) -> Option<f64> {
        self.marks
            .iter()
            .filter(|m| (m.mark == "allowed" || m.mark == "ceiling") && applies(m, fuel))
            .map(|m| m.v)
            .filter(|&x| x != v && self.meets(x, v))
            .fold(None, |b: Option<f64>, x| match b {
                Some(y) if self.meets(x, y) => Some(y),
                _ => Some(x),
            })
    }
    /// The better of two values.
    pub fn best(&self, a: f64, b: f64) -> f64 {
        if self.meets(a, b) {
            a
        } else {
            b
        }
    }
    /// How many times better `a` is than `b` (1 or more when a is better).
    pub fn ratio(&self, a: f64, b: f64) -> f64 {
        if self.higher {
            a / b
        } else {
            b / a
        }
    }
}

fn applies(m: &Mark, fuel: Option<&str>) -> bool {
    match &m.for_fuel {
        None => true,
        Some(f) => fuel == Some(f.as_str()),
    }
}

/// A family's maturity rule (design 5.2; the defaults are rules of thumb in
/// the fudge register).
#[derive(Clone, PartialEq, Debug, Serialize)]
#[serde(tag = "rule", rename_all = "snake_case")]
pub enum Maturity {
    /// The figure divided by `factor` per level short (multiplied where lower is better).
    WorseBy { factor: f64 },
    /// One level short, `share` of the way from their older figure to the mature one.
    Toward { share: f64 },
    /// No change: mass is mass.
    Same,
    /// No change, and never smaller: the advance states its floor.
    Floor,
    /// Nothing known: the figure stays, the part's size scales (fictional devices).
    Size {
        size_per_level_short: f64,
        shrink_per_level_beyond: f64,
        shrink_levels: i64,
    },
}

#[derive(Clone, PartialEq, Debug, Serialize)]
pub struct Family {
    pub id: String,
    pub name: String,
    pub maturity: Maturity,
}

impl Family {
    fn fictional() -> Family {
        Family {
            id: "fictional".into(),
            name: "Fictional devices and anything unknown".into(),
            maturity: Maturity::Size {
                size_per_level_short: 10.0,
                shrink_per_level_beyond: 0.5,
                shrink_levels: 2,
            },
        }
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
    /// The family whose maturity rule it follows (absent: fictional).
    pub family: String,
    /// Figures physics fixes, which maturity never moves.
    pub fixed: Vec<String>,
    /// The fuel it burns, for marks `for` one fuel.
    pub fuel: Option<String>,
    /// For a fictional advance: where its light, heat, momentum and energy go.
    pub r#where: Option<serde_json::Value>,
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
    /// Past an advance's home level its figure improves by this per level.
    pub improve_per_level_beyond: f64,
    pub levels: Vec<Level>,
    pub pack: String,
}

/// x multiplied by itself n times (n ≥ 0), exactly as written out.
fn times(x: f64, n: i64) -> f64 {
    (0..n).fold(1.0, |a, _| a * x)
}

/// The stack's technology: one ladder (the top pack's), every figure and
/// every advance by id, later packs replacing earlier ones.
#[derive(Clone, PartialEq, Debug, Default)]
pub struct Tech {
    pub ladder: Option<Ladder>,
    pub figures: BTreeMap<String, Figure>,
    pub advances: BTreeMap<String, Advance>,
    pub families: BTreeMap<String, Family>,
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
                            for_fuel: m["for"].as_str().map(str::to_string),
                            law: m["law"].as_str().map(str::to_string),
                            what: m["what"].as_str().map(str::to_string),
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
                let improve = num(&l["improve_per_level_beyond"]).unwrap_or(1.0);
                if improve < 1.0 || levels.is_empty() {
                    problems.push(format!(
                        "{pack}: ladder: levels, and improve_per_level_beyond of 1 or more"
                    ));
                }
                t.ladder = Some(Ladder {
                    id: l["id"].as_str().unwrap_or("ladder").to_string(),
                    name: l["name"].as_str().unwrap_or("").to_string(),
                    improve_per_level_beyond: improve,
                    levels,
                    pack: pack.clone(),
                });
            }
            for f in v["families"].as_array().into_iter().flatten() {
                let id = f["id"].as_str().unwrap_or("").to_string();
                let m = &f["maturity"];
                let n = |k: &str| num(&m[k]);
                let maturity = match m["rule"].as_str() {
                    Some("worse_by") if n("factor").is_some_and(|x| x >= 1.0) => {
                        Maturity::WorseBy {
                            factor: n("factor").unwrap_or(1.0),
                        }
                    }
                    Some("toward") if n("share").is_some_and(|x| (0.0..=1.0).contains(&x)) => {
                        Maturity::Toward {
                            share: n("share").unwrap_or(1.0),
                        }
                    }
                    Some("same") => Maturity::Same,
                    Some("floor") => Maturity::Floor,
                    Some("size") => Maturity::Size {
                        size_per_level_short: n("size_per_level_short").unwrap_or(10.0),
                        shrink_per_level_beyond: n("shrink_per_level_beyond").unwrap_or(0.5),
                        shrink_levels: m["shrink_levels"].as_i64().unwrap_or(2),
                    },
                    _ => {
                        problems.push(format!(
                            "{pack}: family {id:?}: maturity is worse_by (factor 1 or more), toward (share 0 to 1), same, floor or size"
                        ));
                        continue;
                    }
                };
                t.families.insert(
                    id.clone(),
                    Family {
                        name: f["name"].as_str().unwrap_or(&id).to_string(),
                        id,
                        maturity,
                    },
                );
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
                        family: a["family"].as_str().unwrap_or("fictional").to_string(),
                        fixed: a["fixed"]
                            .as_array()
                            .into_iter()
                            .flatten()
                            .filter_map(|x| x.as_str().map(str::to_string))
                            .collect(),
                        fuel: a["fuel"].as_str().map(str::to_string),
                        r#where: a.get("where").cloned(),
                    },
                );
            }
        }
        t.families
            .entry("fictional".into())
            .or_insert_with(Family::fictional);
        for a in t.advances.values() {
            if !t.families.contains_key(&a.family) {
                problems.push(format!(
                    "{}: advance {:?}: no family {:?}",
                    a.pack, a.id, a.family
                ));
            }
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
    /// The fuel the part burns, for ceilings `for` one fuel.
    #[serde(default)]
    pub fuel: Option<String>,
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

/// What maturity makes of one advance's figure `d` levels above its builder.
struct Matured {
    /// The figure as built now.
    now: f64,
    /// The rule that set it: fixed, worse_by, toward, same, floor, size, improved, home.
    how: &'static str,
    /// The older advance it is floored at, by name.
    floored_at: Option<String>,
    size_factor: f64,
}

#[allow(clippy::too_many_arguments)]
fn matured(
    tech: &Tech,
    ladder: &Ladder,
    of: &sse_record::tech::TechOf,
    a: &Advance,
    f: &Figure,
    fid: &str,
    d: i64,
    fuel: Option<&str>,
) -> Matured {
    let v = a.sets[fid];
    let fam = tech
        .families
        .get(&a.family)
        .map_or_else(Family::fictional, Clone::clone);
    let plain = |now: f64, how| Matured {
        now,
        how,
        floored_at: None,
        size_factor: if now == 0.0 || v == 0.0 {
            1.0
        } else {
            f.ratio(v, now)
        },
    };
    if let Maturity::Size {
        size_per_level_short,
        shrink_per_level_beyond,
        shrink_levels,
    } = fam.maturity
    {
        let size_factor = if d > 0 {
            times(size_per_level_short, d)
        } else {
            times(shrink_per_level_beyond, (-d).min(shrink_levels).max(0))
        };
        return Matured {
            now: v,
            how: "size",
            floored_at: None,
            size_factor,
        };
    }
    if a.fixed.iter().any(|x| x == fid) || d == 0 {
        return plain(v, if d == 0 { "home" } else { "fixed" });
    }
    if d < 0 {
        // Past home: better by the ladder's step per level, up to the next mark.
        return match fam.maturity {
            Maturity::WorseBy { .. } | Maturity::Toward { .. } => match f.next_mark(v, fuel) {
                Some(cap) => {
                    let k = times(ladder.improve_per_level_beyond, -d);
                    let up = if f.higher { v * k } else { v / k };
                    plain(if f.meets(up, cap) { cap } else { up }, "improved")
                }
                None => plain(v, "improved"),
            },
            _ => plain(v, "same"),
        };
    }
    // Short: their best older advance for the same figure and part.
    let older = tech
        .advances
        .values()
        .filter(|o| o.id != a.id && o.level < a.level && tech.holds(of, o))
        .filter(|o| a.part.is_empty() || o.part.is_empty() || o.part == a.part)
        .filter_map(|o| o.sets.get(fid).map(|&x| (x, o)))
        .fold(None, |b: Option<(f64, &Advance)>, (x, o)| match b {
            Some((y, _)) if !f.meets(x, y) || x == y => b,
            _ => Some((x, o)),
        });
    let first = match fam.maturity {
        Maturity::WorseBy { factor } => {
            let k = times(factor, d);
            if f.higher {
                v / k
            } else {
                v * k
            }
        }
        Maturity::Toward { share } => match older {
            Some((x, _)) => x + (v - x) * times(share, d),
            None => v,
        },
        Maturity::Floor => return plain(v, "floor"),
        _ => return plain(v, "same"),
    };
    let how = match fam.maturity {
        Maturity::WorseBy { .. } => "worse_by",
        _ => "toward",
    };
    match older {
        Some((x, o)) if !f.meets(first, x) || first == x => Matured {
            floored_at: Some(o.name.clone()),
            ..plain(x, how)
        },
        _ => plain(first, how),
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
            // Advances on the ladder whose mature figure meets the need,
            // lowest level first.
            let mut meets: Vec<&Advance> = tech
                .advances
                .values()
                .filter(|a| a.sets.get(fid).is_some_and(|&v| f.meets(v, need)))
                .collect();
            meets.sort_by(|a, b| a.level.cmp(&b.level).then(a.id.cmp(&b.id)));
            let held = meets.iter().find(|a| tech.holds(of, a));
            let by = held.or(meets.first()).copied();
            let fuel = p.fuel.as_deref().or(by.and_then(|a| a.fuel.as_deref()));
            let fiction = f.past_ceiling(need, fuel).map(|m| {
                format!(
                    "past the ceiling of {} {} {}: breaks {}",
                    fmt_x(m.v),
                    f.unit,
                    f.plain,
                    m.law.as_deref().unwrap_or("a law")
                )
            });
            let mut row = serde_json::json!({
                "part": p.part, "figure": fid, "plain": f.plain, "unit": f.unit, "need": need,
            });
            if let Some(w) = &fiction {
                row["fictional"] = serde_json::json!(w);
            }
            let reading = match (held, by) {
                (Some(_), _) => "can build",
                (None, Some(a)) => {
                    let short = level.map_or(a.level + 1, |l| (a.level - l).max(1));
                    row["levels_short"] = serde_json::json!(short);
                    worst = worst.max(1);
                    "short"
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
                            Some(x) => Some(f.best(v, x)),
                        });
                    if let Some(b) = best.filter(|b| *b != 0.0) {
                        let times = f.ratio(need, b);
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
                    "past known"
                }
            };
            row["reading"] = serde_json::json!(reading);
            let mut factor = 1.0;
            if let Some(a) = by {
                // Levels above the builder (negative: below).
                let d = match (held, level) {
                    (Some(_), Some(l)) => a.level - l,
                    (Some(_), None) => 0,
                    _ => row["levels_short"].as_i64().unwrap_or(1),
                };
                let m = matured(tech, ladder, of, a, f, fid, d, fuel);
                factor = m.size_factor;
                row["figure_now"] = serde_json::json!(m.now);
                row["maturity"] = serde_json::json!(m.how);
                row["family"] = serde_json::json!(a.family);
                if let Some(o) = &m.floored_at {
                    row["floored_at"] = serde_json::json!(o);
                }
                row["advance"] = serde_json::json!(a.id);
                row["home_level"] = serde_json::json!(a.level);
                if reading == "short" {
                    let lv = if d == 1 {
                        "one level".to_string()
                    } else {
                        format!("{d} levels")
                    };
                    row["words"] = serde_json::json!(match &m.floored_at {
                        _ if m.how == "size" => format!(
                            "{lv} short: {} (a prototype {} times the size)",
                            a.name,
                            fmt_x(factor)
                        ),
                        Some(o) => format!(
                            "{lv} short: {}, no better yet than their {} ({} {})",
                            a.name,
                            o,
                            fmt_x(m.now),
                            f.unit
                        ),
                        None => format!(
                            "{lv} short: {}, first generation at {} {} ({} times the size)",
                            a.name,
                            fmt_x(m.now),
                            f.unit,
                            fmt_x(factor)
                        ),
                    });
                }
                let ink = if fiction.is_some() {
                    "fictional"
                } else {
                    a.mark.as_str()
                };
                leans.entry(a.id.clone()).or_insert_with(|| {
                    let mut c = serde_json::json!({"advance": a.id, "name": a.name, "ink": ink, "level": a.level, "pack": a.pack, "family": a.family});
                    if let Some(w) = &a.r#where {
                        c["where"] = w.clone();
                    }
                    c
                });
            }
            row["size_factor"] = serde_json::json!(factor);
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

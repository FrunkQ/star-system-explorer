//! K1 `sweep` (editing design 15): one field of one body swept across a
//! range on a what-if branch, as a cancellable job. Each step resolves the
//! system with the field at that value and reads the asked results, the
//! class label and the stability verdict about the body; where the label or
//! the verdict changes between two steps, the tipping point is narrowed by
//! halving. The branch is thrown away when the job ends, so nothing on the
//! main line moves.

use crate::protocol::{AuthoredEdit, JobSpec, Request, Response};
use crate::{err, json_to_value, Kernel};
use sse_core::cbor::Value;
use sse_core::math as m;
use sse_chronicle::UniverseTime;
use sse_core::Address;
use std::collections::BTreeMap;

/// At most this many grid steps, and halvings per tipping point.
pub const MAX_COUNT: u32 = 64;
pub const MAX_REFINE: u32 = 8;

pub(crate) struct Sweep {
    system: String,
    address: Address,
    field: String,
    values: Vec<f64>,
    results: Vec<String>,
    atlas: Option<(sse_sift::Atlas, String)>,
    at_s: Option<i64>,
    refine: u32,
    log: bool,
    branch: String,
    /// Grid samples in order, then the narrowing ones.
    grid: Vec<serde_json::Value>,
    tips: Vec<Tip>,
    /// Narrowing in hand: (index of the tip, low, high, halvings left).
    pending: Vec<(usize, f64, f64, u32)>,
    units: u32,
}

struct Tip {
    what: &'static str,
    from: serde_json::Value,
    to: serde_json::Value,
    low: f64,
    high: f64,
}

fn key(s: &serde_json::Value, what: &str) -> serde_json::Value {
    match what {
        "class" => s["class"].clone(),
        _ => s["verdict"].clone(),
    }
}

impl Sweep {
    pub(crate) fn start(k: &mut Kernel, job: JobSpec) -> Result<Sweep, Response> {
        let JobSpec::Sweep {
            system,
            address,
            field,
            from,
            to,
            count,
            log,
            results,
            packs,
            scheme,
            refine,
            at_s,
        } = job
        else {
            return Err(err("bad_job", "not a sweep"));
        };
        if !(2..=MAX_COUNT).contains(&count) {
            return Err(err(
                "bad_job",
                format!("a sweep takes 2 to {MAX_COUNT} steps"),
            ));
        }
        if !from.is_finite() || !to.is_finite() || from == to {
            return Err(err(
                "bad_job",
                "a sweep runs between two different finite values",
            ));
        }
        let log = log.unwrap_or(false);
        if log && (from <= 0.0 || to <= 0.0) {
            return Err(err(
                "bad_job",
                "a log sweep runs between two positive values",
            ));
        }
        let refine = refine.unwrap_or(4).min(MAX_REFINE);
        let atlas = match (packs, scheme) {
            (Some(ps), Some(sc)) => {
                let mut ms = Vec::new();
                for (i, p) in ps.iter().enumerate() {
                    ms.push(
                        serde_json::from_value::<sse_pack::PackManifest>(p.clone())
                            .map_err(|e| err("bad_pack", format!("pack {i}: {e}")))?,
                    );
                }
                let a = sse_sift::load(&ms)
                    .map_err(|es| err("bad_stack", es.join("; ")))?
                    .atlas;
                Some((a, sc))
            }
            (None, None) => None,
            _ => return Err(err("bad_job", "a class label needs both packs and scheme")),
        };
        let u = k.universe()?;
        let address = crate::in_universe(&u.root, &address)?;
        let sys = crate::in_universe(&u.root, &system)?;
        if !sys.contains(&address) {
            return Err(err("bad_address", format!("{address} is not in {sys}")));
        }
        let values = (0..count)
            .map(|i| {
                let t = i as f64 / (count - 1) as f64;
                if log {
                    m::pow(10.0, m::log10(from) + t * (m::log10(to) - m::log10(from)))
                } else {
                    from + t * (to - from)
                }
            })
            .collect();
        let branch = match k.dispatch(Request::Branch {
            label: format!("sweep {address} {field}"),
        })? {
            Response::BranchOpened { branch } => branch,
            other => return Err(other),
        };
        Ok(Sweep {
            system,
            address,
            field,
            values,
            results,
            atlas,
            at_s,
            refine,
            log,
            branch,
            grid: vec![],
            tips: vec![],
            pending: vec![],
            units: 0,
        })
    }

    /// Units known so far: the grid, plus the halvings of every tipping
    /// point found (the total grows as tipping points turn up).
    pub(crate) fn total(&self) -> u32 {
        self.values.len() as u32 + self.tips.len() as u32 * self.refine
    }
    pub(crate) fn units_done(&self) -> u32 {
        self.units
    }
    pub(crate) fn done(&self) -> bool {
        self.grid.len() == self.values.len() && self.pending.is_empty()
    }

    /// One resolve at `v`: the sample row.
    fn sample(&mut self, k: &mut Kernel, v: f64) -> Result<serde_json::Value, Response> {
        let occurred_at_s = self.at_s.unwrap_or(0);
        k.dispatch(Request::BranchSet {
            branch: self.branch.clone(),
            edit: AuthoredEdit {
                address: self.address.to_string(),
                field: self.field.clone(),
                value: serde_json::json!(v),
                occurred_at_s,
                impossible: false,
            },
        })?;
        self.units += 1;
        let (facts, surfaces) = match k.dispatch(Request::ResolvedSystem {
            system: self.system.clone(),
            at_s: self.at_s,
            tiles: None,
            branch: Some(self.branch.clone()),
        })? {
            Response::SystemResolved {
                facts, surfaces, ..
            } => (facts, surfaces),
            other => return Err(other),
        };
        let a = self.address.to_string();
        let f = &facts[&a];
        let surface = surfaces.iter().find(|s| s["address"] == a.as_str());
        let mut results = serde_json::Map::new();
        for r in &self.results {
            let got = f.get(r).cloned().or_else(|| {
                let mut x = surface?;
                for part in r.split('.') {
                    x = x.get(part)?;
                }
                Some(x.clone())
            });
            results.insert(r.clone(), got.unwrap_or(serde_json::Value::Null));
        }
        let class = self.atlas.as_ref().map(|(atlas, scheme)| {
            let mut row = BTreeMap::new();
            for (k2, v2) in f.as_object().into_iter().flatten() {
                if let Ok(v2) = json_to_value(v2) {
                    row.insert(k2.clone(), v2);
                }
            }
            let l = atlas.label(scheme, &sse_sift::Subject::new(row), None);
            serde_json::json!(l.base)
        });
        let verdict = self.verdict(k)?;
        Ok(serde_json::json!({
            "value": v,
            "results": results,
            "class": class,
            "verdict": verdict,
        }))
    }

    /// The stability verdict about the body on the branch.
    fn verdict(&self, k: &mut Kernel) -> Result<String, Response> {
        let u = k.universe()?;
        let id = *u
            .branches
            .get(&self.branch)
            .ok_or_else(|| err("unknown_branch", self.branch.clone()))?;
        let ch = u
            .chronicle
            .branch_as_main(&id)
            .map_err(|e| err("chronicle", format!("{e}")))?;
        let sys: Address = self
            .system
            .parse()
            .map_err(|e| err("bad_address", format!("{e}")))?;
        let mut st = ch.state_at(UniverseTime::seconds(self.at_s.unwrap_or(i64::MAX)));
        crate::withdraw::drop_withdrawn(&mut st.facts);
        let facts: BTreeMap<Address, BTreeMap<String, Value>> = st
            .facts
            .iter()
            .filter(|(x, _)| sys.contains(x))
            .map(|(x, r)| (x.clone(), r.clone()))
            .collect();
        let (checks, _) = crate::stability::checks(&facts, &sys);
        let focus = vec![self.address.to_string()];
        let checks = crate::stability::concerning(checks, &focus, &[]);
        let worst = checks.iter().map(|c| c.level).max().unwrap_or(0);
        Ok(crate::stability::verdict_word(worst).to_string())
    }

    pub(crate) fn step(&mut self, k: &mut Kernel, budget: u32) -> Result<(), Response> {
        let mut left = budget;
        while left > 0 && self.grid.len() < self.values.len() {
            let v = self.values[self.grid.len()];
            let s = self.sample(k, v)?;
            if let Some(prev) = self.grid.last() {
                for what in ["class", "verdict"] {
                    if key(prev, what) != key(&s, what) {
                        let low = prev["value"].as_f64().unwrap_or(v);
                        self.tips.push(Tip {
                            what,
                            from: key(prev, what),
                            to: key(&s, what),
                            low,
                            high: v,
                        });
                        if self.refine > 0 {
                            self.pending
                                .push((self.tips.len() - 1, low, v, self.refine));
                        }
                    }
                }
            }
            self.grid.push(s);
            left -= 1;
        }
        while left > 0 {
            let Some((i, low, high, n)) = self.pending.pop() else {
                break;
            };
            let mid = if self.log {
                m::sqrt(low * high)
            } else {
                0.5 * (low + high)
            };
            let s = self.sample(k, mid)?;
            let what = self.tips[i].what;
            let (nl, nh) = if key(&s, what) == self.tips[i].from {
                (mid, high)
            } else {
                (low, mid)
            };
            self.tips[i].low = nl;
            self.tips[i].high = nh;
            if n > 1 {
                self.pending.push((i, nl, nh, n - 1));
            }
            left -= 1;
        }
        Ok(())
    }

    /// Throw the branch away and give the result.
    pub(crate) fn finish(self, k: &mut Kernel) -> serde_json::Value {
        let _ = k.dispatch(Request::Discard {
            branch: self.branch.clone(),
        });
        serde_json::json!({
            "address": self.address.to_string(),
            "field": self.field,
            "steps": self.grid,
            "tipping_points": self.tips.iter().map(|t| serde_json::json!({
                "what": t.what,
                "from": t.from,
                "to": t.to,
                "between": [t.low, t.high],
            })).collect::<Vec<_>>(),
        })
    }
}

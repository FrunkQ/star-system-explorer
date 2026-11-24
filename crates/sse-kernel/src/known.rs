//! SV1 through the kernel: `known_to`, `set_known`, `undo_known` and
//! `known_in` (secrets and visibility design 3 to 7). Every answer is read off
//! the scene each audience would be sent, so what the GM is told a seat
//! knows is exactly what reaches that seat.

use crate::protocol::{Request, Response};
use crate::{err, latest_hash, Kernel};
use sse_chronicle::{Publisher, UniverseTime};
use sse_core::{Address, ContentHash};
use sse_observe::known::{exceptions, KnownChange, Landmark};
use sse_observe::scene::{Scene, GM};
use sse_observe::EVERYONE;
use std::collections::BTreeMap;

fn bad(code: &str, m: impl Into<String>) -> Response {
    err(code, m)
}

impl Kernel {
    fn known_setup(&self) -> Result<sse_observe::ObserveSetup, Response> {
        self.observe.setup.clone().ok_or_else(|| {
            bad(
                "no_data_views",
                "the pack stack defines no data views (provides.other.observe)",
            )
        })
    }

    fn known_address(&mut self, address: &str) -> Result<Address, Response> {
        let a: Address = address
            .parse()
            .map_err(|e| bad("bad_address", format!("{e}")))?;
        let u = self.universe()?;
        if !u.root.contains(&a) {
            return Err(bad(
                "bad_address",
                format!("{a} is not in universe {}", u.root),
            ));
        }
        Ok(a)
    }

    fn known_now(&mut self, at_s: Option<i64>) -> Result<i64, Response> {
        let u = self.universe()?;
        // With no time asked: the campaign's now, else the latest time any
        // event in the record speaks of (never the origin by default, which
        // would read a reveal made at the present as not yet made).
        Ok(at_s.unwrap_or_else(|| {
            crate::orbits::campaign_now(&u.chronicle, &u.root)
                .0
                .unwrap_or_else(|| {
                    u.chronicle
                        .events()
                        .iter()
                        .map(|e| e.body.occurred_at.s.max(e.body.activated_at.s))
                        .max()
                        .unwrap_or(0)
                })
        }))
    }

    /// The audiences a GM sets knowledge for: everyone (the table), then
    /// every other seat's audience, each with a label.
    fn known_audiences(&mut self) -> Result<Vec<(String, String)>, Response> {
        let reg = self.observe.intents.clone();
        let u = self.universe()?;
        let mut out = vec![(EVERYONE.to_string(), "The table".to_string())];
        for s in sse_trigger::intents::seats(&reg, &u.record) {
            if s.authority == sse_trigger::intents::Authority::Gm
                || s.audience == GM
                || out.iter().any(|(a, _)| *a == s.audience)
            {
                continue;
            }
            let label = if s.label.is_empty() {
                s.audience.clone()
            } else {
                s.label.clone()
            };
            out.push((s.audience.clone(), label));
        }
        Ok(out)
    }

    fn known_scene(
        &mut self,
        audience: &str,
        scope: &Address,
        at_s: i64,
    ) -> Result<Scene, Response> {
        match self.dispatch(Request::Scene {
            audience: audience.into(),
            scope: scope.to_string(),
            data_view: None,
            at_s,
            observe: None,
        })? {
            Response::Scene { scene, .. } => {
                Ok(serde_json::from_value(scene).expect("scenes round-trip"))
            }
            other => Err(other),
        }
    }

    pub(crate) fn known_to(
        &mut self,
        address: &str,
        field: Option<&str>,
        at_s: Option<i64>,
    ) -> Result<Response, Response> {
        let setup = self.known_setup()?;
        let a = self.known_address(address)?;
        let at_s = self.known_now(at_s)?;
        let marks = setup.landmarks();
        let views = setup.views_by_id();
        let gm = self.known_scene(GM, &a, at_s)?;
        let all: Vec<String> = gm
            .subjects
            .iter()
            .find(|s| s.address == a)
            .map(|s| s.facts.iter().map(|f| f.field.clone()).collect())
            .unwrap_or_default();
        let mut rows = Vec::new();
        for (aud, label) in self.known_audiences()? {
            let scene = self.known_scene(&aud, &a, at_s)?;
            let sub = scene.subjects.iter().find(|s| s.address == a);
            // The lines this audience is not sent a value for (pencil on the card).
            let withheld: Vec<&String> = all
                .iter()
                .filter(|f| {
                    !sub.is_some_and(|s| {
                        s.facts
                            .iter()
                            .any(|x| &x.field == *f && x.placeholder.is_none())
                    })
                })
                .collect();
            let u = self.universe()?;
            let state = u.chronicle.state_at(UniverseTime::seconds(at_s));
            let rung = match sub {
                Some(s) => s.view.clone(),
                None => sse_observe::effective_view(&state, &a, &aud, &setup.campaign),
            };
            let v = views.get(&rung);
            let thing = match (sub, v) {
                (None, _) => Landmark::GmOnly,
                (Some(_), Some(v)) => marks.of(v),
                (Some(_), None) => Landmark::HeardOf,
            };
            let ex = exceptions(&state, &a, &aud);
            let landmark = match field {
                None => thing,
                Some(f) => match (ex.get(f), sub) {
                    (Some(l), _) if thing != Landmark::GmOnly => *l,
                    (_, None) => Landmark::GmOnly,
                    (_, Some(s)) => match s.facts.iter().find(|x| x.field == f) {
                        Some(x) if x.placeholder.is_none() => Landmark::Known,
                        Some(_) => Landmark::HeardOf,
                        None => Landmark::GmOnly,
                    },
                },
            };
            // A change for this audience here that has not reached them yet.
            let pending = u
                .chronicle
                .events()
                .iter()
                .filter(|e| {
                    e.body.kind == sse_chronicle::kinds::OBSERVE_GRANT
                        && e.body.activated_at > UniverseTime::seconds(at_s)
                        && e.body.subject.contains(&a)
                        && matches!(&e.body.payload.data, Some(sse_core::cbor::Value::Map(m))
                            if m.get("audience") == Some(&sse_core::cbor::Value::Text(aud.clone())))
                })
                .map(|e| e.body.activated_at.s)
                .min();
            let mut row = serde_json::json!({
                "audience": aud,
                "label": label,
                "rung": rung,
                "rung_label": v.map_or(rung.as_str(), |v| v.label.as_str()),
                "landmark": landmark.id(),
                "top": rung == marks.known,
                "exceptions": ex.iter().map(|(f, l)| serde_json::json!({"field": f, "landmark": l.id()})).collect::<Vec<_>>(),
                "cover": [],
                "withheld": withheld,
            });
            if let Some(p) = pending {
                row["from_s"] = p.into();
            }
            rows.push(row);
        }
        let mut value =
            serde_json::json!({"address": a.to_string(), "at_s": at_s, "audiences": rows});
        if let Some(f) = field {
            value["field"] = f.into();
        }
        Ok(Response::KnownTo { value })
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn set_known(
        &mut self,
        address: &str,
        audience: &str,
        landmark: Option<&str>,
        field: Option<&str>,
        scope: Option<&serde_json::Value>,
        from_s: Option<i64>,
        at_s: i64,
    ) -> Result<Response, Response> {
        let setup = self.known_setup()?;
        if audience == GM || audience.is_empty() || audience.contains('.') {
            return Err(bad(
                "bad_audience",
                "set what a player audience knows (not the GM; no dots in the id)",
            ));
        }
        let to = match landmark {
            None => None,
            Some(l) => Some(Landmark::parse(l).ok_or_else(|| {
                bad(
                    "bad_landmark",
                    format!("{l:?}: a landmark is gm_only, heard_of or known"),
                )
            })?),
        };
        if let Some(f) = field {
            if f.is_empty() || setup.disclosure.is_internal(f) {
                return Err(bad(
                    "bad_field",
                    format!("{f:?} is not a fact a seat can be shown"),
                ));
            }
        }
        let a = self.known_address(address)?;
        let mut targets = vec![a.clone()];
        match scope {
            None | Some(serde_json::Value::Null) => {}
            Some(serde_json::Value::String(s)) if s == "this" => {}
            Some(serde_json::Value::String(s)) if s == "children" => {
                // Everything under it that has its own setting for this
                // audience; the rest inherit the thing's own.
                let key = match field {
                    Some(f) => sse_observe::known::except_field(audience, f),
                    None if audience == EVERYONE => "observe.view".to_string(),
                    None => format!("observe.view.{audience}"),
                };
                let u = self.universe()?;
                let state = u
                    .chronicle
                    .state_at(UniverseTime::seconds(from_s.unwrap_or(at_s)));
                targets.extend(
                    state
                        .facts
                        .iter()
                        .filter(|(x, f)| {
                            **x != a && a.contains(x) && (field.is_some() || f.contains_key(&key))
                        })
                        .filter(|(_, f)| {
                            field.is_none_or(|fd| f.contains_key(fd) || f.contains_key(&key))
                        })
                        .map(|(x, _)| x.clone()),
                );
            }
            Some(v) => match v.get("list").and_then(|l| l.as_array()) {
                Some(l) => {
                    for x in l {
                        let s = x
                            .as_str()
                            .ok_or_else(|| bad("bad_scope", "a list holds addresses"))?;
                        let x = self.known_address(s)?;
                        if !targets.contains(&x) {
                            targets.push(x);
                        }
                    }
                }
                None => {
                    return Err(bad(
                        "bad_scope",
                        "scope is \"this\", \"children\" or {\"list\": [addresses]}",
                    ))
                }
            },
        }
        let changes: Vec<KnownChange> = targets
            .into_iter()
            .map(|address| KnownChange {
                address,
                field: field.map(str::to_string),
                to,
            })
            .collect();
        let marks = setup.landmarks();
        let views = setup.views_by_id();
        let u = self.universe()?;
        let id = sse_observe::set_known(
            &mut u.chronicle,
            &changes,
            audience,
            &marks,
            &views,
            &setup.campaign,
            UniverseTime::seconds(at_s),
            UniverseTime::seconds(from_s.unwrap_or(at_s)),
            &Publisher::new("gm", "set_known", "1"),
        )
        .map_err(|e| bad("chronicle", format!("{e}")))?;
        Ok(self.known_set(id))
    }

    fn known_set(&mut self, id: Option<ContentHash>) -> Response {
        let u = self.universe.as_ref().expect("checked");
        let changed = id
            .and_then(|i| u.chronicle.events().iter().find(|e| e.id == i))
            .map_or(0, |e| e.body.subject.len() as u32);
        Response::KnownSet {
            undo: id.map(|i| i.hex()),
            changed,
            state_hash: latest_hash(&u.chronicle),
        }
    }

    pub(crate) fn undo_known(&mut self, undo: &str, at_s: i64) -> Result<Response, Response> {
        let u = self.universe()?;
        let id = u
            .chronicle
            .events()
            .iter()
            .find(|e| e.id.hex() == undo)
            .map(|e| e.id)
            .ok_or_else(|| bad("bad_undo", format!("no set_known change {undo}")))?;
        let id = sse_observe::undo_known(
            &mut u.chronicle,
            &id,
            UniverseTime::seconds(at_s),
            &Publisher::new("gm", "undo_known", "1"),
        )
        .map_err(|e| match e {
            sse_observe::known::UndoError::NoSuchChange => {
                bad("bad_undo", format!("no set_known change {undo}"))
            }
            sse_observe::known::UndoError::Chronicle(m) => bad("chronicle", m),
        })?;
        Ok(self.known_set(Some(id)))
    }

    pub(crate) fn known_in(
        &mut self,
        bounds: &str,
        at_s: Option<i64>,
    ) -> Result<Response, Response> {
        let setup = self.known_setup()?;
        let b = self.known_address(bounds)?;
        let at_s = self.known_now(at_s)?;
        let marks = setup.landmarks();
        let views = setup.views_by_id();
        let all = self.known_scene(GM, &b, at_s)?;
        let mut rows = Vec::new();
        for (aud, label) in self.known_audiences()? {
            let scene = self.known_scene(&aud, &b, at_s)?;
            let seen: BTreeMap<&Address, &str> = scene
                .subjects
                .iter()
                .map(|s| (&s.address, s.view.as_str()))
                .collect();
            let (mut k, mut h, mut g) = (0u32, 0u32, 0u32);
            for s in &all.subjects {
                match seen
                    .get(&s.address)
                    .and_then(|v| views.get(*v))
                    .map(|v| marks.of(v))
                {
                    Some(Landmark::Known) => k += 1,
                    Some(Landmark::HeardOf) => h += 1,
                    _ => g += 1,
                }
            }
            rows.push(serde_json::json!({"audience": aud, "label": label, "known": k, "heard_of": h, "gm_only": g}));
        }
        Ok(Response::KnownCounted {
            bounds: b.to_string(),
            audiences: rows,
        })
    }
}

//! NM2: names on real addresses (naming design 4.2 to 4.5).
//!
//! Names live in the chronicle as ordinary facts, so they are dated,
//! replayed and disclosed like any other fact:
//!
//! - `naming.<layer>` (or `naming.<layer>.<kind>`) on an address: the
//!   scheme that names things here and beneath, an authored setting.
//!   Settings inherit down the naming tree (a moon hangs from its planet,
//!   the planet from its star, the star from its system); the stack's
//!   `defaults` stand above the universe.
//! - `name.<layer>`: a typed name, plain text that always wins and never
//!   follows. The older `name` field is the typed common name.
//! - `names.fixed.<layer>`: a drawn name made permanent (the stored form
//!   and its parts), written when the object is promoted or `fix_names`
//!   is asked. A fixed name keeps its form, so it still follows its parent.
//!
//! Everything else is drawn when read: a keyed draw, the same on every
//! machine, settled among siblings in address order. A scene carries each
//! subject's shown names as `name.<layer>` facts, so a seat sees exactly
//! the layers its data view discloses. `naming.*` and `names.fixed.*` are
//! bookkeeping and never reach a scene.

use sse_chronicle::State;
use sse_core::cbor::Value;
use sse_core::{Address, Seed};
use sse_names::{Library, Named, Request};
use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};

pub(crate) const SETTING: &str = "naming.";
pub(crate) const FIXED: &str = "names.fixed.";
pub(crate) const TYPED: &str = "name.";
/// The field a typed common name had before layers.
pub(crate) const LEGACY: &str = "name";
pub(crate) const COMMON: &str = "common";
pub(crate) const DESIGNATION: &str = "designation";

/// A name as shown, with where it came from.
#[derive(Clone, Debug, serde::Serialize)]
pub(crate) struct Shown {
    pub layer: String,
    pub text: String,
    /// `typed`, `fixed` or `drawn`.
    pub source: &'static str,
    /// The scheme in force here for this layer (none for a typed name with
    /// no scheme above it).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scheme: Option<String>,
    /// The drawn or fixed name with its parts and faces.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub named: Option<Named>,
    /// Under a typed name: the name it would have otherwise (restore).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub original: Option<String>,
}

fn text_of(v: &Value) -> Option<&str> {
    match v {
        Value::Text(t) if !t.is_empty() => Some(t),
        _ => None,
    }
}

fn word(v: &Value) -> Option<String> {
    match v {
        Value::Text(t) => Some(t.clone()),
        Value::Int(i) => Some(i.to_string()),
        Value::Float(f) => Some(format!("{f}")),
        Value::Bool(b) => Some(b.to_string()),
        _ => None,
    }
}

/// Whether any name scheme is in play: the stack sets a default, or some
/// address carries a setting or a fixed name. Without one, scenes are
/// left exactly as they were before names.
pub(crate) fn in_play(state: &State, lib: &Library) -> bool {
    !lib.defaults.is_empty()
        || state.facts.values().any(|f| {
            f.keys()
                .any(|k| k.starts_with(SETTING) || k.starts_with(FIXED))
        })
}

type Group = Vec<(Address, Result<Named, String>)>;

pub(crate) struct Names<'a> {
    state: &'a State,
    lib: &'a Library,
    seed: &'a Seed,
    root: &'a Address,
    /// Every layer named anywhere: defaults, settings, typed and fixed.
    pub layers: BTreeSet<String>,
    children: BTreeMap<(Address, String), Vec<Address>>,
    memo: RefCell<BTreeMap<(Address, String), Option<Shown>>>,
    groups: RefCell<BTreeMap<(Address, String, String), std::rc::Rc<Group>>>,
    busy: RefCell<BTreeSet<(Address, String)>>,
}

impl<'a> Names<'a> {
    pub fn new(state: &'a State, lib: &'a Library, seed: &'a Seed, root: &'a Address) -> Self {
        let mut layers: BTreeSet<String> = lib.defaults.keys().cloned().collect();
        let mut children: BTreeMap<(Address, String), Vec<Address>> = BTreeMap::new();
        for (a, f) in &state.facts {
            for k in f.keys() {
                if k == LEGACY {
                    layers.insert(COMMON.into());
                } else if let Some(l) = k
                    .strip_prefix(SETTING)
                    .or_else(|| k.strip_prefix(FIXED))
                    .or_else(|| k.strip_prefix(TYPED))
                {
                    let l = l.split('.').next().unwrap_or(l);
                    if sse_names::scheme::layer_ok(l) {
                        layers.insert(l.to_string());
                    }
                }
            }
            if let Some(p) = naming_parent(state, root, a) {
                children
                    .entry((p, kind_of(state, a)))
                    .or_default()
                    .push(a.clone());
            }
        }
        Names {
            state,
            lib,
            seed,
            root,
            layers,
            children,
            memo: RefCell::new(BTreeMap::new()),
            groups: RefCell::new(BTreeMap::new()),
            busy: RefCell::new(BTreeSet::new()),
        }
    }

    fn fact(&self, a: &Address, f: &str) -> Option<&'a Value> {
        self.state.facts.get(a).and_then(|m| m.get(f))
    }

    /// The scheme naming `layer` for a thing of `kind` at `a`: the nearest
    /// setting at or above it, a per-kind setting before a plain one, then
    /// the stack's default. An empty setting means "no names here".
    pub fn scheme_for(&self, a: &Address, layer: &str, kind: &str) -> Option<String> {
        for x in up(self.state, self.root, a) {
            for f in [
                format!("{SETTING}{layer}.{kind}"),
                format!("{SETTING}{layer}"),
            ] {
                match self.fact(&x, &f) {
                    Some(Value::Text(s)) if s.is_empty() => return None,
                    Some(Value::Text(s)) => return Some(s.clone()),
                    _ => {}
                }
            }
        }
        self.lib.defaults.get(layer).cloned()
    }

    fn typed(&self, a: &Address, layer: &str) -> Option<String> {
        self.fact(a, &format!("{TYPED}{layer}"))
            .and_then(text_of)
            .or_else(|| {
                (layer == COMMON)
                    .then(|| self.fact(a, LEGACY).and_then(text_of))
                    .flatten()
            })
            .map(str::to_string)
    }

    fn fixed(&self, a: &Address, layer: &str) -> Option<Named> {
        self.fact(a, &format!("{FIXED}{layer}"))
            .and_then(text_of)
            .and_then(|t| serde_json::from_str(t).ok())
    }

    fn words(&self, a: &Address) -> BTreeMap<String, String> {
        let mut out = BTreeMap::new();
        for (k, v) in self.state.facts.get(a).into_iter().flatten() {
            if k.starts_with(SETTING) || k.starts_with(FIXED) || k.starts_with(TYPED) {
                continue;
            }
            if let Some(w) = word(v) {
                out.insert(k.clone(), w);
            }
        }
        out
    }

    /// The parent's name in this layer, else its common name, else its
    /// designation (design 4.5).
    fn parent_text(&self, a: &Address, layer: &str) -> Option<String> {
        let p = naming_parent(self.state, self.root, a)?;
        [layer, COMMON, DESIGNATION]
            .iter()
            .find_map(|l| self.shown(&p, l))
            .map(|s| s.text)
    }

    fn system_text(&self, a: &Address, layer: &str) -> Option<String> {
        let mut at = a.parent();
        while let Some(x) = at {
            if x.last().kind() == "sys" {
                return [layer, COMMON, DESIGNATION]
                    .iter()
                    .find_map(|l| self.shown(&x, l))
                    .map(|s| s.text);
            }
            at = x.parent();
        }
        None
    }

    /// Draw the whole sibling group of `a` (same naming parent, same kind)
    /// in this layer, each with the scheme in force at it. Every sibling is
    /// drawn whether or not it is typed or fixed, so fixing one never moves
    /// another.
    fn group(&self, a: &Address, layer: &str) -> std::rc::Rc<Group> {
        let kind = kind_of(self.state, a);
        let p = naming_parent(self.state, self.root, a).unwrap_or_else(|| self.root.clone());
        let key = (p.clone(), kind.clone(), layer.to_string());
        if let Some(g) = self.groups.borrow().get(&key) {
            return g.clone();
        }
        let sibs: Vec<Address> = self
            .children
            .get(&(p, kind.clone()))
            .cloned()
            .unwrap_or_else(|| vec![a.clone()]);
        let schemes: Vec<Option<String>> = sibs
            .iter()
            .map(|s| self.scheme_for(s, layer, &kind))
            .collect();
        let parent = self.parent_text(a, layer);
        let system = self.system_text(a, layer);
        let words: Vec<BTreeMap<String, String>> = sibs.iter().map(|s| self.words(s)).collect();
        let mut idx = vec![];
        let mut reqs = vec![];
        for (i, s) in sibs.iter().enumerate() {
            if let Some(sc) = &schemes[i] {
                idx.push(i);
                reqs.push(Request {
                    seed: self.seed,
                    address: s,
                    layer,
                    scheme: sc,
                    kind: &kind,
                    index: i as u32,
                    serial: serial_of(s),
                    parent: parent.as_deref(),
                    system: system.as_deref(),
                    facts: &words[i],
                });
            }
        }
        let made = sse_names::siblings(self.lib, &reqs);
        let g: Group = idx
            .into_iter()
            .zip(made)
            .map(|(i, r)| (sibs[i].clone(), r))
            .collect();
        let g = std::rc::Rc::new(g);
        self.groups.borrow_mut().insert(key, g.clone());
        g
    }

    /// The drawn name of `a` in `layer`, whatever is typed or fixed.
    pub fn drawn(&self, a: &Address, layer: &str) -> Option<Result<Named, String>> {
        self.group(a, layer)
            .iter()
            .find(|(x, _)| x == a)
            .map(|(_, r)| r.clone())
    }

    /// The name shown at `a` in `layer`, or none.
    pub fn shown(&self, a: &Address, layer: &str) -> Option<Shown> {
        let key = (a.clone(), layer.to_string());
        if let Some(s) = self.memo.borrow().get(&key) {
            return s.clone();
        }
        // A cycle of parent facts names nothing rather than looping.
        if !self.busy.borrow_mut().insert(key.clone()) {
            return None;
        }
        let out = self.work(a, layer);
        self.busy.borrow_mut().remove(&key);
        self.memo.borrow_mut().insert(key, out.clone());
        out
    }

    fn work(&self, a: &Address, layer: &str) -> Option<Shown> {
        let kind = kind_of(self.state, a);
        let scheme = self.scheme_for(a, layer, &kind);
        let made = match self.fixed(a, layer) {
            Some(n) => Some((n, "fixed")),
            None if scheme.is_some() => self
                .drawn(a, layer)
                .and_then(Result::ok)
                .map(|n| (n, "drawn")),
            None => None,
        };
        let made = made.map(|(n, src)| {
            let t = n.show(
                self.parent_text(a, layer).as_deref(),
                self.system_text(a, layer).as_deref(),
            );
            (n, src, t)
        });
        if let Some(t) = self.typed(a, layer) {
            return Some(Shown {
                layer: layer.into(),
                text: t,
                source: "typed",
                scheme,
                original: made.as_ref().map(|m| m.2.clone()),
                named: None,
            });
        }
        let (n, src, t) = made?;
        (!t.is_empty()).then(|| Shown {
            layer: layer.into(),
            text: t,
            source: src,
            scheme,
            named: Some(n),
            original: None,
        })
    }

    /// Every layer's shown name at `a`.
    pub fn all(&self, a: &Address) -> Vec<Shown> {
        self.layers
            .iter()
            .filter_map(|l| self.shown(a, l))
            .collect()
    }
}

/// The thing a name follows: the `parent` fact when it names an address in
/// the universe (a moon's planet), else the address above.
pub(crate) fn naming_parent(state: &State, root: &Address, a: &Address) -> Option<Address> {
    let fact = state
        .facts
        .get(a)
        .and_then(|f| f.get("parent"))
        .and_then(text_of)
        .and_then(|t| t.parse::<Address>().ok())
        .filter(|p| root.contains(p) && p != a);
    fact.or_else(|| a.parent())
}

/// `a` and the things it hangs from, nearest first, by naming parents (a
/// moon, its planet, the star, the system, ...), at most 64 steps so a
/// loop of `parent` facts ends.
pub(crate) fn up(state: &State, root: &Address, a: &Address) -> Vec<Address> {
    let mut out = vec![a.clone()];
    while out.len() < 64 {
        match naming_parent(state, root, out.last().expect("not empty")) {
            Some(p) if !out.contains(&p) => out.push(p),
            _ => break,
        }
    }
    out
}

/// Whether `a` is `scope` or hangs beneath it in the naming tree.
pub(crate) fn beneath(state: &State, root: &Address, scope: &Address, a: &Address) -> bool {
    scope.contains(a) || up(state, root, a).iter().any(|x| x == scope)
}

/// What kind of thing `a` is, for the scheme's forms: `body.kind`, else
/// `kind`, else the address segment's kind.
pub(crate) fn kind_of(state: &State, a: &Address) -> String {
    let f = state.facts.get(a);
    f.and_then(|f| f.get("body.kind"))
        .or_else(|| f.and_then(|f| f.get("kind")))
        .and_then(text_of)
        .map(str::to_string)
        .unwrap_or_else(|| a.last().kind().to_string())
}

/// The catalogue serial for `a`: the number of the system it is in.
fn serial_of(a: &Address) -> Option<u64> {
    a.segments()
        .iter()
        .rev()
        .find(|s| s.kind() == "sys")
        .and_then(|s| s.key().parse().ok())
}

/// Put every subject's shown names into the facts as `name.<layer>`, and
/// take the bookkeeping fields out (scenes, `names` for a seat).
pub(crate) fn inject(state: &mut State, lib: &Library, seed: &Seed, root: &Address) {
    if !in_play(state, lib) {
        return;
    }
    let shown: Vec<(Address, Vec<Shown>)> = {
        let n = Names::new(state, lib, seed, root);
        state.facts.keys().map(|a| (a.clone(), n.all(a))).collect()
    };
    for (a, names) in shown {
        let f = state.facts.entry(a).or_default();
        f.retain(|k, _| !k.starts_with(SETTING) && !k.starts_with(FIXED));
        for s in names {
            f.insert(format!("{TYPED}{}", s.layer), Value::Text(s.text));
        }
    }
}

use crate::protocol::Response;
use crate::{err, in_universe, latest_hash, withdraw};
use sse_chronicle::{kinds, Event, EventBody, Evidence, Payload, Publisher, UniverseTime};

/// One dated event setting and unsetting facts on several addresses.
fn event(
    u: &crate::Universe,
    set: BTreeMap<Address, BTreeMap<String, Value>>,
    unset: BTreeMap<Address, Vec<String>>,
    at_s: i64,
    evidence: Evidence,
    kind: &str,
    publisher: Publisher,
) -> Result<Event, Response> {
    let t = UniverseTime::seconds(at_s);
    let subject: BTreeSet<Address> = set.keys().chain(unset.keys()).cloned().collect();
    Event::new(EventBody {
        kind: kind.into(),
        subject: subject.into_iter().collect(),
        occurred_at: t,
        activated_at: t,
        recognised_at: u.chronicle.next_stamp(),
        causes: vec![],
        payload: Payload {
            set,
            unset,
            ..Default::default()
        },
        evidence,
        publisher,
    })
    .map_err(|e| err("bad_event", format!("{e}")))
}

fn layer_checked(l: &str) -> Result<(), Response> {
    if sse_names::scheme::layer_ok(l) {
        Ok(())
    } else {
        Err(err(
            "bad_layer",
            format!("a name layer is lower-case letters, digits and _, not {l:?}"),
        ))
    }
}

/// Where a campaign clash is checked when a name is fixed (design 4.4):
/// the system the address is in, else the universe.
fn clash_scope(root: &Address, a: &Address) -> Address {
    let mut at = Some(a.clone());
    while let Some(x) = at {
        if x.last().kind() == "sys" {
            return x;
        }
        at = x.parent();
    }
    root.clone()
}

/// The fixed names a fix or a promotion adds under `scope`: every drawn
/// name not yet typed or fixed, as its stored record. A name that clashes
/// (folded) with one already fixed or typed in its system, or fixed
/// earlier in this same call, moves to its next draw.
pub(crate) fn fixes(
    state: &State,
    lib: &Library,
    seed: &Seed,
    root: &Address,
    scope: &Address,
) -> BTreeMap<Address, BTreeMap<String, Value>> {
    let n = Names::new(state, lib, seed, root);
    let mut out: BTreeMap<Address, BTreeMap<String, Value>> = BTreeMap::new();
    // Names already fixed or typed, by (system, layer).
    let mut taken: BTreeMap<(Address, String), BTreeSet<String>> = BTreeMap::new();
    for a in state.facts.keys() {
        for l in &n.layers {
            if let Some(s) = n.shown(a, l).filter(|s| s.source != "drawn") {
                taken
                    .entry((clash_scope(root, a), l.clone()))
                    .or_default()
                    .insert(sse_names::text::fold(&s.text));
            }
        }
    }
    for a in state
        .facts
        .keys()
        .filter(|a| beneath(state, root, scope, a))
    {
        for l in &n.layers {
            let Some(s) = n.shown(a, l).filter(|s| s.source == "drawn") else {
                continue;
            };
            let Some(mut named) = s.named.clone() else {
                continue;
            };
            let key = (clash_scope(root, a), l.clone());
            let have = taken.entry(key).or_default();
            if !named.numbered && have.contains(&sse_names::text::fold(&s.text)) {
                let kind = kind_of(state, a);
                let words = n.words(a);
                let parent = n.parent_text(a, l);
                let system = n.system_text(a, l);
                let sc = s.scheme.clone().unwrap_or_default();
                let idx = naming_parent(state, root, a)
                    .and_then(|p| n.children.get(&(p, kind.clone())))
                    .and_then(|v| v.iter().position(|x| x == a))
                    .unwrap_or(0);
                let r = Request {
                    seed,
                    address: a,
                    layer: l,
                    scheme: &sc,
                    kind: &kind,
                    index: idx as u32,
                    serial: serial_of(a),
                    parent: parent.as_deref(),
                    system: system.as_deref(),
                    facts: &words,
                };
                match sse_names::make::name_with(lib, &r, &mut |f, _| have.contains(f)) {
                    Ok(m) => named = m,
                    // Out of draws: the object keeps only its other layers.
                    Err(_) => continue,
                }
            }
            let shown = named.show(
                n.parent_text(a, l).as_deref(),
                n.system_text(a, l).as_deref(),
            );
            have.insert(sse_names::text::fold(&shown));
            out.entry(a.clone()).or_default().insert(
                format!("{FIXED}{l}"),
                Value::Text(serde_json::to_string(&named).expect("plain data")),
            );
        }
    }
    out
}

impl crate::Kernel {
    /// `read_names` (NM2).
    pub(crate) fn read_names(
        &mut self,
        address: &str,
        at_s: Option<i64>,
        audience: Option<&str>,
    ) -> Result<Response, Response> {
        let at = at_s.unwrap_or(i64::MAX);
        let lib = self.observe.names.clone();
        let u = self.universe()?;
        let a = in_universe(&u.root, address)?;
        let mut state = u.chronicle.state_at(UniverseTime::seconds(at));
        withdraw::drop_withdrawn(&mut state.facts);
        let (seed, root) = (u.seed, u.root.clone());
        let n = Names::new(&state, &lib, &seed, &root);
        let mut all = n.all(&a);
        let kind = kind_of(&state, &a);
        if let Some(aud) = audience.filter(|x| *x != sse_observe::scene::GM) {
            let scene = match self.dispatch(crate::protocol::Request::Scene {
                audience: aud.to_string(),
                scope: a.to_string(),
                data_view: None,
                at_s: at,
                observe: None,
            })? {
                Response::Scene { scene, .. } => scene,
                other => return Err(other),
            };
            let scene: sse_observe::scene::Scene =
                serde_json::from_value(scene).expect("scenes round-trip");
            let seen: BTreeSet<String> = scene
                .subjects
                .iter()
                .filter(|s| s.address == a)
                .flat_map(|s| s.facts.iter())
                .filter(|f| f.value.is_some() && f.placeholder.is_none())
                .filter_map(|f| f.field.strip_prefix(TYPED).map(str::to_string))
                .collect();
            all.retain(|s| seen.contains(&s.layer));
            // A seat learns the text, not how it was made.
            for s in &mut all {
                s.original = None;
            }
        }
        Ok(Response::Names {
            value: serde_json::json!({
                "address": a.to_string(),
                "kind": kind,
                "names": all,
            }),
        })
    }

    /// `set_naming` (NM2).
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn set_naming(
        &mut self,
        address: &str,
        layer: &str,
        scheme: Option<&str>,
        kind: Option<&str>,
        at_s: i64,
        rename_fixed: bool,
    ) -> Result<Response, Response> {
        layer_checked(layer)?;
        let lib = self.observe.names.clone();
        if let Some(s) = scheme.filter(|s| !s.is_empty()) {
            if !lib.schemes.contains_key(s) {
                return Err(err(
                    "unknown_scheme",
                    format!("no name scheme {s:?} is stacked"),
                ));
            }
        }
        if let Some(k) = kind {
            if k.is_empty() || k.contains('.') || k.len() > 64 {
                return Err(err("bad_kind", format!("{k:?} is not a kind of thing")));
            }
        }
        let u = self.universe()?;
        let a = in_universe(&u.root, address)?;
        let field = match kind {
            Some(k) => format!("{SETTING}{layer}.{k}"),
            None => format!("{SETTING}{layer}"),
        };
        let mut set: BTreeMap<Address, BTreeMap<String, Value>> = BTreeMap::new();
        let mut unset: BTreeMap<Address, Vec<String>> = BTreeMap::new();
        match scheme {
            Some(s) => {
                set.entry(a.clone())
                    .or_default()
                    .insert(field.clone(), Value::Text(s.to_string()));
            }
            None => unset.entry(a.clone()).or_default().push(field.clone()),
        }
        if rename_fixed {
            // The state as it will be, with the new setting in force.
            let mut after = u.chronicle.state_at(UniverseTime::seconds(at_s));
            let f = after.facts.entry(a.clone()).or_default();
            match scheme {
                Some(s) => {
                    f.insert(field.clone(), Value::Text(s.to_string()));
                }
                None => {
                    f.remove(&field);
                }
            }
            let fixed_field = format!("{FIXED}{layer}");
            let (seed, root) = (u.seed, u.root.clone());
            let names = Names::new(&after, &lib, &seed, &root);
            let mut moved = vec![];
            for (x, facts) in &after.facts {
                if !beneath(&after, &root, &a, x) {
                    continue;
                }
                let Some(old) = facts
                    .get(&fixed_field)
                    .and_then(text_of)
                    .and_then(|t| serde_json::from_str::<Named>(t).ok())
                else {
                    continue;
                };
                let now = names.scheme_for(x, layer, &kind_of(&after, x));
                if now.as_deref() == Some(old.scheme.as_str()) {
                    continue;
                }
                moved.push((x.clone(), now.is_some()));
            }
            // Drop the old records, then draw afresh as the new scheme says.
            for (x, _) in &moved {
                after.facts.get_mut(x).map(|f| f.remove(&fixed_field));
            }
            let fresh = fixes(&after, &lib, &seed, &root, &a);
            for (x, has_scheme) in moved {
                match fresh.get(&x).and_then(|f| f.get(&fixed_field)) {
                    Some(v) if has_scheme => {
                        set.entry(x)
                            .or_default()
                            .insert(fixed_field.clone(), v.clone());
                    }
                    _ => unset.entry(x).or_default().push(fixed_field.clone()),
                }
            }
        }
        let e = event(
            u,
            set,
            unset,
            at_s,
            Evidence::Authored,
            kinds::AUTHORED_SET,
            Publisher::new("gm", "names", "1"),
        )?;
        let affected = e.body.subject.iter().map(|a| a.to_string()).collect();
        let id = u
            .chronicle
            .append(e)
            .map_err(|e| err("chronicle", format!("{e}")))?;
        Ok(Response::Authored {
            event_id: id.hex(),
            affected,
            state_hash: latest_hash(&u.chronicle),
            declared_tag: None,
        })
    }

    /// `rename` (NM2).
    pub(crate) fn rename(
        &mut self,
        address: &str,
        layer: &str,
        text: Option<&str>,
        at_s: i64,
    ) -> Result<Response, Response> {
        layer_checked(layer)?;
        let u = self.universe()?;
        let a = in_universe(&u.root, address)?;
        let field = format!("{TYPED}{layer}");
        let mut set: BTreeMap<Address, BTreeMap<String, Value>> = BTreeMap::new();
        let mut unset: BTreeMap<Address, Vec<String>> = BTreeMap::new();
        match text {
            Some(t) => {
                let t = sse_names::text::compose(t);
                sse_names::text::check(&t).map_err(|m| err("bad_name", m))?;
                set.entry(a.clone())
                    .or_default()
                    .insert(field, Value::Text(t));
            }
            None => {
                let mut f = vec![field];
                if layer == COMMON {
                    f.push(LEGACY.to_string());
                }
                unset.insert(a.clone(), f);
            }
        }
        let e = event(
            u,
            set,
            unset,
            at_s,
            Evidence::Authored,
            kinds::AUTHORED_SET,
            Publisher::new("gm", "names", "1"),
        )?;
        let id = u
            .chronicle
            .append(e)
            .map_err(|e| err("chronicle", format!("{e}")))?;
        Ok(Response::Authored {
            event_id: id.hex(),
            affected: vec![a.to_string()],
            state_hash: latest_hash(&u.chronicle),
            declared_tag: None,
        })
    }

    /// `fix_names` (NM2).
    pub(crate) fn fix_names(&mut self, address: &str, at_s: i64) -> Result<Response, Response> {
        let lib = self.observe.names.clone();
        let u = self.universe()?;
        let a = in_universe(&u.root, address)?;
        let mut state = u.chronicle.state_at(UniverseTime::seconds(at_s));
        withdraw::drop_withdrawn(&mut state.facts);
        let set = fixes(&state, &lib, &u.seed, &u.root, &a);
        if set.is_empty() {
            return Err(err(
                "nothing_to_fix",
                format!("{a}: no drawn name at or under it is waiting to be fixed"),
            ));
        }
        let e = event(
            u,
            set,
            BTreeMap::new(),
            at_s,
            Evidence::Resolved,
            "names.fixed",
            Publisher::new("sse.names", "fix", "1"),
        )?;
        let affected = e.body.subject.iter().map(|a| a.to_string()).collect();
        let id = u
            .chronicle
            .append(e)
            .map_err(|e| err("chronicle", format!("{e}")))?;
        Ok(Response::Authored {
            event_id: id.hex(),
            affected,
            state_hash: latest_hash(&u.chronicle),
            declared_tag: None,
        })
    }
}

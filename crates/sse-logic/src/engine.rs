//! Engine blocks (record design 3.5, Alex's ruling of 19 Nov): a dedicated
//! engine behind a block face. The declaration is an atom's (ports, params,
//! cost class, explain), so it wires into any flow, box or program; opening
//! one opens the engine's own editor, not blocks. Its explanation is the
//! engine's own, carried on the node's note.
//!
//! - `engine.classify`: the class atlas (editor: the Class atlas screen).
//! - `engine.place`: reverse placement, "put one of those here".

use serde::Deserialize;
use serde_json::Value as J;
use sse_core::{Dimension, Seed};
use sse_flow::{Atom, AtomDecl, AtomError, BlockClass, Call, CostClass, Out, Port, Shape, Val};
use sse_place::{Model, Placer, Request, System};
use sse_sift::classify::Atlas;
use sse_sift::predicate::Subject;

fn port(name: &str, shape: Shape) -> Port {
    Port {
        name: name.into(),
        kind: shape.to_string(),
        dim: Dimension::NONE,
        valid: None,
        shape: (shape != Shape::Number).then_some(shape),
        any_dim: false,
    }
}
fn decl(
    id: &str,
    cost: CostClass,
    inputs: Vec<Port>,
    outputs: Vec<Port>,
    c: &str,
    s: &str,
) -> AtomDecl {
    AtomDecl {
        id: id.into(),
        version: "1.0.0".into(),
        inputs,
        outputs,
        params: vec![],
        cost_class: cost,
        references: vec![],
        explain: sse_flow::decl::ExplainText {
            curious: c.into(),
            science: s.into(),
            formula: String::new(),
        },
        class: Some(BlockClass::Engine),
        dims: None,
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ClassifyCfg {
    scheme: String,
    /// Adds a `held` input: the label held before, for stickiness.
    #[serde(default)]
    sticky: bool,
}

/// `engine.classify`: the subject's facts (from the world) and a scheme in; the class, its
/// modifiers and how well it fits out. The note is the atlas's own label,
/// with its axes, region and runners-up.
pub struct Classify {
    atlas: Atlas,
    decl: AtomDecl,
}

impl Classify {
    pub fn new(atlas: Atlas) -> Self {
        let (inputs, outputs) = Self::ports_for(false);
        Classify {
            atlas,
            decl: decl(
                "engine.classify",
                CostClass::Light,
                inputs,
                outputs,
                "Says what kind of thing this is in one scheme (a garden world, a hot Jupiter).",
                "The class atlas: each class is a region on named axes; the label is the class whose region fits best, with its modifiers. Edited in the Class atlas, not as blocks.",
            ),
        }
    }
    fn ports_for(sticky: bool) -> (Vec<Port>, Vec<Port>) {
        let mut ins = vec![];
        if sticky {
            ins.push(port("held", Shape::Choice));
        }
        (
            ins,
            vec![
                port("class", Shape::Choice),
                port("classified", Shape::Flag),
                port("modifiers", Shape::List(Box::new(Shape::Choice))),
                port("fit", Shape::Number),
            ],
        )
    }
    fn cfg(&self, config: Option<&J>) -> Result<ClassifyCfg, String> {
        let j = config.ok_or("engine.classify needs `scheme` in its config")?;
        let c: ClassifyCfg =
            serde_json::from_value(j.clone()).map_err(|e| format!("engine.classify: {e}"))?;
        if !self.atlas.schemes.contains_key(&c.scheme)
            && !self.atlas.classes.iter().any(|k| k.scheme == c.scheme)
        {
            return Err(format!(
                "engine.classify: no scheme {:?} in the atlas",
                c.scheme
            ));
        }
        Ok(c)
    }
}

impl Atom for Classify {
    fn decl(&self) -> &AtomDecl {
        &self.decl
    }
    fn eval(&self, _: &[f64], _: &[f64]) -> Result<Vec<f64>, AtomError> {
        Err(AtomError(
            "engine.classify is evaluated through its typed form".into(),
        ))
    }
    fn ports(&self, config: Option<&J>) -> Option<(Vec<Port>, Vec<Port>)> {
        Some(Self::ports_for(self.cfg(config).is_ok_and(|c| c.sticky)))
    }
    fn check_config(&self, config: Option<&J>) -> Result<(), String> {
        self.cfg(config).map(|_| ())
    }
    fn eval_typed(&self, v: &[Val], _: &[f64], cx: &mut Call) -> Result<Out, AtomError> {
        let c = self.cfg(cx.config).map_err(AtomError)?;
        let w = cx
            .env
            .world
            .ok_or_else(|| AtomError("engine.classify reads the world and has none here".into()))?;
        let s = Subject {
            address: cx.env.subject.clone(),
            facts: w.facts(None),
            parent: w.parent_facts(None),
            tags: w.tags(None),
            now_s: cx.env.now_s,
        };
        let held = v.first().and_then(|h| h.text()).filter(|t| !t.is_empty());
        let label = self.atlas.label(&c.scheme, &s, held);
        Ok(Out {
            values: vec![
                Val::Choice(label.base.clone().unwrap_or_default()),
                Val::Flag(label.base.is_some()),
                Val::List(label.modifiers.iter().cloned().map(Val::Choice).collect()),
                Val::Number(label.fit),
            ],
            note: Some(serde_json::to_value(&label).map_err(|e| AtomError(e.to_string()))?),
            ..Default::default()
        })
    }
}

/// What `engine.place` reads: the atlas, the base-physics model and the
/// life suite. Placement resolves the universe, so it is keyed on the
/// universe seed in the environment (never the table secret).
pub struct PlaceEngine {
    pub atlas: Atlas,
    pub model: Model,
    pub life: Box<dyn sse_life::LifeSuite + Send + Sync>,
    pub life_rules: Vec<sse_life::HabitatRule>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PlaceCfg {
    scheme: String,
    class: String,
    #[serde(default)]
    scope_au: Option<(f64, f64)>,
    #[serde(default)]
    force: bool,
}

/// `engine.place`: the class atlas's second port set. A system record in;
/// where one of the requested class goes, or why it cannot, out. The note
/// is the whole placement with its bounds and causes.
pub struct Place {
    engine: PlaceEngine,
    decl: AtomDecl,
}

impl Place {
    pub fn new(engine: PlaceEngine) -> Self {
        Place {
            engine,
            decl: decl(
                "engine.place",
                CostClass::Medium,
                vec![port("system", Shape::Record)],
                vec![
                    port("outcome", Shape::Choice),
                    port("label", Shape::Choice),
                    port("recognised", Shape::Flag),
                    port("fit", Shape::Number),
                    port("a_au", Shape::Number),
                ],
                "Puts a world of a chosen class into a system, or says what stands in the way.",
                "Reverse placement (C3): bounds from the class's region, then candidates run through the physics flows until the atlas recognises one. Edited in the Class atlas.",
            ),
        }
    }
    fn cfg(&self, config: Option<&J>) -> Result<PlaceCfg, String> {
        let j = config.ok_or("engine.place needs `scheme` and `class` in its config")?;
        let c: PlaceCfg =
            serde_json::from_value(j.clone()).map_err(|e| format!("engine.place: {e}"))?;
        if !self
            .engine
            .atlas
            .classes
            .iter()
            .any(|k| k.id == c.class && k.scheme == c.scheme)
        {
            return Err(format!(
                "engine.place: no class {:?} in scheme {:?}",
                c.class, c.scheme
            ));
        }
        Ok(c)
    }
}

impl Atom for Place {
    fn decl(&self) -> &AtomDecl {
        &self.decl
    }
    fn eval(&self, _: &[f64], _: &[f64]) -> Result<Vec<f64>, AtomError> {
        Err(AtomError(
            "engine.place is evaluated through its typed form".into(),
        ))
    }
    fn check_config(&self, config: Option<&J>) -> Result<(), String> {
        self.cfg(config).map(|_| ())
    }
    fn eval_typed(&self, v: &[Val], _: &[f64], cx: &mut Call) -> Result<Out, AtomError> {
        let c = self.cfg(cx.config).map_err(AtomError)?;
        let e = |m: String| AtomError(format!("engine.place: {m}"));
        let sys: System = serde_json::to_value(&v[0])
            .and_then(serde_json::from_value)
            .map_err(|x| e(format!("system record: {x}")))?;
        let seed: Seed = cx
            .env
            .seed
            .ok_or_else(|| e("needs the universe seed".into()))?;
        let at = cx
            .env
            .subject
            .clone()
            .ok_or_else(|| e("needs a subject (the system's address)".into()))?;
        let p = Placer {
            atlas: &self.engine.atlas,
            model: &self.engine.model,
            life: self.engine.life.as_ref(),
            life_rules: &self.engine.life_rules,
        };
        let req = Request {
            scheme: c.scheme,
            class: c.class,
            scope_au: c.scope_au,
            force: c.force,
        };
        let r = p.place(&seed, &at, &sys, &req).map_err(e)?;
        let outcome = serde_json::to_value(r.outcome)
            .ok()
            .and_then(|j| j.as_str().map(String::from))
            .unwrap_or_default();
        Ok(Out {
            values: vec![
                Val::Choice(outcome),
                Val::Choice(r.label.clone().unwrap_or_default()),
                Val::Flag(r.recognised),
                Val::Number(r.fit),
                Val::Number(r.causes.as_ref().map_or(0.0, |k| k.a_au)),
            ],
            note: Some(serde_json::to_value(&r).map_err(|x| e(x.to_string()))?),
            ..Default::default()
        })
    }
}

//! Fixtures and the golden scenario.

use crate::epoch::{SystemSpec, TimeSettings};
use sse_pack::PackManifest;

pub fn base_time_pack() -> PackManifest {
    PackManifest::from_json(include_str!("../../../packs/base-time/pack.json"))
        .expect("base time pack")
}
pub fn settings() -> TimeSettings {
    TimeSettings::from_packs(&[base_time_pack()]).expect("time settings")
}
pub fn solar_system() -> SystemSpec {
    serde_json::from_str(include_str!(
        "../../../tests/fixtures/time/solar-system.json"
    ))
    .expect("solar fixture")
}

use crate::branch::{self, LookAhead, Request};
use sse_chronicle::{Chronicle, PackRef, RunManifest};
use sse_core::{ContentHash, Seed};

/// How far the scenario runs: 10 Gyr past the planetary nebula.
pub const COOLING_MYR: f64 = 10_000.0;
/// Where the GM moves "now" on commit: 1 Gyr into the white dwarf's life.
pub const NOW_AFTER_NEBULA_MYR: f64 = 1_000.0;

pub struct Aged {
    pub chronicle: Chronicle,
    pub look: LookAhead,
    pub manifest: RunManifest,
}

/// "Age the Solar System" end to end: look ahead to a cold white dwarf on a
/// branch, commit it, and move now to 1 Gyr after the planetary nebula.
pub fn age_the_solar_system() -> Aged {
    let settings = settings();
    let sys = solar_system();
    let star =
        crate::star::AgeingStar::new(sys.star_mass, sys.z, settings.star.clone()).expect("star");
    let t_end = star.line.t_end;
    let req = Request::new(sys, settings, t_end + COOLING_MYR);
    let mut ch = Chronicle::new();
    let look = branch::look_ahead(&mut ch, req, &mut |_| true)
        .expect("look-ahead")
        .expect("not cancelled");
    let la = look.clone();
    branch::commit(&mut ch, &la, Some(t_end + NOW_AFTER_NEBULA_MYR)).expect("commit");
    let p = base_time_pack();
    let manifest = RunManifest::new(
        Seed::from_text("sol"),
        "0.1.0",
        vec![PackRef {
            id: p.id.clone(),
            version: p.version.clone(),
            content_hash: p.content_hash(),
        }],
        ch.authored_hash(),
    );
    Aged {
        chronicle: ch,
        look,
        manifest,
    }
}

/// T2 golden: the run's events (dated in universe years), digests of the
/// whole run and the chronicle head, identical on every device.
pub fn golden() -> serde_json::Value {
    let a = age_the_solar_system();
    let sp = &a.look.request.system;
    let events: Vec<serde_json::Value> = a
        .look
        .run
        .events
        .iter()
        .map(|e| serde_json::json!([branch::universe_time(sp, e.age_myr).s, e.kind, e.subjects]))
        .collect();
    let digest = ContentHash::of(&serde_json::to_string(&a.look.run).expect("json")).expect("hash");
    let last = a.look.run.snapshots.last().expect("snapshots");
    serde_json::json!({
        "events": events,
        "windows": a.look.run.windows.iter().map(|w| serde_json::json!([w.age_myr, w.steps])).collect::<Vec<_>>(),
        "work": a.look.run.work,
        "run_digest": digest.hex(),
        "final_star": [last.star.phase, last.star.colour, last.star.rgb],
        "chronicle_events": a.chronicle.len(),
        "chronicle_head": a.chronicle.head().hex(),
        "manifest": a.manifest.id().hex(),
    })
}

use sse_formation::pipeline::{run as form_run, Block, SystemRun};

/// The first `n` G0 solar-patch systems through the formation pipeline.
pub fn generated_runs(n: usize) -> Vec<SystemRun> {
    sse_formation::scenario::solar_patch()
        .into_iter()
        .flatten()
        .take(n)
        .map(|h| form_run(h, Block::Modern))
        .collect()
}

/// A Sun-like star (1 solar mass, alone, 4.57 Gyr) from the stand-in envelope.
pub fn sunlike(n: u64) -> SystemRun {
    let mut man = sse_formation::Manifest::new(
        "f7-sunlike",
        sse_formation::scenario::system(n),
        sse_formation::scenario::envelope(),
    );
    man.star_pins.primary_mass = Some(1.0);
    man.star_pins.count = Some(1);
    man.inherit_pins.age_gyr = Some(4.57);
    form_run(
        sse_formation::FormationHistory::form(man).expect("forms"),
        Block::Modern,
    )
}

/// Ten billion years past now, or to 1 Gyr past the star's end if sooner.
pub fn target(r: &SystemRun) -> f64 {
    let s = settings();
    let now = r.history.inheritance.age_gyr * 1000.0;
    let star =
        crate::star::AgeingStar::new(r.history.stars[0].mass, r.history.inheritance.z, s.star)
            .expect("star");
    (now + 10_000.0)
        .min(star.line.t_end + 1000.0)
        .max(now + 1.0)
}

/// F7 golden: three G0 systems and one Sun-like system aged through their
/// long life, the merged story dated, identical on every device.
pub fn generated_golden() -> serde_json::Value {
    let settings = settings();
    let mut all = generated_runs(3);
    all.push(sunlike(5));
    let out: Vec<_> = all
        .iter()
        .map(|r| {
            let a = crate::generated::long_life(r, &settings, target(r)).expect("ages");
            serde_json::json!({
                "system": a.spec.system,
                "story": a.story.iter().map(|s| serde_json::json!({
                    "age_myr": s.age_myr, "kind": s.kind, "from": s.from,
                })).collect::<Vec<_>>(),
                "digest": ContentHash::of(&serde_json::to_string(&a).expect("json")).expect("hash").hex(),
            })
        })
        .collect();
    serde_json::Value::Array(out)
}

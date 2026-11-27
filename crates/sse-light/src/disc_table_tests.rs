//! ER11a: the disc table agrees with the kernel's own trace, samples
//! within its bound at every size, and follows the block in force.

use crate::disc_table::{self, SHIFT_BLOCK};
use crate::paths::{self, LightPaths};
use crate::*;
use sse_flow::Atom;
use sse_formula::FormulaAtom;
use sse_physics::table::{Size, Table};
use std::sync::Arc;

fn blocks() -> Vec<Arc<dyn Atom>> {
    // The blocks in force for a bare physics: the shipped library, as shipped.
    sse_physics::Physics::bare(Arc::new(sse_reference::Reference::default()))
        .blocks()
        .iter()
        .map(|(_, a)| a.clone())
        .collect()
}

fn fixtures() -> Vec<LightPaths> {
    crate::scenario::fixtures()
        .iter()
        .map(|a| paths::read(a, &emit::Model::base().table).unwrap())
        .collect()
}

fn subject(lp: &LightPaths) -> serde_json::Value {
    let f = lp.feed.as_ref().expect("a fed hole");
    serde_json::json!({
        "gravitational_radius_m": lp.gravitational_radius_m,
        "rate_kg_s": f.disc_rate_kg_s(),
        "inner_m": f.disc.inner_m,
        "outer_m": f.disc.outer_m,
    })
}

fn lookup(blocks: &[Arc<dyn Atom>]) -> impl Fn(&str) -> Option<Arc<dyn Atom>> + '_ {
    move |id: &str| blocks.iter().find(|a| a.decl().id == id).cloned()
}

fn seal(s: serde_json::Value, size: Size, blocks: &[Arc<dyn Atom>]) -> Table {
    let b = disc_table::bake(&s, size, &lookup(blocks)).unwrap();
    Table::seal(disc_table::ID, s, size, None, "test".into(), b).unwrap()
}

#[test]
fn every_fixture_samples_within_its_bound_at_every_size() {
    for lp in fixtures() {
        if lp.feed.as_ref().is_none_or(|f| f.disc_rate_kg_s() <= 0.0) {
            continue;
        }
        for size in [Size::Small, Size::Medium, Size::Large] {
            let t = seal(subject(&lp), size, &blocks());
            assert!(
                t.sampling.within,
                "{} at {size:?}: {:?}",
                lp.address, t.sampling
            );
            assert!(t.stand_ins.is_empty(), "{:?}", t.stand_ins);
        }
    }
}

/// The picture's brightest, approaching and receding points as the kernel
/// traced them (XO3) read back off the table: the same shift and the same
/// temperature, within the table's bound, at the kernel's own rays.
#[test]
fn the_table_gives_the_traced_points_their_shift_and_temperature() {
    let mut checked = 0;
    for lp in fixtures() {
        let (Some(img), Some(_)) = (lp.image.as_ref(), lp.feed.as_ref()) else {
            continue;
        };
        let t = seal(subject(&lp), Size::Medium, &blocks());
        let (z, t4) = (t.channel("redshift").unwrap(), t.channel("t4").unwrap());
        let bound = disc_table::bounds(Size::Medium);
        for s in [&img.brightest, &img.approaching, &img.receding] {
            let q = s.impact_rg * m::sin(img.tilt_rad) * m::sin(s.image_angle_rad) / s.r_rg;
            let v = t.sample(&[s.r_rg, q]);
            let te = m::sqrt(m::sqrt(v[t4]));
            assert!(
                (1.0 / v[z] - s.shift).abs() / s.shift < bound[0],
                "{}: shift {} against traced {}",
                lp.address,
                1.0 / v[z],
                s.shift
            );
            assert!(
                (te - s.t_emitted_k).abs() / s.t_emitted_k < bound[1],
                "{}: T {} against traced {}",
                lp.address,
                te,
                s.t_emitted_k
            );
            checked += 1;
        }
    }
    assert!(checked >= 6, "checked {checked} points");
}

fn edited(formula_from: &str, formula_to: &str, face_on_g: f64) -> Vec<Arc<dyn Atom>> {
    let text = sse_atoms::formulas::FILES
        .iter()
        .find(|(f, _)| *f == "light")
        .unwrap()
        .1;
    let doc: serde_json::Value = serde_json::from_str(text).unwrap();
    let mut entry = doc["atoms"]
        .as_array()
        .unwrap()
        .iter()
        .find(|a| a["id"] == SHIFT_BLOCK)
        .unwrap()
        .clone();
    let f = entry["equation"]["formula"].as_str().unwrap().to_string();
    assert!(f.contains(formula_from), "{f}");
    entry["equation"]["formula"] = f.replace(formula_from, formula_to).into();
    entry["equation"]["examples"] = serde_json::json!([{"rg": 1.0, "r": 6.0, "b": 0.0, "i": 0.0, "al": 0.0, "g": face_on_g, "tolerance": 1e-12}]);
    let mine: Arc<dyn Atom> = Arc::new(FormulaAtom::from_json(&entry).unwrap());
    blocks()
        .into_iter()
        .map(|a| {
            if a.decl().id == SHIFT_BLOCK {
                mine.clone()
            } else {
                a
            }
        })
        .collect()
}

/// The design's check: editing `sse.light.disc_shift` changes the table.
/// Here the gravitational redshift is taken out of the law (orbital Doppler
/// only); the shift channel follows the edited block exactly, and the
/// temperature does not move.
#[test]
fn editing_the_shift_block_changes_the_table() {
    let lp = &fixtures()[0];
    let base = seal(subject(lp), Size::Small, &blocks());
    let mine = edited("/ sqrt(1 - 3 / x)", "/ 1", 1.0);
    let changed = seal(subject(lp), Size::Small, &mine);
    assert_ne!(base.hash, changed.hash);
    let z = base.channel("redshift").unwrap();
    let t4 = base.channel("t4").unwrap();
    // At the inner edge, face on (λ = 0): Luminet's g is √(1 − 3/x); the
    // edited block's is 1.
    let r = base.axes[0].from;
    let before = base.sample(&[r, 0.0]);
    let after = changed.sample(&[r, 0.0]);
    assert!(
        (1.0 / before[z] - m::sqrt(1.0 - 3.0 / r)).abs() < 1e-6,
        "{before:?}"
    );
    assert!((1.0 / after[z] - 1.0).abs() < 1e-6, "{after:?}");
    assert_eq!(before[t4], after[t4]);
    assert_eq!(
        changed
            .reads
            .iter()
            .find(|r| r.id == SHIFT_BLOCK)
            .unwrap()
            .id,
        SHIFT_BLOCK
    );
}

/// A block in force that reads the tilt other than through b sin i sin α
/// cannot be indexed this way, and the table says so.
#[test]
fn a_block_the_indexing_does_not_hold_for_is_named() {
    let lp = &fixtures()[0];
    let mine = edited(
        "bb * sin(i) * sin(al)",
        "bb * sin(i) * sin(i) * sin(al)",
        0.5f64.sqrt(),
    );
    let t = seal(subject(lp), Size::Small, &mine);
    assert!(
        t.stand_ins.iter().any(|s| s.contains("does not hold")),
        "{:?}",
        t.stand_ins
    );
}

#[test]
fn a_disc_inside_the_horizon_s_double_is_refused() {
    let s = serde_json::json!({"gravitational_radius_m": 1.0, "rate_kg_s": 1.0, "inner_m": 1.5, "outer_m": 10.0});
    assert!(disc_table::bake(&s, Size::Small, &lookup(&blocks())).is_err());
}

/// The picture asks with the hole's feed as `read_light_paths` gives it; the
/// kernel takes the disc's edges and rate from it, the same table as the
/// numbers give.
#[test]
fn a_feed_gives_the_same_table_as_its_numbers() {
    let lp = &fixtures()[0];
    let by_feed = serde_json::json!({
        "gravitational_radius_m": lp.gravitational_radius_m,
        "feed": serde_json::to_value(lp.feed.as_ref().unwrap()).unwrap(),
    });
    assert_eq!(
        seal(by_feed, Size::Small, &blocks()).hash,
        seal(subject(lp), Size::Small, &blocks()).hash
    );
}

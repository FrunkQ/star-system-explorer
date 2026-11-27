//! BX1: two stars aged together.
use crate::binary::BinaryRules;
use crate::pair::*;

fn star_set() -> crate::star::StarSettings {
    crate::scenario::settings().star
}

fn two(m1: f64, m2: f64, a_au: f64, e: f64, to: f64, planets: &[PlanetIn]) -> PairLine {
    let n = Node::Pair {
        a: Box::new(Node::Star {
            address: "u:t/sys:0/star:0".into(),
            mass_msun: m1,
        }),
        b: Box::new(Node::Star {
            address: "u:t/sys:0/star:1".into(),
            mass_msun: m2,
        }),
        separation_au: a_au,
        eccentricity: e,
    };
    age_node(
        &n,
        0.02,
        to,
        planets,
        &star_set(),
        &BinaryRules::default(),
        &PairSettings::default(),
    )
    .unwrap()
}

fn total(r: &PairRow) -> f64 {
    r.a.mass_msun + r.b.as_ref().map_or(0.0, |b| b.mass_msun)
}

/// Far apart, two stars age as if alone: no events, and the orbit widens
/// as a·M stays constant while the heavier one sheds its envelope.
#[test]
fn a_wide_pair_ages_as_two_lone_stars() {
    let l = two(1.5, 0.8, 50.0, 0.1, 4000.0, &[]);
    assert!(
        l.events.is_empty(),
        "{:?}",
        l.events.iter().map(|e| &e.kind).collect::<Vec<_>>()
    );
    let (first, last) = (l.rows.first().unwrap(), l.rows.last().unwrap());
    assert_eq!(last.a.phase, "white dwarf");
    let am0 = first.separation_au * total(first);
    let am1 = last.separation_au * total(last);
    assert!((am1 / am0 - 1.0).abs() < 1e-6, "{am0} {am1}");
    assert!(last.separation_au > 1.5 * first.separation_au);
}

/// Algol-like: a 3 + 1.5 solar mass pair a few days apart. The heavier star
/// fills its lobe as it leaves the main sequence, gives most of its mass to its
/// companion (the mass ratio reverses, as in Algol), and the orbit first
/// shrinks then widens. Mass is conserved apart from what is said to be lost.
#[test]
fn a_close_pair_trades_mass_and_the_ratio_reverses() {
    let l = two(3.0, 1.5, 0.08, 0.0, 450.0, &[]);
    let kinds: Vec<&str> = l.events.iter().map(|e| e.kind.as_str()).collect();
    assert_eq!(kinds.first(), Some(&"stable-mass-transfer"), "{kinds:?}");
    let last = l.rows.last().unwrap();
    let b = last.b.as_ref().expect("still two stars");
    assert!(
        b.mass_msun > last.a.mass_msun,
        "{} {}",
        last.a.mass_msun,
        b.mass_msun
    );
    assert!(b.mass_msun > 2.0, "{}", b.mass_msun);
    let lowest = l
        .rows
        .iter()
        .map(|r| r.separation_au)
        .fold(f64::INFINITY, f64::min);
    assert!(
        lowest < 0.08 && last.separation_au > lowest,
        "{lowest} {}",
        last.separation_au
    );
    // Budget: what the pair has plus what left it equals what it began with, less winds.
    let start = total(&l.rows[0]);
    assert!(total(last) + last.lost_msun <= start + 1e-9);
    assert!(
        total(last) + last.lost_msun > 0.9 * start,
        "{} + {} of {start}",
        total(last),
        last.lost_msun
    );
}

/// The Sun-like star with a red dwarf 1 au away: on its giant branch it
/// engulfs the companion in a common envelope, leaving a white dwarf and the
/// dwarf days apart, the envelope gone from the pair.
#[test]
fn a_giant_and_a_dwarf_share_an_envelope() {
    let l = two(1.0, 0.3, 1.0, 0.0, 13_000.0, &[]);
    let ce = l
        .events
        .iter()
        .find(|e| e.kind == "common-envelope")
        .expect("a common envelope");
    let after = l.rows.iter().find(|r| r.age_myr > ce.age_myr).unwrap();
    assert_eq!(after.a.phase, "white dwarf");
    // A post-common-envelope pair: days apart (observed 0.1 to 10 days, Nebot Gomez-Moran et al. 2011).
    assert!(
        after.period_days < 10.0 && after.period_days > 0.05,
        "{}",
        after.period_days
    );
    assert!(after.lost_msun > 0.2);
}

/// Two main-sequence stars in contact with a ratio past the critical one merge.
#[test]
fn unequal_dwarfs_in_contact_merge() {
    // A 2 and a 0.5 solar mass star 4 solar radii apart: the heavier fills its lobe as it
    // swells on the main sequence with a ratio of 4, past the critical 3.
    let l = two(2.0, 0.5, 0.02, 0.0, 2000.0, &[]);
    assert!(
        l.events.iter().any(|e| e.kind == "merger"),
        "{:?}",
        l.events.iter().map(|e| &e.kind).collect::<Vec<_>>()
    );
    assert!(l.rows.last().unwrap().b.is_none());
}

/// A planet close in round the Sun-like star is swallowed on the giant branch;
/// one far out is kept, its orbit wider.
#[test]
fn a_giant_swallows_its_close_planet_and_keeps_the_far_one() {
    let p = |a: &str, au: f64| PlanetIn {
        address: a.into(),
        host: "u:t/sys:0/star:0".into(),
        semi_major_axis_au: au,
    };
    let l = two(
        1.0,
        0.5,
        200.0,
        0.0,
        13_000.0,
        &[p("near", 0.4), p("far", 5.0)],
    );
    let near = l.planets.iter().find(|x| x.address == "near").unwrap();
    let far = l.planets.iter().find(|x| x.address == "far").unwrap();
    assert_eq!(near.fate, "engulfed");
    assert_eq!(far.fate, "kept");
    assert!(far.semi_major_axis_au > 6.0, "{}", far.semi_major_axis_au);
}

/// Nested: a close pair inside a wide one; the outer orbit follows both sides' masses.
#[test]
fn a_pair_of_pairs_acts_as_mass_centres() {
    let inner = Node::Pair {
        a: Box::new(Node::Star {
            address: "s0".into(),
            mass_msun: 1.2,
        }),
        b: Box::new(Node::Star {
            address: "s1".into(),
            mass_msun: 1.0,
        }),
        separation_au: 0.5,
        eccentricity: 0.0,
    };
    let n = Node::Pair {
        a: Box::new(inner),
        b: Box::new(Node::Star {
            address: "s2".into(),
            mass_msun: 0.7,
        }),
        separation_au: 100.0,
        eccentricity: 0.3,
    };
    let l = age_node(
        &n,
        0.02,
        3000.0,
        &[],
        &star_set(),
        &BinaryRules::default(),
        &PairSettings::default(),
    )
    .unwrap();
    assert_eq!(l.inner.len(), 1);
    let first = &l.rows[0];
    assert!((first.a.mass_msun - 2.2).abs() < 1e-9);
    let last = l.rows.last().unwrap();
    let am = |r: &PairRow| r.separation_au * total(r);
    assert!((am(last) / am(first) - 1.0).abs() < 1e-6);
}

#[test]
#[ignore]
fn show_algol() {
    let l = two(3.0, 1.5, 0.08, 0.0, 600.0, &[]);
    for e in &l.events {
        eprintln!("{:.2} {} {}", e.age_myr, e.kind, e.why);
    }
    for r in l.rows.iter().step_by((l.rows.len() / 40).max(1)) {
        let Some(b) = r.b.as_ref() else { continue };
        eprintln!("{:8.3} a {:.3} {} R{:.2} f{:.2} | b {:.3} {} R{:.2} f{:.2} | sep {:.4} rate {:.2e} lost {:.3}", r.age_myr, r.a.mass_msun, r.a.phase, r.a.radius_rsun, r.a.lobe_fill, b.mass_msun, b.phase, b.radius_rsun, b.lobe_fill, r.separation_au, r.transfer_msun_yr, r.lost_msun);
    }
    eprintln!("rows {}", l.rows.len());
}

/// Prints a post-common-envelope pair aged on (run with --ignored --nocapture).
#[test]
#[ignore]
fn show_cv() {
    let l = two(2.0, 0.4, 1.0, 0.0, 13_700.0, &[]);
    for e in &l.events {
        println!("{:9.1} {} {}", e.age_myr, e.kind, e.why);
    }
    for r in l
        .rows
        .iter()
        .filter(|r| r.b.as_ref().is_some_and(|b| b.mass_msun < 0.2))
        .step_by(10)
        .take(30)
    {
        println!(
            "{:?} {:?}",
            r.a.phase,
            r.b.as_ref().map(|b| b.phase.clone())
        );
        println!(
            "{:9.1} a {:.3e} P {:.4} d donor {:?} rate {:.2e} {:.3} {:.3} R {:.3} fill {:.3}",
            r.age_myr,
            r.separation_au,
            r.period_days,
            r.donor,
            r.transfer_msun_yr,
            r.a.mass_msun,
            r.b.as_ref().map_or(0.0, |b| b.mass_msun),
            r.b.as_ref().map_or(0.0, |b| b.radius_rsun),
            r.b.as_ref().map_or(0.0, |b| b.lobe_fill)
        );
    }
}

/// BX3: a 2 + 0.4 solar mass pair 1 au apart. The 2 makes a common envelope as a
/// giant; magnetic braking then draws the white dwarf and the red dwarf into
/// touch, a cataclysmic variable. The red dwarf is fed off near the rates seen
/// above the period gap, stops braking once it is convective through (0.35),
/// is drawn on by gravitational waves alone at far lower rates, and shrinks to
/// a brown dwarf: the period falls to a minimum near an hour and rises again.
#[test]
fn braking_and_waves_make_a_cataclysmic_variable() {
    let l = two(2.0, 0.4, 1.0, 0.0, 13_700.0, &[]);
    let cv = l
        .events
        .iter()
        .find(|e| e.kind == "cataclysmic-variable")
        .expect("a cataclysmic variable");
    // Contact a few hours apart (observed CVs, 1.3 to about 10 hours).
    let p_h = cv.numbers["period_days"] * 24.0;
    assert!((2.0..8.0).contains(&p_h), "{p_h}");
    let fed: Vec<&PairRow> = l
        .rows
        .iter()
        .filter(|r| r.age_myr > cv.age_myr && r.transfer_msun_yr > 0.0)
        .collect();
    let mass = |r: &PairRow| r.b.as_ref().unwrap().mass_msun;
    // Above the gap: about 1e-9 to 1e-8 solar masses a year (Knigge et al. 2011).
    let above = fed
        .iter()
        .filter(|r| mass(r) > 0.36)
        .map(|r| r.transfer_msun_yr);
    let top = above.fold(0.0, f64::max);
    assert!((1e-9..1e-7).contains(&top), "{top}");
    // Below it, gravitational waves alone: about 1e-11 to 1e-10.
    // The rows catch the flow at moments of a step, so the middle value is read.
    let mut below: Vec<f64> = fed
        .iter()
        .filter(|r| mass(r) < 0.2 && mass(r) > 0.08)
        .map(|r| r.transfer_msun_yr)
        .collect();
    assert!(!below.is_empty());
    below.sort_by(f64::total_cmp);
    let mid = below[below.len() / 2];
    assert!((1e-11..1e-9).contains(&mid), "{mid}");
    assert!(below.iter().all(|x| *x < 1e-9));
    // The period minimum (observed near 80 minutes) and the bounce.
    let after: Vec<&PairRow> = l.rows.iter().filter(|r| r.age_myr > cv.age_myr).collect();
    let (i, p_min) = after
        .iter()
        .enumerate()
        .map(|(i, r)| (i, r.period_days))
        .fold((0, f64::MAX), |b, x| if x.1 < b.1 { x } else { b });
    assert!((0.03..0.07).contains(&p_min), "{p_min}");
    let last = after.last().unwrap();
    assert!(i < after.len() - 1 && last.period_days > p_min * 1.2);
    assert_eq!(last.b.as_ref().unwrap().phase, "brown dwarf");
    assert_eq!(last.a.phase, "white dwarf");
}

/// BX3: two white dwarfs from Algol's second episode draw together by waves.
#[test]
fn white_dwarf_pairs_spiral_in() {
    let l = two(3.0, 1.5, 0.08, 0.0, 3_000.0, &[]);
    let rows: Vec<&PairRow> = l
        .rows
        .iter()
        .filter(|r| {
            r.a.phase == "white dwarf" && r.b.as_ref().is_some_and(|b| b.phase == "white dwarf")
        })
        .collect();
    assert!(rows.len() >= 2, "no double white dwarf");
    let (first, last) = (rows[0], rows[rows.len() - 1]);
    assert!(
        last.separation_au < first.separation_au,
        "{} {}",
        first.separation_au,
        last.separation_au
    );
}

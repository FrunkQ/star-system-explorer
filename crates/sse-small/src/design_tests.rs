//! RW16a: the designer's pins, the designed field and what a design takes.

use super::*;
use sse_core::{Address, Seed};

fn sun() -> Host {
    Host {
        luminosity_lsun: 1.0,
        mass_msun: 1.0,
        age_myr: 4_570.0,
    }
}

fn addr(k: u64) -> Address {
    Address::universe("t")
        .unwrap()
        .child("sys", 1)
        .unwrap()
        .child("small", k)
        .unwrap()
}

fn rocky() -> Makeup {
    Makeup::from_array([0.05, 0.85, 0.1, 0.0])
}

fn design(k: u64, d_km: f64, pins: &Pins) -> Member {
    let s = Settings::default();
    let mk = pins.makeup.unwrap_or_else(rocky);
    let mass = mass_for_makeup(&mk, d_km, &s);
    let f = designed_field(addr(k), "made", &mk, 2.5, 0.1, mass);
    let mut b = resolve_pinned(
        &Seed::from_text("design"),
        &f,
        1,
        &sun(),
        None,
        None,
        &s,
        pins,
    );
    b.address = addr(k).to_string();
    b
}

#[test]
fn a_designed_body_is_the_same_lump_every_time_and_differs_by_address() {
    let a = design(1, 3.0, &Pins::default());
    assert_eq!(a, design(1, 3.0, &Pins::default()));
    let b = design(2, 3.0, &Pins::default());
    assert_ne!(a.figure.axes_m, b.figure.axes_m, "two addresses, two lumps");
    assert!(a.figure.relief > 0.0, "a 3 km body is never smooth");
    assert_ne!(a.structure.kind, "relaxed", "a 3 km body is never round");
}

#[test]
fn nothing_below_the_round_size_comes_out_as_a_sphere() {
    let s = Settings::default();
    // Across sizes from a boulder to just under the round size: a ratio of axes
    // under 0.97 or a relief over 1 % (absolute: a sphere has 1.0 and 0).
    for (k, d) in [0.01, 0.3, 1.0, 10.0, 100.0, 250.0].iter().enumerate() {
        let b = design(10 + k as u64, *d, &Pins::default());
        let [a, _, c] = b.figure.axes_m;
        assert!(
            c / a < 0.97 || b.figure.relief > 0.01,
            "{d} km: axes {:?} relief {}",
            b.figure.axes_m,
            b.figure.relief
        );
        assert!(*d < round_km(&b.makeup, &s));
    }
}

#[test]
fn pins_are_taken_as_given() {
    let p = Pins {
        pair: Some("contact".into()),
        structure: Some("rubble-pile".into()),
        spin_hours: Some(7.5),
        battered: Some(1.0),
        size_ratio: Some(0.6),
        ..Pins::default()
    };
    let b = design(3, 2.0, &p);
    assert_eq!(b.pair.kind, "contact");
    assert!((b.pair.size_ratio - 0.6).abs() < 1e-12);
    assert_eq!(b.figure.lobes.len(), 2);
    assert_eq!(b.structure.kind, "rubble-pile");
    assert!((b.spin_hours - 7.5).abs() < 1e-9);
    let s = Settings::default();
    assert!(
        (b.figure.relief - s.figure.relief_rubble[1]).abs() < 1e-9,
        "fully battered is the top of the range"
    );
    let single = design(
        3,
        2.0,
        &Pins {
            pair: Some("single".into()),
            ..Pins::default()
        },
    );
    assert_eq!(single.pair.kind, "single");
    assert!(single.figure.lobes.is_empty());
    let icy = design(
        4,
        2.0,
        &Pins {
            makeup: Some(Makeup::from_array([0.0, 0.3, 0.1, 0.6])),
            ..Pins::default()
        },
    );
    assert!((icy.makeup.ice - 0.6).abs() < 1e-12);
}

#[test]
fn implausible_is_allowed_and_says_what_it_takes() {
    // A 5 km rubble pile turning once an hour: past the spin barrier (about 2.2 h).
    let fast = design(
        5,
        5.0,
        &Pins {
            structure: Some("rubble-pile".into()),
            spin_hours: Some(1.0),
            ..Pins::default()
        },
    );
    assert!((fast.spin_hours - 1.0).abs() < 1e-9, "the pin is kept");
    let t = takes(&fast, &Settings::default());
    let spin = t.iter().find(|x| x.about == "spin").expect("a spin note");
    assert!(spin.needs_pa > spin.holds_pa, "{spin:?}");
    assert!(spin.says.contains("fly apart"), "{}", spin.says);
    // A lumpy 1,500 km rock: past the round size.
    let big = design(
        6,
        1500.0,
        &Pins {
            structure: Some("monolith".into()),
            ..Pins::default()
        },
    );
    assert_ne!(big.structure.kind, "relaxed");
    assert!(big.figure.relief > 0.05, "kept lumpy as asked");
    let t = takes(&big, &Settings::default());
    let shape = t.iter().find(|x| x.about == "shape").expect("a shape note");
    // Absolute: a 1,500 km rock presses its centre at about half a gigapascal
    // ((2 pi / 3) G rho^2 R^2 with rho 2,700 and R 750 km is 0.54 GPa).
    assert!(
        shape.needs_pa > 4.0e8 && shape.needs_pa < 7.0e8,
        "{shape:?}"
    );
    assert!(shape.needs_pa > shape.holds_pa);
    // A slow, small, ordinary body takes nothing special.
    let calm = design(
        7,
        2.0,
        &Pins {
            structure: Some("monolith".into()),
            spin_hours: Some(8.0),
            ..Pins::default()
        },
    );
    assert!(takes(&calm, &Settings::default()).is_empty());
}

#[test]
fn makeup_follows_a_measured_density() {
    let s = Settings::default();
    let icy = makeup_for_density(1_200.0, &s);
    assert!(icy.ice > 0.5, "{icy:?}");
    let rock = makeup_for_density(2_700.0, &s);
    assert!(rock.ice < 0.2 && rock.metal < 0.05, "{rock:?}");
    let iron = makeup_for_density(5_000.0, &s);
    assert!(iron.metal > 0.3, "{iron:?}");
}

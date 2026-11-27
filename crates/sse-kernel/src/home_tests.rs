//! GZ12: home is one of the web's own galaxies, and every op that reads the
//! campaign's galaxy reads it. Standing rule (grand zoom 6.5): no op falls
//! back to a stand-in place.

use crate::Kernel;

fn open(seed: &str) -> Kernel {
    let mut k = Kernel::new();
    let r = k.handle_json(&format!(
        r#"{{"op":"open_universe","id":"{seed}","seed":"{seed}"}}"#
    ));
    assert!(r.contains("universe_opened"), "{r}");
    k
}

fn ask(k: &mut Kernel, req: &str) -> serde_json::Value {
    serde_json::from_str(&k.handle_json(req)).unwrap()
}

const STAND_IN: &str = "halo:milky-way-like";

#[test]
fn home_is_a_catalogued_halo_that_fits_the_wish() {
    let mut k = open("g3");
    let r = ask(&mut k, r#"{"op":"home_galaxy"}"#);
    assert_eq!(r["type"], "home_galaxy", "{r}");
    let h = &r["home"];
    let addr = h["halo"].as_str().unwrap();
    assert!(addr.starts_with("u:g3/cell:"), "{addr}");
    assert!(!addr.contains(STAND_IN));
    assert_eq!(h["by"], "web");
    assert_eq!(h["fits_wish"], true, "{h}");
    let mass = h["mass_msun"].as_f64().unwrap();
    assert!((5.0e11..=2.0e12).contains(&mass), "{mass}");
    assert_eq!(h["neighbour"]["bound"], true);
    // It is one of its own cell's haloes, at the place the cell gives it.
    let cell = ask(
        &mut k,
        &format!(r#"{{"op":"halo_cell","cell":{}}}"#, h["cell"]),
    );
    let found = cell["cell"]["haloes"]
        .as_array()
        .unwrap()
        .iter()
        .find(|x| x["address"] == h["halo"])
        .unwrap_or_else(|| panic!("home not in its cell: {h}"));
    assert_eq!(found["position_mpc"], h["position_mpc"]);
    assert_eq!(found["mass_msun"], h["mass_msun"]);
}

/// The galaxy chart's default, the stars, the sky and a reconstructed
/// origin all read the one home, never the median stand-in.
#[test]
fn every_campaign_galaxy_read_is_home() {
    let mut k = open("g3");
    let home = ask(&mut k, r#"{"op":"home_galaxy"}"#)["home"].clone();
    let addr = home["halo"].as_str().unwrap().to_string();
    let chart = ask(&mut k, r#"{"op":"galaxy_chart"}"#);
    assert_eq!(chart["chart"]["halo"], addr, "{}", chart["chart"]["home"]);
    assert_eq!(chart["chart"]["home"]["halo"], addr);
    let named = ask(&mut k, r#"{"op":"galaxy_chart","halo":"home"}"#);
    assert_eq!(named["chart"]["halo"], addr);
    // A reference galaxy is still there by name, and is not home.
    let dwarf = ask(&mut k, r#"{"op":"galaxy_chart","halo":"dwarf"}"#);
    assert!(dwarf["chart"]["home"].is_null());
    let cell = ask(&mut k, r#"{"op":"star_cell"}"#);
    assert_eq!(cell["cell"]["galaxy"], addr, "{cell}");
    let sky = ask(
        &mut k,
        r#"{"op":"sky_from_here","limit_mag":9,"stars":false}"#,
    );
    let walk = &sky["sky"]["galaxy_walk"];
    assert_eq!(walk["home_mpc"], home["position_mpc"], "{walk}");
    let gals = sky["sky"]["galaxies"].as_array().unwrap();
    assert!(!gals.is_empty());
    for g in gals {
        let a = g["address"].as_str().unwrap_or("");
        assert_ne!(a, addr, "our own halo in our own sky");
        assert!(!a.contains(STAND_IN), "{a}");
    }
    let text = serde_json::to_string(&[&chart, &cell, &sky]).unwrap();
    assert!(!text.contains(STAND_IN), "a stand-in place leaked");
}

/// "Make this home": the GM's choice is a record fact every read follows,
/// and clearing it returns to the web's choice.
#[test]
fn the_gm_can_make_another_galaxy_home_and_undo_it() {
    let mut k = open("g3");
    let web = ask(&mut k, r#"{"op":"home_galaxy"}"#)["home"].clone();
    let cell = ask(&mut k, r#"{"op":"halo_cell","cell":[0,0,0]}"#);
    let other = cell["cell"]["haloes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|h| h["address"].as_str().unwrap().to_string())
        .find(|a| *a != web["halo"].as_str().unwrap())
        .unwrap();
    let set = ask(&mut k, &format!(r#"{{"op":"set_home","halo":"{other}"}}"#));
    assert_eq!(set["type"], "home_set", "{set}");
    assert_eq!(set["home"]["halo"], other);
    assert_eq!(set["home"]["by"], "gm");
    assert_eq!(
        ask(&mut k, r#"{"op":"home_galaxy"}"#)["home"]["halo"],
        other
    );
    assert_eq!(
        ask(&mut k, r#"{"op":"galaxy_chart"}"#)["chart"]["halo"],
        other
    );
    assert_eq!(
        ask(&mut k, r#"{"op":"star_cell"}"#)["cell"]["galaxy"],
        other
    );
    let back = ask(&mut k, r#"{"op":"set_home"}"#);
    assert_eq!(back["home"]["halo"], web["halo"], "{back}");
    assert_eq!(back["home"]["by"], "web");
    // Not a catalogued halo of this universe: refused, nothing written.
    for bad in [
        "u:g3/halo:milky-way-like",
        "u:other/cell:0.0.0/halo:0",
        "u:g3/cell:0.0.0/halo:99999",
    ] {
        let r = ask(&mut k, &format!(r#"{{"op":"set_home","halo":"{bad}"}}"#));
        assert_eq!(r["type"], "error", "{bad}: {r}");
    }
    assert_eq!(
        ask(&mut k, r#"{"op":"home_galaxy"}"#)["home"]["halo"],
        web["halo"]
    );
}

/// What must hold for any universe's real default home (the coordinator's
/// condition with the reference-galaxy calibration checks, 26 Nov): the
/// wish's own bounds, a whole finite history with stars formed in it, and
/// a sky whose other galaxies are lit. The two bugs GZ12 found (a NaN that
/// wiped an early-forming galaxy, merger-brought stars left dark) fail here.
#[test]
fn every_default_home_is_a_whole_lit_galaxy_within_its_wish() {
    for seed in ["g3", "gz12", "sse4", "alpha", "beta", "gamma"] {
        let mut k = open(seed);
        let h = ask(&mut k, r#"{"op":"home_galaxy"}"#)["home"].clone();
        assert_eq!(h["fits_wish"], true, "{seed}: {h}");
        let mass = h["mass_msun"].as_f64().unwrap();
        assert!((5.0e11..=2.0e12).contains(&mass), "{seed}: {mass}");
        assert_eq!(h["neighbour"]["bound"], true, "{seed}");
        assert!(
            h["neighbour"]["distance_mpc"].as_f64().unwrap() <= 1.0,
            "{seed}"
        );
        let c = ask(&mut k, r#"{"op":"galaxy_chart"}"#)["chart"].clone();
        let hist = c["history"].as_array().unwrap();
        for s in hist {
            for f in ["stars", "gas", "sfr"] {
                assert!(
                    s[f].as_f64().is_some_and(f64::is_finite),
                    "{seed}: {f} at {}",
                    s["t_gyr"]
                );
            }
        }
        assert!(
            hist.iter().any(|s| s["sfr"].as_f64().unwrap() > 0.1),
            "{seed}: no star formation"
        );
        let stars = c["now"]["stars"].as_f64().unwrap();
        assert!(stars > 1.0e9 && stars < 2.0e11, "{seed}: {stars}");
        // Last major merger at least the wished 8 Gyr ago.
        let now = c["now_gyr"].as_f64().unwrap();
        for m in c["mergers"].as_array().unwrap() {
            if m["ratio"].as_f64().unwrap() >= 0.25 {
                assert!(now - m["t_gyr"].as_f64().unwrap() >= 8.0, "{seed}: {m}");
            }
        }
        let sky = ask(
            &mut k,
            r#"{"op":"sky_from_here","limit_mag":9,"bands":["V"],"stars":false}"#,
        );
        let gals = sky["sky"]["galaxies"].as_array().unwrap().clone();
        assert!(!gals.is_empty(), "{seed}: an empty sky");
        for g in &gals {
            let v = g["magnitudes"][0]["m_vega"].as_f64();
            assert!(v.is_some_and(f64::is_finite), "{seed}: {}", g["address"]);
            let m = g["stellar_mass_msun"].as_f64().unwrap_or(0.0);
            let abs_v = g["absolute"][0]["m_vega"].as_f64().unwrap();
            // Lit: no galaxy darker than M/L_V 10 (old populations read 3 to
            // 6; the unlit merger-brought stars read 13).
            let l_v = sse_core::math::pow(10.0, (4.83 - abs_v) / 2.5);
            assert!(
                m / l_v < 10.0,
                "{seed}: M/L_V {} for {}",
                m / l_v,
                g["address"]
            );
        }
    }
}

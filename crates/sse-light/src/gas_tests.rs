//! Light through gas and colour to the eye (ND2).

use crate::colour::colour_of;
use crate::gas::*;
use crate::planck;
use crate::table::LightTable;

fn sheet() -> sse_reference::Reference {
    sse_reference::Reference::base()
}

fn eye() -> crate::table::Eye {
    LightTable::base().eye.expect("base light pack has the eye")
}

fn black_body(t: f64) -> crate::colour::Colour {
    let e = eye();
    let light: Vec<f64> = e.wavelengths_m().iter().map(|l| planck(*l, t)).collect();
    colour_of(&e, &light).unwrap()
}

fn near(a: [u8; 3], b: [u8; 3], tol: i32, what: &str) {
    for k in 0..3 {
        let d = a[k] as i32 - b[k] as i32;
        assert!(d.abs() <= tol, "{what}: {a:?} against {b:?}");
    }
}

#[test]
fn black_bodies_have_the_papers_colours() {
    // Hollands et al. 2026 section 2.4 (new-physics demo design 2.4): a
    // Sun-like 5800 K model is #FFF1EC and a pure hydrogen 10 000 K white
    // dwarf (200, 218, 255). Ours are bare black bodies (no lines, no limb),
    // so the check is to within a few steps of 255.
    let sun = black_body(5800.0);
    near(sun.srgb, [255, 241, 236], 2, "5800 K");
    assert!(!sun.outside_screen);
    let wd = black_body(10_000.0);
    near(wd.srgb, [200, 218, 255], 6, "10 000 K");
    // Cooler is redder, hotter bluer.
    let red = black_body(3000.0);
    assert!(red.srgb[0] == 255 && red.srgb[2] < 120, "{:?}", red.srgb);
    assert_eq!(black_body(5800.0).hex, sun.hex, "same bits every time");
}

fn band(id: &str, from_nm: f64, to_nm: f64, sigma: f64) -> Feature {
    Feature::Table {
        id: id.into(),
        lambda_nm: vec![from_nm, to_nm],
        sigma: Values {
            v: vec![sigma, sigma],
            unit: "m2".into(),
        },
        at_k: 300.0,
        t_exponent: 0.0,
        valid_k: Some([100.0, 400.0]),
        src: "test".into(),
        note: String::new(),
        check: None,
    }
}

fn gas(id: &str, features: Option<Vec<Feature>>) -> GasOptics {
    GasOptics {
        gas: id.into(),
        particle_mass_kg: 2.0 * 1.66e-27,
        features,
        polarisability_m3: None,
        ionisation_j: None,
    }
}

fn slab(gases: &[(&str, f64)], t: f64, path: f64) -> Layer {
    Layer {
        t_k: t,
        path_m: path,
        gases: gases
            .iter()
            .map(|(g, n)| Amount {
                gas: (*g).into(),
                number_density_m3: *n,
            })
            .collect(),
    }
}

#[test]
fn a_measured_band_dims_only_where_it_was_measured() {
    let g = [gas("x", Some(vec![band("x.red", 600.0, 700.0, 1e-26)]))];
    let wl = [550e-9, 650e-9, 750e-9];
    let p = Passage {
        address: "t".into(),
        layers: vec![slab(&[("x", 1e25)], 300.0, 10.0)],
        path_factor: 2.0,
    };
    let r = pass(&p, &g, &wl, &[1.0; 3], &sheet()).unwrap();
    // τ = σ N X = 1e-26 · 1e26 · 2 = 2 inside the band, nothing outside.
    assert!((r.optical_depth[1] - 2.0).abs() < 1e-12);
    assert!((r.light_out[1] - sse_core::math::exp(-2.0)).abs() < 1e-12);
    assert_eq!((r.light_out[0], r.light_out[2]), (1.0, 1.0));
    assert!(r
        .outside_measured
        .iter()
        .any(|s| s.contains("600 to 700 nm")));
    // Past its temperature range it is still used, and said.
    let hot = Passage {
        layers: vec![slab(&[("x", 1e25)], 500.0, 10.0)],
        ..p
    };
    let r = pass(&hot, &g, &wl, &[1.0; 3], &sheet()).unwrap();
    assert!(r
        .outside_measured
        .iter()
        .any(|s| s.contains("used at 500 K")));
}

#[test]
fn gases_with_no_rows_are_said_to_be_clear() {
    let g = [gas("a", None), gas("b", Some(vec![]))];
    let p = Passage {
        address: "t".into(),
        layers: vec![slab(&[("a", 1e25), ("b", 1e25)], 300.0, 10.0)],
        path_factor: 1.0,
    };
    let r = pass(&p, &g, &[500e-9], &[3.0], &sheet()).unwrap();
    assert_eq!(r.light_out, vec![3.0]);
    assert!(r.not_looked_up[0].starts_with("a:"));
    assert!(r.none_known[0].starts_with("b:"));
}

fn sodium_like(cw: f64) -> GasOptics {
    GasOptics {
        gas: "na".into(),
        particle_mass_kg: 23.0 * 1.66e-27,
        features: Some(vec![Feature::Line {
            id: "na.d".into(),
            lambda_nm: 589.3,
            f: 0.961,
            lower_ev: 0.0,
            gamma_rad_s: 6.2e7,
            broadening: Some(Broadening {
                perturber: "any".into(),
                gamma_per_n: Scalar {
                    v: cw,
                    unit: "m3 s-1".into(),
                },
                t_exponent: 0.3,
                at_k: 5000.0,
            }),
            valid_k: None,
            src: "test".into(),
            note: String::new(),
            check: None,
        }]),
        polarisability_m3: None,
        ionisation_j: None,
    }
}

#[test]
fn a_line_holds_its_strength_and_crowding_widens_it() {
    // Optically thin: the optical depth summed over frequency is the line's
    // strength times the column, whatever its width.
    let c = 299_792_458.0;
    let n = 1e10;
    let path = 1.0;
    let fine: Vec<f64> = (0..8001)
        .map(|i| 589.2e-9 + 0.2e-9 * i as f64 / 8000.0)
        .collect();
    let wide: Vec<f64> = (0..2001)
        .map(|i| 579.3e-9 + 20e-9 * i as f64 / 2000.0)
        .collect();
    let run = |cw: f64, nh: f64, wl: &[f64]| {
        let p = Passage {
            address: "t".into(),
            layers: vec![slab(&[("na", n), ("he", nh)], 4000.0, path)],
            path_factor: 1.0,
        };
        let g = [sodium_like(cw), gas("he", Some(vec![]))];
        pass(&p, &g, wl, &vec![1.0; wl.len()], &sheet()).unwrap()
    };
    let wl = &fine;
    let thin = run(0.0, 0.0, wl);
    let mut sum = 0.0;
    for k in 1..wl.len() {
        let dnu = c / wl[k - 1] - c / wl[k];
        sum += 0.5 * (thin.optical_depth[k] + thin.optical_depth[k - 1]) * dnu;
    }
    let s =
        crate::colour::run("sse.gas.line_strength", &[0.961, 0.0, 589.3e-9, 4000.0]).unwrap()[0];
    assert!(
        (sum / (s * n * path) - 1.0).abs() < 0.02,
        "sum {sum} against {}",
        s * n * path
    );
    // A dense perturbing gas spreads the line: the far wing deepens and the
    // centre gets shallower.
    let thin = run(0.0, 0.0, &wide);
    let crowded = run(1e-15, 1e25, &wide);
    let wl = &wide;
    let far = wl
        .iter()
        .position(|l| (l - 584.3e-9).abs() < 3e-12)
        .unwrap();
    let centre = wl
        .iter()
        .position(|l| (l - 589.3e-9).abs() < 3e-12)
        .unwrap();
    assert!(crowded.optical_depth[far] > 100.0 * thin.optical_depth[far]);
    assert!(crowded.optical_depth[centre] < thin.optical_depth[centre]);
}

#[test]
fn colliding_pairs_grow_with_density_squared() {
    let pair = Feature::Pair {
        id: "h2-h2".into(),
        partner: "h2".into(),
        lambda_nm: vec![500.0, 3000.0],
        k: Rows {
            v: vec![vec![1e-56, 1e-56], vec![2e-56, 2e-56]],
            unit: "m5".into(),
        },
        at_k: vec![100.0, 300.0],
        valid_k: None,
        src: "test".into(),
        note: String::new(),
        check: None,
    };
    let g = [gas("h2", Some(vec![pair]))];
    let at = |n: f64, t: f64| {
        let p = Passage {
            address: "t".into(),
            layers: vec![slab(&[("h2", n)], t, 1e3)],
            path_factor: 1.0,
        };
        pass(&p, &g, &[1e-6], &[1.0], &sheet()).unwrap()
    };
    let one = at(1e25, 200.0).optical_depth[0];
    // k halfway between the rows: 1.5e-56 · 1e25 · 1e25 · 1e3.
    assert!((one / 1.5e-3 - 1.0).abs() < 1e-12, "{one}");
    assert!((at(2e25, 200.0).optical_depth[0] / one - 4.0).abs() < 1e-12);
    let cold = at(1e25, 50.0);
    assert!(cold
        .outside_measured
        .iter()
        .any(|s| s.contains("coldest row")));
}

#[test]
fn red_absorbing_gas_turns_reflected_sunlight_blue_green() {
    // The shape of a methane giant (ND2 acceptance, with the real rows in
    // part 2): a gas that absorbs in the red and near infrared over a deep
    // path leaves reflected sunlight blue-green.
    let e = eye();
    let wl = e.wavelengths_m();
    let sun: Vec<f64> = wl.iter().map(|l| planck(*l, 5772.0)).collect();
    let g = [gas("m", Some(vec![band("m.red", 600.0, 780.0, 2e-27)]))];
    let p = Passage {
        address: "giant".into(),
        layers: vec![slab(&[("m", 1e25)], 80.0, 1e2)],
        path_factor: 2.0,
    };
    let r = pass(&p, &g, &wl, &sun, &sheet()).unwrap();
    let before = colour_of(&e, &sun).unwrap();
    let after = colour_of(&e, &r.light_out).unwrap();
    assert!(before.srgb[0] > before.srgb[2]);
    assert!(
        after.srgb[2] > after.srgb[0] && after.srgb[1] > after.srgb[0],
        "{:?}",
        after.srgb
    );
    assert!(r.why[0].contains("m.red"));
}

#[test]
fn methane_from_the_sheet_turns_a_giant_blue_green() {
    // ND2 acceptance on the real rows (Karkoschka 1994): sunlight down to a
    // cloud deck and back through methane. A few km-amagat leaves it pale
    // green-blue, more leaves it the blue of Uranus and Neptune.
    let s = sheet();
    let ch4 = GasOptics::from_sheet(&s, "ch4").unwrap();
    assert!(ch4
        .features
        .as_ref()
        .is_some_and(|f| f[0].id() == "ch4.visible-bands"));
    let e = eye();
    let wl = e.wavelengths_m();
    let sun: Vec<f64> = wl.iter().map(|l| planck(*l, 5772.0)).collect();
    let km_amagat = 2.686_78e24;
    let seen = |km: f64| {
        let p = Passage {
            address: "giant".into(),
            layers: vec![slab(&[("ch4", km * km_amagat)], 120.0, 1.0)],
            path_factor: 2.0,
        };
        let r = pass(&p, std::slice::from_ref(&ch4), &wl, &sun, &s).unwrap();
        (colour_of(&e, &r.light_out).unwrap(), r)
    };
    let (pale, _) = seen(3.0);
    assert!(
        pale.srgb[2] >= pale.srgb[0] && pale.srgb[1] > pale.srgb[0],
        "{:?}",
        pale.srgb
    );
    let (deep, r) = seen(30.0);
    assert!(deep.srgb[2] == 255 && deep.srgb[0] < 180, "{:?}", deep.srgb);
    assert!(deep.srgb[1] > deep.srgb[0]);
    // The strongest band in the eye's range is at 725 nm.
    assert!(
        (r.features[0].at_m - 725e-9).abs() < 3e-9,
        "{:?}",
        r.features[0]
    );
    // The eye sees from 380 nm; the table starts at 410 nm, and says so.
    assert_eq!(r.outside_measured.len(), 1, "{:?}", r.outside_measured);
    assert!(r.outside_measured[0].contains("410 to 1000 nm"));
}

#[test]
fn sodium_and_potassium_lines_from_the_sheet() {
    let s = sheet();
    for (gas, d2, d1) in [("na", "na.d2", "na.d1"), ("k", "k.d2", "k.d1")] {
        let g = GasOptics::from_sheet(&s, gas).unwrap();
        let f = g.features.as_ref().unwrap();
        let strength = |id: &str| match f.iter().find(|x| x.id() == id).unwrap() {
            Feature::Line { f, .. } => *f,
            _ => panic!("{id} is a line"),
        };
        // D2 is twice D1 (statistical weights 4 and 2).
        assert!((strength(d2) / strength(d1) - 2.0).abs() < 0.02, "{gas}");
    }
    // A thin vapour dims light at 589 nm and not at 550 nm.
    let na = GasOptics::from_sheet(&s, "na").unwrap();
    let p = Passage {
        address: "t".into(),
        layers: vec![slab(&[("na", 1e16)], 1000.0, 1.0)],
        path_factor: 1.0,
    };
    let r = pass(&p, &[na], &[550e-9, 588.995e-9], &[1.0, 1.0], &s).unwrap();
    assert!(r.light_out[0] > 0.999_999, "{:?}", r.light_out);
    assert!(r.light_out[1] < 0.5, "{:?}", r.light_out);
}

/// ND2 part 2a: a line with no measured widening row is widened by its
/// neighbours by van der Waals forces, worked out from the sheet: the
/// atom's ionisation energy and each neighbour's polarisability.
#[test]
fn sodium_crowded_by_helium_widens_by_van_der_waals() {
    let s = sheet();
    let na = GasOptics::from_sheet(&s, "na").unwrap();
    let he = GasOptics::from_sheet(&s, "he").unwrap();
    assert!((na.ionisation_j.unwrap() / s.constant("e").unwrap() - 5.139076).abs() < 1e-9);
    assert!((he.polarisability_m3.unwrap() - 0.208e-30).abs() < 1e-40);
    // Unsöld's C6 for sodium D2 against helium, by hand.
    let (e, eps0, a0, h, c) = (
        s.constant("e").unwrap(),
        s.constant("eps0").unwrap(),
        s.constant("a0").unwrap(),
        s.constant("h").unwrap(),
        s.constant("c").unwrap(),
    );
    let ry = h * c * s.constant("R_inf").unwrap();
    let ion = na.ionisation_j.unwrap();
    let eu = h * c / 588.995e-9;
    let c6 = e * e / (4.0 * sse_core::math::PI * eps0) * 0.208e-30 * a0 * a0 * 2.5 * ry * ry / h
        * (1.0 / ((ion - eu) * (ion - eu)) - 1.0 / (ion * ion));
    let w = crate::colour::run(
        "sse.gas.van_der_waals_widening",
        &[
            ion,
            0.0,
            588.995e-9,
            6000.0,
            na.particle_mass_kg,
            he.particle_mass_kg,
            0.208e-30,
        ],
    )
    .unwrap();
    assert!((w[1] / c6 - 1.0).abs() < 1e-12, "{} {}", w[1], c6);
    // About 7e-33 cm⁶/s in the old units: Unsöld's 2e-32 for hydrogen
    // scaled by helium's smaller polarisability (0.208 against 0.667 Å³).
    assert!(w[1] * 1e12 > 5e-33 && w[1] * 1e12 < 9e-33, "{}", w[1]);
    // A thin sodium vapour in a dense helium air: the wing 1 nm off the
    // line darkens many times over once the helium is there.
    let wl = [589.995e-9];
    let pass_with = |n_he: f64| {
        let p = Passage {
            address: "t".into(),
            layers: vec![slab(&[("na", 1e18), ("he", n_he)], 6000.0, 1e3)],
            path_factor: 1.0,
        };
        pass(&p, &[na.clone(), he.clone()], &wl, &[1.0], &s).unwrap()
    };
    let thin = pass_with(0.0);
    let dense = pass_with(1e27);
    assert!(
        dense.optical_depth[0] > 10.0 * thin.optical_depth[0],
        "{} {}",
        dense.optical_depth[0],
        thin.optical_depth[0]
    );
    assert!(
        dense.why.iter().any(|w| w.contains("van der Waals")),
        "{:?}",
        dense.why
    );
    assert!(dense.references.iter().any(|r| r.text.contains("Unsöld")));
    // With nothing to work it out from, it is said.
    assert!(
        thin.stand_ins.iter().any(|w| w.contains("no widening row")),
        "{:?}",
        thin.stand_ins
    );
}

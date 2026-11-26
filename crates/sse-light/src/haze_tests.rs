//! Light off layers of particles: a material that absorbs blue more than red
//! sends back an orange light, a material that absorbs nothing sends back
//! white, a thicker layer hides the floor, and the blocks give the papers'
//! limits.
use crate::colour::run;
use crate::haze::{reflect_with, IndexTable, Particles};
use crate::table::LightTable;

fn eye() -> crate::table::Eye {
    LightTable::base().eye.expect("base light pack has the eye")
}

/// Sunlight-like: a 5772 K black body at the eye's wavelengths.
fn light() -> Vec<f64> {
    eye()
        .wavelengths_m()
        .iter()
        .map(|l| crate::planck(*l, 5772.0))
        .collect()
}

fn layer(material: &str, tau: f64) -> Particles {
    Particles {
        material: material.into(),
        radius_m: 0.5e-6,
        optical_depth: tau,
        at_nm: 550.0,
        asymmetry: 0.7,
    }
}

/// A made-up material shaped like tholin: k falls tenfold from blue to red.
fn browning() -> IndexTable {
    IndexTable {
        lambda_nm: vec![400.0, 500.0, 600.0, 700.0],
        n: vec![1.7, 1.68, 1.66, 1.65],
        k: vec![0.08, 0.03, 0.012, 0.006],
        src: "test".into(),
    }
}

#[test]
fn a_material_absorbing_blue_sends_back_orange() {
    let r = reflect_with(
        &eye(),
        &[layer("browning", 8.0)],
        &[Some(browning())],
        0.0,
        &light(),
    )
    .unwrap();
    let [red, green, blue] = r.reflectance_rgb;
    assert!(red > green && green > blue, "{:?}", r.reflectance_rgb);
    assert!(r.colour.hex.len() == 7);
    // The table covers 400 to 700 nm; the eye runs wider, so that is said.
    assert!(
        r.said.iter().any(|s| s.contains("nearest measured")),
        "{:?}",
        r.said
    );
}

#[test]
fn a_material_not_looked_up_is_grey_and_said() {
    let r = reflect_with(&eye(), &[layer("unlooked", 8.0)], &[None], 0.0, &light()).unwrap();
    let [a, b, c] = r.reflectance_rgb;
    assert!(
        (a - b).abs() < 0.01 && (b - c).abs() < 0.01,
        "{:?}",
        r.reflectance_rgb
    );
    assert!(r.said.iter().any(|s| s.contains("no refractive index")));
}

#[test]
fn a_thicker_layer_hides_the_floor() {
    let thin = reflect_with(&eye(), &[layer("x", 0.5)], &[None], 0.9, &light()).unwrap();
    let thick = reflect_with(
        &eye(),
        &[layer("x", 0.5), layer("browning", 20.0)],
        &[None, Some(browning())],
        0.9,
        &light(),
    )
    .unwrap();
    // The bright floor shows through the thin grey layer; the thick brown layer hides it.
    assert!(thin.reflectance_rgb[2] > 0.5, "{:?}", thin.reflectance_rgb);
    assert!(
        thick.reflectance_rgb[2] < thin.reflectance_rgb[2] - 0.3,
        "{:?}",
        thick.reflectance_rgb
    );
}

#[test]
fn the_blocks_meet_their_limits() {
    // A conservative slab: Bohren 1987's (1 - g)τ / (2 + (1 - g)τ), and R + T = 1.
    let s = run("sse.light.slab_reflectance", &[3.0, 1.0, 0.7]).unwrap();
    assert!((s[0] - 0.9 / 2.9).abs() < 1e-12, "{s:?}");
    assert!((s[0] + s[1] - 1.0).abs() < 1e-12);
    // A thick absorbing slab: nothing gets through.
    let s = run("sse.light.slab_reflectance", &[200.0, 0.9, 0.7]).unwrap();
    assert!(s[1] < 1e-6 && s[0] > 0.1 && s[0] < 0.5, "{s:?}");
    // A large, non-absorbing sphere: extinction tends to twice its cross-section, albedo one.
    let a = run("sse.light.particle_albedo", &[50e-6, 550e-9, 1.33, 0.0]).unwrap();
    assert!((a[0] - 2.0).abs() < 0.05 && a[2] == 1.0, "{a:?}");
    // A tiny absorption: the series and the closed form agree where they meet.
    let near = |k: f64| run("sse.light.particle_albedo", &[1e-6, 550e-9, 1.5, k]).unwrap()[1];
    let x = 2.0 * sse_core::math::PI * 1e-6 / 550e-9;
    let k_switch = 0.001 / (4.0 * x);
    assert!((near(k_switch * 0.999) / near(k_switch * 1.001) - 1.0).abs() < 0.01);
}

#[test]
fn index_tables_are_checked_and_read_between_rows() {
    let t = browning();
    let (n, k, out) = t.at(450.0);
    assert!((n - 1.69).abs() < 1e-12 && (k - 0.055).abs() < 1e-12 && !out);
    assert!(t.at(380.0).2 && t.at(750.0).2);
    let mut bad = browning();
    bad.k[1] = -1.0;
    assert!(bad.check().is_err());
}

/// Venus's cloud droplets from the sheet's measured acid (MP36, the
/// `sulfuric-acid-75` row and its source): sulphuric acid barely absorbs
/// visible light, so a thick deck of 1.05 um droplets (the renderer's
/// cloud-rows.json) is white. Venus's
/// pale yellow comes from its unknown ultraviolet absorber, which the sheet
/// does not hold yet, not from the droplets.
#[test]
fn venus_acid_droplets_from_the_sheet_make_a_white_deck() {
    let sheet = sse_reference::Reference::base();
    let r = crate::haze::reflect(
        &sheet,
        &eye(),
        &[Particles {
            material: "sulfuric-acid-75".into(),
            radius_m: 1.05e-6,
            optical_depth: 1.0e4,
            at_nm: 550.0,
            asymmetry: 0.85,
        }],
        0.0,
        &light(),
    )
    .unwrap();
    let [red, green, blue] = r.reflectance_rgb;
    for c in [red, green, blue] {
        assert!(c > 0.99, "{:?}", r.reflectance_rgb);
    }
    assert!((red - blue).abs() < 0.005, "{:?}", r.reflectance_rgb);
    assert!(
        r.references.iter().any(|s| s == "palmer-williams1975"),
        "{:?}",
        r.references
    );
}

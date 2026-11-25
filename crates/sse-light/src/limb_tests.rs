//! ND2 part 2b: the limb law.

use crate::limb::*;

fn eye() -> crate::table::Eye {
    crate::table::LightTable::base().eye.unwrap()
}

#[test]
fn a_grey_star_darkens_to_the_edge_by_eddingtons_law() {
    // Summed over all wavelengths the share is T(μ)⁴ / T(1)⁴ = (2 + 3μ) / 5,
    // Eddington's law: 0.4 at the very edge.
    for mu in [0.0, 0.3, 0.7] {
        let t = crate::colour::run("sse.light.limb_temperature", &[5772.0, mu]).unwrap()[0];
        let t1 = crate::colour::run("sse.light.limb_temperature", &[5772.0, 1.0]).unwrap()[0];
        let r = (t / t1) * (t / t1) * (t / t1) * (t / t1);
        assert!((r - (2.0 + 3.0 * mu) / 5.0).abs() < 1e-12, "{mu}: {r}");
    }
    // The centre is the effective temperature at τ = 2/3 looked at
    // square on: (3/4 · 5/3)^(1/4) = 1.057 times it.
    let p = glowing(5772.0, &[1.0, 0.5, 0.1], &[4.0e-7, 8.0e-7], &eye()).unwrap();
    assert!((p[0].temperature_k.unwrap() / 5772.0 - 1.0574).abs() < 1e-3);
    assert!((p[0].share[0] - 1.0).abs() < 1e-12);
    // Per colour: the edge loses more blue than red.
    for q in &p[1..] {
        assert!(q.share[0] < q.share[1], "{:?}", q.share);
    }
    // And looks redder to the eye.
    let rb = |c: &crate::colour::Colour| c.linear_srgb[0] / c.linear_srgb[2].max(1e-9);
    assert!(rb(p[2].colour.as_ref().unwrap()) > rb(p[0].colour.as_ref().unwrap()));
}

#[test]
fn a_matt_ball_is_lambert_and_a_hazy_one_stays_bright() {
    // k = 1: brightness is μ₀ whatever the viewing angle (Lambert).
    let l = lit(&[1.0], 0.8, &[1.0, 0.5, 0.1]).unwrap();
    for q in &l {
        assert!((q.share[0] - 0.8).abs() < 1e-12);
    }
    // Seen from the sun's direction (μ₀ = μ), a matt ball fades as μ and a
    // hazy one (k = 0.5) as √μ: brighter at the edge.
    let mus = [0.2];
    let matt = lit(&[1.0], 0.2, &mus).unwrap()[0].share[0];
    let hazy = lit(&[0.5], 0.2, &mus).unwrap()[0].share[0];
    assert!(hazy > matt);
    assert!(lit(&[], 0.5, &mus).is_err());
    assert_eq!(references().len(), 2);
}

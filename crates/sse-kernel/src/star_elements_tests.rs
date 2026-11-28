//! MP7 checks (matter pack design, MP7): the Sun returns solar values for
//! every element; a star's own inheritance moves the elements its sources
//! made and no others; two stars with different r-process histories get
//! different gold.

use crate::star_elements::{source_scales, star_element_ratios};
use sse_formation::inherit::Inheritance;

fn sheet() -> sse_reference::Reference {
    let p = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../packs/base-reference/pack.json");
    let m = sse_pack::PackManifest::from_json(&std::fs::read_to_string(p).unwrap()).unwrap();
    sse_reference::Reference::from_packs(&[m]).unwrap()
}

fn sun() -> Inheritance {
    Inheritance {
        feh: 0.0,
        z: sse_formation::hurley::Z_SUN,
        age_gyr: 4.57,
        born_gyr: 9.2,
        c_o: 0.55,
        mg_si: 1.2,
        al26_rel: 1.0,
        cluster_members: 1000.0,
        fuv_g0: 1000.0,
        alpha_fe: 0.0,
        r_process_rel: 1.0,
        k40_rel: 1.0,
        system_mass: Some(1.0),
        population: None,
    }
}

#[test]
fn the_sun_gets_the_suns_elements() {
    let r = star_element_ratios(&sun(), &sheet()).unwrap();
    assert!(r.len() > 80, "{}", r.len());
    for (el, x) in &r {
        // Human-made elements had no share in the Sun's cloud to scale.
        let human = sheet()
            .row("elements", el)
            .and_then(|row| row["origin"]["shares"].get("human_made").cloned())
            .is_some();
        if !human {
            assert!((x - 1.0).abs() < 1e-9, "{el}: {x}");
        }
    }
}

#[test]
fn an_old_halo_star_has_more_oxygen_than_iron() {
    // [Fe/H] = -1 and [alpha/Fe] = +0.3: oxygen (all from massive stars) at
    // 10^-0.7, iron at 10^-1 exactly, hydrogen untouched.
    let mut h = sun();
    h.feh = -1.0;
    h.alpha_fe = 0.3;
    h.r_process_rel = 0.1;
    let r = star_element_ratios(&h, &sheet()).unwrap();
    assert!(
        (r["o"] - sse_core::math::pow(10.0, -0.7)).abs() < 1e-12,
        "{}",
        r["o"]
    );
    assert!((r["fe"] - 0.1).abs() < 1e-12, "{}", r["fe"]);
    assert!((r["h"] - 1.0).abs() < 1e-12);
    assert!(r["o"] / r["fe"] > 1.99, "alpha-rich");
    // Past [alpha/Fe] of log 2 the white dwarfs would owe iron back: floored at none.
    h.alpha_fe = 0.5;
    let s = source_scales(&h, &crate::star_elements::origin_shares(&sheet())).unwrap();
    assert_eq!(s["white_dwarfs"], 0.0);
}

#[test]
fn more_neutron_star_mergers_more_gold() {
    let mut a = sun();
    let mut b = sun();
    a.r_process_rel = 0.5;
    b.r_process_rel = 3.0;
    let (ra, rb) = (
        star_element_ratios(&a, &sheet()).unwrap(),
        star_element_ratios(&b, &sheet()).unwrap(),
    );
    assert!(rb["au"] > 4.0 * ra["au"], "{} {}", ra["au"], rb["au"]);
    // Iron and oxygen do not come from mergers and do not move.
    assert_eq!(ra["fe"], rb["fe"]);
    assert_eq!(ra["o"], rb["o"]);
}

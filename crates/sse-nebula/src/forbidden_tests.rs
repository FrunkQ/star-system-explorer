use crate::forbidden::*;
use crate::*;

const RSUN: f64 = 6.957e8;

fn line(d: &LightDescription, id: &str) -> f64 {
    d.lines
        .iter()
        .filter(|l| l.line == id)
        .map(|l| l.power_w)
        .sum()
}

fn zone(d: &LightDescription, id: &str) -> sse_light::Zone {
    d.lines
        .iter()
        .find(|l| l.line == id)
        .and_then(|l| l.zone)
        .unwrap_or_else(|| panic!("{id} has no zone"))
}

fn glow(teff_k: f64, radius_rsun: f64, n_cm3: f64) -> Descriptor {
    let p = GlowInput {
        address: "u:t/sys:0/nebula:0".into(),
        cluster: "u:t/cluster:0".into(),
        stars: vec![HotStar {
            address: "u:t/sys:0/star:0".into(),
            teff_k,
            radius_m: radius_rsun * RSUN,
            born_s: 0.0,
            dies_s: 1e15,
        }],
        cloud_h_m3: n_cm3 * 1e6,
    };
    star_forming_glow(&Seed::from_text("forbidden"), &p, 1e13)
        .unwrap()
        .unwrap()
}

/// The sulphur doublet is the textbook density gauge: 6716/6731 sits near 1.4
/// in thin gas and falls toward 0.44 in dense gas, where knocks empty the
/// longer-lived level before it shines (Osterbrock and Ferland figure 5.8).
#[test]
fn the_sulphur_pair_measures_density() {
    let ratio = |n: f64| {
        let d = glow(38_000.0, 10.0, n);
        line(&d.light, "sii") / line(&d.light, "sii-6731")
    };
    let thin = ratio(10.0);
    let dense = ratio(1e5);
    eprintln!(
        "sulphur ratio: thin {thin:.3}, 1e3 {:.3}, dense {dense:.3}",
        ratio(1e3)
    );
    assert!(thin > 1.35 && thin < 1.55, "thin {thin}");
    assert!(dense > 0.4 && dense < 0.6, "dense {dense}");
    assert!(ratio(1e3) < thin && ratio(1e3) > dense);
}

/// An O star's glow: oxygen's doubly ionised layer in the middle, sulphur's
/// once-ionised layer at the rim; line strengths against H-beta of the order
/// seen in H II regions (Orion's [O III] 5007/Hβ is about 3, its [S II] pair
/// against Hα a few per cent to a tenth).
#[test]
fn an_o_stars_glow_has_a_teal_heart_and_a_red_rim() {
    let d = glow(40_000.0, 10.0, 100.0);
    let hb = line(&d.light, "hb");
    let ha = line(&d.light, "ha");
    let o3 = line(&d.light, "oiii") / hb;
    let s2 = (line(&d.light, "sii") + line(&d.light, "sii-6731")) / ha;
    let ratio = line(&d.light, "oiii") / line(&d.light, "oiii-4959");
    // Three photons to one from the same level, each 5007 photon a little
    // less energetic.
    assert!((ratio - 3.0 * 495.891 / 500.684).abs() < 1e-9, "{ratio}");
    assert!(o3 > 0.3 && o3 < 10.0, "[O III]/Hβ {o3}");
    assert!(s2 > 0.01 && s2 < 0.5, "[S II]/Hα {s2}");
    let (zo, zs) = (zone(&d.light, "oiii"), zone(&d.light, "sii"));
    eprintln!("O star: [O III]/Hb {o3:.3}, [S II]/Ha {s2:.3}, zones {zo:?} {zs:?}");
    assert!(zo.inner < 0.2, "{zo:?}");
    assert!(zo.outer < 1.0 && zs.outer == 1.0 && zs.inner > 0.0);
    assert!(zs.inner >= zo.outer - 1e-12, "sulphur sits outside oxygen");
    assert!(d.why.iter().any(|w| w.contains("[O III] 500.7 nm")));
}

/// A cooler star can barely strip oxygen twice: [O III] is a few per cent of
/// H-beta at most, and sulphur shines.
#[test]
fn a_cool_b_star_gives_no_oxygen_light() {
    let d = glow(25_000.0, 6.0, 100.0);
    let o3 = line(&d.light, "oiii") / line(&d.light, "hb");
    eprintln!("B star: [O III]/Hb {o3:.4}");
    assert!(o3 < 0.1, "[O III]/Hβ {o3}");
    assert!(line(&d.light, "sii") > 0.0);
}

/// A planetary nebula round a hundred-thousand-kelvin core: oxygen glows
/// strongly (5007/Hβ of several to twenty in real ones), and the hottest
/// light strips oxygen a third time nearest the star.
#[test]
fn a_planetary_nebula_glows_in_oxygen() {
    let s = PnSettings::default();
    let p = PnInput {
        address: "u:t/sys:0/star:0/nebula:0".into(),
        star: "u:t/sys:0/star:0".into(),
        died_at_s: 1e17,
        core_teff_k: 100_000.0,
        core_radius_m: 0.05 * RSUN,
        envelope_kg: 0.3 * MSUN_KG,
        ambient_h_m3: 1e6,
        companion: false,
        fast_spin: false,
        axis: None,
    };
    let d = planetary_nebula(
        &Seed::from_text("forbidden"),
        &p,
        &s,
        p.died_at_s + 5000.0 * YEAR_S,
    )
    .unwrap()
    .unwrap();
    let o3 = line(&d.light, "oiii") / line(&d.light, "hb");
    assert!(o3 > 2.0 && o3 < 30.0, "[O III]/Hβ {o3}");
    let z = zone(&d.light, "oiii");
    eprintln!(
        "PN: [O III]/Hb {o3:.3}, zone {z:?}, [S II] {:e}",
        line(&d.light, "sii")
    );
    assert!(z.inner > 0.0 && z.inner < z.outer, "{z:?}");
}

#[test]
fn abundances_come_from_the_sheet() {
    let o = per_hydrogen("o").unwrap();
    assert!(o > 4e-4 && o < 8e-4, "{o}");
    assert!(per_hydrogen("unobtainium").is_err());
}

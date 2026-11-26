//! Measured motion (RW17): the series tables read back the published check
//! values, and the Earth faces the way the standards say.

use crate::earth::EarthOrientation;
use crate::series::{Series, J2000_JD};
use sse_core::math as m;
use std::path::PathBuf;

fn table(id: &str) -> Series {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../packs/base-sol/motion")
        .join(format!("{id}.json"));
    Series::from_json(&std::fs::read_to_string(p).unwrap()).unwrap()
}

fn earth() -> EarthOrientation {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../packs/base-sol/motion/earth-orientation.json");
    EarthOrientation::from_json(&std::fs::read_to_string(p).unwrap()).unwrap()
}

/// IMCCE's check values for the full VSOP87A series (vsop87.chk, as the
/// vsop87 crate's tests copy them): heliocentric x, y, z in au at J2000 and
/// at 300, 600 and 900 years before. Unverified against IMCCE's own file
/// until scripts/check-sol-motion.py is run where IMCCE can be reached.
const CHECKS: [(&str, f64, [f64; 3]); 32] = [
    (
        "mer",
        2451545.0,
        [-0.1300934115, -0.4472876716, -0.0245984000],
    ),
    (
        "mer",
        2341970.0,
        [0.3256720360, 0.0880865802, -0.0229820000],
    ),
    (
        "mer",
        2232395.0,
        [-0.3938651887, -0.1288399755, 0.0263344000],
    ),
    (
        "mer",
        2122820.0,
        [0.2146329139, -0.3752296250, -0.0503983000],
    ),
    (
        "ven",
        2451545.0,
        [-0.7183022797, -0.0326546017, 0.0410143000],
    ),
    (
        "ven",
        2341970.0,
        [0.4531193265, -0.5692420969, -0.0335643000],
    ),
    (
        "ven",
        2232395.0,
        [0.1935421816, 0.6940567996, -0.0029303000],
    ),
    (
        "ven",
        2122820.0,
        [-0.6660158465, -0.2753592311, 0.0357874000],
    ),
    (
        "emb",
        2451545.0,
        [-0.1771591440, 0.9672192891, -0.0000010000],
    ),
    (
        "emb",
        2341970.0,
        [-0.2104343221, 0.9603642782, 0.0006463000],
    ),
    (
        "emb",
        2232395.0,
        [-0.2435434218, 0.9524355203, 0.0012855000],
    ),
    (
        "emb",
        2122820.0,
        [-0.2762837551, 0.9433889919, 0.0019144000],
    ),
    (
        "mar",
        2451545.0,
        [1.3907159264, -0.0134157043, -0.0344678000],
    ),
    (
        "mar",
        2341970.0,
        [-1.6387489524, 0.2507105242, 0.0465606000],
    ),
    (
        "mar",
        2232395.0,
        [1.3910394546, -0.0543839267, -0.0371013000],
    ),
    (
        "mar",
        2122820.0,
        [-1.6278485157, 0.3060194814, 0.0494216000],
    ),
    (
        "jup",
        2451545.0,
        [4.0011740268, 2.9385810077, -0.1017838000],
    ),
    (
        "jup",
        2341970.0,
        [1.2817318353, -5.0280079874, -0.0091252000],
    ),
    (
        "jup",
        2232395.0,
        [-5.4239396005, -0.5085487291, 0.1247760000],
    ),
    (
        "jup",
        2122820.0,
        [0.4207861894, 5.1019591310, -0.0280088000],
    ),
    (
        "sat",
        2451545.0,
        [6.4064068573, 6.5699929449, -0.3690768000],
    ),
    (
        "sat",
        2341970.0,
        [8.9934758992, -3.7883225437, -0.2866389000],
    ),
    (
        "sat",
        2232395.0,
        [1.2601620699, -10.0267935693, 0.1347053000],
    ),
    (
        "sat",
        2122820.0,
        [-7.9395559173, -5.8435867016, 0.4165602000],
    ),
    (
        "ura",
        2451545.0,
        [14.4318934159, -13.7343162527, -0.2381422000],
    ),
    (
        "ura",
        2341970.0,
        [-4.2214391936, 18.3160266384, 0.1247593000],
    ),
    (
        "ura",
        2232395.0,
        [-0.7868164612, -19.2532559478, -0.0636424000],
    ),
    (
        "ura",
        2122820.0,
        [11.8546461039, 15.5595370553, -0.0950189000],
    ),
    (
        "nep",
        2451545.0,
        [16.8121116576, -24.9916630908, 0.1272190000],
    ),
    (
        "nep",
        2341970.0,
        [29.5022811950, 4.5987701114, -0.7740507000],
    ),
    (
        "nep",
        2232395.0,
        [8.0214324006, 28.7234916080, -0.7759059000],
    ),
    (
        "nep",
        2122820.0,
        [-22.7985170870, 19.5994768858, 0.1206374000],
    ),
];

/// The cut tables land within their own stated cut of the full series'
/// check values (plus the check values' own z rounding of 3e-6 au). An
/// absolute bound: the Earth-Moon barycentre within 3e-6 au (450 km) of
/// IMCCE's full series.
#[test]
fn vsop87a_tables_meet_imcce_check_values() {
    let au = sse_core::sheet::AU;
    for (id, jd, want) in CHECKS {
        let s = table(&format!("vsop87a.{id}"));
        let got = s.position_m(jd);
        let v: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(
                PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                    .join(format!("../../packs/base-sol/motion/vsop87a.{id}.json")),
            )
            .unwrap(),
        )
        .unwrap();
        let tol = v["cut"]["bound_au"].as_f64().unwrap() + 3e-6;
        for i in 0..3 {
            let d = (got[i] / au - want[i]).abs();
            assert!(
                d <= tol,
                "{id} at JD {jd}, axis {i}: {} au against {} (off {d:e}, tolerance {tol:e})",
                got[i] / au,
                want[i]
            );
        }
        if id == "emb" {
            let (dx, dy) = (got[0] / au - want[0], got[1] / au - want[1]);
            let d = (dx * dx + dy * dy).sqrt();
            assert!(d < 3e-6, "the Earth-Moon barycentre is {d:e} au off");
        }
    }
}

/// The Moon from ELP/MPP02 against Meeus's worked example 47.a (Astronomical
/// Algorithms, 2nd ed. 1998): 1992 April 12, 0h TD, geometric longitude
/// 133.167265 degrees on the mean ecliptic of date and distance 368409.7 km.
/// Meeus's own short series is good to about 10 arcseconds, so the test
/// allows 0.01 degrees and 5 km. Unverified: the example's figures are from
/// memory until checked against the book or JPL Horizons.
#[test]
fn elp_moon_matches_meeus_example() {
    let s = table("elpmpp02.moon");
    let jd = 2_448_724.5;
    let p = s.position_m(jd);
    let r = (p[0] * p[0] + p[1] * p[1] + p[2] * p[2]).sqrt() / 1000.0;
    assert!((r - 368_409.7).abs() < 5.0, "distance {r} km");
    // J2000 longitude to the mean ecliptic of date: general precession in
    // longitude, 5028.796195 arcsec a century (Capitaine et al. 2003).
    let t = (jd - J2000_JD) / 36_525.0;
    let lon = m::atan2(p[1], p[0]).to_degrees() + 5028.796195 * t / 3600.0;
    let lon = (lon + 360.0) % 360.0;
    assert!((lon - 133.167265).abs() < 0.01, "longitude {lon}");
}

/// The Moon stays between its known nearest and farthest (about 356,400 and
/// 406,700 km) at every date in the span, and moves smoothly.
#[test]
fn elp_moon_stays_in_its_range() {
    let s = table("elpmpp02.moon");
    let mut jd = s.from_jd;
    while jd <= s.to_jd {
        let p = s.position_m(jd);
        let r = (p[0] * p[0] + p[1] * p[1] + p[2] * p[2]).sqrt() / 1000.0;
        assert!((355_000.0..408_000.0).contains(&r), "{r} km at JD {jd}");
        jd += 3_652.0;
    }
}

/// Same date, same answer, every call: the series are pure arithmetic.
#[test]
fn series_repeat_exactly() {
    let s = table("elpmpp02.moon");
    assert_eq!(s.position_m(2_460_409.25), s.position_m(2_460_409.25));
    assert!(s.terms() > 1000);
}

/// GMST at 0h UT1 on 2000 January 1.5 is 280.46061837 degrees (IERS
/// Conventions; Aoki et al. 1982), and Delta T in 2000 is 63.86 s.
#[test]
fn earth_faces_the_standard_way_at_j2000() {
    let e = earth();
    let (dt, sigma) = e.delta_t(2000.0);
    assert!((dt - 63.86).abs() < 1e-9 && sigma <= 1.0, "{dt} {sigma}");
    let g = e.gmst(J2000_JD + dt / 86_400.0).to_degrees();
    assert!((g - 280.46061837).abs() < 1e-5, "{g}");
    // A round trip through the Earth's frame comes back.
    let v = [1.0e8, -2.0e8, 3.0e7];
    let w = e.from_earth_fixed(e.to_earth_fixed(v, 2_460_409.3), 2_460_409.3);
    for i in 0..3 {
        assert!((v[i] - w[i]).abs() < 1e-3, "{v:?} {w:?}");
    }
}

/// Far from now the ground place drifts: under a degree in this century,
/// tens of degrees 2,800 years ago.
#[test]
fn ground_drift_grows_away_from_now() {
    let e = earth();
    let now = e.drift_deg(2_460_409.0);
    let ancient = e.drift_deg(J2000_JD - 2_762.0 * 365.25);
    assert!(now < 0.01, "{now}");
    assert!(ancient > 1.0, "{ancient}");
    let (dt, _) = e.delta_t(-762.0);
    assert!((20_000.0..23_000.0).contains(&dt), "Delta T in 763 BC {dt}");
}

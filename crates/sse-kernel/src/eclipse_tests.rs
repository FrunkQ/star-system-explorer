//! Eclipses on time (RW17; real-catalogues design 6A.2): the catalogued
//! Solar System, entered as the start sheet's door enters it, puts the
//! Moon's shadow on the right place at the right minute.
//!
//! Everything here is read from `orbits_at`, the call the drawing reads
//! (EC1 places shadows from it), so these are the shadows the app shows.
//!
//! The published figures are NASA's (Espenak, eclipse.gsfc.nasa.gov):
//! greatest eclipse in TD (TT), gamma (how far the shadow's axis passes from
//! Earth's centre, in Earth radii, plus to the north) and the place under
//! greatest eclipse. **Unverified**: they are copied from memory until
//! `scripts/check-sol-motion.py` reads them at source on a machine that can
//! reach NASA. The tolerances are this package's stated ones: 10 s in time,
//! 0.002 in gamma, 0.15 degrees on the ground. On 26 Nov 2026 every eclipse
//! here came within 2 s, 0.0006 and 0.05 degrees.

use crate::sol_tests::open_sol;
use sse_time::earth::EarthOrientation;

const SOL: &str = "u:home/sys:sol";
const EARTH: &str = "u:home/sys:sol/body:2";
const MOON: &str = "u:home/sys:sol/body:2/body:0";
/// The universe second of J2000.0 in base-sol (`time.j2000_s`).
const J2000_S: i64 = 63_115_200_000;

type V = [f64; 3];
/// Sun, Earth and Moon from the system barycentre, and whether all were measured.
type Row = (V, V, V, bool);
fn sub(a: V, b: V) -> V {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}
fn add(a: V, b: V) -> V {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}
fn mul(a: V, k: f64) -> V {
    [a[0] * k, a[1] * k, a[2] * k]
}
fn dot(a: V, b: V) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}
fn unit(a: V) -> V {
    mul(a, 1.0 / dot(a, a).sqrt())
}

std::thread_local! {
    /// The anchor this test's Sky was entered with (each test is its own thread).
    static ANCHOR: std::cell::Cell<i64> = const { std::cell::Cell::new(J2000_S) };
}
fn at_s(jd: f64) -> i64 {
    crate::motion::at_s(jd, ANCHOR.with(|a| a.get()))
}
fn jd(at_s: i64) -> f64 {
    crate::motion::jd(at_s, ANCHOR.with(|a| a.get()))
}
/// The universe's present to the second (base-cosmology `present_s`, time
/// design ruling 12A.1): where J2000.0 sits for a universe whose home is Sol.
fn present_s() -> i64 {
    let j: serde_json::Value =
        serde_json::from_str(include_str!("../../../packs/base-cosmology/pack.json")).unwrap();
    j["provides"]["other"]["cosmology"]["params"]["present_s"]
        .as_i64()
        .unwrap()
}

struct Sky {
    k: crate::Kernel,
    /// For a lunar eclipse the light passes Earth first and reaches the
    /// Moon about 1.3 s later, so it is Earth that is read earlier.
    lunar: bool,
}

/// Sun, Earth and Moon from the system's barycentre at each time, and
/// whether every one of them was measured.
impl Sky {
    fn new() -> Sky {
        Sky::anchored(J2000_S)
    }
    /// The Solar System with J2000.0 at universe second `anchor`.
    fn anchored(anchor: i64) -> Sky {
        ANCHOR.with(|a| a.set(anchor));
        let k = if anchor == J2000_S {
            open_sol(&[])
        } else {
            open_sol(&[(
                "sun",
                crate::motion::ANCHOR_FIELD,
                serde_json::json!(anchor),
            )])
        };
        Sky { k, lunar: false }
    }
    /// The Moon is read where it was when the light that reaches Earth's
    /// centre now passed it (about 1.3 s earlier): in the system's frame
    /// the Moon moves about 30 km/s with Earth, so that is some 40 km, or
    /// some 37 s of eclipse time. The Sun's 8 minutes matter far less: it
    /// moves only metres a second in this frame.
    fn at(&mut self, times: &[i64]) -> Vec<Row> {
        let both: Vec<i64> = times.iter().flat_map(|&t| [t, t - 1]).collect();
        let rows = self.raw(&both);
        (0..times.len())
            .map(|i| {
                let (sun, earth, moon, measured) = rows[2 * i];
                let (_, earth_before, moon_before, _) = rows[2 * i + 1];
                let d = sub(moon, earth);
                let lt = dot(d, d).sqrt() / sse_core::sheet::C;
                if self.lunar {
                    (
                        sun,
                        sub(earth, mul(sub(earth, earth_before), lt)),
                        moon,
                        measured,
                    )
                } else {
                    (
                        sun,
                        earth,
                        sub(moon, mul(sub(moon, moon_before), lt)),
                        measured,
                    )
                }
            })
            .collect()
    }
    fn raw(&mut self, times: &[i64]) -> Vec<Row> {
        assert!(times.len() <= 64);
        let r: serde_json::Value = serde_json::from_str(&self.k.handle_json(
            &serde_json::json!({"op":"orbits_at","system":SOL,"at_s":times}).to_string(),
        ))
        .unwrap();
        assert_eq!(r["type"], "orbits_placed", "{r}");
        let bodies = r["bodies"].as_array().unwrap();
        let get = |a: &str| {
            bodies
                .iter()
                .find(|b| b["address"] == a)
                .unwrap_or_else(|| panic!("no {a}"))
                .clone()
        };
        let (e, mo) = (get(EARTH), get(MOON));
        let v = |b: &serde_json::Value, i: usize| -> V {
            let x = &b["at"][i]["sys_m"];
            [
                x[0].as_f64().unwrap(),
                x[1].as_f64().unwrap(),
                x[2].as_f64().unwrap(),
            ]
        };
        (0..times.len())
            .map(|i| {
                let earth = v(&e, i);
                let moon = v(&mo, i);
                // The Sun is the root: its place is minus the pull of the rest,
                // read from its children's offsets.
                let sun = sub(earth, {
                    let x = &e["at"][i]["r_m"];
                    [
                        x[0].as_f64().unwrap(),
                        x[1].as_f64().unwrap(),
                        x[2].as_f64().unwrap(),
                    ]
                });
                let measured =
                    e["at"][i]["motion"] == "measured" && mo["at"][i]["motion"] == "measured";
                (sun, earth, moon, measured)
            })
            .collect()
    }
}

/// How far a point `p` is from the line through `a` along `d` (unit), and
/// the nearest point of the line.
fn off_line(p: V, a: V, d: V) -> (f64, V) {
    let q = add(a, mul(d, dot(sub(p, a), d)));
    (dot(sub(p, q), sub(p, q)).sqrt(), q)
}

/// The time (universe s) at which `f` is least within `around` +- `half` s:
/// a coarse scan then golden-section refinement to a second.
fn least(sky: &mut Sky, around: i64, half: i64, f: &dyn Fn(&Row) -> f64) -> i64 {
    let step = (2 * half) / 30;
    let times: Vec<i64> = (0..=30).map(|i| around - half + i * step).collect();
    let rows = sky.at(&times);
    let (i, _) = rows
        .iter()
        .map(f)
        .enumerate()
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .unwrap();
    let (mut lo, mut hi) = (times[i] - step, times[i] + step);
    while hi - lo > 2 {
        let (a, b) = (lo + (hi - lo) / 3, hi - (hi - lo) / 3);
        let r = sky.at(&[a, b]);
        if f(&r[0]) < f(&r[1]) {
            hi = b;
        } else {
            lo = a;
        }
    }
    let last: Vec<i64> = (lo..=hi).collect();
    let r = sky.at(&last);
    let (i, _) = r
        .iter()
        .map(f)
        .enumerate()
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .unwrap();
    last[i]
}

fn re() -> f64 {
    sse_core::sheet::R_EARTH_EQUATORIAL
}
fn axis_ratio() -> f64 {
    sse_core::sheet::R_EARTH_POLAR / sse_core::sheet::R_EARTH_EQUATORIAL
}

struct Solar {
    at_s: i64,
    gamma: f64,
    lat: f64,
    lon: f64,
    measured: bool,
}

/// Greatest eclipse of the solar eclipse near `guess_jd` (TT): when the
/// shadow's axis passes nearest Earth's centre, gamma, and the ground place
/// under the axis then.
fn solar(sky: &mut Sky, guess_jd: f64) -> Solar {
    let dist = |r: &Row| off_line(r.1, r.2, unit(sub(r.2, r.0))).0;
    let t = least(sky, at_s(guess_jd), 6 * 3600, &dist);
    let (sun, earth, moon, measured) = sky.at(&[t])[0];
    let d = unit(sub(moon, sun));
    let (off, q) = off_line(earth, moon, d);
    let eo = crate::motion::earth();
    let jt = jd(t);
    let north = eo.to_earth_fixed(sub(q, earth), jt)[2];
    let gamma = off / re() * north.signum();
    // The axis meets the ellipsoid: squash z to a sphere, meet it, unsquash.
    let k = axis_ratio();
    let squash = |v: V| [v[0], v[1], v[2] / k];
    let m = squash(eo.to_earth_fixed(sub(moon, earth), jt));
    let dd = unit(squash(eo.to_earth_fixed(d, jt)));
    let b = dot(m, dd);
    let c = dot(m, m) - re() * re();
    let s = -b - (b * b - c).max(0.0).sqrt();
    let p = add(m, mul(dd, s));
    let g = EarthOrientation::ground([p[0], p[1], p[2] * k], k);
    Solar {
        at_s: t,
        gamma,
        lat: g.lat_deg,
        lon: g.lon_deg,
        measured,
    }
}

/// Greatest eclipse of a lunar eclipse: when the Moon passes nearest the
/// axis of Earth's shadow, and gamma in Earth radii.
fn lunar(sky: &mut Sky, guess_jd: f64) -> (i64, f64, bool) {
    let dist = |r: &Row| off_line(r.2, r.1, unit(sub(r.1, r.0))).0;
    let t = least(sky, at_s(guess_jd), 6 * 3600, &dist);
    let r = sky.at(&[t])[0];
    (t, dist(&r) / re(), r.3)
}

/// TD time of day of a universe second, as hh:mm:ss, for messages.
fn hms(t: i64, day_jd: f64) -> String {
    let s = (t - at_s(day_jd)) as f64;
    format!(
        "{:02}:{:02}:{:02}",
        (s / 3600.0) as i64,
        ((s % 3600.0) / 60.0) as i64,
        (s % 60.0) as i64
    )
}

fn check_solar(day_jd: f64, td: (i64, i64, i64), gamma: f64, lat: f64, lon: f64, what: &str) {
    check_solar_on(J2000_S, day_jd, td, gamma, lat, lon, what)
}

fn check_solar_on(
    anchor: i64,
    day_jd: f64,
    td: (i64, i64, i64),
    gamma: f64,
    lat: f64,
    lon: f64,
    what: &str,
) {
    let mut sky = Sky::anchored(anchor);
    let want = at_s(day_jd) + td.0 * 3600 + td.1 * 60 + td.2;
    let got = solar(&mut sky, jd(want));
    assert!(
        got.measured,
        "{what}: the Sun, Earth and Moon should all be measured"
    );
    let msg = format!(
        "{what}: greatest eclipse {} TD (published {:02}:{:02}:{:02}), gamma {:.4} (published {gamma}), at {:.2}, {:.2} (published {lat}, {lon})",
        hms(got.at_s, day_jd),
        td.0,
        td.1,
        td.2,
        got.gamma,
        got.lat,
        got.lon
    );
    eprintln!("{msg}");
    assert!((got.at_s - want).abs() <= 10, "{msg}");
    assert!((got.gamma - gamma).abs() <= 0.002, "{msg}");
    assert!((got.lat - lat).abs() <= 0.15, "{msg}");
    assert!((got.lon - lon).abs() <= 0.15, "{msg}");
}

/// The Great American Eclipse, 21 August 2017: greatest eclipse 18:26:40 TD
/// in Kentucky, 36 58'N 87 40'W, gamma 0.4367.
#[test]
fn total_solar_eclipse_2017_aug_21() {
    check_solar(
        2_457_986.5,
        (18, 26, 40),
        0.4367,
        36.97,
        -87.67,
        "2017 Aug 21",
    );
}

/// 8 April 2024: greatest eclipse 18:18:29 TD near Nazas, Durango, 25 17'N
/// 104 08'W, gamma 0.3431.
#[test]
fn total_solar_eclipse_2024_apr_8() {
    check_solar(
        2_460_408.5,
        (18, 18, 29),
        0.3431,
        25.29,
        -104.13,
        "2024 Apr 8",
    );
}

/// 12 August 2026: greatest eclipse 17:47:06 TD off western Iceland,
/// 65 10'N 25 14'W, gamma 0.8977.
#[test]
fn total_solar_eclipse_2026_aug_12() {
    check_solar(
        2_461_264.5,
        (17, 47, 6),
        0.8977,
        65.17,
        -25.23,
        "2026 Aug 12",
    );
}

/// The total lunar eclipse of 8 November 2022: greatest eclipse 11:00:22 TD
/// (10:59:11 UT), gamma 0.2570.
#[test]
fn total_lunar_eclipse_2022_nov_8() {
    let mut sky = Sky::new();
    sky.lunar = true;
    let day = 2_459_891.5;
    let want = at_s(day) + 11 * 3600 + 22;
    let (t, gamma, measured) = lunar(&mut sky, jd(want));
    let msg = format!(
        "2022 Nov 8 lunar: greatest {} TD, gamma {gamma:.4}",
        hms(t, day)
    );
    eprintln!("{msg}");
    assert!(measured, "{msg}");
    assert!((t - want).abs() <= 10, "{msg}");
    assert!((gamma - 0.2570).abs() <= 0.002, "{msg}");
}

/// An ancient eclipse: the one in the Assyrian eponym list for the year of
/// Bur-Sagale, 15 June 763 BC (Julian calendar; year -762), total across
/// northern Assyria. Inside the series' span its time is still good, so the
/// shadow passes near Earth's centre on that day; the place is uncertain by
/// degrees, because Delta T was not measured then, and the drift says so.
/// (Its date anchors ancient Near Eastern chronology. On 26 Nov 2026 the
/// kernel put greatest eclipse at 14:07 TD, about 08:12 UT with Delta T
/// near 21,300 s, gamma 0.27. Not yet compared with NASA's canon.)
#[test]
fn ancient_eclipse_763_bc_with_place_drift() {
    let mut sky = Sky::new();
    let day = 1_442_902.5;
    let got = solar(&mut sky, day + 0.5);
    let msg = format!(
        "763 BC: greatest {} TD, gamma {:.4}, at {:.2}, {:.2}",
        hms(got.at_s, day),
        got.gamma,
        got.lat,
        got.lon
    );
    eprintln!("{msg}");
    assert!(got.measured, "{msg}");
    assert!(
        got.at_s >= at_s(day) && got.at_s < at_s(day + 1.0),
        "on 15 June: {msg}"
    );
    assert!(got.gamma.abs() < 0.9, "central: {msg}");
    let drift = crate::motion::earth().drift_deg(jd(got.at_s));
    eprintln!("ground place drift {drift:.2} degrees");
    assert!(drift > 1.0, "the place drifts by degrees: {drift}");
}

/// Outside the series' span the bodies hand over to plain orbits fitted at
/// the edge, labelled plausible, and stay near where the series left them.
#[test]
fn outside_the_span_is_plausible() {
    let mut sky = Sky::new();
    let edge = at_s(2_451_545.0 + 3000.0 * 365.25);
    let r = sky.at(&[edge - 60, edge + 60]);
    assert!(r[0].3, "inside the span: measured");
    assert!(!r[1].3, "past the span: not measured");
    // In two minutes Earth moves about 3,600 km and the Moon about 120 km
    // round it: the hand-over adds no jump beyond that.
    let earth = dot(sub(r[1].1, r[0].1), sub(r[1].1, r[0].1)).sqrt();
    let (m0, m1) = (sub(r[0].2, r[0].1), sub(r[1].2, r[1].1));
    let moon = dot(sub(m1, m0), sub(m1, m0)).sqrt();
    assert!(earth < 5.0e6, "Earth jumped {earth} m at the hand-over");
    assert!(
        moon < 3.0e5,
        "the Moon jumped {moon} m round Earth at the hand-over"
    );
}

/// On the universe's own axis (time design 12A.1, the condition for moving
/// Sol there): with J2000.0 at the present, 13.8 Gyr of whole seconds from
/// the origin, the 2024 eclipse still lands on its published second, and 64
/// one-second steps give 64 distinct places for a measured body (Earth's
/// group) and for a plain one (a moon with no series). A double at 4.35e17 s
/// would give two.
#[test]
fn eclipse_on_time_at_the_universes_present() {
    let present = present_s();
    check_solar_on(
        present,
        2_460_408.5,
        (18, 18, 29),
        0.3431,
        25.29,
        -104.13,
        "2024 Apr 8, J2000 at the present",
    );
    let mut sky = Sky::anchored(present);
    let t0 = at_s(2_460_408.5);
    let times: Vec<i64> = (0..64).map(|i| t0 + i).collect();
    let r: serde_json::Value =
        serde_json::from_str(&sky.k.handle_json(
            &serde_json::json!({"op":"orbits_at","system":SOL,"at_s":times}).to_string(),
        ))
        .unwrap();
    let bodies = r["bodies"].as_array().unwrap();
    let distinct = |b: &serde_json::Value| {
        let mut seen: Vec<String> = b["at"]
            .as_array()
            .unwrap()
            .iter()
            .map(|a| a["r_m"].to_string())
            .collect();
        seen.sort();
        seen.dedup();
        seen.len()
    };
    let earth = bodies.iter().find(|b| b["address"] == EARTH).unwrap();
    assert_eq!(earth["at"][0]["motion"], "measured");
    assert_eq!(distinct(earth), 64, "Earth's group");
    let plain = bodies
        .iter()
        .find(|b| b["placed"] == true && b["motion"].is_null() && b["parent"] != SOL)
        .expect("a moon with no series");
    assert_eq!(distinct(plain), 64, "{}", plain["address"]);
}

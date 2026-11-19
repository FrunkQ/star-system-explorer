use crate::*;
use sse_core::math as m;

fn t() -> LightTable {
    LightTable::base()
}

fn blackbody(t_k: f64, area: f64) -> LightDescription {
    let mut d = LightDescription::new("test");
    d.parts.push(Part::Thermal {
        t_k,
        area_m2: area,
        emissivity: 1.0,
        cause: "test".into(),
    });
    d
}

fn rel(a: f64, b: f64) -> f64 {
    (a - b).abs() / b.abs()
}

const MSUN: f64 = 1.988_41e30;
const YEAR: f64 = sse_core::quantity::consts::JULIAN_YEAR;

#[test]
fn base_table_parses_and_checks() {
    let t = t();
    assert_eq!(t.bands.len(), 9);
    assert_eq!(t.bands[0].id, "gamma");
    assert!(t.line("ha").is_some() && t.line("hi-21").is_some());
    let mut bad = t.clone();
    bad.bands[1].from_m *= 1.1;
    assert!(bad.check().is_err());
}

/// The band set covers the whole of a blackbody's light from 3 K to 10⁷ K:
/// summed bands equal σT⁴ per unit area (Stefan-Boltzmann).
#[test]
fn bands_close_on_stefan_boltzmann() {
    let t = t();
    for temp in [3.0, 30.0, 300.0, 5772.0, 1e5, 1e7] {
        let p = publish(&blackbody(temp, 1.0), &t);
        let want = SIGMA * temp * temp * temp * temp;
        assert!(
            rel(p.total_w, want) < 1e-6,
            "{temp} K: {} vs {want}",
            p.total_w
        );
    }
}

/// The Sun as a 5772 K blackbody of radius 695,700 km gives its nominal
/// luminosity (IAU 2015 B3: 3.828e26 W), and its λL_λ peaks near 0.64 µm (Wien,
/// for power per log wavelength: λT = 3670 µm K).
#[test]
fn sun_luminosity_and_peak() {
    let t = t();
    let r = 6.957e8;
    let p = publish(&blackbody(5772.0, 4.0 * m::PI * r * r), &t);
    assert!(rel(p.total_w, 3.828e26) < 2e-3, "{}", p.total_w);
    let step = m::pow(10.0, 18.0 / 144.0);
    assert!(p.peak_m > 3.67e-3 / 5772.0 / step && p.peak_m < 3.67e-3 / 5772.0 * step);
    let vis = p
        .bands
        .iter()
        .find(|b| b.band == "visible")
        .unwrap()
        .power_w;
    // 43.775 % of a 5772 K blackbody's light is in 380 to 750 nm (independent
    // midpoint integration, 200,000 steps, in Python).
    assert!(
        rel(vis / p.total_w, 0.437_747) < 1e-4,
        "{}",
        vis / p.total_w
    );
}

#[test]
fn visible_slices_sum_to_visible() {
    let t = t();
    let p = publish(&blackbody(5772.0, 1.0), &t);
    let vis = p
        .bands
        .iter()
        .find(|b| b.band == "visible")
        .unwrap()
        .power_w;
    let s: f64 = p.visible_slices.iter().map(|b| b.power_w).sum();
    assert!(rel(s, vis) < 1e-6, "{s} vs {vis}");
}

#[test]
fn lines_land_in_their_band() {
    let t = t();
    let mut d = LightDescription::new("neb");
    d.lines.push(LineEmission {
        line: "ha".into(),
        power_w: 1e20,
        cause: "test".into(),
    });
    d.lines.push(LineEmission {
        line: "hi-21".into(),
        power_w: 1e10,
        cause: "test".into(),
    });
    let p = publish(&d, &t);
    let get = |id: &str| p.bands.iter().find(|b| b.band == id).unwrap().power_w;
    assert_eq!(get("visible"), 1e20);
    assert_eq!(get("radio"), 1e10);
    assert_eq!(p.visible_slices[2].power_w, 1e20);
    assert_eq!(p.total_w, 1e20 + 1e10);
}

#[test]
fn power_law_carries_its_power() {
    let t = t();
    for alpha in [-1.0, 0.0, 0.7, 1.0, 2.0] {
        let mut d = LightDescription::new("pl");
        d.parts.push(Part::PowerLaw {
            power_w: 1e11,
            from_m: 1e-2,
            to_m: 1e2,
            alpha,
            cause: "test".into(),
        });
        let p = publish(&d, &t);
        let inside = power_between(&d, 1e-2, 1e2, &t);
        assert!(rel(p.total_w, inside) < 1e-12);
        assert!(
            rel(power_between(&d, 1e-3, 1e3, &t), 1e11) < 1e-12,
            "alpha {alpha}"
        );
    }
}

/// A thin disc from r_in to far out radiates GMṀ/(2 r_in) (Shakura and Sunyaev
/// 1973; Frank, King and Raine, Accretion Power in Astrophysics, eq. 5.44).
#[test]
fn disc_luminosity_and_xray_peak() {
    let t = t();
    let mass = 10.0 * MSUN;
    let mdot = 1e-8 * MSUN / YEAR;
    let r_in = 6.0 * G * mass / (C * C);
    let r_out = 1e5 * r_in;
    let mut d = LightDescription::new("bh");
    d.parts.push(Part::Disc {
        mass_kg: mass,
        accretion_kg_s: mdot,
        r_inner_m: r_in,
        r_outer_m: r_out,
        cause: "test".into(),
    });
    let p = publish(&d, &t);
    let x = r_in / r_out;
    let want = G * mass * mdot / (2.0 * r_in) * (1.0 - 3.0 * x + 2.0 * x * m::sqrt(x));
    assert!(rel(p.total_w, want) < 0.05, "{} vs {want}", p.total_w);
    let brightest = p.bands.iter().fold(
        &p.bands[0],
        |a, b| if b.power_w > a.power_w { b } else { a },
    );
    assert_eq!(brightest.band, "xray");
}

#[test]
fn brightness_temperature_inverts_planck() {
    let t = t();
    for temp in [40.0, 124.0, 5772.0] {
        let d = blackbody(temp, 2.0);
        for (a, b) in [(5e-6, 2.5e-5), (2.5e-5, 3.5e-4), (3.8e-7, 7.5e-7)] {
            if power_between(&d, a, b, &t) > 0.0 {
                let tb = brightness_temperature(&d, a, b, 2.0, &t).unwrap();
                assert!(rel(tb, temp) < 1e-6, "{temp} K in {a}..{b}: {tb}");
            }
        }
    }
}

#[test]
fn planck_matches_rayleigh_jeans_and_wien_limits() {
    // Rayleigh-Jeans: B ≈ 2ckT/λ⁴ when hc/λkT ≪ 1.
    let (l, temp) = (1.0, 300.0);
    let rj = 2.0 * C * K_B * temp / (l * l * l * l);
    assert!(rel(planck(l, temp), rj) < 1e-4);
    assert_eq!(planck(1e-12, 300.0), 0.0);
    assert_eq!(planck(1e-6, 0.0), 0.0);
}

/// Every published number is the same on every device: a pinned set of
/// descriptions, published, must match the golden file byte for byte.
pub fn golden_value() -> serde_json::Value {
    let t = t();
    let r = 6.957e8;
    let sun = blackbody(5772.0, 4.0 * m::PI * r * r);
    let dust = blackbody(20.0, 1e30);
    let mass = 10.0 * MSUN;
    let r_in = 6.0 * G * mass / (C * C);
    let mut bh = LightDescription::new("bh");
    bh.parts.push(Part::Disc {
        mass_kg: mass,
        accretion_kg_s: 1e-8 * MSUN / YEAR,
        r_inner_m: r_in,
        r_outer_m: 1e5 * r_in,
        cause: "golden".into(),
    });
    let mut radio = LightDescription::new("radio");
    radio.parts.push(Part::PowerLaw {
        power_w: 1e11,
        from_m: 7.0,
        to_m: 300.0,
        alpha: 0.0,
        cause: "golden".into(),
    });
    radio.lines.push(LineEmission {
        line: "oi-5577".into(),
        power_w: 1e9,
        cause: "golden".into(),
    });
    let out: Vec<_> = [sun, dust, bh, radio]
        .iter()
        .map(|d| {
            let p = publish(d, &t);
            serde_json::json!({ "bands": p.bands, "slices": p.visible_slices, "total_w": p.total_w, "peak_m": p.peak_m,
                "samples_hash": sse_core::hash::ContentHash::of(&p.samples).unwrap().hex() })
        })
        .collect();
    serde_json::json!(out)
}

#[test]
fn golden() {
    let g = golden_value();
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../tests/golden/light.json");
    let text = serde_json::to_string_pretty(&g).unwrap() + "\n";
    if std::env::var("SSE_BLESS").is_ok() {
        std::fs::write(path, &text).unwrap();
    }
    let want = std::fs::read_to_string(path).expect("golden exists (bless with SSE_BLESS=1)");
    assert_eq!(text, want);
}

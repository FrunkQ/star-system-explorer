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

/// RK10: the band set is the sheet's Bands table (its light rows), read in
/// order of wavelength; a pack that gives its own list still overrides it.
#[test]
fn bands_come_from_the_sheet_unless_a_pack_gives_its_own() {
    let sheet = sse_reference::Reference::base();
    let from_sheet = crate::table::bands_from_sheet(&sheet).unwrap();
    assert_eq!(t().bands, from_sheet);
    assert_eq!(from_sheet.last().unwrap().id, "radio");
    assert_eq!(from_sheet.last().unwrap().to_m, 1.0e4);
    let mut v: serde_json::Value =
        serde_json::from_str(include_str!("../../../packs/base-light/pack.json")).unwrap();
    assert!(v["provides"]["other"]["light"].get("bands").is_none());
    v["provides"]["other"]["light"]["bands"] = serde_json::json!([
        {"id": "short", "name": "Short", "from_m": 1e-9, "to_m": 1e-6},
        {"id": "long", "name": "Long", "from_m": 1e-6, "to_m": 1.0}]);
    let p = sse_pack::PackManifest::from_json(&v.to_string()).unwrap();
    let own = crate::table::LightTable::from_pack(&p).unwrap();
    assert_eq!(own.bands.len(), 2);
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

// ---- O2: bodies' light from their conditions ----------------------------

mod o2 {
    use crate::emit::*;
    use crate::*;

    fn solar() -> Vec<BodyLight> {
        publish_all(&solar_system().bodies, Model::base()).unwrap()
    }

    fn by_name<'a>(all: &'a [BodyLight], n: &str) -> &'a BodyLight {
        all.iter().find(|b| b.name.as_deref() == Some(n)).unwrap()
    }

    fn band(b: &BodyLight, id: &str) -> f64 {
        b.light.bands.iter().find(|x| x.band == id).unwrap().power_w
    }

    /// Effective temperatures from the energy balance match those measured
    /// (NSSDC black-body temperatures; Voyager and Cassini for the giants).
    #[test]
    fn effective_temperatures_match_observed() {
        let all = solar();
        let fx = solar_system();
        let obs = fx.observed["effective_temperature_k"].as_object().unwrap();
        for (name, t) in obs {
            let got = by_name(&all, name).facts.effective_temperature_k;
            let want = t.as_f64().unwrap();
            assert!((got - want).abs() < 1.5, "{name}: {got:.2} K vs {want} K");
        }
        // The Sun's own luminosity gives its effective temperature, by the same block.
        let sun = by_name(&all, "Sun").facts.effective_temperature_k;
        assert!((sun - 5772.0).abs() < 2.0, "Sun {sun}");
    }

    /// Infrared brightness temperatures: in the thermal infrared a body is seen
    /// by its own heat at its effective temperature; in the near infrared and
    /// visible, by reflected sunlight.
    #[test]
    fn infrared_brightness_temperatures() {
        let all = solar();
        let t = LightTable::base();
        for name in ["Earth", "Mars", "Jupiter", "Saturn", "Neptune"] {
            let b = by_name(&all, name);
            let r = solar_system()
                .bodies
                .iter()
                .find(|c| c.name.as_deref() == Some(name))
                .unwrap()
                .radius_m;
            let area = 4.0 * sse_core::math::PI * r * r;
            let far = t.band("far-ir").unwrap();
            let tb = brightness_temperature(&b.light.description, far.from_m, far.to_m, area, &t)
                .unwrap();
            let te = b.facts.effective_temperature_k;
            assert!(
                (tb - te).abs() / te < 0.01,
                "{name}: far-IR {tb:.1} K vs {te:.1} K"
            );
            assert!(
                band(b, "visible")
                    > 1e3 * {
                        let only_own: f64 = b
                            .light
                            .description
                            .parts
                            .iter()
                            .filter(|p| matches!(p, Part::Thermal { .. }))
                            .map(|p| part_power(p, 3.8e-7, 7.5e-7, &t))
                            .sum();
                        only_own
                    },
                "{name} is seen by reflected light"
            );
        }
    }

    /// Auroral radio from the radio-magnetic Bode's law, within a factor of 3
    /// of the measured average powers; none without a magnetic field.
    #[test]
    fn auroral_radio_only_with_a_field() {
        let all = solar();
        let fx = solar_system();
        for (name, p) in fx.observed["auroral_radio_w"].as_object().unwrap() {
            let got = by_name(&all, name).facts.auroral_radio_w;
            let want = p.as_f64().unwrap();
            assert!(
                got / want < 3.0 && want / got < 3.0,
                "{name}: {got:.2e} W vs {want:.2e} W"
            );
            assert!(band(by_name(&all, name), "radio") > 0.0);
        }
        for name in ["Venus", "Mars", "Moon"] {
            let b = by_name(&all, name);
            assert_eq!(b.facts.auroral_radio_w, 0.0, "{name}");
            // Still faintly radio-bright by its own heat (the Rayleigh-Jeans tail), and nothing else.
            assert!(band(b, "radio") > 0.0, "{name}");
            assert!(
                !b.light
                    .description
                    .parts
                    .iter()
                    .any(|p| matches!(p, Part::PowerLaw { .. })),
                "{name}"
            );
        }
        // Jupiter is the loudest, and Earth's magnetopause is about 10 radii out.
        let j = by_name(&all, "Jupiter").facts.auroral_radio_w;
        assert!(all
            .iter()
            .filter(|b| b.name.as_deref() != Some("Sun"))
            .all(|b| b.facts.auroral_radio_w <= j));
        let e = by_name(&all, "Earth");
        let rmp = e.facts.magnetopause_radius_m / 6.378_137e6;
        assert!(rmp > 8.5 && rmp < 11.5, "{rmp}");
    }

    /// The same conditions with the field switched off lose their radio and
    /// nothing else.
    #[test]
    fn field_switch_changes_only_radio() {
        let mut c = solar_system()
            .bodies
            .into_iter()
            .find(|c| c.name.as_deref() == Some("Jupiter"))
            .unwrap();
        let with = publish_all(std::slice::from_ref(&c), Model::base()).unwrap();
        c.magnetic_moment_a_m2 = 0.0;
        let without = publish_all(&[c], Model::base()).unwrap();
        for (a, b) in with[0].light.bands.iter().zip(&without[0].light.bands) {
            if a.band == "radio" {
                // The difference is exactly the auroral radio (it all lies inside the radio band).
                let extra = a.power_w - b.power_w;
                assert!(
                    (extra - with[0].facts.auroral_radio_w).abs() < 1e-6 * extra,
                    "{extra}"
                );
            } else {
                assert_eq!(a.power_w, b.power_w, "{}", a.band);
            }
        }
    }

    /// Built things: a hot ship is bright in the mid infrared, a cold one is
    /// dim; a radar lights itself up in radio; sodium lamps put their light in
    /// the sodium line.
    #[test]
    fn built_things() {
        let sun = Source {
            address: "sun".into(),
            luminosity_w: 3.828e26,
            t_k: 5772.0,
            distance_m: 1.496e11,
        };
        let ship = |t_k: f64, radar: bool| Conditions {
            address: "ship".into(),
            radius_m: 20.0,
            bond_albedo: 0.2,
            lit_by: vec![sun.clone()],
            radiators: vec![Radiator {
                t_k,
                area_m2: 400.0,
            }],
            transmitters: if radar {
                vec![Transmitter {
                    frequency_hz: 9.4e9,
                    bandwidth_hz: 1e7,
                    power_w: 2e4,
                }]
            } else {
                vec![]
            },
            ..Default::default()
        };
        let all = publish_all(&[ship(600.0, true), ship(150.0, false)], Model::base()).unwrap();
        let hot = band(&all[0], "mid-ir");
        let cold = band(&all[1], "mid-ir");
        // The cold ship is dim but never zero: its hull still sits near the
        // equilibrium temperature in sunlight.
        assert!(hot > 2.0 * cold && cold > 0.0, "{hot} vs {cold}");
        assert!(band(&all[0], "radio") > 1e4 && band(&all[1], "radio") < 1.0);
        let city = Conditions {
            address: "city".into(),
            radius_m: 6.4e6,
            lamps: vec![Lamp {
                kind: LampKind::Sodium,
                power_w: 1e9,
            }],
            ..Default::default()
        };
        let p = publish_all(&[city], Model::base()).unwrap();
        assert_eq!(p[0].light.visible_slices[2].power_w, 1e9);
    }

    /// Every device gives the same bytes for the Solar System's light.
    #[test]
    fn golden_o2() {
        let all = solar();
        let g: Vec<_> = all
            .iter()
            .map(|b| {
                serde_json::json!({
                    "name": b.name, "facts": b.facts, "bands": b.light.bands, "total_w": b.light.total_w,
                    "hash": sse_core::hash::ContentHash::of(&b.light).unwrap().hex(),
                })
            })
            .collect();
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/golden/light-bodies.json"
        );
        let text = serde_json::to_string_pretty(&g).unwrap() + "\n";
        if std::env::var("SSE_BLESS").is_ok() {
            std::fs::write(path, &text).unwrap();
        }
        let want = std::fs::read_to_string(path).expect("golden exists (bless with SSE_BLESS=1)");
        assert_eq!(text, want);
    }
}

// ---- O7: shifts ---------------------------------------------------------

mod o7 {
    use crate::shift::*;
    use crate::*;

    fn rel(a: f64, b: f64) -> f64 {
        (a - b).abs() / b.abs()
    }

    fn star(t: f64) -> LightDescription {
        let mut d = LightDescription::new("s");
        d.parts.push(Part::Thermal {
            t_k: t,
            area_m2: 1e18,
            emissivity: 1.0,
            cause: "test".into(),
        });
        d
    }

    #[test]
    fn doppler_factor_special_cases() {
        let b = 0.6;
        let head_on = Shift::doppler(b, 1.0, false).factor;
        assert!(rel(head_on, sse_core::math::sqrt((1.0 + b) / (1.0 - b))) < 1e-12);
        let away = Shift::doppler(b, -1.0, false).factor;
        assert!(rel(away, sse_core::math::sqrt((1.0 - b) / (1.0 + b))) < 1e-12);
        // Side-on: only time dilation, D = 1/γ (the transverse Doppler effect).
        let side = Shift::doppler(b, 0.0, false).factor;
        assert!(rel(side, 0.8) < 1e-12);
    }

    /// A moving black body is seen as a black body at D times its temperature
    /// (the Planck function's own symmetry), and its total light rises as D⁴.
    #[test]
    fn moving_blackbody_is_hotter_blackbody() {
        let t = LightTable::base();
        let s = Shift::doppler(0.3, 1.0, false);
        let seen = publish_shifted(&star(5000.0), &s, &t);
        let hot = publish(&star(5000.0 * s.factor), &t);
        for (a, b) in seen.bands.iter().zip(&hot.bands) {
            if b.power_w > 1e-30 * hot.total_w {
                assert!(
                    rel(a.power_w, b.power_w) < 1e-6,
                    "{}: {} vs {}",
                    a.band,
                    a.power_w,
                    b.power_w
                );
            }
        }
        let rest = publish(&star(5000.0), &t).total_w;
        assert!(rel(seen.total_w, rest * sse_core::math::pow(s.factor, 4.0)) < 1e-6);
    }

    /// Lines move to λ₀/D and carry D^(p+1) of their power: Hα from a source
    /// approaching at 0.15 c lands in the green slice.
    #[test]
    fn lines_shift_between_slices() {
        let t = LightTable::base();
        let mut d = LightDescription::new("n");
        d.lines.push(LineEmission {
            line: "ha".into(),
            power_w: 1e20,
            cause: "test".into(),
        });
        let s = Shift::doppler(0.15, 1.0, false);
        let p = publish_shifted(&d, &s, &t);
        let at = s.line_at(656.281e-9);
        assert!(at > 4.9e-7 && at < 5.8e-7, "{at}");
        assert!(
            rel(
                p.visible_slices[1].power_w,
                1e20 * sse_core::math::pow(s.factor, 4.0)
            ) < 1e-12
        );
        assert_eq!(p.visible_slices[2].power_w, 0.0);
    }

    /// No visible change at ordinary speeds: at 30 km/s (Earth's orbital
    /// speed) the visible band changes by under 0.1 %.
    #[test]
    fn ordinary_speeds_change_nothing_visible() {
        let t = LightTable::base();
        let s = Shift::doppler(3e4 / C, 1.0, false);
        let a = publish(&star(5772.0), &t).bands[3].power_w;
        let b = publish_shifted(&star(5772.0), &s, &t).bands[3].power_w;
        assert!(rel(b, a) < 1e-3, "{}", rel(b, a));
    }

    /// Twin jets: the approaching one outshines the receding one by
    /// ((1 + β cos θ) / (1 − β cos θ))^(p+1) in total for a steady jet (p = 2).
    #[test]
    fn twin_jets_differ_by_the_beaming_formula() {
        let t = LightTable::base();
        let mut jet = LightDescription::new("jet");
        jet.parts.push(Part::PowerLaw {
            power_w: 1e30,
            from_m: 1e-12,
            to_m: 1e3,
            alpha: 0.7,
            cause: "synchrotron".into(),
        });
        let (b, c) = (0.9, 0.5);
        let near = Shift::doppler(b, c, true);
        let far = Shift::doppler(b, -c, true);
        // Over a wide window both see all of the jet's light.
        let pn = shifted_power(&jet, 1e-16, 1e6, &near, &t);
        let pf = shifted_power(&jet, 1e-16, 1e6, &far, &t);
        let want = sse_core::math::pow((1.0 + b * c) / (1.0 - b * c), 3.0);
        assert!(rel(pn / pf, want) < 1e-9, "{} vs {want}", pn / pf);
    }

    /// Sirius B: 1.018 M☉ in 0.00864 R☉ gives a gravitational redshift of
    /// about 75 km/s; Barstow et al. 2005 measured 80.4 ± 4.8 km/s.
    #[test]
    fn sirius_b_gravitational_redshift() {
        let s = Shift::gravitational(1.018 * 1.988_41e30, 0.008_64 * 6.957e8);
        let v = (1.0 / s.factor - 1.0) * C / 1e3;
        assert!((v - 80.4).abs() < 2.0 * 4.8, "{v} km/s");
    }

    /// At z = 1 every wavelength doubles: H-alpha moves into the near infrared,
    /// carrying (1 + z)⁻² of its power (with the comoving distance).
    #[test]
    fn cosmological_redshift_stretches() {
        let t = LightTable::base();
        let s = Shift::cosmological(1.0);
        assert!(rel(s.line_at(656.281e-9), 1312.562e-9) < 1e-12);
        let mut d = LightDescription::new("g");
        d.lines.push(LineEmission {
            line: "ha".into(),
            power_w: 1.0,
            cause: "test".into(),
        });
        let p = publish_shifted(&d, &s, &t);
        assert_eq!(p.bands[3].power_w, 0.0);
        assert!(rel(p.bands[4].power_w, 0.25) < 1e-12);
        let c = s.then(&Shift::doppler(0.1, 1.0, false));
        assert!(composite_power(&d, 1e-9, 1e-5, &c, &t) > 0.0);
    }
}

mod o12 {
    use crate::observe::*;
    use crate::*;
    const PC: f64 = 3.085_677_581e16;

    fn star(addr: &str, t_k: f64, r_m: f64) -> LightDescription {
        let mut d = LightDescription::new(addr);
        d.parts.push(Part::Thermal {
            t_k,
            area_m2: 4.0 * m::PI * r_m * r_m,
            emissivity: 1.0,
            cause: "photosphere".into(),
        });
        d
    }

    fn one(t_k: f64, v: f64) -> Component {
        Component {
            light: star("u:t/sys:0/star:0", t_k, 6.957e8),
            radial_velocity_m_s: v,
            photosphere_k: Some(t_k),
        }
    }

    fn gm() -> Observer {
        Observer {
            coverage: vec![],
            resolving_power: 1e4,
        }
    }

    fn has(s: &Spectrum, id: &str) -> bool {
        s.lines.iter().any(|l| l.line == id)
    }

    fn reading(s: &Spectrum, what: &str) -> Option<f64> {
        s.readings
            .iter()
            .find(|r| r.what.starts_with(what))
            .map(|r| r.value)
    }

    /// The Sun from 10 pc: Wien's law gives back 5772 K; the Fraunhofer lines are listed; TiO and He II are not.
    #[test]
    fn the_sun_from_ten_parsecs() {
        let table = LightTable::base();
        let t = Target {
            address: "u:sol/sys:0/star:0".into(),
            name: Some("Sun".into()),
            distance_m: 10.0 * PC,
            components: vec![one(5772.0, 0.0)],
        };
        let s = observe(&t, &gm(), &table).unwrap();
        let tk = reading(&s, "temperature").unwrap();
        assert!((tk / 5772.0 - 1.0).abs() < 1e-3, "{tk}");
        for id in ["ca-k", "ca-h", "ch-g", "mg-b", "fe-e", "na-d", "ha", "hb"] {
            assert!(has(&s, id), "{id}");
        }
        assert!(!has(&s, "tio-7054") && !has(&s, "heii"));
        let sigma_t4 = SIGMA * 5772.0 * 5772.0 * 5772.0 * 5772.0;
        let want = sigma_t4 * 4.0 * m::PI * 6.957e8 * 6.957e8 / (4.0 * m::PI * 100.0 * PC * PC);
        let got = reading(&s, "flux").unwrap();
        assert!((got / want - 1.0).abs() < 1e-3, "{got} {want}");
    }

    /// Temperature decides the lines: helium in a 20 000 K star, titanium oxide in a 3200 K one.
    #[test]
    fn lines_follow_temperature() {
        let table = LightTable::base();
        let at = |tk: f64| {
            let t = Target {
                address: "u:t/sys:0/star:0".into(),
                name: None,
                distance_m: PC,
                components: vec![one(tk, 0.0)],
            };
            observe(&t, &gm(), &table).unwrap()
        };
        let b = at(20_000.0);
        assert!(has(&b, "hei") && !has(&b, "tio-7054") && !has(&b, "na-d"));
        let md = at(3200.0);
        assert!(has(&md, "tio-7054") && has(&md, "na-d") && !has(&md, "hei"));
    }

    /// A double star shows doubled lines when the spectrograph can split them.
    #[test]
    fn double_star_doubles_its_lines() {
        let table = LightTable::base();
        let t = Target {
            address: "u:t/sys:0/star:0".into(),
            name: None,
            distance_m: 20.0 * PC,
            components: vec![one(5800.0, -30e3), one(5600.0, 30e3)],
        };
        let s = observe(&t, &gm(), &table).unwrap();
        assert_eq!(reading(&s, "lines doubled"), Some(1.0));
        let v0 = reading(&s, "line-of-sight speed, component 0").unwrap();
        let v1 = reading(&s, "line-of-sight speed, component 1").unwrap();
        assert!(
            (v0 + 30e3).abs() < 10.0 && (v1 - 30e3).abs() < 10.0,
            "{v0} {v1}"
        );
        let coarse = Observer {
            coverage: vec![],
            resolving_power: 2000.0,
        };
        assert_eq!(
            reading(&observe(&t, &coarse, &table).unwrap(), "lines doubled"),
            Some(0.0)
        );
    }

    /// An observer reads only what its bands cover: optical sees the peak and the
    /// lines; radio alone sees neither, and the chart greys the rest.
    #[test]
    fn coverage_greys_the_rest() {
        let table = LightTable::base();
        let t = Target {
            address: "u:sol/sys:0/star:0".into(),
            name: None,
            distance_m: 10.0 * PC,
            components: vec![one(5772.0, 12e3)],
        };
        let optical = Observer {
            coverage: vec![Coverage {
                name: "optical telescope".into(),
                from_m: 3.5e-7,
                to_m: 1.0e-6,
            }],
            resolving_power: 5000.0,
        };
        let s = observe(&t, &optical, &table).unwrap();
        assert!(reading(&s, "temperature").is_some());
        assert!(s.samples.iter().any(|x| x[2] == 0.0) && s.samples.iter().any(|x| x[2] == 1.0));
        assert!(s
            .samples
            .iter()
            .filter(|x| x[0] > 2e-6)
            .all(|x| x[2] == 0.0));
        assert!((reading(&s, "line-of-sight speed").unwrap() - 12e3).abs() < 5.0);
        let radio = Observer {
            coverage: vec![Coverage {
                name: "radio dish".into(),
                from_m: 1e-2,
                to_m: 1.0,
            }],
            resolving_power: 5000.0,
        };
        let r = observe(&t, &radio, &table).unwrap();
        assert!(reading(&r, "temperature").is_none());
        assert!(reading(&r, "line-of-sight speed").is_none());
        assert!(r.why.iter().any(|w| w.contains("outside your instruments")));
        assert!(r.lines.iter().all(|l| !l.covered));
    }

    /// Emission lines arrive thinned by distance and moved by speed.
    #[test]
    fn emission_lines_arrive() {
        let table = LightTable::base();
        let mut neb = LightDescription::new("u:t/sys:0/nebula:0");
        neb.lines.push(LineEmission {
            line: "ha".into(),
            power_w: 1e27,
            cause: "recombination".into(),
        });
        let t = Target {
            address: "u:t/sys:0/nebula:0".into(),
            name: None,
            distance_m: 500.0 * PC,
            components: vec![Component {
                light: neb,
                radial_velocity_m_s: -20e3,
                photosphere_k: None,
            }],
        };
        let s = observe(&t, &gm(), &table).unwrap();
        let l = s.lines.iter().find(|l| l.line == "ha").unwrap();
        assert_eq!(l.kind, "emission");
        let want = 1e27 / (4.0 * m::PI * 500.0 * PC * 500.0 * PC);
        assert!((l.amount / want - 1.0).abs() < 1e-3);
        assert!(l.observed_m < l.rest_m);
    }
}

mod air_optics {
    use crate::air::*;
    use crate::table::LightTable;

    fn mix(list: &[(&str, f64)]) -> Vec<Share> {
        list.iter()
            .map(|(g, f)| Share {
                gas: g.to_string(),
                fraction: *f,
            })
            .collect()
    }

    fn earth(gases: Vec<Share>) -> Air {
        Air {
            address: "u:sol/sys:0/body:2".into(),
            gases,
            surface_pressure_pa: 101_325.0,
            surface_temperature_k: 288.15,
            gravity_m_s2: 9.80665,
        }
    }

    /// The gases in V3 mixes all resolve, and scatter in the order of their
    /// refractivity squared: neon far below air, sulphur dioxide well above.
    #[test]
    fn v3_mix_gases_resolve() {
        let t = LightTable::base();
        let at = |g: &str| cross_section(&earth(mix(&[(g, 1.0)])), 550e-9, &t).unwrap();
        let air = at("air");
        assert!(at("ne") < 0.1 * air);
        assert!((0.9 * air..1.6 * air).contains(&at("co")));
        assert!(at("nh3") > air && at("nh3") < at("so2"));
        assert!(at("so2") > 4.0 * air);
    }

    /// Earth from its gases against Bucholtz 1995 (4.84, 11.49 and 28.6 per
    /// million metres at 680, 550 and 440 nm) and a scale height of 8.43 km.
    #[test]
    fn earth_from_its_gases() {
        let t = LightTable::base();
        for gases in [
            mix(&[
                ("n2", 0.7808),
                ("o2", 0.2095),
                ("ar", 0.0093),
                ("co2", 0.0004),
            ]),
            mix(&[("air", 1.0)]),
        ] {
            let o = optics(&earth(gases), &[680e-9, 550e-9, 440e-9], &t).unwrap();
            let want = [4.84e-6, 11.49e-6, 28.6e-6];
            for (x, w) in o.at.iter().zip(want) {
                assert!(
                    (x.beta_per_m / w - 1.0).abs() < 0.015,
                    "{} vs {w}",
                    x.beta_per_m
                );
            }
            assert!(
                (o.scale_height_m - 8430.0).abs() < 30.0,
                "{}",
                o.scale_height_m
            );
            assert!((o.at[1].tau / 0.0971 - 1.0).abs() < 0.01, "{}", o.at[1].tau);
            assert_eq!(o.visible_slices.len(), 3);
            assert!(o.visible_slices[0].1.tau > o.visible_slices[2].1.tau);
        }
    }

    /// Mars: thin carbon dioxide air, scale height about 11 km, far clearer than Earth's.
    /// Titan: thick cold nitrogen, much more scattering than Earth.
    #[test]
    fn mars_and_titan() {
        let t = LightTable::base();
        let mars = Air {
            address: "u:sol/sys:0/body:3".into(),
            gases: mix(&[
                ("co2", 0.951),
                ("n2", 0.0259),
                ("ar", 0.0194),
                ("o2", 0.0037),
            ]),
            surface_pressure_pa: 610.0,
            surface_temperature_k: 210.0,
            gravity_m_s2: 3.71,
        };
        let m = optics(&mars, &[550e-9], &t).unwrap();
        assert!(
            (m.scale_height_m / 1e3 - 11.0).abs() < 0.6,
            "{}",
            m.scale_height_m
        );
        assert!(
            m.at[0].tau > 0.001 && m.at[0].tau < 0.006,
            "{}",
            m.at[0].tau
        );
        let titan = Air {
            address: "u:sol/sys:0/body:6/moon:5".into(),
            gases: mix(&[("n2", 0.95), ("ch4", 0.05)]),
            surface_pressure_pa: 146_700.0,
            surface_temperature_k: 93.7,
            gravity_m_s2: 1.352,
        };
        let ti = optics(&titan, &[550e-9], &t).unwrap();
        assert!(ti.at[0].tau > 5.0 * 0.0971, "{}", ti.at[0].tau);
        assert!(optics(&earth(mix(&[("xenon-ish", 1.0)])), &[550e-9], &t).is_err());
        assert!(optics(&earth(mix(&[("n2", 0.5)])), &[550e-9], &t).is_err());
    }
}

/// The Sun as a 5772 K blackbody at 10 pc: M_V 4.78 and B−V 0.65 measured
/// (Blanton and Roweis 2007 Table 1, Vega system; Ramírez et al. 2012 for
/// B−V, doi:10.1088/0004-637X/752/1/5). Dust: E(B−V) = A_V / R_V within a few
/// per cent, and the infrared nearly clear.
#[test]
fn sun_magnitudes_and_reddening() {
    use crate::photometry::*;
    let table = t();
    let bands = bands_of(&table, &[]).unwrap();
    let sun = Photosphere {
        temperature_k: 5772.0,
        radius_m: Some(6.957e8),
        luminosity_w: None,
    };
    let pc = 3.085_677_581e16;
    let p = star_photometry(
        &sun,
        &Sight {
            distance_m: 10.0 * pc,
            a_v: 0.0,
            r_v: 3.1,
        },
        &bands,
    )
    .unwrap();
    let v = p.magnitudes.iter().find(|x| x.band == "V").unwrap();
    assert!((v.m_vega - 4.78).abs() < 0.15, "M_V {}", v.m_vega);
    let bv = p.colours.iter().find(|c| c.name == "B-V").unwrap();
    assert!((bv.value - 0.65).abs() < 0.12, "B−V {}", bv.value);
    // Same star from its luminosity.
    let by_l = Photosphere {
        temperature_k: 5772.0,
        radius_m: None,
        luminosity_w: Some(3.828e26),
    };
    let q = star_photometry(
        &by_l,
        &Sight {
            distance_m: 10.0 * pc,
            a_v: 0.0,
            r_v: 3.1,
        },
        &bands,
    )
    .unwrap();
    assert!((q.magnitudes[2].m_vega - v.m_vega).abs() < 0.01);
    // Through one magnitude of dust in V.
    let d = star_photometry(
        &sun,
        &Sight {
            distance_m: 10.0 * pc,
            a_v: 1.0,
            r_v: 3.1,
        },
        &bands,
    )
    .unwrap();
    let dv = d.magnitudes.iter().find(|x| x.band == "V").unwrap();
    assert!((dv.m_vega - v.m_vega - 1.0).abs() < 0.02, "V dims by A_V");
    let ex = d.colours.iter().find(|c| c.name == "B-V").unwrap().excess;
    assert!((ex - 1.0 / 3.1).abs() < 0.03, "E(B−V) {ex}");
    let k = d.magnitudes.iter().find(|x| x.band == "Ks").unwrap();
    assert!(k.extinction < 0.15, "Ks nearly clear {}", k.extinction);
    // Hotter is bluer.
    let hot = Photosphere {
        temperature_k: 10_000.0,
        ..sun.clone()
    };
    let h = star_photometry(
        &hot,
        &Sight {
            distance_m: 10.0 * pc,
            a_v: 0.0,
            r_v: 3.1,
        },
        &bands,
    )
    .unwrap();
    let hb = h.colours.iter().find(|c| c.name == "B-V").unwrap().value;
    // Ballesteros 2012 (doi:10.1209/0295-5075/97/34008) puts a 10,000 K
    // blackbody near B−V 0; the effective-wavelength stand-in reads 0.14.
    assert!(hb < 0.2 && hb < bv.value - 0.4, "10,000 K B−V {hb}");
}

/// MF9 part 2: every line names the element or substance that gives it off,
/// by its id on the master data sheet, so its card can show "Light it gives
/// off" and an invented gas can carry lines of its own.
#[test]
fn every_line_names_its_matter_on_the_sheet() {
    let s = sse_reference::Reference::base();
    let lines = &t().lines;
    let named: Vec<&LineDef> = lines.iter().filter(|l| l.matter.is_some()).collect();
    assert!(named.len() >= 24, "{}", named.len());
    for l in named {
        let id = l.matter.as_deref().unwrap();
        assert!(
            s.row("elements", id).is_some() || s.row("substances", id).is_some(),
            "line {}: no sheet row {id}",
            l.id
        );
    }
    let o: Vec<&str> = lines
        .iter()
        .filter(|l| l.matter.as_deref() == Some("o"))
        .map(|l| l.id.as_str())
        .collect();
    assert!(o.contains(&"oi-5577") && o.contains(&"oi-6300"), "{o:?}");
}

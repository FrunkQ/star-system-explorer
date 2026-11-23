use crate::*;
use sse_light::emit::{describe, Conditions, Model, Radiator, Source, Transmitter};

const AU: f64 = 1.495_978_707e11;
const PC: f64 = 3.085_677_581e16;

fn ship(radiator_k: f64, transmitting: bool) -> LightDescription {
    let c = Conditions {
        address: "u:t/sys:0/ship:0".into(),
        radius_m: 50.0,
        own_luminosity_w: 0.0,
        bond_albedo: 0.1,
        lit_by: vec![Source {
            address: "u:t/sys:0/star:0".into(),
            luminosity_w: 3.828e26,
            t_k: 5772.0,
            distance_m: 5.0 * AU,
        }],
        radiators: if radiator_k > 0.0 {
            vec![Radiator {
                t_k: radiator_k,
                area_m2: 2000.0,
            }]
        } else {
            vec![]
        },
        transmitters: if transmitting {
            vec![Transmitter {
                frequency_hz: 3e9,
                bandwidth_hz: 1e6,
                power_w: 1e4,
            }]
        } else {
            vec![]
        },
        ..Default::default()
    };
    describe(&c, Model::base()).unwrap().0
}

fn ir_telescope() -> Sensor {
    Sensor {
        name: "mid-infrared telescope".into(),
        kind: "photon".into(),
        from_m: 8e-6,
        to_m: 14e-6,
        aperture_m: 1.0,
        efficiency: 0.5,
        dark_photons_s: 100.0,
        background_w_m2_sr: 0.0,
        resolution_rad: None,
        system_temperature_k: 50.0,
        bandwidth_hz: None,
        threshold_snr: 5.0,
        stare_s: 60.0,
        method: "image".into(),
        resolving_power: None,
        transmit_power_w: None,
        noise_density: None,
        field_clutter: 0.0,
        authored_range_m: None,
    }
}

fn visible_camera() -> Sensor {
    Sensor {
        name: "visible camera".into(),
        from_m: 4e-7,
        to_m: 7e-7,
        aperture_m: 0.5,
        stare_s: 10.0,
        method: "image".into(),
        resolving_power: None,
        transmit_power_w: None,
        noise_density: None,
        field_clutter: 0.0,
        authored_range_m: None,
        dark_photons_s: 1.0,
        ..ir_telescope()
    }
}

fn seen(light: LightDescription, d: f64) -> Seen {
    Seen {
        address: "u:t/sys:0/ship:0".into(),
        light,
        distance_m: d,
        extinction_av: 0.0,
        neighbours: vec![],
        magnetic_moment_a_m2: 0.0,
        mass_kg: 0.0,
        radar_cross_section_m2: None,
    }
}

/// A hot ship is seen in infrared across a system; a cold one is not. At the
/// range the arithmetic gives, S/N is exactly the threshold.
#[test]
fn hot_and_cold_ships() {
    let t = LightTable::base();
    let hot = detect(&seen(ship(600.0, false), 2.0 * AU), &ir_telescope(), &t).unwrap();
    assert!(hot.detectable, "{:?}", hot.why);
    assert!(hot.range_m > 2.0 * AU);
    let at_range = detect(&seen(ship(600.0, false), hot.range_m), &ir_telescope(), &t).unwrap();
    assert!((at_range.snr / 5.0 - 1.0).abs() < 1e-6, "{}", at_range.snr);
    let cold = detect(&seen(ship(0.0, false), 2.0 * AU), &ir_telescope(), &t).unwrap();
    assert!(!cold.detectable);
    assert!(
        cold.range_m < hot.range_m / 10.0,
        "{} {}",
        cold.range_m,
        hot.range_m
    );
    // Staring for the stated time at the stated range reaches the threshold exactly.
    let mut s = ir_telescope();
    s.stare_s = cold.stare_needed_s;
    let c2 = detect(&seen(ship(0.0, false), 2.0 * AU), &s, &t).unwrap();
    assert!((c2.snr / 5.0 - 1.0).abs() < 1e-6);
}

/// In front of its star, a ship is lost in the glare in visible light.
#[test]
fn lost_in_front_of_the_star() {
    let t = LightTable::base();
    let mut sun = LightDescription::new("u:t/sys:0/star:0");
    sun.parts.push(sse_light::Part::Thermal {
        t_k: 5772.0,
        area_m2: 4.0 * m::PI * 6.957e8 * 6.957e8,
        emissivity: 1.0,
        cause: "photosphere".into(),
    });
    let mut s = seen(ship(600.0, false), 0.01 * AU);
    let clear = detect(&s, &visible_camera(), &t).unwrap();
    assert!(clear.detectable);
    s.neighbours.push(Neighbour {
        address: "u:t/sys:0/star:0".into(),
        light: sun,
        distance_m: 5.0 * AU,
        separation_rad: 1e-7,
    });
    let glare = detect(&s, &visible_camera(), &t).unwrap();
    assert!(!glare.detectable && glare.lost_in_glare, "{:?}", glare.why);
}

/// Dust: 30 magnitudes of visual extinction blocks visible light but barely dims the mid infrared.
#[test]
fn dust_blocks_visible_not_infrared() {
    let t = LightTable::base();
    let mut s = seen(ship(600.0, false), 0.01 * AU);
    s.extinction_av = 30.0;
    let v = detect(&s, &visible_camera(), &t).unwrap();
    let ir = detect(&s, &ir_telescope(), &t).unwrap();
    assert!(!v.detectable && v.extinction_mag > 25.0);
    assert!(
        ir.detectable && ir.extinction_mag < 0.3,
        "{}",
        ir.extinction_mag
    );
}

/// A 10 kW transmitter, heard by a 10 m dish from 1 au in a minute (S/N about 16); a silent ship is dark there.
#[test]
fn talking_ships_shine_in_radio() {
    let t = LightTable::base();
    let dish = Sensor {
        name: "S-band dish".into(),
        kind: "radio".into(),
        from_m: C / 3.01e9,
        to_m: C / 2.99e9,
        aperture_m: 10.0,
        bandwidth_hz: Some(1e6),
        system_temperature_k: 50.0,
        ..ir_telescope()
    };
    let talking = detect(&seen(ship(0.0, true), AU), &dish, &t).unwrap();
    assert!(talking.snr > 10.0 && talking.snr < 25.0, "{}", talking.snr);
    assert!(talking.detectable, "{:?}", talking.why);
    let silent = detect(&seen(ship(0.0, false), AU), &dish, &t).unwrap();
    assert!(!silent.detectable);
}

/// A radar is heard further than it can see.
#[test]
fn radar_reveals_the_sender() {
    let r = radar_ranges(&Radar {
        power_w: 1e5,
        gain: 1e4,
        wavelength_m: 0.1,
        cross_section_m2: 100.0,
        min_power_w: 1e-16,
    });
    assert!(
        r.heard_to_m > 100.0 * r.sees_to_m,
        "{} {}",
        r.heard_to_m,
        r.sees_to_m
    );
    // Doubling range costs 16 times the power for the echo, 4 for the listener.
    let r2 = radar_ranges(&Radar {
        power_w: 1.6e6,
        gain: 1e4,
        wavelength_m: 0.1,
        cross_section_m2: 100.0,
        min_power_w: 1e-16,
    });
    assert!((r2.sees_to_m / r.sees_to_m - 2.0).abs() < 1e-9);
    assert!((r2.heard_to_m / r.heard_to_m - 4.0).abs() < 1e-9);
}

/// The same star seen from 10 pc by a visible camera.
#[test]
fn a_star_from_ten_parsecs() {
    let t = LightTable::base();
    let mut sun = LightDescription::new("u:t/sys:0/star:0");
    sun.parts.push(sse_light::Part::Thermal {
        t_k: 5772.0,
        area_m2: 4.0 * m::PI * 6.957e8 * 6.957e8,
        emissivity: 1.0,
        cause: "photosphere".into(),
    });
    let d = detect(
        &Seen {
            address: "u:t/sys:0/star:0".into(),
            light: sun,
            distance_m: 10.0 * PC,
            extinction_av: 0.0,
            neighbours: vec![],
            magnetic_moment_a_m2: 0.0,
            mass_kg: 0.0,
            radar_cross_section_m2: None,
        },
        &visible_camera(),
        &t,
    )
    .unwrap();
    assert!(d.detectable && d.range_m > 100.0 * PC);
}

// RK11: one instrument shape with a method; field channels; authored ranges.
// Constants typed here for the test only (tests are outside the sheet audit).
const NATURE: Nature = Nature {
    g: 6.674_30e-11,
    mu0: 1.256_637_062_12e-6,
};

fn dark(d: f64, moment: f64, mass: f64) -> Seen {
    Seen {
        magnetic_moment_a_m2: moment,
        mass_kg: mass,
        ..seen(LightDescription::new("u:t/sys:0/body:2"), d)
    }
}

/// A magnetometer finds an Earth-strength dipole tens of radii out, and at
/// the range it computes S/N is exactly the threshold.
#[test]
fn magnetometer_reads_a_dipole() {
    let t = Model::base().table.clone();
    let mag = Sensor {
        method: "magnetometer".into(),
        noise_density: Some(1e-11),
        field_clutter: 1e-9,
        stare_s: 100.0,
        ..ir_telescope()
    };
    let d = detect_with(&dark(1e8, 8e22, 0.0), &mag, &t, Some(&NATURE)).unwrap();
    assert_eq!(d.channel, "magnetic");
    assert_eq!(d.range_source, "computed");
    assert!(d.detectable, "{:?}", d.why);
    let r_earth = 6.371e6;
    assert!(
        (10.0..30.0).contains(&(d.range_m / r_earth)),
        "{}",
        d.range_m
    );
    let at = detect_with(&dark(d.range_m, 8e22, 0.0), &mag, &t, Some(&NATURE)).unwrap();
    assert!((at.snr - 5.0).abs() < 1e-6, "{}", at.snr);
    // Without the sheet's constants it says so.
    assert!(detect_with(&dark(1e8, 8e22, 0.0), &mag, &t, None).is_err());
}

/// Finding a ship by its mass is real but short: a thousand-tonne ship is
/// seen by a GOCE-class gradiometer only from hundreds of metres.
#[test]
fn gradiometer_finds_mass_only_close() {
    let t = Model::base().table.clone();
    let grad = Sensor {
        method: "gradiometer".into(),
        noise_density: Some(1e-11),
        stare_s: 100.0,
        ..ir_telescope()
    };
    let d = detect_with(&dark(1e3, 0.0, 1e6), &grad, &t, Some(&NATURE)).unwrap();
    assert_eq!(d.channel, "gravity");
    assert!((100.0..1000.0).contains(&d.range_m), "{}", d.range_m);
    assert!(!d.detectable);
    assert!(d.why.iter().any(|w| w.contains("only this close")));
}

/// A spectrum splits the light, so it reaches less far than an image; an
/// astrometric fix is the resolution over the S/N.
#[test]
fn spectrum_and_astrometry() {
    let t = Model::base().table.clone();
    let hot = seen(ship(600.0, false), 1e9);
    let img = detect(&hot, &ir_telescope(), &t).unwrap();
    let spec = Sensor {
        method: "spectrum".into(),
        resolving_power: Some(1000.0),
        ..ir_telescope()
    };
    let sp = detect(&hot, &spec, &t).unwrap();
    assert!(sp.range_m < img.range_m, "{} {}", sp.range_m, img.range_m);
    let ast = Sensor {
        method: "astrometry".into(),
        ..ir_telescope()
    };
    let a = detect(&hot, &ast, &t).unwrap();
    let p = a.precision_rad.unwrap();
    assert!((p - ast.resolution() / a.snr).abs() < 1e-12 * p.max(1e-30));
}

/// An active instrument sees by its echo and is heard much further; what it
/// sends is a load.
#[test]
fn active_sees_and_is_heard() {
    let t = Model::base().table.clone();
    let radar = Sensor {
        name: "search radar".into(),
        kind: "radio".into(),
        from_m: 0.03,
        to_m: 0.031,
        aperture_m: 3.0,
        method: "active".into(),
        transmit_power_w: Some(1e5),
        ..ir_telescope()
    };
    let target = Seen {
        radar_cross_section_m2: Some(100.0),
        ..seen(ship(0.0, false), 1e7)
    };
    let d = detect(&target, &radar, &t).unwrap();
    assert_eq!(d.transmit_power_w, Some(1e5));
    let heard = d.heard_to_m.unwrap();
    assert!(heard > 10.0 * d.range_m, "{heard} {}", d.range_m);
    assert_eq!(d.detectable, 1e7 <= d.range_m);
    // No cross-section: refused in words.
    assert!(detect(&seen(ship(0.0, false), 1e7), &radar, &t).is_err());
}

/// A typed range from an older file is kept, marked authored, and used only
/// when the instrument cannot compute one.
#[test]
fn authored_ranges_are_kept_and_marked() {
    let t = Model::base().table.clone();
    let old = Sensor {
        aperture_m: 0.0,
        authored_range_m: Some(7.5e12),
        ..ir_telescope()
    };
    let d = detect(&seen(ship(0.0, false), 1e12), &old, &t).unwrap();
    assert_eq!(d.range_source, "authored");
    assert!(d.detectable);
    let both = Sensor {
        authored_range_m: Some(7.5e12),
        ..ir_telescope()
    };
    let c = detect(&seen(ship(300.0, false), 1e12), &both, &t).unwrap();
    assert_eq!(c.range_source, "computed");
    assert_eq!(c.authored_range_m, Some(7.5e12));
    // An unknown method is refused.
    let odd = Sensor {
        method: "bio".into(),
        ..ir_telescope()
    };
    assert!(detect(&seen(ship(0.0, false), 1e9), &odd, &t).is_err());
}

/// Range bands are a labelled reading of a real distance.
#[test]
fn range_bands_label_a_distance() {
    let bands = vec![
        RangeBand {
            id: "far".into(),
            label: "far".into(),
            range_m: 1.5e11,
        },
        RangeBand {
            id: "close".into(),
            label: "close".into(),
            range_m: 1e4,
        },
        RangeBand {
            id: "short".into(),
            label: "short".into(),
            range_m: 1.25e6,
        },
    ];
    assert_eq!(band_of(5e3, &bands).unwrap().id, "close");
    assert_eq!(band_of(1e6, &bands).unwrap().id, "short");
    assert_eq!(band_of(1e9, &bands).unwrap().id, "far");
    assert!(band_of(1e12, &bands).is_none());
}

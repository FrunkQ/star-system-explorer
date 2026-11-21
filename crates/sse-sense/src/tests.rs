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
    }
}

fn visible_camera() -> Sensor {
    Sensor {
        name: "visible camera".into(),
        from_m: 4e-7,
        to_m: 7e-7,
        aperture_m: 0.5,
        stare_s: 10.0,
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
        },
        &visible_camera(),
        &t,
    )
    .unwrap();
    assert!(d.detectable && d.range_m > 100.0 * PC);
}

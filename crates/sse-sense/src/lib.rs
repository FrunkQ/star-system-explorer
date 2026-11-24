//! O9: sensing (nebulae-and-light design 5A). What a sensor can detect falls
//! out of two things: the target's light in the sensor's band, after
//! distance and dust, and the sensor's noise (background, dark counts, glare
//! from anything bright beside the target). No stealth rating.
//!
//! - Light bands: photon counting. Signal S and background B in photons per
//!   second; S/N after a stare t is S t / √((S + B + D) t).
//! - Radio: the radiometer equation. S/N = (P / k T_sys B) √(B t), with
//!   P = flux × effective area.
//! - Radar: the radar equation for the echo against the direct pulse that
//!   any listener in the beam receives, so a radar is heard further than it
//!   can see.
//!
//! RK11 (Atomic Rockets plan section 12): one instrument shape with a
//! `method` (image, spectrum, astrometry, active, magnetometer,
//! gradiometer), and objects publish on channels: light by band, their
//! magnetic field, their gravity. No range is typed: how far an instrument
//! reaches is computed from what the target gives off. A range typed in an
//! older file (SSE3's sensors) is kept as an authored range, marked, and used
//! only where the instrument has no aperture or noise figure to compute with.

use serde::{Deserialize, Serialize};
use sse_core::math as m;
use sse_flow::Reference;
use sse_light::table::LightTable;
use sse_light::{power_between, LightDescription, C, H, K_B};

pub mod reach;
#[cfg(test)]
mod reach_tests;
#[cfg(test)]
mod tests;

const R_PHOTON: &str = "Photon-counting signal to noise, S t / √((S + B + D) t): Howell, S. B. 2006, Handbook of CCD Astronomy, 2nd edn, Cambridge University Press, chapter 4 (the CCD equation). doi:10.1017/CBO9780511807909";
const R_RADIOMETER: &str = "Radiometer equation ΔT = T_sys / √(B t): Wilson, T. L., Rohlfs, K. and Hüttemeister, S. 2013, Tools of Radio Astronomy, 6th edn, Springer, chapter 4. doi:10.1007/978-3-642-39950-3";
const R_RADAR: &str = "Radar equation P_r = P G² λ² σ / ((4π)³ R⁴) and effective aperture A_e = G λ² / 4π: Skolnik, M. I. 2001, Introduction to Radar Systems, 3rd edn, McGraw-Hill, chapter 2";
const R_DUST: &str = "Interstellar extinction falls with wavelength: A_λ/A_V ≈ (λ/0.55 µm)^−1.75 in the near and mid infrared (Draine, B. T. 2003, Interstellar dust grains, ARA&A 41, 241. doi:10.1146/annurev.astro.41.011802.094840); the optical shape follows Cardelli, Clayton and Mathis 1989, ApJ 345, 245. doi:10.1086/167900";
const R_DIFFRACTION: &str = "Diffraction limit 1.22 λ / D (Airy disc first minimum)";
const R_DIPOLE: &str = "Kivelson, M. G. and Russell, C. T. (eds) 1995, Introduction to Space Physics, Cambridge University Press: a dipole's field falls as μ0 M / (4π r³) on its equator plane, twice that along its axis. https://www.cambridge.org/9780521457149";
const R_GRADIENT: &str = "The gravity gradient of a mass M at distance r is 2GM / r³ along the line of sight (the second derivative of GM / r); a gradiometer's noise is a spectral density in s⁻² per √Hz, 1 eötvös = 1e-9 s⁻² (Rummel, R., Yi, W. and Stummer, C. 2011, GOCE gravitational gradiometry, J. Geod. 85, 777. doi:10.1007/s00190-011-0500-0)";
const R_NOISE: &str = "White noise of spectral density n averaged for t seconds leaves σ = n / √(2t) (the equivalent noise bandwidth 1 / (2t) of an average): Wilson, Rohlfs and Hüttemeister 2013, Tools of Radio Astronomy, chapter 4. doi:10.1007/978-3-642-39950-3";
const R_CENTROID: &str = "A star image's position error, relative to the image's size, is very like its brightness error relative to its brightness, so position is found to about the image's width over the signal to noise: King, I. R. 1983, Accuracy of measurement of star images on a pixel array, PASP 95, 163. https://ui.adsabs.harvard.edu/abs/1983PASP...95..163K/abstract";

/// Constants of nature the field channels need, from the master data sheet.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Nature {
    pub g: f64,
    pub mu0: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Sensor {
    pub name: String,
    /// "photon" (gamma to far infrared) or "radio".
    #[serde(default = "photon")]
    pub kind: String,
    pub from_m: f64,
    pub to_m: f64,
    pub aperture_m: f64,
    #[serde(default = "half")]
    pub efficiency: f64,
    /// Photon sensors: detector dark counts per second in the resolution element.
    #[serde(default = "one")]
    pub dark_photons_s: f64,
    /// Diffuse background radiance in the band, W m⁻² sr⁻¹ (zodiacal light, nebula glow, warm optics).
    #[serde(default)]
    pub background_w_m2_sr: f64,
    /// Resolution, rad; default the diffraction limit at the band centre.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resolution_rad: Option<f64>,
    /// Radio: system temperature, K, and bandwidth, Hz (default the whole band).
    #[serde(default = "t_sys")]
    pub system_temperature_k: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bandwidth_hz: Option<f64>,
    /// Signal to noise needed to call it a detection.
    #[serde(default = "five")]
    pub threshold_snr: f64,
    /// How long it may stare, s.
    #[serde(default = "one")]
    pub stare_s: f64,
    /// What it does with what it catches: "image" (default), "spectrum",
    /// "astrometry", "active" (radar or lidar), "magnetometer", "gradiometer".
    #[serde(default = "image")]
    pub method: String,
    /// Spectrum: resolving power λ/Δλ, which splits the light into that many
    /// elements per e-fold of wavelength.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resolving_power: Option<f64>,
    /// Active: power sent, W. It is a load on the electric bus and lights
    /// the ship up for anyone in the beam.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transmit_power_w: Option<f64>,
    /// Field instruments: noise spectral density in the channel's own unit
    /// per √Hz (T for a magnetometer, s⁻² for a gradiometer).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub noise_density: Option<f64>,
    /// Field instruments: the channel's own wander here (the wind's field,
    /// a planet's gravity left after modelling), same unit as the reading.
    #[serde(default)]
    pub field_clutter: f64,
    /// A range typed in an older file (SSE3), kept and marked "authored";
    /// used only when the instrument lacks the figures to compute one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub authored_range_m: Option<f64>,
}

fn photon() -> String {
    "photon".into()
}
fn image() -> String {
    "image".into()
}
fn half() -> f64 {
    0.5
}
fn one() -> f64 {
    1.0
}
fn five() -> f64 {
    5.0
}
fn t_sys() -> f64 {
    50.0
}

impl Sensor {
    pub fn centre_m(&self) -> f64 {
        m::sqrt(self.from_m * self.to_m)
    }
    pub fn resolution(&self) -> f64 {
        self.resolution_rad
            .unwrap_or(1.22 * self.centre_m() / self.aperture_m)
    }
    fn area(&self) -> f64 {
        m::PI * 0.25 * self.aperture_m * self.aperture_m * self.efficiency
    }
    fn bandwidth(&self) -> f64 {
        self.bandwidth_hz.unwrap_or(C / self.from_m - C / self.to_m)
    }
}

/// Something bright near the target on the sky (its star, a nebula behind it).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Neighbour {
    pub address: String,
    pub light: LightDescription,
    pub distance_m: f64,
    pub separation_rad: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Seen {
    pub address: String,
    pub light: LightDescription,
    pub distance_m: f64,
    /// Dust between, as visual extinction A_V in magnitudes.
    #[serde(default)]
    pub extinction_av: f64,
    #[serde(default)]
    pub neighbours: Vec<Neighbour>,
    /// The magnetic channel: the target's dipole moment, A m².
    #[serde(default)]
    pub magnetic_moment_a_m2: f64,
    /// The gravity channel: the target's mass, kg.
    #[serde(default)]
    pub mass_kg: f64,
    /// For an active instrument: the target's radar cross-section, m².
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub radar_cross_section_m2: Option<f64>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Detection {
    pub target: String,
    pub sensor: String,
    pub method: String,
    /// "light", "magnetic" or "gravity".
    pub channel: String,
    /// "computed" from what the target gives off, or "authored" (a range
    /// typed in an older file, kept as it was).
    pub range_source: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub authored_range_m: Option<f64>,
    /// Astrometry: how precisely its position is fixed, rad.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub precision_rad: Option<f64>,
    /// Active: power sent, W, and how far the pulse is heard by a listener
    /// with the same receiver.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transmit_power_w: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub heard_to_m: Option<f64>,
    /// Flux in the band at the sensor, after dust, W/m².
    pub flux_w_m2: f64,
    /// Dimming by dust in the band, magnitudes.
    pub extinction_mag: f64,
    pub snr: f64,
    pub detectable: bool,
    /// Stare needed for the threshold, s (infinite if never).
    pub stare_needed_s: f64,
    /// Furthest distance at which this sensor would detect it with its stare (dust held as now).
    pub range_m: f64,
    pub lost_in_glare: bool,
    pub why: Vec<String>,
    pub references: Vec<Reference>,
}

/// Dust dimming at λ for visual extinction `av`, magnitudes. Optical to mid
/// infrared by the power law; ultraviolet held at the 0.3 µm value; nothing
/// in radio; X-rays not modelled (dust absorbs soft X-rays only weakly here).
pub fn extinction_mag(av: f64, lambda_m: f64) -> f64 {
    if av <= 0.0 || !(1e-8..=1e-3).contains(&lambda_m) {
        return 0.0;
    }
    let l = lambda_m.max(3e-7);
    av * m::pow(l / 0.55e-6, -1.75)
}

fn dimming(mag: f64) -> f64 {
    m::pow(10.0, -0.4 * mag)
}

/// Flux of a light description in [a, b] at distance d, W/m².
pub fn flux_in(light: &LightDescription, a: f64, b: f64, d: f64, table: &LightTable) -> f64 {
    power_between(light, a, b, table) / (4.0 * m::PI * d * d)
}

/// S/N for signal and noise rates over a stare.
fn snr_photon(s: f64, b: f64, t: f64) -> f64 {
    if s <= 0.0 {
        return 0.0;
    }
    s * t / m::sqrt((s + b) * t)
}

pub fn detect(seen: &Seen, sensor: &Sensor, table: &LightTable) -> Result<Detection, String> {
    detect_with(seen, sensor, table, None)
}

/// Detection on any channel; field instruments need the constants of nature.
pub fn detect_with(
    seen: &Seen,
    sensor: &Sensor,
    table: &LightTable,
    nature: Option<&Nature>,
) -> Result<Detection, String> {
    match sensor.method.as_str() {
        "magnetometer" | "gradiometer" => return detect_field(seen, sensor, nature),
        "image" | "spectrum" | "astrometry" | "active" => {}
        other => {
            return Err(format!(
                "{}: no method {other}; methods: image, spectrum, astrometry, active, magnetometer, gradiometer",
                sensor.name
            ))
        }
    }
    if sensor.aperture_m <= 0.0 {
        if let Some(r) = sensor.authored_range_m {
            return Ok(authored(seen, sensor, "light", r));
        }
    }
    if !(sensor.to_m > sensor.from_m
        && sensor.from_m > 0.0
        && sensor.aperture_m > 0.0
        && seen.distance_m > 0.0)
    {
        return Err(format!(
            "{} / {}: bad band, aperture or distance",
            seen.address, sensor.name
        ));
    }
    let lc = sensor.centre_m();
    let ext = extinction_mag(seen.extinction_av, lc);
    let flux = flux_in(
        &seen.light,
        sensor.from_m,
        sensor.to_m,
        seen.distance_m,
        table,
    ) * dimming(ext);
    let res = sensor.resolution();
    let mut why = vec![];
    let mut refs = vec![Reference::parse(R_DIFFRACTION)];
    if ext > 0.01 {
        why.push(format!(
            "Dust between dims this band by {:.2} magnitudes (a factor {:.3}).",
            ext,
            dimming(ext)
        ));
        refs.push(Reference::parse(R_DUST));
    }
    // Glare: neighbours inside the resolution element add their light to the noise.
    let mut glare_flux = 0.0;
    for n in &sensor_neighbours(seen, res) {
        glare_flux +=
            flux_in(&n.light, sensor.from_m, sensor.to_m, n.distance_m, table) * dimming(ext);
        why.push(format!(
            "{} lies within the sensor's resolution ({:.2e} rad), so its light swamps the pixel.",
            n.address, res
        ));
    }
    let (snr, stare_needed, range) = if sensor.kind == "radio" {
        refs.push(Reference::parse(R_RADIOMETER));
        let bw = sensor.bandwidth();
        let p = flux * sensor.area();
        let pg = glare_flux * sensor.area();
        // Glare adds to the system temperature.
        let t_sys = sensor.system_temperature_k + pg / (K_B * bw);
        let noise = K_B * t_sys * bw / m::sqrt(bw * sensor.stare_s);
        let snr = p / noise;
        let need = if p > 0.0 {
            sensor.stare_s * (sensor.threshold_snr / snr) * (sensor.threshold_snr / snr)
        } else {
            f64::INFINITY
        };
        // Signal falls as 1/d² (glare held as now).
        let range = if p > 0.0 {
            seen.distance_m * m::sqrt(snr / sensor.threshold_snr)
        } else {
            0.0
        };
        why.push(format!(
            "{:.3e} W reaches the {} against {:.3e} W of noise ({:.0} K over {:.3e} Hz for {:.0} s): S/N {:.2}.",
            p, sensor.name, noise, t_sys, bw, sensor.stare_s, snr
        ));
        (snr, need, range)
    } else {
        refs.push(Reference::parse(R_PHOTON));
        let e_ph = H * C / lc;
        // A spectrum spreads the light over its resolution elements; each must
        // reach the threshold on its own (dark counts are per element already).
        let elements = match (sensor.method.as_str(), sensor.resolving_power) {
            ("spectrum", Some(r)) if r > 0.0 => (r * m::ln(sensor.to_m / sensor.from_m)).max(1.0),
            _ => 1.0,
        };
        if elements > 1.0 {
            why.push(format!(
                "As a spectrum at resolving power {:.0}, the light is split over {:.0} elements, and each must stand out on its own.",
                sensor.resolving_power.unwrap_or(0.0),
                elements
            ));
        }
        let s = flux * sensor.area() / e_ph / elements;
        let omega = m::PI * res * res;
        let bg = (sensor.background_w_m2_sr * omega * sensor.area() / e_ph
            + glare_flux * sensor.area() / e_ph)
            / elements;
        let b = bg + sensor.dark_photons_s;
        let snr = snr_photon(s, b, sensor.stare_s);
        let k2 = sensor.threshold_snr * sensor.threshold_snr;
        let need = if s > 0.0 {
            k2 * (s + b) / (s * s)
        } else {
            f64::INFINITY
        };
        // Range: S scales as (d0/d)²; solve S0² t x² − k² S0 x − k² B = 0 for x = (d0/d)².
        let range = if s > 0.0 {
            let t = sensor.stare_s;
            let x =
                (k2 * s + m::sqrt(k2 * k2 * s * s + 4.0 * s * s * t * k2 * b)) / (2.0 * s * s * t);
            seen.distance_m / m::sqrt(x)
        } else {
            0.0
        };
        why.push(format!(
            "{:.3e} photons/s arrive in {:.0} to {:.0} nm against {:.3e}/s of background and dark counts; staring {:.0} s gives S/N {:.2}.",
            s,
            sensor.from_m * 1e9,
            sensor.to_m * 1e9,
            b,
            sensor.stare_s,
            snr
        ));
        (snr, need, range)
    };
    let mut detectable = snr >= sensor.threshold_snr;
    let mut range = range;
    let lost_in_glare = !detectable && glare_flux > flux;
    // Astrometry: the position to about the resolution over the S/N.
    let precision = (sensor.method == "astrometry" && snr > 0.0).then(|| {
        refs.push(Reference::parse(R_CENTROID));
        let p = res / snr;
        why.push(format!(
            "Its position is fixed to about {:.2e} rad (the resolution {:.2e} rad over S/N {:.1}).",
            p, res, snr
        ));
        p
    });
    // Active: the echo of what it sends, by the radar equation, with the
    // gain of its own aperture; the pulse is heard much further.
    let mut heard = None;
    if sensor.method == "active" {
        let (Some(pw), Some(sigma)) = (sensor.transmit_power_w, seen.radar_cross_section_m2) else {
            return Err(format!(
                "{} / {}: an active instrument needs transmit_power_w, and the target radar_cross_section_m2",
                seen.address, sensor.name
            ));
        };
        let gain = 4.0 * m::PI * sensor.area() / (lc * lc);
        let bw = sensor.bandwidth();
        let p_min = sensor.threshold_snr * K_B * sensor.system_temperature_k * bw
            / m::sqrt(bw * sensor.stare_s);
        let rr = radar_ranges(&Radar {
            power_w: pw,
            gain,
            wavelength_m: lc,
            cross_section_m2: sigma,
            min_power_w: p_min,
        });
        refs.extend(rr.references);
        why.extend(rr.why);
        why.push(format!(
            "Sending {:.3e} W draws that much from the electric bus, and lights the ship up for anyone in the beam.",
            pw
        ));
        range = rr.sees_to_m;
        detectable = seen.distance_m <= rr.sees_to_m;
        heard = Some(rr.heard_to_m);
    }
    why.push(if detectable {
        format!(
            "Detected (needs S/N {}); it would still be seen out to {:.3e} m.",
            sensor.threshold_snr, range
        )
    } else if lost_in_glare {
        "Not detected: lost in the glare of something brighter beside it.".into()
    } else {
        format!(
            "Not detected: it would need a {:.3e} s stare, or to be within {:.3e} m.",
            stare_needed, range
        )
    });
    if let Some(a) = sensor.authored_range_m {
        why.push(format!(
            "An older file typed its range as {:.3e} m; that is kept as authored, and the computed range above is the one used.",
            a
        ));
    }
    Ok(Detection {
        target: seen.address.clone(),
        sensor: sensor.name.clone(),
        method: sensor.method.clone(),
        channel: "light".into(),
        range_source: "computed".into(),
        authored_range_m: sensor.authored_range_m,
        precision_rad: precision,
        transmit_power_w: sensor
            .transmit_power_w
            .filter(|_| sensor.method == "active"),
        heard_to_m: heard,
        flux_w_m2: flux,
        extinction_mag: ext,
        snr,
        detectable,
        stare_needed_s: stare_needed,
        range_m: range,
        lost_in_glare,
        why,
        references: refs,
    })
}

/// An instrument with only a typed range: kept as authored, marked.
fn authored(seen: &Seen, sensor: &Sensor, channel: &str, r: f64) -> Detection {
    let detectable = seen.distance_m <= r;
    Detection {
        target: seen.address.clone(),
        sensor: sensor.name.clone(),
        method: sensor.method.clone(),
        channel: channel.into(),
        range_source: "authored".into(),
        authored_range_m: Some(r),
        precision_rad: None,
        transmit_power_w: None,
        heard_to_m: None,
        flux_w_m2: 0.0,
        extinction_mag: 0.0,
        snr: 0.0,
        detectable,
        stare_needed_s: if detectable { 0.0 } else { f64::INFINITY },
        range_m: r,
        lost_in_glare: false,
        why: vec![format!(
            "This instrument has only a typed range, {:.3e} m, kept as authored from an older file: the target is {} it. Give it an aperture (or a noise figure) and the range is computed from what the target gives off.",
            r,
            if detectable { "within" } else { "beyond" }
        )],
        references: vec![],
    }
}

/// Field channels: a magnetometer reads the target's dipole field, a
/// gradiometer its gravity gradient. Noise averages down as 1/√(2t), and the
/// channel's own wander here adds in quadrature.
fn detect_field(
    seen: &Seen,
    sensor: &Sensor,
    nature: Option<&Nature>,
) -> Result<Detection, String> {
    let magnetic = sensor.method == "magnetometer";
    let channel = if magnetic { "magnetic" } else { "gravity" };
    let Some(n) = sensor.noise_density.filter(|n| *n > 0.0) else {
        if let Some(r) = sensor.authored_range_m {
            return Ok(authored(seen, sensor, channel, r));
        }
        return Err(format!(
            "{}: a {} needs noise_density (its noise per √Hz)",
            sensor.name, sensor.method
        ));
    };
    let Some(k) = nature else {
        return Err(format!(
            "{}: the constants of nature (G, μ0) were not supplied from the data sheet",
            sensor.name
        ));
    };
    if seen.distance_m <= 0.0 || sensor.stare_s <= 0.0 {
        return Err(format!(
            "{} / {}: bad distance or stare",
            seen.address, sensor.name
        ));
    }
    // What the target gives off on this channel, as strength × r³.
    let (strength, unit, what, source) = if magnetic {
        (
            k.mu0 * seen.magnetic_moment_a_m2 / (4.0 * m::PI),
            "T",
            "its magnetic field",
            R_DIPOLE,
        )
    } else {
        (
            2.0 * k.g * seen.mass_kg,
            "s⁻²",
            "its gravity gradient",
            R_GRADIENT,
        )
    };
    let d = seen.distance_m;
    let signal = strength / (d * d * d);
    let sigma = n / m::sqrt(2.0 * sensor.stare_s);
    let noise = m::sqrt(sigma * sigma + sensor.field_clutter * sensor.field_clutter);
    let snr = signal / noise;
    let detectable = snr >= sensor.threshold_snr;
    let need_signal = sensor.threshold_snr * noise;
    let range = if strength > 0.0 {
        m::cbrt(strength / need_signal)
    } else {
        0.0
    };
    // Stare needed: only the instrument's own noise averages down.
    let floor = sensor.threshold_snr * sensor.field_clutter;
    let stare_needed = if signal > floor && signal > 0.0 {
        let s_allowed = m::sqrt(
            (signal / sensor.threshold_snr) * (signal / sensor.threshold_snr)
                - sensor.field_clutter * sensor.field_clutter,
        );
        n * n / (2.0 * s_allowed * s_allowed)
    } else {
        f64::INFINITY
    };
    let mut why = vec![format!(
        "{} reaches the {} as {:.3e} {} at {:.3e} m, against {:.3e} {} of noise ({:.3e} per √Hz over {:.0} s{}): S/N {:.2}.",
        what,
        sensor.name,
        signal,
        unit,
        d,
        noise,
        unit,
        n,
        sensor.stare_s,
        if sensor.field_clutter > 0.0 { ", with the channel's own wander here" } else { "" },
        snr
    )];
    why.push(format!(
        "It falls as the cube of distance, so this instrument reaches {:.3e} m{}",
        range,
        if magnetic {
            "."
        } else {
            ": detecting a ship by its mass is real, but only this close."
        }
    ));
    if let Some(a) = sensor.authored_range_m {
        why.push(format!(
            "An older file typed its range as {:.3e} m; that is kept as authored, and the computed range is the one used.",
            a
        ));
    }
    Ok(Detection {
        target: seen.address.clone(),
        sensor: sensor.name.clone(),
        method: sensor.method.clone(),
        channel: channel.into(),
        range_source: "computed".into(),
        authored_range_m: sensor.authored_range_m,
        precision_rad: None,
        transmit_power_w: None,
        heard_to_m: None,
        flux_w_m2: 0.0,
        extinction_mag: 0.0,
        snr,
        detectable,
        stare_needed_s: stare_needed,
        range_m: range,
        lost_in_glare: false,
        why,
        references: vec![Reference::parse(source), Reference::parse(R_NOISE)],
    })
}

fn sensor_neighbours(seen: &Seen, res: f64) -> Vec<&Neighbour> {
    seen.neighbours
        .iter()
        .filter(|n| n.separation_rad < res)
        .collect()
}

/// A setting's named slice of distance ("adjacent" to "far" in Traveller):
/// a labelled reading of a real distance, a measure of the setting and not a
/// sensor. Bands are read in order; a distance takes the first band it fits.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RangeBand {
    pub id: String,
    pub label: String,
    /// Upper edge, m.
    pub range_m: f64,
}

/// The band a distance falls in, or None beyond the last.
pub fn band_of(distance_m: f64, bands: &[RangeBand]) -> Option<&RangeBand> {
    let mut sorted: Vec<&RangeBand> = bands.iter().collect();
    sorted.sort_by(|a, b| a.range_m.total_cmp(&b.range_m));
    sorted.into_iter().find(|b| distance_m <= b.range_m)
}

/// A radar and who can hear it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Radar {
    pub power_w: f64,
    /// Antenna gain (linear).
    pub gain: f64,
    pub wavelength_m: f64,
    /// Target's radar cross-section, m².
    pub cross_section_m2: f64,
    /// Smallest power the receivers can pick out, W (the same for the radar and a listener).
    pub min_power_w: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RadarRanges {
    /// Furthest the radar can see the target by its echo.
    pub sees_to_m: f64,
    /// Furthest a listener with the same antenna, in the beam, hears the pulse.
    pub heard_to_m: f64,
    pub why: Vec<String>,
    pub references: Vec<Reference>,
}

pub fn radar_ranges(r: &Radar) -> RadarRanges {
    let fp3 = 4.0 * m::PI * 4.0 * m::PI * 4.0 * m::PI;
    let sees = m::pow(
        r.power_w * r.gain * r.gain * r.wavelength_m * r.wavelength_m * r.cross_section_m2
            / (fp3 * r.min_power_w),
        0.25,
    );
    // Listener with effective aperture G λ²/4π, in the main beam: P_i = P G · G λ² / ((4π)² R²).
    let heard = m::sqrt(
        r.power_w * r.gain * r.gain * r.wavelength_m * r.wavelength_m
            / (4.0 * m::PI * 4.0 * m::PI * r.min_power_w),
    );
    RadarRanges {
        sees_to_m: sees,
        heard_to_m: heard,
        why: vec![
            format!("The echo falls as the fourth power of distance, so the radar sees a {:.0} m² target out to {:.3e} m.", r.cross_section_m2, sees),
            format!("The pulse itself falls only as the square, so anyone in the beam with the same receiver hears it out to {:.3e} m: {:.0} times further.", heard, heard / sees),
        ],
        references: vec![Reference::parse(R_RADAR)],
    }
}

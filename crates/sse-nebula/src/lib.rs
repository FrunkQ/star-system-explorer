//! SSE4 nebulae (track O3; nebulae-and-light design section 3).
//!
//! A nebula is an addressable object with a cause, a birth date, a life and an
//! end. Each kind here is made by something the model already tracks: a dying
//! Sun-like star (the `planetary_nebula` event on its life line), a supernova
//! (a resolved star's or a cohort's), or hot young stars in gas. The kernel
//! publishes a small **descriptor** (shells with radii and speeds, lobes and
//! their axis, the surroundings, the lighting stars, light, a seed); fine
//! wisps are decoration drawn on the GPU from the seed and never facts.
//!
//! The physics is the cited blocks in `sse_atoms::nebula` (Strömgren sphere,
//! case B recombination light, constant-speed shells, the Sedov-Taylor and
//! snowplough remnant). This crate puts them together at a date.
#![forbid(unsafe_code)]

pub mod forbidden;
#[cfg(test)]
mod forbidden_tests;
pub mod outflow;
#[cfg(test)]
mod outflow_tests;
#[cfg(test)]
mod tests;

use serde::{Deserialize, Serialize};
use sse_atoms::nebula as nb;
use sse_core::draw::{Draws, Stream};
use sse_core::math as m;
use sse_core::{Address, Seed};
use sse_light::{LightDescription, LineEmission, Reference};

pub const YEAR_S: f64 = sse_core::quantity::consts::JULIAN_YEAR;
pub const PC_M: f64 = sse_core::sheet::PC;
pub const MSUN_KG: f64 = sse_core::sheet::M_SUN;
/// Hα/Hβ in case B at 10⁴ K (Osterbrock and Ferland 2006, table 4.2).
pub const HA_OVER_HB: f64 = 2.86;

const OF06: &str = "Osterbrock, D. E. and Ferland, G. J. 2006, Astrophysics of Gaseous Nebulae and Active Galactic Nuclei, 2nd edn, University Science Books https://uscibooks.aip.org/books/astrophysics-of-gaseous-nebulae-and-active-galactic-nuclei-second-edition/";
const BF02: &str = "Balick, B. and Frank, A. 2002, Shapes and shaping of planetary nebulae, ARA&A 40, 439: binary companions and fast spin give bipolar and butterfly shapes; concentric rings round many nebulae record earlier pulses of mass loss. doi:10.1146/annurev.astro.40.060401.093849";
const CMB88: &str = "Cioffi, D. F., McKee, C. F. and Bertschinger, E. 1988, Dynamics of radiative supernova remnants, ApJ 334, 252. doi:10.1086/166834";

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Kind {
    PlanetaryNebula,
    SupernovaRemnant,
    StarFormingGlow,
    /// A young star's bipolar jets (O3b).
    Jet,
    /// A cool giant's dusty wind (O3b).
    DustShell,
}

/// One shell, ring or ionised sphere.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Shell {
    pub label: String,
    pub radius_m: f64,
    pub speed_m_s: f64,
    pub thickness_m: f64,
    pub mass_kg: f64,
    /// Free electrons per m³ in the glowing gas (O4, VP-L1): given where the
    /// shell glows, absent for dust, rings and shocks that publish no light.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub n_e_m3: Option<f64>,
}

/// One line's colour to the eye (VP-L1): linear-light sRGB at unit
/// luminance (Y = 1), negative channels clipped to zero and the luminance
/// put back to one. Every pure line lies outside a screen's colours, so most
/// are clipped.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LineColour {
    pub line: String,
    pub rgb_linear: [f64; 3],
    /// True when a channel was clipped: a screen cannot show the line's
    /// colour, only the nearest it can.
    pub clipped: bool,
}

/// The middle row of a 3×3 matrix's inverse: the weights that give
/// luminance Y from linear sRGB, for the XYZ-to-sRGB matrix `a`.
fn luminance_row(a: &[[f64; 3]; 3]) -> [f64; 3] {
    let det = a[0][0] * (a[1][1] * a[2][2] - a[1][2] * a[2][1])
        - a[0][1] * (a[1][0] * a[2][2] - a[1][2] * a[2][0])
        + a[0][2] * (a[1][0] * a[2][1] - a[1][1] * a[2][0]);
    [
        (a[1][2] * a[2][0] - a[1][0] * a[2][2]) / det,
        (a[0][0] * a[2][2] - a[0][2] * a[2][0]) / det,
        (a[0][2] * a[1][0] - a[0][0] * a[1][2]) / det,
    ]
}

/// The eye colour of every line in a light description that the eye sees,
/// from the base light table's CIE observer.
pub fn line_colours(light: &LightDescription) -> Result<Vec<LineColour>, String> {
    let table = forbidden::light_table();
    let eye = table.eye.as_ref().ok_or("the light table has no eye")?;
    let mut out: Vec<LineColour> = Vec::new();
    for l in &light.lines {
        if out.iter().any(|c| c.line == l.line) {
            continue;
        }
        let Some(def) = table.line(&l.line) else {
            continue;
        };
        let nm = def.at_m * 1e9;
        if nm < eye.from_nm || nm > eye.to_nm {
            continue;
        }
        let xyz = sse_light::colour::response(eye, def.at_m)?;
        if xyz[1] <= 0.0 || xyz[1].is_nan() {
            continue;
        }
        let mut rgb = [0.0; 3];
        for (k, row) in eye.xyz_to_linear_srgb.iter().enumerate() {
            rgb[k] = (row[0] * xyz[0] + row[1] * xyz[1] + row[2] * xyz[2]) / xyz[1];
        }
        let clipped = rgb.iter().any(|v| *v < 0.0);
        for v in &mut rgb {
            *v = v.max(0.0);
        }
        // Back to unit luminance after clipping: Y of the clipped colour,
        // through the inverse of the eye's own XYZ-to-sRGB matrix.
        let y = luminance_row(&eye.xyz_to_linear_srgb)
            .iter()
            .zip(&rgb)
            .map(|(a, b)| a * b)
            .sum::<f64>();
        if y > 0.0 {
            for v in &mut rgb {
                *v /= y;
            }
        }
        out.push(LineColour {
            line: l.line.clone(),
            rgb_linear: rgb,
            clipped,
        });
    }
    Ok(out)
}

/// Two lobes along an axis (a unit vector in the system frame).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Lobes {
    pub axis: [f64; 3],
    /// Lobe length over waist radius.
    pub ratio: f64,
    pub why: String,
}

/// One knot in a jet: a blob from one burst of accretion, moving out.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Knot {
    pub distance_m: f64,
    pub age_s: f64,
}

/// A pair of jets along the lobes' axis.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Jet {
    /// Length of each jet from the star, m.
    pub length_m: f64,
    pub speed_m_s: f64,
    /// Full opening angle, rad.
    pub opening_rad: f64,
    /// Mass both jets carry away, kg/s.
    pub mass_rate_kg_s: f64,
    pub knots: Vec<Knot>,
}

/// What made it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Cause {
    pub event: String,
    /// The body whose event it was (the star that died, the cluster).
    pub source: String,
    pub at_s: f64,
}

/// The fact every device agrees on.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Descriptor {
    pub address: String,
    pub kind: Kind,
    pub cause: Cause,
    /// The date asked about, its age then, and the date it fades away.
    pub at_s: f64,
    pub age_s: f64,
    pub fades_s: f64,
    pub phase: String,
    pub shells: Vec<Shell>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lobes: Option<Lobes>,
    /// A young star's jets: their length, speed, opening and knots.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub jet: Option<Jet>,
    /// Elongation of a single shell (1 = round).
    pub elongation: f64,
    pub ambient_h_m3: f64,
    /// Bodies whose light ionises it.
    pub lighting: Vec<String>,
    /// Seed for decoration (wisps, knots); never a fact.
    pub decoration_seed: String,
    /// Each seen line's colour to the eye, for the renderer and the chart.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub line_colours: Vec<LineColour>,
    pub light: LightDescription,
    pub why: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub stand_ins: Vec<String>,
    pub references: Vec<Reference>,
}

fn reference(text: &str) -> Reference {
    Reference::parse(text)
}

fn draws(seed: &Seed, address: &str, stream: &str) -> Result<Draws, String> {
    let a: Address = address.parse().map_err(|e| format!("{address}: {e:?}"))?;
    Ok(Draws::new(*seed, a, Stream::new(stream).map_err(|e| e.0)?))
}

fn decoration_seed(seed: &Seed, address: &str) -> Result<String, String> {
    Ok(format!(
        "{:016x}",
        draws(seed, address, "nebula.decoration.v1")?.u64(0)
    ))
}

/// Light from recombining hydrogen: Hα and Hβ for this many recombinations a second.
fn balmer(d: &mut LightDescription, recombinations: f64) {
    let ha = nb::H_ALPHA_PER_RECOMBINATION * nb::H_ALPHA_J * recombinations;
    d.lines.push(LineEmission {
        line: "ha".into(),
        power_w: ha,
        cause: "hydrogen recombination (case B)".into(),
        zone: None,
    });
    d.lines.push(LineEmission {
        line: "hb".into(),
        power_w: ha / HA_OVER_HB,
        cause: "hydrogen recombination (case B)".into(),
        zone: None,
    });
    d.references.push(reference(OF06));
}

const METALS_LATER: &str = "metal lines are [O III] and [S II] only so far, at a fixed electron temperature with each ion's layer from photon counting (fudge register F61): nitrogen, neon, helium and [O II] lines, and the heating and cooling that set the temperature, come later in O4";

// ── Planetary nebulae ────────────────────────────────────────────────────

/// What a dying star gives a planetary nebula.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PnInput {
    /// The nebula's own address (by convention `<star>/nebula:0`).
    pub address: String,
    pub star: String,
    /// Date of the `planetary_nebula` event, s.
    pub died_at_s: f64,
    /// The newborn white dwarf: temperature and radius.
    pub core_teff_k: f64,
    pub core_radius_m: f64,
    /// Envelope mass thrown off at the end, kg.
    pub envelope_kg: f64,
    pub ambient_h_m3: f64,
    /// A close companion or fast spin shapes two lobes.
    #[serde(default)]
    pub companion: bool,
    #[serde(default)]
    pub fast_spin: bool,
    /// The companion's orbit normal, if known.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub axis: Option<[f64; 3]>,
}

/// White dwarf radius from its mass (Nauenberg 1972, zero-temperature
/// electron-degenerate star): R = 0.0112 R☉ √((M/M_Ch)^(-2/3) − (M/M_Ch)^(2/3)),
/// M_Ch = 1.44 M☉.
pub fn white_dwarf_radius_m(mass_msun: f64) -> f64 {
    let x = m::cbrt(mass_msun / 1.44);
    let x2 = x * x;
    let inner = 1.0 / x2 - x2;
    0.0112 * 6.957e8 * m::sqrt(inner.max(0.0))
}

/// A star's death on its life line, as the time track records it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Death {
    pub star: String,
    pub died_at_s: f64,
    pub initial_mass_msun: f64,
    pub remnant_mass_msun: f64,
    pub ambient_h_m3: f64,
    #[serde(default)]
    pub companion: bool,
}

/// The newborn central star's temperature, K (hot cores at the top of the
/// white dwarf track are about 10⁵ K; Bloecker 1995 tracks reach 1-2 × 10⁵ K).
pub const CORE_TEFF_K: f64 = 100_000.0;

impl PnInput {
    /// From a `planetary_nebula` event: the envelope is everything the star
    /// lost but its final wind took most of it, so the bright shell is the
    /// share set in the settings.
    pub fn from_death(d: &Death) -> Self {
        PnInput {
            address: format!("{}/nebula:0", d.star),
            star: d.star.clone(),
            died_at_s: d.died_at_s,
            core_teff_k: CORE_TEFF_K,
            core_radius_m: white_dwarf_radius_m(d.remnant_mass_msun),
            envelope_kg: (d.initial_mass_msun - d.remnant_mass_msun).max(0.0) * MSUN_KG,
            ambient_h_m3: d.ambient_h_m3,
            companion: d.companion,
            fast_spin: false,
            axis: None,
        }
    }
}

/// Settings (pack `provides.other.nebulae.planetary`).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PnSettings {
    /// Swept shell speed, km/s (observed tens of km/s).
    pub speed_km_s: [f64; 2],
    /// Share of the shed envelope in the bright shell (the rest left earlier in the slow giant wind).
    pub shell_share: f64,
    /// Shell thickness over radius.
    pub thickness: f64,
    /// It fades when the shell is only this many times denser than its surroundings.
    pub fade_contrast: f64,
    /// The slow giant wind's speed and the spacing of its earlier pulses (rings).
    pub slow_wind_km_s: f64,
    pub pulse_years: [f64; 2],
    pub max_rings: u64,
}

impl Default for PnSettings {
    fn default() -> Self {
        PnSettings {
            speed_km_s: [15.0, 35.0],
            shell_share: 0.3,
            thickness: 0.2,
            fade_contrast: 10.0,
            slow_wind_km_s: 15.0,
            pulse_years: [500.0, 1500.0],
            max_rings: 4,
        }
    }
}

fn unit_vector(d: &Draws, i: u64) -> [f64; 3] {
    let z = d.uniform(i, -1.0, 1.0);
    let phi = d.uniform(i + 1, 0.0, m::TAU);
    let s = m::sqrt(1.0 - z * z);
    [s * m::cos(phi), s * m::sin(phi), z]
}

/// Radius at which a planetary nebula's shell of `shell_kg` has thinned to
/// `fade_contrast` times the surrounding gas (density M / (mu m_H 4 pi R^2 dR)).
pub fn planetary_fade_radius_m(shell_kg: f64, ambient_h_m3: f64, s: &PnSettings) -> f64 {
    let per_h = nb::MU_H * nb::M_H;
    m::cbrt(shell_kg / (per_h * 4.0 * m::PI * s.thickness * s.fade_contrast * ambient_h_m3))
}

/// How long a typical planetary nebula stays visible, s: a star that sheds
/// `envelope_kg` into gas of `ambient_h_m3`, its shell coasting at the middle
/// of the settings' speed range.
pub fn planetary_fades_after(envelope_kg: f64, ambient_h_m3: f64, s: &PnSettings) -> f64 {
    let v = 0.5 * (s.speed_km_s[0] + s.speed_km_s[1]) * 1e3;
    planetary_fade_radius_m(envelope_kg * s.shell_share, ambient_h_m3, s) / v
}

/// The nebula a dying star leaves, as it is at `at_s`; `None` before the
/// star dies or after the nebula has faded into its surroundings.
pub fn planetary_nebula(
    seed: &Seed,
    p: &PnInput,
    s: &PnSettings,
    at_s: f64,
) -> Result<Option<Descriptor>, String> {
    let age = at_s - p.died_at_s;
    if age < 0.0 {
        return Ok(None);
    }
    let d = draws(seed, &p.address, "nebula.planetary.v1")?;
    let v = d.uniform(0, s.speed_km_s[0], s.speed_km_s[1]) * 1e3;
    let mass = p.envelope_kg * s.shell_share;
    // Fades when shell density M / (μ m_H 4π R² ΔR) reaches fade_contrast × ambient.
    let per_h = nb::MU_H * nb::M_H;
    let t_fade = planetary_fade_radius_m(mass, p.ambient_h_m3, s) / v;
    if age > t_fade {
        return Ok(None);
    }
    let r = v * age;
    let shell_volume = 4.0 * m::PI * r * r * s.thickness * r;
    let n = if shell_volume > 0.0 {
        mass / (per_h * shell_volume)
    } else {
        0.0
    };
    let q = nb::ionising_photons(p.core_teff_k, p.core_radius_m, nb::E_ION_H);
    let needed = nb::ALPHA_B * n * n * shell_volume;
    let recombinations = q.min(needed);
    let mut light = LightDescription::new(&p.address);
    balmer(&mut light, recombinations);
    let metal_why = forbidden::add_forbidden_lines(
        &mut light,
        &[(p.core_teff_k, p.core_radius_m)],
        n,
        recombinations,
        forbidden::Layout::Shell { radius_m: r },
    )?;
    let mut why = vec![format!(
        "Shed by {} {:.0} years ago; the shell coasts out at {:.0} km/s and is now {:.2} pc across.",
        p.star,
        age / YEAR_S,
        v / 1e3,
        2.0 * r / PC_M
    )];
    why.push(if q >= needed {
        format!(
            "The white dwarf ({:.0} K) gives {:.1e} ionising photons a second, more than the shell's {:.1e} recombinations: the whole shell glows, and starlight escapes beyond it.",
            p.core_teff_k, q, needed
        )
    } else {
        format!(
            "The white dwarf ({:.0} K) gives {:.1e} ionising photons a second, fewer than the {:.1e} recombinations the dense young shell would need: only its inner part glows.",
            p.core_teff_k, q, needed
        )
    });
    let mut shells = vec![Shell {
        label: "bright shell".into(),
        radius_m: r,
        speed_m_s: v,
        thickness_m: s.thickness * r,
        mass_kg: mass,
        n_e_m3: Some(n * (1.0 + forbidden::per_hydrogen("he")?)),
    }];
    // Shed layers: earlier pulses of the slow giant wind, each a faint ring.
    let rings = d.below(2, s.max_rings + 1);
    let pulse = d.uniform(3, s.pulse_years[0], s.pulse_years[1]) * YEAR_S;
    let slow = s.slow_wind_km_s * 1e3;
    for k in 1..=rings {
        let rk = slow * (age + k as f64 * pulse);
        if rk > r {
            shells.push(Shell {
                label: format!("halo ring {k}"),
                radius_m: rk,
                speed_m_s: slow,
                thickness_m: 0.05 * rk,
                mass_kg: 0.0,
                n_e_m3: None,
            });
        }
    }
    why.extend(metal_why);
    if rings > 0 {
        why.push(format!(
            "{rings} faint rings outside it are earlier puffs of the giant's slow wind, {:.0} years apart: the shed layers.",
            pulse / YEAR_S
        ));
    }
    let lobes = if p.companion || p.fast_spin {
        let axis = p.axis.unwrap_or_else(|| unit_vector(&d, 4));
        let ratio = d.uniform(6, 2.0, 4.0);
        let by = if p.companion {
            "a close companion"
        } else {
            "fast spin"
        };
        why.push(format!(
            "{by} squeezes the outflow at the waist, so it blows out two lobes {ratio:.1} times longer than wide."
        ));
        Some(Lobes {
            axis,
            ratio,
            why: format!("shaped by {by}"),
        })
    } else {
        None
    };
    let elongation = if lobes.is_some() {
        1.0
    } else {
        d.uniform(7, 1.0, 1.5)
    };
    Ok(Some(Descriptor {
        address: p.address.clone(),
        kind: Kind::PlanetaryNebula,
        cause: Cause {
            event: "planetary_nebula".into(),
            source: p.star.clone(),
            at_s: p.died_at_s,
        },
        at_s,
        age_s: age,
        fades_s: p.died_at_s + t_fade,
        phase: if q >= needed { "fully lit" } else { "partly lit" }.into(),
        shells,
        lobes,
        jet: None,
        elongation,
        ambient_h_m3: p.ambient_h_m3,
        lighting: vec![p.star.clone()],
        decoration_seed: decoration_seed(seed, &p.address)?,
        line_colours: line_colours(&light)?,
        light,
        why,
        stand_ins: vec![
            METALS_LATER.into(),
            "the white dwarf's ionising light is held at its birth temperature; its cooling (and the nebula dimming with it) comes when the remnant track gives the cooling curve".into(),
            "the shell is uniform: no clumps or density gradient in the surroundings".into(),
        ],
        references: vec![
            reference("Kwok, S., Purton, C. R. and Fitzgerald, P. M. 1978, On the origin of planetary nebulae, ApJ 219, L125. doi:10.1086/182621"),
            reference(BF02),
            reference(OF06),
        ],
    }))
}

// ── Supernova remnants ───────────────────────────────────────────────────

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SnrInput {
    pub address: String,
    pub star: String,
    pub exploded_at_s: f64,
    /// Core collapse (leaves a neutron star or black hole) or thermonuclear (none).
    pub core_collapse: bool,
    pub energy_j: f64,
    pub ejecta_kg: f64,
    pub ambient_h_m3: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub remnant: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SnrSettings {
    /// It merges with its surroundings when the shell has slowed to this, km/s.
    pub merge_km_s: f64,
}

impl Default for SnrSettings {
    fn default() -> Self {
        SnrSettings { merge_km_s: 10.0 }
    }
}

/// Shock radius and speed at age `t`.
pub fn remnant_motion(p: &SnrInput, t: f64) -> (f64, f64, &'static str) {
    let (r, phase) = nb::remnant_radius(p.energy_j, p.ejecta_kg, p.ambient_h_m3, t);
    let v = match phase {
        "free expansion" => m::sqrt(2.0 * p.energy_j / p.ejecta_kg),
        "Sedov-Taylor" => 0.4 * r / t,
        _ => {
            let e51 = p.energy_j / 1e44;
            let t_pds = 1.33e4
                * YEAR_S
                * m::pow(e51, 3.0 / 14.0)
                * m::pow(p.ambient_h_m3 * 1e-6, -4.0 / 7.0);
            let x = 4.0 * t / (3.0 * t_pds) - 1.0 / 3.0;
            r * 0.3 * (4.0 / (3.0 * t_pds)) / x
        }
    };
    (r, v, phase)
}

/// When the shell has slowed to the merge speed: bisection in ln t, 200 steps.
pub fn remnant_fades_after(p: &SnrInput, s: &SnrSettings) -> f64 {
    let target = s.merge_km_s * 1e3;
    let (mut lo, mut hi) = (m::ln(YEAR_S), m::ln(1e9 * YEAR_S));
    for _ in 0..200 {
        let mid = 0.5 * (lo + hi);
        if remnant_motion(p, m::exp(mid)).1 > target {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    m::exp(0.5 * (lo + hi))
}

pub fn supernova_remnant(
    seed: &Seed,
    p: &SnrInput,
    s: &SnrSettings,
    at_s: f64,
) -> Result<Option<Descriptor>, String> {
    let age = at_s - p.exploded_at_s;
    if age <= 0.0 {
        return Ok(None);
    }
    let t_fade = remnant_fades_after(p, s);
    if age > t_fade {
        return Ok(None);
    }
    let (r, v, phase) = remnant_motion(p, age);
    let swept = 4.0 / 3.0 * m::PI * r * r * r * nb::MU_H * nb::M_H * p.ambient_h_m3;
    let d = draws(seed, &p.address, "nebula.remnant.v1")?;
    let mut why = vec![format!(
        "{} exploded {:.0} years ago with {:.1e} J; the blast is in its {} phase, {:.1} pc across and moving at {:.0} km/s.",
        p.star,
        age / YEAR_S,
        p.energy_j,
        phase,
        2.0 * r / PC_M,
        v / 1e3
    )];
    why.push(format!(
        "It has swept up {:.1} solar masses of gas and will merge with its surroundings in {:.0} years, when it has slowed to {:.0} km/s.",
        swept / MSUN_KG,
        (t_fade - age) / YEAR_S,
        s.merge_km_s
    ));
    let mut lighting = vec![];
    if let Some(rem) = &p.remnant {
        why.push(format!("The collapsed core, {rem}, sits at its centre."));
        lighting.push(rem.clone());
    }
    Ok(Some(Descriptor {
        address: p.address.clone(),
        kind: Kind::SupernovaRemnant,
        cause: Cause {
            event: if p.core_collapse { "supernova_cc" } else { "supernova_ia" }.into(),
            source: p.star.clone(),
            at_s: p.exploded_at_s,
        },
        at_s,
        age_s: age,
        fades_s: p.exploded_at_s + t_fade,
        phase: phase.into(),
        shells: vec![Shell {
            label: "blast wave".into(),
            radius_m: r,
            speed_m_s: v,
            thickness_m: 0.08 * r,
            mass_kg: swept + p.ejecta_kg,
            n_e_m3: None,
        }],
        lobes: None,
        jet: None,
        elongation: if p.core_collapse { d.uniform(0, 1.0, 1.3) } else { d.uniform(0, 1.0, 1.05) },
        ambient_h_m3: p.ambient_h_m3,
        lighting,
        decoration_seed: decoration_seed(seed, &p.address)?,
        line_colours: Vec::new(),
        light: LightDescription::new(&p.address),
        why,
        stand_ins: vec![
            "no light yet: shocked-gas lines, the hot X-ray interior and the radio shell come from the shock grid in O4".into(),
            "uniform surroundings: no bulging into thin gas or flattening against dense gas".into(),
        ],
        references: vec![
            reference("Taylor, G. I. 1950, The formation of a blast wave by a very intense explosion, Proc. R. Soc. A 201, 159. doi:10.1098/rspa.1950.0049"),
            reference(CMB88),
        ],
    }))
}

// ── Star-forming glows (H II regions) ────────────────────────────────────

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct HotStar {
    pub address: String,
    pub teff_k: f64,
    pub radius_m: f64,
    /// When it was born and when it dies (s), from its life line.
    pub born_s: f64,
    pub dies_s: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GlowInput {
    pub address: String,
    pub cluster: String,
    pub stars: Vec<HotStar>,
    pub cloud_h_m3: f64,
}

pub fn star_forming_glow(
    seed: &Seed,
    p: &GlowInput,
    at_s: f64,
) -> Result<Option<Descriptor>, String> {
    let live: Vec<&HotStar> = p
        .stars
        .iter()
        .filter(|s| s.born_s <= at_s && at_s < s.dies_s)
        .collect();
    let q: f64 = live
        .iter()
        .map(|s| nb::ionising_photons(s.teff_k, s.radius_m, nb::E_ION_H))
        .sum();
    if q <= 0.0 {
        return Ok(None);
    }
    let born = live.iter().map(|s| s.born_s).fold(f64::INFINITY, f64::min);
    let fades = p
        .stars
        .iter()
        .filter(|s| nb::ionising_photons(s.teff_k, s.radius_m, nb::E_ION_H) > 0.0)
        .map(|s| s.dies_s)
        .fold(0.0, f64::max);
    let r = nb::stromgren_radius(q, p.cloud_h_m3);
    let mut light = LightDescription::new(&p.address);
    balmer(&mut light, q);
    let lit: Vec<(f64, f64)> = live.iter().map(|s| (s.teff_k, s.radius_m)).collect();
    let metal_why = forbidden::add_forbidden_lines(
        &mut light,
        &lit,
        p.cloud_h_m3,
        q,
        forbidden::Layout::Sphere,
    )?;
    let mut why = vec![
        format!(
            "{} hot star{} in {} give {:.1e} ionising photons a second.",
            live.len(),
            if live.len() == 1 { "" } else { "s" },
            p.cluster,
            q
        ),
        format!(
            "In gas of {:.0} hydrogen atoms per cm³ they keep a sphere {:.1} pc across ionised (the Strömgren sphere), glowing red in hydrogen alpha. It goes dark when the last of these stars dies.",
            p.cloud_h_m3 * 1e-6,
            2.0 * r / PC_M
        ),
    ];
    why.extend(metal_why);
    Ok(Some(Descriptor {
        address: p.address.clone(),
        kind: Kind::StarFormingGlow,
        cause: Cause {
            event: "hot_stars_born".into(),
            source: p.cluster.clone(),
            at_s: born,
        },
        at_s,
        age_s: at_s - born,
        fades_s: fades,
        phase: "ionised".into(),
        shells: vec![Shell {
            label: "ionised sphere".into(),
            radius_m: r,
            speed_m_s: 0.0,
            thickness_m: r,
            mass_kg: 4.0 / 3.0 * m::PI * r * r * r * nb::MU_H * nb::M_H * p.cloud_h_m3,
            n_e_m3: Some(p.cloud_h_m3 * (1.0 + forbidden::per_hydrogen("he")?)),
        }],
        lobes: None,
        jet: None,
        elongation: 1.0,
        ambient_h_m3: p.cloud_h_m3,
        lighting: live.iter().map(|s| s.address.clone()).collect(),
        decoration_seed: decoration_seed(seed, &p.address)?,
        line_colours: line_colours(&light)?,
        light,
        why,
        stand_ins: vec![
            METALS_LATER.into(),
            "the cloud is uniform and the sphere fills at once: no expansion, blister break-out, pillars or dust lanes yet".into(),
            "hot stars' ionising light from a black body at their effective temperature".into(),
        ],
        references: vec![
            reference("Strömgren, B. 1939, The physical state of interstellar hydrogen, ApJ 89, 526. doi:10.1086/144074"),
            reference(OF06),
        ],
    }))
}

// ── Unvisited regions: how many there are ────────────────────────────────

/// Expected number of nebulae of one kind in a volume: the cause's rate times
/// how long each lasts (a steady state).
pub fn expected_count(rate_per_pc3_gyr: f64, lifetime_s: f64, volume_pc3: f64) -> f64 {
    rate_per_pc3_gyr * volume_pc3 * lifetime_s / (1e9 * YEAR_S)
}

/// Expected supernova remnants in a patch volume, from its core-collapse and
/// thermonuclear rates (sse-cosmos patch conditions) and the lifetime of a
/// typical remnant in gas of `ambient_h_m3`.
pub fn expected_remnants(
    cc_per_pc3_gyr: f64,
    ia_per_pc3_gyr: f64,
    ambient_h_m3: f64,
    volume_pc3: f64,
    s: &SnrSettings,
) -> (f64, f64) {
    let typical = |cc: bool| SnrInput {
        address: String::new(),
        star: String::new(),
        exploded_at_s: 0.0,
        core_collapse: cc,
        energy_j: 1e44,
        ejecta_kg: if cc { 5.0 } else { 1.4 } * MSUN_KG,
        ambient_h_m3,
        remnant: None,
    };
    (
        expected_count(
            cc_per_pc3_gyr,
            remnant_fades_after(&typical(true), s),
            volume_pc3,
        ),
        expected_count(
            ia_per_pc3_gyr,
            remnant_fades_after(&typical(false), s),
            volume_pc3,
        ),
    )
}

//! `describe_star` (K18, editing design section 9): one star read from the
//! same life line formation draws, with no system and no events. Mass, age
//! and [Fe/H] in; the phase now, its temperature, luminosity and radius, each
//! phase's ages, the remnant it ends as, and the mass walls where the answer
//! changes kind (for the star chooser's mass strip and age bar).

use crate::history::Z_FITS;
use crate::hurley::Fits;
use crate::inherit::z_of_feh;
use crate::lifeline::{
    Level, LifeLine, OutOfRange, Phase, BROWN_DWARF_MIN, MASS_RANGE, M_BH, M_UP_WD,
};
use serde::Serialize;
use sse_core::sheet::{L_SUN, M_SUN, R_SUN};

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Now {
    pub phase: Phase,
    pub label: &'static str,
    pub teff_k: f64,
    pub luminosity_w: f64,
    pub radius_m: f64,
    pub mass_kg: f64,
    pub luminosity_lsun: f64,
    pub radius_rsun: f64,
    pub mass_msun: f64,
    pub level: Level,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Span {
    pub phase: Phase,
    pub label: &'static str,
    /// Ages from birth, years.
    pub from_yr: f64,
    /// None: it lasts (a remnant or a brown dwarf).
    pub to_yr: Option<f64>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Remnant {
    pub phase: Phase,
    pub label: &'static str,
    pub mass_msun: f64,
    /// Age from birth at which it is left, years (None for a brown dwarf).
    pub from_yr: Option<f64>,
}

/// Initial masses (solar) where the answer changes kind.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Walls {
    /// Below: a planetary-mass object, not described.
    pub deuterium_msun: f64,
    /// Below: a brown dwarf (never fuses hydrogen).
    pub hydrogen_msun: f64,
    /// Below: ends as a white dwarf.
    pub white_dwarf_msun: f64,
    /// Below: a neutron star; above, a black hole.
    pub black_hole_msun: f64,
    /// Above: outside the stellar fits.
    pub fits_msun: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Description {
    pub mass_msun: f64,
    pub feh: f64,
    pub z: f64,
    /// The Z the fits were read at: the fits cover 1e-4 to 0.03, and a Z
    /// outside is read at the nearest edge (as formation does).
    pub z_fits: f64,
    pub age_yr: f64,
    pub now: Now,
    pub phases: Vec<Span>,
    pub remnant: Remnant,
    pub walls: Walls,
    pub notes: Vec<String>,
    pub sources: Vec<&'static str>,
}

const MYR: f64 = 1.0e6;

pub fn walls() -> Walls {
    Walls {
        deuterium_msun: BROWN_DWARF_MIN,
        hydrogen_msun: MASS_RANGE.0,
        white_dwarf_msun: M_UP_WD,
        black_hole_msun: M_BH,
        fits_msun: MASS_RANGE.1,
    }
}

/// One star of `mass_msun` at `age_yr` from birth with `feh` ([Fe/H]).
pub fn describe(mass_msun: f64, age_yr: f64, feh: f64) -> Result<Description, OutOfRange> {
    if age_yr.is_nan() || age_yr < 0.0 {
        return Err(OutOfRange(format!(
            "age {age_yr} years: give an age from birth, 0 or more"
        )));
    }
    let z = z_of_feh(feh);
    let z_fits = z.clamp(Z_FITS.0, Z_FITS.1);
    let fits = Fits::new(z_fits);
    let line = LifeLine::new(&fits, mass_msun)?;
    let t = age_yr / MYR;
    let s = line.state_at(&fits, t);
    let mut notes = vec![];
    if z_fits != z {
        notes.push(format!(
            "Z = {z:.2e} is outside the stellar fits (1e-4 to 0.03): read at {z_fits}"
        ));
    }
    if mass_msun < 0.1 && !line.is_substellar() {
        notes.push("0.08 to 0.1 solar masses is a short extrapolation of the fits".into());
    }
    let yr = |myr: f64| myr * MYR;
    let (phases, from) = if line.is_substellar() {
        (
            vec![Span {
                phase: Phase::BrownDwarf,
                label: Phase::BrownDwarf.label(),
                from_yr: 0.0,
                to_yr: None,
            }],
            None,
        )
    } else {
        let span = |p: Phase, a: f64, b: Option<f64>| Span {
            phase: p,
            label: p.label(),
            from_yr: yr(a),
            to_yr: b.map(yr),
        };
        // Phases a star skips (a red dwarf's giant branch, a stripped star's
        // red giant) are left out rather than listed with no length.
        let mut v = vec![
            span(Phase::PreMainSequence, 0.0, Some(line.t_zams)),
            span(Phase::MainSequence, line.t_zams, Some(line.t_tms)),
        ];
        for (p, a, b) in [
            (Phase::HertzsprungGap, line.t_tms, line.t_bgb),
            (Phase::Giant, line.t_bgb, line.t_strip),
            (Phase::WolfRayet, line.t_strip, line.t_end),
        ] {
            if b > a {
                v.push(span(p, a, Some(b)));
            }
        }
        v.push(span(line.remnant, line.t_end, None));
        (v, Some(yr(line.t_end)))
    };
    Ok(Description {
        mass_msun,
        feh,
        z,
        z_fits,
        age_yr,
        now: Now {
            phase: s.phase,
            label: s.phase.label(),
            teff_k: s.teff,
            luminosity_w: s.luminosity * L_SUN,
            radius_m: s.radius * R_SUN,
            mass_kg: s.mass * M_SUN,
            luminosity_lsun: s.luminosity,
            radius_rsun: s.radius,
            mass_msun: s.mass,
            level: s.level,
        },
        phases,
        remnant: Remnant {
            phase: line.remnant,
            label: line.remnant.label(),
            mass_msun: line.remnant_mass,
            from_yr: from,
        },
        walls: walls(),
        notes,
        sources: vec![
            "Hurley, Pols and Tout 2000, MNRAS 315, 543 (main sequence, calculated; other phases sketch rules on the same fits)",
            "Cummings et al. 2018, ApJ 866, 21 (white dwarf masses)",
            "Burrows et al. 2001, Rev. Mod. Phys. 73, 719 (brown dwarf cooling)",
        ],
    })
}

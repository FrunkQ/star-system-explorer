//! CS1: a world's sunlight and year from its own star.
//!
//! - Where the world is round its star: its own orbit, or for a moon its
//!   planet's (`Body::star_au`, `Body::star_e`), so a moon's seasons follow
//!   its planet's year and its distance from the star is its planet's.
//! - How bright the star is: its catalogued luminosity when it was measured
//!   (`Anchor::luminosity_lsun`), carried through time by its life line;
//!   for a generated star, the life line itself. Companion stars add their
//!   own light ([`crate::OtherLight`]).
//! - The arithmetic is the formula blocks `sse.climate.insolation`
//!   (S = L / 4πd²) and `sse.orbit.period` (Kepler), and the Sun's
//!   luminosity and the masses come from the data sheet, all read through
//!   the physics in force (ER). Outside a kernel call nothing is in force,
//!   and the climate says it assumed a sun-like star instead.

use crate::{Body, Climate};
use sse_core::sheet::{AU, DAY};
use sse_flow::Atom;
use sse_formation::hurley::Fits;

/// The world's orbit round its star (a moon's is its planet's).
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct StarPlace {
    /// Semi-major axis, AU.
    pub d_au: f64,
    pub e: f64,
    /// The star's mass, solar masses.
    pub star_msun: f64,
    /// Everything else going round the star with the world: a moon's
    /// planet, solar masses (zero for a planet).
    pub with_msun: f64,
}

/// Run one formula block of the physics in force. The blocks read here
/// have no settings; ER2's `physics::run` will be the door for any that do.
fn block(id: &str, inputs: &[f64]) -> Option<f64> {
    let physics = sse_physics::now()?;
    let a = physics.blocks().iter().find(|a| a.decl().id == id)?;
    a.eval(inputs, &[])
        .ok()?
        .first()
        .copied()
        .filter(|x| x.is_finite())
}

/// The star's mass from the life line the world's light follows.
fn star_msun(b: &Body) -> Option<f64> {
    match &b.climate {
        Climate::Star { life, .. } => Some(life.initial_mass),
        Climate::Fixed { then: Some(an), .. } => Some(an.life.initial_mass),
        Climate::Fixed { then: None, .. } => None,
    }
}

/// Where `b` is round its star. A planet's orbit is round the star; a
/// moon's (`star_au` set) is round its planet, which carries it round the
/// star. None when the star is not known (a moon with no star mass).
pub fn star_place(b: &Body) -> Option<StarPlace> {
    let o = b.orbit.as_ref();
    match b.star_au {
        Some(d) => Some(StarPlace {
            d_au: d,
            e: b.star_e.unwrap_or(0.0),
            star_msun: star_msun(b)?,
            with_msun: o.map(|o| o.host_msun).unwrap_or(0.0),
        }),
        None => {
            let o = o?;
            Some(StarPlace {
                d_au: o.a_au,
                e: o.e,
                star_msun: star_msun(b).unwrap_or(o.host_msun),
                with_msun: 0.0,
            })
        }
    }
}

/// The Sun's and the Earth's masses in force, kg.
fn masses() -> Option<(f64, f64)> {
    Some((
        sse_physics::constant("M_sun")?,
        sse_physics::constant("M_earth")?,
    ))
}

/// Kepler's period, days, for an orbit of `a_au` round `msun` solar
/// masses plus a body of `body_earth` Earth masses (`sse.orbit.period`),
/// and whether nothing was in force to run it with. Then it falls back to
/// Kepler's law in years and AU, P = √(a³ / M), without the body's mass,
/// and the caller says so.
pub fn period_days(a_au: f64, msun: f64, body_earth: f64) -> (f64, bool) {
    let run = masses().and_then(|(m_sun, m_earth)| {
        block(
            "sse.orbit.period",
            &[a_au * AU, msun * m_sun + body_earth * m_earth],
        )
    });
    match run {
        Some(p) => (p / DAY, false),
        None => (
            365.256_363 * sse_core::math::sqrt(a_au * a_au * a_au / msun.max(1e-12)),
            true,
        ),
    }
}

/// The world's year, days: once round its star, with everything that goes
/// round with it.
pub fn year_days(b: &Body, p: &StarPlace) -> (f64, bool) {
    period_days(p.d_au, p.star_msun + p.with_msun, b.mass_earth)
}

/// Once round what the world orbits, days (a moon's month, a planet's
/// year): a locked world's day.
pub fn orbit_days(b: &Body) -> Option<(f64, bool)> {
    let o = b.orbit.as_ref()?;
    Some(period_days(o.a_au, o.host_msun, b.mass_earth))
}

/// Starlight arriving at `b`, W/m² facing the star, at `t_gyr` after the
/// world formed: each star's own luminosity at that age over its distance
/// squared (`sse.climate.insolation`). None when nothing says how bright
/// the star is.
pub fn starlight_w_m2(b: &Body, t_gyr: f64) -> Option<f64> {
    let l_sun = sse_physics::constant("L_sun")?;
    // Nothing in force: no block to run, so say so rather than guess.
    block("sse.climate.insolation", &[l_sun, AU])?;
    let at = |l_lsun: f64, d_au: f64| -> f64 {
        if l_lsun <= 0.0 || d_au <= 0.0 {
            return 0.0;
        }
        block("sse.climate.insolation", &[l_lsun * l_sun, d_au * AU]).unwrap_or(0.0)
    };
    match &b.climate {
        Climate::Star {
            life,
            z,
            a_au,
            formed_myr,
            others,
            ..
        } => {
            let fits = Fits::new(*z);
            let age = formed_myr + t_gyr * 1000.0;
            let d = b.star_au.unwrap_or(*a_au);
            let mut s = at(life.state_at(&fits, age).luminosity, d);
            for o in others {
                s += at(o.life.state_at(&fits, age).luminosity, o.d_au);
            }
            Some(s)
        }
        Climate::Fixed { then: Some(an), .. } => {
            let fits = Fits::new(an.z);
            let then = an.life.state_at(&fits, an.measured_myr).luminosity;
            let now = an
                .life
                .state_at(&fits, an.formed_myr + t_gyr * 1000.0)
                .luminosity;
            // The measured light, carried by the life line's change since.
            let l = match an.luminosity_lsun {
                Some(l) if then > 0.0 => l * now / then,
                Some(l) => l,
                None => now,
            };
            Some(at(l, star_place(b)?.d_au))
        }
        Climate::Fixed { then: None, .. } => None,
    }
}

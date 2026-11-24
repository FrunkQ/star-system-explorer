//! W9 part 2: the orbit history a surveyed world reads its ice ages from.
//!
//! A planet's orbit wobbles because its siblings pull on it: T1's
//! Laplace-Lagrange secular theory (`sse_time::secular`) over the planets
//! that share its star gives its eccentricity, perihelion, inclination and
//! node through the window. Formation does not publish inclinations, nodes or
//! perihelia, so those are drawn from each planet's address (up to
//! `forcing.drawn_inclination_deg`, angles uniform) and the track's source
//! says so. A world with no siblings, or a moon, gets no track (no cycles).
//! Its moons pull on its bulge (the β of the precession constant).

use sse_climate::forcing::{MoonPull, OrbitTrack, TrackSample};
use sse_core::draw::{Draws, Seed, Stream};
use sse_core::math as m;
use sse_surface::Body;

const M_EARTH_MSUN: f64 = sse_core::sheet::GM_EARTH / sse_core::sheet::GM_SUN;
const M_EARTH_KG: f64 = sse_core::sheet::M_EARTH;
const AU_M: f64 = sse_core::sheet::AU;

/// The orbit track and moons for `body` among `bodies` (one system).
pub(crate) fn orbit_inputs(
    cs: &sse_climate::ClimateSettings,
    seed: &Seed,
    bodies: &[Body],
    body: &Body,
) -> (Option<OrbitTrack>, Vec<MoonPull>) {
    let moons: Vec<MoonPull> = bodies
        .iter()
        .filter(|b| {
            // Nested under the world, or a sibling whose orbit's host is the
            // world's own mass (an import that lists moons flat).
            let own = body.mass_earth * M_EARTH_MSUN;
            b.address.parent().as_ref() == Some(&body.address)
                || (b.address != body.address
                    && b.mass_earth < body.mass_earth
                    && b.orbit
                        .as_ref()
                        .is_some_and(|o| (o.host_msun - own).abs() <= 1e-3 * own))
        })
        .filter_map(|b| {
            let o = b.orbit.as_ref()?;
            Some(MoonPull {
                mass_kg: b.mass_earth * M_EARTH_KG,
                a_m: o.a_au * AU_M,
            })
        })
        .collect();
    let Some(own) = body.orbit.as_ref() else {
        return (None, moons);
    };
    if body
        .address
        .segments()
        .last()
        .is_none_or(|s| s.kind() != "body")
    {
        return (None, moons);
    }
    let parent = body.address.parent();
    let sibs: Vec<&Body> = bodies
        .iter()
        .filter(|b| {
            b.address.parent() == parent
                && b.address
                    .segments()
                    .last()
                    .is_some_and(|s| s.kind() == "body")
                && b.orbit
                    .as_ref()
                    .is_some_and(|o| o.host_msun == own.host_msun && o.a_au > 0.0)
        })
        .collect();
    if sibs.len() < 2 {
        return (None, moons);
    }
    let f = &cs.forcing;
    let stream = Stream::new("climate.orbit.v1").expect("valid");
    let planets: Vec<sse_time::secular::Planet> = sibs
        .iter()
        .map(|b| {
            let d = Draws::new(*seed, b.address.clone(), stream.clone());
            let o = b.orbit.as_ref().expect("filtered");
            sse_time::secular::Planet {
                mass: b.mass_earth * M_EARTH_MSUN,
                a: o.a_au,
                e: o.e,
                varpi: d.uniform(0, 0.0, m::TAU),
                i: d.uniform(1, 0.0, f.drawn_inclination_deg) * m::PI / 180.0,
                node: d.uniform(2, 0.0, m::TAU),
            }
        })
        .collect();
    let me = sibs
        .iter()
        .position(|b| b.address == body.address)
        .expect("in list");
    let sec = sse_time::secular::Secular::new(own.host_msun, &planets);
    let n = m::ceil(f.window_yr / f.sample_yr.max(1.0)).max(1.0) as usize;
    let samples = (0..=n)
        .map(|k| {
            let t = k as f64 * f.sample_yr;
            let p = sec.advance(&planets, t)[me];
            TrackSample {
                t_yr: t,
                e: p.e,
                varpi: p.varpi,
                incl: p.i,
                node: p.node,
            }
        })
        .collect();
    (
        Some(OrbitTrack {
            samples,
            source: format!(
                "T1 secular theory over {} planets (inclinations, nodes and perihelia drawn: formation does not publish them)",
                sibs.len()
            ),
        }),
        moons,
    )
}

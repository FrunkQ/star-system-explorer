//! The shipped scenario of the light crate: the black hole fixture bodies
//! of exotic objects design 3.4 (XO2 and XO3), read by the kernel's
//! `read_light_paths` as fixture `black-holes`.

use crate::feed::FeedSource;
use crate::paths::{PathsAsk, Viewer};
use crate::{C, G, K_B};
use sse_core::math as m;

const PC: f64 = sse_core::sheet::PC;

/// The three fixture bodies of design 3.4, each standing for a column of
/// its table (kernel fixture `black-holes`). Rates are stated in kg/s
/// with what they come to; masses in Suns.
pub fn fixtures() -> Vec<PathsAsk> {
    let msun = sse_core::sheet::M_SUN;
    let m_p = sse_core::sheet::M_P;
    vec![
        PathsAsk {
            address: "fixture/stellar-hole".into(),
            name: Some("Stellar black hole, 10 Suns, well fed".into()),
            mass_kg: 10.0 * msun,
            radius_m: 0.0,
            spin: 0.0,
            magnetic_moment_a_m2: 0.0,
            // 2.4e15 kg/s is 3.8e-8 Suns a year, about a tenth of its
            // Eddington rate; a Sun-like donor 4e9 m away (an orbit of about
            // half a day, its lobe about 1.2 Sun radii), like the low-mass
            // X-ray binaries.
            feed: Some(FeedSource::Companion {
                rate_kg_s: 2.4e15,
                separation_m: 4e9,
                donor_mass_kg: msun,
                donor: Some("a Sun-like companion overflowing its lobe".into()),
            }),
            // Luminet's view, 10° above the disc's plane.
            viewer: Viewer {
                distance_m: 2000.0 * PC,
                tilt_deg: 80.0,
            },
            about: Some("design 3.4, first column: an X-ray binary".into()),
        },
        PathsAsk {
            address: "fixture/quasar".into(),
            name: Some("Supermassive black hole, 100 million Suns, well fed".into()),
            mass_kg: 1e8 * msun,
            radius_m: 0.0,
            spin: 0.0,
            magnetic_moment_a_m2: 0.0,
            // 7.3e22 kg/s is about 1.2 Suns a year, a third of its
            // Eddington rate.
            feed: Some(FeedSource::Given {
                rate_kg_s: 7.3e22,
                outer_m: 1000.0 * G * 1e8 * msun / (C * C),
                why: "Fed from its galaxy at a stated rate; the disc is cut off at a thousand gravitational radii, about where a quasar's disc grows heavy enough to break up into stars (a stated edge until a self-gravity block lands; Goodman 2003, unverified).".into(),
            }),
            viewer: Viewer {
                distance_m: 1e9 * PC,
                tilt_deg: 30.0,
            },
            about: Some("design 3.4, second column: a quasar".into()),
        },
        PathsAsk {
            address: "fixture/starved-nucleus".into(),
            name: Some("Starved galactic centre, 4.15 million Suns".into()),
            mass_kg: 4.15e6 * msun,
            radius_m: 0.0,
            spin: 0.0,
            magnetic_moment_a_m2: 0.0,
            // Hot gas round the Milky Way's centre: 130 particles per cm³
            // (mean mass 0.6 proton masses) at 1.3e7 K, γ = 5/3 (Baganoff
            // et al. 2003, unverified).
            feed: Some(FeedSource::Surroundings {
                gas_density_kg_m3: 130e6 * 0.6 * m_p,
                sound_speed_m_s: m::sqrt(5.0 / 3.0 * K_B * 1.3e7 / (0.6 * m_p)),
                relative_speed_m_s: 0.0,
            }),
            viewer: Viewer {
                distance_m: 8200.0 * PC,
                tilt_deg: 30.0,
            },
            about: Some("design 3.4, third column: the Milky Way's centre".into()),
        },
    ]
}

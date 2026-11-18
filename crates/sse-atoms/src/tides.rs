//! Tides (P1 first wave).

use crate::common::*;
use sse_core::math as m;
use sse_core::quantity::consts::*;
use sse_core::Dimension as D;
use sse_flow::{Atom, CostClass};

const PCR: &str = "Peale, S. J., Cassen, P. and Reynolds, R. T. 1979, Science 203, 892: melting of Io by tidal dissipation. doi:10.1126/science.203.4383.892";
const SEGATZ: &str = "Segatz, M., Spohn, T., Ross, M. N. and Schubert, G. 1988, Icarus 75, 187: tidal dissipation in Io https://ui.adsabs.harvard.edu/abs/1988Icar...75..187S";
const GLADMAN: &str = "Gladman, B. et al. 1996, Icarus 122, 166: synchronous locking of tidally evolving satellites, eq. 9 https://ui.adsabs.harvard.edu/abs/1996Icar..122..166G";

pub fn atoms() -> Vec<Box<dyn Atom>> {
    vec![
        Def {
            id: "sse.tides.eccentricity_heating",
            version: "1.0.0",
            inputs: vec![
                port("host_mass", "orbit.host_mass", D::MASS, None),
                port("radius", "body.radius", D::LENGTH, None),
                port("semi_major_axis", "orbit.semi_major_axis", D::LENGTH, None),
                port("eccentricity", "orbit.eccentricity", D::NONE, Some((0.0, 0.1))),
            ],
            outputs: vec![port("power", "tides.heating_power", D::POWER, None)],
            params: vec![param(
                "k2_over_q",
                D::NONE,
                0.015,
                (1.0e-6, 0.1),
                "tidal response: Love number k2 over quality factor Q (Io 0.015, Lainey et al. 2009)",
            )],
            cost: CostClass::Trivial,
            references: &[PCR, SEGATZ, "Lainey, V. et al. 2009, Nature 459, 957 (Io's k2/Q). doi:10.1038/nature08108"],
            curious: "A moon on a slightly oval orbit is squeezed and relaxed every lap, and the flexing heats it. It makes Io the most volcanic place known.",
            science: "Heating of a synchronously rotating satellite by its eccentricity tide, for small e. Power scales as the fifth power of the satellite's radius and falls as distance to the power -7.5, so it is sensitive to everything.",
            formula: "Ė = (21/2) (k2/Q) G M² R⁵ n e² / a⁶, n = √(G M / a³)",
            f: |i, p| {
                let mh = positive(i[0], "host mass")?;
                let r = positive(i[1], "radius")?;
                let a = positive(i[2], "semi-major axis")?;
                let e = i[3];
                let n = m::sqrt(G * mh / (a * a * a));
                let r5 = r * r * r * r * r;
                let a6 = a * a * a * a * a * a;
                Ok(vec![10.5 * p[0] * G * mh * mh * r5 * n * e * e / a6])
            },
        }
        .build(),
        Def {
            id: "sse.tides.locking_time",
            version: "1.0.0",
            inputs: vec![
                port("initial_spin_rate", "body.initial_spin_rate", dims::RATE, None),
                port("semi_major_axis", "orbit.semi_major_axis", D::LENGTH, None),
                port("mass", "body.mass", D::MASS, None),
                port("radius", "body.radius", D::LENGTH, None),
                port("host_mass", "orbit.host_mass", D::MASS, None),
            ],
            outputs: vec![port("time", "tides.locking_time", D::TIME, None)],
            params: vec![
                param("q", D::NONE, 100.0, (10.0, 1.0e6), "tidal quality factor Q"),
                param("k2", D::NONE, 0.3, (1.0e-3, 1.5), "Love number k2"),
                param("moment_factor", D::NONE, 0.4, (0.05, 0.4), "moment of inertia factor I/(m R²); 0.4 for a uniform sphere"),
            ],
            cost: CostClass::Trivial,
            references: &[GLADMAN],
            curious: "Tides slowly brake a spinning world until one face always points at its partner, as the Moon does. Close in, it takes a few million years; far out, longer than the universe has existed.",
            science: "Despinning time for a body whose tidal response is a constant Q, from an initial spin to synchronous. Order-of-magnitude only: Q and k2 are uncertain by factors of several, and Q depends on frequency.",
            formula: "t = ω a⁶ I Q / (3 G M² k2 R⁵), I = α m R²",
            f: |i, p| {
                let w = positive(i[0], "initial spin rate")?;
                let a = positive(i[1], "semi-major axis")?;
                let mb = positive(i[2], "mass")?;
                let r = positive(i[3], "radius")?;
                let mh = positive(i[4], "host mass")?;
                let inertia = p[2] * mb * r * r;
                let a6 = a * a * a * a * a * a;
                let r5 = r * r * r * r * r;
                Ok(vec![w * a6 * inertia * p[0] / (3.0 * G * mh * mh * p[1] * r5)])
            },
        }
        .build(),
    ]
}

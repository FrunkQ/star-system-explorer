//! RK7: the published mission tables as fixtures. Hohmann transfers from
//! Earth to every planet (circular, coplanar orbits: the form the tables use),
//! read from SSE4's own Sol (`tests/fixtures/nav/sol.json`, J2000 semi-major
//! axes) and the Sun's GM, against the standard values the published
//! tables give (the method as Curtis, *Orbital Mechanics for Engineering
//! Students*, chapter 8, works it; the planet-by-planet table as Braeunig,
//! *Rocket and Space Technology*, and Atomic Rockets' mission table, list
//! it); synodic periods as the NASA planetary fact sheets give them. The
//! figures below were recomputed independently from the same method on
//! 23 Nov. Tolerances are under a percent: the tables round to three figures
//! and use slightly older elements.
use crate::courses::{hohmann_dv, hohmann_time};
use crate::scenario::sol;
use sse_core::math as m;
use sse_core::quantity::consts::{AU, DAY};

/// Planet, Hohmann time in days, departure and arrival speed change in km/s, synodic period in days.
const TABLE: [(&str, f64, f64, f64, f64); 7] = [
    ("Mercury", 105.5, 7.53, 9.61, 115.88),
    ("Venus", 146.0, 2.50, 2.71, 583.92),
    ("Mars", 258.9, 2.94, 2.65, 779.94),
    ("Jupiter", 997.0, 8.79, 5.64, 398.88),
    ("Saturn", 2208.0, 10.29, 5.44, 378.09),
    ("Uranus", 5857.0, 11.28, 4.66, 369.66),
    ("Neptune", 11199.0, 11.65, 4.05, 367.49),
];

fn close(got: f64, want: f64, share: f64, what: &str) {
    assert!(
        (got - want).abs() <= share * want.abs(),
        "{what}: SSE4 {got:.4}, the table {want} (allowed {:.2}%)",
        share * 100.0
    );
}

#[test]
fn sol_reproduces_the_published_hohmann_table_from_earth() {
    let sys = sol();
    let mu = sys.mu();
    let a = |n: &str| {
        sys.bodies
            .iter()
            .find(|b| b.name == n)
            .map(|b| b.a_au * AU)
            .expect(n)
    };
    let earth = a("Earth");
    // Orbital periods from Kepler's third law, so the synodic period is computed too.
    let period = |r: f64| 2.0 * m::PI * m::sqrt(r * r * r / mu);
    for (name, days, dep, arr, syn) in TABLE {
        let r = a(name);
        let t = hohmann_time(mu, earth, r) / DAY;
        let (d1, d2) = hohmann_dv(mu, earth, r);
        close(t, days, 0.008, &format!("{name} transfer days"));
        close(d1 / 1000.0, dep, 0.005, &format!("{name} departure km/s"));
        close(d2 / 1000.0, arr, 0.005, &format!("{name} arrival km/s"));
        let s = 1.0 / (1.0 / period(earth) - 1.0 / period(r)).abs() / DAY;
        close(s, syn, 0.003, &format!("{name} synodic days"));
    }
}

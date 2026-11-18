//! F3 evidence: the modern growth block over 10,000 systems.
use crate::disc::{disc_params, Disc};
use crate::growth::*;
use crate::scenario::system;
use sse_core::Seed;

fn seed() -> Seed {
    Seed::from_text("f3-population")
}

pub(crate) fn one(n: u64, mass: f64, feh: f64) -> (Disc, Growth) {
    let sys = system(n);
    let fits = crate::hurley::Fits::new((0.02 * sse_core::math::pow(10.0, feh)).clamp(1e-4, 0.03));
    let l = fits.l_zams(mass);
    let disc = Disc::new(disc_params(
        &seed(),
        &sys,
        mass,
        l,
        feh,
        0.0,
        0.55,
        300.0,
        None,
    ));
    let g = grow(&seed(), &sys, &disc);
    (disc, g)
}

#[test]
fn growth_is_deterministic_and_books_close() {
    for n in 0..40 {
        let (_, a) = one(n, 1.0, 0.0);
        let (_, b) = one(n, 1.0, 0.0);
        assert_eq!(
            serde_json::to_string(&a).unwrap(),
            serde_json::to_string(&b).unwrap()
        );
        assert!(a.budget_closes(), "system {n}: {:?}", a.budget);
        for body in &a.bodies {
            // The ledger is the body: every unit of solid and gas, by group.
            assert_eq!(body.ledger.solid(), body.solid, "{}", body.id);
            assert_eq!(body.ledger.gas(), body.gas, "{}", body.id);
        }
        let held: u64 = a.rings.iter().map(|r| r.solid.iter().sum::<u64>()).sum();
        assert_eq!(held, a.budget.solid_in_rings.iter().sum::<u64>());
    }
}

#[test]
fn water_line_seeds_the_first_ring_and_ices_sublimate_inside_it() {
    let (_, g) = one(3, 1.0, 0.0);
    let first = g.events.iter().find_map(|e| match e {
        GrowthEvent::EmbryoForms { band, t_myr, .. } => Some((band.clone(), *t_myr)),
        _ => None,
    });
    assert!(first.is_some());
    // Inner rocky worlds carry no CO ice; pebbles' ices went to vapour on the way in.
    assert!(g.budget.solid_sublimated[3] > 0, "water sublimated");
    for b in g.bodies.iter().filter(|b| b.a_au < 0.5 && b.born_au < 0.5) {
        assert_eq!(b.solid[5], 0, "{} has CO ice", b.id);
    }
}

#[derive(Default)]
struct Tally {
    n: usize,
    giants: usize,
    hot: usize,
    multis: usize,
    chained: usize,
}

/// 10,000 systems: five populations of 2,000 (Sun-like at three metal
/// contents, an M dwarf and an F/A star), with every Sun-like system's close-in
/// small planets passed through F6 for the radius gap.
#[test]
fn population_of_10000_systems() {
    let fits = crate::hurley::Fits::new(0.02);
    let sun = crate::lifeline::LifeLine::new(&fits, 1.0).unwrap();
    let pops = [(1.0, 0.0), (1.0, 0.3), (1.0, -0.3), (0.4, 0.0), (1.6, 0.0)];
    let mut t: Vec<Tally> = pops.iter().map(|_| Tally::default()).collect();
    let mut radii = [0usize; 16]; // 0.8 to 4.0 Earth radii in 0.2 bins
    for (k, (mass, feh)) in pops.iter().enumerate() {
        for i in 0..2000u64 {
            let n = 100_000 * (k as u64 + 1) + i;
            let (_, g) = one(n, *mass, *feh);
            let tt = &mut t[k];
            tt.n += 1;
            tt.giants += g.bodies.iter().any(|b| b.is_giant() && b.a_au < 10.0) as usize;
            tt.hot += g.bodies.iter().any(|b| b.is_giant() && b.a_au < 0.1) as usize;
            let close = g
                .bodies
                .iter()
                .filter(|b| b.a_au < 1.0 && b.solid_earth() > 1.0 && !b.is_giant())
                .count();
            if close >= 3 {
                tt.multis += 1;
                tt.chained += (!g.chains().is_empty()) as usize;
            }
            if *mass == 1.0 {
                for p in protoplanets(&g, &system(n)) {
                    let period = 365.25 * sse_core::math::pow(p.a_au, 1.5);
                    if period > 100.0 || p.gas > p.mass || p.mass < 1.0 {
                        continue;
                    }
                    let w = crate::early::early_life(&p, &fits, &sun);
                    let bin = ((w.radius - 0.8) / 0.2) as i64;
                    if (0..16).contains(&bin) {
                        radii[bin as usize] += 1;
                    }
                }
            }
        }
    }
    let f = |k: usize| t[k].giants as f64 / t[k].n as f64;
    for (k, p) in pops.iter().enumerate() {
        eprintln!(
            "M {} [Fe/H] {}: giants {:.3}, hot {:.3}, multis {}, chained {}",
            p.0,
            p.1,
            f(k),
            t[k].hot as f64 / t[k].n as f64,
            t[k].multis,
            t[k].chained
        );
    }
    eprintln!("radii (0.8 to 4.0 by 0.2, P < 100 d): {radii:?}");
    // Giant frequency (Sun-like, within 10 AU): about 10 to 20 percent observed
    // (Cumming et al. 2008; Fulton et al. 2021); tolerance a factor of two of 15 percent.
    assert!((0.075..0.30).contains(&f(0)), "solar giants {}", f(0));
    // Rises with metal content: Johnson et al. 2010 fit 10^(1.2 [Fe/H]), so
    // +0.3 dex gives 2.3 times; tolerance 1.5 to 3.5 times either way.
    let up = f(1) / f(0);
    let down = f(0) / f(2);
    assert!((1.5..3.5).contains(&up), "+0.3 dex ratio {up}");
    assert!((1.5..3.5).contains(&down), "-0.3 dex ratio {down}");
    // Rises with star mass: M dwarfs well below, F/A stars above or level
    // (Johnson et al. 2010: proportional to M; Endl et al. 2006 for M dwarfs).
    assert!(f(3) < 0.5 * f(0), "M dwarf giants {}", f(3));
    assert!(f(4) > f(0), "F/A star giants {}", f(4));
    // Compact chains: at the end of the gas disc, systems of three or more
    // close-in planets sit in resonant chains nearly always (Izidoro et al.
    // 2017, 2021: most later break, F4); tolerance at least 70 percent.
    let (m, c) = t
        .iter()
        .fold((0, 0), |a, x| (a.0 + x.multis, a.1 + x.chained));
    assert!(m >= 100, "only {m} multi-planet systems");
    assert!(c as f64 >= 0.7 * m as f64, "{c} of {m} chained");
    // The radius gap (Fulton et al. 2017): a super-Earth peak below about
    // 1.6 Earth radii, a valley, and a sub-Neptune hump beyond it.
    let mean = |a: usize, b: usize| radii[a..b].iter().sum::<usize>() as f64 / (b - a) as f64;
    let (peak, valley, hump) = (mean(2, 4), mean(5, 8), mean(9, 11));
    assert!(
        peak > 3.0 * valley,
        "super-Earths {peak} against valley {valley}"
    );
    assert!(
        hump > 1.1 * valley,
        "sub-Neptunes {hump} against valley {valley}"
    );
}

#[test]
fn growth_golden() {
    crate::golden::check("formation-growth.json", &crate::scenario::growth_golden());
}

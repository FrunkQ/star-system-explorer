//! F4 evidence: the violent phase over thousands of systems.
use crate::growth_tests::one;
use crate::scenario::system;
use crate::violent::*;
use sse_core::Seed;

fn run(n: u64, mass: f64, feh: f64) -> Upheaval {
    let (disc, g) = one(n, mass, feh);
    upheave(&Seed::from_text("f3-population"), &system(n), &disc, &g)
}

fn water_sum(w: &WaterFrom) -> u64 {
    w.iter().map(|x| x.1).sum()
}

fn check_books(u: &Upheaval, n: u64) {
    assert!(u.budget.closes(), "system {n}: mass books {:?}", u.budget);
    assert!(u.dynamics.closes(), "system {n}: dynamics {:?}", u.dynamics);
    let scale = u.dynamics.energy_start.abs();
    assert!(
        u.dynamics.energy_heat >= -1e-12 * scale,
        "system {n}: negative heat"
    );
    assert!(u.dynamics.energy_rogues >= 0.0);
    for w in &u.worlds {
        assert_eq!(w.ledger.solid(), w.solid, "system {n}: ledger of {}", w.id);
        assert_eq!(w.ledger.gas(), w.gas, "system {n}: gas ledger of {}", w.id);
        let mut by = vec![];
        w.ledger.water_by_band(&mut by);
        assert!(
            by.iter().all(|x| x.1 >= 0),
            "system {n}: negative water band in {}",
            w.id
        );
        assert_eq!(
            by.iter().map(|x| x.1).sum::<i128>(),
            w.solid[3] as i128,
            "system {n}: water provenance of {}",
            w.id
        );
    }
    for r in &u.rogues {
        assert_eq!(r.ledger.solid(), r.solid);
    }
    for b in &u.belts {
        assert_eq!(
            water_sum(&b.water_from),
            b.solid[3],
            "system {n}: belt {}",
            b.id
        );
    }
    for r in &u.reservoirs {
        assert_eq!(water_sum(&r.water_from), r.solid[3]);
    }
    for mo in &u.moons {
        assert_eq!(water_sum(&mo.water_from), mo.solid[3]);
    }
    for w in u.worlds.windows(2) {
        assert!(w[0].a_au <= w[1].a_au);
    }
}

#[test]
fn upheaval_is_deterministic_and_books_close() {
    for n in 0..300 {
        let a = run(n, 1.0, if n % 2 == 0 { 0.0 } else { 0.3 });
        if n < 30 {
            let b = run(n, 1.0, if n % 2 == 0 { 0.0 } else { 0.3 });
            assert_eq!(
                serde_json::to_string(&a).unwrap(),
                serde_json::to_string(&b).unwrap()
            );
        }
        check_books(&a, n);
    }
}

#[test]
fn rogue_planets_are_addressable() {
    let mut seen = std::collections::BTreeSet::new();
    let mut count = 0;
    for n in 0..400 {
        let u = run(n, 1.0, 0.3);
        for r in &u.rogues {
            assert!(r.address.to_string().starts_with(&system(n).to_string()));
            assert!(
                seen.insert(r.address.to_string()),
                "duplicate rogue address"
            );
            let again = run(n, 1.0, 0.3);
            assert_eq!(again.rogue(&r.address), Some(r));
            count += 1;
        }
    }
    assert!(count > 0, "no rogue planets in 400 metal-rich systems");
}

struct Pop {
    ratios: Vec<f64>,
    hill: Vec<f64>,
    multi_e: Vec<f64>,
    giant_e: Vec<f64>,
    near_res: usize,
    pairs: usize,
    chained_before: usize,
    chained_after: usize,
    multis: usize,
    rogue_giants: usize,
    rogues: usize,
    rocky: usize,
    rocky_moon: usize,
    systems: usize,
    broke: usize,
    outcomes: std::collections::BTreeMap<String, usize>,
}

fn survey(range: std::ops::Range<u64>, mass: f64, feh: f64) -> Pop {
    let mut p = Pop {
        ratios: vec![],
        hill: vec![],
        multi_e: vec![],
        giant_e: vec![],
        near_res: 0,
        pairs: 0,
        chained_before: 0,
        chained_after: 0,
        multis: 0,
        rogue_giants: 0,
        rogues: 0,
        rocky: 0,
        rocky_moon: 0,
        systems: 0,
        broke: 0,
        outcomes: Default::default(),
    };
    for n in range {
        let (disc, g) = one(n, mass, feh);
        let u = upheave(&Seed::from_text("f3-population"), &system(n), &disc, &g);
        p.systems += 1;
        for e in &u.events {
            if let UpheavalEvent::Impact { outcome, .. } = e {
                *p.outcomes.entry(outcome.clone()).or_default() += 1;
            }
        }
        if u.break_myr.is_some() {
            p.broke += 1;
        }
        // Kepler-like sample: P < 400 days, above half an Earth mass, not giants.
        let per = |a: f64| 365.25 * sse_core::math::pow(a, 1.5) / sse_core::math::sqrt(mass);
        let close: Vec<&World> = u
            .worlds
            .iter()
            .filter(|w| per(w.a_au) < 400.0 && w.mass_earth() > 0.5 && !w.is_giant())
            .collect();
        let close_before = g
            .bodies
            .iter()
            .filter(|b| per(b.a_au) < 400.0 && b.mass_earth() > 0.5 && !b.is_giant())
            .count();
        if close_before >= 3 && !g.chains().is_empty() {
            p.chained_before += 1;
        }
        if close.len() >= 3 {
            p.multis += 1;
            if close.iter().any(|w| w.resonance.is_some()) {
                p.chained_after += 1;
            }
            for w in &close {
                if per(w.a_au) < 100.0 {
                    p.multi_e.push(w.e);
                }
            }
        }
        if close.len() >= 2 {
            for w in close.windows(2) {
                let r = per(w[1].a_au) / per(w[0].a_au);
                p.ratios.push(r);
                let rh = sse_core::math::cbrt(
                    (w[0].mass_earth() + w[1].mass_earth()) / (3.0 * 332_946.0 * mass),
                ) * 0.5
                    * (w[0].a_au + w[1].a_au);
                p.hill.push((w[1].a_au - w[0].a_au) / rh);
                p.pairs += 1;
                if [2.0, 1.5, 4.0 / 3.0, 5.0 / 4.0]
                    .iter()
                    .any(|res| r / res >= 1.0 && r / res < 1.03)
                {
                    p.near_res += 1;
                }
            }
        }
        for w in &u.worlds {
            if w.is_giant() && w.mass_earth() > 50.0 && w.a_au > 0.1 && w.a_au < 5.0 {
                p.giant_e.push(w.e);
            }
            if !w.is_giant()
                && w.mass_earth() > 0.5
                && w.mass_earth() < 2.0
                && w.gas_earth() < 0.01 * w.mass_earth()
            {
                p.rocky += 1;
                if u.moons.iter().any(|mo| mo.parent == w.id) {
                    p.rocky_moon += 1;
                }
            }
        }
        for r in &u.rogues {
            p.rogues += 1;
            if r.mass_earth() > 50.0 {
                p.rogue_giants += 1;
            }
        }
    }
    p
}

fn median(v: &mut [f64]) -> f64 {
    v.sort_by(|a, b| a.total_cmp(b));
    if v.is_empty() {
        return f64::NAN;
    }
    v[v.len() / 2]
}

fn mean(v: &[f64]) -> f64 {
    v.iter().sum::<f64>() / v.len().max(1) as f64
}

#[test]
fn population_spacing_and_eccentricities() {
    let mut solar = survey(0..2000, 1.0, 0.0);
    let rich = survey(2000..3000, 1.0, 0.3);
    let mut giant_e: Vec<f64> = solar
        .giant_e
        .iter()
        .chain(rich.giant_e.iter())
        .copied()
        .collect();
    let med_ratio = median(&mut solar.ratios);
    let med_hill = median(&mut solar.hill);
    let tight = solar.hill.iter().filter(|x| **x < 8.0).count() as f64 / solar.hill.len() as f64;
    let close_ratio =
        solar.ratios.iter().filter(|x| **x < 1.3).count() as f64 / solar.ratios.len() as f64;
    let med_e = median(&mut solar.multi_e);
    let giant_mean = mean(&giant_e);
    let giant_med = median(&mut giant_e);
    let near_res = solar.near_res as f64 / solar.pairs as f64;
    let rogue_giants = (solar.rogue_giants + rich.rogue_giants) as f64 / 3000.0;
    let moon_rate = solar.rocky_moon as f64 / solar.rocky.max(1) as f64;
    eprintln!(
        "F4 population: {} systems, {} broke; pairs {}; median period ratio {med_ratio:.3}; \
         P2/P1 < 1.3: {close_ratio:.3}; median spacing {med_hill:.1} Hill; spacing < 8: {tight:.3}; \
         near resonance {near_res:.3}; multis {} (chained before {}, after {}); \
         median e (multis, P < 100 d) {med_e:.4}; giants {} mean e {giant_mean:.3} median {giant_med:.3}; \
         rogues {} + {} ({rogue_giants:.4} giant rogues per star); rocky {} with impact moon {moon_rate:.3}; impacts {:?}",
        solar.systems,
        solar.broke,
        solar.pairs,
        solar.multis,
        solar.chained_before,
        solar.chained_after,
        giant_e.len(),
        solar.rogues,
        rich.rogues,
        solar.rocky,
        solar.outcomes,
    );
    // Kepler period ratios: median about 1.9 to 2.2, few below 1.3 (Fabrycky et al. 2014,
    // ApJ 790, 146; Steffen and Hwang 2015, MNRAS 448, 1956).
    assert!(
        (1.6..2.6).contains(&med_ratio),
        "median period ratio {med_ratio}"
    );
    assert!(
        close_ratio < 0.05,
        "too many pairs inside 1.3: {close_ratio}"
    );
    // Spacing in mutual Hill radii: almost none below about 8 to 10, median of order
    // 20 (Pu and Wu 2015, ApJ 807, 44; Weiss et al. 2018, AJ 155, 48); this sample
    // counts planets Kepler would miss, so it may read wider, not tighter.
    assert!(
        (15.0..40.0).contains(&med_hill),
        "median spacing {med_hill}"
    );
    assert!(tight < 0.05, "pairs tighter than 8 Hill radii: {tight}");
    // Near first-order resonances: a modest excess, about 10 to 15 percent (Fabrycky 2014).
    assert!(near_res < 0.25, "near-resonant share {near_res}");
    // Chains: nearly all break (Izidoro et al. 2017, 2021).
    assert!(
        (solar.chained_after as f64) < 0.25 * solar.multis as f64,
        "chains surviving {} of {}",
        solar.chained_after,
        solar.multis
    );
    // Kepler multis are nearly circular: TTV rms about 0.02 (Hadden and Lithwick 2014,
    // ApJ 787, 80), transit durations Rayleigh 0.05 (Van Eylen et al. 2019, AJ 157, 61).
    assert!(
        (0.005..0.06).contains(&med_e),
        "median multi eccentricity {med_e}"
    );
    // RV giants: Beta(0.867, 3.03), mean 0.22 (Kipping 2013, MNRAS 434, L51).
    assert!(
        (0.12..0.35).contains(&giant_mean),
        "giant mean e {giant_mean}"
    );
    // Free-floating Jupiters: under 0.25 per star (Mroz et al. 2017, Nature 548, 183).
    assert!(rogue_giants < 0.25, "giant rogues per star {rogue_giants}");
    assert!(rich.rogues > 0);
    // Moon-forming impacts: a few to tens of percent of rocky planets (Elser et al. 2011).
    assert!(
        (0.02..0.25).contains(&moon_rate),
        "impact moon rate {moon_rate}"
    );
}

#[test]
fn golden() {
    crate::golden::check(
        "formation-upheaval.json",
        &crate::scenario::upheaval_golden(),
    );
}

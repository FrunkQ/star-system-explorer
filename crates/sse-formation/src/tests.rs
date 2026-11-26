#![allow(clippy::needless_range_loop)]
use crate::history::{FormationHistory, How, Manifest};
use crate::hurley::Fits;
use crate::inherit::InheritancePins;
use crate::lifeline::{white_dwarf_mass, LifeLine, Phase, L_HD};
use crate::scenario::{envelope, system};
use crate::stars::{kroupa_quantile, multiplicity_fraction, roche_lobe_fraction, StarPins};

fn close(got: f64, want: f64, rel: f64, what: &str) {
    assert!(
        (got - want).abs() <= rel * want.abs(),
        "{what}: got {got}, want {want} within {rel}"
    );
}

/// The Sun as a pinned system: one 1.0 solar-mass star, [Fe/H] 0, 4.57 Gyr.
fn sun_manifest() -> Manifest {
    let mut m = Manifest::new("sol", system(9999), envelope());
    m.inherit_pins = InheritancePins {
        feh: Some(0.0),
        age_gyr: Some(4.57),
        ..Default::default()
    };
    m.star_pins = StarPins {
        primary_mass: Some(1.0),
        count: Some(1),
    };
    m
}

#[test]
fn zero_age_fits_match_tout_1996() {
    // Tout et al. 1996 quote their Z = 0.02, 1 solar-mass fit as L = 0.698, R = 0.888
    // (their models: 0.70, 0.89). Checks the port of the coefficient tables.
    let f = Fits::new(0.02);
    close(f.l_zams(1.0), 0.698, 0.002, "L_ZAMS(1)");
    close(f.r_zams(1.0), 0.888, 0.002, "R_ZAMS(1)");
    // Lower metal content: brighter and hotter at the same mass (Tout et al. 1996
    // figure 1: a metal-poor solar-mass ZAMS star near 6700 K).
    let poor = Fits::new(0.001);
    let t = |f: &Fits| crate::lifeline::teff(f.l_zams(1.0), f.r_zams(1.0));
    assert!(poor.l_zams(1.0) > 2.0 * f.l_zams(1.0));
    assert!(
        (6400.0..7000.0).contains(&t(&poor)) && t(&poor) > t(&f) + 800.0,
        "{} {}",
        t(&poor),
        t(&f)
    );
}

#[test]
fn main_sequence_matches_hurley_and_the_sun() {
    let f = Fits::new(0.02);
    // HPT00 section 5.1: t_MS of a solar-mass Z = 0.02 star about 11 Gyr (their 1.1e4 Myr).
    close(f.t_ms(1.0), 11_000.0, 0.03, "t_MS(1)");
    // Massive stars live millions of years; HPT00 figure 6 (about 25 Myr at 10, 8.7 at 20).
    close(f.t_ms(10.0), 24.3, 0.05, "t_MS(10)");
    close(f.t_ms(20.0), 8.7, 0.05, "t_MS(20)");
    // Metal-poor solar-mass star burns faster (HPT00: about 6 Gyr at Z = 0.001).
    let poor = Fits::new(0.001).t_ms(1.0);
    assert!(
        (5_000.0..7_500.0).contains(&poor),
        "t_MS(1, Z=0.001) = {poor}"
    );

    // The Sun today from its pinned history: within the fits' stated accuracy
    // (HPT00 quote about 5 percent against the models, and the models' Sun is
    // not tuned to the real one).
    let h = FormationHistory::form(sun_manifest()).unwrap();
    let now = h.state_at(4570.0);
    assert_eq!(now[0].phase, Phase::MainSequence);
    close(now[0].luminosity, 1.0, 0.06, "Sun L today");
    close(now[0].radius, 1.0, 0.03, "Sun R today");
    close(now[0].teff, 5772.0, 0.03, "Sun Teff today");
    // Its end: a white dwarf of about 0.57 solar masses (Cummings et al. 2018).
    let l = &h.stars[0].life;
    assert_eq!(l.remnant, Phase::WhiteDwarf);
    close(l.remnant_mass, 0.569, 0.01, "Sun's white dwarf");
    assert!(
        l.t_end > 11_000.0 && l.t_end < 13_500.0,
        "Sun dies at {}",
        l.t_end
    );
}

#[test]
fn remnants_follow_the_initial_final_mass_relation() {
    // Sirius B: about 1.02 solar masses from a progenitor near 5 (Cummings 2018 figure 7).
    close(white_dwarf_mass(5.0), 1.006, 0.001, "IFMR(5)");
    let f = Fits::new(0.02);
    assert_eq!(LifeLine::new(&f, 12.0).unwrap().remnant, Phase::NeutronStar);
    assert_eq!(LifeLine::new(&f, 40.0).unwrap().remnant, Phase::BlackHole);
    let bd = LifeLine::new(&f, 0.05).unwrap();
    assert_eq!(bd.state_at(&f, 1000.0).phase, Phase::BrownDwarf);
    // Burrows et al. 2001: a 0.05 solar-mass brown dwarf at 1 Gyr is near 4e-5 L_sun
    // and about 1500 K, and it keeps cooling.
    let s = bd.state_at(&f, 1000.0);
    close(s.luminosity, 4.0e-5, 1e-9, "brown dwarf L at 1 Gyr");
    assert!(
        (1200.0..1900.0).contains(&s.teff),
        "brown dwarf Teff {}",
        s.teff
    );
    assert!(bd.state_at(&f, 5000.0).luminosity < s.luminosity);
    assert!(LifeLine::new(&f, 0.005)
        .unwrap_err()
        .0
        .contains("planetary-mass"));
    // A white dwarf fades: ten times dimmer over a few Gyr of cooling.
    let sun = LifeLine::new(&f, 1.0).unwrap();
    let (a, b) = (
        sun.state_at(&f, sun.t_end + 100.0),
        sun.state_at(&f, sun.t_end + 5000.0),
    );
    assert!(
        a.luminosity > 10.0 * b.luminosity,
        "{} {}",
        a.luminosity,
        b.luminosity
    );
    close(a.radius, 0.0125, 0.15, "white dwarf radius");
}

/// Every star's whole life stays inside what real stars do (26 Nov 2026 sweep):
/// before, massive giants brightened ten times (a 25 M_sun star reached 7,200
/// R_sun at 2,600 K, a 100 M_sun star 31,600 R_sun) and red dwarfs had a 2,500
/// L_sun giant tip they can never grow a core for.
#[test]
fn every_life_line_stays_inside_real_stars() {
    for z in [0.02, 0.001] {
        let f = Fits::new(z);
        for i in 0..=60 {
            let mass = 0.1 * sse_core::math::pow(1000.0, f64::from(i) / 60.0);
            let l = LifeLine::new(&f, mass).unwrap();
            assert!(l.t_tms <= l.t_bgb && l.t_bgb <= l.t_strip && l.t_strip <= l.t_end);
            for k in 0..=400 {
                let t = l.t_tms + (l.t_end - l.t_tms) * f64::from(k) / 400.0 * 0.999_999;
                if l.t_end <= l.t_tms {
                    break;
                }
                let s = l.state_at(&f, t);
                let at = format!("{mass:.2} M_sun, Z {z}, {t:.4e} Myr ({:?})", s.phase);
                // No measured star is wider than about 2,000 R_sun; the radius
                // law runs about a quarter large on the red side (F-S33).
                assert!(s.radius < 3000.0, "{at}: {} R_sun", s.radius);
                // Cool giants sit on the Hayashi line, never below about 2,900 K.
                assert!(s.teff > 2900.0, "{at}: {} K", s.teff);
                // Nothing cool is brighter than the Humphreys-Davidson limit.
                if s.teff < 8000.0 {
                    assert!(s.luminosity <= 1.001 * L_HD, "{at}: {} L_sun", s.luminosity);
                }
                if s.phase == Phase::WolfRayet {
                    assert!(s.teff > 30_000.0 && s.radius < 30.0, "{at}");
                }
            }
        }
    }
    let f = Fits::new(0.02);
    // Red dwarfs below 0.25 M_sun never become giants: straight to a helium
    // white dwarf of their whole mass.
    let rd = LifeLine::new(&f, 0.15).unwrap();
    assert_eq!(rd.t_end, rd.t_tms);
    assert_eq!(rd.phase_at(rd.t_tms * 1.01), Phase::WhiteDwarf);
    close(rd.remnant_mass, 0.15, 1e-12, "red dwarf remnant");
    // A 0.3 M_sun star cannot grow the 0.47 M_sun flash core: its tip is set
    // by its whole mass, far below 2,500 L_sun.
    assert!(LifeLine::new(&f, 0.3).unwrap().l_tip < 500.0);
    // A 25 M_sun star at 7.5 Myr is a red supergiant of about 2,000 R_sun,
    // not the 6,090 R_sun at 2,650 K reported before the fix.
    let s = LifeLine::new(&f, 25.0).unwrap().state_at(&f, 7.5);
    assert_eq!(s.phase, Phase::Giant);
    assert!((1500.0..2300.0).contains(&s.radius), "{}", s.radius);
    // A 60 M_sun star never becomes a red supergiant: stripped to a Wolf-Rayet star.
    let big = LifeLine::new(&f, 60.0).unwrap();
    assert!(big.t_strip < big.t_end);
    assert_eq!(
        big.phase_at(0.5 * (big.t_strip + big.t_end)),
        Phase::WolfRayet
    );
}

#[test]
fn same_manifest_gives_the_same_history_in_any_order() {
    let env = envelope();
    let one = |n| {
        FormationHistory::form(Manifest::new("order", system(n), env.clone()))
            .ok()
            .map(|h| h.hash())
    };
    let forward: Vec<_> = (0..12).map(one).collect();
    let backward: Vec<_> = (0..12)
        .rev()
        .map(one)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect();
    assert_eq!(forward, backward);
    // A different seed is a different system.
    let other = FormationHistory::form(Manifest::new("other", system(0), env.clone()))
        .ok()
        .map(|h| h.hash());
    assert_ne!(forward[0], other);
}

#[test]
fn a_slice_reproduces_from_its_manifest() {
    let h = FormationHistory::form(Manifest::new("slice", system(3), envelope())).unwrap();
    let text = serde_json::to_string(&h.manifest).unwrap();
    let again = FormationHistory::form(serde_json::from_str(&text).unwrap()).unwrap();
    let k = h.slices.len() / 2;
    assert_eq!(h.slices[k], again.slices[k]);
    assert_eq!(h.hash(), again.hash());
}

#[test]
fn the_ruler_reads_slices_and_never_recomputes() {
    let h = FormationHistory::form(sun_manifest()).unwrap();
    let f = h.fits();
    // On a slice: exactly the stored state.
    let s = &h.slices[20];
    assert_eq!(h.state_at(s.t_myr), s.stars);
    // Between slices on the main sequence: within 1 percent of the life line.
    for t in [1234.5, 4570.0, 8001.0] {
        let read = &h.state_at(t)[0];
        let direct = h.stars[0].life.state_at(&f, t);
        close(read.luminosity, direct.luminosity, 0.01, "interpolated L");
        close(read.radius, direct.radius, 0.01, "interpolated R");
    }
    // Slices are dense in the first 300 Myr and include every phase boundary.
    assert!(h.slices.iter().filter(|s| s.t_myr <= 300.0).count() >= 35);
    for b in h.stars[0].life.boundaries() {
        assert!(
            h.slices.iter().any(|s| s.t_myr == b),
            "boundary {b} is a slice"
        );
    }
}

#[test]
fn budgets_and_records_close() {
    for n in 0..40 {
        let Ok(h) = FormationHistory::form(Manifest::new("budget", system(n), envelope())) else {
            continue;
        };
        let sum: f64 = h.stars.iter().map(|s| s.mass).sum();
        assert_eq!(h.budgets.stellar_mass, sum);
        // Every star has its five life events, and each slice has every star.
        let born = h.events.iter().filter(|e| e.kind == "star.born").count();
        assert_eq!(born, h.stars.len());
        assert!(h.slices.iter().all(|s| s.stars.len() == h.stars.len()));
        // Events are in date order, and every value has its evidence.
        assert!(h.events.windows(2).all(|w| w[0].t_myr <= w[1].t_myr));
        assert!(
            h.evidence.contains_key("inheritance.feh") && h.evidence.contains_key("stars.0.mass")
        );
        // Chronicle events accept the history and keep its order.
        let ce = h.chronicle_events();
        let limit = crate::history::CHRONICLE_LIMIT_YEARS;
        let inside = h
            .events
            .iter()
            .filter(|e| h.inheritance.born_gyr * 1e9 + e.t_myr * 1e6 < limit)
            .count();
        assert_eq!(ce.len(), inside);
        assert!(ce
            .windows(2)
            .all(|w| w[0].body.occurred_at <= w[1].body.occurred_at));
    }
}

#[test]
fn populations_match_their_sources() {
    // Kroupa 2001 on 0.08 to 100: 76.1 percent of stars are below 0.5 solar masses.
    let n = 4000;
    let below = (0..n)
        .filter(|i| kroupa_quantile((*i as f64 + 0.5) / n as f64) < 0.5)
        .count();
    close(
        below as f64 / n as f64,
        0.7607,
        0.01,
        "IMF fraction below 0.5",
    );
    assert!(
        (kroupa_quantile(0.0) - 0.08).abs() < 1e-12
            && (kroupa_quantile(1.0 - 1e-15) - 100.0).abs() < 1e-6
    );
    // Multiplicity of Sun-like primaries over many drawn systems (Duchene and Kraus 2013: 44 percent).
    let env = envelope();
    let (mut sunlike, mut multiple) = (0, 0);
    for i in 0..6000 {
        let mut m = Manifest::new("pop", system(i), env.clone());
        m.inherit_pins.feh = Some(0.0);
        let Ok(h) = FormationHistory::form(m) else {
            continue;
        };
        if (0.7..1.3).contains(&h.stars[0].mass) {
            sunlike += 1;
            if h.stars.len() > 1 {
                multiple += 1;
            }
        }
    }
    assert!(sunlike > 250, "{sunlike} Sun-like primaries");
    let frac = multiple as f64 / sunlike as f64;
    assert!(
        (frac - multiplicity_fraction(1.0)).abs() < 0.07,
        "multiple fraction {frac} of {sunlike}"
    );
}

#[test]
fn close_pairs_get_dated_mass_transfer() {
    // Eggleton 1983: equal masses give a lobe of 0.379 of the separation.
    close(roche_lobe_fraction(1.0), 0.3789, 0.001, "Roche lobe q=1");
    let env = envelope();
    let mut found = 0;
    for i in 0..400 {
        let mut m = Manifest::new("pairs", system(i), env.clone());
        m.star_pins.count = Some(2);
        let Ok(h) = FormationHistory::form(m) else {
            continue;
        };
        let o = h.stars[1].orbit.as_ref().unwrap();
        let mt = h.events.iter().find(|e| e.kind == "binary.mass_transfer");
        // Wide pairs never touch; pairs inside a few AU often do once a star swells.
        if o.semi_major_axis_au > 50.0 {
            assert!(
                mt.is_none(),
                "wide pair at {} AU transfers mass",
                o.semi_major_axis_au
            );
        }
        if let Some(e) = mt {
            found += 1;
            let donor = h.stars.iter().find(|s| s.address == e.subject).unwrap();
            assert!(e.t_myr <= donor.life.t_end);
        }
    }
    assert!(found > 20, "only {found} mass-transfer events in 400 pairs");
}

#[test]
fn metals_outside_the_fits_use_the_nearest_edge_and_say_so() {
    let mut m = sun_manifest();
    m.inherit_pins.feh = Some(-3.0);
    let h = FormationHistory::form(m).unwrap();
    let How::Calculated { source } = &h.evidence["stars.metallicity_for_fits"] else {
        panic!()
    };
    assert!(source.contains("nearest edge"), "{source}");
    assert_eq!(h.fits().z, 1.0e-4);
}

#[test]
fn golden_digest() {
    crate::golden::check("formation.json", &crate::scenario::golden());
}

#[test]
fn forms_every_system_of_the_g0_solar_patch() {
    // G0's acceptance check: system generation runs from its envelope fixture.
    let all = crate::scenario::solar_patch();
    assert!(all.len() > 250, "{} systems", all.len());
    let (mut formed, mut wd, mut refusals) = (
        0usize,
        0usize,
        std::collections::BTreeMap::<String, usize>::new(),
    );
    for h in &all {
        match h {
            Ok(h) => {
                formed += 1;
                // The cloud core's mass is shared exactly among its stars.
                let Some(total) = h.inheritance.system_mass else {
                    panic!("birth mass kept")
                };
                close(
                    h.budgets.stellar_mass,
                    total,
                    1e-12,
                    "stars share the core mass",
                );
                let now = h.state_at(h.inheritance.age_gyr * 1000.0);
                if now.iter().any(|s| s.phase == Phase::WhiteDwarf) {
                    wd += 1;
                }
            }
            Err(e) => {
                let key = if e.0.contains("planetary-mass") {
                    "planetary mass"
                } else {
                    "other"
                };
                *refusals.entry(key.to_string()).or_default() += 1;
            }
        }
    }
    // G0's cores are all stars or brown dwarfs: every system forms.
    assert!(refusals.is_empty(), "{refusals:?}");
    assert_eq!(formed, all.len());
    // Remnants: about 6 to 8 percent of nearby systems hold a white dwarf
    // (Hollands et al. 2018, 20 pc sample); the sketch giant phases allow a wide band.
    let f = wd as f64 / formed as f64;
    assert!(
        (0.02..0.2).contains(&f),
        "white dwarf fraction {f} ({wd} of {formed})"
    );
    // Same patch twice: identical histories.
    let again = crate::scenario::solar_patch();
    assert!(all.iter().zip(&again).all(|(a, b)| match (a, b) {
        (Ok(a), Ok(b)) => a.hash() == b.hash(),
        (Err(a), Err(b)) => a == b,
        _ => false,
    }));
}

fn sun_disc() -> crate::disc::Disc {
    use crate::disc::*;
    Disc::new(DiscParams {
        mass0: 0.03,
        rc0: 30.0,
        alpha: 0.01,
        host_mass: 1.0,
        host_luminosity: 0.7,
        kappa: 3.0,
        mix: mix(0.0, 0.0, 0.55),
        mdot_pe: 2.0e-9,
        inner_edge: 0.0,
        truncation: None,
        circumbinary: false,
    })
}

#[test]
fn ice_lines_sweep_inward_as_cited_models_say() {
    use crate::disc::*;
    let d = sun_disc();
    let water = |t: f64| d.slice(t).lines[2];
    // Three ages of a young-Sun disc. Early, accretion heating pushes the water
    // line out to about 5 to 7 AU; near 1e-8 solar masses a year it sits at 2 to 3
    // AU (Garaud and Lin 2007, ApJ 654, 606; Oka, Nakamoto and Ida 2011, ApJ 738, 141);
    // once accretion fades it approaches the passive flared-disc value (CG97).
    let (w0, w1, w2) = (water(0.0), water(0.5), water(d.t_disperse * 0.999));
    assert!((4.0..8.0).contains(&w0), "water line at birth {w0}");
    let s = d.slice(0.5);
    assert!(
        (0.5e-8..2.0e-8).contains(&s.mdot),
        "accretion at 0.5 Myr {}",
        s.mdot
    );
    assert!((1.5..4.0).contains(&w1), "water line near 1e-8 {w1}");
    let passive = crate::disc::irradiation_temperature(&d.params, 1.0);
    let r_passive = m_pow(150.0 * m_pow(0.7, 2.0 / 7.0) / 135.0, 7.0 / 3.0);
    close(
        passive,
        150.0 * m_pow(0.7, 2.0 / 7.0),
        1e-12,
        "CG97 at 1 AU",
    );
    assert!(
        w2 > r_passive && w2 < 1.6 * r_passive,
        "late water line {w2}, passive {r_passive}"
    );
    // Every line moves inward, and they stay in temperature order.
    let (a, b) = (d.slice(0.0), d.slice(1.0));
    for k in 0..5 {
        assert!(b.lines[k] <= a.lines[k], "line {} moved out", LINES[k].0);
        if k > 0 {
            assert!(a.lines[k] > a.lines[k - 1]);
        }
    }
    // Hartmann et al. 1998: about 1e-8 a year at 1 Myr (within half a dex).
    let md = d.slice(1.0).mdot;
    assert!((3.0e-9..3.0e-8).contains(&md), "accretion at 1 Myr {md}");
}

fn m_pow(x: f64, y: f64) -> f64 {
    sse_core::math::pow(x, y)
}

#[test]
fn disc_budgets_close_exactly() {
    use crate::disc::*;
    let d = sun_disc();
    let l = &d.ledger;
    assert_eq!(d.bands.iter().map(|b| b.gas).sum::<u64>(), l.gas_initial);
    for g in 0..6 {
        let sum: u64 = d.bands.iter().map(|b| b.solid[g] + b.vapour[g]).sum();
        assert_eq!(sum, l.groups_initial[g], "group {:?}", GROUPS[g]);
    }
    assert_eq!(l.gas_accreted + l.gas_dispersed, l.gas_initial);
    // Ices are solid only beyond their own lines: water ice never inside the water-ice band.
    let water_band = d.bands.iter().position(|b| b.key == "water-ice").unwrap();
    for (k, b) in d.bands.iter().enumerate() {
        assert_eq!(b.solid[3] > 0, k >= water_band, "water solid in {}", b.key);
        assert!(
            b.vapour[0] == 0 || k == 0,
            "metal vapour outside the hot band in {}",
            b.key
        );
    }
    // And over many drawn systems, every disc closes.
    for n in 0..60 {
        let h = FormationHistory::form(Manifest::new("discs", system(n), envelope())).unwrap();
        let l = &h.disc.ledger;
        assert_eq!(
            h.disc.bands.iter().map(|b| b.gas).sum::<u64>(),
            l.gas_initial
        );
        assert_eq!(l.gas_accreted + l.gas_dispersed, l.gas_initial);
    }
}

#[test]
fn discs_live_a_few_million_years_shorter_under_glare() {
    // Over many Sun-like systems: half the discs gone by about 3 Myr, few last
    // past 6 (Haisch et al. 2001; Mamajek 2009).
    let env = envelope();
    let mut life = vec![];
    for i in 0..400 {
        let mut m = Manifest::new("lifetimes", system(i), env.clone());
        m.star_pins = StarPins {
            primary_mass: Some(1.0),
            count: Some(1),
        };
        m.inherit_pins.fuv_g0 = Some(300.0);
        life.push(FormationHistory::form(m).unwrap().disc.t_disperse);
    }
    life.sort_by(f64::total_cmp);
    let median = life[life.len() / 2];
    assert!(
        (1.5..5.0).contains(&median),
        "median disc lifetime {median} Myr"
    );
    let late = life.iter().filter(|t| **t > 6.0).count() as f64 / life.len() as f64;
    assert!(late < 0.25, "{late} of discs last past 6 Myr");
    // A harsh cluster (1e5 G0) shortens the same disc.
    let mut m = Manifest::new("lifetimes", system(1), env.clone());
    m.star_pins = StarPins {
        primary_mass: Some(1.0),
        count: Some(1),
    };
    m.inherit_pins.fuv_g0 = Some(10.0);
    let calm = FormationHistory::form(m.clone()).unwrap().disc.t_disperse;
    m.inherit_pins.fuv_g0 = Some(1.0e5);
    let harsh = FormationHistory::form(m).unwrap().disc.t_disperse;
    assert!(harsh < 0.5 * calm, "harsh {harsh} calm {calm}");
}

#[test]
fn metals_companions_and_records() {
    let env = envelope();
    let solids = |feh: f64| {
        let mut m = Manifest::new("metals", system(7), env.clone());
        m.star_pins = StarPins {
            primary_mass: Some(1.0),
            count: Some(1),
        };
        m.inherit_pins.feh = Some(feh);
        let h = FormationHistory::form(m).unwrap();
        h.disc.ledger.groups_initial[0] + h.disc.ledger.groups_initial[1]
    };
    // Dust scales with metal content (galaxy design 8.4): +0.3 dex is about twice the rock and metal.
    let r = solids(0.3) as f64 / solids(0.0) as f64;
    close(r, 1.995, 0.01, "rock and metal at +0.3 dex");
    // Companions: wide pairs truncate the disc, close pairs share one with a cavity.
    let mut seen_trunc = false;
    let mut seen_cb = false;
    for i in 0..300 {
        let mut m = Manifest::new("pairs", system(i), env.clone());
        m.star_pins.count = Some(2);
        let h = FormationHistory::form(m).unwrap();
        let o = h.stars[1].orbit.as_ref().unwrap();
        let p = &h.disc.params;
        if p.circumbinary {
            seen_cb = true;
            close(p.inner_edge, 2.5 * o.semi_major_axis_au, 1e-12, "cavity");
            assert_eq!(h.disc.bands[0].inner_au, p.inner_edge);
        } else if let Some(tr) = p.truncation {
            seen_trunc = true;
            assert!(p.rc0 <= tr / 3.0 + 1e-12);
        }
    }
    assert!(seen_trunc && seen_cb);
    // The history records the disc: forms and clears events, disc slices until it clears.
    let h = FormationHistory::form(sun_manifest()).unwrap();
    assert!(h.events.iter().any(|e| e.kind == "disc.forms"));
    let clears = h.events.iter().find(|e| e.kind == "disc.clears").unwrap();
    assert_eq!(clears.t_myr, h.disc.t_disperse);
    assert!(h
        .slices
        .iter()
        .all(|s| s.disc.is_some() == (s.t_myr < h.disc.t_disperse)));
    assert!(h.slices.iter().filter(|s| s.disc.is_some()).count() >= 6);
}

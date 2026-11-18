use crate::history::{FormationHistory, Manifest};
use crate::hurley::Fits;
use crate::inherit::InheritancePins;
use crate::lifeline::{white_dwarf_mass, LifeLine, Phase};
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
    assert!(LifeLine::new(&f, 0.05)
        .unwrap_err()
        .0
        .contains("brown dwarf"));
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
    // Kroupa 2001 on 0.1 to 100: 72.9 percent of stars are below 0.5 solar masses.
    let n = 4000;
    let below = (0..n)
        .filter(|i| kroupa_quantile((*i as f64 + 0.5) / n as f64) < 0.5)
        .count();
    close(
        below as f64 / n as f64,
        0.729,
        0.01,
        "IMF fraction below 0.5",
    );
    assert!(
        (kroupa_quantile(0.0) - 0.1).abs() < 1e-12
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
fn out_of_range_is_refused_with_a_reason() {
    let mut m = sun_manifest();
    m.inherit_pins.feh = Some(-3.0);
    let e = FormationHistory::form(m).unwrap_err();
    assert!(e.0.contains("outside the stellar fits"), "{}", e.0);
}

#[test]
fn golden_digest() {
    crate::golden::check("formation.json", &crate::scenario::golden());
}

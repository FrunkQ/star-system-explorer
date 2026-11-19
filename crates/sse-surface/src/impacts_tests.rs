use crate::impacts::*;
use crate::scenario::{fixtures, settings};
use crate::wear::WaterHistory;
use crate::*;
use sse_core::draw::Seed;

fn body(name: &str) -> Body {
    fixtures()
        .into_iter()
        .find(|f| f.body.label == name)
        .unwrap()
        .body
}

fn run_with(
    s: &SurfaceSettings,
    name: &str,
    hist: &ImpactHistory,
    water: Option<WaterHistory>,
) -> Result<(Features, CraterField), String> {
    let b = body(name);
    let e = envelope(s, &b);
    let w = water.unwrap_or_else(|| WaterHistory::steady(b.oceans, b.pressure_bar, b.age_gyr));
    let seed = Seed::from_text("sse4-impacts-tests");
    let plates = crate::plates::history(s, &e, &seed, b.age_gyr);
    let inp = ImpactInputs {
        history: hist,
        water: &w,
        pressure_bar: b.pressure_bar,
        plates: Some(&plates),
    };
    impacts(s, &e, &seed, &inp, b.age_gyr)
}

fn run_for(name: &str) -> (Features, CraterField) {
    run_with(&settings(), name, &ImpactHistory::lunar(), None).unwrap()
}

#[test]
fn what_survives_follows_how_the_surface_is_renewed() {
    let survey: Vec<(String, usize, f64)> = [
        "Moon", "Mercury", "Mars", "Vesta", "Earth", "Venus", "Ganymede", "Europa", "Io", "Titan",
    ]
    .iter()
    .map(|n| {
        let (f, _) = run_for(n);
        eprintln!(
            "{n}: listed >= {:.0} km, expected {:.0}, kept {}, erased {:?}, plains {}, shields {}",
            f.listed_d_km,
            f.expected_listed,
            f.craters.len(),
            f.erased,
            f.plains.len(),
            f.shields.len()
        );
        (n.to_string(), f.craters.len(), f.expected_listed)
    })
    .collect();
    let kept = |n: &str| survey.iter().find(|x| x.0 == n).unwrap().1;
    // Airless lid worlds keep a saturated population, within Poisson error.
    for (n, k, lam) in &survey[..4] {
        assert!(
            (*k as f64 - lam).abs() < 4.0 * lam.sqrt() + 1.0,
            "{n}: kept {k}, expected {lam}"
        );
    }
    // Renewed surfaces keep few or none, in the order of their renewal.
    assert!(kept("Earth") < 20 && kept("Venus") < 20 && kept("Titan") < 20);
    assert_eq!(kept("Europa"), 0);
    assert_eq!(kept("Io"), 0);
    assert!(kept("Ganymede") > kept("Europa") && kept("Ganymede") < kept("Moon") / 4);
    // No surface, no craters.
    let (j, _) = run_for("Jupiter");
    assert!(j.surface_from_myr.is_none() && j.craters.is_empty());
}

#[test]
fn crater_density_matches_the_chronicle_on_a_renewed_surface() {
    // Venus with craters listed down to about 24 km: unsaturated, so the count
    // must match production times the regime's survival, and the crater-count
    // model age is the surface's retention age.
    let mut s = settings();
    s.impacts.listed_d_frac = 0.004;
    s.impacts.listed_max = 200_000;
    s.impacts.volcanism.bury_d_km = 0.0;
    let (f, field) = run_with(&s, "Venus", &ImpactHistory::lunar(), None).unwrap();
    let want = field.expected_after_renewal(f.listed_d_km);
    let got = f.craters.len() as f64;
    eprintln!(
        "Venus: {got} craters >= {:.1} km, chronicle expects {want:.1}; drew {:.0}, erased {:?}",
        f.listed_d_km, f.expected_listed, f.erased
    );
    assert!(want > 50.0, "{want}");
    assert!(
        (got - want).abs() < 4.0 * want.sqrt(),
        "got {got}, want {want}"
    );
    let area = 4.0 * sse_core::math::PI * field.radius_km() * field.radius_km();
    let age = Features::model_age_myr(&field, f.listed_d_km, f.craters.len(), area);
    eprintln!("Venus crater-count model age {age:.0} Myr");
    assert!(age > 150.0 && age < 1500.0, "{age}");
    // Craters are spread over the surface, not clumped: each hemisphere has
    // its share.
    let north = f.craters.iter().filter(|c| c.dir[2] > 0.0).count() as f64;
    assert!(
        (north - got / 2.0).abs() < 4.0 * (got / 4.0).sqrt(),
        "{north} of {got}"
    );
}

#[test]
fn shapes_follow_size_and_gravity() {
    let s = settings().impacts;
    // Simple-to-complex transition: about 15 km on the Moon, 6.5 on Mars, 2.5 on Earth.
    let first_complex = |g: f64| {
        let mut d = 0.5;
        while crater_shape(&s, g, d).0 == "bowl" {
            d *= 1.01;
        }
        d
    };
    let (moon, mars, earth) = (
        first_complex(1.62),
        first_complex(3.71),
        first_complex(9.81),
    );
    assert!(moon > 14.0 && moon < 16.0, "{moon}");
    assert!(mars > 6.0 && mars < 7.0, "{mars}");
    assert!(earth > 2.3 && earth < 2.7, "{earth}");
    // Bowls are about a fifth as deep as they are wide; big craters are shallow.
    let (_, d1, _, _) = crater_shape(&s, 1.62, 5.0);
    assert!((d1 / 5000.0 - 0.196).abs() < 1e-9);
    let (shape, d2, _, peak) = crater_shape(&s, 1.62, 100.0);
    assert_eq!(shape, "central-peak");
    assert!(d2 / 100_000.0 < 0.05 && peak > 0.0);
    assert_eq!(crater_shape(&s, 1.62, 300.0).0, "peak-ring");
    assert_eq!(crater_shape(&s, 1.62, 900.0).0, "multi-ring");
}

#[test]
fn every_feature_answers_why_it_is_here() {
    for name in ["Moon", "Mars", "Venus", "Ganymede"] {
        let (f, field) = run_for(name);
        let r = field.radius_km();
        for c in &f.craters {
            assert!(
                c.cause.contains("Gyr ago") && c.cause.contains("km across"),
                "{}",
                c.cause
            );
            let why = f.explain(c.lat_deg, c.lon_deg, r);
            assert!(
                why.iter().any(|l| l.starts_with(&format!("{}:", c.id))),
                "{name} {}",
                c.id
            );
        }
        for p in &f.plains {
            assert!(p.cause.contains("heat flow"), "{}", p.cause);
        }
        for sh in &f.shields {
            assert!(sh.cause.to_lowercase().contains("gravity"), "{}", sh.cause);
            assert!(sh.height_m <= 9000.0 * 9.81 / field.gravity() + 1.0);
        }
        assert!(
            !f.refs.is_empty()
                && f.refs
                    .iter()
                    .all(|x| x.contains("doi:") || x.contains("http"))
        );
    }
    // The ground away from any feature still says what holds it.
    let (f, field) = run_for("Moon");
    let why = f.explain(0.0, 0.0, field.radius_km());
    assert!(why.last().unwrap().contains("held craters since"));
}

#[test]
fn chronicle_spikes_add_dated_craters() {
    let mut s = settings();
    s.impacts.listed_d_frac = 0.004;
    s.impacts.listed_max = 200_000;
    s.impacts.volcanism.bury_d_km = 0.0;
    let quiet = run_with(&s, "Venus", &ImpactHistory::lunar(), None)
        .unwrap()
        .0;
    let spiked = ImpactHistory {
        flux_scale: 1.0,
        spikes: vec![Spike {
            t_myr: 4300.0,
            earth: 1e-6,
            efold_myr: 20.0,
            event: "upheaval event 7 (a stirred belt)".into(),
        }],
        source: "test spike".into(),
    };
    let (loud, _) = run_with(&s, "Venus", &spiked, None).unwrap();
    let from_spike = loud
        .craters
        .iter()
        .filter(|c| c.cause.contains("upheaval event 7"))
        .count();
    eprintln!(
        "quiet {} craters, spiked {} ({from_spike} from the spike)",
        quiet.craters.len(),
        loud.craters.len()
    );
    assert!(loud.craters.len() > quiet.craters.len());
    assert!(from_spike > 0);
    for c in loud
        .craters
        .iter()
        .filter(|c| c.cause.contains("upheaval event 7"))
    {
        assert!(c.formed_myr >= 4300.0, "{}", c.formed_myr);
    }
}

#[test]
fn lava_plains_bury_and_flood_older_craters() {
    let mut s = settings();
    s.impacts.volcanism.bury_d_km = 120.0;
    let (f, _) = run_with(&s, "Moon", &ImpactHistory::lunar(), None).unwrap();
    assert!(
        f.erased.get("buried by lava plains").copied().unwrap_or(0) > 0,
        "{:?}",
        f.erased
    );
    let (m, _) = run_for("Mars");
    let flooded: Vec<_> = m.craters.iter().filter(|c| c.flooded).collect();
    assert!(!flooded.is_empty());
    assert!(flooded
        .iter()
        .all(|c| c.cause.contains("flooded by plains:")));
    // Mars builds tall shields; the Moon, too small, builds none.
    assert!(m.shields.iter().any(|x| x.height_m > 15_000.0));
    assert!(run_for("Moon").0.shields.is_empty());
}

#[test]
fn small_craters_resolve_per_cell_and_respect_the_air() {
    let (f, field) = run_for("Moon");
    let a = field.cell_craters(f.listed_d_km, 6, 2, 3, 4);
    let b = field.cell_craters(f.listed_d_km, 6, 2, 3, 4);
    assert_eq!(a, b);
    let (lo, hi) = field.band(f.listed_d_km, 6);
    let n = field.band_cells(hi);
    let mut total = 0usize;
    for i in 0..n.min(20) {
        let c = field.cell_craters(f.listed_d_km, 6, 2, i, 0);
        assert!(c.iter().all(|x| x.d_km >= lo && x.d_km < hi));
        total += c.len();
    }
    assert!(total > 0);
    // Venus's thick air stops anything under about 3 km.
    let (v, vf) = run_for("Venus");
    assert!(
        vf.atm_cut_km() > 2.5 && vf.atm_cut_km() < 3.5,
        "{}",
        vf.atm_cut_km()
    );
    for k in 0..8 {
        let (lo, hi) = vf.band(v.listed_d_km, k);
        let n = vf.band_cells(hi);
        for i in 0..n.min(30) {
            for c in vf.cell_craters(v.listed_d_km, k, 0, i, n / 2) {
                assert!(c.d_km >= vf.atm_cut_km(), "{} km (band {lo}..{hi})", c.d_km);
            }
        }
    }
}

#[test]
fn mechanism_slots_refuse_unknown_atoms() {
    let mut s = settings();
    s.impacts.mechanisms.chronology = "kzinti-bombardment".into();
    let e = run_with(&s, "Moon", &ImpactHistory::lunar(), None).unwrap_err();
    assert!(e.contains("chronology") && e.contains("neukum-2001"), "{e}");
    s = settings();
    s.impacts.mechanisms.volcanism = "none".into();
    let (f, _) = run_with(&s, "Mars", &ImpactHistory::lunar(), None).unwrap();
    assert!(f.plains.is_empty() && f.shields.is_empty());
}

#[test]
fn the_same_world_gives_the_same_features() {
    let (a, _) = run_for("Mercury");
    let (b, _) = run_for("Mercury");
    assert_eq!(a, b);
}

fn cratered_atlas(name: &str, samples: u32) -> crate::atlas::Atlas {
    let mut s = settings();
    s.atlas.tile_samples = samples;
    let b = body(name);
    let e = envelope(&s, &b);
    let seed = Seed::from_text("sse4-impacts-tests");
    let w = WaterHistory::steady(b.oceans, b.pressure_bar, b.age_gyr);
    let plates = crate::plates::history(&s, &e, &seed, b.age_gyr);
    let inp = ImpactInputs {
        history: &ImpactHistory::lunar(),
        water: &w,
        pressure_bar: b.pressure_bar,
        plates: Some(&plates),
    };
    let (f, field) = impacts(&s, &e, &seed, &inp, b.age_gyr).unwrap();
    crate::atlas::Atlas::new(&s, &e, &seed, b.age_gyr, vec![]).with_impacts(f, field)
}

#[test]
fn craters_refine_to_the_fact_floor_within_tolerance() {
    use crate::atlas::{downsample_error_m, TileKey};
    for name in ["Moon", "Mars", "Mercury", "Vesta"] {
        let a = cratered_atlas(name, 32);
        let floor = a.level_for(a.settings().fact_floor_m);
        let mut worst: f64 = 0.0;
        // A chain of parents down to the fact floor, at a crater on the list.
        let c = a
            .features()
            .unwrap()
            .craters
            .iter()
            .filter(|c| !c.ghost && c.d_km >= a.min_feature_km(2))
            .nth(2)
            .unwrap()
            .clone();
        let (f, u, v) = crate::atlas::face_of(c.dir);
        for level in 0..floor.min(12) {
            let n = (1u64 << level) as f64;
            let key = TileKey::new(
                f,
                level,
                (((u + 1.0) / 2.0) * n) as u32,
                (((v + 1.0) / 2.0) * n) as u32,
            )
            .unwrap();
            let p = a.build(key);
            if level == 2 {
                assert!(
                    p.tags.iter().any(|t| t == &c.id),
                    "{name}: {:?} lacks {}",
                    p.tags,
                    c.id
                );
                assert!(p.pending.iter().all(|x| !x.contains("S4")));
            }
            for ch in key.children() {
                let t = a.build(ch);
                let err = downsample_error_m(&p, &t);
                let tol = a.tolerance_m(ch.level);
                worst = worst.max(err / tol);
                assert!(err <= tol, "{name} {key:?} -> {ch:?}: {err} m over {tol} m");
            }
        }
        eprintln!("{name}: worst error {:.0}% of tolerance", 100.0 * worst);
    }
}

#[test]
fn a_crater_shows_in_the_tiles_heights() {
    use crate::atlas::TileKey;
    let a = cratered_atlas("Moon", 64);
    let f = a.features().unwrap();
    // The biggest fresh crater: its centre sits well below its rim.
    let c = f
        .craters
        .iter()
        .filter(|c| !c.ghost)
        .max_by(|x, y| x.d_km.total_cmp(&y.d_km))
        .unwrap();
    let plain = crate::atlas::Atlas::new(
        &settings(),
        &envelope(&settings(), &body("Moon")),
        &Seed::from_text("sse4-impacts-tests"),
        4.5,
        vec![],
    );
    let (fc, u, v) = crate::atlas::face_of(c.dir);
    let level = 2u8;
    let n = (1u64 << level) as f64;
    let key = TileKey::new(
        fc,
        level,
        (((u + 1.0) / 2.0) * n) as u32,
        (((v + 1.0) / 2.0) * n) as u32,
    )
    .unwrap();
    let with = a.build(key);
    let _ = plain;
    let lo = *with.elevation_dm.iter().min().unwrap() as f64 / 10.0;
    let hi = *with.elevation_dm.iter().max().unwrap() as f64 / 10.0;
    eprintln!(
        "Moon tile over {} ({:.0} km, depth {:.0} m): {lo:.0} to {hi:.0} m",
        c.id, c.d_km, c.depth_m
    );
    assert!(hi - lo > 0.5 * c.depth_m);
}

#[test]
fn golden_impacts() {
    let got = crate::scenario::impacts_golden();
    let path = format!(
        "{}/../../tests/golden/surface-impacts.json",
        env!("CARGO_MANIFEST_DIR")
    );
    let text = serde_json::to_string_pretty(&got).unwrap() + "\n";
    if std::env::var("SSE_BLESS").is_ok() {
        std::fs::write(&path, &text).unwrap();
        return;
    }
    let want = std::fs::read_to_string(&path).expect("run with SSE_BLESS=1");
    assert_eq!(
        text.replace("\r\n", "\n"),
        want.replace("\r\n", "\n"),
        "golden surface-impacts changed"
    );
}

//! G6 checks: edit locality, event rates against the rings' rates (section 6),
//! cross-patch events, encounters, arm crossings, and the "ink a merger" scenario.

use crate::cosmology::Cosmology;
use crate::envelope::GalaxyEnvelope;
use crate::events::{EventSite, Events, Kind, SitePath};
use crate::galaxy::{self, Edit, Galaxy};
use crate::haloes::Halo;
use crate::rings::Disc;
use crate::scenario::{base_cosmology_pack, CosmologyPack};
use crate::web::{Web, WebParams};
use sse_core::math as m;
use sse_core::{Address, Seed};

struct Setup {
    cp: CosmologyPack,
    c: Cosmology,
    halo: Halo,
}

fn setup() -> Setup {
    let cp = CosmologyPack::from_packs(&[base_cosmology_pack()]).expect("settings");
    let c = Cosmology::new(cp.params.clone());
    let u = Address::universe("g6").unwrap();
    let web = Web::new(
        &c,
        WebParams {
            octaves: 4,
            ..cp.web.clone()
        },
        &Seed::from_text("g6"),
        &u,
    )
    .unwrap();
    let hs = crate::haloes::Haloes {
        cosmo: &c,
        web: &web,
        params: cp.haloes.clone(),
        seed: Seed::from_text("g6"),
        universe: u,
    };
    let halo = hs.median(1.0e12, "milky-way-like");
    Setup { cp, c, halo }
}

fn grow(s: &Setup, edits: &[Edit]) -> (Galaxy, Disc) {
    let g = galaxy::evolve_with(&s.c, &s.cp.galaxy, &s.halo, edits);
    let base = galaxy::evolve(&s.c, &s.cp.galaxy, &s.halo);
    let clock = crate::rings::inflow_clock(&base, &s.cp.rings);
    let addr: Address = s.halo.address.parse().unwrap();
    let d = crate::rings::evolve_clocked(
        &g,
        &s.cp.galaxy,
        &s.cp.rings,
        Seed::from_text("g6"),
        &addr,
        Some(clock),
    );
    if edits.is_empty() {
        assert_eq!(
            d,
            crate::rings::evolve(&g, &s.cp.galaxy, &s.cp.rings, Seed::from_text("g6"), &addr)
        );
    }
    (g, d)
}

fn env<'a>(s: &Setup, g: &'a Galaxy, d: &'a Disc) -> GalaxyEnvelope<'a> {
    GalaxyEnvelope {
        galaxy: g,
        disc: d,
        params: s.cp.envelope.clone(),
        seed: Seed::from_text("g6"),
        address: s.halo.address.parse().unwrap(),
        remains: Default::default(),
    }
}

fn events<'a>(s: &'a Setup, e: &'a GalaxyEnvelope<'a>, edits: &'a [Edit]) -> Events<'a> {
    Events {
        env: e,
        halo: &s.halo,
        edits,
        green: s.cp.green.clone(),
        stars: s.cp.stars.clone(),
        params: s.cp.events.clone(),
        seed: Seed::from_text("g6"),
    }
}

fn site(name: &str, grid_pc: [f64; 3]) -> EventSite {
    EventSite {
        address: Address::universe("g6")
            .unwrap()
            .child_key("site", name)
            .unwrap(),
        path: SitePath::Fixed { grid_pc },
    }
}

#[test]
fn g6_edits_change_only_what_follows() {
    let s = setup();
    let (g0, d0) = grow(&s, &[]);
    assert_eq!(
        g0,
        galaxy::evolve(&s.c, &s.cp.galaxy, &s.halo),
        "no edits is the plain galaxy"
    );
    let cases = [
        vec![Edit::Merger {
            t_gyr: 9.0,
            ratio: 0.5,
        }],
        vec![Edit::Starburst {
            t_gyr: 6.0,
            gas_fraction: 0.6,
        }],
        vec![Edit::QuietNucleus {
            from_gyr: 4.0,
            to_gyr: 8.0,
        }],
        vec![Edit::RemoveMerger { index: 0 }],
    ];
    for edits in cases {
        assert!(galaxy::check_edits(&s.halo, &edits).is_ok());
        let t = edits[0].from_gyr(&s.halo);
        let (g1, d1) = grow(&s, &edits);
        let before = |g: &Galaxy| {
            g.history
                .iter()
                .filter(|x| x.t_gyr < t)
                .cloned()
                .collect::<Vec<_>>()
        };
        assert_eq!(
            before(&g0),
            before(&g1),
            "{edits:?}: history before {t} Gyr is untouched"
        );
        assert!(!before(&g0).is_empty());
        assert_ne!(g0.history, g1.history, "{edits:?} changes something");
        let k_edit = ((t - g0.t_start_gyr) / g0.step_gyr) as usize;
        let steps = |x: &[f64]| x[..k_edit].to_vec();
        assert_eq!(steps(&g0.inflow), steps(&g1.inflow));
        // Rings: every ring state stamped before the edit is identical.
        for (a, b) in d0.states.iter().zip(&d1.states) {
            // State k is the ring at the end of step k.
            let n = (0..a.len()).take_while(|&k| d0.time(k + 1) <= t).count();
            assert!(n > 0);
            assert_eq!(a[..n], b[..n], "{edits:?}: ring history before the edit");
        }
        println!(
            "{edits:?}: first change at {:?} Gyr, morphology {:?} -> {:?}, bulge {:.3} -> {:.3}, black hole {:.2e} -> {:.2e}",
            crate::events::compare(&g0, &g1).first_change_gyr,
            g0.morphology,
            g1.morphology,
            g0.now.bulge / g0.now.stars,
            g1.now.bulge / g1.now.stars,
            g0.now.black_hole,
            g1.now.black_hole
        );
    }
    // What the edits do.
    let (gm, _) = grow(
        &s,
        &[Edit::Merger {
            t_gyr: 9.0,
            ratio: 0.5,
        }],
    );
    assert!(
        gm.now.bulge / gm.now.stars > g0.now.bulge / g0.now.stars,
        "a merger grows the bulge"
    );
    let (gq, _) = grow(
        &s,
        &[Edit::QuietNucleus {
            from_gyr: 0.0,
            to_gyr: 20.0,
        }],
    );
    // A quiet nucleus never feeds; it still gains the holes that arrive with
    // merging galaxies (ex-situ stars), and nothing else.
    let exsitu = gq.now.black_hole - s.cp.galaxy.seed_black_hole;
    assert!(gq.active_nuclei.is_empty() && exsitu >= 0.0);
    assert!(
        exsitu <= s.cp.galaxy.exsitu_black_hole * gq.now.bulge * (1.0 + 1e-9),
        "{exsitu}"
    );
    assert!(gq.now.black_hole < g0.now.black_hole);
    // Bad edits are refused.
    assert!(galaxy::check_edits(
        &s.halo,
        &[Edit::Merger {
            t_gyr: 9.0,
            ratio: 1.5
        }]
    )
    .is_err());
    assert!(galaxy::check_edits(&s.halo, &[Edit::RemoveMerger { index: 999 }]).is_err());
    assert!(galaxy::check_edits(
        &s.halo,
        &[Edit::QuietNucleus {
            from_gyr: 5.0,
            to_gyr: 4.0
        }]
    )
    .is_err());
}

#[test]
fn g6_event_rates_follow_the_rings() {
    let s = setup();
    let (g, d) = grow(&s, &[]);
    let e = env(&s, &g, &d);
    let ev = events(&s, &e, &[]);
    let p = &s.cp.events;
    let c = p.sn_cell_pc;
    // The whole disc now: expected events per Gyr summed over annuli (32 azimuths, a
    // 2 kpc column) against the rings' own present rates (section 6).
    let bin = m::floor(13_700.0 / p.sn_bin_myr) as i64;
    let (mut cc, mut ia) = (0.0, 0.0);
    for ring in &d.rings {
        let r = ring.radius_kpc * 1000.0;
        let area =
            m::PI * (ring.outer_kpc * ring.outer_kpc - ring.inner_kpc * ring.inner_kpc) * 1.0e6;
        let (mut a, mut b) = (0.0, 0.0);
        for q in 0..32 {
            let phi = m::TAU * f64::from(q) / 32.0;
            let (x, y) = (r * m::cos(phi), r * m::sin(phi));
            for k in -10..10 {
                let x = ev.expected_cell([m::floor(x / c) as i64, m::floor(y / c) as i64, k], bin);
                a += x[0];
                b += x[1];
            }
        }
        let per_gyr = 1000.0 / p.sn_bin_myr / (32.0 * c * c);
        cc += a * per_gyr * area;
        ia += b * per_gyr * area;
    }
    let (cc_rings, ia_rings) = (d.core_collapse_rate() * 1.0e9, d.ia_rate() * 1.0e9);
    println!(
        "disc now: core collapse {:.3} per century (rings {:.3}), type Ia {:.3} (rings {:.3})",
        cc / 1.0e7,
        cc_rings / 1.0e7,
        ia / 1.0e7,
        ia_rings / 1.0e7
    );
    assert!(
        (cc / cc_rings - 1.0).abs() < 0.25,
        "core collapse {cc} against {cc_rings}"
    );
    assert!(
        (ia / ia_rings - 1.0).abs() < 0.25,
        "type Ia {ia} against {ia_rings}"
    );

    // Drawn against expected, over a block of cells near the Sun across 1 Gyr.
    let r0 = e.sun_radius_kpc() * 1000.0;
    let (mut want, mut got) = ([0.0; 3], [0.0; 3]);
    for i in 0..4 {
        for k in -2..2 {
            for b in 0..100 {
                let cell = [m::floor(r0 / c) as i64 + i, 0, k];
                let bin = 1270 + b;
                let x = ev.expected_cell(cell, bin);
                for kind in ev.cell_events(cell, bin).unwrap() {
                    let s = match kind.kind {
                        Kind::CoreCollapse => 0,
                        Kind::TypeIa => 1,
                        _ => 2,
                    };
                    got[s] += 1.0;
                    // Every event inside its cell and bin.
                    assert!((kind.t_myr / p.sn_bin_myr).floor() as i64 == bin);
                    assert!((kind.grid_pc[2] / c).floor() as i64 == k);
                }
                for s in 0..3 {
                    want[s] += x[s];
                }
            }
        }
    }
    println!("drawn near the Sun, 16 cells x 1 Gyr: core collapse {} (expected {:.1}), Ia {} ({:.1}), kilonovae {} ({:.2})", got[0], want[0], got[1], want[1], got[2], want[2]);
    for s in 0..3 {
        assert!(
            (got[s] - want[s]).abs() <= 4.0 * m::sqrt(want[s]) + 2.0,
            "{s}: {} against {}",
            got[s],
            want[s]
        );
    }
    assert!(want[0] > 100.0 && want[1] > 20.0, "enough events to test");

    // Gamma-ray bursts: columns within grb_kpc give the envelope's rate at the site.
    let l = e.local(
        crate::envelope::Place {
            r_kpc: r0 / 1000.0,
            phi: 0.0,
            z_pc: 0.0,
            t_gyr: 13.75,
        },
        &s.cp.green,
    );
    let gc = p.grb_cell_pc;
    let reach = s.cp.envelope.grb_kpc * 1000.0;
    let n = (reach / gc) as i64 + 1;
    let mut sum = 0.0;
    let sx = ev.to_grid([r0, 0.0, 0.0], 13_750.0);
    for i in -n..=n {
        for j in -n..=n {
            let col = [
                m::floor(sx[0] / gc) as i64 + i,
                m::floor(sx[1] / gc) as i64 + j,
            ];
            let mid = [(col[0] as f64 + 0.5) * gc, (col[1] as f64 + 0.5) * gc];
            if m::hypot(mid[0] - sx[0], mid[1] - sx[1]) <= reach {
                sum += ev.expected_column(col, 137);
            }
        }
    }
    let per_gyr = sum * 1000.0 / p.grb_bin_myr;
    println!(
        "lethal bursts at the Sun now: {per_gyr:.3} per Gyr from columns (envelope {:.3})",
        l.grb_per_gyr
    );
    assert!((per_gyr / l.grb_per_gyr - 1.0).abs() < 0.35);
}

#[test]
fn g6_events_cross_patch_boundaries() {
    let s = setup();
    let (g, d) = grow(&s, &[]);
    let e = env(&s, &g, &d);
    let ev = events(&s, &e, &[]);
    let r0 = e.sun_radius_kpc() * 1000.0;
    // Two sites 60 pc apart, either side of a cell face, over 500 Myr.
    let a = site("a", [r0 - 30.0, 0.0, 0.0]);
    let b = site("b", [r0 + 30.0, 0.0, 0.0]);
    let (t0, t1) = (13_000.0, 13_500.0);
    let xa = ev.exposure(&a, t0, t1).unwrap();
    let xb = ev.exposure(&b, t0, t1).unwrap();
    let sn = |k: Kind| {
        matches!(
            k,
            Kind::CoreCollapse | Kind::TypeIa | Kind::NeutronStarMerger
        )
    };
    let shared: Vec<_> = xa
        .hits
        .iter()
        .filter(|h| sn(h.kind))
        .filter_map(|h| xb.hits.iter().find(|x| x.id == h.id).map(|x| (h, x)))
        .collect();
    println!(
        "site a: {} supernova-scale hits, b: {}, shared {}",
        xa.hits.iter().filter(|h| sn(h.kind)).count(),
        xb.hits.iter().filter(|h| sn(h.kind)).count(),
        shared.len()
    );
    assert!(!shared.is_empty(), "neighbours see the same explosions");
    for (ha, hb) in &shared {
        assert_eq!(ha.occurred_myr, hb.occurred_myr);
        assert!(
            (ha.distance_pc - hb.distance_pc).abs() <= 60.0 + 1.0e-9,
            "triangle inequality"
        );
        // Nearer site is reached first, by light and by the shock.
        if ha.distance_pc < hb.distance_pc && ha.harmful == hb.harmful && ha.remnant == hb.remnant {
            assert!(ha.activated_myr <= hb.activated_myr);
        }
        assert!(ha.activated_myr >= ha.occurred_myr);
    }
    // The supernova count matches the volume rate around the site (4 sigma).
    let near: Vec<_> = xa
        .hits
        .iter()
        .filter(|h| sn(h.kind) && h.distance_pc <= s.cp.events.remnant_pc)
        .collect();
    let l = e.local(
        crate::envelope::Place {
            r_kpc: r0 / 1000.0,
            phi: 0.0,
            z_pc: 0.0,
            t_gyr: 13.25,
        },
        &s.cp.green,
    );
    let rp = s.cp.events.remnant_pc;
    let lam = (l.cc_gyr_pc3 + l.ia_gyr_pc3) * 4.0 / 3.0 * m::PI * rp * rp * rp * 0.5;
    println!(
        "within {rp} pc: {} (about {lam:.1} expected, mid-plane rates)",
        near.len()
    );
    assert!((near.len() as f64) < lam * 3.0 + 10.0 && near.len() as f64 > lam / 3.0 - 3.0);
    // Windows compose: [t0, tm) and [tm, t1) hold every hit of [t0, t1).
    let tm = 13_237.0;
    let x1 = ev.exposure(&a, t0, tm).unwrap();
    let x2 = ev.exposure(&a, tm, t1).unwrap();
    for h in &xa.hits {
        assert!(
            x1.hits.contains(h) || x2.hits.contains(h),
            "{} lost at the window seam",
            h.id
        );
    }
    for h in x1.hits.iter().chain(&x2.hits) {
        assert!(xa.hits.contains(h), "{} only in a part", h.id);
    }
    // A star on its orbit sees the same cells' events as a fixed site where their paths meet.
    let orbit = crate::stars::Orbit {
        r_guide_pc: r0,
        amp_r_pc: 300.0,
        amp_z_pc: 80.0,
        phase_r: 0.0,
        phase_z: 0.0,
        phi0: 0.0,
        t0_myr: t0,
        omega: s.cp.events.grid_rotation_kms_kpc * 1.022_712 / 1000.0,
        kappa: 0.037,
        nu: 0.07,
    };
    let star = EventSite {
        address: a.address.clone(),
        path: SitePath::Orbit(orbit),
    };
    let xs = ev.exposure(&star, t0, t1).unwrap();
    for h in xs.hits.iter().filter(|h| sn(h.kind)) {
        let at = ev.site_at(&star.path, h.occurred_myr);
        let near_fixed = site("probe", at);
        let probe = ev
            .exposure(&near_fixed, h.occurred_myr - 1.0, h.occurred_myr + 2.0)
            .unwrap();
        assert!(
            probe.hits.iter().any(|x| x.id == h.id),
            "the moving star's hit {} is a real cell event",
            h.id
        );
    }
    println!(
        "star on an epicycle: {} supernova-scale hits",
        xs.hits.iter().filter(|h| sn(h.kind)).count()
    );
}

#[test]
fn g6_encounters_and_arms() {
    let s = setup();
    let (g, d) = grow(&s, &[]);
    let e = env(&s, &g, &d);
    let ev = events(&s, &e, &[]);
    let r0 = e.sun_radius_kpc() * 1000.0;
    let a = site("sun-like", [r0, 0.0, 20.0]);
    let (t0, t1) = (11_800.0, 13_800.0);
    let enc = ev.encounters(&a, t0, t1).unwrap();
    let l = e.local(
        crate::envelope::Place {
            r_kpc: r0 / 1000.0,
            phi: 0.0,
            z_pc: 20.0,
            t_gyr: 12.8,
        },
        &s.cp.green,
    );
    let lam = l.encounters_per_myr * (t1 - t0);
    let showers = enc.iter().filter(|h| h.shower).count() as f64;
    let mut v: Vec<f64> = enc.iter().map(|h| h.passage.unwrap()[1]).collect();
    v.sort_by(f64::total_cmp);
    let mut ms: Vec<f64> = enc.iter().map(|h| h.passage.unwrap()[2]).collect();
    ms.sort_by(f64::total_cmp);
    println!(
        "encounters within {} pc over 2 Gyr: {} (about {lam:.0}), showers {showers} (share {:.2}, area share {:.2}), median speed {:.0} km/s, median mass {:.2}",
        s.cp.envelope.encounter_pc,
        enc.len(),
        showers / enc.len() as f64,
        m::pow(s.cp.events.shower_pc / s.cp.envelope.encounter_pc, 2.0),
        v[v.len() / 2],
        ms[ms.len() / 2]
    );
    let n = enc.len() as f64;
    assert!(
        (n - lam).abs() < 5.0 * m::sqrt(lam) + 0.1 * lam,
        "count {n} against {lam}"
    );
    let share = (s.cp.events.shower_pc / s.cp.envelope.encounter_pc)
        * (s.cp.events.shower_pc / s.cp.envelope.encounter_pc);
    assert!((showers / n - share).abs() < 0.08);
    assert!(v[v.len() / 2] > 20.0 && v[v.len() / 2] < 120.0);
    assert!(ms[ms.len() / 2] < 0.6, "most passing stars are M dwarfs");
    assert!(enc
        .iter()
        .all(|h| h.occurred_myr >= t0 && h.occurred_myr < t1));

    // Arm crossings alternate, and in-arm stretches are where the arm factor is high.
    let arms = ev.arm_crossings(&a.path, 4_000.0, 13_800.0);
    println!("arm crossings 4 to 13.8 Gyr: {}", arms.len());
    assert!(arms.len() >= 2);
    for w in arms.windows(2) {
        assert_ne!(w[0].kind, w[1].kind, "entries and exits alternate");
        assert!(w[0].occurred_myr < w[1].occurred_myr);
    }
}

#[test]
fn g6_ink_a_merger_reproduces_from_its_manifest() {
    let cp = CosmologyPack::from_packs(&[base_cosmology_pack()]).unwrap();
    let man = crate::scenario::ink_a_merger();
    // The manifest round-trips, and the run is the same each time.
    let text = serde_json::to_string(&man).unwrap();
    let again: crate::events::Manifest = serde_json::from_str(&text).unwrap();
    assert_eq!(man, again);
    let r1 = crate::events::run_manifest(&cp, &man).unwrap();
    let r2 = crate::events::run_manifest(&cp, &again).unwrap();
    assert_eq!(r1, r2);
    println!("{}", serde_json::to_string_pretty(&r1).unwrap());
    assert_eq!(r1["history_unchanged_before_edit"], true);
    assert_eq!(r1["events_unchanged_before_edit"], true);
    assert!(r1["after"]["bulge_fraction"].as_f64() > r1["before"]["bulge_fraction"].as_f64());
    assert_eq!(
        r1["after"]["hits"]["merger"].as_u64().unwrap(),
        r1["before"]["hits"]["merger"].as_u64().unwrap_or(0) + 1
    );
    // A wrong manifest is refused.
    let mut bad = man.clone();
    bad.edits = vec![Edit::Merger {
        t_gyr: 9.0,
        ratio: 0.0,
    }];
    assert!(crate::events::run_manifest(&cp, &bad).is_err());
    crate::golden::check(
        "galaxy-events.json",
        &crate::scenario::galaxy_events_golden(),
    );
}

#[test]
fn g6_pack_holds_the_defaults() {
    let cp = CosmologyPack::from_packs(&[base_cosmology_pack()]).unwrap();
    assert_eq!(cp.events, crate::events::EventParams::default());
}

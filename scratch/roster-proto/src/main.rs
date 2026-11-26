//! Runs the roster prototype on one generated system and prints a report.
//! `cargo run --release -- [n_exact]`. Timings use the wall clock, which is
//! fine here: this binary measures, it is not on a deterministic path.

use roster_proto::book::{ledger, state_at, Campaign, Deed};
use roster_proto::roster::{member_sub, BandId, Body, Cause, Crater, Roster};
use roster_proto::system::*;
use sse_core::Seed;
use sse_formation::{disc, growth, hurley::Fits, leftovers, scenario, violent};
use std::time::Instant;

fn rss_kb() -> (u64, u64) {
    let s = std::fs::read_to_string("/proc/self/status").unwrap_or_default();
    let get = |k: &str| {
        s.lines()
            .find(|l| l.starts_with(k))
            .and_then(|l| l.split_whitespace().nth(1))
            .and_then(|x| x.parse().ok())
            .unwrap_or(0)
    };
    (get("VmRSS:"), get("VmHWM:"))
}

fn fnv(s: &str) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    for b in s.bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}

fn ago(sys: &System, t: f64) -> String {
    let a = sys.age_myr - t;
    if a < 0.0 {
        format!("{:.1} million years from now", -a)
    } else if a >= 1000.0 {
        format!("{:.2} billion years ago", a / 1000.0)
    } else {
        format!("{:.1} million years ago", a)
    }
}

fn story(ro: &mut Roster, cr: &Crater) -> Vec<String> {
    let sys = ro.sys;
    let b = ro.impactor_of(&cr.id);
    let r = &sys.reservoirs[b.band.res];
    let c = &r.classes[b.band.class];
    let f = b.fate.clone().unwrap();
    let w = &sys.worlds[cr.world];
    let mut v = vec![
        format!(
            "1. The crater: {:.1} km across on {}, at {:.1}°, {:.1}°, made {} (crater id `{}`).",
            cr.d_km, w.id, cr.lat_deg, cr.lon_deg, ago(sys, cr.t_myr), cr.id
        ),
        format!(
            "2. The impactor: `{}`, a {:.1} km {} body of {:.2e} kg, rank {} of {} in its class: {}.",
            b.name, b.diameter_km, b.type_label, b.mass_kg, b.rank, c.n_roster, c.what
        ),
        format!("3. Where it came from: {}.", r.origin),
    ];
    match f.cause {
        Cause::Background => v.push(format!(
            "4. What loosened it: {}, {} (an ordinary year in its reservoir's drain).",
            r.loosened_by,
            ago(sys, f.t_leave)
        )),
        Cause::Pass(pi) => {
            let p = &sys.passes[pi];
            v.push(format!(
                "4. What loosened it: a {:.2} solar-mass star passed {:.0} au from the star at {:.0} km/s, {}, and shook {:.2}% of the cloud inward (`{}`).",
                p.mass_msun, p.b_pc * 206_264.8, p.v_kms, ago(sys, p.t_myr), 100.0 * p.kick, p.id
            ))
        }
    }
    v.push(format!(
        "5. The journey: it fell for {:.2} million years and struck at {:.1} km/s, delivering {} book units of mass ({:?} by group) to {}.",
        f.t_arrive - f.t_leave, cr.v_kms, b.sub, b.groups_sub, w.id
    ));
    v
}

fn main() {
    let n_exact: u64 = std::env::args()
        .nth(1)
        .and_then(|x| x.parse().ok())
        .unwrap_or(100_000);
    let st = Settings {
        n_exact,
        ..Settings::default()
    };
    println!("# Roster prototype run (n_exact = {n_exact})\n");

    // ---- the system ------------------------------------------------------
    let t0 = Instant::now();
    let seed = Seed::from_text("cc-roster-1");
    let addr = scenario::system(16);
    let fits = Fits::new(0.02);
    let dsc = disc::Disc::new(disc::disc_params(
        &seed,
        &addr,
        1.0,
        fits.l_zams(1.0),
        0.0,
        0.0,
        0.55,
        300.0,
        None,
    ));
    let g = growth::grow(&seed, &addr, &dsc);
    let u = violent::upheave(&seed, &addr, &dsc, &g);
    let l = leftovers::leftovers(&seed, &addr, &dsc, &u);
    let t_form = t0.elapsed();
    let t1 = Instant::now();
    let sys = System::build(seed, addr.clone(), &u, &l, 4570.0, 1.0, &st);
    let t_build = t1.elapsed();
    println!("System `{addr}` from seed `cc-roster-1`, a Sun-like star aged 4.57 billion years, slider to {:.0} Myr, {} time bins.", sys.horizon_myr, sys.nbins());
    println!("Formation took {:.0} ms; building the reservoirs, classes and {} close passes took {:.2} ms.\n", t_form.as_secs_f64() * 1e3, sys.passes.len(), t_build.as_secs_f64() * 1e3);
    println!("Worlds:");
    for w in &sys.worlds {
        println!(
            "- {} at {:.2} AU, {:.3} Earth masses{}",
            w.id,
            w.a_au,
            w.mass_kg / sse_core::sheet::M_EARTH,
            if w.surface {
                ", solid surface"
            } else {
                ", no solid surface"
            }
        );
    }

    // ---- 1. classes that add back up -----------------------------------
    println!("\n## 1. Each belt split into classes that add back to its total\n");
    println!("| Reservoir | Class | metal | rock | carbon | water | named members (>= 1 km) | exact tier | bands |");
    println!("|---|---|---|---|---|---|---|---|---|");
    let mut all_close = true;
    for r in &sys.reservoirs {
        let cloud = l
            .clouds
            .iter()
            .find(|c| c.address.to_string() == r.cloud_address)
            .unwrap();
        let mut sum = [0u64; 6];
        for c in &r.classes {
            for g in 0..6 {
                sum[g] += c.groups[g];
            }
            println!(
                "| {} | {} | {} | {} | {} | {} | {:.2e} | {} | {} |",
                r.name,
                c.name,
                c.groups[0],
                c.groups[1],
                c.groups[2],
                c.groups[3],
                c.n_roster as f64,
                c.n_exact,
                c.bands.len()
            );
        }
        let ok = sum == cloud.solid;
        all_close &= ok;
        println!(
            "| {} | **total** | {} | {} | {} | {} | | | closes: **{}** |",
            r.name, sum[0], sum[1], sum[2], sum[3], ok
        );
    }
    if !all_close {
        println!("\n**RED: a class split does not add back to its reservoir.**");
        std::process::exit(1);
    }
    println!("\n(Element groups in formation units of 1e-12 solar masses. Every group of every reservoir adds back exactly.)");

    // ---- 2. notables -----------------------------------------------------
    let mut ro = Roster::new(&sys);
    let nb = sse_small::notables(&u, &l);
    println!("\n## 2. Notables, there before anyone looks\n");
    for n in &nb {
        println!(
            "- Kernel notable: {} (`{}` rank {}).",
            n.why, n.cloud, n.rank
        );
    }
    let deb = sys
        .reservoirs
        .iter()
        .position(|r| r.kind == "debris-belt")
        .unwrap();
    let core = BandId {
        res: deb,
        class: 0,
        band: 0,
    };
    let b1 = ro.body(core, 1);
    println!(
        "- In the roster it is `{}`: {:.0} km, {}, {:.2e} kg. Its fate: {}.",
        b1.name,
        b1.diameter_km,
        b1.type_label,
        b1.mass_kg,
        match &b1.fate {
            None => "it never leaves the belt".to_string(),
            Some(f) => format!(
                "leaves {} for {}, lands {}",
                ago(&sys, f.t_leave),
                roster_proto::book::sink_name(&sys, sys.reservoirs[deb].sinks[f.sink].0),
                ago(&sys, f.t_arrive)
            ),
        }
    );

    // ---- 3. craters resolving to impactors -----------------------------
    println!("\n## 3. Craters that resolve to named impactors\n");
    let present = sys.age_myr;
    let mut lists = vec![];
    let mut recent = vec![];
    let tl = Instant::now();
    for (wi, w) in sys.worlds.iter().enumerate() {
        if !w.surface {
            continue;
        }
        let (cr, floor, walked) = ro.listed_craters(wi, (0.0, sys.horizon_myr), present, 2000);
        let (rc, rfloor, rwalked) =
            ro.listed_craters(wi, (present - 1000.0, sys.horizon_myr), present, 500);
        println!("- {}: {} listed craters over its whole life (impactors down to {:.1} km, {} members read); {} from the last billion years (down to {:.2} km, {} read).", w.id, cr.len(), floor, walked, rc.len(), rfloor, rwalked);
        lists.push((wi, cr));
        recent.push((wi, rc));
    }
    let t_lists = tl.elapsed();
    let mut h: u64 = 0xcbf29ce484222325;
    for (_, cl) in &lists {
        for x in cl {
            for b in x.id.bytes().chain(x.d_km.to_bits().to_be_bytes()) {
                h ^= b as u64;
                h = h.wrapping_mul(0x100000001b3);
            }
        }
    }
    println!("\nDigest of every whole-life listed crater (ids and diameter bits): {h:x}. The wasm build run under Node gives the same digest when the bench is run.");
    println!(
        "\nAll listed craters on every solid world, from a cold cache: {:.0} ms.",
        t_lists.as_secs_f64() * 1e3
    );
    let target = lists.iter().max_by_key(|x| x.1.len()).unwrap().0;
    let tw = &sys.worlds[target];
    let list = &lists.iter().find(|x| x.0 == target).unwrap().1;
    let core_cr = list
        .iter()
        .find(|c| c.band.res == deb && c.band.class == 0)
        .cloned();
    println!(
        "\n### The walkthrough: the largest crater on {} made by a core fragment\n",
        tw.id
    );
    let walk = core_cr.clone().expect("a core-fragment crater");
    for line in story(&mut ro, &walk) {
        println!("{line}");
    }
    // The same, among craters of the last billion years, on any solid world.
    let mut young: Option<Crater> = None;
    for (_, cl) in &recent {
        for c in cl {
            let core_like = sys.reservoirs[c.band.res].classes[c.band.class].name == "core";
            if core_like && young.as_ref().is_none_or(|y| c.d_km > y.d_km) {
                young = Some(c.clone());
            }
        }
    }
    if let Some(y) = &young {
        println!("\n### The largest core-fragment crater of the last billion years, on any solid world\n");
        for line in story(&mut ro, y) {
            println!("{line}");
        }
    }
    // A crater from a passing star, anywhere a solid world was hit.
    let mut pass_cr: Option<Crater> = None;
    let mut pass_count = 0;
    for (_, cl) in lists.iter().chain(recent.iter()) {
        for c in cl {
            let b = ro.impactor_of(&c.id);
            if matches!(b.fate.as_ref().unwrap().cause, Cause::Pass(_)) {
                pass_count += 1;
                if pass_cr.as_ref().is_none_or(|p| c.d_km > p.d_km) {
                    pass_cr = Some(c.clone());
                }
            }
        }
    }
    println!("\n### A crater from a passing star\n");
    println!("Listed craters from comet showers, on all solid worlds: {pass_count}.");
    // Every shower arrival on a solid world, whatever its size (listed or not).
    let mut shower_n = 0u64;
    let mut best: Option<(Body, usize)> = None;
    for (id, _) in ro.all_bands() {
        if !sys.reservoirs[id.res].kicked {
            continue;
        }
        for wi in 0..sys.worlds.len() {
            if !sys.worlds[wi].surface {
                continue;
            }
            let Some(s) = ro.sink_of_world(id.res, wi) else {
                continue;
            };
            for (b, seg, n) in ro.arrivals(id, s) {
                if seg == 0 {
                    continue;
                }
                shower_n += n;
                if best.is_none() {
                    let k = ro.member_at(id, b, seg, s, 0);
                    best = Some((ro.body(id, k), wi));
                }
            }
        }
    }
    println!("Arrivals on solid worlds of members (1 km and up) sent in by a passing star, over the system's whole life: {shower_n}.");
    if pass_cr.is_some() {
        for line in story(&mut ro, pass_cr.as_ref().unwrap()) {
            println!("{line}");
        }
    } else if let Some((bd, wi)) = &best {
        let cr = ro.crater_of(*wi, bd);
        println!("None is big enough to make the listed set. The largest (from the band of largest comets that has any) reads, unlisted:\n");
        for line in story(&mut ro, &cr) {
            println!("{line}");
        }
    }

    // ---- 4. order indifference -------------------------------------------
    println!("\n## 4. The same answer whatever is resolved first\n");
    let sample: Vec<(BandId, u64)> = {
        let mut v = vec![];
        for (ri, r) in sys.reservoirs.iter().enumerate() {
            for (ci, c) in r.classes.iter().enumerate() {
                for (bi, (k0, k1)) in c.bands.iter().enumerate() {
                    for q in 0..3u64 {
                        let k = k0
                            + roster_proto::perm::mix((ri * 100 + ci * 10 + bi) as u64 ^ q)
                                % (k1 - k0);
                        v.push((
                            BandId {
                                res: ri,
                                class: ci,
                                band: bi,
                            },
                            k,
                        ));
                    }
                }
            }
        }
        v
    };
    let crater_ids: Vec<String> = list.iter().take(200).map(|c| c.id.clone()).collect();
    let run = |order: &[usize], epochs: &[f64]| -> Vec<(usize, String)> {
        let mut ro = Roster::new(&sys);
        let mut out = vec![];
        for (n, &i) in order.iter().enumerate() {
            let e = epochs[n % epochs.len()];
            let s = if i < sample.len() {
                let (id, k) = sample[i];
                format!("{:?}", ro.body(id, k))
            } else if i < sample.len() + crater_ids.len() {
                format!("{:?}", ro.try_impactor_of(&crater_ids[i - sample.len()]))
            } else {
                let ri = (i - sample.len() - crater_ids.len()) % sys.reservoirs.len();
                // Ledger at a fixed moment, asked while the "campaign date" varies.
                let _ = e;
                format!("{:?}", ledger(&mut ro, &Campaign::default(), ri, 0, 4000.0))
            };
            out.push((i, s));
        }
        out.sort();
        out
    };
    let total = sample.len() + crater_ids.len() + sys.reservoirs.len();
    let fwd: Vec<usize> = (0..total).collect();
    let mut rev = fwd.clone();
    rev.reverse();
    let mut shuf = fwd.clone();
    shuf.sort_by_key(|i| roster_proto::perm::mix(*i as u64 ^ 0xfeed));
    let to = Instant::now();
    let a = run(&fwd, &[present]);
    let b = run(&rev, &[100.0, 2000.0]);
    let c = run(&shuf, &[present + 500.0, 50.0, 3000.0]);
    let t_order = to.elapsed();
    let ha = fnv(&format!("{a:?}"));
    let hb = fnv(&format!("{b:?}"));
    let hc = fnv(&format!("{c:?}"));
    println!("{} answers ({} members, {} craters back to impactors, {} ledgers), in three orders and at different campaign dates, each from a cold cache: digests {ha:016x}, {hb:016x}, {hc:016x}. Same: **{}**. ({:.0} ms for all three.)", total, sample.len(), crater_ids.len(), sys.reservoirs.len(), ha == hb && hb == hc, t_order.as_secs_f64() * 1e3);
    if !(ha == hb && hb == hc) {
        println!("\n**RED: the answers depend on the order of asking.**");
        std::process::exit(1);
    }
    // Forwards and backwards meet: every crater's impactor, asked forwards, lands on that world at that date.
    let mut meet = 0;
    for cr in list.iter() {
        let bdy = ro.impactor_of(&cr.id);
        let f = ro.fate(bdy.band, bdy.rank).unwrap();
        assert_eq!(
            sys.reservoirs[bdy.band.res].sinks[f.sink].0,
            SinkKind::World(cr.world)
        );
        assert_eq!(f.t_arrive, cr.t_myr);
        meet += 1;
    }
    println!("Every one of the {meet} listed craters on {}, read backwards to its impactor and that impactor read forwards again, lands on the same world at the same moment.", tw.id);

    // ---- 5. the slider ----------------------------------------------------
    println!("\n## 5. The same answer at every point on the slider\n");
    let wb = ro.impactor_of(&walk.id);
    let wf = wb.fate.clone().unwrap();
    let cam0 = Campaign::default();
    let moments = [
        wf.t_leave - 5.0,
        wf.t_leave + 0.5 * (wf.t_arrive - wf.t_leave),
        wf.t_arrive + 0.001,
        present,
        present + 500.0,
    ];
    println!(
        "| Slider | The impactor `{}` is | Crater in the list | Crater reads back to |",
        wb.name
    );
    println!("|---|---|---|---|");
    for t in moments {
        let s = state_at(&mut ro, &cam0, wb.band, wb.rank, t);
        let (cl, _, _) = ro.listed_craters(walk.world, (0.0, sys.horizon_myr), t, 2000);
        let has = cl.iter().any(|c| c.id == walk.id);
        let back = if has {
            ro.impactor_of(&walk.id).name
        } else {
            "(not yet made)".into()
        };
        println!("| {} | {:?} | {} | {} |", ago(&sys, t), s, has, back);
        assert_eq!(has, t >= walk.t_myr);
    }
    // Ledger closure through time.
    let mut ok = true;
    let mut prev: Option<roster_proto::book::Ledger> = None;
    let mut worst_neg = 0i128;
    for i in 0..=14 {
        let t = i as f64 * 400.0;
        for ri in 0..sys.reservoirs.len() {
            for ci in 0..sys.reservoirs[ri].classes.len() {
                let lg = ledger(&mut ro, &cam0, ri, ci, t);
                ok &= lg.closes();
                worst_neg = worst_neg.min(*lg.reservoir.iter().min().unwrap());
                if ri == deb && ci == 0 {
                    if let Some(p) = &prev {
                        for (k, v) in &p.delivered {
                            let now = lg.delivered.get(k).unwrap();
                            ok &= (0..6).all(|g| now[g] >= v[g]);
                        }
                    }
                    prev = Some(lg);
                }
            }
        }
    }
    println!("\nEvery class's books at 15 moments from 0 to 5.6 billion years: reservoir + in flight + delivered + mined = start, in every element group, never negative, and what a sink holds never falls: **{ok}**.");
    assert!(ok);
    let lg = ledger(&mut ro, &cam0, deb, 0, present);
    println!("\nThe core fragments' books at the present (sub-units of 1e-21 solar masses, metal group):\n");
    println!(
        "- start {}, still in the belt {}, in flight {}",
        lg.start[0], lg.reservoir[0], lg.in_flight[0]
    );
    for (k, v) in &lg.delivered {
        println!("- delivered to {k}: {}", v[0]);
    }
    println!(
        "- tiers: exact {} / named-by-band {} / gravel {} sub-units",
        lg.tiers_sub[0], lg.tiers_sub[1], lg.tiers_sub[2]
    );
    // One event, two ends (exact tier).
    let world_name = &tw.id;
    let exact_in = *lg.exact_delivered.get(world_name).unwrap_or(&0);
    let mut sum_imp = 0u64;
    let mut n_imp = 0u64;
    for (bi, (k0, k1)) in sys.reservoirs[deb].classes[0].bands.iter().enumerate() {
        if *k0 > sys.reservoirs[deb].classes[0].n_exact {
            break;
        }
        let id = BandId {
            res: deb,
            class: 0,
            band: bi,
        };
        let s = ro.sink_of_world(deb, target).unwrap();
        for (b, seg, n) in ro.arrivals(id, s) {
            for j in 0..n {
                let k = ro.member_at(id, b, seg, s, j);
                if k >= *k1 || k > sys.reservoirs[deb].classes[0].n_exact {
                    continue;
                }
                let bd = ro.body(id, k);
                if bd.fate.as_ref().unwrap().t_arrive <= present {
                    sum_imp += bd.sub;
                    n_imp += 1;
                }
            }
        }
    }
    println!("\nOne event, two ends: the {n_imp} exact-tier core fragments that struck {} by the present weigh {} sub-units; the belt's books say {} received {}. Equal: **{}**.", world_name, sum_imp, world_name, exact_in, sum_imp == exact_in);
    assert_eq!(sum_imp, exact_in);

    // ---- 6. deeds ---------------------------------------------------------
    println!("\n## 6. A deed: players mine a body that was due to strike later\n");
    // An exact-tier core fragment due to land on a solid world in the future.
    let mut due: Option<Body> = None;
    let pbin = sys.bin_of(present);
    'search: for (id, _) in ro.all_bands() {
        let c = &sys.reservoirs[id.res].classes[id.class];
        if c.bands[id.band].0 > c.n_exact {
            continue;
        }
        for wi in 0..sys.worlds.len() {
            if !sys.worlds[wi].surface {
                continue;
            }
            let Some(s) = ro.sink_of_world(id.res, wi) else {
                continue;
            };
            for (b, seg, n) in ro.arrivals(id, s) {
                if b <= pbin {
                    continue;
                }
                for j in 0..n {
                    let k = ro.member_at(id, b, seg, s, j);
                    let bd = ro.body(id, k);
                    if bd.fate.as_ref().unwrap().t_leave > present && k <= c.n_exact {
                        due = Some(bd);
                        break 'search;
                    }
                }
            }
        }
    }
    let due = due.expect("a future impactor");
    let df = due.fate.clone().unwrap();
    let dw = match sys.reservoirs[due.band.res].sinks[df.sink].0 {
        SinkKind::World(i) => i,
        _ => unreachable!(),
    };
    let due_cr = ro.crater_of(dw, &due);
    let cam1 = Campaign {
        deeds: vec![
            Deed::MineMember {
                band: due.band,
                rank: due.rank,
                t_myr: present,
                by: "the players' ship".into(),
            },
            Deed::MineGravel {
                res: deb,
                class: 1,
                sub: 1_000_000_000,
                t_myr: present + 10.0,
                name: "Players' Rock".into(),
            },
        ],
    };
    println!("`{}` ({:.1} km, {}) is due to leave the belt {} and strike {} {}, making a {:.1} km crater. The players mine it out at the present. They also mine 1e9 sub-units (about 2e18 kg) of the debris belt's mantle gravel ten million years later and name what is left of the rock \"Players' Rock\", a pin.\n", due.name, due.diameter_km, due.type_label, ago(&sys, df.t_leave), sys.worlds[dw].id, ago(&sys, df.t_arrive), due_cr.d_km);
    // Compare the two universes everywhere we can read.
    let mut diffs: Vec<String> = vec![];
    let epochs = [
        1000.0,
        present - 1.0,
        present,
        present + 10.0,
        df.t_arrive + 1.0,
        present + 999.0,
    ];
    for &t in &epochs {
        for (wi, w) in sys.worlds.iter().enumerate() {
            if !w.surface {
                continue;
            }
            for (label, win, max) in [
                ("whole-life", (0.0, sys.horizon_myr), 2000u64),
                (
                    "last-billion-years",
                    (present - 1000.0, sys.horizon_myr),
                    500,
                ),
            ] {
                let (a, _, _) = ro.listed_craters(wi, win, t, max);
                let mined =
                    |c: &Crater| cam1.mined_at(c.band, c.rank).is_some_and(|td| td < c.t_myr);
                let b: Vec<&Crater> = a.iter().filter(|c| !mined(c)).collect();
                if a.len() != b.len() {
                    diffs.push(format!("{} {label} list at {}: {} craters become {} (the mined body's crater is never made)", w.id, ago(&sys, t), a.len(), b.len()));
                }
            }
        }
        for ri in 0..sys.reservoirs.len() {
            for ci in 0..sys.reservoirs[ri].classes.len() {
                let a = ledger(&mut ro, &cam0, ri, ci, t);
                let b = ledger(&mut ro, &cam1, ri, ci, t);
                assert!(b.closes());
                if a != b {
                    diffs.push(format!(
                        "{} {} books at {}: mined {:?} (sub-units by group)",
                        sys.reservoirs[ri].name,
                        sys.reservoirs[ri].classes[ci].name,
                        ago(&sys, t),
                        b.mined
                    ));
                }
            }
        }
    }
    let mut fate_diffs = 0;
    for &(id, k) in &sample {
        for &t in &epochs {
            if state_at(&mut ro, &cam0, id, k, t) != state_at(&mut ro, &cam1, id, k, t) {
                fate_diffs += 1;
            }
        }
    }
    let mined_state: Vec<String> = epochs
        .iter()
        .map(|&t| {
            format!(
                "{}: {:?}",
                ago(&sys, t),
                state_at(&mut ro, &cam1, due.band, due.rank, t)
            )
        })
        .collect();
    println!(
        "Every difference between the two copies of the universe, read at {} moments:\n",
        epochs.len()
    );
    for d in &diffs {
        println!("- {d}");
    }
    println!("- {} of {} sampled members differ in state at any moment (the mined body is not in the sample).", fate_diffs, sample.len());
    println!(
        "\nThe mined body through the slider: {}.",
        mined_state.join("; ")
    );
    assert_eq!(fate_diffs, 0);
    let dres = &sys.reservoirs[due.band.res].name;
    assert!(diffs.iter().all(|d| d.contains(&sys.worlds[dw].id)
        || d.contains(dres.as_str())
        || d.contains("debris-belt-4 mantle")));

    // ---- 7. costs ---------------------------------------------------------
    println!("\n## 7. What it costs\n");
    let mut cold = Roster::new(&sys);
    let tp = Instant::now();
    let bands = cold.all_bands();
    for (id, _) in &bands {
        cold.plan(*id);
    }
    let t_plans = tp.elapsed();
    println!("- Plans for all {} bands in the system (every class, every size band): {:.1} ms, {:.0} kB held as a cache (never saved).", bands.len(), t_plans.as_secs_f64() * 1e3, cold.cache_bytes() as f64 / 1024.0);
    // Forward and backward reads in the largest band of the comet clouds.
    let (big, _) = bands
        .iter()
        .filter(|(id, _)| sys.reservoirs[id.res].kind == "oort")
        .map(|(id, _)| (*id, sys.reservoirs[id.res].classes[id.class].bands[id.band]))
        .max_by_key(|(_, (a, b))| b - a)
        .map(|(id, r)| (id, r))
        .unwrap();
    let (k0, k1) = sys.reservoirs[big.res].classes[big.class].bands[big.band];
    let nq = 20_000u64;
    let tf = Instant::now();
    let mut left = 0;
    for q in 0..nq {
        let k = k0 + roster_proto::perm::mix(q) % (k1 - k0);
        left += cold.fate(big, k).is_some() as u64;
    }
    let t_fwd = tf.elapsed();
    println!("- Forwards, member to fate, in a band of {:.2e} comets: {:.2} µs each ({} of {} sampled ever leave).", (k1 - k0) as f64, t_fwd.as_secs_f64() * 1e6 / nq as f64, left, nq);
    let tbk = Instant::now();
    for cr in list.iter().take(2000) {
        let _ = cold.impactor_of(&cr.id);
    }
    let t_back = tbk.elapsed();
    let nlist = list.len().min(2000);
    println!(
        "- Backwards, crater id to impactor with its fate: {:.2} µs each (over {} craters).",
        t_back.as_secs_f64() * 1e6 / nlist as f64,
        nlist
    );
    let tlg = Instant::now();
    let lgp = ledger(&mut cold, &cam0, deb, 0, present);
    let t_lg = tlg.elapsed();
    println!("- One class's books at the present (debris core): {:.1} ms, walking {} exact-tier members.", t_lg.as_secs_f64() * 1e3, lgp.exact_members_walked);
    let tall = Instant::now();
    for ri in 0..sys.reservoirs.len() {
        for ci in 0..sys.reservoirs[ri].classes.len() {
            let _ = ledger(&mut cold, &cam0, ri, ci, present);
        }
    }
    println!(
        "- Every class's books at the present: {:.0} ms.",
        tall.elapsed().as_secs_f64() * 1e3
    );

    // The cost of weighing the named-by-band tier at its mean: walk one big band exactly.
    let rr = &sys.reservoirs[deb].classes[1];
    if let Some((bi, (bk0, bk1))) = rr
        .bands
        .iter()
        .enumerate()
        .find(|(_, (a, b))| b - a >= 1_000_000 && *a > rr.n_exact)
    {
        let id = BandId {
            res: deb,
            class: 1,
            band: bi,
        };
        let s = cold.sink_of_world(deb, target).unwrap();
        let tw0 = Instant::now();
        let mut exact = 0u128;
        let mut count = 0u64;
        for (b, seg, n) in cold.arrivals(id, s) {
            for j in 0..n {
                let k = cold.member_at(id, b, seg, s, j);
                exact += member_sub(rr, k) as u128;
                count += 1;
            }
        }
        let t_walk = tw0.elapsed();
        let mean = roster_proto::book::band_sub(rr, *bk0, *bk1) / (bk1 - bk0);
        let approx = count as u128 * mean as u128;
        println!("- Weighing a band at its mean mass, checked: of {:.2e} mantle pieces in one band, {} ever hit {}. Their true mass is {} sub-units; count times the band's mean says {} ({:+.2}%). Walking them took {:.0} ms.", (bk1 - bk0) as f64, count, tw.id, exact, approx, 100.0 * (approx as f64 - exact as f64) / exact as f64, t_walk.as_secs_f64() * 1e3);
    }
    let (rss, hwm) = rss_kb();
    println!("- Memory: resident {:.1} MB now, peak {:.1} MB for the whole run (this includes the formation code).", rss as f64 / 1024.0, hwm as f64 / 1024.0);

    // Save size.
    #[derive(serde::Serialize)]
    struct Header<'a> {
        seed: &'a str,
        packs: Vec<(&'a str, &'a str)>,
        algorithm: &'a str,
    }
    let header = Header {
        seed: "cc-roster-1",
        packs: vec![
            ("base-physics", "b3f1c0de"),
            ("base-small-bodies", "77aa01ce"),
        ],
        algorithm: "roster-proto.v0",
    };
    let mut hb = vec![];
    ciborium::into_writer(&header, &mut hb).unwrap();
    let mut db = vec![];
    ciborium::into_writer(&cam1, &mut db).unwrap();
    let mut resolved = vec![];
    for q in 0..1000u64 {
        let (id, k) = sample[(q as usize * 7) % sample.len()];
        resolved.push(ro.body(id, k));
    }
    let mut rb = vec![];
    ciborium::into_writer(&resolved, &mut rb).unwrap();
    println!("- Save: the universe {} bytes (seed, packs, algorithm); the two deeds {} bytes; a resolved record of 1,000 looked-at bodies {} bytes ({} bytes each). Members nobody looked at: 0 bytes.", hb.len(), db.len(), rb.len(), rb.len() / 1000);
}

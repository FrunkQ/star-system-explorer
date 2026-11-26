//! CC1 scratch: measure how far off the joins are (conserved chain design,
//! section 13 item 3). Report only: every test is `#[ignore]` and asserts
//! nothing about the physics; run with
//! `cargo test --release -p sse-kernel cc1_ -- --ignored --nocapture --test-threads=1`.
//! Lines starting `CC1|` are the numbers the report quotes. Not for merging.
#![allow(clippy::disallowed_methods, clippy::disallowed_types, clippy::all)]

use crate::cosmos::with_world;
use sse_core::{Address, Seed};
use sse_cosmos::envelope::Place;
use sse_cosmos::star_tree::TreeWalk;
use sse_cosmos::stars::StarSystem;
use std::time::Instant;

fn seed_root() -> (Seed, Address) {
    // What `open_universe {"id":"g3","seed":"g3"}` gives the kernel.
    (Seed::from_text("g3"), Address::universe("g3").unwrap())
}

fn pct(v: &mut [f64], q: f64) -> f64 {
    v.sort_by(|a, b| a.total_cmp(b));
    if v.is_empty() {
        return f64::NAN;
    }
    v[((v.len() - 1) as f64 * q).round() as usize]
}

fn mean(v: &[f64]) -> f64 {
    v.iter().sum::<f64>() / v.len().max(1) as f64
}

fn sd(v: &[f64]) -> f64 {
    let m = mean(v);
    (v.iter().map(|x| (x - m) * (x - m)).sum::<f64>() / (v.len().max(2) - 1) as f64).sqrt()
}

/// Present mass of a system: primary and companions at their age (the
/// companions age as the primary does).
fn mass_now(st: &sse_cosmos::stars::Stars, s: &StarSystem, t_gyr: f64) -> f64 {
    let age = t_gyr - s.birth_gyr;
    st.params.state(s.mass_msun, age).mass_msun
        + s.companions
            .iter()
            .map(|c| st.params.state(c.mass_msun, age).mass_msun)
            .sum::<f64>()
}

fn mass_born(s: &StarSystem) -> f64 {
    s.mass_msun + s.companions.iter().map(|c| c.mass_msun).sum::<f64>()
}

/// Join "place to stars": drawn mass in each 10 pc cell against the ring's
/// stellar mass in that cell (`expected x mean_star_msun` is exactly the
/// ring's surface density x vertical profile x volume).
#[test]
#[ignore]
fn cc1_ring_to_stars() {
    let (seed, root) = seed_root();
    with_world(&seed, &root, None, |st, _w, now| {
        let frame = st.frame_of(now);
        let t0 = frame as f64 * st.params.frame_myr;
        let t_gyr = t0 / 1000.0;
        let r_sun = st.env.sun_radius_kpc();
        let mean_star = st.env.params.mean_star_msun;
        println!(
            "CC1|setup|r_sun_kpc={r_sun:.3}|now_myr={now:.1}|frame_myr={}|cell_pc={}|mean_star_msun={mean_star}|thin_h={}|thick_h={}|thick_share={}",
            st.params.frame_myr, st.params.cell_pc, st.env.params.thin_height_pc,
            st.env.params.thick_height_pc, st.env.params.thick_share
        );
        let walk = TreeWalk::new(st, frame);
        for f in [0.5, 0.75, 1.0, 1.5] {
            let r = f * r_sun * 1000.0;
            let c0 = st.cell_at(st.from_grid([r, 0.0, 0.0], t0), t0);
            let (mut budget, mut born, mut nowm, mut n, mut mu) = (0.0, 0.0, 0.0, 0u64, 0.0);
            let mut per_cell = vec![];
            let mut per_cell_born = vec![];
            let mut prim = vec![];
            let mut comp_share = (0.0, 0.0);
            let mut dead = 0u64;
            let (nx, nz) = (3i64, 40i64);
            let tic = Instant::now();
            for dx in 0..nx {
                for dy in 0..nx {
                    for dz in -nz..nz {
                        let cell = [c0[0] + dx, c0[1] + dy, c0[2] + dz];
                        let e = st.expected(cell, frame);
                        let b = e * mean_star;
                        let sys = walk.draw(cell, None).unwrap();
                        let mb: f64 = sys.iter().map(mass_born).sum();
                        let mn: f64 = sys.iter().map(|s| mass_now(st, s, t_gyr)).sum();
                        for s in &sys {
                            prim.push(s.mass_msun);
                            comp_share.0 += s.mass_msun;
                            comp_share.1 += mass_born(s) - s.mass_msun;
                            if st.params.state(s.mass_msun, t_gyr - s.birth_gyr).phase
                                != "main sequence"
                                && st.params.state(s.mass_msun, t_gyr - s.birth_gyr).phase
                                    != "giant"
                            {
                                dead += 1;
                            }
                        }
                        budget += b;
                        born += mb;
                        nowm += mn;
                        n += sys.len() as u64;
                        mu += e;
                        if dz.abs() <= 3 {
                            per_cell.push(mn / b);
                            per_cell_born.push(mb / b);
                        }
                    }
                }
            }
            let ms = tic.elapsed().as_secs_f64() * 1000.0;
            let cells = (nx * nx * 2 * nz) as f64;
            let mut pc = per_cell.clone();
            let mut pcb = per_cell_born.clone();
            println!(
                "CC1|ring_to_stars|r_over_rsun={f}|r_kpc={:.2}|cells={cells}|systems={n}|expected={mu:.0}|ring_budget_msun={budget:.0}|drawn_now_msun={nowm:.0}|drawn_born_msun={born:.0}|now_over_budget={:.3}|born_over_budget={:.3}|mean_primary={:.3}|companion_mass_share={:.3}|remnant_share_by_number={:.3}|midplane_cells={}|cell_now_ratio_p10={:.3}|p50={:.3}|p90={:.3}|cell_born_ratio_p10={:.3}|p50={:.3}|p90={:.3}|cell_now_ratio_sd={:.3}|draw_ms={ms:.0}",
                r / 1000.0,
                nowm / budget,
                born / budget,
                mean(&prim),
                comp_share.1 / (comp_share.0 + comp_share.1),
                dead as f64 / n as f64,
                per_cell.len(),
                pct(&mut pc, 0.1),
                pct(&mut pc, 0.5),
                pct(&mut pc, 0.9),
                pct(&mut pcb, 0.1),
                pct(&mut pcb, 0.5),
                pct(&mut pcb, 0.9),
                sd(&per_cell),
            );
        }
        Ok(())
    })
    .unwrap();
}

/// Join "ring to a place": the arm and texture factors on the gas, averaged
/// round the ring (phi) and through its height, against 1 (a reshuffle).
#[test]
#[ignore]
fn cc1_arm_and_texture() {
    let (seed, root) = seed_root();
    with_world(&seed, &root, None, |st, _w, now| {
        let env = st.env;
        let t_gyr = now / 1000.0;
        let r_sun = env.sun_radius_kpc();
        let h = env.params.gas_height_pc;
        for f in [0.5, 1.0, 1.5] {
            let r = f * r_sun;
            let (mut a, mut t, mut at, mut w) = (0.0, 0.0, 0.0, 0.0);
            let mut each = vec![];
            let mut armv = vec![];
            let nphi = 2000;
            for k in 0..nphi {
                let phi = 2.0 * std::f64::consts::PI * (k as f64 + 0.5) / nphi as f64;
                for zi in -20..=20 {
                    let z = zi as f64 * 25.0;
                    let p = Place {
                        r_kpc: r,
                        phi,
                        z_pc: z,
                        t_gyr,
                    };
                    let wz = (-z.abs() / h).exp();
                    let ar = env.arm(&p);
                    let tx = env.texture(&p);
                    a += wz * ar;
                    t += wz * tx;
                    at += wz * ar * tx;
                    w += wz;
                    if zi == 0 {
                        each.push(ar * tx);
                        armv.push(ar);
                    }
                }
            }
            let mut e2 = each.clone();
            println!(
                "CC1|arm_texture|r_over_rsun={f}|ring_mean_arm={:.4}|ring_mean_texture={:.4}|ring_mean_arm_x_texture={:.4}|midplane_factor_min={:.3}|p10={:.3}|p50={:.3}|p90={:.3}|max={:.3}|arm_min={:.3}|arm_max={:.3}",
                a / w,
                t / w,
                at / w,
                pct(&mut e2, 0.0),
                pct(&mut e2, 0.1),
                pct(&mut e2, 0.5),
                pct(&mut e2, 0.9),
                pct(&mut e2, 1.0),
                armv.iter().cloned().fold(f64::INFINITY, f64::min),
                armv.iter().cloned().fold(0.0, f64::max),
            );
        }
        // How the timeline runs: where the rings stop against the present.
        let d = env.disc;
        let steps = d.states[0].len();
        println!(
            "CC1|timeline|ring_t_start_gyr={:.3}|ring_step_gyr={}|ring_steps={steps}|ring_t_end_gyr={:.3}|galaxy_now_gyr={:.3}|rings={}",
            d.t_start_gyr,
            d.step_gyr,
            d.t_start_gyr + d.step_gyr * steps as f64,
            env.galaxy.now.t_gyr,
            d.rings.len()
        );
        // The gas after the end: does it coast or hold?
        let pnow = Place { r_kpc: r_sun, phi: 0.0, z_pc: 0.0, t_gyr };
        let plate = Place { t_gyr: t_gyr + 5.0, ..pnow };
        println!(
            "CC1|timeline|gas_pc3_now={:.5}|gas_pc3_plus5gyr={:.5}|surface_stars_now={:.3}|surface_stars_plus5gyr={:.3}",
            env.gas_pc3(pnow).0,
            env.gas_pc3(plate).0,
            env.surface(r_sun, t_gyr).stars,
            env.surface(r_sun, t_gyr + 5.0).stars
        );
        Ok(())
    })
    .unwrap();
}

/// Join "star to system": the disc each drawn star would get, against the
/// star and its birth cluster; and the recipe the fixed stand-in envelope
/// gives against the drawn stars at three radii.
#[test]
#[ignore]
fn cc1_star_to_system() {
    let (seed, root) = seed_root();
    with_world(&seed, &root, None, |st, _w, now| {
        let frame = st.frame_of(now);
        let t0 = frame as f64 * st.params.frame_myr;
        let t_gyr = t0 / 1000.0;
        let r_sun = st.env.sun_radius_kpc();
        let walk = TreeWalk::new(st, frame);
        let fseed = Seed::from_text("cc1");
        for f in [0.5, 1.0, 1.5] {
            let r = f * r_sun * 1000.0;
            let c0 = st.cell_at(st.from_grid([r, 0.0, 0.0], t0), t0);
            let mut sys = vec![];
            for dx in 0..4 {
                for dz in -20..20 {
                    sys.extend(walk.draw([c0[0] + dx, c0[1], c0[2] + dz], None).unwrap());
                }
            }
            let (mut sd_, mut ss, mut sb) = (0.0, 0.0, 0.0);
            let mut fr = vec![];
            let mut cl = vec![];
            let mut feh = vec![];
            let mut age = vec![];
            for s in &sys {
                let b = st.birth(s);
                let dp = sse_formation::disc::disc_params(
                    &fseed,
                    &s.address,
                    s.mass_msun,
                    1.0,
                    b.feh,
                    b.alpha_fe,
                    b.c_o,
                    b.cluster_fuv_g0,
                    None,
                );
                sd_ += dp.mass0;
                ss += s.mass_msun;
                sb += mass_born(s);
                fr.push(dp.mass0 / s.mass_msun);
                cl.push(dp.mass0 / s.cluster_mass_msun);
                feh.push(s.feh);
                age.push(t_gyr - s.birth_gyr);
            }
            let mut fr2 = fr.clone();
            let mut cl2 = cl.clone();
            let mut feh2 = feh.clone();
            let mut age2 = age.clone();
            println!(
                "CC1|star_to_disc|r_over_rsun={f}|systems={}|disc_over_primary_p50={:.4}|mean={:.4}|p90={:.4}|max={:.4}|sum_disc_over_sum_born={:.4}|disc_over_cluster_p50={:.2e}|drawn_feh_mean={:.3}|sd={:.3}|p10={:.3}|p90={:.3}|drawn_age_mean={:.2}|age_p10={:.2}|age_p90={:.2}|standin_feh=-0.05+-0.20|standin_age=U(0.05,10)",
                sys.len(),
                pct(&mut fr2, 0.5),
                mean(&fr),
                pct(&mut fr2, 0.9),
                pct(&mut fr2, 1.0),
                sd_ / sb,
                pct(&mut cl2, 0.5),
                mean(&feh),
                sd(&feh),
                pct(&mut feh2, 0.1),
                pct(&mut feh2, 0.9),
                mean(&age),
                pct(&mut age2, 0.1),
                pct(&mut age2, 0.9),
            );
            let _ = ss;
        }
        Ok(())
    })
    .unwrap();
}

/// Events: supernovae drawn from a rate against the massive stars drawn in
/// the same volume; the passing star's mass against the field's.
#[test]
#[ignore]
fn cc1_events() {
    let (seed, root) = seed_root();
    with_world(&seed, &root, None, |st, w, now| {
        let frame = st.frame_of(now);
        let t0 = frame as f64 * st.params.frame_myr;
        let t_gyr = t0 / 1000.0;
        let ev = sse_cosmos::events::Events {
            env: st.env,
            halo: w.host,
            edits: &[],
            green: st.green.clone(),
            stars: st.params.clone(),
            params: w.cp.events.clone(),
            seed: st.seed,
        };
        let sun = [st.env.sun_radius_kpc() * 1000.0, 0.0, 0.0];
        let walk = TreeWalk::new(st, frame);
        let radius = 150.0;
        let p = &st.params;
        let classes = p.class_edges_lsun.len() + 1;
        let (mut alive8, mut die10, mut die_mass) = (0u64, 0u64, 0.0);
        let tic = Instant::now();
        // Classes from 1e3 Lsun up hold every living star above about 6 Msun.
        let first = p.class_edges_lsun.iter().position(|e| *e >= 1.0e3).unwrap() + 1;
        let window_gyr = ev.params.sn_bin_myr / 1000.0;
        for c in first..classes {
            for (cell, _) in walk.cells_within(c, sun, radius).unwrap() {
                for s in walk.draw(cell, Some(c)).unwrap() {
                    let g = st.from_grid(
                        [
                            (cell[0] as f64 + 0.5) * p.cell_pc,
                            (cell[1] as f64 + 0.5) * p.cell_pc,
                            (cell[2] as f64 + 0.5) * p.cell_pc,
                        ],
                        t0,
                    );
                    let d2 = (g[0] - st.from_grid(sun, t0)[0]).powi(2)
                        + (g[1] - st.from_grid(sun, t0)[1]).powi(2)
                        + g[2].powi(2);
                    if d2 > radius * radius || s.mass_msun < p.ns_min_msun {
                        continue;
                    }
                    alive8 += 1;
                    let end = s.birth_gyr + p.lifetime_gyr(s.mass_msun) * (1.0 + p.giant_share);
                    if end >= t_gyr && end < t_gyr + window_gyr {
                        die10 += 1;
                        die_mass += s.mass_msun;
                    }
                }
            }
        }
        let ms = tic.elapsed().as_secs_f64() * 1000.0;
        // The event layer's expectation in the same sphere and window.
        let c = ev.params.sn_cell_pc;
        let bin = (t0 / ev.params.sn_bin_myr).floor() as i64;
        let vol = 4.0 / 3.0 * std::f64::consts::PI * radius.powi(3);
        let cell = [
            (sun[0] / c).floor() as i64,
            (sun[1] / c).floor() as i64,
            0,
        ];
        let lam = ev.expected_cell(cell, bin);
        let lam_cc = lam[0] * vol / (c * c * c);
        let mut drawn_cc = 0usize;
        let n = (radius / c).ceil() as i64 + 1;
        for dx in -n..=n {
            for dy in -n..=n {
                for dz in -n..=n {
                    let k = [cell[0] + dx, cell[1] + dy, dz];
                    for e in ev.cell_events(k, bin).unwrap() {
                        let dd = (e.grid_pc[0] - sun[0]).powi(2)
                            + (e.grid_pc[1] - sun[1]).powi(2)
                            + e.grid_pc[2].powi(2);
                        if e.kind == sse_cosmos::events::Kind::CoreCollapse
                            && dd <= radius * radius
                        {
                            drawn_cc += 1;
                        }
                    }
                }
            }
        }
        println!(
            "CC1|supernovae|sphere_pc={radius}|window_myr={}|stars_above_8msun_alive={alive8}|of_them_die_in_window={die10}|their_mass={die_mass:.0}|event_layer_expected_cc={lam_cc:.2}|event_layer_drawn_cc={drawn_cc}|same_objects=0 (separate keys)|bright_walk_ms={ms:.0}",
            ev.params.sn_bin_myr
        );
        // Passing stars: initial-mass IMF draws against the field's present masses.
        let k = 200_000;
        let (mut over8, mut sum) = (0u64, 0.0);
        for i in 0..k {
            let u = (i as f64 + 0.5) / k as f64;
            let m = sse_cosmos::stars::imf_quantile(p, u);
            sum += m;
            if m >= p.ns_min_msun {
                over8 += 1;
            }
        }
        println!(
            "CC1|passing_star|mean_mass_drawn={:.3}|share_above_8msun={:.4}|encounter_pc={}|note=compare field present mean in ring_to_stars",
            sum / k as f64,
            over8 as f64 / k as f64,
            st.env.params.encounter_pc
        );
        Ok(())
    })
    .unwrap();
}

/// Indifference: the order and date of looking, and the mark a pin leaves.
#[test]
#[ignore]
fn cc1_indifference() {
    let (seed, root) = seed_root();
    with_world(&seed, &root, None, |st, _w, now| {
        let frame = st.frame_of(now);
        let t0 = frame as f64 * st.params.frame_myr;
        let c0 = st.cell_at(
            st.from_grid([st.env.sun_radius_kpc() * 1000.0, 0.0, 0.0], t0),
            t0,
        );
        let mut cells = vec![];
        for dx in -1..=1 {
            for dy in -1..=1 {
                for dz in -1..=1 {
                    cells.push([c0[0] + dx, c0[1] + dy, c0[2] + dz]);
                }
            }
        }
        let a = st.cells(&cells, frame, &[]).unwrap();
        let mut rev = cells.clone();
        rev.reverse();
        let mut b: Vec<Vec<StarSystem>> = rev.iter().map(|c| st.cell(*c, frame, &[]).unwrap()).collect();
        b.reverse();
        let same_order = serde_json::to_string(&a).unwrap() == serde_json::to_string(&b).unwrap();
        // A later frame is a different draw (the deep-time rule), by design.
        let later = st.cells(&cells, frame + 1, &[]).unwrap();
        let n_a: usize = a.iter().map(|v| v.len()).sum();
        let same_addresses = a
            .iter()
            .zip(&later)
            .map(|(x, y)| x.iter().filter(|s| y.iter().any(|q| q.address == s.address && q.mass_msun == s.mass_msun)).count())
            .sum::<usize>();
        println!(
            "CC1|indifference|cells=27|systems={n_a}|two_orders_identical={same_order}|systems_identical_next_frame={same_addresses}|frame_myr={}",
            st.params.frame_myr
        );
        // Promote every star of the Sun's cell, then look again after the
        // pins have drifted: what the fill gives up for them.
        let pins: Vec<StarSystem> = st
            .cell(c0, frame, &[])
            .unwrap()
            .into_iter()
            .map(|s| st.promote(&s))
            .collect();
        let pin_mass: f64 = pins.iter().map(mass_born).sum();
        for k in [10i64, 100, 1000, 10000] {
            let f2 = frame + k;
            let t2 = f2 as f64 * st.params.frame_myr;
            let mut where_: Vec<[i64; 3]> = pins.iter().map(|s| st.cell_at(s.orbit.xyz(t2), t2)).collect();
            where_.sort();
            where_.dedup();
            let with = st.cells(&where_, f2, &pins).unwrap();
            let without = st.cells(&where_, f2, &[]).unwrap();
            let (mut lost_n, mut lost_m, mut extra) = (0usize, 0.0, 0.0);
            for (w, o) in with.iter().zip(&without) {
                for s in o {
                    if !w.iter().any(|q| q.address == s.address) {
                        lost_n += 1;
                        lost_m += mass_born(s);
                    }
                }
                extra += w.iter().filter(|s| s.pinned).map(mass_born).sum::<f64>();
            }
            println!(
                "CC1|pin_mark|after_myr={:.1}|pins={}|pin_mass={pin_mass:.1}|cells_they_are_in={}|fill_given_up={lost_n}|fill_mass_given_up={lost_m:.1}|pin_mass_there={extra:.1}|net_mass_change={:.1}",
                k as f64 * st.params.frame_myr,
                pins.len(),
                where_.len(),
                extra - lost_m
            );
        }
        Ok(())
    })
    .unwrap();
}

fn ask(k: &mut crate::Kernel, req: serde_json::Value) -> (serde_json::Value, f64, usize) {
    let tic = Instant::now();
    let out = k.handle_json(&req.to_string());
    let ms = tic.elapsed().as_secs_f64() * 1000.0;
    (serde_json::from_str(&out).unwrap(), ms, out.len())
}

/// Save sizes and run-time costs through the kernel's own ops.
#[test]
#[ignore]
fn cc1_saves_and_costs() {
    let dir = std::env::var("CC1_OUT").unwrap_or_else(|_| "/tmp/cc1".into());
    std::fs::create_dir_all(&dir).unwrap();
    let mut k = crate::Kernel::new();
    let (r, ms, _) = ask(
        &mut k,
        serde_json::json!({"op":"open_universe","id":"g3","seed":"g3"}),
    );
    assert_eq!(r["type"], "universe_opened", "{r}");
    println!("CC1|cost|open_universe_ms={ms:.1}");
    let export = |k: &mut crate::Kernel, name: &str| -> usize {
        let (r, ms, n) = ask(k, serde_json::json!({"op":"export_chronicle"}));
        assert_eq!(
            r["type"],
            "chronicle_exported",
            "{}",
            r.to_string().chars().take(300).collect::<String>()
        );
        let body = r["export"].to_string();
        std::fs::write(format!("{dir}/{name}.json"), &body).unwrap();
        println!(
            "CC1|save|{name}|export_bytes={}|response_bytes={n}|export_ms={ms:.1}",
            body.len()
        );
        body.len()
    };
    let b0 = export(&mut k, "s0-empty");
    let (sky, ms, n) = ask(
        &mut k,
        serde_json::json!({"op":"sky_from_here","limit_mag":6.5,"galaxies":false}),
    );
    let stars: Vec<String> = sky["sky"]["stars"]
        .as_array()
        .map(|a| {
            a.iter()
                .filter_map(|s| s["address"].as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();
    println!(
        "CC1|cost|sky_from_here_6.5mag_cold_ms={ms:.0}|stars={}|bytes={n}",
        stars.len()
    );
    let (_, ms, _) = ask(
        &mut k,
        serde_json::json!({"op":"sky_from_here","limit_mag":6.5,"galaxies":false}),
    );
    println!("CC1|cost|sky_from_here_6.5mag_warm_ms={ms:.0}");
    let (c, ms, n) = ask(&mut k, serde_json::json!({"op":"star_cell"}));
    println!(
        "CC1|cost|star_cell_sun_ms={ms:.1}|systems={}|bytes={n}",
        c["systems"].as_array().map_or(0, |a| a.len())
    );
    let (_, ms, _) = ask(&mut k, serde_json::json!({"op":"star_cell"}));
    println!("CC1|cost|star_cell_sun_warm_ms={ms:.1}");
    // Promote 1, then 9 more, then 90 more.
    let mut done = 0;
    let mut last = b0;
    let mut promote_ms = vec![];
    for (upto, name) in [
        (1, "s1-one-star"),
        (10, "s2-ten-stars"),
        (100, "s3-hundred-stars"),
    ] {
        while done < upto && done < stars.len() {
            let (r, ms, _) = ask(
                &mut k,
                serde_json::json!({"op":"promote_star","address":stars[done]}),
            );
            assert!(r["type"].is_string(), "{r}");
            promote_ms.push(ms);
            done += 1;
        }
        let b = export(&mut k, name);
        println!(
            "CC1|save|{name}|per_star_bytes={:.0}",
            (b - last) as f64
                / (if upto == 1 {
                    1
                } else if upto == 10 {
                    9
                } else {
                    90
                }) as f64
        );
        last = b;
    }
    println!("CC1|cost|promote_star_ms_mean={:.2}", mean(&promote_ms));
    // A system grown from a drawn star.
    let star = stars[done].clone();
    let (r, ms, n) = ask(
        &mut k,
        serde_json::json!({"op":"run_formation","seed":star,"from_star":star}),
    );
    println!(
        "CC1|cost|run_formation_from_star_ms={ms:.0}|bytes={n}|type={}",
        r["type"]
    );
    let (r, ms, _) = ask(
        &mut k,
        serde_json::json!({"op":"adopt","formation":{"seed":star,"from_star":star}}),
    );
    println!("CC1|cost|adopt_from_star_ms={ms:.0}|type={}", r["type"]);
    let b = export(&mut k, "s4-grown-system");
    println!("CC1|save|s4-grown-system|delta_bytes={}", b - last);
    last = b;
    // A system from Make a system (the stand-in envelope).
    let (r, ms, n) = ask(
        &mut k,
        serde_json::json!({"op":"run_formation","seed":"first light","system":"u:g3/sys:@first-light","star_pins":{"primary_mass":1.0}}),
    );
    println!(
        "CC1|cost|run_formation_standin_ms={ms:.0}|bytes={n}|type={}",
        r["type"]
    );
    let (r, _, _) = ask(
        &mut k,
        serde_json::json!({"op":"adopt","formation":{"seed":"first light","system":"u:g3/sys:@first-light","star_pins":{"primary_mass":1.0}}}),
    );
    println!("CC1|save|adopt_standin_type={}", r["type"]);
    let b = export(&mut k, "s5-made-system");
    println!("CC1|save|s5-made-system|delta_bytes={}", b - last);
    last = b;
    // A V3 import (the repo's synthetic Sol fixture).
    let v3 = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/fixtures/v3/synthetic-sol.json"
    ))
    .unwrap();
    let (r, ms, _) = ask(&mut k, serde_json::json!({"op":"adopt","import_v3":v3}));
    println!(
        "CC1|cost|adopt_v3_ms={ms:.0}|type={}|v3_bytes={}",
        r["type"],
        v3.len()
    );
    let b = export(&mut k, "s6-v3-import");
    println!("CC1|save|s6-v3-import|delta_bytes={}", b - last);
}

/// Run-time cost of the walks the design needs, natively.
#[test]
#[ignore]
fn cc1_lookup_costs() {
    let (seed, root) = seed_root();
    let tic = Instant::now();
    with_world(&seed, &root, None, |st, _w, now| {
        println!(
            "CC1|cost|with_world_cold_ms={:.0}",
            tic.elapsed().as_secs_f64() * 1000.0
        );
        let frame = st.frame_of(now);
        let t0 = frame as f64 * st.params.frame_myr;
        let sun = st.from_grid([st.env.sun_radius_kpc() * 1000.0, 0.0, 0.0], t0);
        let c0 = st.cell_at(sun, t0);
        let g = [st.env.sun_radius_kpc() * 1000.0, 0.0, 0.0];
        let classes = st.params.class_edges_lsun.len() + 1;
        // One cell, cold walk: the counts only (what a mass split would also walk).
        let tic = Instant::now();
        let w = TreeWalk::new(st, frame);
        let n: u64 = (0..classes).map(|c| w.cell_count(c0, c).unwrap()).sum();
        println!(
            "CC1|cost|cell_counts_cold_walk_ms={:.2}|systems={n}",
            tic.elapsed().as_secs_f64() * 1000.0
        );
        let tic = Instant::now();
        let _ = w.draw(c0, None).unwrap();
        println!(
            "CC1|cost|cell_draw_after_counts_ms={:.2}",
            tic.elapsed().as_secs_f64() * 1000.0
        );
        let tic = Instant::now();
        let _ = st.cell(c0, frame, &[]).unwrap();
        println!(
            "CC1|cost|cell_draw_fresh_walk_ms={:.2}",
            tic.elapsed().as_secs_f64() * 1000.0
        );
        // A neighbourhood 10 pc round home, every class, drawn whole.
        for radius in [10.0, 20.0, 50.0] {
            let tic = Instant::now();
            let w = TreeWalk::new(st, frame);
            let mut cells = vec![];
            for c in 0..classes {
                for (cell, _) in w.cells_within(c, g, radius).unwrap() {
                    cells.push(cell);
                }
            }
            cells.sort();
            cells.dedup();
            let mut n = 0usize;
            for cell in &cells {
                n += w.draw(*cell, None).unwrap().len();
            }
            println!(
                "CC1|cost|neighbourhood_r{radius}pc_ms={:.1}|cells={}|systems={n}",
                tic.elapsed().as_secs_f64() * 1000.0,
                cells.len()
            );
        }
        // The arm normalising factor: one ring's mean of arm x texture.
        let tic = Instant::now();
        let mut s = 0.0;
        for k in 0..2000 {
            let phi = 2.0 * std::f64::consts::PI * k as f64 / 2000.0;
            let p = Place {
                r_kpc: st.env.sun_radius_kpc(),
                phi,
                z_pc: 0.0,
                t_gyr: t0 / 1000.0,
            };
            s += st.env.arm(&p) * st.env.texture(&p);
        }
        println!(
            "CC1|cost|ring_norm_2000_samples_ms={:.2}|mean={:.4}",
            tic.elapsed().as_secs_f64() * 1000.0,
            s / 2000.0
        );
        Ok(())
    })
    .unwrap();
}

/// Supernovae, column for column: the ring's core-collapse rate against the
/// deaths of the massive stars the star field draws in the same column.
#[test]
#[ignore]
fn cc1_supernovae_column() {
    let (seed, root) = seed_root();
    with_world(&seed, &root, None, |st, _w, now| {
        let frame = st.frame_of(now);
        let t0 = frame as f64 * st.params.frame_myr;
        let t_gyr = t0 / 1000.0;
        let p = &st.params;
        let classes = p.class_edges_lsun.len() + 1;
        let first = p.class_edges_lsun.iter().position(|e| *e >= 1.0e3).unwrap() + 1;
        let walk = TreeWalk::new(st, frame);
        let r_sun = st.env.sun_radius_kpc();
        for f in [0.5, 1.0] {
            let c0 = st.cell_at(st.from_grid([f * r_sun * 1000.0, 0.0, 0.0], t0), t0);
            let side = 40i64;
            let (mut alive, mut die40, mut sfr_mass_young) = (0u64, 0u64, 0.0);
            let mut ages = vec![];
            for dx in 0..side {
                for dy in 0..side {
                    for dz in -60..60 {
                        let cell = [c0[0] + dx, c0[1] + dy, c0[2] + dz];
                        for c in first..classes {
                            if walk.cell_count(cell, c).unwrap() == 0 {
                                continue;
                            }
                            for s in walk.draw(cell, Some(c)).unwrap() {
                                if s.mass_msun < p.ns_min_msun {
                                    continue;
                                }
                                let end = s.birth_gyr
                                    + p.lifetime_gyr(s.mass_msun) * (1.0 + p.giant_share);
                                if end < t_gyr {
                                    continue;
                                }
                                alive += 1;
                                ages.push((t_gyr - s.birth_gyr) * 1000.0);
                                sfr_mass_young += s.mass_msun;
                                if end < t_gyr + 0.04 {
                                    die40 += 1;
                                }
                            }
                        }
                    }
                }
            }
            let area = (side as f64 * p.cell_pc).powi(2);
            let s = st.env.surface(f * r_sun, t_gyr);
            let ring_cc_10myr = s.cc_rate * area * 1.0e7;
            let ring_sfr_10myr = s.sfr * area * 1.0e7;
            let mut a2 = ages.clone();
            println!(
                "CC1|sn_column|r_over_rsun={f}|area_pc2={area}|z_range_pc=+-600|alive_above_8msun={alive}|die_next_40myr={die40}|drawn_deaths_per_10myr={:.2}|ring_cc_per_10myr={ring_cc_10myr:.2}|ratio_ring_over_drawn={:.1}|ring_sfr_msun_per_10myr={ring_sfr_10myr:.1}|cc_per_msun_formed={:.4}|alive_age_p50_myr={:.1}|alive_age_max_myr={:.1}",
                die40 as f64 / 4.0,
                ring_cc_10myr / (die40 as f64 / 4.0).max(1e-9),
                s.cc_rate / s.sfr.max(1e-30),
                pct(&mut a2, 0.5),
                pct(&mut a2, 1.0),
            );
        }
        // The births the star field uses: its last steps.
        let (cdf, acc) = {
            let rc = r_sun;
            let steps = 200;
            let t_first = st.env.disc.t_start_gyr;
            let mut acc = 0.0;
            let mut cdf = vec![(t_first, 0.0)];
            for k in 1..=steps {
                let t = t_first + (t_gyr - t_first) * k as f64 / steps as f64;
                acc += st.env.surface(rc, t).sfr.max(0.0);
                cdf.push((t, acc));
            }
            (cdf, acc)
        };
        let last = cdf[cdf.len() - 1].1 - cdf[cdf.len() - 2].1;
        println!(
            "CC1|birth_cdf|steps=200|step_myr={:.1}|share_of_births_in_last_step={:.4}|expected_if_flat={:.4}",
            (cdf[1].0 - cdf[0].0) * 1000.0,
            last / acc,
            1.0 / 200.0
        );
        Ok(())
    })
    .unwrap();
}

/// What the field's own rules say the column should hold above 8 Msun.
#[test]
#[ignore]
fn cc1_supernovae_expectation() {
    let (seed, root) = seed_root();
    with_world(&seed, &root, None, |st, _w, now| {
        let frame = st.frame_of(now);
        let t0 = frame as f64 * st.params.frame_myr;
        let t_gyr = t0 / 1000.0;
        let p = &st.params;
        let r_sun = st.env.sun_radius_kpc();
        let c0 = st.cell_at(st.from_grid([r_sun * 1000.0, 0.0, 0.0], t0), t0);
        let mut n_exp = 0.0;
        for dx in 0..40 {
            for dy in 0..40 {
                for dz in -60..60 {
                    n_exp += st.expected([c0[0] + dx, c0[1] + dy, c0[2] + dz], frame);
                }
            }
        }
        // Flat births over the disc's life (the birth history's last step
        // holds 1/200 of births, as flat would).
        let span = t_gyr - st.env.disc.t_start_gyr;
        let k = 2_000_000;
        let (mut f_alive, mut f_die10, mut p8) = (0.0, 0.0, 0.0);
        for i in 0..k {
            let u = (i as f64 + 0.5) / k as f64;
            let m = sse_cosmos::stars::imf_quantile(p, u);
            if m < p.ns_min_msun {
                continue;
            }
            p8 += 1.0 / k as f64;
            let life = p.lifetime_gyr(m) * (1.0 + p.giant_share);
            f_alive += life.min(span) / span / k as f64;
            f_die10 += 0.01 / span / k as f64;
        }
        let walk = TreeWalk::new(st, frame);
        let mix = walk.class_mix(walk.node_of(c0, 0).1);
        let first = p.class_edges_lsun.iter().position(|e| *e >= 1.0e3).unwrap() + 1;
        let bright: f64 = mix.share[first..].iter().sum();
        println!(
            "CC1|sn_expect|column_expected_systems={n_exp:.0}|share_primaries_above_8={p8:.5}|expected_alive_above_8={:.1}|expected_primary_deaths_per_10myr={:.2}|tree_share_classes_ge_1e3_lsun={bright:.3e}|tree_expected_bright={:.1}",
            n_exp * f_alive,
            n_exp * f_die10,
            n_exp * bright
        );
        Ok(())
    })
    .unwrap();
}

/// Join "belt to its members": mass closes by rank, makeup is a scatter
/// round the mean. How far the members' metal strays from the belt's.
#[test]
#[ignore]
fn cc1_belt_makeup() {
    use sse_formation::leftovers::{Cloud, Spread, Tilt, ZETA_1_2};
    let c = Cloud {
        address: Address::universe("t").unwrap().child("cloud", 0).unwrap(),
        kind: "asteroid-belt".into(),
        around: None,
        source: "belt-0".into(),
        a_min_au: 2.1,
        a_max_au: 3.3,
        a_log: false,
        e: Spread::Rayleigh { sigma: 0.12 },
        i: Tilt::Rayleigh { sigma_deg: 8.0 },
        solid: [2_860, 5_550, 1_590, 0, 0, 0],
        water_from: vec![],
        largest_km: 900.0,
        over_1km: 1.2e6,
        n_top: 1.0,
        norm: ZETA_1_2,
    };
    let host = sse_small::Host {
        luminosity_lsun: 1.0,
        mass_msun: 1.0,
        age_myr: 4_570.0,
    };
    let s = sse_small::Settings::default();
    let seed = Seed::from_text(&std::env::var("CC1_SEED").unwrap_or_else(|_| "cc1".into()));
    let belt_kg = c.mass_earth() * sse_core::sheet::M_EARTH;
    let tot = (c.solid[0] + c.solid[1] + c.solid[2]) as f64;
    let mean_metal = c.solid[0] as f64 / tot;
    let tic = Instant::now();
    let (mut mass, mut metal) = (0.0, 0.0);
    let mut shares = vec![];
    for (n_top, label) in [
        (10u64, "top10"),
        (100, "top100"),
        (1000, "top1000"),
        (10000, "top10000"),
    ] {
        let start = shares.len() as u64 + 1;
        for k in start..=n_top {
            let m = sse_small::resolve(&seed, &c, k, &host, None, None, &s);
            let t = m.makeup.total().max(1e-30);
            mass += m.mass_kg;
            metal += m.mass_kg * m.makeup.metal / t;
            shares.push(m.makeup.metal / t);
        }
        let mut sh = shares.clone();
        println!(
            "CC1|belt|{label}|share_of_belt_mass={:.4}|members_metal_share={:.4}|belt_metal_share={mean_metal:.4}|ratio={:.3}|member_metal_p10={:.3}|p50={:.3}|p90={:.3}|ms={:.0}",
            mass / belt_kg,
            metal / mass,
            metal / mass / mean_metal,
            pct(&mut sh, 0.1),
            pct(&mut sh, 0.5),
            pct(&mut sh, 0.9),
            tic.elapsed().as_secs_f64() * 1000.0
        );
    }
}

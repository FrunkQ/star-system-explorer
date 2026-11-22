//! G5 checks (galaxy design sections 8 and 12).

use crate::envelope::{EnvelopeParams, GalaxyEnvelope, GreenParams};
use crate::g3_tests::{milky_way_like, world};
use crate::stars::*;
use sse_core::{Address, Seed};

/// Systems within `radius` of a point given in grid coordinates.
fn census(
    st: &Stars,
    centre: [f64; 3],
    radius: f64,
    frame: i64,
    pins: &[StarSystem],
) -> Vec<StarSystem> {
    let c = st.params.cell_pc;
    let lo = |k: usize| sse_core::math::floor((centre[k] - radius) / c) as i64;
    let hi = |k: usize| sse_core::math::floor((centre[k] + radius) / c) as i64;
    let t0 = frame as f64 * st.params.frame_myr;
    let centre = st.from_grid(centre, t0);
    let mut out = vec![];
    for ix in lo(0)..=hi(0) {
        for iy in lo(1)..=hi(1) {
            for iz in lo(2)..=hi(2) {
                for s in st.cell([ix, iy, iz], frame, pins).unwrap() {
                    let p = s.orbit.xyz(t0);
                    let d2 = (0..3)
                        .map(|k| (p[k] - centre[k]) * (p[k] - centre[k]))
                        .sum::<f64>();
                    if d2 <= radius * radius {
                        out.push(s);
                    }
                }
            }
        }
    }
    out
}

macro_rules! setup {
    ($st:ident, $now:ident, $sun:ident) => {
        let (c, w) = world();
        let (g, d) = milky_way_like(&c, &w);
        let env = GalaxyEnvelope {
            galaxy: &g,
            disc: &d,
            params: EnvelopeParams::default(),
            seed: Seed::from_text("g4"),
            address: Address::universe("g4").unwrap(),
        };
        let $st = Stars {
            env: &env,
            green: GreenParams::default(),
            params: StarParams::default(),
            seed: Seed::from_text("g5"),
            galaxy: Address::universe("g5").unwrap(),
        };
        let $now = $st.frame_of(g.now.t_gyr * 1000.0);
        let $sun = [env.sun_radius_kpc() * 1000.0, 0.0, 0.0];
    };
}

#[test]
fn the_suns_neighbourhood_against_observation() {
    setup!(st, now, sun);
    let t = now as f64 * st.params.frame_myr;
    let all = census(&st, sun, 10.0, now, &[]);
    let states: Vec<StarState> = all.iter().map(|s| st.at(s, t).state).collect();
    let n = all.len() as f64;
    let born: Vec<&StarState> = states.iter().filter(|s| s.phase != "unborn").collect();
    let ms: Vec<&&StarState> = born.iter().filter(|s| s.phase == "main sequence").collect();
    let m_dwarfs = ms.iter().filter(|s| s.mass_msun < 0.6).count() as f64 / ms.len() as f64;
    let wd = born.iter().filter(|s| s.phase == "white dwarf").count() as f64 / born.len() as f64;
    let multiple = all.iter().filter(|s| !s.companions.is_empty()).count() as f64 / n;
    let mean_age = born.iter().map(|s| s.age_gyr).sum::<f64>() / born.len() as f64;
    let g_type = ms
        .iter()
        .filter(|s| (0.8..1.04).contains(&s.mass_msun))
        .count();
    println!(
        "within 10 pc: {n} systems, M dwarfs {m_dwarfs:.3} of main sequence, white dwarfs {wd:.3}, multiple {multiple:.3}, mean age {mean_age:.2} Gyr, G dwarfs {g_type}"
    );
    // RECONS 10 pc census (Henry et al. 2018): 317 systems; ours from G4's 0.099 per pc^3.
    assert!((317.0 / 1.6..317.0 * 1.6).contains(&n), "{n}");
    // About 75% of nearby stars are M dwarfs (Henry et al. 2018); a Kroupa IMF
    // from 0.08 solar masses gives more (logged); white dwarfs about 6% of systems.
    assert!((0.65..0.9).contains(&m_dwarfs), "{m_dwarfs}");
    assert!((0.03..0.12).contains(&wd), "{wd}");
    // Multiplicity of nearby systems about 0.3 (dominated by M dwarfs).
    assert!((0.2..0.45).contains(&multiple), "{multiple}");
    assert!((3.0..8.0).contains(&mean_age), "{mean_age}");
}

#[test]
fn older_stars_wander_further_from_the_plane() {
    setup!(st, now, sun);
    let all = census(&st, sun, 20.0, now, &[]);
    let t = now as f64 * st.params.frame_myr;
    let sz = |lo: f64, hi: f64| {
        let v: Vec<f64> = all
            .iter()
            .filter(|s| (lo..hi).contains(&(t / 1000.0 - s.birth_gyr)))
            .map(|s| s.orbit.amp_z_pc)
            .collect();
        v.iter().sum::<f64>() / v.len() as f64
    };
    let (young, old) = (sz(0.0, 2.0), sz(8.0, 14.0));
    println!("mean vertical amplitude: young {young:.0} pc, old {old:.0} pc");
    assert!(old > 1.5 * young);
}

#[test]
fn order_independent_and_deterministic() {
    setup!(st, now, sun);
    let base = st.cell_at(st.from_grid(sun, now as f64), now as f64);
    let cells: Vec<[i64; 3]> = (0..4).map(|k| [base[0] + k, base[1], base[2]]).collect();
    let fwd: Vec<Vec<StarSystem>> = cells
        .iter()
        .map(|c| st.cell(*c, now, &[]).unwrap())
        .collect();
    // Another frame first, then the cells backwards.
    let _ = st.cell(base, now + 3, &[]).unwrap();
    let mut back: Vec<Vec<StarSystem>> = cells
        .iter()
        .rev()
        .map(|c| st.cell(*c, now, &[]).unwrap())
        .collect();
    back.reverse();
    assert_eq!(fwd, back);
    assert_ne!(
        st.cell(base, now, &[]).unwrap(),
        st.cell(base, now + 1, &[]).unwrap(),
        "fill is drawn afresh each frame"
    );
}

#[test]
fn a_promoted_system_is_the_same_at_every_time() {
    setup!(st, now, sun);
    let base = st.cell_at(st.from_grid(sun, now as f64), now as f64);
    let fill = st.cell(base, now, &[]).unwrap();
    let pin = st.promote(&fill[fill.len() / 2]);
    // Its own frame shows the same system, once.
    let again = st.cell(base, now, std::slice::from_ref(&pin)).unwrap();
    assert_eq!(again.len(), fill.len());
    assert_eq!(again.iter().filter(|s| s.address == pin.address).count(), 1);
    // 20 times over 4 Gyr, read in a keyed random order and in order: bit-identical.
    let d = sse_core::Draws::new(
        Seed::from_text("order"),
        pin.address.clone(),
        sse_core::Stream::new("test.order.v1").unwrap(),
    );
    let times: Vec<f64> = (0..20)
        .map(|k| now as f64 - 4000.0 + 200.0 * k as f64 + d.unit(k))
        .collect();
    let mut order: Vec<usize> = (0..20).collect();
    for k in (1..20).rev() {
        order.swap(k, d.below(100 + k as u64, k as u64 + 1) as usize);
    }
    let sorted: Vec<SystemAt> = times.iter().map(|t| st.at(&pin, *t)).collect();
    let mut shuffled: Vec<(usize, SystemAt)> =
        order.iter().map(|&k| (k, st.at(&pin, times[k]))).collect();
    shuffled.sort_by_key(|x| x.0);
    for (k, (_, s)) in shuffled.iter().enumerate() {
        assert_eq!(
            serde_json::to_string(s).unwrap(),
            serde_json::to_string(&sorted[k]).unwrap()
        );
        assert_eq!(
            s.xyz_pc.map(f64::to_bits),
            sorted[k].xyz_pc.map(f64::to_bits)
        );
    }
    // It is at its drawn place at its frame, and it moves on its orbit.
    let p0 = pin.orbit.xyz(now as f64);
    assert_eq!(st.cell_at(p0, now as f64), base);
    let p1 = pin.orbit.xyz(now as f64 + 100.0);
    assert!((0..3).map(|k| (p1[k] - p0[k]).abs()).sum::<f64>() > 100.0);
}

#[test]
fn pins_claim_their_share_of_the_budget() {
    setup!(st, now, sun);
    let base = st.cell_at(st.from_grid(sun, now as f64), now as f64);
    // Promote systems from a block of cells; follow every cell that holds a
    // pin over the next 30 frames.
    let mut pins = vec![];
    for dx in -1..=1 {
        for dy in -1..=1 {
            for s in st
                .cell([base[0] + dx, base[1] + dy, base[2]], now, &[])
                .unwrap()
                .iter()
                .take(4)
            {
                pins.push(st.promote(s));
            }
        }
    }
    let (mut cells_checked, mut pin_visits, mut exact) = (0usize, 0usize, 0usize);
    for f in now..now + 30 {
        let t = f as f64;
        let mut cells: Vec<[i64; 3]> = pins.iter().map(|p| st.cell_at(p.orbit.xyz(t), t)).collect();
        cells.sort();
        cells.dedup();
        for c in cells.iter().take(8) {
            let a = st.cell(*c, f, &pins).unwrap();
            let b = st.cell(*c, f, &[]).unwrap();
            let p = a.iter().filter(|s| s.pinned).count();
            assert!(p > 0);
            pin_visits += p;
            cells_checked += 1;
            // Exact closure whenever the draw has room for the pins.
            if b.len() >= p {
                assert_eq!(a.len(), b.len(), "frame {f} cell {c:?}");
                exact += 1;
            }
            let mut ids: Vec<String> = a.iter().map(|s| s.address.to_string()).collect();
            ids.sort();
            ids.dedup();
            assert_eq!(ids.len(), a.len(), "no system twice");
        }
    }
    println!("{cells_checked} cells with pins checked, {pin_visits} pin visits, {exact} with exact closure");
    assert!(cells_checked >= 200 && exact == cells_checked);
}

#[test]
fn birth_feeds_formation() {
    setup!(st, now, sun);
    let s = &st
        .cell(
            st.cell_at(st.from_grid(sun, now as f64), now as f64),
            now,
            &[],
        )
        .unwrap()[0];
    let b = st.birth(s);
    assert_eq!(b.address, s.address);
    assert!(b.mass_msun >= s.mass_msun && b.feh.is_finite() && b.birth_gyr < now as f64 / 1000.0);
}

#[test]
fn pack_and_golden() {
    let cp = crate::scenario::CosmologyPack::from_packs(&[crate::scenario::base_cosmology_pack()])
        .unwrap();
    assert_eq!(cp.stars, StarParams::default());
    crate::golden::check("stars.json", &crate::scenario::stars_golden());
}

#[test]
fn a_pin_survives_a_change_to_the_fill_rules() {
    // Y7a: a promoted system is stored whole, so new draw keys (standing in
    // for any change to the fill rules) redraw the fill but never the pin.
    setup!(st, now, sun);
    let base = st.cell_at(st.from_grid(sun, now as f64), now as f64);
    let fill = st.cell(base, now, &[]).unwrap();
    let chosen = &fill[fill.len() / 2];
    assert_eq!(st.draw_one(&chosen.address).unwrap().as_ref(), Some(chosen));
    let pin = st.promote(chosen);
    let stored: StarSystem = serde_json::from_str(&serde_json::to_string(&pin).unwrap()).unwrap();
    assert_eq!(stored, pin);
    let v2 = Stars {
        params: StarParams {
            key_version: 2,
            ..st.params.clone()
        },
        env: st.env,
        green: st.green.clone(),
        seed: st.seed,
        galaxy: st.galaxy.clone(),
    };
    let refill = v2.cell(base, now, &[]).unwrap();
    assert_ne!(refill, fill, "new keys redraw the fill");
    assert_ne!(v2.draw_one(&chosen.address).unwrap().as_ref(), Some(chosen));
    let with_pin = v2.cell(base, now, std::slice::from_ref(&stored)).unwrap();
    let found: Vec<&StarSystem> = with_pin
        .iter()
        .filter(|s| s.address == pin.address)
        .collect();
    assert_eq!(found, vec![&pin]);
    assert_eq!(
        serde_json::to_string(&v2.at(found[0], now as f64 + 50.0)).unwrap(),
        serde_json::to_string(&st.at(&pin, now as f64 + 50.0)).unwrap()
    );
    // Version 1 is today's draw exactly (the G5 golden holds it too).
    assert_eq!(StarParams::default().key_version, 1);
    assert!(st.draw_one(&"u:g5/sys:1".parse().unwrap()).is_err());
}

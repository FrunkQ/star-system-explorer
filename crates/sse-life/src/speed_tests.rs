//! LE4: the speed read-out (life-from-energy design sections 6.1 and 7).

use crate::budget::*;
use crate::history::HistoryParams;
use crate::scenario::*;
use crate::speed::*;

fn hp() -> HistoryParams {
    HistoryParams::from_packs(&[base_life_pack()]).unwrap()
}

fn read(g: &crate::habitat::HabitatGraph) -> Speed {
    let bp = budget_params();
    let ids: Vec<String> = bp.metabolisms.iter().map(|m| m.id.clone()).collect();
    let held = world_budget(g, &bp, &ids);
    let l = crate::ledger::ledger(g, &bp, Some(&held));
    speed(g, &bp, &hp(), &l, Some(&held)).expect("spread settings")
}

#[test]
fn earth_reads_one() {
    let s = read(&graph("earth"));
    println!("{}", s.words);
    for h in &s.habitats {
        println!(
            "{} r {:?} ({}) fill {:?} yr",
            h.kind, h.r_per_s, h.r_limited_by, h.fill_yr
        );
    }
    assert!((0.5..2.0).contains(&s.pace_x_earth), "{}", s.pace_x_earth);
    assert!(s.words.contains("times Earth's pace"), "{}", s.words);
}

#[test]
fn fronts_have_the_right_orders_of_magnitude() {
    let s = read(&graph("earth"));
    let fill = |k: &str| {
        s.habitats
            .iter()
            .find(|h| h.kind == k)
            .and_then(|h| h.fill_yr)
            .unwrap_or_else(|| panic!("{k}"))
    };
    // Microbes cross an ocean in years (section 6.1).
    let ocean = fill("ocean-sunlit");
    assert!((0.3..30.0).contains(&ocean), "ocean {ocean} yr");
    // At the history's 5 Myr steps the ocean and the land fill at once.
    let land = fill("land-surface");
    assert!(land < 1.0e4, "land {land} yr");
    // A kilometre of rock: even well fed, pore water barely mixes.
    let mut g = graph("earth");
    g.habitats.retain(|h| h.kind == "deep-crust");
    let p = &mut g.habitats[0].profile;
    p.temperature_k = Some(crate::Range {
        lo: 330.0,
        hi: 330.0,
    });
    // A vent field's crust, 10 km across and a kilometre deep, well fed.
    p.area_m2 = Some(1.0e8);
    p.volume_m3 = Some(1.0e8 * 1000.0);
    p.materials.clear();
    for k in ["h2_mol_yr", "reductant_mol_yr", "o2_mol_yr", "fe2_mol_yr"] {
        p.materials.insert(k.into(), 1.0e12);
    }
    p.materials.insert("p_new_mol_yr".into(), 1.0e12);
    let rock = read(&g);
    let h = &rock.habitats[0];
    println!("rock {h:?}");
    // Absolute: the rock front moves at metres a year or less, the ocean's
    // at tenths of a metre a second.
    let rock_v = h.front_m_s.expect("a front");
    assert!(rock_v < 1.0e-5, "rock {rock_v} m/s");
    let sea_v = s
        .habitats
        .iter()
        .find(|x| x.kind == "ocean-sunlit")
        .unwrap()
        .front_m_s
        .unwrap();
    assert!(sea_v > 0.01, "sea {sea_v} m/s");
    assert!(h.fill_yr.expect("fills") > ocean, "{h:?}");
}

#[test]
fn growth_falls_to_zero_smoothly_at_the_keep_alive_line() {
    let bp = budget_params();
    let sp = hp().spread.unwrap();
    let h = hp();
    let mut last = f64::INFINITY;
    // Surplus from 1e3 W down to 1e-9 W for a gram: r falls monotonically
    // and continuously to zero.
    for k in 0..=12 {
        let s = sse_core::math::pow(10.0, 3.0 - k as f64);
        let (r, _) = growth_rate(&sp, &h, &bp, 288.0, s, 1.0);
        assert!(r <= last && r > 0.0, "{s}: {r}");
        last = r;
    }
    assert!(last < 1e-12, "{last}");
    assert_eq!(growth_rate(&sp, &h, &bp, 288.0, 0.0, 1.0).0, 0.0);
    assert_eq!(growth_rate(&sp, &h, &bp, 288.0, -1.0, 1.0).0, 0.0);
}

#[test]
fn a_barely_alive_ocean_has_no_front() {
    let s = read(&graph("europa"));
    for h in &s.habitats {
        assert_eq!(h.r_per_s, Some(0.0), "{h:?}");
        assert_eq!(h.fill_yr, None, "{h:?}");
    }
    assert!(s.pace_x_earth < 0.1, "{}", s.pace_x_earth);
}

//! The prototype's checks in test form (the binary prints the full report).
use roster_proto::book::{ledger, state_at, Campaign, Deed, State};
use roster_proto::roster::{BandId, Roster};
use roster_proto::system::*;
use sse_core::Seed;
use sse_formation::{disc, growth, hurley::Fits, leftovers, scenario, violent};

fn system() -> (System, Vec<[u64; 6]>) {
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
    let st = Settings {
        n_exact: 10_000,
        ..Settings::default()
    };
    let sys = System::build(seed, addr, &u, &l, 4570.0, 1.0, &st);
    let solids = sys
        .reservoirs
        .iter()
        .map(|r| {
            l.clouds
                .iter()
                .find(|c| c.address.to_string() == r.cloud_address)
                .unwrap()
                .solid
        })
        .collect();
    (sys, solids)
}

#[test]
fn classes_add_back_to_their_belt() {
    let (sys, solids) = system();
    for (r, s) in sys.reservoirs.iter().zip(solids) {
        let mut sum = [0u64; 6];
        for c in &r.classes {
            for g in 0..6 {
                sum[g] += c.groups[g];
            }
        }
        assert_eq!(sum, s, "{}", r.name);
    }
}

#[test]
fn craters_read_back_and_forth_in_any_order() {
    let (sys, _) = system();
    let w = sys.worlds.iter().position(|w| w.id == "embryo-04").unwrap();
    let mut ro = Roster::new(&sys);
    let (list, _, _) = ro.listed_craters(w, (0.0, sys.horizon_myr), sys.age_myr, 300);
    assert!(list.len() >= 300);
    let ids: Vec<String> = list.iter().take(300).map(|c| c.id.clone()).collect();
    let ask = |order: Vec<usize>| {
        let mut ro = Roster::new(&sys);
        let mut out: Vec<(usize, String)> = order
            .into_iter()
            .map(|i| (i, format!("{:?}", ro.impactor_of(&ids[i]))))
            .collect();
        out.sort();
        out
    };
    let a = ask((0..300).collect());
    let b = ask((0..300).rev().collect());
    assert_eq!(a, b);
    for c in list.iter().take(300) {
        let bd = ro.impactor_of(&c.id);
        let f = ro.fate(bd.band, bd.rank).unwrap();
        assert_eq!(f.t_arrive, c.t_myr);
        assert_eq!(
            sys.reservoirs[bd.band.res].sinks[f.sink].0,
            SinkKind::World(w)
        );
    }
}

#[test]
fn books_close_at_every_moment_and_a_deed_only_touches_its_own() {
    let (sys, _) = system();
    let deb = sys
        .reservoirs
        .iter()
        .position(|r| r.kind == "debris-belt")
        .unwrap();
    let mut ro = Roster::new(&sys);
    let id = BandId {
        res: deb,
        class: 1,
        band: 3,
    };
    let k = 500;
    let cam = Campaign {
        deeds: vec![Deed::MineMember {
            band: id,
            rank: k,
            t_myr: 3000.0,
            by: "test".into(),
        }],
    };
    for i in 0..12 {
        let t = i as f64 * 500.0;
        for ri in 0..sys.reservoirs.len() {
            for ci in 0..sys.reservoirs[ri].classes.len() {
                let a = ledger(&mut ro, &Campaign::default(), ri, ci, t);
                let b = ledger(&mut ro, &cam, ri, ci, t);
                assert!(a.closes() && b.closes(), "{ri} {ci} {t}");
                if (ri, ci) != (deb, 1) {
                    assert_eq!(a, b);
                }
            }
        }
    }
    assert!(matches!(
        state_at(&mut ro, &cam, id, k, 3500.0),
        State::Mined { .. }
    ));
    assert_eq!(
        state_at(&mut ro, &cam, id, k + 1, 3500.0),
        state_at(&mut ro, &Campaign::default(), id, k + 1, 3500.0)
    );
}

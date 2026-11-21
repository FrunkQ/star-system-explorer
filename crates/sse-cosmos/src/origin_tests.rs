//! Reconstructed origins (top-level design 17.1).

use crate::envelope::{EnvelopeParams, GalaxyEnvelope};
use crate::g3_tests::{milky_way_like, world};
use crate::origin::*;
use sse_core::{Address, Seed};

macro_rules! setup {
    ($env:ident, $now:ident, $sun:ident) => {
        let (c, w) = world();
        let (g, d) = milky_way_like(&c, &w);
        let $env = GalaxyEnvelope {
            galaxy: &g,
            disc: &d,
            params: EnvelopeParams::default(),
            seed: Seed::from_text("g4"),
            address: Address::universe("g4").unwrap(),
        };
        let $now = g.now.t_gyr;
        let $sun = $env.sun_radius_kpc();
    };
}

fn ask(now: f64, age: f64, r: f64, keep: Option<f64>) -> OriginAsk {
    OriginAsk {
        now_gyr: now,
        age_gyr: age,
        r_now_kpc: r,
        phi_now_deg: 30.0,
        keep_feh: keep,
    }
}

#[test]
fn an_old_star_beside_the_sun_came_from_the_inner_disc() {
    setup!(env, now, sun);
    let p = OriginParams::default();
    let young = reconstruct(&env, &p, &ask(now, 0.5, sun, None));
    let mid = reconstruct(&env, &p, &ask(now, 4.6, sun, None));
    let old = reconstruct(&env, &p, &ask(now, 10.0, sun, None));
    for o in [&young, &mid, &old] {
        assert!(o.impossible.is_none(), "{o:?}");
        assert_eq!(o.label, "one plausible past");
        let last = o.path.last().unwrap();
        assert!(
            (last[1] - sun).abs() < 1e-9 && (last[2] - 30.0).abs() < 1e-6,
            "{last:?}"
        );
        assert!((o.path[0][0] - o.birth_gyr).abs() < 1e-12);
        assert_eq!(o.composition, "follows birthplace");
        assert_eq!(o.feh, o.feh_gas);
    }
    // The drift spread grows as the square root of age (3.6 kpc at 8 Gyr).
    assert!((old.sigma_kpc - 3.6 * (10.0f64 / 8.0).sqrt()).abs() < 1e-9);
    // A young star was born near its orbit; an old one further in, where the
    // early disc was forming stars, and from poorer gas than the young one.
    assert!(young.migrated_kpc.abs() < 1.0, "{young:?}");
    assert!(old.migrated_kpc > young.migrated_kpc, "{old:?}");
    assert!(old.birth_r_kpc < sun, "{old:?}");
    assert!(old.feh < young.feh, "{} vs {}", old.feh, young.feh);
    assert!(old.why.contains("nearer the centre"), "{}", old.why);
    // Deterministic.
    assert_eq!(old, reconstruct(&env, &p, &ask(now, 10.0, sun, None)));
}

#[test]
fn a_kept_composition_steers_the_birthplace_and_is_marked_when_odd() {
    setup!(env, now, sun);
    let p = OriginParams::default();
    let rich = reconstruct(&env, &p, &ask(now, 8.0, sun, Some(0.3)));
    let poor = reconstruct(&env, &p, &ask(now, 8.0, sun, Some(-0.5)));
    assert_eq!(rich.composition, "kept");
    assert_eq!(rich.feh, 0.3);
    // The disc is richer inside: metal-rich stars come from further in.
    assert!(
        rich.birth_r_kpc < poor.birth_r_kpc,
        "{} vs {}",
        rich.birth_r_kpc,
        poor.birth_r_kpc
    );
    // A composition no birthplace of that age explains is allowed and marked.
    let odd = reconstruct(&env, &p, &ask(now, 11.0, sun, Some(1.5)));
    assert!(odd.impossible.is_none());
    assert!(
        odd.unusual.as_deref().unwrap_or("").contains("sigma"),
        "{odd:?}"
    );
}

#[test]
fn impossible_only_when_no_origin_fits() {
    setup!(env, now, sun);
    let p = OriginParams::default();
    let before_disc = now - env.disc.t_start_gyr + 0.1;
    let o = reconstruct(&env, &p, &ask(now, before_disc, sun, None));
    assert!(
        o.impossible
            .as_deref()
            .unwrap_or("")
            .contains("disc formed"),
        "{o:?}"
    );
    let o = reconstruct(&env, &p, &ask(now, now + 1.0, sun, None));
    assert!(
        o.impossible.as_deref().unwrap_or("").contains("Big Bang"),
        "{o:?}"
    );
    // The oldest stars the disc could make still have an origin.
    let o = reconstruct(
        &env,
        &p,
        &ask(now, now - env.disc.t_start_gyr - 0.2, sun, None),
    );
    assert!(o.impossible.is_none(), "{o:?}");
}

#[test]
fn the_pack_carries_the_defaults() {
    let cp = crate::scenario::CosmologyPack::from_packs(&[crate::scenario::base_cosmology_pack()])
        .unwrap();
    assert_eq!(cp.origin, OriginParams::default());
}

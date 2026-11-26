use crate::describe::describe;
use crate::lifeline::Phase;

#[test]
fn the_sun_now_is_a_main_sequence_star_near_its_measured_values() {
    let d = describe(1.0, 4.57e9, 0.0).unwrap();
    assert_eq!(d.now.phase, Phase::MainSequence);
    assert!((d.now.teff_k - 5772.0).abs() < 300.0, "{}", d.now.teff_k);
    assert!(
        (d.now.luminosity_lsun - 1.0).abs() < 0.2,
        "{}",
        d.now.luminosity_lsun
    );
    assert_eq!(d.remnant.phase, Phase::WhiteDwarf);
    assert!(
        (0.5..0.6).contains(&d.remnant.mass_msun),
        "{}",
        d.remnant.mass_msun
    );
    let ms = &d.phases[1];
    assert_eq!(ms.phase, Phase::MainSequence);
    let life = ms.to_yr.unwrap() - ms.from_yr;
    assert!((8.0e9..1.3e10).contains(&life), "{life}");
    // Phases join end to end and finish in the remnant.
    for w in d.phases.windows(2) {
        assert_eq!(w[0].to_yr, Some(w[1].from_yr));
    }
    assert_eq!(d.phases.last().unwrap().to_yr, None);
}

#[test]
fn heavier_stars_end_as_neutron_stars_and_black_holes() {
    let ns = describe(12.0, 1.0e6, 0.0).unwrap();
    assert_eq!(ns.remnant.phase, Phase::NeutronStar);
    let bh = describe(40.0, 1.0e9, 0.0).unwrap();
    assert_eq!(bh.remnant.phase, Phase::BlackHole);
    assert_eq!(bh.now.phase, Phase::BlackHole);
    assert!(bh.now.teff_k > 0.0 || bh.now.luminosity_w > 0.0);
}

#[test]
fn below_hydrogen_fusion_a_brown_dwarf_cools_and_outside_the_range_is_refused() {
    let bd = describe(0.05, 1.0e9, 0.0).unwrap();
    assert_eq!(bd.now.phase, Phase::BrownDwarf);
    assert_eq!(bd.phases.len(), 1);
    assert_eq!(bd.remnant.from_yr, None);
    let older = describe(0.05, 5.0e9, 0.0).unwrap();
    assert!(older.now.luminosity_w < bd.now.luminosity_w);
    assert!(describe(0.001, 1.0e9, 0.0).is_err());
    assert!(describe(1.0, -1.0, 0.0).is_err());
}

#[test]
fn metal_poor_stars_live_shorter_and_off_the_fits_says_so() {
    let rich = describe(1.0, 1.0e9, 0.0).unwrap();
    let poor = describe(1.0, 1.0e9, -1.5).unwrap();
    let ms = |d: &crate::describe::Description| d.phases[1].to_yr.unwrap();
    assert!(ms(&poor) < ms(&rich));
    let off = describe(1.0, 1.0e9, -3.0).unwrap();
    assert_eq!(off.z_fits, 1.0e-4);
    assert!(!off.notes.is_empty());
}

#[test]
fn formation_draws_the_same_star() {
    // The same life line formation uses: a formed star of the same mass, age
    // and metallicity reads the same state.
    let fits = crate::hurley::Fits::new(crate::inherit::z_of_feh(0.0));
    let line = crate::lifeline::LifeLine::new(&fits, 1.3).unwrap();
    let s = line.state_at(&fits, 2000.0);
    let d = describe(1.3, 2.0e9, 0.0).unwrap();
    assert_eq!(d.now.teff_k, s.teff);
    assert_eq!(d.now.luminosity_lsun, s.luminosity);
}

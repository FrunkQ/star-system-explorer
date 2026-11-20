use crate::*;

fn seed() -> Seed {
    Seed::from_text("nebula-tests")
}

const RSUN: f64 = 6.957e8;

fn ring(companion: bool) -> PnInput {
    PnInput {
        address: "u:t/sys:0/star:0/nebula:0".into(),
        star: "u:t/sys:0/star:0".into(),
        died_at_s: 1e17,
        core_teff_k: 100_000.0,
        core_radius_m: 0.05 * RSUN,
        envelope_kg: 0.3 * MSUN_KG,
        ambient_h_m3: 1e6,
        companion,
        fast_spin: false,
        axis: None,
    }
}

#[test]
fn planetary_nebula_grows_as_age_times_speed_and_fades() {
    let s = PnSettings::default();
    let p = ring(false);
    assert!(planetary_nebula(&seed(), &p, &s, p.died_at_s - 1e6)
        .unwrap()
        .is_none());
    let a = planetary_nebula(&seed(), &p, &s, p.died_at_s + 5000.0 * YEAR_S)
        .unwrap()
        .unwrap();
    let b = planetary_nebula(&seed(), &p, &s, p.died_at_s + 10_000.0 * YEAR_S)
        .unwrap()
        .unwrap();
    let (sa, sb) = (&a.shells[0], &b.shells[0]);
    assert_eq!(sa.speed_m_s, sb.speed_m_s, "same nebula, same draw");
    assert!((sa.radius_m - sa.speed_m_s * 5000.0 * YEAR_S).abs() < 1e-6 * sa.radius_m);
    assert!((sb.radius_m / sa.radius_m - 2.0).abs() < 1e-12);
    // Visible for ten to forty thousand years (design: ten to thirty thousand).
    let life = (a.fades_s - p.died_at_s) / YEAR_S;
    assert!(life > 1e4 && life < 4e4, "{life}");
    assert!(planetary_nebula(&seed(), &p, &s, a.fades_s + 1e6)
        .unwrap()
        .is_none());
    // Glows red in hydrogen, and the Hα/Hβ ratio is case B's.
    let ha = a
        .light
        .lines
        .iter()
        .find(|l| l.line == "ha")
        .unwrap()
        .power_w;
    let hb = a
        .light
        .lines
        .iter()
        .find(|l| l.line == "hb")
        .unwrap()
        .power_w;
    assert!(ha > 0.0 && (ha / hb - 2.86).abs() < 1e-12);
    assert!(a.lobes.is_none() && a.elongation >= 1.0 && a.elongation < 1.5);
}

#[test]
fn a_companion_makes_two_lobes() {
    let s = PnSettings::default();
    let mut p = ring(true);
    p.axis = Some([0.0, 0.0, 1.0]);
    let d = planetary_nebula(&seed(), &p, &s, p.died_at_s + 3000.0 * YEAR_S)
        .unwrap()
        .unwrap();
    let l = d.lobes.unwrap();
    assert_eq!(l.axis, [0.0, 0.0, 1.0]);
    assert!(l.ratio >= 2.0 && l.ratio < 4.0);
}

#[test]
fn halo_rings_lie_outside_the_shell() {
    let s = PnSettings::default();
    for k in 0..20 {
        let mut p = ring(false);
        p.address = format!("u:t/sys:{k}/star:0/nebula:0");
        let d = planetary_nebula(&seed(), &p, &s, p.died_at_s + 2000.0 * YEAR_S)
            .unwrap()
            .unwrap();
        for sh in &d.shells[1..] {
            assert!(sh.radius_m > d.shells[0].radius_m);
        }
    }
}

fn snr(cc: bool, n_cc: f64) -> SnrInput {
    SnrInput {
        address: "u:t/sys:1/nebula:0".into(),
        star: "u:t/sys:1/star:0".into(),
        exploded_at_s: 0.0,
        core_collapse: cc,
        energy_j: 1e44,
        ejecta_kg: if cc { 5.0 } else { 1.4 } * MSUN_KG,
        ambient_h_m3: n_cc * 1e6,
        remnant: cc.then(|| "u:t/sys:1/star:0".to_string()),
    }
}

/// In a uniform fixture the remnant follows the blast-wave law: R ∝ t^(2/5) in
/// the Sedov phase, R = 1.15 (E t² / ρ)^(1/5).
#[test]
fn remnant_follows_the_blast_wave_law() {
    let s = SnrSettings::default();
    let p = snr(true, 1.0);
    let at = |yr: f64| {
        supernova_remnant(&seed(), &p, &s, yr * YEAR_S)
            .unwrap()
            .unwrap()
    };
    let a = at(3000.0);
    let b = at(6000.0);
    assert_eq!(a.phase, "Sedov-Taylor");
    let ratio = b.shells[0].radius_m / a.shells[0].radius_m;
    assert!((ratio - sse_core::math::pow(2.0, 0.4)).abs() < 1e-9);
    let rho = sse_atoms::nebula::MU_H * sse_atoms::nebula::M_H * 1e6;
    let t = 3000.0 * YEAR_S;
    let want = 1.1517 * sse_core::math::pow(1e44 * t * t / rho, 0.2);
    assert!((a.shells[0].radius_m - want).abs() < 1e-9 * want);
    // Merges with its surroundings after a hundred thousand years or more.
    let life = a.fades_s / YEAR_S;
    assert!(life > 1e5 && life < 5e6, "{life}");
    // Denser surroundings stop it sooner and keep it smaller.
    let dense = supernova_remnant(&seed(), &snr(true, 100.0), &s, 3000.0 * YEAR_S)
        .unwrap()
        .unwrap();
    assert!(dense.shells[0].radius_m < a.shells[0].radius_m && dense.fades_s < a.fades_s);
    assert_eq!(a.lighting, vec!["u:t/sys:1/star:0".to_string()]);
}

#[test]
fn star_forming_glow_lasts_as_long_as_its_hot_stars() {
    let o_star = HotStar {
        address: "u:t/sys:2/star:0".into(),
        teff_k: 37_000.0,
        radius_m: 8.0 * RSUN,
        born_s: 0.0,
        dies_s: 6e6 * YEAR_S,
    };
    let p = GlowInput {
        address: "u:t/cluster:0/nebula:0".into(),
        cluster: "u:t/cluster:0".into(),
        stars: vec![o_star],
        cloud_h_m3: 100.0 * 1e6,
    };
    let d = star_forming_glow(&seed(), &p, 1e6 * YEAR_S)
        .unwrap()
        .unwrap();
    let r_pc = d.shells[0].radius_m / PC_M;
    // A late O dwarf in 100 cm⁻³ gas ionises a few parsecs (Osterbrock and Ferland, table 2.3, order of magnitude).
    assert!(r_pc > 1.0 && r_pc < 10.0, "{r_pc}");
    assert_eq!(d.fades_s, 6e6 * YEAR_S);
    assert!(star_forming_glow(&seed(), &p, 7e6 * YEAR_S)
        .unwrap()
        .is_none());
}

/// Counts in an unvisited region are the cause's rate times each nebula's life.
#[test]
fn counts_follow_rates_times_lifetimes() {
    let s = SnrSettings::default();
    let (cc1, ia1) = expected_remnants(10.0, 2.0, 1e6, 1e6, &s);
    let (cc2, ia2) = expected_remnants(20.0, 4.0, 1e6, 1e6, &s);
    assert!((cc2 / cc1 - 2.0).abs() < 1e-12 && (ia2 / ia1 - 2.0).abs() < 1e-12);
    let life = remnant_fades_after(&snr(true, 1.0), &s);
    assert!((cc1 - 10.0 * 1e6 * life / (1e9 * YEAR_S)).abs() < 1e-9 * cc1);
}

/// Same descriptor on every device: pinned descriptors hash to the golden file.
#[test]
fn golden() {
    let pn = planetary_nebula(
        &seed(),
        &ring(true),
        &PnSettings::default(),
        1e17 + 4000.0 * YEAR_S,
    )
    .unwrap();
    let rem = supernova_remnant(
        &seed(),
        &snr(true, 1.0),
        &SnrSettings::default(),
        20_000.0 * YEAR_S,
    )
    .unwrap();
    let ia = supernova_remnant(
        &seed(),
        &snr(false, 0.1),
        &SnrSettings::default(),
        400.0 * YEAR_S,
    )
    .unwrap();
    let g = serde_json::json!({ "pn": pn, "cc": rem, "ia": ia });
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/golden/nebulae.json"
    );
    let text = serde_json::to_string_pretty(&g).unwrap() + "\n";
    if std::env::var("SSE_BLESS").is_ok() {
        std::fs::write(path, &text).unwrap();
    }
    let want = std::fs::read_to_string(path).expect("golden exists (bless with SSE_BLESS=1)");
    assert_eq!(text, want);
}

/// Nauenberg's white dwarf radius: about 0.0125 R☉ (8,700 km) at 0.6 M☉.
#[test]
fn white_dwarf_radius() {
    let r = white_dwarf_radius_m(0.6) / RSUN;
    assert!((r - 0.0125).abs() < 0.0008, "{r}");
    assert!(white_dwarf_radius_m(1.2) < white_dwarf_radius_m(0.6));
}

/// The Sun's death (T2's age-the-Solar-System run: about 0.54 M☉ white dwarf)
/// gives a nebula tied to the star, born on the event's date.
#[test]
fn the_suns_nebula() {
    let d = Death {
        star: "u:sol/sys:0/star:0".into(),
        died_at_s: 2.2e17,
        initial_mass_msun: 1.0,
        remnant_mass_msun: 0.54,
        ambient_h_m3: 1e6,
        companion: false,
    };
    let p = PnInput::from_death(&d);
    assert_eq!(p.address, "u:sol/sys:0/star:0/nebula:0");
    let n = planetary_nebula(
        &seed(),
        &p,
        &PnSettings::default(),
        d.died_at_s + 1e4 * YEAR_S,
    )
    .unwrap()
    .unwrap();
    assert_eq!(n.cause.source, d.star);
    assert_eq!(n.cause.at_s, d.died_at_s);
    assert_eq!(n.lighting, vec![d.star.clone()]);
}

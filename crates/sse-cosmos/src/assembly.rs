//! GH1: one assembly tree per halo (galaxy histories design, section 7.1).
//!
//! The haloes a halo swallowed, as one list. The major mergers G2 drew for
//! it are pins: they claim their share first, and the unevolved subhalo
//! mass function fills the rest, bin by bin (Giocoli, Tormen and van den
//! Bosch 2008, eq. 2, doi:10.1111/j.1365-2966.2008.13182.x), with keyed
//! Poisson counts. Bins are anchored at ψ = m_acc / M₀ = 10⁻³ (Chart level),
//! so a deeper level (Survey, 10⁻⁵) only adds bins below and never changes a
//! member already drawn. Each progenitor falls in when the host held a
//! share of its mass uniform above ψ (accretion follows the host's growth).
//! A progenitor is itself a halo: its own tree is the same function at its
//! address, read at its infall. Fudge F42.

use crate::cosmology::Cosmology;
use crate::haloes::{poisson, Halo};
use serde::{Deserialize, Serialize};
use sse_core::math as m;
use sse_core::{Address, Draws, Seed, Stream};

/// The tree's settings (`SkyParams` carries the satellites' processes).
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct AssemblyParams {
    /// dN/d ln ψ = N0 x^-α exp(-k x³), x = ψ / α: N0 0.21, α 0.8, k 6.283.
    pub shmf_n0: f64,
    pub shmf_alpha: f64,
    pub shmf_k: f64,
    /// Bins per decade of ψ.
    pub bins_per_decade: u32,
    /// The bins' anchor and the Chart floor (ψ 10⁻³), and the Survey floor.
    pub chart_psi: f64,
    pub survey_psi: f64,
    /// Most members drawn in one bin.
    pub max_per_bin: u64,
}

impl Default for AssemblyParams {
    // 6.283 is Giocoli's fitted constant as printed, not 2π.
    #[allow(clippy::approx_constant)]
    fn default() -> Self {
        AssemblyParams {
            shmf_n0: 0.21,
            shmf_alpha: 0.8,
            shmf_k: 6.283,
            bins_per_decade: 8,
            chart_psi: 1.0e-3,
            survey_psi: 1.0e-5,
            max_per_bin: 400,
        }
    }
}

/// How deep a tree is read.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Level {
    /// Down to ψ 10⁻³ (about 55 for a Milky-Way-like halo).
    Chart,
    /// Down to ψ 10⁻⁵ (about 2,000).
    Survey,
}

/// One swallowed halo.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Progenitor {
    pub address: String,
    /// Its mass at infall over the host's mass at the time read.
    pub psi: f64,
    pub mass_at_infall_msun: f64,
    pub z_infall: f64,
    pub t_infall_gyr: f64,
    /// A G2 major merger (a pin), not a fill member.
    pub pinned: bool,
    /// Where its keyed draws start (its own orbit and place use the next
    /// counters after these two).
    pub draw_base: u64,
}

/// A halo's assembly tree as read at a redshift.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Tree {
    pub host: String,
    pub host_mass_msun: f64,
    pub z_read: f64,
    pub floor_psi: f64,
    /// Expected fill members above the floor (before pins claim theirs).
    pub expected: f64,
    /// Mass that fell in as these haloes, over the host's mass.
    pub accreted_psi: f64,
    /// Members, largest first (ties by address).
    pub progenitors: Vec<Progenitor>,
}

fn stream(s: &str) -> Stream {
    Stream::new(s).expect("valid stream")
}

/// The fill's keyed stream (Y4 part 2's, so its satellites keep their draws).
pub const FILL_STREAM: &str = "cosmos.galaxy.satellite.v1";
/// Pins' counters start here, above any fill bin's.
const PIN_BASE: u64 = 1 << 40;

/// The halo's tree at redshift `z_read` down to `level`.
pub fn tree(
    c: &Cosmology,
    seed: Seed,
    host: &Halo,
    z_read: f64,
    ap: &AssemblyParams,
    level: Level,
) -> Result<Tree, String> {
    let addr: Address = host.address.parse().map_err(|e| format!("{e:?}"))?;
    let d = Draws::new(seed, addr.clone(), stream(FILL_STREAM));
    let m_host = host.mass_at(z_read);
    let floor = match level {
        Level::Chart => ap.chart_psi,
        Level::Survey => ap.survey_psi.min(ap.chart_psi),
    };
    let step = m::ln(10.0) / f64::from(ap.bins_per_decade);
    let anchor = m::ln(ap.chart_psi);
    let n_up = m::ceil(-anchor / step).max(1.0) as i64;
    let n_down = m::ceil((anchor - m::ln(floor)) / step).max(0.0) as i64;
    let shmf = |ln_psi: f64| {
        let x = m::exp(ln_psi) / ap.shmf_alpha;
        ap.shmf_n0 * m::pow(x, -ap.shmf_alpha) * m::exp(-ap.shmf_k * x * x * x)
    };
    // The bin a ψ falls in (index from the anchor, up positive).
    let bin_of = |psi: f64| m::floor((m::ln(psi) - anchor) / step) as i64;
    // Pins: G2's major mergers before the time read.
    let mut pins: Vec<Progenitor> = vec![];
    for (k, mg) in host.mergers.iter().enumerate() {
        if mg.z < z_read {
            continue;
        }
        // FMBK's rate is per descendant mass, ξ the smaller over the
        // larger: the smaller progenitor held ξ / (1 + ξ) of it.
        let m_sat = mg.ratio / (1.0 + mg.ratio) * mg.mass;
        let psi = m_sat / m_host;
        pins.push(Progenitor {
            address: addr
                .child_key("prog", &format!("m{k}"))
                .map_err(|e| format!("{e:?}"))?
                .to_string(),
            psi,
            mass_at_infall_msun: m_sat,
            z_infall: mg.z,
            t_infall_gyr: mg.t_gyr,
            pinned: true,
            draw_base: PIN_BASE + k as u64 * 8,
        });
    }
    // The fill's expected mass below each bin's lower edge, down to the
    // Survey floor whatever the level read, so a level never changes a
    // member above it.
    let n_all = m::ceil((anchor - m::ln(ap.survey_psi.min(ap.chart_psi))) / step).max(0.0) as i64;
    let mshmf = |ln_psi: f64| m::exp(ln_psi) * shmf(ln_psi);
    let mut below = vec![0.0; (n_up + n_all + 1) as usize];
    for j in -n_all..n_up {
        let lo = anchor + step * j as f64;
        let k = (j + n_all + 1) as usize;
        below[k] = below[k - 1] + crate::cosmology::quad(&mshmf, lo, lo + step, 8);
    }
    // below[j + n_all] is the expected mass in the bins under bin j.
    let reserve_under = |j: i64| below[(j + n_all) as usize];
    let mut out = pins.clone();
    let mut expected = 0.0;
    // Bins top down, so the largest claim the budget first.
    let mut budget = 1.0 - pins.iter().map(|p| p.psi).sum::<f64>();
    for j in (-n_down..n_up).rev() {
        let lo = anchor + step * j as f64;
        let lambda = crate::cosmology::quad(&shmf, lo, lo + step, 8);
        expected += lambda;
        let claimed = pins.iter().filter(|p| bin_of(p.psi) == j).count() as f64;
        // Counters: bins at and above the anchor keep Y4's numbering; the
        // deeper ones follow after them.
        let cj = if j >= 0 {
            j as u64
        } else {
            n_up as u64 + (-j - 1) as u64
        };
        let count = poisson(&d, cj * 4096, (lambda - claimed).max(0.0)).min(ap.max_per_bin);
        for i in 0..count {
            let k0 = cj * 4096 + 16 + i * 8;
            let psi = m::exp(lo + step * d.unit(k0));
            // Closure: a member fits only with room left for the mass
            // expected under it (fudge F42).
            if psi + reserve_under(j) > budget {
                continue;
            }
            budget -= psi;
            let f = psi + (1.0 - psi) * d.unit(k0 + 1);
            let (mut zl, mut zh) = (z_read, 20.0_f64);
            for _ in 0..60 {
                let mid = 0.5 * (zl + zh);
                if host.mass_at(mid) > f * m_host {
                    zl = mid;
                } else {
                    zh = mid;
                }
            }
            let z = 0.5 * (zl + zh);
            out.push(Progenitor {
                address: addr
                    .child_key("prog", &format!("b{cj}-{i}"))
                    .map_err(|e| format!("{e:?}"))?
                    .to_string(),
                psi,
                mass_at_infall_msun: psi * m_host,
                z_infall: z,
                t_infall_gyr: c.time_of_scale(1.0 / (1.0 + z)),
                pinned: false,
                draw_base: k0,
            });
        }
    }
    out.sort_by(|a, b| {
        b.psi
            .total_cmp(&a.psi)
            .then_with(|| a.address.cmp(&b.address))
    });
    Ok(Tree {
        host: host.address.clone(),
        host_mass_msun: m_host,
        z_read,
        floor_psi: floor,
        expected,
        accreted_psi: 1.0 - budget,
        progenitors: out,
    })
}

/// Today's mass of a median halo that had `m_infall` at `z_infall`: the
/// progenitor as a halo of its own (its growth stops being its own at
/// infall; this is the growth it would have had).
pub fn mass_today_for(c: &Cosmology, m_infall: f64, z_infall: f64) -> f64 {
    let mut m0 = m_infall;
    for _ in 0..12 {
        let (al, be) = crate::haloes::correa(c, m0, 0.0);
        m0 = m_infall / (m::pow(1.0 + z_infall, al) * m::exp(be * z_infall));
    }
    m0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::haloes::{HaloParams, Haloes};
    use sse_core::Address;

    fn trees(n: usize, level: Level) -> Vec<Tree> {
        let (c, w) = crate::g3_tests::world();
        let hs = Haloes {
            cosmo: &c,
            web: &w,
            params: HaloParams::default(),
            seed: Seed::from_text("gh1"),
            universe: Address::universe("gh1").unwrap(),
        };
        let ap = AssemblyParams::default();
        (0..n)
            .map(|i| {
                let h = hs.median(1.0e12, &format!("h{i}"));
                tree(&c, hs.seed, &h, 0.0, &ap, level).unwrap()
            })
            .collect()
    }

    #[test]
    fn a_tree_holds_no_more_mass_than_its_host() {
        for t in trees(20, Level::Survey) {
            let sum: f64 = t.progenitors.iter().map(|p| p.psi).sum();
            assert!(
                (sum - t.accreted_psi).abs() < 1e-9 && sum <= 1.0 + 1e-12,
                "{sum} {} {:?}",
                t.accreted_psi,
                t.progenitors
                    .iter()
                    .filter(|p| p.pinned)
                    .map(|p| p.psi)
                    .collect::<Vec<_>>()
            );
            assert!(t.progenitors.windows(2).all(|w| w[0].psi >= w[1].psi));
            for p in &t.progenitors {
                assert!(p.z_infall >= t.z_read && p.psi > 0.0);
            }
        }
    }

    #[test]
    fn counts_follow_the_subhalo_mass_function() {
        let chart = trees(40, Level::Chart);
        let n: usize = chart.iter().map(|t| t.progenitors.len()).sum();
        let e: f64 = chart.iter().map(|t| t.expected).sum();
        // About 55 a Milky-Way-like halo above 10⁻³ (Y4 part 2's count).
        assert!((e / 40.0 - 55.0).abs() < 5.0, "{}", e / 40.0);
        // Closure turns some away (most where G2's major mergers already
        // built much of the host), never more than drawn.
        assert!(
            n as f64 > 0.8 * e && (n as f64) < e + 4.0 * m::sqrt(e),
            "{n} {e}"
        );
        let survey = trees(8, Level::Survey);
        let n: usize = survey.iter().map(|t| t.progenitors.len()).sum();
        let e: f64 = survey.iter().map(|t| t.expected).sum();
        assert!(e / 8.0 > 2000.0, "{}", e / 8.0);
        assert!(
            n as f64 > 0.85 * e && (n as f64) < e + 4.0 * m::sqrt(e),
            "{n} {e}"
        );
    }

    #[test]
    fn a_deeper_level_keeps_every_member_already_drawn() {
        let chart = trees(10, Level::Chart);
        let survey = trees(10, Level::Survey);
        for (a, b) in chart.iter().zip(&survey) {
            for p in &a.progenitors {
                assert!(b.progenitors.contains(p), "{} lost", p.address);
            }
        }
        // And the same tree read twice is the same tree.
        assert_eq!(chart, trees(10, Level::Chart));
    }

    #[test]
    fn major_mergers_are_pinned_into_the_tree() {
        let (c, w) = crate::g3_tests::world();
        let hs = Haloes {
            cosmo: &c,
            web: &w,
            params: HaloParams::default(),
            seed: Seed::from_text("gh1"),
            universe: Address::universe("gh1").unwrap(),
        };
        let ap = AssemblyParams::default();
        let mut seen = 0;
        for i in 0..40 {
            let h = hs.median(1.0e12, &format!("h{i}"));
            let t = tree(&c, hs.seed, &h, 0.0, &ap, Level::Chart).unwrap();
            let pins: Vec<_> = t.progenitors.iter().filter(|p| p.pinned).collect();
            assert_eq!(pins.len(), h.mergers.len());
            for (p, mg) in pins.iter().zip(&h.mergers) {
                let _ = mg;
                assert!(p.mass_at_infall_msun > 0.0);
            }
            for mg in &h.mergers {
                assert!(t
                    .progenitors
                    .iter()
                    .any(|p| p.pinned && (p.z_infall - mg.z).abs() < 1e-12));
            }
            seen += pins.len();
            // A progenitor is a halo of its own, with its own tree.
            let p = &t.progenitors[0];
            let m0 = mass_today_for(&c, p.mass_at_infall_msun, p.z_infall);
            let ph = hs.median_at(m0, &p.address.parse().unwrap());
            assert!((ph.mass_at(p.z_infall) / p.mass_at_infall_msun - 1.0).abs() < 1e-3);
            let pt = tree(&c, hs.seed, &ph, p.z_infall, &ap, Level::Chart).unwrap();
            assert!(pt.progenitors.iter().all(|q| q.address != p.address));
        }
        assert!(seen > 0, "no major mergers in 40 haloes");
    }
}

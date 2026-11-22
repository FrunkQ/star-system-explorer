//! E4: soil as state, made by life (evolution design section 8).
//!
//! One land region per world for now. The surface track's regions (P4) will
//! split it on a fixed grid, and each region then runs this same box. A
//! region carries the five soil quantities of section 8.2:
//!
//! - **Loose depth.** Soil production falls exponentially with depth
//!   (Heimsath et al. 1997), against erosion. The 5 Myr history step is
//!   thousands of times longer than a soil's response time (h0 / P0, a few
//!   thousand years), so each step holds the steady depth
//!   `h = h0 ln(P / E)`. It is bounded by construction, and zero when
//!   production cannot keep up with erosion (bare rock).
//! - **Fine fraction.** It relaxes towards the share chemical weathering
//!   can hold at this weathering intensity, over `fines_tau_myr`.
//! - **Organic carbon.** It is at steady state between litter and
//!   decomposition, `C = NPP / k`. Decomposition `k` follows temperature (Q10)
//!   and oxygen. Without free oxygen only anaerobic decomposers work, much
//!   more slowly, so carbon piles up. This is the coal-swamp lever.
//! - **Nutrient stock**, relative to today's Earth: weathering release over
//!   leaching, with decomposers' reuse before burial.
//! - **Water-holding capacity**, derived from depth, fines and organic carbon.
//!
//! Two roles, by function (section 8.1). **Weatherers** are the land life that
//! holds the region, measured by its production. **Decomposers** are the
//! oxidant-using life on dead matter, measured by the oxygen they have.
//!
//! What it feeds:
//! - the derived weathering factor, which replaces L2's fixed 4;
//! - the soil carbon reservoir in the oxygen ledger. Carbon held in soil left
//!   its O2 in the air, and carbon lost from soil takes it back;
//! - the temperature change that the weathering factor's CO2 drawdown gives,
//!   at a stated climate sensitivity.

use serde::{Deserialize, Serialize};
use sse_core::math as m;

/// Settings (`life_feedback.soil`). Opt-in: a run without them keeps L2's
/// fixed land weathering factor.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct SoilParams {
    /// Soil production at zero depth, m/Myr, and its e-folding depth, m
    /// (Heimsath et al. 1997, Tennessee Valley: 77 m/Myr, 0.5 m).
    pub p0_m_myr: f64,
    pub h0_m: f64,
    /// Geological erosion of the land surface, m/Myr (Montgomery 2007).
    pub erosion_m_myr: f64,
    /// Production with no life on the land, as a share of `p0` (fudge F31).
    pub abiotic_share: f64,
    /// Apparent activation energy of silicate weathering, J/mol, and the
    /// temperature its rates are quoted at, K (Brady 1991).
    pub ea_j_mol: f64,
    pub t_ref_k: f64,
    /// Land production, g C m-2 yr-1, at which weatherers hold half the
    /// land (cover `b = npp / (npp + half)`; fudge F31).
    pub npp_half_gc_m2_yr: f64,
    /// Today's Earth land production, g C m-2 yr-1 (about 60 Gt C a year on
    /// 1.5e14 m2), the reference the factor and nutrients are scaled to.
    pub npp_ref_gc_m2_yr: f64,
    /// Today's Earth soil depth, m, at which roots and acids reach rock.
    pub h_contact_m: f64,
    /// Weathering factor at today's Earth (Berner 1997's 4) and its cap.
    pub factor_ref: f64,
    pub factor_max: f64,
    /// Soil carbon turnover at `t_ref_k` with today's oxygen, years
    /// (Carvalhais et al. 2014's global mean ecosystem turnover); Q10 of
    /// decomposition (Davidson and Janssens 2006).
    pub turnover_yr: f64,
    pub q10: f64,
    /// Decomposition without free oxygen, as a share of the oxic rate
    /// (fudge F31).
    pub anoxic_decomposition: f64,
    /// Largest fine fraction a soil holds, and its relaxation time, Myr
    /// (fudge F31).
    pub fines_max: f64,
    pub fines_tau_myr: f64,
    /// Share of nutrients decomposers return before burial at today's
    /// oxygen.
    pub reuse_max: f64,
    /// Equilibrium climate sensitivity, K per doubling of CO2 (IPCC AR6
    /// WG1 chapter 7 best estimate).
    pub sensitivity_k: f64,
    pub sources: Vec<String>,
}

impl Default for SoilParams {
    fn default() -> Self {
        SoilParams {
            p0_m_myr: 77.0,
            h0_m: 0.5,
            erosion_m_myr: 17.0,
            abiotic_share: 0.15,
            ea_j_mol: 63_000.0,
            t_ref_k: 288.0,
            npp_half_gc_m2_yr: 100.0,
            npp_ref_gc_m2_yr: 400.0,
            h_contact_m: 0.3,
            factor_ref: 4.0,
            factor_max: 10.0,
            turnover_yr: 23.0,
            q10: 2.0,
            anoxic_decomposition: 0.1,
            fines_max: 0.5,
            fines_tau_myr: 20.0,
            reuse_max: 0.9,
            sensitivity_k: 3.0,
            sources: vec![],
        }
    }
}

/// One land region's soil.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize, Default)]
pub struct Soil {
    pub depth_m: f64,
    pub fines: f64,
    pub organic_c_kg_m2: f64,
    /// Nutrient stock relative to today's Earth.
    pub nutrient_rel: f64,
    /// Plant-available water the column holds, mm.
    pub water_mm: f64,
    /// Weatherer cover (0 to 1) and decomposer activity (0 to 1).
    pub weatherers: f64,
    pub decomposers: f64,
    /// Silicate weathering relative to bare rock.
    pub weathering_factor: f64,
    /// Soil carbon held over the whole land, mol.
    pub carbon_mol: f64,
}

impl Soil {
    /// The land as a whole: area-weighted means, carbon summed. With no
    /// land, bare (factor 1).
    pub fn area_mean(
        regions: &std::collections::BTreeMap<String, Soil>,
        land: &[crate::history::LandRegion],
    ) -> Soil {
        let mut out = Soil::default();
        let mut area = 0.0;
        for r in land {
            let Some(s) = regions.get(&r.key) else {
                continue;
            };
            let a = r.area_m2;
            area += a;
            out.depth_m += a * s.depth_m;
            out.fines += a * s.fines;
            out.organic_c_kg_m2 += a * s.organic_c_kg_m2;
            out.nutrient_rel += a * s.nutrient_rel;
            out.water_mm += a * s.water_mm;
            out.weatherers += a * s.weatherers;
            out.decomposers += a * s.decomposers;
            out.weathering_factor += a * s.weathering_factor;
            out.carbon_mol += s.carbon_mol;
        }
        if area <= 0.0 {
            return Soil {
                weathering_factor: 1.0,
                ..Soil::default()
            };
        }
        for v in [
            &mut out.depth_m,
            &mut out.fines,
            &mut out.organic_c_kg_m2,
            &mut out.nutrient_rel,
            &mut out.water_mm,
            &mut out.weatherers,
            &mut out.decomposers,
            &mut out.weathering_factor,
        ] {
            *v /= area;
        }
        out
    }
}

/// One step's inputs.
pub struct SoilStep {
    pub dt_myr: f64,
    pub land_area_m2: f64,
    pub temperature_k: f64,
    /// Land production, g C a year.
    pub land_npp_gc_yr: f64,
    /// Free O2 relative to today's level.
    pub o2_rel: f64,
}

impl SoilParams {
    fn arrhenius(&self, t: f64) -> f64 {
        let r = sse_atoms::chem::R_GAS;
        m::exp(self.ea_j_mol / r * (1.0 / self.t_ref_k - 1.0 / t))
    }

    fn cover(&self, npp_m2: f64) -> f64 {
        if npp_m2 <= 0.0 {
            0.0
        } else {
            npp_m2 / (npp_m2 + self.npp_half_gc_m2_yr)
        }
    }

    /// Steady loose depth for a production rate.
    fn depth(&self, p_m_myr: f64) -> f64 {
        if p_m_myr <= self.erosion_m_myr || self.erosion_m_myr <= 0.0 {
            0.0
        } else {
            self.h0_m * m::ln(p_m_myr / self.erosion_m_myr)
        }
    }

    /// How much of the rock roots and acids reach, 0 to 1.
    fn contact(&self, h: f64) -> f64 {
        h / (h + self.h_contact_m)
    }

    /// The weathering factor at today's Earth by construction, 1 on bare
    /// land, capped.
    fn factor(&self, b: f64, h: f64) -> f64 {
        let b_ref = self.cover(self.npp_ref_gc_m2_yr);
        let h_ref = self.depth(self.p0_m_myr);
        let reach = (b * self.contact(h)) / (b_ref * self.contact(h_ref)).max(1e-12);
        (1.0 + (self.factor_ref - 1.0) * reach).min(self.factor_max)
    }

    /// The soil after one step.
    pub fn step(&self, prev: &Soil, s: &SoilStep) -> Soil {
        if s.land_area_m2 <= 0.0 {
            return Soil {
                weathering_factor: 1.0,
                ..Soil::default()
            };
        }
        let npp_m2 = s.land_npp_gc_yr / s.land_area_m2;
        let b = self.cover(npp_m2);
        let f_t = self.arrhenius(s.temperature_k);
        let p = self.p0_m_myr * (self.abiotic_share + (1.0 - self.abiotic_share) * b) * f_t;
        let depth = self.depth(p);
        // Fines: the share chemical weathering can hold, approached over
        // fines_tau.
        let intensity = (p / (p + self.erosion_m_myr)).clamp(0.0, 1.0);
        let fines_eq = if depth > 0.0 {
            self.fines_max * intensity
        } else {
            0.0
        };
        let fines = fines_eq + (prev.fines - fines_eq) * m::exp(-s.dt_myr / self.fines_tau_myr);
        // Decomposers: aerobic in proportion to oxygen up to today's level,
        // anaerobic below.
        let d = s.o2_rel.clamp(0.0, 1.0);
        let k_yr = (self.anoxic_decomposition + (1.0 - self.anoxic_decomposition) * d)
            * m::pow(self.q10, (s.temperature_k - self.t_ref_k) / 10.0)
            / self.turnover_yr;
        // Litter only stays where there is loose ground to hold it.
        let organic = if depth > 0.0 && k_yr > 0.0 {
            npp_m2 / k_yr / 1000.0
        } else {
            0.0
        };
        let reuse = self.reuse_max * d;
        let reuse_ref = self.reuse_max;
        let p_ref = self.p0_m_myr
            * (self.abiotic_share + (1.0 - self.abiotic_share) * self.cover(self.npp_ref_gc_m2_yr));
        let nutrient_rel = (p / p_ref) * (1.0 - reuse_ref) / (1.0 - reuse).max(1e-6);
        // Plant-available water, mm: about 5 % of a column of coarse
        // material, up to 20 % more with fines and up to 10 % more with
        // organic matter (fudge F31).
        let theta =
            0.05 + 0.15 * fines / self.fines_max.max(1e-12) + 0.1 * (organic / (organic + 10.0));
        let water_mm = depth * theta * 1000.0;
        let factor = self.factor(b, depth);
        Soil {
            depth_m: depth,
            fines,
            organic_c_kg_m2: organic,
            nutrient_rel,
            water_mm,
            weatherers: b,
            decomposers: d,
            weathering_factor: factor,
            carbon_mol: organic * 1000.0 * s.land_area_m2 / 12.011,
        }
    }

    /// Temperature change from a CO2 level relative to bare-land weathering.
    pub fn temperature_change_k(&self, co2_rel: f64) -> f64 {
        if co2_rel <= 0.0 {
            0.0
        } else {
            self.sensitivity_k * m::log2(co2_rel)
        }
    }
}

/// A sample of the soil through time.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct SoilPoint {
    pub at_myr: f64,
    pub soil: Soil,
    /// Climate change from land weathering, K (negative: cooler).
    pub delta_t_k: f64,
}

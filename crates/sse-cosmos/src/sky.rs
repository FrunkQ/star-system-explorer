//! Y2: which stars a sky draw fetches (night sky design section 4.3).
//!
//! For an observer at a place and time, each luminosity class has a
//! visibility radius: the distance at which its brightest member reaches the
//! limiting magnitude. The count tree (Y1) is walked inside that radius, so
//! the cost follows the stars found, not the volume. The kernel turns each
//! kept star into band magnitudes through dust (`sse_light::photometry`) and
//! keeps those at or above the limit.
//!
//! The radius here uses no dust, so it is an upper bound: dust only ever
//! dims, and every star inside is checked one by one with its own sight
//! line. It is capped at `SkyParams::max_radius_pc` (fudge F35).
//!
//! Dust: A_V along a sight line is the gas column met on the way, from the
//! galaxy envelope (`Local::gas_pc3`), over the gas column that gives one
//! magnitude, scaled by the gas's metallicity (dust follows metals).

use crate::stars::{StarSystem, Stars};
use serde::{Deserialize, Serialize};
use sse_core::math as m;

/// The sky draw's settings (`provides.other.cosmology.sky`).
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct SkyParams {
    /// Gas column (hydrogen and helium) per magnitude of V extinction at
    /// solar metallicity, solar masses per pc². From N_H / A_V = 2.21e21
    /// cm⁻² mag⁻¹ (Güver and Özel 2009, doi:10.1111/j.1365-2966.2009.15598.x)
    /// with helium (a mass factor of 1.36).
    pub gas_per_av_msun_pc2: f64,
    /// Steps along a sight line for its dust column.
    pub dust_steps: u32,
    /// Farthest a class is walked, pc (fudge F35).
    pub max_radius_pc: f64,
    /// Luminosity of the brightest class's brightest member, L☉ (the top
    /// class has no upper edge).
    pub top_lsun: f64,
    /// Margin added to the limit when choosing radii, magnitudes, so a star
    /// whose band correction is a little brighter than the bound still
    /// falls inside.
    pub radius_margin_mag: f64,
    /// Y4: where our galaxy sits in the cosmic web (comoving Mpc; the
    /// centre of region cell 0 by default), how far other galaxies are
    /// looked for, and the radius inside which a web halo is taken to be
    /// our own (fudge F37).
    pub home_mpc: [f64; 3],
    pub galaxy_reach_mpc: f64,
    pub home_exclusion_mpc: f64,
    /// Y4 part 2: our galaxy's satellites.
    pub satellites: SatelliteParams,
}

/// How our galaxy's satellites are drawn (Y4 part 2, fudge F39).
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct SatelliteParams {
    /// Unevolved subhalo mass function, dN/d ln ψ = N0 x^-α exp(-k x³),
    /// x = ψ / α, ψ = m_acc / M0 (Giocoli, Tormen and van den Bosch 2008,
    /// eq. 2, doi:10.1111/j.1365-2966.2008.13182.x): N0 0.21, α 0.8, k 6.283.
    pub shmf_n0: f64,
    pub shmf_alpha: f64,
    pub shmf_k: f64,
    /// Smallest subhalo drawn, as ψ (1e-3 of a 1e12 M☉ host is 1e9 M☉,
    /// about where galaxies brighter than the classical dwarfs live).
    pub psi_min: f64,
    /// ln ψ bins per decade for the keyed Poisson counts.
    pub bins_per_decade: u32,
    /// Merging by dynamical friction after infall (Boylan-Kolchin, Ma and
    /// Quataert 2008, eqs. 5 and 6, doi:10.1111/j.1365-2966.2007.12530.x):
    /// τ/τ_dyn = A (M/m)^b / ln(1 + M/m) exp(c η) (r_c/r_vir)^d with
    /// τ_dyn = 0.1 / H(z); A 0.216, b 1.3, c 1.9, d 1.0.
    pub df_a: f64,
    pub df_b: f64,
    pub df_c: f64,
    pub df_d: f64,
    /// Orbit ranges drawn uniformly: circularity η and r_c / r_vir.
    pub eta_range: [f64; 2],
    pub rc_range: [f64; 2],
    /// Radial number density of surviving subhaloes, Einasto with shape α
    /// and r_-2 as a fraction of r_200 (Springel et al. 2008, Aquarius,
    /// doi:10.1111/j.1365-2966.2008.14066.x): 0.678 and 0.81. Drawn
    /// between `r_min_frac` and 1 of r_200.
    pub einasto_alpha: f64,
    pub einasto_r2_frac: f64,
    pub r_min_frac: f64,
    /// Star formation goes on this long after infall, then stops ("delayed
    /// then rapid": satellite star formation "unaffected for 2-4 Gyr after
    /// infall", then quenched in under 0.8 Gyr, Wetzel et al. 2013,
    /// doi:10.1093/mnras/stt469; here one delay for every mass, fudge F39).
    pub quench_delay_gyr: f64,
}

impl Default for SatelliteParams {
    // 6.283 is Giocoli's fitted constant as printed, not 2π.
    #[allow(clippy::approx_constant)]
    fn default() -> Self {
        SatelliteParams {
            shmf_n0: 0.21,
            shmf_alpha: 0.8,
            shmf_k: 6.283,
            psi_min: 1.0e-3,
            bins_per_decade: 8,
            df_a: 0.216,
            df_b: 1.3,
            df_c: 1.9,
            df_d: 1.0,
            eta_range: [0.3, 1.0],
            rc_range: [0.5, 1.0],
            einasto_alpha: 0.678,
            einasto_r2_frac: 0.81,
            r_min_frac: 0.05,
            quench_delay_gyr: 3.0,
        }
    }
}

impl Default for SkyParams {
    fn default() -> Self {
        SkyParams {
            gas_per_av_msun_pc2: 21.0,
            dust_steps: 24,
            max_radius_pc: 4000.0,
            top_lsun: 1.0e6,
            radius_margin_mag: 0.5,
            home_mpc: [4.0, 4.0, 4.0],
            galaxy_reach_mpc: 12.0,
            home_exclusion_mpc: 0.3,
            satellites: SatelliteParams::default(),
        }
    }
}

/// One star a walk found, before its photometry.
#[derive(Clone, Debug)]
pub struct Found {
    pub system: StarSystem,
    pub class: usize,
}

impl Stars<'_> {
    /// The brightest luminosity a class can hold, L☉.
    pub fn class_top_lsun(&self, class: usize, sky: &SkyParams) -> f64 {
        self.params
            .class_edges_lsun
            .get(class)
            .copied()
            .unwrap_or(sky.top_lsun)
    }

    /// Grid coordinates (co-rotating, pc) of a galactic position at a time.
    pub fn to_grid(&self, xyz: [f64; 3], t_myr: f64) -> [f64; 3] {
        // from_grid rotates by the grid angle; undo it.
        let back = self.from_grid([xyz[0], -xyz[1], xyz[2]], t_myr);
        [back[0], -back[1], xyz[2]]
    }

    /// V extinction along a straight line between two galactic positions
    /// (pc) at a time, magnitudes.
    pub fn dust_av(&self, from: [f64; 3], to: [f64; 3], t_myr: f64, sky: &SkyParams) -> f64 {
        let d = [to[0] - from[0], to[1] - from[1], to[2] - from[2]];
        let len = m::sqrt(d[0] * d[0] + d[1] * d[1] + d[2] * d[2]);
        if len <= 0.0 {
            return 0.0;
        }
        let n = sky.dust_steps.max(1);
        let ds = len / n as f64;
        let mut col = 0.0;
        for i in 0..n {
            let f = (i as f64 + 0.5) / n as f64;
            let p = [from[0] + f * d[0], from[1] + f * d[1], from[2] + f * d[2]];
            let (gas, feh) = self.env.gas_pc3(self.place(p, t_myr));
            col += gas * m::pow(10.0, feh) * ds;
        }
        col / sky.gas_per_av_msun_pc2
    }

    /// The envelope's place for a galactic position (pc) at a time (Myr).
    pub fn place(&self, xyz: [f64; 3], t_myr: f64) -> crate::envelope::Place {
        crate::envelope::Place {
            r_kpc: m::sqrt(xyz[0] * xyz[0] + xyz[1] * xyz[1]) / 1000.0,
            phi: m::atan2(xyz[1], xyz[0]),
            z_pc: xyz[2],
            t_gyr: t_myr / 1000.0,
        }
    }

    /// Every fill star within its class's radius of a galactic position in
    /// the frame holding `t_myr`, through the count tree: `radii_pc[c]` is
    /// class c's radius. One walk serves every class, so the envelope reads
    /// and class shares are made once. Returns the stars, each class's
    /// count, and the walk's envelope reads.
    pub fn sky_walk(
        &self,
        radii_pc: &[f64],
        observer: [f64; 3],
        t_myr: f64,
    ) -> Result<(Vec<Found>, Vec<usize>, usize), String> {
        let frame = self.frame_of(t_myr);
        let w = crate::star_tree::TreeWalk::new(self, frame);
        let centre = self.to_grid(observer, frame as f64 * self.params.frame_myr);
        let mut out = vec![];
        let mut per = vec![];
        for (class, r) in radii_pc.iter().enumerate() {
            let before = out.len();
            for (cell, _) in w.cells_within(class, centre, *r)? {
                for s in w.draw(cell, Some(class))? {
                    out.push(Found { system: s, class });
                }
            }
            per.push(out.len() - before);
        }
        Ok((out, per, w.reads()))
    }
}

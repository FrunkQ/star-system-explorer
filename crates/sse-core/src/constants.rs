//! The one handle onto the master data sheet's constants (matter pack
//! design section 7A.2, MP1c).
//!
//! A block that "follows" a constant reads it here, not from
//! [`crate::quantity::consts`]. With nothing edited the handle holds exactly
//! the compiled values, bit for bit, so every result and golden stays the
//! same. The kernel sets the stack's values for the length of one call
//! with [`with`]; outside a call the compiled values hold.
//!
//! The handle is per thread and always restored, so a call cannot leak an
//! edited constant into the next one.

use crate::quantity::consts as k;
use std::cell::Cell;

/// The constants a "follows" block may read, in SI.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Constants {
    /// Newtonian constant of gravitation, m3 kg-1 s-2.
    pub g: f64,
    /// Speed of light, m s-1.
    pub c: f64,
    /// Stefan-Boltzmann constant, W m-2 K-4.
    pub sigma_sb: f64,
    /// Boltzmann constant, J K-1.
    pub k_b: f64,
    /// The Cosmos group (DM1, galaxy histories design section 3.2): the
    /// measured parameters of the universe, read by the cosmos chain.
    pub cosmos: Cosmos,
}

/// The Cosmos group's values in force (Planck 2018 VI, table 2,
/// TT,TE,EE+lowE+lensing+BAO; the sheet's `cosmos` rows). Dark matter and
/// dark energy are the sheet's "theoretical" rows: their amounts are
/// measured through gravity, their nature is unknown.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Cosmos {
    /// Hubble constant, km s-1 Mpc-1 (as astronomers quote it).
    pub h0: f64,
    /// Ordinary matter today, Omega_b.
    pub omega_b: f64,
    /// Dark matter today, Omega_c (theoretical).
    pub omega_c: f64,
    /// Massive neutrinos today, Omega_nu.
    pub omega_nu: f64,
    /// All matter, Omega_m = Omega_b + Omega_c + Omega_nu (worked out on the sheet).
    pub omega_m: f64,
    /// Fluctuation size on 8 Mpc/h, sigma_8.
    pub sigma8: f64,
    /// Tilt of the primordial spectrum, n_s.
    pub n_s: f64,
    /// Background temperature today, K.
    pub t_cmb: f64,
    /// Effective number of neutrino species.
    pub n_eff: f64,
    /// Dark energy pressure today and its drift, w = w0 + wa (1 - a) (theoretical).
    pub w0: f64,
    pub wa: f64,
}

impl Cosmos {
    /// The compiled values, equal to the base sheet's rows bit for bit.
    pub const REAL: Cosmos = Cosmos {
        h0: 67.66,
        omega_b: 0.048_97,
        omega_c: 0.260_722_8,
        omega_nu: 0.001_407_2,
        omega_m: 0.3111,
        sigma8: 0.8102,
        n_s: 0.9665,
        t_cmb: 2.7255,
        n_eff: 3.046,
        w0: -1.0,
        wa: 0.0,
    };
}

impl Constants {
    /// The compiled values (CODATA 2018, as `quantity::consts` and
    /// `sse-atoms` hold them today).
    pub const REAL: Constants = Constants {
        g: k::G,
        c: k::C,
        sigma_sb: k::STEFAN_BOLTZMANN,
        k_b: 1.380_649e-23,
        cosmos: Cosmos::REAL,
    };

    /// The sheet's id for each field, for the reader and the reach report.
    pub const IDS: [&'static str; 15] = [
        "G", "c", "sigma_SB", "k_B", "H0", "Omega_b", "Omega_c", "Omega_nu", "Omega_m", "sigma8",
        "n_s", "T_cmb", "N_eff", "w0", "wa",
    ];

    pub fn get(&self, id: &str) -> Option<f64> {
        match id {
            "G" => Some(self.g),
            "c" => Some(self.c),
            "sigma_SB" => Some(self.sigma_sb),
            "k_B" => Some(self.k_b),
            "H0" => Some(self.cosmos.h0),
            "Omega_b" => Some(self.cosmos.omega_b),
            "Omega_c" => Some(self.cosmos.omega_c),
            "Omega_nu" => Some(self.cosmos.omega_nu),
            "Omega_m" => Some(self.cosmos.omega_m),
            "sigma8" => Some(self.cosmos.sigma8),
            "n_s" => Some(self.cosmos.n_s),
            "T_cmb" => Some(self.cosmos.t_cmb),
            "N_eff" => Some(self.cosmos.n_eff),
            "w0" => Some(self.cosmos.w0),
            "wa" => Some(self.cosmos.wa),
            _ => None,
        }
    }

    pub fn set(&mut self, id: &str, v: f64) -> bool {
        match id {
            "G" => self.g = v,
            "c" => self.c = v,
            "sigma_SB" => self.sigma_sb = v,
            "k_B" => self.k_b = v,
            "H0" => self.cosmos.h0 = v,
            "Omega_b" => self.cosmos.omega_b = v,
            "Omega_c" => self.cosmos.omega_c = v,
            "Omega_nu" => self.cosmos.omega_nu = v,
            "Omega_m" => self.cosmos.omega_m = v,
            "sigma8" => self.cosmos.sigma8 = v,
            "n_s" => self.cosmos.n_s = v,
            "T_cmb" => self.cosmos.t_cmb = v,
            "N_eff" => self.cosmos.n_eff = v,
            "w0" => self.cosmos.w0 = v,
            "wa" => self.cosmos.wa = v,
            _ => return false,
        }
        true
    }

    /// Whether any value differs from the real one.
    pub fn edited(&self) -> bool {
        *self != Self::REAL
    }
}

impl Default for Constants {
    fn default() -> Self {
        Self::REAL
    }
}

thread_local! {
    static NOW: Cell<Constants> = const { Cell::new(Constants::REAL) };
}

/// The constants in force.
pub fn now() -> Constants {
    NOW.with(Cell::get)
}

/// Newtonian G in force.
pub fn g() -> f64 {
    now().g
}

/// Stefan-Boltzmann constant in force.
pub fn sigma_sb() -> f64 {
    now().sigma_sb
}

/// Boltzmann constant in force.
pub fn k_b() -> f64 {
    now().k_b
}

/// Speed of light in force.
pub fn c() -> f64 {
    now().c
}

/// The Cosmos group in force.
pub fn cosmos() -> Cosmos {
    now().cosmos
}

/// Run `f` with `c` in force, then restore what was there (also on a panic).
pub fn with<R>(c: Constants, f: impl FnOnce() -> R) -> R {
    struct Restore(Constants);
    impl Drop for Restore {
        fn drop(&mut self) {
            NOW.with(|n| n.set(self.0));
        }
    }
    let _r = Restore(NOW.with(|n| n.replace(c)));
    f()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn with_sets_and_restores() {
        assert_eq!(now(), Constants::REAL);
        let mut c = Constants::REAL;
        c.g *= 2.0;
        let inner = with(c, g);
        assert_eq!(inner, 2.0 * k::G);
        assert_eq!(g(), k::G);
        let r = std::panic::catch_unwind(|| with(c, || panic!("boom")));
        assert!(r.is_err());
        assert_eq!(g(), k::G);
    }
}

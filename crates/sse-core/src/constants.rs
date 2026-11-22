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
}

impl Constants {
    /// The compiled values (CODATA 2018, as `quantity::consts` and
    /// `sse-atoms` hold them today).
    pub const REAL: Constants = Constants {
        g: k::G,
        c: k::C,
        sigma_sb: k::STEFAN_BOLTZMANN,
        k_b: 1.380_649e-23,
    };

    /// The sheet's id for each field, for the reader and the reach report.
    pub const IDS: [&'static str; 4] = ["G", "c", "sigma_SB", "k_B"];

    pub fn get(&self, id: &str) -> Option<f64> {
        match id {
            "G" => Some(self.g),
            "c" => Some(self.c),
            "sigma_SB" => Some(self.sigma_sb),
            "k_B" => Some(self.k_b),
            _ => None,
        }
    }

    pub fn set(&mut self, id: &str, v: f64) -> bool {
        match id {
            "G" => self.g = v,
            "c" => self.c = v,
            "sigma_SB" => self.sigma_sb = v,
            "k_B" => self.k_b = v,
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

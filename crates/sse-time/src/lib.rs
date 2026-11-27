//! Time (handoff 6.3; build plan T1 and T2).
#![forbid(unsafe_code)]
// Numerical code: index loops mirror the published equations.
#![allow(clippy::needless_range_loop)]

pub mod binary;
pub mod branch;
pub mod earth;
pub mod epoch;
pub mod generated;
pub mod kepler;
pub mod nbody;
pub mod pair;
pub mod scenario;
pub mod secular;
pub mod series;
pub mod star;

#[cfg(test)]
mod binary_tests;
#[cfg(test)]
mod generated_tests;
#[cfg(test)]
mod pair_tests;
#[cfg(test)]
mod series_tests;
#[cfg(test)]
mod tests;

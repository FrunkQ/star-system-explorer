//! Pinned maths (H3). Every transcendental function used on a deterministic
//! path goes through here, and here only calls `libm` (exact version pinned in
//! the workspace). `+ - * /` and `sqrt` are correctly rounded by IEEE 754 on
//! every target, so they are used directly. Rust never contracts `a*b+c` into
//! a fused multiply-add on its own, and the workspace forbids `mul_add`.
//!
//! Clippy (clippy.toml) rejects the std `f64::sin` family so nothing bypasses
//! this module by accident.

macro_rules! unary {
    ($($name:ident),* $(,)?) => { $(
        #[inline] pub fn $name(x: f64) -> f64 { libm::$name(x) }
    )* };
}
unary!(
    sin, cos, tan, asin, acos, atan, exp, exp2, expm1, log, log2, log10, log1p, cbrt, sinh, cosh,
    tanh, asinh, acosh, atanh, floor, ceil, round, trunc
);

/// Natural logarithm (alias of `log`, for readability in formulas).
#[inline]
pub fn ln(x: f64) -> f64 {
    libm::log(x)
}
#[inline]
pub fn pow(x: f64, y: f64) -> f64 {
    libm::pow(x, y)
}
#[inline]
pub fn atan2(y: f64, x: f64) -> f64 {
    libm::atan2(y, x)
}
#[inline]
pub fn hypot(x: f64, y: f64) -> f64 {
    libm::hypot(x, y)
}
#[inline]
pub fn fmod(x: f64, y: f64) -> f64 {
    libm::fmod(x, y)
}
/// Square root: IEEE correctly rounded on every target.
#[inline]
pub fn sqrt(x: f64) -> f64 {
    libm::sqrt(x)
}

pub const PI: f64 = core::f64::consts::PI;
pub const TAU: f64 = core::f64::consts::TAU;

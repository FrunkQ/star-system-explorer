//! Quantities, dimensions and units (H5.2).
//!
//! Values are stored in SI base units. A [`Dimension`] is the vector of SI base
//! exponents; arithmetic checks it. Display units (AU, solar masses, bar, ...)
//! are [`Unit`]s with an exact scale to SI, chosen by packs.
//!
//! Constants and their sources (checked 18 Nov 2026):
//! - au = 149 597 870 700 m exactly: IAU 2012 Resolution B2.
//! - Nominal solar radius 6.957e8 m, luminosity 3.828e26 W, GM_sun
//!   1.3271244e20 m^3 s^-2, Earth equatorial radius 6.3781e6 m,
//!   GM_earth 3.986004e14 m^3 s^-2, Jupiter equatorial radius 7.1492e7 m,
//!   GM_jupiter 1.2668653e17 m^3 s^-2: IAU 2015 Resolution B3 (nominal values).
//! - G = 6.67430e-11 m^3 kg^-1 s^-2: CODATA 2018. Masses below are GM / G.
//! - c = 299 792 458 m/s exactly (SI definition); Julian year 365.25 d of 86400 s.
//! - parsec = 648000/pi au: IAU 2015 Resolution B2.
//! - bar = 1e5 Pa exactly.

// add/sub return Result (dimension check), so they cannot be the std operator traits.
#![allow(clippy::should_implement_trait)]

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt;

/// SI base exponents: metre, kilogram, second, ampere, kelvin, mole, candela.
#[derive(
    Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize, Default,
)]
pub struct Dimension(pub [i8; 7]);

const BASE: [&str; 7] = ["m", "kg", "s", "A", "K", "mol", "cd"];

impl Dimension {
    pub const NONE: Dimension = Dimension([0; 7]);
    pub const LENGTH: Dimension = Dimension([1, 0, 0, 0, 0, 0, 0]);
    pub const MASS: Dimension = Dimension([0, 1, 0, 0, 0, 0, 0]);
    pub const TIME: Dimension = Dimension([0, 0, 1, 0, 0, 0, 0]);
    pub const TEMPERATURE: Dimension = Dimension([0, 0, 0, 0, 1, 0, 0]);
    pub const POWER: Dimension = Dimension([2, 1, -3, 0, 0, 0, 0]);
    pub const PRESSURE: Dimension = Dimension([-1, 1, -2, 0, 0, 0, 0]);
    pub const FLUX: Dimension = Dimension([0, 1, -3, 0, 0, 0, 0]); // W m^-2
    pub const VELOCITY: Dimension = Dimension([1, 0, -1, 0, 0, 0, 0]);
    pub const ACCELERATION: Dimension = Dimension([1, 0, -2, 0, 0, 0, 0]);
    pub const DENSITY: Dimension = Dimension([-3, 1, 0, 0, 0, 0, 0]);
    pub const GRAV_PARAM: Dimension = Dimension([3, 0, -2, 0, 0, 0, 0]);

    pub fn mul(self, o: Dimension) -> Dimension {
        let mut r = [0i8; 7];
        for (i, x) in r.iter_mut().enumerate() {
            *x = self.0[i] + o.0[i];
        }
        Dimension(r)
    }
    pub fn div(self, o: Dimension) -> Dimension {
        let mut r = [0i8; 7];
        for (i, x) in r.iter_mut().enumerate() {
            *x = self.0[i] - o.0[i];
        }
        Dimension(r)
    }
    pub fn powi(self, n: i8) -> Dimension {
        Dimension(self.0.map(|x| x * n))
    }
    /// Integer root, if every exponent divides.
    pub fn root(self, n: i8) -> Option<Dimension> {
        self.0
            .iter()
            .all(|x| x % n == 0)
            .then(|| Dimension(self.0.map(|x| x / n)))
    }
}

impl fmt::Display for Dimension {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if *self == Dimension::NONE {
            return f.write_str("1");
        }
        let parts: Vec<String> = self
            .0
            .iter()
            .zip(BASE)
            .filter(|(e, _)| **e != 0)
            .map(|(e, b)| {
                if *e == 1 {
                    b.to_string()
                } else {
                    format!("{b}^{e}")
                }
            })
            .collect();
        f.write_str(&parts.join(" "))
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum QuantityError {
    DimensionMismatch {
        expected: Dimension,
        found: Dimension,
        context: String,
    },
    UnknownUnit(String),
    NotFinite(String),
    NoRoot(Dimension),
}
impl fmt::Display for QuantityError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            QuantityError::DimensionMismatch {
                expected,
                found,
                context,
            } => {
                write!(f, "{context}: expected [{expected}], found [{found}]")
            }
            QuantityError::UnknownUnit(u) => write!(f, "unknown unit {u:?}"),
            QuantityError::NotFinite(c) => write!(f, "{c}: value is not finite"),
            QuantityError::NoRoot(d) => write!(f, "no integer root of [{d}]"),
        }
    }
}
impl std::error::Error for QuantityError {}

/// A value in SI base units with its dimension.
#[derive(Clone, Copy, PartialEq, Debug, Serialize, Deserialize)]
pub struct Quantity {
    pub value: f64,
    pub dim: Dimension,
}

impl Quantity {
    pub fn si(value: f64, dim: Dimension) -> Self {
        Quantity { value, dim }
    }
    pub fn scalar(value: f64) -> Self {
        Quantity {
            value,
            dim: Dimension::NONE,
        }
    }
    pub fn expect(&self, dim: Dimension, context: &str) -> Result<f64, QuantityError> {
        if self.dim != dim {
            return Err(QuantityError::DimensionMismatch {
                expected: dim,
                found: self.dim,
                context: context.into(),
            });
        }
        Ok(self.value)
    }
    pub fn add(self, o: Quantity) -> Result<Quantity, QuantityError> {
        o.expect(self.dim, "add")?;
        Ok(Quantity::si(self.value + o.value, self.dim))
    }
    pub fn sub(self, o: Quantity) -> Result<Quantity, QuantityError> {
        o.expect(self.dim, "sub")?;
        Ok(Quantity::si(self.value - o.value, self.dim))
    }
    pub fn mul(self, o: Quantity) -> Quantity {
        Quantity::si(self.value * o.value, self.dim.mul(o.dim))
    }
    pub fn div(self, o: Quantity) -> Quantity {
        Quantity::si(self.value / o.value, self.dim.div(o.dim))
    }
    pub fn powi(self, n: i8) -> Quantity {
        Quantity::si(crate::math::pow(self.value, n as f64), self.dim.powi(n))
    }
    pub fn sqrt(self) -> Result<Quantity, QuantityError> {
        let d = self.dim.root(2).ok_or(QuantityError::NoRoot(self.dim))?;
        Ok(Quantity::si(crate::math::sqrt(self.value), d))
    }
    pub fn is_finite(&self) -> bool {
        self.value.is_finite()
    }
}

/// A display unit: `si = value * scale + offset`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Unit {
    pub id: String,
    pub symbol: String,
    pub dim: Dimension,
    pub scale: f64,
    #[serde(default)]
    pub offset: f64,
}

/// The units known to a session. Packs add to it (K5).
#[derive(Clone, Debug, Default)]
pub struct Units {
    by_id: BTreeMap<String, Unit>,
}

pub mod consts {
    pub const G: f64 = 6.674_30e-11;
    pub const C: f64 = 299_792_458.0;
    pub const AU: f64 = 149_597_870_700.0;
    pub const DAY: f64 = 86_400.0;
    pub const JULIAN_YEAR: f64 = 365.25 * DAY;
    pub const GM_SUN: f64 = 1.327_124_4e20;
    pub const GM_EARTH: f64 = 3.986_004e14;
    pub const GM_JUPITER: f64 = 1.266_865_3e17;
    pub const R_SUN: f64 = 6.957e8;
    pub const L_SUN: f64 = 3.828e26;
    pub const R_EARTH: f64 = 6.378_1e6;
    pub const R_JUPITER: f64 = 7.149_2e7;
    pub const STEFAN_BOLTZMANN: f64 = 5.670_374_419e-8; // CODATA 2018, derived exactly from h, k, c
}

impl Units {
    pub fn get(&self, id: &str) -> Result<&Unit, QuantityError> {
        self.by_id
            .get(id)
            .ok_or_else(|| QuantityError::UnknownUnit(id.into()))
    }
    pub fn insert(&mut self, u: Unit) {
        self.by_id.insert(u.id.clone(), u);
    }
    pub fn ids(&self) -> impl Iterator<Item = &str> {
        self.by_id.keys().map(String::as_str)
    }
    /// Make a quantity from a number in a named unit.
    pub fn q(&self, value: f64, unit: &str) -> Result<Quantity, QuantityError> {
        let u = self.get(unit)?;
        Ok(Quantity::si(value * u.scale + u.offset, u.dim))
    }
    /// Express a quantity in a named unit; dimension must match.
    pub fn to(&self, q: Quantity, unit: &str) -> Result<f64, QuantityError> {
        let u = self.get(unit)?;
        q.expect(u.dim, &format!("convert to {unit}"))?;
        Ok((q.value - u.offset) / u.scale)
    }

    /// SI base units plus the common astronomical display units.
    pub fn standard() -> Self {
        use consts::*;
        use Dimension as D;
        let mut s = Units::default();
        let mut add = |id: &str, symbol: &str, dim: D, scale: f64, offset: f64| {
            s.insert(Unit {
                id: id.into(),
                symbol: symbol.into(),
                dim,
                scale,
                offset,
            })
        };
        add("one", "", D::NONE, 1.0, 0.0);
        add("m", "m", D::LENGTH, 1.0, 0.0);
        add("km", "km", D::LENGTH, 1e3, 0.0);
        add("au", "au", D::LENGTH, AU, 0.0);
        add("pc", "pc", D::LENGTH, 648_000.0 / crate::math::PI * AU, 0.0);
        add("ly", "ly", D::LENGTH, C * JULIAN_YEAR, 0.0);
        add("r_sun", "R☉", D::LENGTH, R_SUN, 0.0);
        add("r_earth", "R⊕", D::LENGTH, R_EARTH, 0.0);
        add("r_jupiter", "RJ", D::LENGTH, R_JUPITER, 0.0);
        add("kg", "kg", D::MASS, 1.0, 0.0);
        add("m_sun", "M☉", D::MASS, GM_SUN / G, 0.0);
        add("m_earth", "M⊕", D::MASS, GM_EARTH / G, 0.0);
        add("m_jupiter", "MJ", D::MASS, GM_JUPITER / G, 0.0);
        add("s", "s", D::TIME, 1.0, 0.0);
        add("day", "d", D::TIME, DAY, 0.0);
        add("yr", "yr", D::TIME, JULIAN_YEAR, 0.0);
        add("myr", "Myr", D::TIME, 1e6 * JULIAN_YEAR, 0.0);
        add("gyr", "Gyr", D::TIME, 1e9 * JULIAN_YEAR, 0.0);
        add("k", "K", D::TEMPERATURE, 1.0, 0.0);
        add("degc", "°C", D::TEMPERATURE, 1.0, 273.15);
        add("w", "W", D::POWER, 1.0, 0.0);
        add("l_sun", "L☉", D::POWER, L_SUN, 0.0);
        add("w_m2", "W/m²", D::FLUX, 1.0, 0.0);
        add("pa", "Pa", D::PRESSURE, 1.0, 0.0);
        add("bar", "bar", D::PRESSURE, 1e5, 0.0);
        add("m_s", "m/s", D::VELOCITY, 1.0, 0.0);
        add("km_s", "km/s", D::VELOCITY, 1e3, 0.0);
        add("m_s2", "m/s²", D::ACCELERATION, 1.0, 0.0);
        add("kg_m3", "kg/m³", D::DENSITY, 1.0, 0.0);
        add("g_cm3", "g/cm³", D::DENSITY, 1e3, 0.0);
        s
    }
}

/// A quantity kind: what a flow wire carries (H5.2).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct QuantityKind {
    pub id: String,
    pub dim: Dimension,
    /// Physically valid range in SI, if bounded.
    #[serde(default)]
    pub range: Option<(f64, f64)>,
    pub description: String,
}

impl QuantityKind {
    /// Check a value against this kind: dimension must match and value be finite.
    pub fn check(&self, q: &Quantity) -> Result<(), QuantityError> {
        q.expect(self.dim, &self.id)?;
        if !q.is_finite() {
            return Err(QuantityError::NotFinite(self.id.clone()));
        }
        Ok(())
    }
    pub fn in_range(&self, q: &Quantity) -> bool {
        self.range
            .is_none_or(|(lo, hi)| q.value >= lo && q.value <= hi)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn units_and_mismatch() {
        let u = Units::standard();
        let one_au = u.q(1.0, "au").unwrap();
        assert_eq!(one_au.value, 149_597_870_700.0);
        assert_eq!(u.to(u.q(0.0, "degc").unwrap(), "k").unwrap(), 273.15);
        let m = u.q(1.0, "m_sun").unwrap();
        assert!((m.value - 1.988_409_87e30).abs() < 1e23, "{}", m.value);
        assert!(one_au.add(m).is_err());
        assert!(u.to(m, "au").is_err());
        let area = one_au.mul(one_au);
        assert_eq!(area.sqrt().unwrap().dim, Dimension::LENGTH);
        assert!(m.sqrt().is_err());
        let pc = u.to(u.q(1.0, "pc").unwrap(), "ly").unwrap();
        assert!((pc - 3.261_563_777).abs() < 1e-8, "{pc}");
        assert_eq!(
            Dimension::POWER.div(Dimension::LENGTH.powi(2)),
            Dimension::FLUX
        );
        assert_eq!(Dimension::FLUX.to_string(), "kg s^-3");
    }

    proptest! {
        #[test]
        fn conversions_round_trip(v in -1e6f64..1e6, i in 0usize..30) {
            let u = Units::standard();
            let id: Vec<&str> = u.ids().collect();
            let id = id[i % id.len()];
            let q = u.q(v, id).unwrap();
            let back = u.to(q, id).unwrap();
            prop_assert!((back - v).abs() <= 1e-9 * v.abs().max(1.0));
        }

        #[test]
        fn kind_rejects_wrong_dimension(e in proptest::array::uniform7(-3i8..3)) {
            let k = QuantityKind { id: "planet.radius".into(), dim: Dimension::LENGTH, range: Some((0.0, 1e9)), description: String::new() };
            let q = Quantity::si(1.0, Dimension(e));
            prop_assert_eq!(k.check(&q).is_ok(), Dimension(e) == Dimension::LENGTH);
        }
    }
}

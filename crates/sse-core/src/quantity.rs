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
    /// Tesla: kg s^-2 A^-1.
    pub const MAGNETIC_FIELD: Dimension = Dimension([0, 1, -2, -1, 0, 0, 0]);
    /// A m² (magnetic dipole moment).
    pub const MAGNETIC_MOMENT: Dimension = Dimension([2, 0, 0, 1, 0, 0, 0]);

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

/// A unit written as text, as an Equation's ports and the sheet write them:
/// symbols with an optional power (`m3`, `m-3`, `m^3`), separated by spaces
/// or `*`, and an optional `/` after which every symbol divides (`g/cm3`,
/// `eV cm3/g`, `W m-2 K-4`, `m3 kg-1 s-2`). `1` or nothing is a pure number.
/// SI symbols take the prefixes G M k c m u µ n p f; a few others are known
/// by name (g, eV, erg, bar, au, pc, ly, yr, day). Returns the scale to SI
/// and the dimension. Units with an offset (degrees Celsius) are not units
/// here. The scale is built by plain multiplication, the same on every device.
pub fn parse_unit(text: &str) -> Result<(f64, Dimension), String> {
    use consts::*;
    use Dimension as D;
    const A: D = D([0, 0, 0, 1, 0, 0, 0]);
    const MOL: D = D([0, 0, 0, 0, 0, 1, 0]);
    const CD: D = D([0, 0, 0, 0, 0, 0, 1]);
    const ENERGY: D = D([2, 1, -2, 0, 0, 0, 0]);
    // (symbol, scale, dimension, takes prefixes)
    let base: [(&str, f64, D, bool); 28] = [
        ("m", 1.0, D::LENGTH, true),
        ("g", 1e-3, D::MASS, true),
        ("s", 1.0, D::TIME, true),
        ("A", 1.0, A, true),
        ("K", 1.0, D::TEMPERATURE, true),
        ("mol", 1.0, MOL, true),
        ("cd", 1.0, CD, false),
        ("J", 1.0, ENERGY, true),
        ("W", 1.0, D::POWER, true),
        ("N", 1.0, D([1, 1, -2, 0, 0, 0, 0]), true),
        ("Pa", 1.0, D::PRESSURE, true),
        ("Hz", 1.0, D([0, 0, -1, 0, 0, 0, 0]), true),
        ("C", 1.0, D([0, 0, 1, 1, 0, 0, 0]), true),
        ("V", 1.0, D([2, 1, -3, -1, 0, 0, 0]), true),
        ("S", 1.0, D([-2, -1, 3, 2, 0, 0, 0]), true),
        ("T", 1.0, D::MAGNETIC_FIELD, true),
        ("H", 1.0, D([2, 1, -2, -2, 0, 0, 0]), true),
        ("F", 1.0, D([-2, -1, 4, 2, 0, 0, 0]), true),
        ("Wb", 1.0, D([2, 1, -2, -1, 0, 0, 0]), true),
        ("ohm", 1.0, D([2, 1, -3, -2, 0, 0, 0]), true),
        ("eV", crate::sheet::E, ENERGY, true),
        ("erg", 1e-7, ENERGY, false),
        ("bar", 1e5, D::PRESSURE, true),
        ("au", AU, D::LENGTH, false),
        ("pc", 648_000.0 / crate::math::PI * AU, D::LENGTH, true),
        ("ly", C * JULIAN_YEAR, D::LENGTH, false),
        ("yr", JULIAN_YEAR, D::TIME, true),
        ("day", DAY, D::TIME, false),
    ];
    let prefixes: [(&str, f64); 11] = [
        ("G", 1e9),
        ("M", 1e6),
        ("k", 1e3),
        ("c", 1e-2),
        ("m", 1e-3),
        ("u", 1e-6),
        ("µ", 1e-6),
        ("n", 1e-9),
        ("p", 1e-12),
        ("f", 1e-15),
        ("", 1.0),
    ];
    let find = |sym: &str| -> Option<(f64, D)> {
        if let Some(b) = base.iter().find(|b| b.0 == sym) {
            return Some((b.1, b.2));
        }
        for (p, f) in prefixes.iter().filter(|p| !p.0.is_empty()) {
            if let Some(rest) = sym.strip_prefix(p) {
                if let Some(b) = base.iter().find(|b| b.0 == rest && b.3) {
                    return Some((f * b.1, b.2));
                }
            }
        }
        None
    };
    let (num, den) = match text.split_once('/') {
        Some((_, b)) if b.contains('/') => {
            return Err(format!("unit {text:?}: write one `/` at most"));
        }
        Some((a, b)) => (a, b),
        None => (text, ""),
    };
    let mut scale = 1.0;
    let mut dim = D::NONE;
    for (part, sign) in [(num, 1i8), (den, -1i8)] {
        for tok in part.split(|c: char| c.is_whitespace() || c == '*' || c == '·') {
            if tok.is_empty() || tok == "1" {
                continue;
            }
            let split = tok
                .find(|c: char| c == '-' || c == '^' || c.is_ascii_digit())
                .unwrap_or(tok.len());
            let (sym, exp) = tok.split_at(split);
            let exp = exp.strip_prefix('^').unwrap_or(exp);
            let n: i8 = if exp.is_empty() {
                1
            } else {
                exp.parse()
                    .map_err(|_| format!("unit {text:?}: cannot read the power in {tok:?}"))?
            };
            let (f, d) = find(sym)
                .ok_or_else(|| format!("unit {text:?}: {sym:?} is not a unit this reader knows"))?;
            let n = n * sign;
            dim = dim.mul(d.powi(n));
            for _ in 0..n.unsigned_abs() {
                if n > 0 {
                    scale *= f;
                } else {
                    scale /= f;
                }
            }
        }
    }
    Ok((scale, dim))
}

/// A dimension written as a unit [`parse_unit`] reads back (`m3 kg-1 s-2`).
pub fn si_unit(d: Dimension) -> String {
    if d == Dimension::NONE {
        return "1".into();
    }
    d.0.iter()
        .zip(BASE)
        .filter(|(e, _)| **e != 0)
        .map(|(e, b)| {
            if *e == 1 {
                b.to_string()
            } else {
                format!("{b}{e}")
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
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

/// The kernel's names for the sheet's constants (generated into `crate::sheet`
/// from packs/base-reference; MP22). No number is typed here.
pub mod consts {
    use crate::sheet as s;
    pub const G: f64 = s::G;
    pub const C: f64 = s::C;
    pub const AU: f64 = s::AU;
    pub const DAY: f64 = s::DAY;
    pub const JULIAN_YEAR: f64 = s::YR_JULIAN;
    pub const GM_SUN: f64 = s::GM_SUN;
    pub const GM_EARTH: f64 = s::GM_EARTH;
    pub const GM_JUPITER: f64 = s::GM_JUP;
    pub const R_SUN: f64 = s::R_SUN;
    pub const L_SUN: f64 = s::L_SUN;
    pub const R_EARTH: f64 = s::R_EARTH_EQUATORIAL;
    pub const R_JUPITER: f64 = s::R_JUP_EQUATORIAL;
    pub const STEFAN_BOLTZMANN: f64 = s::SIGMA_SB;
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
        add("degc", "°C", D::TEMPERATURE, 1.0, crate::sheet::T0C);
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
        add("t", "T", D::MAGNETIC_FIELD, 1.0, 0.0);
        add("ut", "µT", D::MAGNETIC_FIELD, 1e-6, 0.0);
        add("gauss", "G", D::MAGNETIC_FIELD, 1e-4, 0.0);
        add("a_m2", "A m²", D::MAGNETIC_MOMENT, 1.0, 0.0);
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

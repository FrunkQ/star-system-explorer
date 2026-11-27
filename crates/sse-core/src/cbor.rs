//! Canonical CBOR, SSE4 profile (docs/adr/0003-canonical-encoding.md).
//!
//! The profile is a strict subset of RFC 8949 section 4.2 "core deterministic
//! encoding", chosen so that one value has exactly one byte form:
//!
//! - integers in the shortest head; range -2^64 ..= 2^64-1
//! - definite lengths only
//! - map keys are text strings only, sorted by their encoded bytes
//!   (length first, then bytewise), no duplicates
//! - floats are always 64-bit (major 7, additional info 27); NaN is rejected;
//!   -0.0 and 0.0 are distinct values
//! - no tags, no undefined, no simple values other than false/true/null
//!
//! The decoder rejects anything that is not in canonical form, so
//! `encode(decode(b)) == b` for every accepted `b`.

use std::collections::BTreeMap;
use std::fmt;

/// A canonical CBOR value. Maps are keyed by text; ordering on encode is the
/// canonical one, independent of the order keys were inserted.
#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Null,
    Bool(bool),
    /// Range -2^64 ..= 2^64 - 1.
    Int(i128),
    Float(f64),
    Bytes(Vec<u8>),
    Text(String),
    Array(Vec<Value>),
    Map(BTreeMap<String, Value>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CborError {
    NaN,
    IntOutOfRange,
    NonTextKey,
    Truncated,
    TrailingBytes,
    NonCanonical(&'static str),
    Unsupported(&'static str),
    InvalidUtf8,
    TooDeep,
    Serde(String),
}

impl fmt::Display for CborError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for CborError {}

const MAX_DEPTH: usize = 256;
const INT_MIN: i128 = -(1i128 << 64);
const INT_MAX: i128 = (1i128 << 64) - 1;

fn head(out: &mut Vec<u8>, major: u8, n: u64) {
    let m = major << 5;
    if n < 24 {
        out.push(m | n as u8);
    } else if n <= 0xff {
        out.push(m | 24);
        out.push(n as u8);
    } else if n <= 0xffff {
        out.push(m | 25);
        out.extend_from_slice(&(n as u16).to_be_bytes());
    } else if n <= 0xffff_ffff {
        out.push(m | 26);
        out.extend_from_slice(&(n as u32).to_be_bytes());
    } else {
        out.push(m | 27);
        out.extend_from_slice(&n.to_be_bytes());
    }
}

/// Encode a value in canonical form.
pub fn encode(v: &Value) -> Result<Vec<u8>, CborError> {
    let mut out = Vec::new();
    encode_into(v, &mut out, 0)?;
    Ok(out)
}

fn encode_into(v: &Value, out: &mut Vec<u8>, depth: usize) -> Result<(), CborError> {
    if depth > MAX_DEPTH {
        return Err(CborError::TooDeep);
    }
    match v {
        Value::Null => out.push(0xf6),
        Value::Bool(false) => out.push(0xf4),
        Value::Bool(true) => out.push(0xf5),
        Value::Int(i) => {
            if *i < INT_MIN || *i > INT_MAX {
                return Err(CborError::IntOutOfRange);
            }
            if *i >= 0 {
                head(out, 0, *i as u64);
            } else {
                head(out, 1, (-1 - *i) as u64);
            }
        }
        Value::Float(f) => {
            if f.is_nan() {
                return Err(CborError::NaN);
            }
            out.push(0xfb);
            out.extend_from_slice(&f.to_bits().to_be_bytes());
        }
        Value::Bytes(b) => {
            head(out, 2, b.len() as u64);
            out.extend_from_slice(b);
        }
        Value::Text(s) => {
            head(out, 3, s.len() as u64);
            out.extend_from_slice(s.as_bytes());
        }
        Value::Array(a) => {
            head(out, 4, a.len() as u64);
            for x in a {
                encode_into(x, out, depth + 1)?;
            }
        }
        Value::Map(m) => {
            head(out, 5, m.len() as u64);
            // Canonical key order: encoded key bytes, i.e. shorter keys first,
            // then bytewise. BTreeMap gives bytewise order; re-sort by length.
            let mut keys: Vec<&String> = m.keys().collect();
            keys.sort_by(|a, b| {
                a.len()
                    .cmp(&b.len())
                    .then_with(|| a.as_bytes().cmp(b.as_bytes()))
            });
            for k in keys {
                head(out, 3, k.len() as u64);
                out.extend_from_slice(k.as_bytes());
                encode_into(&m[k], out, depth + 1)?;
            }
        }
    }
    Ok(())
}

struct Reader<'a> {
    b: &'a [u8],
    p: usize,
}

impl<'a> Reader<'a> {
    fn take(&mut self, n: usize) -> Result<&'a [u8], CborError> {
        if self.b.len() - self.p < n {
            return Err(CborError::Truncated);
        }
        let s = &self.b[self.p..self.p + n];
        self.p += n;
        Ok(s)
    }
    fn byte(&mut self) -> Result<u8, CborError> {
        Ok(self.take(1)?[0])
    }
    /// Read the argument of a head, enforcing shortest form.
    fn arg(&mut self, ai: u8) -> Result<u64, CborError> {
        let n = match ai {
            0..=23 => return Ok(ai as u64),
            24 => {
                let n = self.byte()? as u64;
                if n < 24 {
                    return Err(CborError::NonCanonical("int head not shortest"));
                }
                n
            }
            25 => {
                let n = u16::from_be_bytes(self.take(2)?.try_into().unwrap()) as u64;
                if n <= 0xff {
                    return Err(CborError::NonCanonical("int head not shortest"));
                }
                n
            }
            26 => {
                let n = u32::from_be_bytes(self.take(4)?.try_into().unwrap()) as u64;
                if n <= 0xffff {
                    return Err(CborError::NonCanonical("int head not shortest"));
                }
                n
            }
            27 => {
                let n = u64::from_be_bytes(self.take(8)?.try_into().unwrap());
                if n <= 0xffff_ffff {
                    return Err(CborError::NonCanonical("int head not shortest"));
                }
                n
            }
            31 => return Err(CborError::NonCanonical("indefinite length")),
            _ => return Err(CborError::Unsupported("reserved additional info")),
        };
        Ok(n)
    }
    fn len(&mut self, ai: u8) -> Result<usize, CborError> {
        let n = self.arg(ai)?;
        if n > (self.b.len() - self.p) as u64 {
            // every element takes at least one byte
            return Err(CborError::Truncated);
        }
        Ok(n as usize)
    }
    fn value(&mut self, depth: usize) -> Result<Value, CborError> {
        if depth > MAX_DEPTH {
            return Err(CborError::TooDeep);
        }
        let ib = self.byte()?;
        let (major, ai) = (ib >> 5, ib & 0x1f);
        Ok(match major {
            0 => Value::Int(self.arg(ai)? as i128),
            1 => Value::Int(-1 - self.arg(ai)? as i128),
            2 => {
                let n = self.len(ai)?;
                Value::Bytes(self.take(n)?.to_vec())
            }
            3 => {
                let n = self.len(ai)?;
                let s = std::str::from_utf8(self.take(n)?).map_err(|_| CborError::InvalidUtf8)?;
                Value::Text(s.to_owned())
            }
            4 => {
                let n = self.len(ai)?;
                let mut a = Vec::with_capacity(n);
                for _ in 0..n {
                    a.push(self.value(depth + 1)?);
                }
                Value::Array(a)
            }
            5 => {
                let n = self.len(ai)?;
                let mut m = BTreeMap::new();
                let mut prev: Option<Vec<u8>> = None;
                for _ in 0..n {
                    let kstart = self.p;
                    let k = match self.value(depth + 1)? {
                        Value::Text(s) => s,
                        _ => return Err(CborError::NonTextKey),
                    };
                    let kbytes = self.b[kstart..self.p].to_vec();
                    if let Some(p) = &prev {
                        if kbytes <= *p {
                            return Err(CborError::NonCanonical("map keys unsorted or duplicate"));
                        }
                    }
                    prev = Some(kbytes);
                    let v = self.value(depth + 1)?;
                    m.insert(k, v);
                }
                Value::Map(m)
            }
            6 => return Err(CborError::Unsupported("tags")),
            _ => match ai {
                20 => Value::Bool(false),
                21 => Value::Bool(true),
                22 => Value::Null,
                27 => {
                    let f = f64::from_bits(u64::from_be_bytes(self.take(8)?.try_into().unwrap()));
                    if f.is_nan() {
                        return Err(CborError::NaN);
                    }
                    Value::Float(f)
                }
                25 | 26 => return Err(CborError::NonCanonical("floats must be 64-bit")),
                _ => return Err(CborError::Unsupported("simple value")),
            },
        })
    }
}

/// Decode canonical bytes. Rejects any non-canonical form and trailing bytes.
pub fn decode(b: &[u8]) -> Result<Value, CborError> {
    let mut r = Reader { b, p: 0 };
    let v = r.value(0)?;
    if r.p != b.len() {
        return Err(CborError::TrailingBytes);
    }
    Ok(v)
}

// ---- serde bridge (via ciborium's Value) ----

fn from_cib(v: ciborium::Value) -> Result<Value, CborError> {
    use ciborium::Value as C;
    Ok(match v {
        C::Null => Value::Null,
        C::Bool(b) => Value::Bool(b),
        C::Integer(i) => Value::Int(i128::from(i)),
        C::Float(f) => {
            if f.is_nan() {
                return Err(CborError::NaN);
            }
            Value::Float(f)
        }
        C::Bytes(b) => Value::Bytes(b),
        C::Text(s) => Value::Text(s),
        C::Array(a) => Value::Array(a.into_iter().map(from_cib).collect::<Result<_, _>>()?),
        C::Map(m) => {
            let mut out = BTreeMap::new();
            for (k, v) in m {
                let k = match k {
                    C::Text(s) => s,
                    _ => return Err(CborError::NonTextKey),
                };
                if out.insert(k, from_cib(v)?).is_some() {
                    return Err(CborError::NonCanonical("duplicate key"));
                }
            }
            Value::Map(out)
        }
        C::Tag(..) => return Err(CborError::Unsupported("tags")),
        _ => return Err(CborError::Unsupported("value kind")),
    })
}

fn to_cib(v: &Value) -> ciborium::Value {
    use ciborium::Value as C;
    match v {
        Value::Null => C::Null,
        Value::Bool(b) => C::Bool(*b),
        Value::Int(i) => C::Integer((*i).try_into().expect("range checked on construction")),
        Value::Float(f) => C::Float(*f),
        Value::Bytes(b) => C::Bytes(b.clone()),
        Value::Text(s) => C::Text(s.clone()),
        Value::Array(a) => C::Array(a.iter().map(to_cib).collect()),
        Value::Map(m) => C::Map(
            m.iter()
                .map(|(k, v)| (C::Text(k.clone()), to_cib(v)))
                .collect(),
        ),
    }
}

/// Convert any serde-serialisable value to a canonical [`Value`].
pub fn to_value<T: serde::Serialize + ?Sized>(t: &T) -> Result<Value, CborError> {
    let c = ciborium::Value::serialized(t).map_err(|e| CborError::Serde(e.to_string()))?;
    from_cib(c)
}

/// Convert a canonical [`Value`] back to a typed value.
pub fn from_value<T: serde::de::DeserializeOwned>(v: &Value) -> Result<T, CborError> {
    to_cib(v)
        .deserialized()
        .map_err(|e| CborError::Serde(e.to_string()))
}

/// Canonical bytes of any serde value.
pub fn to_canonical<T: serde::Serialize + ?Sized>(t: &T) -> Result<Vec<u8>, CborError> {
    encode(&to_value(t)?)
}

/// Typed value from canonical bytes (non-canonical input is rejected).
pub fn from_canonical<T: serde::de::DeserializeOwned>(b: &[u8]) -> Result<T, CborError> {
    from_value(&decode(b)?)
}

impl serde::Serialize for Value {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        use serde::ser::{SerializeMap, SerializeSeq};
        match self {
            Value::Null => s.serialize_unit(),
            Value::Bool(b) => s.serialize_bool(*b),
            Value::Int(i) => {
                if let Ok(x) = i64::try_from(*i) {
                    s.serialize_i64(x)
                } else if let Ok(x) = u64::try_from(*i) {
                    s.serialize_u64(x)
                } else {
                    s.serialize_i128(*i)
                }
            }
            Value::Float(f) => s.serialize_f64(*f),
            Value::Bytes(b) => s.serialize_bytes(b),
            Value::Text(t) => s.serialize_str(t),
            Value::Array(a) => {
                let mut q = s.serialize_seq(Some(a.len()))?;
                for x in a {
                    q.serialize_element(x)?;
                }
                q.end()
            }
            Value::Map(m) => {
                let mut q = s.serialize_map(Some(m.len()))?;
                for (k, v) in m {
                    q.serialize_entry(k, v)?;
                }
                q.end()
            }
        }
    }
}

/// Integral JSON numbers below 2^63 in size are integers (the i64 range,
/// which typed fields such as times in seconds need); from there on a JSON
/// number is a float however it is spelled, so it always decodes the same
/// way and always writes back as JSON.
const JSON_INT_LIMIT: f64 = 9223372036854775808.0;

/// The one rule for a JSON number, whatever client wrote it. JSON's clients
/// hold numbers as doubles (JavaScript), so a JSON number is first the
/// nearest double, then: integral and below 2^63 in size, the integer that
/// double is exactly; anything else that double. `281`, `281.0` and `2.81e2`
/// are one value, and so are `4.560563969097498e17` and the
/// `456056396909749800` a browser writes for it. Packs read their numbers by
/// it too (`sse_pack::json_numbers`).
pub fn json_number(x: f64) -> Value {
    if x.is_finite() && x.fract() == 0.0 && x.abs() < JSON_INT_LIMIT {
        Value::Int(x as i128)
    } else {
        Value::Float(x)
    }
}

impl<'de> serde::Deserialize<'de> for Value {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        // JSON (any human-readable format) has one number type, and the
        // kernel's clients are JavaScript: `JSON.stringify(281.0)` is "281".
        // So a JSON number decodes by its value alone, never by spelling:
        // integral and below 2^63 in size is an integer, anything else a
        // float. "281", "281.0" and "2.81e2" are then one value with one
        // hash on every client. Binary CBOR keeps the two types as encoded.
        struct V {
            json: bool,
        }
        impl<'de> serde::de::Visitor<'de> for V {
            type Value = Value;
            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("a canonical SSE value")
            }
            fn visit_unit<E>(self) -> Result<Value, E> {
                Ok(Value::Null)
            }
            fn visit_none<E>(self) -> Result<Value, E> {
                Ok(Value::Null)
            }
            fn visit_some<D2: serde::Deserializer<'de>>(self, d: D2) -> Result<Value, D2::Error> {
                serde::Deserialize::deserialize(d)
            }
            fn visit_bool<E>(self, b: bool) -> Result<Value, E> {
                Ok(Value::Bool(b))
            }
            fn visit_i64<E>(self, i: i64) -> Result<Value, E> {
                if self.json {
                    return Ok(json_number(i as f64));
                }
                Ok(Value::Int(i as i128))
            }
            fn visit_u64<E>(self, i: u64) -> Result<Value, E> {
                if self.json {
                    return Ok(json_number(i as f64));
                }
                Ok(Value::Int(i as i128))
            }
            fn visit_i128<E: serde::de::Error>(self, i: i128) -> Result<Value, E> {
                if !(INT_MIN..=INT_MAX).contains(&i) {
                    return Err(E::custom("integer out of range"));
                }
                Ok(Value::Int(i))
            }
            fn visit_f64<E: serde::de::Error>(self, f: f64) -> Result<Value, E> {
                if f.is_nan() {
                    return Err(E::custom("NaN is not a value"));
                }
                if self.json {
                    return Ok(json_number(f));
                }
                Ok(Value::Float(f))
            }
            fn visit_str<E>(self, t: &str) -> Result<Value, E> {
                Ok(Value::Text(t.to_owned()))
            }
            fn visit_string<E>(self, t: String) -> Result<Value, E> {
                Ok(Value::Text(t))
            }
            fn visit_bytes<E>(self, b: &[u8]) -> Result<Value, E> {
                Ok(Value::Bytes(b.to_vec()))
            }
            fn visit_byte_buf<E>(self, b: Vec<u8>) -> Result<Value, E> {
                Ok(Value::Bytes(b))
            }
            fn visit_seq<A: serde::de::SeqAccess<'de>>(self, mut a: A) -> Result<Value, A::Error> {
                let mut v = Vec::new();
                while let Some(x) = a.next_element()? {
                    v.push(x);
                }
                Ok(Value::Array(v))
            }
            fn visit_map<A: serde::de::MapAccess<'de>>(self, mut a: A) -> Result<Value, A::Error> {
                let mut m = BTreeMap::new();
                while let Some((k, v)) = a.next_entry::<String, Value>()? {
                    if m.insert(k, v).is_some() {
                        return Err(serde::de::Error::custom("duplicate key"));
                    }
                }
                Ok(Value::Map(m))
            }
        }
        let json = d.is_human_readable();
        d.deserialize_any(V { json })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_numbers_decode_by_value_as_javascript_writes_them() {
        let j = |t: &str| serde_json::from_str::<Value>(t).unwrap();
        for t in ["281", "281.0", "2.81e2", "2.810e+2"] {
            assert_eq!(j(t), Value::Int(281), "{t}");
        }
        assert_eq!(j("-0.0"), Value::Int(0));
        assert_eq!(j("0.5"), Value::Float(0.5));
        // Integers up to the i64 range, in any spelling; beyond it, floats.
        assert_eq!(j("31557600000000000"), Value::Int(31557600000000000));
        assert_eq!(j("3.15576e16"), Value::Int(31557600000000000));
        // Every number is first the nearest double, as in a browser: the
        // digits JavaScript writes for a double past 2^53 are its shortest
        // spelling, not its exact value, and read back as the same integer.
        assert_eq!(j("456056396909749800"), Value::Int(456056396909749824));
        assert_eq!(j("4.560563969097498e17"), Value::Int(456056396909749824));
        assert_eq!(j("9007199254740993"), Value::Int(9007199254740992));
        for t in [
            "9223372036854775807",
            "9223372036854775808",
            "9.223372036854775808e18",
            "-9223372036854775808",
        ] {
            assert!(matches!(j(t), Value::Float(_)), "{t}");
        }
        assert_eq!(j("22693600000000000000"), j("2.26936e19"));
        assert_eq!(j("123456789012345678901"), j("1.2345678901234568e20"));
        assert_eq!(j("1.9e27"), Value::Float(1.9e27));
        // Whatever JSON decodes, writes back as JSON.
        for t in ["22693600000000000000", "1e300", "-12", "0.1"] {
            assert!(serde_json::to_string(&j(t)).is_ok(), "{t}");
        }
        // Exact parse (float_roundtrip): the shortest text round-trips.
        assert_eq!(j("1.2e27"), Value::Float(1.2e27));
        assert_eq!(serde_json::to_string(&j("1.2e27")).unwrap(), "1.2e+27");
        // Binary CBOR keeps the encoded type.
        let b = encode(&Value::Float(281.0)).unwrap();
        assert_eq!(decode(&b).unwrap(), Value::Float(281.0));
        assert_eq!(
            from_value::<Value>(&Value::Float(281.0)).unwrap(),
            Value::Float(281.0)
        );
    }

    fn rt(v: Value) {
        let b = encode(&v).unwrap();
        assert_eq!(decode(&b).unwrap(), v);
        assert_eq!(encode(&decode(&b).unwrap()).unwrap(), b);
    }

    #[test]
    fn rfc8949_appendix_a_integers() {
        let cases: &[(i128, &str)] = &[
            (0, "00"),
            (1, "01"),
            (23, "17"),
            (24, "1818"),
            (100, "1864"),
            (1000, "1903e8"),
            (1000000, "1a000f4240"),
            (1000000000000, "1b000000e8d4a51000"),
            (18446744073709551615, "1bffffffffffffffff"),
            (-18446744073709551616, "3bffffffffffffffff"),
            (-1, "20"),
            (-10, "29"),
            (-100, "3863"),
            (-1000, "3903e7"),
        ];
        for (i, hex) in cases {
            let b = encode(&Value::Int(*i)).unwrap();
            assert_eq!(crate::hash::to_hex(&b), *hex, "int {i}");
            rt(Value::Int(*i));
        }
        assert_eq!(
            encode(&Value::Int(INT_MAX + 1)),
            Err(CborError::IntOutOfRange)
        );
    }

    #[test]
    fn floats_are_64_bit_and_nan_rejected() {
        assert_eq!(
            crate::hash::to_hex(&encode(&Value::Float(1.5)).unwrap()),
            "fb3ff8000000000000"
        );
        assert_ne!(
            encode(&Value::Float(0.0)).unwrap(),
            encode(&Value::Float(-0.0)).unwrap()
        );
        assert_eq!(encode(&Value::Float(f64::NAN)), Err(CborError::NaN));
        assert_eq!(
            decode(&[0xfb, 0x7f, 0xf8, 0, 0, 0, 0, 0, 0]),
            Err(CborError::NaN)
        );
        assert!(matches!(
            decode(&[0xf9, 0x3c, 0x00]),
            Err(CborError::NonCanonical(_))
        ));
        rt(Value::Float(f64::INFINITY));
        rt(Value::Float(f64::MIN_POSITIVE / 4.0));
    }

    #[test]
    fn map_order_is_canonical_regardless_of_insertion() {
        let mut a = BTreeMap::new();
        for k in ["zz", "a", "bb", "b", ""] {
            a.insert(k.to_string(), Value::Int(k.len() as i128));
        }
        let b = encode(&Value::Map(a.clone())).unwrap();
        // keys: "" , "a", "b", "bb", "zz"
        assert_eq!(
            crate::hash::to_hex(&b),
            "a5600061610161620162626202627a7a02"
        );
        rt(Value::Map(a));
        // unsorted map must be rejected: {"b":0,"a":0}
        assert!(matches!(
            decode(&crate::hash::from_hex("a2616200616100").unwrap()),
            Err(CborError::NonCanonical(_))
        ));
    }

    #[test]
    fn rejects_non_canonical_heads_and_trailing() {
        assert!(matches!(
            decode(&[0x18, 0x05]),
            Err(CborError::NonCanonical(_))
        ));
        assert!(matches!(
            decode(&[0x9f, 0xff]),
            Err(CborError::NonCanonical(_))
        ));
        assert_eq!(decode(&[0x01, 0x01]), Err(CborError::TrailingBytes));
        assert_eq!(decode(&[0x62, 0x61]), Err(CborError::Truncated));
        assert!(matches!(
            decode(&[0xc1, 0x01]),
            Err(CborError::Unsupported(_))
        ));
    }

    #[test]
    fn serde_bridge_round_trips() {
        #[derive(serde::Serialize, serde::Deserialize, PartialEq, Debug)]
        struct S {
            name: String,
            mass: f64,
            n: u64,
            tags: Vec<String>,
            opt: Option<i32>,
        }
        let s = S {
            name: "Tarn".into(),
            mass: 5.97e24,
            n: 3,
            tags: vec!["rocky".into()],
            opt: None,
        };
        let b = to_canonical(&s).unwrap();
        assert_eq!(from_canonical::<S>(&b).unwrap(), s);
        assert_eq!(to_canonical(&f64::NAN), Err(CborError::NaN));
    }

    #[test]
    fn value_is_serde_through_cbor_and_json() {
        let mut m = BTreeMap::new();
        m.insert("big".to_string(), Value::Int(INT_MAX));
        m.insert("neg".to_string(), Value::Int(INT_MIN));
        m.insert("f".to_string(), Value::Float(-0.0));
        m.insert(
            "a".to_string(),
            Value::Array(vec![
                Value::Null,
                Value::Bool(true),
                Value::Text("星".into()),
            ]),
        );
        let v = Value::Map(m);
        assert_eq!(
            from_canonical::<Value>(&to_canonical(&v).unwrap()).unwrap(),
            v
        );
        assert_eq!(to_value(&v).unwrap(), v);
    }
}

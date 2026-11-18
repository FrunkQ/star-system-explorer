//! SHA-256 content addresses (H3: "SHA-256 for content addresses and state hashes").

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fmt;

/// A SHA-256 digest. Displays and serialises as lowercase hex.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ContentHash(pub [u8; 32]);

impl ContentHash {
    pub fn of_bytes(b: &[u8]) -> Self {
        ContentHash(Sha256::digest(b).into())
    }
    /// Hash of a value's canonical CBOR encoding.
    pub fn of<T: Serialize + ?Sized>(t: &T) -> Result<Self, crate::cbor::CborError> {
        Ok(Self::of_bytes(&crate::cbor::to_canonical(t)?))
    }
    pub fn hex(&self) -> String {
        to_hex(&self.0)
    }
    pub fn parse(s: &str) -> Option<Self> {
        let v = from_hex(s)?;
        Some(ContentHash(v.try_into().ok()?))
    }
}

impl fmt::Debug for ContentHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ContentHash({})", self.hex())
    }
}
impl fmt::Display for ContentHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.hex())
    }
}
impl Serialize for ContentHash {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&self.hex())
    }
}
impl<'de> Deserialize<'de> for ContentHash {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let s = String::deserialize(d)?;
        ContentHash::parse(&s).ok_or_else(|| serde::de::Error::custom("bad content hash"))
    }
}

/// Incremental hasher with domain separation.
#[derive(Clone)]
pub struct Hasher(Sha256);
impl Hasher {
    pub fn new(domain: &str) -> Self {
        let mut h = Sha256::new();
        h.update((domain.len() as u64).to_be_bytes());
        h.update(domain.as_bytes());
        Hasher(h)
    }
    pub fn update(&mut self, b: &[u8]) -> &mut Self {
        self.0.update(b);
        self
    }
    pub fn finish(self) -> ContentHash {
        ContentHash(self.0.finalize().into())
    }
}

pub fn to_hex(b: &[u8]) -> String {
    const H: &[u8; 16] = b"0123456789abcdef";
    let mut s = String::with_capacity(b.len() * 2);
    for x in b {
        s.push(H[(x >> 4) as usize] as char);
        s.push(H[(x & 15) as usize] as char);
    }
    s
}

pub fn from_hex(s: &str) -> Option<Vec<u8>> {
    if !s.len().is_multiple_of(2) {
        return None;
    }
    let d = |c: u8| match c {
        b'0'..=b'9' => Some(c - b'0'),
        b'a'..=b'f' => Some(c - b'a' + 10),
        _ => None,
    };
    s.as_bytes()
        .chunks(2)
        .map(|p| Some(d(p[0])? << 4 | d(p[1])?))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn sha256_known_vectors() {
        assert_eq!(
            ContentHash::of_bytes(b"").hex(),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
        assert_eq!(
            ContentHash::of_bytes(b"abc").hex(),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }
}

//! Refuelling as a ship's capability (V8 part 3c). Alex, 22 Nov: "fuel is
//! determined by ship and engine and fuel type - all physically modelled
//! ... Refuelling is a ship capability - it may need a port and exotic
//! fuel."
//!
//! A stop gives fuel only when three things agree: what the place holds (a
//! kind of source: a giant to skim, ice to crack, a port and its stock),
//! what the ship can do there (a scoop, an ice processor, docking), and the
//! fuels its engines and jump drive burn. Kinds, what they need and what
//! they give are pack data (`nav.voyage.refuel`), so a setting or Orac can
//! add one. A place lists kind ids; a port may add stock beyond the pack's
//! common list as `port:antimatter,tritium`.

use crate::Ship;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// One kind of refuelling source.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct RefuelKind {
    /// Words for the leg ("a gas giant to skim").
    pub words: String,
    /// The ship capability it needs ("scoop"), if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub needs: Option<String>,
    /// Fuels it gives.
    pub gives: Vec<String>,
    /// Why it gives them (physics, for "Why?").
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub why: String,
}

/// The pack's refuelling rules.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct RefuelRules {
    pub kinds: BTreeMap<String, RefuelKind>,
}

impl Default for RefuelRules {
    fn default() -> Self {
        let k = |words: &str, needs: Option<&str>, gives: &[&str]| RefuelKind {
            words: words.into(),
            needs: needs.map(String::from),
            gives: gives.iter().map(|g| g.to_string()).collect(),
            why: String::new(),
        };
        let mut kinds = BTreeMap::new();
        kinds.insert(
            "skim".into(),
            k(
                "a gas giant to skim",
                Some("scoop"),
                &["hydrogen", "deuterium", "helium-3"],
            ),
        );
        kinds.insert(
            "ice".into(),
            k(
                "ice to crack",
                Some("ice processor"),
                &["water", "hydrogen", "hydrogen-oxygen", "deuterium"],
            ),
        );
        kinds.insert(
            "port".into(),
            k(
                "a port",
                Some("docking"),
                &["hydrogen", "water", "hydrogen-oxygen", "deuterium"],
            ),
        );
        RefuelRules { kinds }
    }
}

/// The fuels a ship needs at a stop: its engines' and its jump drive's.
pub fn needs(ship: &Ship) -> Vec<String> {
    let mut out: Vec<String> = ship.fuels.clone();
    if let Some(j) = &ship.jump {
        if !out.contains(&j.fuel) {
            out.push(j.fuel.clone());
        }
    }
    out
}

impl RefuelRules {
    /// Words for an entry ("port:antimatter" reads "a port (antimatter)").
    pub fn words(&self, entry: &str) -> String {
        let (id, stock) = split(entry);
        let base = self
            .kinds
            .get(id)
            .map(|k| k.words.clone())
            .unwrap_or_else(|| id.to_string());
        if stock.is_empty() {
            base
        } else {
            format!("{base} ({})", stock.join(", "))
        }
    }

    /// What one entry gives.
    pub fn gives(&self, entry: &str) -> Vec<String> {
        let (id, stock) = split(entry);
        let mut out = self
            .kinds
            .get(id)
            .map(|k| k.gives.clone())
            .unwrap_or_default();
        for s in stock {
            if !out.contains(&s) {
                out.push(s);
            }
        }
        out
    }

    /// Whether the ship can use this entry: it has the capability, and the
    /// entry gives every fuel the ship burns. An unknown kind gives nothing.
    pub fn usable(&self, ship: &Ship, entry: &str) -> Result<(), String> {
        let (id, _) = split(entry);
        let Some(kind) = self.kinds.get(id) else {
            return Err(format!("{id} is not a refuelling kind in this setting"));
        };
        if let Some(need) = &kind.needs {
            if !ship.can(need) {
                return Err(format!("needs a {need}"));
            }
        }
        let gives = self.gives(entry);
        let missing: Vec<String> = needs(ship)
            .into_iter()
            .filter(|f| !gives.contains(f))
            .collect();
        if missing.is_empty() {
            Ok(())
        } else {
            Err(format!("has no {}", missing.join(" or ")))
        }
    }

    /// The first entry at a place the ship can refuel from, in words.
    pub fn offers(&self, ship: &Ship, entries: &[String]) -> Option<String> {
        entries
            .iter()
            .find(|e| self.usable(ship, e).is_ok())
            .map(|e| self.words(e))
    }
}

fn split(entry: &str) -> (&str, Vec<String>) {
    match entry.split_once(':') {
        Some((id, rest)) => (
            id.trim(),
            rest.split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect(),
        ),
        None => (entry.trim(), vec![]),
    }
}

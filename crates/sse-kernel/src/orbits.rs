//! Orbits at any time, the campaign clock and pack calendars (asked for by
//! the chart room for human-scale time).
//!
//! `orbits_at`: each body with an `orbit.semi_major_axis` and a host mass
//! gets its position local to its host (f64 metres, the host's reference
//! plane as z = 0) at each asked time, from Kepler's equation solved by a
//! fixed number of Newton steps (deterministic on every device). A body
//! with no dated phase (`orbit.mean_anomaly_at_epoch`) takes one from the
//! universe's keyed draw on its address, labelled `resolved`: the same on
//! every device and every call, and pinnable like any fact. A body missing
//! anything else comes back unplaced with the missing fields named, never
//! with a guessed position.
//!
//! The epoch of a dated phase is `orbit.epoch_s` (universe seconds), else
//! an SSE 3 `orbit.epoch_v3_ms` read as milliseconds of universe time, else
//! the universe origin.

use crate::err;
use crate::protocol::Response;
use serde::{Deserialize, Serialize};
use sse_chronicle::{Chronicle, Event, EventBody, Evidence, Payload, Publisher, UniverseTime};
use sse_core::cbor::Value;
use sse_core::draw::{Draws, Stream};
use sse_core::math as m;
use sse_core::{Address, Seed};
use std::collections::BTreeMap;

const G: f64 = sse_core::sheet::G;
/// Times one call may ask for.
pub const MAX_TIMES: usize = 64;
/// Event kind and root field of the campaign clock.
pub const CAMPAIGN_NOW: &str = "time.campaign_now";
pub const CAMPAIGN_NOW_FIELD: &str = "campaign.now_s";

type Facts = BTreeMap<Address, BTreeMap<String, Value>>;

fn num(v: &Value) -> Option<f64> {
    match v {
        Value::Float(f) => Some(*f),
        Value::Int(i) => Some(*i as f64),
        _ => None,
    }
}

/// One body at one time.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct At {
    pub at_s: i64,
    /// Position relative to the host, metres.
    pub r_m: [f64; 3],
    /// Longitude in the host's reference plane, degrees in [0, 360).
    pub phase_deg: f64,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Placed {
    pub address: String,
    pub placed: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub period_s: Option<f64>,
    /// `authored` or `derived` (a dated phase in the facts) or `resolved` (the keyed draw).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub phase_source: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub missing: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub at: Vec<At>,
}

/// Solve E - e sin E = M for E (radians), 16 fixed Newton steps from a safe start.
pub fn eccentric_anomaly(mean: f64, e: f64) -> f64 {
    let mean = mean - m::TAU * m::floor(mean / m::TAU);
    let mut ea = if e < 0.8 { mean } else { m::PI };
    for _ in 0..16 {
        ea -= (ea - e * m::sin(ea) - mean) / (1.0 - e * m::cos(ea));
    }
    ea
}

/// Every orbiting body at or under `system` in `facts`, at each time.
pub fn orbits_at(
    facts: &Facts,
    system: &Address,
    times: &[i64],
    seed: Option<&Seed>,
) -> Vec<Placed> {
    let mut out = vec![];
    for (a, row) in facts.iter().filter(|(a, _)| system.contains(a)) {
        let Some(sma) = row.get("orbit.semi_major_axis") else {
            continue;
        };
        let mut missing = vec![];
        let host: Option<Address> = row
            .get("orbit.host")
            .or_else(|| row.get("parent"))
            .and_then(|v| match v {
                Value::Text(t) => t.parse().ok(),
                _ => None,
            })
            .or_else(|| a.parent());
        let host_mass = host
            .as_ref()
            .and_then(|h| facts.get(h))
            .and_then(|r| r.get("body.mass"))
            .and_then(num);
        let a_m = num(sma).filter(|x| *x > 0.0);
        if a_m.is_none() {
            missing.push("orbit.semi_major_axis".to_string());
        }
        if host_mass.is_none() {
            missing.push(match &host {
                Some(h) => format!("{h} body.mass"),
                None => "orbit.host".into(),
            });
        }
        let e = row.get("orbit.eccentricity").and_then(num).unwrap_or(0.0);
        if !(0.0..1.0).contains(&e) {
            missing.push("orbit.eccentricity below 1".into());
        }
        let (m0, source) = match row.get("orbit.mean_anomaly_at_epoch").and_then(num) {
            Some(x) => (Some(x), "derived"),
            None => match seed {
                Some(s) => {
                    let d = Draws::new(
                        *s,
                        a.clone(),
                        Stream::new("orbit.phase.v1").expect("stream"),
                    );
                    (Some(d.unit(0) * m::TAU), "resolved")
                }
                None => {
                    missing.push("orbit.mean_anomaly_at_epoch (no universe to draw one)".into());
                    (None, "")
                }
            },
        };
        let (Some(a_m), Some(mh), Some(m0), true) = (a_m, host_mass, m0, missing.is_empty()) else {
            out.push(Placed {
                address: a.to_string(),
                placed: false,
                parent: host.map(|h| h.to_string()),
                period_s: None,
                phase_source: None,
                missing,
                at: vec![],
            });
            continue;
        };
        let mass = row.get("body.mass").and_then(num).unwrap_or(0.0);
        let mu = G * (mh + mass);
        let n = m::sqrt(mu / (a_m * a_m * a_m));
        let epoch = row
            .get("orbit.epoch_s")
            .and_then(num)
            .or_else(|| {
                row.get("orbit.epoch_v3_ms")
                    .and_then(num)
                    .map(|x| x / 1000.0)
            })
            .unwrap_or(0.0);
        let inc = row.get("orbit.inclination").and_then(num).unwrap_or(0.0);
        let node = row.get("orbit.ascending_node").and_then(num).unwrap_or(0.0);
        let peri = row
            .get("orbit.argument_of_periapsis")
            .and_then(num)
            .unwrap_or(0.0);
        let (co, so) = (m::cos(node), m::sin(node));
        let (cw, sw) = (m::cos(peri), m::sin(peri));
        let (ci, si) = (m::cos(inc), m::sin(inc));
        let b = a_m * m::sqrt(1.0 - e * e);
        let at = times
            .iter()
            .map(|&t| {
                let mean = m0 + n * (t as f64 - epoch);
                let ea = eccentric_anomaly(mean, e);
                let (xp, yp) = (a_m * (m::cos(ea) - e), b * m::sin(ea));
                // Perifocal to reference frame: R3(-node) R1(-inc) R3(-peri).
                let x = (co * cw - so * sw * ci) * xp + (-co * sw - so * cw * ci) * yp;
                let y = (so * cw + co * sw * ci) * xp + (-so * sw + co * cw * ci) * yp;
                let z = (sw * si) * xp + (cw * si) * yp;
                let mut ph = m::atan2(y, x) * (180.0 / m::PI);
                if ph < 0.0 {
                    ph += 360.0;
                }
                if ph >= 360.0 {
                    ph -= 360.0;
                }
                At {
                    at_s: t,
                    r_m: [x, y, z],
                    phase_deg: ph,
                }
            })
            .collect();
        out.push(Placed {
            address: a.to_string(),
            placed: true,
            parent: host.map(|h| h.to_string()),
            period_s: Some(m::TAU / n),
            phase_source: Some(source.into()),
            missing: vec![],
            at,
        });
    }
    out
}

/// Facts from an inline `[{address, facts: {..}}]` list.
pub fn inline_facts(bodies: &[serde_json::Value]) -> Result<Facts, Response> {
    let mut f = Facts::new();
    for (i, b) in bodies.iter().enumerate() {
        let a: Address = b["address"]
            .as_str()
            .ok_or_else(|| err("bad_request", format!("body {i}: no address")))?
            .parse()
            .map_err(|e| err("bad_address", format!("body {i}: {e}")))?;
        let row: BTreeMap<String, Value> = serde_json::from_value(b["facts"].clone())
            .map_err(|e| err("bad_value", format!("body {i}: facts: {e}")))?;
        f.insert(a, row);
    }
    Ok(f)
}

/// The campaign clock event: `campaign.now_s` on the root, active from the
/// origin so every reading of the universe sees the latest one.
pub fn campaign_now_event(ch: &Chronicle, root: &Address, at_s: i64) -> Result<Event, Response> {
    let mut set = BTreeMap::new();
    set.insert(
        root.clone(),
        [(CAMPAIGN_NOW_FIELD.to_string(), Value::Int(at_s.into()))].into(),
    );
    Event::new(EventBody {
        kind: CAMPAIGN_NOW.into(),
        subject: vec![root.clone()],
        occurred_at: UniverseTime::ORIGIN,
        activated_at: UniverseTime::ORIGIN,
        recognised_at: ch.next_stamp(),
        causes: vec![],
        payload: Payload {
            set,
            ..Default::default()
        },
        evidence: Evidence::Authored,
        publisher: Publisher::new("gm", "campaign-clock", "1"),
    })
    .map_err(|e| err("bad_event", format!("{e}")))
}

/// The campaign's now, and the event that set it.
pub fn campaign_now(ch: &Chronicle, root: &Address) -> (Option<i64>, Option<String>) {
    let e = ch
        .events()
        .iter()
        .rev()
        .find(|e| e.body.kind == CAMPAIGN_NOW && e.body.subject.first() == Some(root));
    let at = e
        .and_then(|e| e.body.payload.set.get(root))
        .and_then(|r| r.get(CAMPAIGN_NOW_FIELD))
        .and_then(|v| match v {
            Value::Int(i) => i64::try_from(*i).ok(),
            _ => None,
        });
    (at, e.map(|e| e.id.hex()))
}

/// A pack calendar (`provides.other.calendar`): the kernel checks and
/// carries it; the UI formats dates with it.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Calendar {
    pub name: String,
    /// Universe time of day 1 of year `epoch_year`.
    pub epoch_s: i64,
    #[serde(default)]
    pub epoch_year: i64,
    /// Seconds in a day and days in a year.
    pub day_s: f64,
    pub year_days: f64,
    /// Months in order; their days must add to `year_days` (whole years only).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub months: Vec<Month>,
    /// For the UI, e.g. `"{day:03}-{year}"` (Traveller) or `"{mday} {month} {year}"`.
    pub format: String,
    /// Named days of the year (1-based), e.g. the Imperial Holiday.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub named_days: Vec<NamedDay>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub week: Option<Week>,
    /// Leap years: every `every` years, except every `except_every`, but every `but_every`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub leap: Option<Leap>,
    /// The month (0-based) that gains the leap day; needed when months and leap are both set.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub leap_month: Option<usize>,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NamedDay {
    pub day: u32,
    pub name: String,
    /// The day belongs to no week (needs a week that restarts each year).
    #[serde(default)]
    pub outside_weeks: bool,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Week {
    pub names: Vec<String>,
    /// 1-based day of the year on which the first week starts.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub first_day: Option<u32>,
    /// True (the default): weeks run on across years. False: they restart each year.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub continuous: Option<bool>,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Leap {
    pub every: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub except_every: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub but_every: Option<u32>,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Month {
    pub name: String,
    pub days: u32,
}

impl Calendar {
    pub fn check(&self) -> Result<(), String> {
        if !(self.day_s.is_finite()
            && self.day_s > 0.0
            && self.year_days.is_finite()
            && self.year_days > 0.0)
        {
            return Err("day_s and year_days must be positive".into());
        }
        if self.format.trim().is_empty() {
            return Err("a calendar needs a format".into());
        }
        if !self.months.is_empty() {
            let sum: u32 = self.months.iter().map(|x| x.days).sum();
            if f64::from(sum) != self.year_days {
                return Err(format!(
                    "months add to {sum} days, the year has {}",
                    self.year_days
                ));
            }
        }
        let max_day = self.year_days + if self.leap.is_some() { 1.0 } else { 0.0 };
        let continuous = self
            .week
            .as_ref()
            .is_none_or(|w| w.continuous != Some(false));
        for d in &self.named_days {
            if d.day == 0 || f64::from(d.day) > max_day || d.name.trim().is_empty() {
                return Err(format!(
                    "named day {} must be named and lie in 1 to {max_day}",
                    d.day
                ));
            }
            if d.outside_weeks && continuous {
                return Err(format!(
                    "named day {}: outside_weeks needs a week with continuous false",
                    d.name
                ));
            }
        }
        if let Some(w) = &self.week {
            if w.names.is_empty() || w.names.iter().any(|n| n.trim().is_empty()) {
                return Err("a week needs its day names".into());
            }
            if let Some(f) = w.first_day {
                if f == 0 || f64::from(f) > max_day {
                    return Err(format!("week first_day {f} must lie in 1 to {max_day}"));
                }
            }
        }
        if let Some(l) = &self.leap {
            let ok = |x: Option<u32>, of: u32| x.is_none_or(|x| x > of && x % of == 0);
            if l.every == 0
                || !ok(l.except_every, l.every)
                || !ok(l.but_every, l.except_every.unwrap_or(u32::MAX))
                || (l.but_every.is_some() && l.except_every.is_none())
            {
                return Err("leap: every > 0; except_every a multiple of it; but_every a multiple of except_every".into());
            }
            if !self.months.is_empty() {
                match self.leap_month {
                    Some(i) if i < self.months.len() => {}
                    _ => {
                        return Err(
                            "leap_month (0-based) is needed when months and leap are both set"
                                .into(),
                        )
                    }
                }
            }
        } else if self.leap_month.is_some() {
            return Err("leap_month needs leap".into());
        }
        Ok(())
    }
}

/// The top pack's calendar, checked (None when no pack has one).
pub fn load_calendar(ms: &[sse_pack::PackManifest]) -> Result<Option<Calendar>, Vec<String>> {
    let Some(p) = ms
        .iter()
        .rev()
        .find(|p| p.provides.other.contains_key("calendar"))
    else {
        return Ok(None);
    };
    let v = serde_json::to_value(&p.provides.other["calendar"]).map_err(|e| vec![e.to_string()])?;
    let c: Calendar =
        serde_json::from_value(v).map_err(|e| vec![format!("{}: calendar: {e}", p.id)])?;
    c.check()
        .map_err(|e| vec![format!("{}: calendar: {e}", p.id)])?;
    Ok(Some(c))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn earth_sun() -> Facts {
        let mut f = Facts::new();
        let sun: Address = "u:o/sys:0/star:0".parse().unwrap();
        let earth: Address = "u:o/sys:0/body:3".parse().unwrap();
        f.insert(
            sun.clone(),
            [("body.mass".to_string(), Value::Float(1.988_47e30))].into(),
        );
        f.insert(
            earth,
            [
                ("orbit.host".to_string(), Value::Text(sun.to_string())),
                (
                    "orbit.semi_major_axis".to_string(),
                    Value::Float(1.495_978_707e11),
                ),
                ("orbit.eccentricity".to_string(), Value::Float(0.0167)),
                ("orbit.mean_anomaly_at_epoch".to_string(), Value::Float(0.0)),
                ("body.mass".to_string(), Value::Float(5.972e24)),
            ]
            .into(),
        );
        f
    }

    #[test]
    fn kepler_orbits_land_where_they_should() {
        let f = earth_sun();
        let sys: Address = "u:o/sys:0".parse().unwrap();
        let year = 31_558_149;
        let p = orbits_at(&f, &sys, &[0, year / 4, year / 2, year], None);
        assert_eq!(p.len(), 1);
        let e = &p[0];
        assert!(e.placed);
        let days = e.period_s.unwrap() / 86_400.0;
        println!("Earth's period {days:.3} days, positions {:?}", e.at);
        assert!((days - 365.256).abs() < 0.01);
        let a = 1.495_978_707e11;
        let r = |x: &At| m::sqrt(x.r_m[0] * x.r_m[0] + x.r_m[1] * x.r_m[1] + x.r_m[2] * x.r_m[2]);
        assert!(
            (r(&e.at[0]) - a * (1.0 - 0.0167)).abs() < 1.0,
            "periapsis at M = 0"
        );
        assert!(
            (r(&e.at[2]) - a * (1.0 + 0.0167)).abs() < 1.0e6,
            "apoapsis half a year on"
        );
        assert!(e.at[0].phase_deg.abs() < 1e-9);
        assert!((e.at[2].phase_deg - 180.0).abs() < 0.05);
        assert!((e.at[3].phase_deg - e.at[0].phase_deg).abs() < 0.01 || e.at[3].phase_deg > 359.99);
        // Kepler's equation is solved to the bit at high eccentricity too.
        for &ecc in &[0.0, 0.3, 0.9, 0.99] {
            for k in 0..64 {
                let mm = k as f64 * 0.1;
                let ea = eccentric_anomaly(mm, ecc);
                let back = ea - ecc * m::sin(ea);
                let want = mm - m::TAU * m::floor(mm / m::TAU);
                assert!((back - want).abs() < 1e-12, "e {ecc}, M {mm}");
            }
        }
        // Vis-viva holds along an inclined, eccentric orbit.
        let mut g = earth_sun();
        let earth: Address = "u:o/sys:0/body:3".parse().unwrap();
        let row = g.get_mut(&earth).unwrap();
        row.insert("orbit.eccentricity".into(), Value::Float(0.5));
        row.insert("orbit.inclination".into(), Value::Float(0.4));
        row.insert("orbit.ascending_node".into(), Value::Float(1.1));
        row.insert("orbit.argument_of_periapsis".into(), Value::Float(2.3));
        let times: Vec<i64> = (0..40).map(|k| k * 1_000_000).collect();
        let q = &orbits_at(&g, &sys, &times, None)[0];
        let mu = G * (1.988_47e30 + 5.972e24);
        for w in q.at.windows(2) {
            let (p0, p1) = (w[0].r_m, w[1].r_m);
            let v = [
                (p1[0] - p0[0]) / 1.0e6,
                (p1[1] - p0[1]) / 1.0e6,
                (p1[2] - p0[2]) / 1.0e6,
            ];
            let mid = r(&w[0]).min(r(&w[1]));
            let v2 = v[0] * v[0] + v[1] * v[1] + v[2] * v[2];
            let lo = mu * (2.0 / r(&w[0]).max(r(&w[1])) - 1.0 / a);
            let hi = mu * (2.0 / mid - 1.0 / a);
            assert!(
                v2 > lo * 0.97 && v2 < hi * 1.03,
                "vis-viva between the ends of each chord"
            );
        }
    }

    #[test]
    fn no_guessed_positions() {
        let mut f = earth_sun();
        let sys: Address = "u:o/sys:0".parse().unwrap();
        let earth: Address = "u:o/sys:0/body:3".parse().unwrap();
        f.get_mut(&earth)
            .unwrap()
            .remove("orbit.mean_anomaly_at_epoch");
        let p = orbits_at(&f, &sys, &[0], None);
        assert!(!p[0].placed && p[0].missing[0].contains("mean_anomaly"));
        let s = Seed::from_text("o");
        let a = orbits_at(&f, &sys, &[0, 100], Some(&s));
        let b = orbits_at(&f, &sys, &[0, 100], Some(&s));
        assert_eq!(a, b);
        assert_eq!(a[0].phase_source.as_deref(), Some("resolved"));
        let sun: Address = "u:o/sys:0/star:0".parse().unwrap();
        f.remove(&sun);
        let p = orbits_at(&f, &sys, &[0], Some(&s));
        assert!(
            !p[0].placed && p[0].missing[0].contains("body.mass"),
            "{:?}",
            p[0]
        );
    }

    #[test]
    fn calendars_with_named_days_weeks_and_leaps() {
        let cal = |v: serde_json::Value| -> Result<(), String> {
            serde_json::from_value::<Calendar>(v)
                .map_err(|e| e.to_string())
                .and_then(|c| c.check())
        };
        // The Imperial calendar: Holiday outside the weeks, weeks restart each year on day 2.
        let imperial = serde_json::json!({
            "name": "Imperial", "epoch_s": 0, "epoch_year": 1105, "day_s": 86400.0,
            "year_days": 365.0, "format": "{day:03}-{year}",
            "named_days": [{"day": 1, "name": "Holiday", "outside_weeks": true}],
            "week": {"names": ["Wonday", "Tuday", "Thirday", "Forday", "Fiday", "Sixday", "Senday"],
                     "first_day": 2, "continuous": false}
        });
        assert_eq!(cal(imperial.clone()), Ok(()));
        let mut v = imperial.clone();
        v["week"]["continuous"] = serde_json::json!(true);
        assert!(cal(v).unwrap_err().contains("outside_weeks"));
        let mut v = imperial.clone();
        v["named_days"][0]["day"] = serde_json::json!(366);
        assert!(cal(v).is_err());
        let mut v = imperial;
        v["week"]["shape"] = serde_json::json!(1);
        assert!(cal(v).is_err(), "unknown keys are still refused");
        // Gregorian: 4, 100, 400, with February gaining the day.
        let months: Vec<serde_json::Value> = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
            .iter()
            .enumerate()
            .map(|(i, d)| serde_json::json!({"name": format!("M{i}"), "days": d}))
            .collect();
        let greg = serde_json::json!({
            "name": "Gregorian", "epoch_s": 0, "day_s": 86400.0, "year_days": 365.0,
            "format": "{mday} {month} {year}", "months": months,
            "leap": {"every": 4, "except_every": 100, "but_every": 400}, "leap_month": 1,
            "named_days": [{"day": 366, "name": "Leap day end"}]
        });
        assert_eq!(cal(greg.clone()), Ok(()));
        let mut v = greg.clone();
        v.as_object_mut().unwrap().remove("leap_month");
        assert!(cal(v).unwrap_err().contains("leap_month"));
        let mut v = greg;
        v["leap"]["except_every"] = serde_json::json!(90);
        assert!(cal(v).is_err());
    }
}

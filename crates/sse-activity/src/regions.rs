//! SG7: active regions, the one place a star's magnetism breaks through its
//! surface. Spots, flares and active-region prominences all sit in them;
//! quiet prominences sit on the channels their field leaves behind as it
//! decays and drifts toward the poles. One list, read by everything, so a
//! flare goes off where the spots are and a filament lies between them.
//!
//! Each region is born, grows quickly and decays at a steady rate in area
//! (`sse.activity.spot_group_life`, the Gnevyshev-Waldmeier rule); it appears
//! at the latitude the cycle has reached (`sse.activity.butterfly_latitude`,
//! Spörer's law), its pair of opposite polarities tilted by Joy's law
//! (`sse.activity.joy_tilt`), the leading sign by Hale's law. It is carried
//! round at its latitude's rate (`sse.activity.differential_rotation`).
//! How many there are follows the star's spot coverage: births at a rate
//! that keeps the mean area covered equal to it, waxing and waning with the
//! cycle. Births are a Poisson process in fixed time bins on their own draw
//! stream, so any window gives the same regions where windows overlap, and
//! the regions alive at one moment can be found without a window at all.

use crate::{addr, cycle_factor, poisson, Activity, Settings};
use serde::{Deserialize, Serialize};
use sse_atoms::activity as act;
use sse_core::draw::{Draws, Stream};
use sse_core::math as m;
use sse_core::sheet::DAY;
use sse_flow::Reference;

/// Settings for active regions (stand-ins in the fudge register).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct RegionSettings {
    /// Peak areas of the Sun's spot groups, millionths of a hemisphere, drawn
    /// evenly in log between these (a stand-in for the measured lognormal).
    pub area_msh: [f64; 2],
    /// `sse.activity.spot_group_life`: decay, millionths of a hemisphere a day.
    pub decay_msh_day: f64,
    /// Share of its life a region spends growing to its peak.
    pub grow_share: f64,
    /// Longest a region lives, days.
    pub max_life_days: f64,
    /// `sse.activity.butterfly_latitude`: degrees at the cycle's start and end; spread either side.
    pub butterfly_start_deg: f64,
    pub butterfly_end_deg: f64,
    pub butterfly_spread_deg: f64,
    /// A star without a cycle: regions anywhere between these latitudes, degrees.
    pub no_cycle_lat_deg: [f64; 2],
    /// `sse.activity.joy_tilt`: degrees at the pole; scatter either side.
    pub joy_scale_deg: f64,
    pub joy_scatter_deg: f64,
    /// Distance between the two polarities, in radii of the group's spot area.
    pub separation_radii: f64,
    /// Share of the area in the leading spot.
    pub leader_share: f64,
    /// `sse.activity.differential_rotation`: A, B, C, degrees a day (the Sun's).
    pub differential: [f64; 3],
    /// Poleward drift of a decayed region's field, m/s.
    pub meridional_m_s: f64,
    /// How long, in turns, a decayed region's channel can hold quiet prominences.
    pub channel_life_turns: f64,
    /// Most regions listed for one window.
    pub max_listed: usize,
}

impl Default for RegionSettings {
    fn default() -> Self {
        RegionSettings {
            area_msh: [30.0, 3000.0],
            decay_msh_day: 10.0,
            grow_share: 0.1,
            max_life_days: 200.0,
            butterfly_start_deg: 28.0,
            butterfly_end_deg: 8.0,
            butterfly_spread_deg: 5.0,
            no_cycle_lat_deg: [15.0, 75.0],
            joy_scale_deg: 32.1,
            joy_scatter_deg: 10.0,
            separation_radii: 3.0,
            leader_share: 0.6,
            differential: [14.713, -2.396, -1.787],
            meridional_m_s: 15.0,
            channel_life_turns: 10.0,
            max_listed: 2000,
        }
    }
}

/// One active region.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Region {
    pub address: String,
    pub born_s: f64,
    /// Time from birth to its peak area.
    pub grow_s: f64,
    pub life_s: f64,
    /// Peak spot area, share of the whole star's surface.
    pub peak_share: f64,
    /// Peak spot area, millionths of a hemisphere.
    pub peak_msh: f64,
    pub lat_deg: f64,
    /// Longitude at birth in the star's own frame, which turns once a
    /// rotation period from age zero (degrees, 0 to 360).
    pub lon_deg: f64,
    /// Direction from the following polarity to the leading one, degrees
    /// from the direction of spin toward north (Joy's law: the leader
    /// nearer the equator).
    pub tilt_deg: f64,
    /// Sign of the leading polarity's field (Hale's law): +1 outward.
    pub leading_sign: i8,
    /// Distance between the polarities, degrees on the star.
    pub separation_deg: f64,
}

impl Region {
    /// Spot area now, share of the star's surface: a straight rise to the
    /// peak, then a steady decay to nothing.
    pub fn share_at(&self, t: f64) -> f64 {
        let age = t - self.born_s;
        if age < 0.0 || age > self.life_s {
            0.0
        } else if age < self.grow_s {
            self.peak_share * age / self.grow_s.max(1.0)
        } else {
            self.peak_share * (self.life_s - age) / (self.life_s - self.grow_s).max(1.0)
        }
    }

    /// Longitude now in the star's frame: carried at its latitude's rate,
    /// drifting against the equator's turn.
    pub fn lon_at(&self, t: f64, period_s: f64, set: &RegionSettings) -> f64 {
        if period_s.is_nan() || period_s <= 0.0 {
            return self.lon_deg;
        }
        let w = share(self.lat_deg, set);
        wrap(
            self.lon_deg + (w - 1.0) * 360.0 * (t - self.born_s) / period_s,
            360.0,
        )
    }

    pub fn end_s(&self) -> f64 {
        self.born_s + self.life_s
    }
}

/// A place on the star tied to a region.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Site {
    pub region: String,
    pub lat_deg: f64,
    /// In the star's own frame at that moment, degrees.
    pub lon_deg: f64,
}

/// The regions alive in a window.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Regions {
    pub star: String,
    pub from_s: f64,
    pub to_s: f64,
    pub rotation_period_s: f64,
    /// `sse.activity.differential_rotation` A, B, C (degrees a day; only the ratios are used).
    pub differential: [f64; 3],
    /// Share of the area in the leading spot of each pair.
    pub leader_share: f64,
    pub regions: Vec<Region>,
    /// Mean number alive at once over the cycle.
    pub mean_at_once: f64,
    pub why: Vec<String>,
    pub stand_ins: Vec<String>,
    pub references: Vec<Reference>,
}

/// The star's frame turns once a rotation period from age zero: its turn at
/// time t, degrees.
pub fn turn_deg(period_s: f64, t: f64) -> f64 {
    if period_s > 0.0 {
        360.0 * wrap(t / period_s, 1.0)
    } else {
        0.0
    }
}

/// x modulo n, from 0 up to n.
pub fn wrap(x: f64, n: f64) -> f64 {
    let r = m::fmod(x, n);
    if r < 0.0 {
        r + n
    } else {
        r
    }
}

fn share(lat_deg: f64, set: &RegionSettings) -> f64 {
    let [a, b, c] = set.differential;
    act::differential_rotation(lat_deg * m::PI / 180.0, a, b, c)
}

/// How a star's regions come: the area scale, the birth rate, the bin.
struct Plan {
    scale: f64,
    rate: f64,
    peak: f64,
    bin: f64,
    max_life: f64,
    mean_life: f64,
}

fn life_of(msh: f64, set: &RegionSettings) -> f64 {
    act::spot_group_life(msh, set.decay_msh_day).min(set.max_life_days * DAY)
}

fn plan(a: &Activity, set: &Settings) -> Option<Plan> {
    let r = &set.regions;
    if !a.flares || a.spot_coverage.is_nan() || a.spot_coverage <= 0.0 {
        return None;
    }
    // A more spotted star has bigger groups as well as more (Doppler images
    // show spots far larger than the Sun's): areas grow as the square root of
    // the coverage over the Sun's, counts with the rest.
    let scale = m::sqrt(a.spot_coverage / set.sun_spot_coverage).max(1.0);
    // Mean area-time of one region over its life, from even steps in log area.
    let (mut area_time, mut life) = (0.0, 0.0);
    let steps = 64;
    for k in 0..steps {
        let msh = scale * log_between((k as f64 + 0.5) / steps as f64, r.area_msh);
        let l = life_of(msh, r);
        area_time += msh * 0.5e-6 * l * 0.5;
        life += l;
    }
    area_time /= steps as f64;
    let mean_life = life / steps as f64;
    let rate = a.spot_coverage / area_time;
    let peak = 1.0
        + if a.cycle.is_some() {
            set.cycle_amplitude
        } else {
            0.0
        };
    let bin = m::pow(2.0, m::round(m::log2(mean_life / 4.0))).max(3600.0);
    Some(Plan {
        scale,
        rate,
        peak,
        bin,
        max_life: r.max_life_days * DAY,
        mean_life,
    })
}

fn log_between(u: f64, r: [f64; 2]) -> f64 {
    m::exp(m::ln(r[0]) + u * (m::ln(r[1]) - m::ln(r[0])))
}

/// Regions born between `t0` and `t1`.
fn born_between(
    seed: &sse_core::Seed,
    a: &Activity,
    pl: &Plan,
    t0: f64,
    t1: f64,
    set: &Settings,
) -> Result<Vec<Region>, String> {
    let r = &set.regions;
    let star = addr(&a.address)?;
    let stream = Stream::new("activity.regions.v1").map_err(|e| e.0)?;
    let dr = Draws::new(*seed, star, stream);
    let (b0, b1) = (m::floor(t0 / pl.bin) as i64, m::floor(t1 / pl.bin) as i64);
    let mut out = vec![];
    for b in b0..=b1 {
        let base = (b as u64).wrapping_mul(4096);
        let n = poisson(pl.rate * pl.peak * pl.bin, dr.unit(base + 4095), 400);
        for k in 0..n {
            let i = base + k * 10;
            let born = (b as f64 + dr.unit(i)) * pl.bin;
            if born < t0 || born > t1 {
                continue;
            }
            // Still a double date, like prominences (TP1 follow-up).
            let born_at = sse_core::time::Instant::from_seconds_f64(born)
                .unwrap_or(sse_core::time::Instant::ORIGIN);
            if dr.unit(i + 1) * pl.peak > cycle_factor(a, born_at, set) {
                continue;
            }
            let msh = pl.scale * log_between(dr.unit(i + 2), r.area_msh);
            let life = life_of(msh, r);
            let peak_share = msh * 0.5e-6;
            let hemi = if dr.unit(i + 4) < 0.5 { -1.0 } else { 1.0 };
            let (lat, number) = match &a.cycle {
                Some(c) => {
                    let x = born / c.period_s + c.phase_offset;
                    // The activity factor is lowest at phase 0.75: the cycle starts there.
                    let progress = wrap(x - 0.75, 1.0);
                    let mean = act::butterfly_latitude(
                        progress,
                        r.butterfly_start_deg,
                        r.butterfly_end_deg,
                    ) * 180.0
                        / m::PI;
                    let spread = r.butterfly_spread_deg * (2.0 * dr.unit(i + 5) - 1.0);
                    ((mean + spread).max(2.0), m::floor(x + 0.25) as i64)
                }
                None => (
                    r.no_cycle_lat_deg[0]
                        + dr.unit(i + 5) * (r.no_cycle_lat_deg[1] - r.no_cycle_lat_deg[0]),
                    0,
                ),
            };
            let lat = hemi * lat;
            let joy = act::joy_tilt(lat * m::PI / 180.0, r.joy_scale_deg) * 180.0 / m::PI
                + r.joy_scatter_deg * (2.0 * dr.unit(i + 6) - 1.0);
            // The leader is ahead in the spin and nearer the equator: toward
            // the south in the north, toward the north in the south.
            let tilt = -hemi * joy;
            let even = number.rem_euclid(2) == 0;
            let leading_sign = if (hemi > 0.0) == even { 1 } else { -1 };
            let radius = m::acos(1.0 - 2.0 * peak_share) * 180.0 / m::PI;
            out.push(Region {
                address: format!("{}/region:{}.{}", a.address, b, k),
                born_s: born,
                grow_s: r.grow_share * life,
                life_s: life,
                peak_share,
                peak_msh: msh,
                lat_deg: lat,
                lon_deg: dr.uniform(i + 3, 0.0, 360.0),
                tilt_deg: tilt,
                leading_sign,
                separation_deg: r.separation_radii * radius,
            });
        }
    }
    Ok(out)
}

/// The regions alive at time t.
pub fn alive_at(
    seed: &sse_core::Seed,
    a: &Activity,
    t: f64,
    set: &Settings,
) -> Result<Vec<Region>, String> {
    let Some(pl) = plan(a, set) else {
        return Ok(vec![]);
    };
    Ok(born_between(seed, a, &pl, t - pl.max_life, t, set)?
        .into_iter()
        .filter(|g| g.end_s() >= t)
        .collect())
}

/// Pick a region alive at t by its spot area now (`u` from 0 to 1), and the
/// place between its two polarities, where flares go off and filaments lie.
pub fn site_at(
    seed: &sse_core::Seed,
    a: &Activity,
    t: f64,
    u: f64,
    set: &Settings,
) -> Result<Option<(Site, Region)>, String> {
    let alive = alive_at(seed, a, t, set)?;
    let total: f64 = alive.iter().map(|g| g.share_at(t)).sum();
    if total.is_nan() || total <= 0.0 {
        return Ok(None);
    }
    let mut acc = 0.0;
    for g in &alive {
        acc += g.share_at(t);
        if u * total <= acc {
            let site = Site {
                region: g.address.clone(),
                lat_deg: g.lat_deg,
                lon_deg: g.lon_at(t, a.rotation_period_s, &set.regions),
            };
            return Ok(Some((site, g.clone())));
        }
    }
    let g = alive.last().expect("total above zero");
    Ok(Some((
        Site {
            region: g.address.clone(),
            lat_deg: g.lat_deg,
            lon_deg: g.lon_at(t, a.rotation_period_s, &set.regions),
        },
        g.clone(),
    )))
}

/// A decayed region's channel at t, picked by its peak area (`u` from 0 to
/// 1): where its field has drifted to by now, toward the pole at the
/// meridional flow's speed, no further than `crown_deg`. Its longitude is
/// where the region was when it died.
pub fn channel_at(
    seed: &sse_core::Seed,
    a: &Activity,
    t: f64,
    u: f64,
    radius_m: f64,
    crown_deg: f64,
    set: &Settings,
) -> Result<Option<Site>, String> {
    let Some(pl) = plan(a, set) else {
        return Ok(None);
    };
    let r = &set.regions;
    let keep = r.channel_life_turns * a.rotation_period_s;
    let dead: Vec<Region> = born_between(seed, a, &pl, t - keep - pl.max_life, t, set)?
        .into_iter()
        .filter(|g| g.end_s() <= t && g.end_s() >= t - keep)
        .collect();
    let total: f64 = dead.iter().map(|g| g.peak_share).sum();
    if total.is_nan() || total <= 0.0 {
        return Ok(None);
    }
    let mut acc = 0.0;
    let mut pick = dead.last().expect("total above zero");
    for g in &dead {
        acc += g.peak_share;
        if u * total <= acc {
            pick = g;
            break;
        }
    }
    let drift = r.meridional_m_s * (t - pick.end_s()) / radius_m * 180.0 / m::PI;
    let lat = (pick.lat_deg.abs() + drift).min(crown_deg.max(pick.lat_deg.abs()));
    Ok(Some(Site {
        region: pick.address.clone(),
        lat_deg: lat * if pick.lat_deg < 0.0 { -1.0 } else { 1.0 },
        lon_deg: pick.lon_at(pick.end_s(), a.rotation_period_s, r),
    }))
}

const HATHAWAY15: &str = "Hathaway, D. H. 2015, The solar cycle, Living Reviews in Solar Physics 12, 4: spot groups appear near 25 to 30 degrees latitude early in a cycle and drift toward about 8 degrees by its end (Spörer's law); Joy's law and Hale's polarity law; a poleward meridional flow of 10 to 20 m/s carries decayed field toward the poles. https://arxiv.org/abs/1502.07020";
const BAUMANN05: &str = "Baumann, I. and Solanki, S. K. 2005, On the size distribution of sunspot groups in the Greenwich sunspot record 1874-1976, A&A 443, 1061: peak group areas follow a lognormal distribution from tens to thousands of millionths of a hemisphere. https://arxiv.org/abs/astro-ph/0508207";
const STRASSMEIER09: &str = "Strassmeier, K. G. 2009, Starspots, Astronomy and Astrophysics Review 17, 251: Doppler images of active stars show spots far larger than the Sun's, often at high latitudes and the poles. doi:10.1007/s00159-009-0020-6";

fn refs_of(ids: &[&str]) -> Vec<Reference> {
    let mut out: Vec<Reference> = vec![];
    for d in sse_atoms::decls_of(ids) {
        for r in d.references {
            if !out.contains(&r) {
                out.push(r);
            }
        }
    }
    out
}

/// The regions alive at any time in a window.
pub fn regions(
    seed: &sse_core::Seed,
    a: &Activity,
    from_s: f64,
    to_s: f64,
    set: &Settings,
) -> Result<Regions, String> {
    if to_s < from_s {
        return Err("the window must not run backwards".into());
    }
    let r = &set.regions;
    let mut out = Regions {
        star: a.address.clone(),
        from_s,
        to_s,
        rotation_period_s: a.rotation_period_s,
        differential: r.differential,
        leader_share: r.leader_share,
        regions: vec![],
        mean_at_once: 0.0,
        why: vec![],
        stand_ins: vec![
            format!("spot group peak areas drawn evenly in log between {} and {} millionths of a hemisphere (the Sun's), scaled by the square root of the star's coverage over the Sun's", r.area_msh[0], r.area_msh[1]),
            format!("a region grows over {} of its life then decays steadily; at most {} days", r.grow_share, r.max_life_days),
            format!("the butterfly line is straight from {}° to {}° through the cycle, spread {}° either side; a star without a cycle has regions anywhere from {}° to {}°", r.butterfly_start_deg, r.butterfly_end_deg, r.butterfly_spread_deg, r.no_cycle_lat_deg[0], r.no_cycle_lat_deg[1]),
            format!("Joy's law scattered {}° either side; polarities {} spot radii apart, {} of the area in the leader", r.joy_scatter_deg, r.separation_radii, r.leader_share),
            format!("the Sun's differential rotation, meridional flow of {} m/s and channels lasting {} turns, for every star", r.meridional_m_s, r.channel_life_turns),
        ],
        references: vec![
            Reference::parse(HATHAWAY15),
            Reference::parse(BAUMANN05),
            Reference::parse(STRASSMEIER09),
        ],
    };
    for x in refs_of(&[
        "sse.activity.differential_rotation",
        "sse.activity.butterfly_latitude",
        "sse.activity.joy_tilt",
        "sse.activity.spot_group_life",
    ]) {
        if !out.references.contains(&x) {
            out.references.push(x);
        }
    }
    let Some(pl) = plan(a, set) else {
        out.why
            .push("No churning outer layer breaking through: no active regions.".into());
        return Ok(out);
    };
    out.mean_at_once = pl.rate * pl.mean_life;
    let expected = pl.rate * pl.peak * (to_s - from_s + pl.max_life);
    if expected > 20.0 * r.max_listed as f64 {
        return Err(format!(
            "about {expected:.0} active regions in this window: ask for a shorter one"
        ));
    }
    let mut list: Vec<Region> = born_between(seed, a, &pl, from_s - pl.max_life, to_s, set)?
        .into_iter()
        .filter(|g| g.end_s() >= from_s)
        .collect();
    list.sort_by(|x, y| x.born_s.total_cmp(&y.born_s));
    if list.len() > r.max_listed {
        let cut = list.len() - r.max_listed;
        out.why.push(format!(
            "{cut} more regions in this window are left out: ask for a shorter one."
        ));
        list.truncate(r.max_listed);
    }
    out.why.push(format!(
        "About {:.1} active regions at once on average, each living {:.0} days on average: they cover {:.2} % of the surface at the cycle's mean, as its spots do.",
        out.mean_at_once,
        pl.mean_life / DAY,
        100.0 * a.spot_coverage
    ));
    if a.cycle.is_some() {
        out.why.push("New regions break out at mid latitudes early in each cycle and nearer the equator as it goes on; each pair is tilted with its leader toward the equator, the leader's sign flipping between hemispheres and cycles.".into());
    } else {
        out.why.push("Without a cycle the regions break out at any latitude, as on fast-spinning active stars.".into());
    }
    out.regions = list;
    Ok(out)
}

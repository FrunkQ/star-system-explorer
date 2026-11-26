//! The roster: for each class and size band, a keyed shuffle of its members
//! laid against the drain sketch, so it reads both ways at the cost of a few
//! sums. Forwards: "member k leaves in this bin, for this world". Backwards:
//! "the j-th arrival on this world in this bin is member k". Nobody else in
//! the band is visited either way.
//!
//! Layout of a band's positions: bin by bin; inside a bin, the background
//! drain first, then each close pass's kick in time order; inside each of
//! those, sink by sink. The shuffle maps a position to a rank.

use crate::draw::binomial;
use crate::perm::{mix, unit, Perm};
use crate::system::*;
use serde::Serialize;
use sse_core::math as m;
use std::collections::BTreeMap;
use std::rc::Rc;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Serialize)]
pub struct BandId {
    pub res: usize,
    pub class: usize,
    pub band: usize,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Serialize)]
pub enum Cause {
    Background,
    Pass(usize),
}

#[derive(Clone, Debug)]
pub struct BandPlan {
    pub n: u64,
    pub k0: u64,
    pub key: u64,
    pub perm: Perm,
    /// Position where each bin's leavers start; the last entry is how many leave by the horizon.
    pub starts: Vec<u64>,
    /// bin -> [(pass index, members it kicked out)]
    pub kicks: BTreeMap<usize, Vec<(usize, u64)>>,
}

impl BandPlan {
    pub fn bytes(&self) -> usize {
        std::mem::size_of::<Self>() + self.starts.len() * 8 + self.kicks.len() * 48
    }
    /// The cause segments of one bin: (cause, first offset, count).
    pub fn segs(&self, b: usize) -> Vec<(Cause, u64, u64)> {
        let total = self.starts[b + 1] - self.starts[b];
        let kicks = self.kicks.get(&b);
        let kicked: u64 = kicks.map(|v| v.iter().map(|x| x.1).sum()).unwrap_or(0);
        let mut out = vec![(Cause::Background, 0, total - kicked)];
        let mut at = total - kicked;
        if let Some(v) = kicks {
            for (pi, c) in v {
                out.push((Cause::Pass(*pi), at, *c));
                at += c;
            }
        }
        out
    }
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Fate {
    pub bin: usize,
    pub cause: Cause,
    pub seg: usize,
    pub sink: usize,
    pub j: u64,
    pub t_leave: f64,
    pub t_arrive: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Body {
    pub name: String,
    pub band: BandId,
    pub rank: u64,
    pub diameter_km: f64,
    pub mass_kg: f64,
    pub sub: u64,
    pub groups_sub: [u64; 6],
    pub type_label: String,
    pub a_au: f64,
    pub e: f64,
    pub i_deg: f64,
    pub fate: Option<Fate>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Crater {
    pub id: String,
    pub world: usize,
    pub d_km: f64,
    pub lat_deg: f64,
    pub lon_deg: f64,
    pub t_myr: f64,
    pub v_kms: f64,
    pub band: BandId,
    pub rank: u64,
}

pub struct Roster<'a> {
    pub sys: &'a System,
    plans: BTreeMap<BandId, Rc<BandPlan>>,
    /// Red control only (CC_BREAK=stateful): key each band from a running
    /// counter, as a stateful random stream would, so answers depend on order.
    pub break_stateful: bool,
    counter: u64,
}

impl<'a> Roster<'a> {
    pub fn new(sys: &'a System) -> Self {
        Roster {
            sys,
            plans: BTreeMap::new(),
            break_stateful: std::env::var("CC_BREAK").is_ok_and(|v| v == "stateful"),
            counter: 0,
        }
    }
    pub fn cache_bytes(&self) -> usize {
        self.plans.values().map(|p| p.bytes() + 32).sum()
    }
    pub fn clear(&mut self) {
        self.plans.clear();
    }

    pub fn band_of(&self, res: usize, class: usize, rank: u64) -> BandId {
        let c = &self.sys.reservoirs[res].classes[class];
        let band = c
            .bands
            .iter()
            .position(|(a, b)| rank >= *a && rank < *b)
            .expect("rank on the roster");
        BandId { res, class, band }
    }

    pub fn plan(&mut self, id: BandId) -> Rc<BandPlan> {
        if let Some(p) = self.plans.get(&id) {
            return p.clone();
        }
        let mut p = build_plan(self.sys, id);
        if self.break_stateful {
            self.counter += 1;
            p.key = mix(self.counter);
            p.perm = Perm::new(p.n, p.key);
        }
        let p = Rc::new(p);
        self.plans.insert(id, p.clone());
        p
    }

    /// Members of one (bin, cause) segment per sink: conditional binomials, exact sum.
    pub fn sink_counts(&self, id: BandId, key: u64, b: usize, seg: usize, count: u64) -> Vec<u64> {
        let r = &self.sys.reservoirs[id.res];
        let k2 = mix(key ^ 0xA5A5_A5A5);
        let base = ((b as u64) << 24) | ((seg as u64) << 8);
        let mut rem = count;
        let mut ptot = 1.0;
        let n = r.sinks.len();
        let mut out = vec![0; n];
        for (s, (_, p)) in r.sinks.iter().enumerate() {
            if rem == 0 {
                break;
            }
            if s + 1 == n {
                out[s] = rem;
                break;
            }
            let q = if ptot > 0.0 { (p / ptot).min(1.0) } else { 0.0 };
            let c = binomial(rem, q, k2, base + s as u64);
            out[s] = c;
            rem -= c;
            ptot -= p;
        }
        out
    }

    /// Forwards: what happens to member `rank`.
    pub fn fate(&mut self, id: BandId, rank: u64) -> Option<Fate> {
        let sys = self.sys;
        let plan = self.plan(id);
        let i = rank - plan.k0;
        let p = plan.perm.index_of(i);
        let nb = sys.nbins();
        if p >= plan.starts[nb] {
            return None;
        }
        // Last bin whose start is <= p (bins can be empty).
        let b = plan.starts.partition_point(|s| *s <= p) - 1;
        let off = p - plan.starts[b];
        let segs = plan.segs(b);
        let (si, (cause, s0, cnt)) = segs
            .iter()
            .enumerate()
            .find(|(_, (_, s0, c))| off >= *s0 && off < s0 + c)
            .map(|(i, x)| (i, *x))
            .expect("offset inside a segment");
        let counts = self.sink_counts(id, plan.key, b, si, cnt);
        let mut o = off - s0;
        let mut sink = 0;
        for (s, c) in counts.iter().enumerate() {
            if o < *c {
                sink = s;
                break;
            }
            o -= c;
        }
        Some(self.timed(id, rank, b, cause, si, sink, o))
    }

    #[allow(clippy::too_many_arguments)]
    pub fn timed(
        &self,
        id: BandId,
        rank: u64,
        b: usize,
        cause: Cause,
        seg: usize,
        sink: usize,
        j: u64,
    ) -> Fate {
        let sys = self.sys;
        let r = &sys.reservoirs[id.res];
        let c = &r.classes[id.class];
        let mk = mix(c.key ^ mix(rank) ^ 0x77);
        let t_leave = match cause {
            Cause::Background => {
                let lo = sys.edges[b].max(r.t0_myr);
                let hi = sys.edges[b + 1].max(lo);
                lo + (hi - lo) * unit(mk, 0)
            }
            Cause::Pass(pi) => sys.passes[pi].t_myr,
        };
        let travel = match r.travel {
            Travel::LogUniform(lo, hi) => lo * m::pow(hi / lo, unit(mk, 1)),
            Travel::HalfPeriod => {
                let a = orbit(c, rank).0;
                0.5 * m::pow(a, 1.5) / m::sqrt(sys.star_msun) / 1e6
            }
        };
        Fate {
            bin: b,
            cause,
            seg,
            sink,
            j,
            t_leave,
            t_arrive: t_leave + travel,
        }
    }

    /// Backwards: the j-th member of (bin, segment, sink).
    pub fn member_at(&mut self, id: BandId, b: usize, seg: usize, sink: usize, j: u64) -> u64 {
        let plan = self.plan(id);
        let segs = plan.segs(b);
        let (_, s0, cnt) = segs[seg];
        let counts = self.sink_counts(id, plan.key, b, seg, cnt);
        assert!(j < counts[sink], "no such arrival");
        let p = plan.starts[b] + s0 + counts[..sink].iter().sum::<u64>() + j;
        plan.k0 + plan.perm.at(p)
    }

    pub fn body(&mut self, id: BandId, rank: u64) -> Body {
        let fate = self.fate(id, rank);
        let sys = self.sys;
        let r = &sys.reservoirs[id.res];
        let c = &r.classes[id.class];
        let sub = member_sub(c, rank);
        let shares: Vec<f64> = c.groups.iter().map(|x| *x as f64).collect();
        let g = split_exact(sub, &shares);
        let mut groups_sub = [0u64; 6];
        groups_sub.copy_from_slice(&g);
        let mass_kg = sub as f64 * SUB_KG;
        let diameter_km = m::cbrt(6.0 * mass_kg / (m::PI * c.density)) / 1000.0;
        let t: f64 = c.groups.iter().sum::<u64>() as f64;
        let f = |i: usize| c.groups[i] as f64 / t;
        let type_label = if f(0) > 0.5 {
            "M (metal)"
        } else if f(3) + f(4) + f(5) > 0.3 {
            "icy"
        } else if f(2) > 0.2 {
            "C (carbon-rich)"
        } else {
            "S (stony)"
        }
        .to_string();
        let o = orbit(c, rank);
        Body {
            name: format!("{}/{}/{}", r.name, c.name, rank),
            band: id,
            rank,
            diameter_km,
            mass_kg,
            sub,
            groups_sub,
            type_label,
            a_au: o.0,
            e: o.1,
            i_deg: o.2,
            fate,
        }
    }

    /// Lifetime arrivals of one band on one sink: (bin, segment, count).
    pub fn arrivals(&mut self, id: BandId, sink: usize) -> Vec<(usize, usize, u64)> {
        let plan = self.plan(id);
        let mut out = vec![];
        for b in 0..self.sys.nbins() {
            if plan.starts[b + 1] == plan.starts[b] {
                continue;
            }
            for (si, (_, _, cnt)) in plan.segs(b).iter().enumerate() {
                if *cnt == 0 {
                    continue;
                }
                let c = self.sink_counts(id, plan.key, b, si, *cnt)[sink];
                if c > 0 {
                    out.push((b, si, c));
                }
            }
        }
        out
    }

    pub fn crater_of(&self, world: usize, body: &Body) -> Crater {
        let sys = self.sys;
        let w = &sys.worlds[world];
        let r = &sys.reservoirs[body.band.res];
        let c = &r.classes[body.band.class];
        let f = body.fate.as_ref().expect("an arrival has a fate");
        let v = m::sqrt(r.v_inf_kms * r.v_inf_kms + w.v_esc_kms * w.v_esc_kms);
        // Collins, Melosh and Marcus 2005, eq. 21 (transient), 45 degree impact; 1.25 for a simple final crater.
        let l = body.diameter_km * 1000.0;
        let dt = 1.161
            * m::cbrt(c.density / 3000.0)
            * m::pow(l, 0.78)
            * m::pow(v * 1000.0, 0.44)
            * m::pow(w.g, -0.22)
            * m::cbrt(m::sin(m::PI / 4.0));
        let dc = 3200.0 * 9.81 / w.g;
        let d = if 1.25 * dt < dc {
            1.25 * dt
        } else {
            1.17 * m::pow(dt, 1.13) / m::pow(dc, 0.13)
        };
        let mk = mix(c.key ^ mix(body.rank) ^ 0x99);
        let z = 2.0 * unit(mk, 0) - 1.0;
        let lon = 360.0 * unit(mk, 1) - 180.0;
        let id = body.band;
        Crater {
            id: format!(
                "{}/crater/{}.{}.{}/{}/{}/{}/{}",
                w.id, id.res, id.class, id.band, f.bin, f.seg, f.sink, f.j
            ),
            world,
            d_km: d / 1000.0,
            lat_deg: m::asin(z).to_degrees(),
            lon_deg: lon,
            t_myr: f.t_arrive,
            v_kms: v,
            band: id,
            rank: body.rank,
        }
    }

    /// As `impactor_of`, but None when the id names no arrival.
    pub fn try_impactor_of(&mut self, crater_id: &str) -> Option<Body> {
        let parts: Vec<&str> = crater_id.rsplitn(6, '/').collect();
        let j: u64 = parts[0].parse().ok()?;
        let sink: usize = parts[1].parse().ok()?;
        let seg: usize = parts[2].parse().ok()?;
        let bin: usize = parts[3].parse().ok()?;
        let rcb: Vec<usize> = parts[4].split('.').filter_map(|x| x.parse().ok()).collect();
        let id = BandId {
            res: rcb[0],
            class: rcb[1],
            band: rcb[2],
        };
        let plan = self.plan(id);
        let segs = plan.segs(bin);
        let (_, _, cnt) = *segs.get(seg)?;
        if j >= self.sink_counts(id, plan.key, bin, seg, cnt)[sink] {
            return None;
        }
        Some(self.impactor_of(crater_id))
    }

    /// Crater id back to its impactor (no listing needed).
    pub fn impactor_of(&mut self, crater_id: &str) -> Body {
        let parts: Vec<&str> = crater_id.rsplitn(6, '/').collect();
        // parts: j, sink, seg, bin, "r.c.b", "<world>/crater"
        let j: u64 = parts[0].parse().unwrap();
        let sink: usize = parts[1].parse().unwrap();
        let seg: usize = parts[2].parse().unwrap();
        let bin: usize = parts[3].parse().unwrap();
        let rcb: Vec<usize> = parts[4].split('.').map(|x| x.parse().unwrap()).collect();
        let id = BandId {
            res: rcb[0],
            class: rcb[1],
            band: rcb[2],
        };
        let k = self.member_at(id, bin, seg, sink, j);
        self.body(id, k)
    }

    /// The sink index of a world in a reservoir.
    pub fn sink_of_world(&self, res: usize, world: usize) -> Option<usize> {
        self.sys.reservoirs[res]
            .sinks
            .iter()
            .position(|(k, _)| *k == SinkKind::World(world))
    }

    /// Every band in the system, largest members first.
    pub fn all_bands(&self) -> Vec<(BandId, f64)> {
        let mut v = vec![];
        for (ri, r) in self.sys.reservoirs.iter().enumerate() {
            for (ci, c) in r.classes.iter().enumerate() {
                for (bi, (_, k1)) in c.bands.iter().enumerate() {
                    let sub = member_sub(c, k1 - 1);
                    let d = m::cbrt(6.0 * sub as f64 * SUB_KG / (m::PI * c.density)) / 1000.0;
                    v.push((
                        BandId {
                            res: ri,
                            class: ci,
                            band: bi,
                        },
                        d,
                    ));
                }
            }
        }
        v.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.0.cmp(&b.0)));
        v
    }

    /// The listed craters of a world: every arrival in the window [from, to]
    /// from the largest members down to the size where the list would pass
    /// `max`. The size is chosen on the whole window's counts, so the list
    /// does not depend on the slider; `epoch` then keeps those that have
    /// happened. Returns the craters, the impactor size floor (km) and how
    /// many members were enumerated to find them.
    pub fn listed_craters(
        &mut self,
        world: usize,
        window: (f64, f64),
        epoch: f64,
        max: u64,
    ) -> (Vec<Crater>, f64, u64) {
        let sys = self.sys;
        let (b_from, b_to) = (
            sys.bin_of((window.0 - 100.0).max(0.0)),
            sys.bin_of(window.1),
        );
        let bands = self.all_bands();
        let mut acc = 0u64;
        let mut floor_km = f64::INFINITY;
        let mut chosen = vec![];
        let mut rest = vec![];
        let mut full = true;
        for (id, dmin) in bands {
            let Some(sink) = self.sink_of_world(id.res, world) else {
                continue;
            };
            let a: Vec<_> = self
                .arrivals(id, sink)
                .into_iter()
                .filter(|x| x.0 >= b_from && x.0 <= b_to)
                .collect();
            let n: u64 = a.iter().map(|x| x.2).sum();
            if full && acc + n <= max {
                acc += n;
                floor_km = dmin;
                chosen.push((id, sink, a));
            } else {
                full = false;
                rest.push((id, sink, a, n));
            }
        }
        let mut out = vec![];
        let mut walked = 0u64;
        let keep = |ro: &mut Self,
                    id: BandId,
                    sink: usize,
                    a: &[(usize, usize, u64)],
                    min_d: f64,
                    out: &mut Vec<Crater>,
                    walked: &mut u64| {
            for (b, seg, n) in a {
                for j in 0..*n {
                    let k = ro.member_at(id, *b, *seg, sink, j);
                    *walked += 1;
                    let body = ro.body(id, k);
                    if body.diameter_km < min_d {
                        continue;
                    }
                    let c = ro.crater_of(world, &body);
                    if c.t_myr >= window.0 && c.t_myr <= window.1 && c.t_myr <= epoch {
                        out.push(c);
                    }
                }
            }
        };
        for (id, sink, a) in &chosen {
            keep(self, *id, *sink, a, 0.0, &mut out, &mut walked);
        }
        // Bands that straddle the floor: their larger members belong on the list too.
        for (id, sink, a, n) in &rest {
            let c = &sys.reservoirs[id.res].classes[id.class];
            let top = member_sub(c, c.bands[id.band].0);
            let dmax = m::cbrt(6.0 * top as f64 * SUB_KG / (m::PI * c.density)) / 1000.0;
            if dmax < floor_km {
                continue;
            }
            assert!(*n <= 2_000_000, "a straddling band holds {n} arrivals");
            keep(self, *id, *sink, a, floor_km, &mut out, &mut walked);
        }
        out.sort_by(|a, b| b.d_km.total_cmp(&a.d_km).then(a.id.cmp(&b.id)));
        (out, floor_km, walked)
    }
}

/// A member's orbit from the class's laws, keyed by the class and rank
/// (the kernel's own `Cloud::resolve` costs three hashes; this costs none).
pub fn orbit(c: &ClassDef, rank: u64) -> (f64, f64, f64) {
    use sse_formation::leftovers::{Spread, Tilt};
    let mk = mix(c.key ^ mix(rank) ^ 0x0b17);
    let cl = &c.cloud;
    let a = if cl.a_log {
        cl.a_min_au * m::pow(cl.a_max_au / cl.a_min_au, unit(mk, 0))
    } else {
        cl.a_min_au + (cl.a_max_au - cl.a_min_au) * unit(mk, 0)
    };
    let ray = |u: f64, s: f64| s * m::sqrt(-2.0 * m::ln(1.0 - u));
    let e = match &cl.e {
        Spread::Rayleigh { sigma } => ray(unit(mk, 1), *sigma),
        Spread::Uniform { lo, hi } => lo + (hi - lo) * unit(mk, 1),
        Spread::Thermal => m::sqrt(unit(mk, 1)),
    }
    .min(0.99);
    let i = match &cl.i {
        Tilt::Rayleigh { sigma_deg } => ray(unit(mk, 2), *sigma_deg).min(180.0),
        Tilt::Isotropic => m::acos(1.0 - 2.0 * unit(mk, 2)) * 180.0 / m::PI,
    };
    (a, e, i)
}

pub fn member_sub(c: &ClassDef, k: u64) -> u64 {
    m::floor(c.total_sub as f64 * weight(k, c.cloud.n_top) / c.cloud.norm) as u64
}

pub fn build_plan(sys: &System, id: BandId) -> BandPlan {
    let r = &sys.reservoirs[id.res];
    let c = &r.classes[id.class];
    let (k0, k1) = c.bands[id.band];
    let n = k1 - k0;
    let key = mix(c.key ^ mix(id.band as u64 + 1));
    let nb = sys.nbins();
    let mut starts = vec![0u64; nb + 1];
    let mut kicks = BTreeMap::new();
    let mut remaining = n;
    for b in 0..nb {
        starts[b] = n - remaining;
        if remaining == 0 {
            continue;
        }
        let s0 = sys.survival(r, sys.edges[b]);
        let s1 = sys.survival(r, sys.edges[b + 1]);
        let p = if s0 > 0.0 { 1.0 - s1 / s0 } else { 0.0 };
        let bg = binomial(remaining, p, key, (b as u64) << 8);
        remaining -= bg;
        if r.kicked {
            let mut v = vec![];
            for (j, (pi, pass)) in sys.passes_in_bin(b).enumerate() {
                if pass.t_myr < r.t0_myr || remaining == 0 {
                    continue;
                }
                let cnt = binomial(remaining, pass.kick, key, ((b as u64) << 8) + 1 + j as u64);
                remaining -= cnt;
                v.push((pi, cnt));
            }
            if !v.is_empty() {
                kicks.insert(b, v);
            }
        }
    }
    starts[nb] = n - remaining;
    BandPlan {
        n,
        k0,
        key,
        perm: Perm::new(n, key),
        starts,
        kicks,
    }
}

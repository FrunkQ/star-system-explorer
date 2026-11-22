//! In-system voyages (V8 part 3b, voyages design 3.1): the same finder as
//! between the stars, with places that are a system's bodies and its belt
//! regions, and legs priced by NAV's own courses. Body to body is
//! `plan` (Hohmann and Lambert transfers with their launch windows, torch
//! courses for sustained drives, escape and capture at each end); a belt is
//! a ring the ship can meet anywhere, so legs to and from it are Hohmann
//! transfers (or torch flights) to its middle radius with no wait for a
//! window. Refuel stops are pack rules read off the bodies: a giant to skim,
//! ice beyond the snow line, a port.

use crate::courses::{end_burn, hohmann_dv, hohmann_time};
use crate::voyage::{dist, Goal, Leg, LegMaker, Options, Voyage, VoyageSystem};
use crate::{plan as nav_plan, Endpoint, NavSettings, NavSystem, Prefer, RouteRequest, Ship};
use serde::{Deserialize, Serialize};
use sse_core::math as m;
use sse_core::quantity::consts::{AU, DAY};

/// Astronomical units in a parsec (IAU: 648000/π).
pub const AU_PER_PC: f64 = 206_264.806_247_096_37;

/// Pack rules for in-system voyages (`nav.voyage.in_system`).
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct InSystemSettings {
    /// How long a leg may wait for its window, days.
    pub window_days: f64,
    /// A body at least this massive can be skimmed for fuel (Earth masses).
    pub skim_min_mass_earth: f64,
    /// The snow line at one solar luminosity, au; it scales as √L (Hayashi 1981).
    pub snow_line_au: f64,
    /// Kept labels per place (NAV courses are dearer than jumps).
    pub labels_per_place: usize,
}

impl Default for InSystemSettings {
    fn default() -> Self {
        InSystemSettings {
            window_days: 800.0,
            skim_min_mass_earth: 10.0,
            snow_line_au: 2.7,
            labels_per_place: 2,
        }
    }
}

/// A place's id: a body's address, or `belt:<name>`.
fn belt_id(name: &str) -> String {
    format!("belt:{name}")
}

/// The places of a system for the finder: bodies, then belts, with the fuel
/// each offers by the pack's rules (everything in the system is known to a
/// crew that is in it; a player's system already holds only what it saw).
pub fn places(sys: &NavSystem, set: &InSystemSettings) -> Vec<VoyageSystem> {
    let snow = set.snow_line_au * m::sqrt(sys.star_luminosity_lsun.max(0.0));
    let at = |a_au: f64| [a_au / AU_PER_PC, 0.0, 0.0];
    let mut out: Vec<VoyageSystem> = sys
        .bodies
        .iter()
        .map(|b| {
            let mut fuel = vec![];
            if b.port {
                fuel.push(if b.stock.is_empty() {
                    "port".to_string()
                } else {
                    format!("port:{}", b.stock.join(","))
                });
            }
            if b.mass_earth >= set.skim_min_mass_earth {
                fuel.push("skim".to_string());
            } else if b.a_au >= snow {
                fuel.push("ice".to_string());
            }
            VoyageSystem {
                id: b.address.clone(),
                name: b.name.clone(),
                xyz_pc: at(b.a_au),
                fuel,
                surveyed: true,
                fuel_odds: None,
                odds_from: String::new(),
                star: None,
            }
        })
        .collect();
    for belt in &sys.belts {
        let mid = 0.5 * (belt.a_in_au + belt.a_out_au);
        out.push(VoyageSystem {
            id: belt_id(&belt.name),
            name: belt.name.clone(),
            xyz_pc: at(mid),
            // Ice where any of the belt lies beyond the snow line (Ceres, at
            // 2.8 au, holds water ice).
            fuel: if belt.a_out_au >= snow {
                vec!["ice".into()]
            } else {
                vec![]
            },
            surveyed: true,
            fuel_odds: None,
            odds_from: String::new(),
            star: None,
        });
    }
    out
}

pub struct InSystem<'a> {
    pub nav: &'a NavSettings,
    pub sys: &'a NavSystem,
    pub ship: &'a Ship,
    pub places: &'a [VoyageSystem],
    /// Universe seconds when the voyage starts.
    pub t0_s: f64,
    pub prefer: Prefer,
    pub set: &'a InSystemSettings,
}

impl InSystem<'_> {
    fn radius_au(&self, i: usize) -> f64 {
        self.places[i].xyz_pc[0] * AU_PER_PC
    }
    fn body(&self, i: usize) -> Option<&crate::NavBody> {
        self.sys.body(&self.places[i].id)
    }
    fn with_propellant(&self, pr: f64) -> Ship {
        Ship {
            propellant_t: Some(pr),
            ..self.ship.clone()
        }
    }
    /// A leg to or from a belt: a Hohmann transfer between the two radii
    /// (escape or capture at a body's end), or a torch flight with turnover
    /// across the gap for a sustained drive. No window: the ring is met
    /// wherever the ship arrives.
    fn belt_legs(&self, from: usize, to: usize, pr: f64) -> Vec<Leg> {
        let ship = self.with_propellant(pr);
        let mu = self.sys.mu();
        let (r1, r2) = (self.radius_au(from) * AU, self.radius_au(to) * AU);
        let usable = ship.usable_dv_m_s();
        let mut out = vec![];
        let (fb, tb) = (self.body(from), self.body(to));
        let (d1, d2) = hohmann_dv(mu, r1, r2);
        let dv = end_burn(self.nav, fb, d1) + end_burn(self.nav, tb, d2);
        let t = hohmann_time(mu, r1, r2);
        let mut push = |kind: &str, dv: f64, t: f64| {
            if dv <= usable {
                let used = ship.propellant_for(dv);
                out.push(Leg {
                    means: "course".into(),
                    course: Some(kind.into()),
                    wait_days: Some(0.0),
                    home_days: t / DAY,
                    ship_days: t / DAY,
                    propellant_t: used,
                    propellant_left_t: pr - used,
                    ..Leg::between(
                        &self.places[from].id,
                        &self.places[to].id,
                        (r2 - r1).abs() / AU / AU_PER_PC,
                    )
                });
            }
        };
        push("hohmann", dv, t);
        if ship.sustained && ship.accel_g > 0.0 {
            let a = ship.accel_g.min(ship.g_limit) * crate::G0;
            let gap = (r2 - r1).abs();
            // Speeds to match at the ends (orbital speeds, escape and capture)
            // come first; what is left buys cruise speed, burnt half each way.
            let ends = (m::sqrt(mu / r1) - m::sqrt(mu / r2)).abs()
                + end_burn(self.nav, fb, 0.0)
                + end_burn(self.nav, tb, 0.0);
            let spare = usable - ends;
            if spare > 0.0 {
                // Turnover at the midpoint, or a coast at the best speed the
                // spare Δv allows: t = 2v/a + (gap − v²/a)/v.
                let v = m::sqrt(gap * a).min(0.5 * spare);
                let tt = 2.0 * v / a + ((gap - v * v / a) / v).max(0.0);
                push("torch", ends + 2.0 * v, tt);
            }
        }
        out
    }
}

impl LegMaker for InSystem<'_> {
    fn legs(&self, from: usize, t_s: f64, jf: f64, pr: f64) -> Vec<(usize, Leg)> {
        let now = self.t0_s + t_s;
        let mut out = vec![];
        for to in 0..self.places.len() {
            if to == from {
                continue;
            }
            let both_bodies = self.body(from).is_some() && self.body(to).is_some();
            let legs: Vec<Leg> = if both_bodies {
                let req = RouteRequest {
                    ship: self.with_propellant(pr),
                    from: Endpoint::Body(self.places[from].id.clone()),
                    to: Some(Endpoint::Body(self.places[to].id.clone())),
                    depart_s: Some(now),
                    window_days: self.set.window_days,
                    prefer: self.prefer,
                    rescue: None,
                };
                let Ok(p) = nav_plan(self.nav, self.sys, &req) else {
                    continue;
                };
                // The quickest to arrive and the cheapest: the search keeps
                // whichever is not beaten on cost and fuel.
                let usable: Vec<_> = p
                    .candidates
                    .iter()
                    .filter(|c| c.difficulty.label != "impossible")
                    .collect();
                let quick = usable
                    .iter()
                    .min_by(|a, b| a.arrive_s.total_cmp(&b.arrive_s).then(a.id.cmp(&b.id)));
                let cheap = usable.iter().min_by(|a, b| {
                    a.propellant_t
                        .total_cmp(&b.propellant_t)
                        .then(a.id.cmp(&b.id))
                });
                let mut picks = vec![];
                for c in [quick, cheap].into_iter().flatten() {
                    if !picks.iter().any(|p: &&&crate::Course| p.id == c.id) {
                        picks.push(c);
                    }
                }
                picks
                    .into_iter()
                    .map(|c| Leg {
                        means: "course".into(),
                        course: Some(c.kind.clone()),
                        wait_days: Some(((c.depart_s - now) / DAY).max(0.0)),
                        home_days: (c.arrive_s - now) / DAY,
                        ship_days: (c.arrive_s - now) / DAY,
                        propellant_t: c.propellant_t,
                        propellant_left_t: pr - c.propellant_t,
                        ..Leg::between(
                            &self.places[from].id,
                            &self.places[to].id,
                            dist(&self.places[from].xyz_pc, &self.places[to].xyz_pc),
                        )
                    })
                    .collect()
            } else {
                self.belt_legs(from, to, pr)
            };
            for mut l in legs {
                l.jump_fuel_left_t = jf;
                out.push((to, l));
            }
        }
        out
    }

    /// Time can be no less than a torch flight straight across the radial
    /// gap at the ship's full thrust (2√(gap/a)), nor than the gap crossed
    /// at the whole Δv of full tanks: admissible, so the search
    /// stays exact while skipping places that cannot help.
    fn lower_bound(&self, from: usize, to: usize, minimise: crate::voyage::Minimise) -> f64 {
        use crate::voyage::Minimise::*;
        if !matches!(minimise, HomeTime | ShipTime) || !self.ship.sustained {
            return 0.0;
        }
        let a = self.ship.accel_g.min(self.ship.g_limit) * crate::G0;
        if a <= 0.0 {
            return 0.0;
        }
        let gap = (self.radius_au(to) - self.radius_au(from)).abs() * AU;
        // Nor faster than the whole Δv of full tanks taken as speed.
        let full = Ship {
            propellant_t: None,
            ..self.ship.clone()
        };
        let v = full.dv_m_s();
        let by_speed = if v > 0.0 { gap / v } else { 0.0 };
        (2.0 * m::sqrt(gap / a)).max(by_speed)
    }

    fn refusals(&self, start: usize, target: Option<usize>, visited: &[usize]) -> Vec<String> {
        let name = if self.ship.name.is_empty() {
            "the ship".to_string()
        } else {
            format!("the {}", self.ship.name)
        };
        let nm = |i: usize| {
            let p = &self.places[i];
            if p.name.is_empty() {
                p.id.clone()
            } else {
                p.name.clone()
            }
        };
        match target {
            Some(t) => {
                // Why the direct course fails, in NAV's own words.
                let mut why = vec![];
                if self.body(start).is_some() && self.body(t).is_some() {
                    let req = RouteRequest {
                        ship: self.ship.clone(),
                        from: Endpoint::Body(self.places[start].id.clone()),
                        to: Some(Endpoint::Body(self.places[t].id.clone())),
                        depart_s: Some(self.t0_s),
                        window_days: self.set.window_days,
                        prefer: self.prefer,
                        rescue: None,
                    };
                    if let Ok(p) = nav_plan(self.nav, self.sys, &req) {
                        why = p
                            .refused
                            .iter()
                            .take(2)
                            .map(|r| format!("{} course {}", r.kind, r.why))
                            .collect();
                    }
                }
                let mut out = vec![format!(
                    "{name} cannot reach {} from {} on its propellant, even refuelling where fuel is known",
                    nm(t),
                    nm(start)
                )];
                out.extend(why);
                out
            }
            None => {
                let left = (0..self.places.len())
                    .filter(|i| *i != start && !visited.contains(i))
                    .count();
                if left == 0 {
                    vec![format!("every place in {} has been visited", self.sys.name)]
                } else {
                    vec![format!(
                        "{name} cannot reach any of the {left} places not yet visited on its propellant"
                    )]
                }
            }
        }
    }
}

/// The cheapest in-system voyage for `ship` in `sys` from a body (or belt)
/// to the goal, through the same search as between the stars.
pub fn plan_in_system(
    nav: &NavSettings,
    sys: &NavSystem,
    ship: &Ship,
    from: &str,
    goal: &Goal,
    t0_s: f64,
    o: &Options,
) -> Result<(Voyage, Vec<VoyageSystem>), Vec<String>> {
    let set = &o.settings.in_system;
    let ps = places(sys, set);
    // Accept a body's name for convenience, as plan_route does.
    let resolve = |x: &str| -> String {
        sys.body(x)
            .map(|b| b.address.clone())
            .or_else(|| {
                sys.belts
                    .iter()
                    .find(|b| b.name == x || belt_id(&b.name) == x)
                    .map(|b| belt_id(&b.name))
            })
            .unwrap_or_else(|| x.to_string())
    };
    let goal = match goal {
        Goal::To(t) => Goal::To(resolve(t)),
        Goal::NearestUnvisited { visited, home } => Goal::NearestUnvisited {
            visited: visited.iter().map(|v| resolve(v)).collect(),
            home: *home,
        },
    };
    let prefer = match o.minimise {
        crate::voyage::Minimise::Fuel => Prefer::SaveFuel,
        crate::voyage::Minimise::HomeTime | crate::voyage::Minimise::ShipTime => Prefer::Fastest,
        crate::voyage::Minimise::Legs => Prefer::Balanced,
    };
    let maker = InSystem {
        nav,
        sys,
        ship,
        places: &ps,
        t0_s,
        prefer,
        set,
    };
    let mut o = o.clone();
    o.settings.labels_per_system = set.labels_per_place;
    let v = crate::voyage::search(ship, None, &ps, &resolve(from), &goal, &o, &maker)?;
    Ok((v, ps))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scenario::{settings, sol};
    use crate::voyage::{Minimise, VoyageSettings};

    fn opts(m: Minimise) -> Options {
        Options {
            minimise: m,
            gambles: false,
            settings: VoyageSettings::default(),
        }
    }

    #[test]
    fn earth_to_mars_is_a_transfer_that_waits_for_its_window() {
        let s = settings();
        let sys = sol();
        let ship = s.ships["nuclear-thermal"].clone();
        let (v, _) = plan_in_system(
            &s,
            &sys,
            &ship,
            "Earth",
            &Goal::To("Mars".into()),
            sys.t0_s,
            &opts(Minimise::Fuel),
        )
        .unwrap();
        assert_eq!(v.legs.len(), 1, "{:#?}", v.legs);
        let l = &v.legs[0];
        assert_eq!(l.means, "course");
        assert!(!l.fictional);
        // A Hohmann-like transfer: 200 to 320 days on the way.
        let on_way = l.home_days - l.wait_days.unwrap();
        assert!(on_way > 150.0 && on_way < 350.0, "{on_way}");
        assert!(l.words.contains("Earth to Mars"), "{}", l.words);
    }

    #[test]
    fn belts_are_places_with_no_window_and_ice_beyond_the_snow_line() {
        let s = settings();
        let sys = sol();
        let ps = places(&sys, &VoyageSettings::default().in_system);
        let main = ps.iter().find(|p| p.id == "belt:Main Belt").unwrap();
        assert_eq!(
            main.fuel,
            vec!["ice".to_string()],
            "the outer main belt is icy"
        );
        assert!(ps.iter().find(|p| p.name == "Mars").unwrap().fuel == vec!["port".to_string()]);
        let kuiper = ps.iter().find(|p| p.id == "belt:Kuiper Belt").unwrap();
        assert_eq!(kuiper.fuel, vec!["ice".to_string()]);
        let jupiter = ps.iter().find(|p| p.name == "Jupiter").unwrap();
        assert_eq!(jupiter.fuel, vec!["skim".to_string()]);
        let ship = s.ships["nuclear-thermal"].clone();
        let (v, _) = plan_in_system(
            &s,
            &sys,
            &ship,
            "Mars",
            &Goal::To("Main Belt".into()),
            sys.t0_s,
            &opts(Minimise::HomeTime),
        )
        .unwrap();
        assert_eq!(v.legs[0].course.as_deref(), Some("hohmann"));
        assert_eq!(v.legs[0].wait_days, Some(0.0));
    }

    #[test]
    fn an_explorer_works_outward_through_a_system() {
        let s = settings();
        let sys = sol();
        let ship = s.ships["fusion-torch"].clone();
        let home = [1.0 / AU_PER_PC, 0.0, 0.0];
        let g = Goal::NearestUnvisited {
            visited: vec!["Earth".into()],
            home,
        };
        let (v, _) = plan_in_system(
            &s,
            &sys,
            &ship,
            "Earth",
            &g,
            sys.t0_s,
            &opts(Minimise::HomeTime),
        )
        .unwrap();
        // A torch reaches the nearest world first.
        assert!(
            ["u:sol/sys:sun/body:venus", "u:sol/sys:sun/body:mars"].contains(&v.to.as_str()),
            "{}",
            v.to
        );
    }

    #[test]
    fn a_short_tank_refuels_at_a_giant_on_the_way_out() {
        let s = settings();
        let sys = sol();
        let mut ship = s.ships["nuclear-thermal"].clone();
        // Not enough for Earth to Saturn in one go.
        ship.propellant_t = Some(100.0);
        let r = plan_in_system(
            &s,
            &sys,
            &ship,
            "Earth",
            &Goal::To("Saturn".into()),
            sys.t0_s,
            &opts(Minimise::Fuel),
        );
        match r {
            Ok((v, _)) => {
                assert!(v.legs.len() >= 2, "{:#?}", v.legs);
                assert!(v.refuels >= 1);
            }
            Err(e) => assert!(e[0].contains("cannot reach Saturn"), "{e:?}"),
        }
    }

    #[test]
    fn in_system_plans_are_repeatable() {
        let s = settings();
        let sys = sol();
        let ship = s.ships["fusion-torch"].clone();
        let g = Goal::To("Jupiter".into());
        let a = plan_in_system(
            &s,
            &sys,
            &ship,
            "Earth",
            &g,
            sys.t0_s,
            &opts(Minimise::HomeTime),
        );
        let b = plan_in_system(
            &s,
            &sys,
            &ship,
            "Earth",
            &g,
            sys.t0_s,
            &opts(Minimise::HomeTime),
        );
        assert_eq!(a, b);
    }
}

//! Detailed windows: a fixed-step symplectic integrator written for SSE4
//! from published methods (not translated from any existing code).
//!
//! - Second-order leapfrog in drift-kick-drift form (Verlet 1967; for the
//!   symplectic property, Yoshida 1990, Phys. Lett. A 150, 262, section 2).
//! - Fourth order by Yoshida's triple-jump composition (Yoshida 1990, eq. 4.12):
//!   w1 = 1 / (2 - 2^(1/3)), w0 = -2^(1/3) w1.
//! - Direct pairwise forces in a fixed order (i < j), inertial barycentric
//!   frame, fixed step: the same work and the same rounding on every device.
//! - The central star's mass may change with time (winds): the kick reads
//!   its mass at the kick's time. The map is then no longer exactly
//!   symplectic; tests bound the drift.

use crate::kepler::{self, Elements, G, V3};
use serde::{Deserialize, Serialize};
use sse_core::math as m;

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Body {
    pub name: String,
    /// Solar masses.
    pub mass: f64,
    pub x: V3,
    pub v: V3,
}

/// A detailed window: bodies (index 0 is the star) and the integrator.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Window {
    pub bodies: Vec<Body>,
    /// Years.
    pub t: f64,
    pub dt: f64,
    /// Steps taken (deterministic work units).
    pub steps: u64,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Order {
    Second,
    Fourth,
}

impl Window {
    /// Star plus planets from Jacobi elements (each planet's orbit about the
    /// centre of mass of the star and the planets inside it: the coordinates
    /// in which outer orbits stay close to Keplerian; Murray and Dermott 1999,
    /// section 9.5); moved to the barycentre.
    pub fn from_elements(star_mass: f64, planets: &[(String, f64, Elements)], dt: f64) -> Self {
        let mut bodies = vec![Body {
            name: "star".into(),
            mass: star_mass,
            x: [0.0; 3],
            v: [0.0; 3],
        }];
        for (n, mass, el) in planets {
            let (cm, cx, cv) = Self::centre(&bodies);
            let (x, v) = kepler::to_state(el, G * (cm + mass));
            bodies.push(Body {
                name: n.clone(),
                mass: *mass,
                x: [cx[0] + x[0], cx[1] + x[1], cx[2] + x[2]],
                v: [cv[0] + v[0], cv[1] + v[1], cv[2] + v[2]],
            });
        }
        let mut w = Window {
            bodies,
            t: 0.0,
            dt,
            steps: 0,
        };
        w.recentre();
        w
    }

    /// Mass, centre of mass and its velocity.
    fn centre(bodies: &[Body]) -> (f64, V3, V3) {
        let mt: f64 = bodies.iter().map(|b| b.mass).sum();
        let mut cx = [0.0; 3];
        let mut cv = [0.0; 3];
        for b in bodies {
            for k in 0..3 {
                cx[k] += b.mass * b.x[k] / mt;
                cv[k] += b.mass * b.v[k] / mt;
            }
        }
        (mt, cx, cv)
    }

    fn recentre(&mut self) {
        let (_, cx, cv) = Self::centre(&self.bodies);
        for b in &mut self.bodies {
            for k in 0..3 {
                b.x[k] -= cx[k];
                b.v[k] -= cv[k];
            }
        }
    }

    fn drift(&mut self, h: f64) {
        for b in &mut self.bodies {
            for k in 0..3 {
                b.x[k] += h * b.v[k];
            }
        }
    }

    fn kick(&mut self, h: f64) {
        let n = self.bodies.len();
        let mut acc = vec![[0.0_f64; 3]; n];
        for i in 0..n {
            for j in (i + 1)..n {
                let d = kepler::sub(self.bodies[j].x, self.bodies[i].x);
                let r2 = kepler::dot(d, d);
                let inv = 1.0 / (r2 * m::sqrt(r2));
                for k in 0..3 {
                    acc[i][k] += G * self.bodies[j].mass * d[k] * inv;
                    acc[j][k] -= G * self.bodies[i].mass * d[k] * inv;
                }
            }
        }
        for (b, a) in self.bodies.iter_mut().zip(&acc) {
            for k in 0..3 {
                b.v[k] += h * a[k];
            }
        }
    }

    /// One drift-kick-drift leapfrog of length `h`; the star's mass is set
    /// from `star_mass` at the kick's time.
    fn leapfrog(&mut self, h: f64, star_mass: &dyn Fn(f64) -> f64) {
        self.drift(0.5 * h);
        self.t += 0.5 * h;
        self.bodies[0].mass = star_mass(self.t);
        self.kick(h);
        self.drift(0.5 * h);
        self.t += 0.5 * h;
    }

    /// Advance `n` fixed steps.
    pub fn run(&mut self, n: u64, order: Order, star_mass: &dyn Fn(f64) -> f64) {
        let w1 = 1.0 / (2.0 - m::cbrt(2.0));
        let w0 = -m::cbrt(2.0) * w1;
        for _ in 0..n {
            match order {
                Order::Second => self.leapfrog(self.dt, star_mass),
                Order::Fourth => {
                    self.leapfrog(w1 * self.dt, star_mass);
                    self.leapfrog(w0 * self.dt, star_mass);
                    self.leapfrog(w1 * self.dt, star_mass);
                }
            }
            self.steps += 1;
        }
    }

    /// Total energy (solar masses au^2 / yr^2).
    pub fn energy(&self) -> f64 {
        let n = self.bodies.len();
        let mut e = 0.0;
        for b in &self.bodies {
            e += 0.5 * b.mass * kepler::dot(b.v, b.v);
        }
        for i in 0..n {
            for j in (i + 1)..n {
                let d = kepler::sub(self.bodies[j].x, self.bodies[i].x);
                e -= G * self.bodies[i].mass * self.bodies[j].mass / kepler::norm(d);
            }
        }
        e
    }

    /// Jacobi osculating elements of body `k` (k >= 1): relative to the
    /// centre of mass of bodies 0 to k-1.
    pub fn elements(&self, k: usize) -> Elements {
        let (cm, cx, cv) = Self::centre(&self.bodies[..k]);
        let b = &self.bodies[k];
        kepler::from_state(
            kepler::sub(b.x, cx),
            kepler::sub(b.v, cv),
            G * (cm + b.mass),
        )
    }
}

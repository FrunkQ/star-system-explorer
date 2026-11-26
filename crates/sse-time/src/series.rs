//! Measured motion (RW17; real-catalogues design 6A.2): a published series
//! that says where a body is at any date inside the span it was fitted over.
//!
//! One form for every series (`sse.series/1`, written by
//! `scripts/build-sol-motion.py`): three coordinates, each a sum over powers
//! k of t^k times a sum of terms `A * wave(p0 + p1 t + p2 t^2 + ...)`, with
//! `wave` sin or cos as the source prints it. Two shapes of output:
//!
//! - `xyz`: the sums are x, y, z (VSOP87A, Bretagnon and Francou 1988).
//! - `lbr`: the sums are longitude (added to a mean longitude polynomial),
//!   latitude and distance on the mean ecliptic of date, turned to the J2000
//!   ecliptic by Laskar's P and Q (ELP/MPP02, Chapront and Francou 2003).
//!
//! Pure arithmetic through `sse_core::math`: the same on every device, any
//! date read directly with no stepping. The span is the series' own; what
//! happens outside it is the reader's choice (the kernel hands over to a
//! plain orbit and says so).

use serde::Deserialize;
use sse_core::math as m;

/// Julian date of J2000.0 (2000 January 1, 12:00 TT): the epoch every series
/// and the Earth orientation table count from.
pub const J2000_JD: f64 = 2_451_545.0; // not-sheet: the J2000.0 epoch is a definition (IAU 1976), not a measured value

#[derive(Clone, Debug, Deserialize)]
struct TimeDoc {
    epoch_jd: f64,
    unit_days: f64,
}

#[derive(Clone, Debug, Deserialize)]
struct SpanDoc {
    from_jd: f64,
    to_jd: f64,
}

#[derive(Clone, Debug, Deserialize)]
struct PqDoc {
    #[serde(rename = "P")]
    p: Vec<f64>,
    #[serde(rename = "Q")]
    q: Vec<f64>,
}

#[derive(Clone, Debug, Deserialize)]
struct Doc {
    format: String,
    id: String,
    body: String,
    centre: String,
    gives: String,
    frame: String,
    form: String,
    wave: String,
    unit_m: f64,
    #[serde(default)]
    angle_unit_rad: Option<f64>,
    #[serde(default)]
    mean_longitude_rad: Option<Vec<f64>>,
    #[serde(default)]
    to_j2000: Option<PqDoc>,
    time: TimeDoc,
    span: SpanDoc,
    #[serde(default)]
    src: String,
    vars: Vec<Vec<Vec<Vec<f64>>>>,
}

#[derive(Clone, Debug)]
enum Form {
    Xyz,
    Lbr {
        w1: Vec<f64>,
        angle: f64,
        p: Vec<f64>,
        q: Vec<f64>,
    },
}

/// One measured motion, read from its table.
#[derive(Clone, Debug)]
pub struct Series {
    pub id: String,
    /// The body the table is for (its id in packs/base-sol).
    pub body: String,
    /// The body it is measured from (its centre).
    pub centre: String,
    /// True when it gives the barycentre of the body and its moons (the
    /// Earth-Moon barycentre), false for the body's own centre.
    pub gives_group: bool,
    /// The source's id in the table's `sources`.
    pub src: String,
    pub from_jd: f64,
    pub to_jd: f64,
    form: Form,
    cos: bool,
    unit_m: f64,
    epoch_jd: f64,
    unit_days: f64,
    vars: Vec<Vec<Vec<Vec<f64>>>>,
}

/// Horner's rule, lowest power first.
fn horner(c: &[f64], t: f64) -> f64 {
    let mut s = 0.0;
    for &x in c.iter().rev() {
        s = s * t + x;
    }
    s
}

impl Series {
    pub fn from_json(text: &str) -> Result<Series, String> {
        let d: Doc = serde_json::from_str(text).map_err(|e| format!("series: {e}"))?;
        if d.format != "sse.series/1" {
            return Err(format!("{}: format {} is not sse.series/1", d.id, d.format));
        }
        if d.frame != "j2000_ecliptic" {
            return Err(format!("{}: frame {} is not j2000_ecliptic", d.id, d.frame));
        }
        if d.vars.len() != 3 {
            return Err(format!("{}: {} coordinates, not 3", d.id, d.vars.len()));
        }
        let width = match d.form.as_str() {
            "xyz" => 3,
            "lbr" => 6,
            f => return Err(format!("{}: form {f} is not xyz or lbr", d.id)),
        };
        if d.vars.iter().flatten().flatten().any(|t| t.len() != width) {
            return Err(format!("{}: a term is not {width} numbers", d.id));
        }
        let form = if d.form == "xyz" {
            Form::Xyz
        } else {
            let pq = d.to_j2000.ok_or(format!("{}: lbr needs to_j2000", d.id))?;
            Form::Lbr {
                w1: d
                    .mean_longitude_rad
                    .ok_or(format!("{}: lbr needs mean_longitude_rad", d.id))?,
                angle: d
                    .angle_unit_rad
                    .ok_or(format!("{}: lbr needs angle_unit_rad", d.id))?,
                p: pq.p,
                q: pq.q,
            }
        };
        let cos = match d.wave.as_str() {
            "cos" => true,
            "sin" => false,
            w => return Err(format!("{}: wave {w} is not sin or cos", d.id)),
        };
        Ok(Series {
            id: d.id,
            body: d.body,
            centre: d.centre,
            gives_group: d.gives == "group",
            src: d.src,
            from_jd: d.span.from_jd,
            to_jd: d.span.to_jd,
            form,
            cos,
            unit_m: d.unit_m,
            epoch_jd: d.time.epoch_jd,
            unit_days: d.time.unit_days,
            vars: d.vars,
        })
    }

    /// Whether `jd` (TDB) is inside the span the series holds over.
    pub fn covers(&self, jd: f64) -> bool {
        jd >= self.from_jd && jd <= self.to_jd
    }

    /// The nearest date inside the span.
    pub fn clamp(&self, jd: f64) -> f64 {
        jd.clamp(self.from_jd, self.to_jd)
    }

    fn sum(&self, var: usize, t: f64) -> f64 {
        let mut total = 0.0;
        let mut tk = 1.0;
        for terms in &self.vars[var] {
            let mut s = 0.0;
            for row in terms {
                let arg = horner(&row[1..], t);
                s += row[0] * if self.cos { m::cos(arg) } else { m::sin(arg) };
            }
            total += s * tk;
            tk *= t;
        }
        total
    }

    /// Where the body is from its centre at `jd` (TDB), metres, on the J2000
    /// ecliptic. Read anywhere; it is only measured inside the span.
    pub fn position_m(&self, jd: f64) -> [f64; 3] {
        let t = (jd - self.epoch_jd) / self.unit_days;
        match &self.form {
            Form::Xyz => [
                self.sum(0, t) * self.unit_m,
                self.sum(1, t) * self.unit_m,
                self.sum(2, t) * self.unit_m,
            ],
            Form::Lbr { w1, angle, p, q } => {
                let l = horner(w1, t) + self.sum(0, t) * angle;
                let b = self.sum(1, t) * angle;
                let r = self.sum(2, t) * self.unit_m;
                let (x, y, z) = (
                    r * m::cos(l) * m::cos(b),
                    r * m::sin(l) * m::cos(b),
                    r * m::sin(b),
                );
                // Laskar's P and Q: the mean ecliptic of date to the J2000
                // ecliptic (Chapront and Francou 2003, section 6).
                let (pp, qq) = (horner(p, t), horner(q, t));
                let sq = m::sqrt(1.0 - pp * pp - qq * qq);
                [
                    (1.0 - 2.0 * pp * pp) * x + 2.0 * pp * qq * y + 2.0 * pp * sq * z,
                    2.0 * pp * qq * x + (1.0 - 2.0 * qq * qq) * y - 2.0 * qq * sq * z,
                    -2.0 * pp * sq * x
                        + 2.0 * qq * sq * y
                        + (1.0 - 2.0 * pp * pp - 2.0 * qq * qq) * z,
                ]
            }
        }
    }

    /// Position (m) and velocity (m/s) at `jd`, the velocity from the series
    /// on either side (a central difference over `half_day` days each way,
    /// pulled inside the span when `jd` is at its edge).
    pub fn state_m(&self, jd: f64, half_day: f64) -> ([f64; 3], [f64; 3]) {
        let c = jd.clamp(self.from_jd + half_day, self.to_jd - half_day);
        let (a, b) = (self.position_m(c - half_day), self.position_m(c + half_day));
        let dt = 2.0 * half_day * sse_core::sheet::DAY;
        let v = [(b[0] - a[0]) / dt, (b[1] - a[1]) / dt, (b[2] - a[2]) / dt];
        let p = self.position_m(jd);
        (p, v)
    }

    /// How many terms the table holds.
    pub fn terms(&self) -> usize {
        self.vars.iter().flatten().map(|t| t.len()).sum()
    }
}

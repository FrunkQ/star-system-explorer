//! O2: light from a body's conditions (nebulae-and-light design sections 1
//! and 5A). One path for every body, keyed on conditions, never on kind: a
//! star is a body whose own luminosity dwarfs what it absorbs; a giant planet
//! has a little of both; a moon has none of its own.
//!
//! The scalar physics (glow temperature, the stellar wind, the magnetopause,
//! auroral radio and its top frequency) is the editable flow
//! `sse.base-light.body-emission` in `packs/base-light`, built from cited
//! atoms in `sse-atoms::light`. Reflected light uses the
//! `sse.light.reflected_area` atom once per lighting body. This module only
//! assembles the spectrum from those numbers.

use crate::*;
use serde::{Deserialize, Serialize};
use sse_core::{Quantity, Units};
use sse_flow::{AtomRegistry, Context, Flow, InputValue, SourceLabel};
use std::collections::BTreeMap;
use std::sync::OnceLock;

pub const EMISSION_FLOW: &str = "sse.base-light.body-emission";
pub const BASE_PHYSICS: &str = include_str!("../../../packs/base-physics/pack.json");

/// A luminous body lighting this one.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Source {
    pub address: String,
    pub luminosity_w: f64,
    /// Its effective temperature: the colour of the light it sends.
    pub t_k: f64,
    pub distance_m: f64,
}

/// Gas falling onto the body through a disc.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Accretion {
    pub mass_kg: f64,
    pub rate_kg_s: f64,
    pub r_inner_m: f64,
    pub r_outer_m: f64,
}

/// A built thing's heat radiator.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Radiator {
    pub t_k: f64,
    pub area_m2: f64,
}

/// A radio transmitter (radar, talk, jamming).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Transmitter {
    pub frequency_hz: f64,
    pub bandwidth_hz: f64,
    pub power_w: f64,
}

/// Outdoor lights, by kind of lamp.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum LampKind {
    /// Low-pressure sodium: almost all its light in the sodium D lines.
    Sodium,
    /// White light (LED and similar), shown as flat across 400 to 700 nm.
    White,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Lamp {
    pub kind: LampKind,
    /// Light emitted (not electrical power), W.
    pub power_w: f64,
}

/// Everything about a body that decides its light. Every field but the
/// address and radius is optional and defaults to "none".
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Conditions {
    pub address: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub radius_m: f64,
    /// Own luminosity: a star's output, a giant's left-over heat, a world's geothermal flow.
    #[serde(default)]
    pub own_luminosity_w: f64,
    #[serde(default)]
    pub bond_albedo: f64,
    #[serde(default)]
    pub magnetic_moment_a_m2: f64,
    #[serde(default)]
    pub lit_by: Vec<Source>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub accretion: Option<Accretion>,
    #[serde(default)]
    pub corona_xray_w: f64,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub radiators: Vec<Radiator>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub transmitters: Vec<Transmitter>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub lamps: Vec<Lamp>,
}

/// The scalar results along the way, for "Why?" and other readers.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EmissionFacts {
    pub insolation_w_m2: f64,
    pub internal_flux_w_m2: f64,
    pub effective_temperature_k: f64,
    pub magnetopause_radius_m: f64,
    pub auroral_radio_w: f64,
    pub radio_max_frequency_hz: f64,
}

/// The compiled emission flow and what it needs to run.
pub struct Model {
    pub flow: Flow,
    pub atoms: AtomRegistry,
    pub kinds: BTreeMap<String, sse_core::QuantityKind>,
    pub units: Units,
    pub table: LightTable,
}

impl Model {
    /// From a pack stack that holds the atoms' kinds (base-physics), a light
    /// table and an emission flow (base-light, or a pack replacing it).
    pub fn from_packs(packs: &[sse_pack::PackManifest]) -> Result<Self, String> {
        let stack = sse_pack::resolve(packs, "0.1.0").map_err(|e| format!("{e:?}"))?;
        let atoms = sse_atoms::registry();
        let doc = stack
            .flows
            .values()
            .find(|f| f.doc.hook == "body.emission")
            .ok_or("no pack provides a body.emission flow")?
            .doc
            .clone();
        let flow = Flow::compile(
            doc,
            &Context {
                atoms: &atoms,
                kinds: &stack.kinds,
                units: &stack.units,
            },
        )
        .map_err(|e| format!("{e:?}"))?;
        Ok(Model {
            flow,
            atoms,
            kinds: stack.kinds,
            units: stack.units,
            table: LightTable::from_packs(packs)?,
        })
    }

    /// The base packs.
    pub fn base() -> &'static Model {
        static CELL: OnceLock<Model> = OnceLock::new();
        CELL.get_or_init(|| {
            let packs = [
                sse_pack::PackManifest::from_json(BASE_PHYSICS).expect("base-physics parses"),
                sse_pack::PackManifest::from_json(table::BASE_PACK).expect("base-light parses"),
            ];
            Model::from_packs(&packs).expect("base light model builds")
        })
    }

    fn cx(&self) -> Context<'_> {
        Context {
            atoms: &self.atoms,
            kinds: &self.kinds,
            units: &self.units,
        }
    }

    fn atom(&self, id: &str, inputs: &[f64]) -> Result<Vec<f64>, String> {
        let a = self.atoms.get(id).ok_or(format!("atom {id} missing"))?;
        let params: Vec<f64> = a.decl().params.iter().map(|p| p.default).collect();
        a.eval(inputs, &params)
            .map_err(|e| format!("{id}: {}", e.0))
    }
}

const SOLAR_WIND: &str = "the stellar wind is the Sun's (Parker spiral, 1 au values) for every star until stellar activity (O10) gives each star its own";
const GREY: &str = "albedo is grey (the same at every wavelength) and the body glows as a black body at its effective temperature: no absorption bands from its air";
const ISOTROPIC_RADIO: &str = "auroral radio is spread evenly over all directions and flat in frequency from a thirtieth of its top frequency up; real auroral radio is beamed into about 1.6 sr and varies with rotation";

/// Build a body's light from its conditions.
pub fn describe(
    c: &Conditions,
    model: &Model,
) -> Result<(LightDescription, EmissionFacts), String> {
    if c.radius_m <= 0.0 {
        return Err(format!("{}: radius must be positive", c.address));
    }
    let mut d = LightDescription::new(&c.address);
    let r = c.radius_m;
    let surface = 4.0 * m::PI * r * r;
    let insolation: f64 = c
        .lit_by
        .iter()
        .filter(|s| s.distance_m > 0.0)
        .map(|s| s.luminosity_w / (4.0 * m::PI * s.distance_m * s.distance_m))
        .sum();
    let internal = c.own_luminosity_w.max(0.0) / surface;
    // The wind comes from the body lighting it most.
    let wind_from =
        c.lit_by
            .iter()
            .filter(|s| s.distance_m > 0.0)
            .fold(None::<&Source>, |best, s| {
                let f = s.luminosity_w / (s.distance_m * s.distance_m);
                match best {
                    Some(b) if b.luminosity_w / (b.distance_m * b.distance_m) >= f => Some(b),
                    _ => Some(s),
                }
            });
    let (moment, distance) = match wind_from {
        Some(s) => (c.magnetic_moment_a_m2.max(0.0), s.distance_m),
        None => (0.0, 1.0),
    };
    let dim = |k: &str| model.kinds.get(k).map(|q| q.dim).unwrap_or_default();
    let mut ins = BTreeMap::new();
    for (name, kind, v) in [
        ("flux", "planet.insolation", insolation),
        (
            "albedo",
            "planet.bond_albedo",
            c.bond_albedo.clamp(0.0, 1.0),
        ),
        ("internal_flux", "body.internal_heat_flux", internal),
        ("radius", "body.radius", r),
        ("moment", "body.magnetic_moment", moment),
        ("distance", "orbit.semi_major_axis", distance),
    ] {
        ins.insert(
            name.to_string(),
            InputValue {
                q: Quantity::si(v, dim(kind)),
                source: SourceLabel::Authored,
            },
        );
    }
    let ev = model
        .flow
        .eval(&model.cx(), &ins)
        .map_err(|e| format!("{}: emission flow: {e:?}", c.address))?;
    let get = |k: &str| ev.published(k).map(|q| q.value).unwrap_or(0.0);
    let facts = EmissionFacts {
        insolation_w_m2: insolation,
        internal_flux_w_m2: internal,
        effective_temperature_k: get("body.effective_temperature"),
        magnetopause_radius_m: get("body.magnetopause_radius"),
        auroral_radio_w: get("radio.auroral_power"),
        radio_max_frequency_hz: get("radio.max_frequency"),
    };

    // Own heat, from absorbed light and internal heat together.
    if facts.effective_temperature_k > 0.0 {
        d.parts.push(Part::Thermal {
            t_k: facts.effective_temperature_k,
            area_m2: surface,
            emissivity: 1.0,
            cause: "own glow at the effective temperature".into(),
        });
        let own = if insolation > 0.0 {
            internal / (internal + (1.0 - c.bond_albedo) * insolation / 4.0)
        } else {
            1.0
        };
        d.why.push(format!(
            "Glows like a black body at {:.0} K over its whole surface; {:.0} % of that heat is its own, the rest is absorbed light.",
            facts.effective_temperature_k,
            100.0 * own
        ));
        if insolation > 0.0 {
            d.stand_ins.push(GREY.into());
        } else {
            d.stand_ins.push("the photosphere is a black body: no limb darkening and no absorption lines yet (O12 adds lines)".into());
        }
    }
    // Reflected light, once per lighting body.
    if c.bond_albedo > 0.0 {
        for s in c
            .lit_by
            .iter()
            .filter(|s| s.distance_m > 0.0 && s.t_k > 0.0)
        {
            let rs =
                m::sqrt(s.luminosity_w / (4.0 * m::PI * SIGMA * s.t_k * s.t_k * s.t_k * s.t_k));
            let area = model.atom(
                "sse.light.reflected_area",
                &[r, c.bond_albedo, rs, s.distance_m],
            )?[0];
            d.parts.push(Part::Reflected {
                source: s.address.clone(),
                t_k: s.t_k,
                area_m2: area,
                cause: "reflected light".into(),
            });
            d.why.push(format!(
                "Reflects {:.0} % of the light from {} ({:.0} K).",
                100.0 * c.bond_albedo,
                s.address,
                s.t_k
            ));
        }
    }
    // Auroral radio, only with a magnetosphere.
    if facts.auroral_radio_w > 0.0 && facts.radio_max_frequency_hz > 0.0 {
        let f = facts.radio_max_frequency_hz;
        d.parts.push(Part::PowerLaw {
            power_w: facts.auroral_radio_w,
            from_m: C / f,
            to_m: C / (f / 30.0),
            alpha: 0.0,
            cause: "auroral radio (electron-cyclotron maser)".into(),
        });
        d.why.push(format!(
            "Its magnetic field holds off the wind {:.1} radii out; the wind's field crossing that bubble powers {:.1e} W of auroral radio up to {:.2} MHz.",
            facts.magnetopause_radius_m / r,
            facts.auroral_radio_w,
            f / 1e6
        ));
        d.stand_ins.push(ISOTROPIC_RADIO.into());
        d.stand_ins.push(SOLAR_WIND.into());
        d.references.push(Reference::parse(
            "Zarka, P. et al. 2001, Magnetically-driven planetary radio emissions and application to extrasolar planets, Astrophys. Space Sci. 277, 293. doi:10.1023/A:1012221527425",
        ));
    } else if c.magnetic_moment_a_m2 <= 0.0 && !c.lit_by.is_empty() {
        d.why
            .push("No global magnetic field, so no magnetosphere and no auroral radio.".into());
    }
    if let Some(a) = &c.accretion {
        d.parts.push(Part::Disc {
            mass_kg: a.mass_kg,
            accretion_kg_s: a.rate_kg_s,
            r_inner_m: a.r_inner_m,
            r_outer_m: a.r_outer_m,
            cause: "accretion disc".into(),
        });
        d.why.push(format!(
            "Gas falling in at {:.2e} kg/s through a disc heats it to {:.2e} K near the inner edge.",
            a.rate_kg_s,
            disc_temperature(a.mass_kg, a.rate_kg_s, a.r_inner_m, a.r_inner_m * 49.0 / 36.0)
        ));
        d.stand_ins
            .push("the disc is seen from all sides alike: no tilt, no relativistic beaming or redshift (O7)".into());
        d.references.push(Reference {
            text: R_SS73.into(),
            doi: None,
            url: Some("https://ui.adsabs.harvard.edu/abs/1973A&A....24..337S".into()),
        });
    }
    if c.corona_xray_w > 0.0 {
        d.parts.push(Part::PowerLaw {
            power_w: c.corona_xray_w,
            from_m: 1.239_84e-10,
            to_m: 1.239_84e-8,
            alpha: 0.0,
            cause: "hot corona".into(),
        });
        d.why.push(format!(
            "A hot corona gives {:.1e} W of X-rays (0.1 to 10 keV).",
            c.corona_xray_w
        ));
        d.stand_ins.push("the corona's X-rays are flat in frequency over 0.1 to 10 keV rather than a hot-gas spectrum at the corona's temperature (O10)".into());
    }
    for rad in &c.radiators {
        d.parts.push(Part::Thermal {
            t_k: rad.t_k,
            area_m2: rad.area_m2,
            emissivity: 0.9,
            cause: "heat radiator".into(),
        });
        d.why.push(format!(
            "A radiator of {:.0} m² at {:.0} K sheds {:.2e} W of heat, mostly in the infrared.",
            rad.area_m2,
            rad.t_k,
            0.9 * SIGMA * rad.area_m2 * rad.t_k * rad.t_k * rad.t_k * rad.t_k
        ));
    }
    for t in &c.transmitters {
        let half = 0.5 * t.bandwidth_hz.max(1.0);
        if t.frequency_hz > half {
            d.parts.push(Part::PowerLaw {
                power_w: t.power_w,
                from_m: C / (t.frequency_hz + half),
                to_m: C / (t.frequency_hz - half),
                alpha: 0.0,
                cause: "radio transmitter".into(),
            });
            d.why.push(format!(
                "Transmits {:.1e} W at {:.3} MHz, which anyone listening in that band can hear.",
                t.power_w,
                t.frequency_hz / 1e6
            ));
        }
    }
    for l in &c.lamps {
        match l.kind {
            LampKind::Sodium => d.lines.push(LineEmission {
                line: "na-d".into(),
                power_w: l.power_w,
                cause: "sodium lamps".into(),
            }),
            LampKind::White => {
                d.parts.push(Part::PowerLaw {
                    power_w: l.power_w,
                    from_m: 4e-7,
                    to_m: 7e-7,
                    alpha: 2.0,
                    cause: "white lamps".into(),
                });
                d.stand_ins.push(
                    "white lamps are flat across 400 to 700 nm rather than a real LED spectrum"
                        .into(),
                );
            }
        }
        d.why.push(format!(
            "Its lights give off {:.1e} W on the night side.",
            l.power_w
        ));
    }
    d.stand_ins.sort();
    d.stand_ins.dedup();
    Ok((d, facts))
}

/// One body's published light with the facts behind it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BodyLight {
    pub address: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub facts: EmissionFacts,
    pub light: Published,
}

/// Describe and publish a list of bodies.
pub fn publish_all(bodies: &[Conditions], model: &Model) -> Result<Vec<BodyLight>, String> {
    bodies
        .iter()
        .map(|c| {
            let (d, facts) = describe(c, model)?;
            Ok(BodyLight {
                address: c.address.clone(),
                name: c.name.clone(),
                facts,
                light: publish(&d, &model.table),
            })
        })
        .collect()
}

/// The Solar System fixture (`tests/fixtures/light/solar-system.json`).
pub const SOLAR_SYSTEM: &str = include_str!("../../../tests/fixtures/light/solar-system.json");

#[derive(Clone, Debug, Deserialize)]
pub struct Fixture {
    pub bodies: Vec<Conditions>,
    #[serde(default)]
    pub observed: serde_json::Value,
}

pub fn solar_system() -> Fixture {
    serde_json::from_str(SOLAR_SYSTEM).expect("light fixture parses")
}

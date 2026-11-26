//! Small bodies up close (track SB, `sse-small`): `small_body` resolves one
//! member of a system's belt, ring or cloud into a definite body, or lists
//! the bodies its formation made before anyone looked (a shattered world's
//! bared core); `make_small_body` makes one a GM adds by hand, out of the
//! system's own nearest cloud. Nothing is stored: the same body comes back
//! on every ask, on every device.

use crate::err;
use crate::protocol::Response;
use sse_core::math as m;
use sse_core::{Address, Seed};
use sse_formation::leftovers::Cloud;
use sse_formation::pipeline::SystemRun;
use sse_small::{Host, Makeup, Member, Pins, Settings};

/// The base pack's numbers, read once (parsing the sheet for grain
/// densities is the slow part).
fn settings() -> Settings {
    static S: std::sync::OnceLock<Settings> = std::sync::OnceLock::new();
    S.get_or_init(Settings::default).clone()
}

/// The star and age the run formed: the primary's luminosity now.
fn host_of(run: &SystemRun) -> Host {
    let h = &run.history;
    let age_myr = h.inheritance.age_gyr * 1000.0;
    let lum = h
        .state_at(age_myr)
        .first()
        .map(|s| s.luminosity)
        .unwrap_or(1.0);
    Host {
        luminosity_lsun: lum,
        mass_msun: h.stars.first().map(|s| s.mass).unwrap_or(1.0),
        age_myr,
    }
}

fn seed_of(run: &SystemRun) -> Seed {
    Seed::from_text(&run.history.manifest.seed)
}

/// Distance from the star of a planet-bound cloud's planet.
fn star_au(run: &SystemRun, c: &Cloud) -> Option<f64> {
    let id = c.around.as_ref()?;
    let l = run.leftovers.as_ref()?;
    l.worlds.iter().find(|w| &w.id == id).map(|w| w.a_au)
}

/// The member as JSON, with its activity when it holds ice (SB4): at
/// `r_au` from the star, or at perihelion (a planet-bound body at its
/// planet's distance). `cloud` gives the shares of its ices.
fn member_json(
    m: &Member,
    cloud: &Cloud,
    host: &Host,
    star: Option<f64>,
    r_au: Option<f64>,
    s: &Settings,
) -> serde_json::Value {
    let mut j = serde_json::to_value(m).expect("plain");
    if m.makeup.ice > 0.0 {
        let r = r_au.or(star).unwrap_or(m.orbit.a_au * (1.0 - m.orbit.e));
        let shares = sse_small::ice_shares(cloud, &s.comet);
        let a = sse_small::activity(m, &shares, r, host, &s.comet);
        j["activity"] = serde_json::to_value(a).expect("plain");
    }
    j
}

/// `small_body`: with `member` (a cloud member's address, `<cloud>/obj:<k>`)
/// or `cloud` and `rank`, that member resolved; with neither, the notable
/// bodies formation made, each resolved.
pub(crate) fn read(
    run: Option<&SystemRun>,
    member: Option<&str>,
    cloud: Option<&str>,
    rank: Option<u64>,
    r_au: Option<f64>,
) -> Result<Response, Response> {
    let run = run.ok_or_else(|| {
        err(
            "no_formation",
            "this system has no formation run, so its belts and rings have no members to resolve",
        )
    })?;
    let l = run.leftovers.as_ref().ok_or_else(|| {
        err(
            "no_formation",
            "this system's formation made no belts or rings",
        )
    })?;
    let s = settings();
    let seed = seed_of(run);
    let host = host_of(run);
    let u = run.upheaval.as_ref();
    let find = |a: &str| l.clouds.iter().find(|c| c.address.to_string() == a);
    let (c, k) = match (member, cloud, rank) {
        (Some(mem), None, None) => {
            let a: Address = mem
                .parse()
                .map_err(|e| err("bad_address", format!("{e}")))?;
            let k = sse_small::rank_of(&a).ok_or_else(|| {
                err(
                    "bad_address",
                    format!("{mem} is not a cloud member (…/obj:<k>)"),
                )
            })?;
            let parent = mem.rsplit_once("/obj:").map(|x| x.0).unwrap_or_default();
            (
                find(parent)
                    .ok_or_else(|| err("not_found", format!("no cloud {parent} in this system")))?,
                k,
            )
        }
        (None, Some(ca), r) => (
            find(ca).ok_or_else(|| err("not_found", format!("no cloud {ca} in this system")))?,
            r.unwrap_or(1),
        ),
        (None, None, None) => {
            let bodies = u
                .map(|u| sse_small::notables(u, l))
                .unwrap_or_default()
                .into_iter()
                .filter_map(|nb| {
                    let c = find(&nb.cloud)?;
                    let m = sse_small::resolve(&seed, c, nb.rank, &host, star_au(run, c), u, &s);
                    let mut j = member_json(&m, c, &host, star_au(run, c), r_au, &s);
                    j["why"] = nb.why.into();
                    Some(j)
                })
                .collect();
            return Ok(Response::SmallBodyIs {
                body: None,
                notables: Some(bodies),
            });
        }
        _ => {
            return Err(err(
                "bad_request",
                "give a member address, or a cloud and a rank, or neither for the notable bodies",
            ))
        }
    };
    if k == 0 {
        return Err(err("bad_request", "members are ranked from 1"));
    }
    let m = sse_small::resolve(&seed, c, k, &host, star_au(run, c), u, &s);
    Ok(Response::SmallBodyIs {
        body: Some(member_json(&m, c, &host, star_au(run, c), r_au, &s)),
        notables: None,
    })
}

/// What `make_small_body` is asked for.
pub(crate) struct Make<'a> {
    pub address: &'a str,
    pub kind: &'a str,
    pub diameter_km: Option<f64>,
    pub mass_kg: Option<f64>,
    pub a_au: f64,
    pub e: Option<f64>,
    /// The designer's pins (RW16a); `pair` here overrides the kind's.
    pub pins: Pins,
}

/// A Sun-like host for a body designed in a system with no formation record
/// (fudge SB-designed-field).
fn sunlike() -> Host {
    Host {
        luminosity_lsun: 1.0,
        mass_msun: 1.0,
        age_myr: 4_570.0,
    }
}

/// The makeup the designer pins, as mass shares of metal, rock, carbon and ice.
pub(crate) fn makeup_pin(v: Option<&[f64]>) -> Result<Option<Makeup>, Response> {
    let Some(v) = v else { return Ok(None) };
    if v.len() != 4 || v.iter().any(|x| !x.is_finite() || *x < 0.0) || v.iter().sum::<f64>() <= 0.0
    {
        return Err(err(
            "bad_request",
            "makeup is four shares, metal, rock, carbon and ice, none below 0",
        ));
    }
    Ok(Some(Makeup::from_array([v[0], v[1], v[2], v[3]])))
}

/// The answer for a made or designed body: the member, what it was made
/// from, and what an implausible design takes (only when it takes something).
fn made_json(m: &Member, template: Option<&Cloud>, host: &Host, s: &Settings) -> serde_json::Value {
    let mut j = match template {
        Some(t) => member_json(m, t, host, None, None, s),
        None => {
            let mut j = serde_json::to_value(m).expect("plain");
            if m.makeup.ice > 0.0 {
                // A designed field's ice is all water.
                let shares = [1.0, 0.0, 0.0];
                let r = m.orbit.a_au * (1.0 - m.orbit.e);
                let a = sse_small::activity(m, &shares, r, host, &s.comet);
                j["activity"] = serde_json::to_value(a).expect("plain");
            }
            j
        }
    };
    let t = sse_small::takes(m, s);
    if !t.is_empty() {
        j["takes"] = serde_json::to_value(t).expect("plain");
    }
    j
}

/// `make_small_body`: a body a GM adds by hand, made of the system's own
/// nearest cloud (a comet nucleus of its icy reservoirs), or of a designed
/// field where the system has no formation record.
pub(crate) fn make(
    run: Option<&SystemRun>,
    universe_seed: &Seed,
    req: Make,
) -> Result<Response, Response> {
    let s = settings();
    let (kinds, pair, cloud_kind): (&[&str], Option<&str>, &str) = match req.kind {
        "asteroid" => (&[], None, "made"),
        "comet-nucleus" => (
            &["oort", "scattered-disc", "outer-belt"],
            None,
            "made-comet",
        ),
        "contact-binary" => (&[], Some("contact"), "made"),
        "binary-pair" => (&[], Some("separated"), "made"),
        k => {
            return Err(err(
                "bad_request",
                format!("kind {k}: asteroid, comet-nucleus, contact-binary or binary-pair"),
            ))
        }
    };
    if req.a_au.is_nan() || req.a_au <= 0.0 {
        return Err(err("bad_request", "a_au must be above 0"));
    }
    let address: Address = req
        .address
        .parse()
        .map_err(|e| err("bad_address", format!("{e}")))?;
    let mut pins = req.pins;
    pins.e = req.e;
    if pins.pair.is_none() {
        pins.pair = pair.map(str::to_string);
    }
    let clouds = run
        .and_then(|r| r.leftovers.as_ref())
        .map(|l| &l.clouds[..])
        .unwrap_or(&[]);
    let template = sse_small::nearest_cloud(clouds, req.a_au, kinds)
        .or_else(|| sse_small::nearest_cloud(clouds, req.a_au, &[]));
    let positive = |x: Option<f64>| x.filter(|v| *v > 0.0 && v.is_finite());
    let (field, host) = match (template, run) {
        (Some(t), Some(run)) => {
            let mass = match (positive(req.mass_kg), positive(req.diameter_km)) {
                (Some(m), _) => m,
                (_, Some(d)) => match pins.makeup {
                    Some(mk) => sse_small::mass_for_makeup(&mk, d, &s),
                    None => sse_small::mass_for_diameter(t, d, &s),
                },
                _ => {
                    return Err(err(
                        "bad_request",
                        "give a diameter_km or a mass_kg above 0",
                    ))
                }
            };
            (
                sse_small::made_field(t, address.clone(), cloud_kind, req.a_au, mass),
                host_of(run),
            )
        }
        _ => {
            // No cloud to make it of: a designed field of the pinned makeup,
            // or the recipe's own (rocky, or icy for a comet).
            let mk = pins.makeup.unwrap_or_else(|| {
                if req.kind == "comet-nucleus" {
                    Makeup::from_array([0.0, 0.3, 0.2, 0.5])
                } else {
                    Makeup::from_array([0.05, 0.85, 0.1, 0.0])
                }
            });
            let mass = match (positive(req.mass_kg), positive(req.diameter_km)) {
                (Some(m), _) => m,
                (_, Some(d)) => sse_small::mass_for_makeup(&mk, d, &s),
                _ => {
                    return Err(err(
                        "bad_request",
                        "give a diameter_km or a mass_kg above 0",
                    ))
                }
            };
            pins.makeup = Some(mk);
            (
                sse_small::designed_field(
                    address.clone(),
                    cloud_kind,
                    &mk,
                    req.a_au,
                    req.e.unwrap_or(0.1),
                    mass,
                ),
                run.map(host_of).unwrap_or_else(sunlike),
            )
        }
    };
    let seed = run.map(seed_of).unwrap_or(*universe_seed);
    let mut m = sse_small::resolve_pinned(&seed, &field, 1, &host, None, None, &s, &pins);
    m.address = address.to_string();
    let mut j = made_json(&m, template.filter(|_| run.is_some()), &host, &s);
    match template.filter(|_| run.is_some()) {
        Some(t) => j["made_from"] = t.address.to_string().into(),
        None => j["made_from"] = "designed".into(),
    }
    Ok(Response::SmallBodyIs {
        body: Some(j),
        notables: None,
    })
}

/// What `small_shape` is asked for.
pub(crate) struct Shape<'a> {
    pub address: &'a str,
    pub diameter_km: Option<f64>,
    pub mass_kg: Option<f64>,
    pub ice: Option<f64>,
    pub spin_hours: Option<f64>,
    pub a_au: Option<f64>,
}

/// `small_shape`: the body at an address as the one resolve makes it,
/// single, of the makeup its density or ice share says, with whether it is
/// round. The seed is the universe's (from `universe_seed`), so the same
/// address is the same lump everywhere.
pub(crate) fn shape(
    run: Option<&SystemRun>,
    universe_seed: &Seed,
    req: Shape,
) -> Result<Response, Response> {
    let s = settings();
    let address: Address = req
        .address
        .parse()
        .map_err(|e| err("bad_address", format!("{e}")))?;
    let positive = |x: Option<f64>| x.filter(|v| *v > 0.0 && v.is_finite());
    let (d, mass) = (positive(req.diameter_km), positive(req.mass_kg));
    let makeup = match (req.ice.filter(|x| x.is_finite()), d, mass) {
        (Some(ice), _, _) => {
            let ice = ice.clamp(0.0, 1.0);
            Makeup::from_array([0.0, 1.0 - ice, 0.0, ice])
        }
        (None, Some(d), Some(m)) => {
            let v = m::PI / 6.0 * m::pow(d * 1000.0, 3.0);
            sse_small::makeup_for_density(m / v, &s)
        }
        _ => Makeup::from_array([0.05, 0.85, 0.1, 0.0]),
    };
    let mass = match (mass, d) {
        (Some(m), _) => m,
        (None, Some(d)) => sse_small::mass_for_makeup(&makeup, d, &s),
        _ => {
            return Err(err(
                "bad_request",
                "give a diameter_km or a mass_kg above 0",
            ))
        }
    };
    let a_au = positive(req.a_au).unwrap_or(2.7);
    let field = sse_small::designed_field(address.clone(), "shape", &makeup, a_au, 0.1, mass);
    let pins = Pins {
        pair: Some("single".into()),
        spin_hours: positive(req.spin_hours),
        makeup: Some(makeup),
        ..Pins::default()
    };
    let host = run.map(host_of).unwrap_or_else(sunlike);
    let seed = *universe_seed;
    let mut b = sse_small::resolve_pinned(&seed, &field, 1, &host, None, None, &s, &pins);
    b.address = address.to_string();
    let mut j = serde_json::to_value(&b).expect("plain");
    j["round"] = (b.structure.kind == "relaxed").into();
    j["round_km"] = sse_small::round_km(&b.makeup, &s).into();
    Ok(Response::SmallBodyIs {
        body: Some(j),
        notables: None,
    })
}

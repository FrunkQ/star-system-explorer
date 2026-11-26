//! Small bodies up close (track SB, `sse-small`): `small_body` resolves one
//! member of a system's belt, ring or cloud into a definite body, or lists
//! the bodies its formation made before anyone looked (a shattered world's
//! bared core); `make_small_body` makes one a GM adds by hand, out of the
//! system's own nearest cloud. Nothing is stored: the same body comes back
//! on every ask, on every device.

use crate::err;
use crate::protocol::Response;
use sse_core::{Address, Seed};
use sse_formation::leftovers::Cloud;
use sse_formation::pipeline::SystemRun;
use sse_small::{Host, Member, Pins, Settings};

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
}

/// `make_small_body`: a body a GM adds by hand, made of the system's own
/// nearest cloud (a comet nucleus of its icy reservoirs).
pub(crate) fn make(run: Option<&SystemRun>, req: Make) -> Result<Response, Response> {
    let run = run.ok_or_else(|| {
        err(
            "no_formation",
            "this system has no formation run to make a small body from",
        )
    })?;
    let l = run.leftovers.as_ref().ok_or_else(|| {
        err(
            "no_formation",
            "this system's formation made no small bodies",
        )
    })?;
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
    let template = sse_small::nearest_cloud(&l.clouds, req.a_au, kinds)
        .or_else(|| sse_small::nearest_cloud(&l.clouds, req.a_au, &[]))
        .ok_or_else(|| {
            err(
                "no_formation",
                "this system has no clouds to make a small body from",
            )
        })?;
    let mass = match (req.mass_kg, req.diameter_km) {
        (Some(m), _) if m > 0.0 => m,
        (_, Some(d)) if d > 0.0 => sse_small::mass_for_diameter(template, d, &s),
        _ => {
            return Err(err(
                "bad_request",
                "give a diameter_km or a mass_kg above 0",
            ))
        }
    };
    let address: Address = req
        .address
        .parse()
        .map_err(|e| err("bad_address", format!("{e}")))?;
    let field = sse_small::made_field(template, address.clone(), cloud_kind, req.a_au, mass);
    let pins = Pins {
        pair: pair.map(str::to_string),
        e: req.e,
    };
    let host = host_of(run);
    let mut m = sse_small::resolve_pinned(&seed_of(run), &field, 1, &host, None, None, &s, &pins);
    m.address = address.to_string();
    let mut j = member_json(&m, template, &host, None, None, &s);
    j["made_from"] = template.address.to_string().into();
    Ok(Response::SmallBodyIs {
        body: Some(j),
        notables: None,
    })
}

/// `ring_structure` from a system's formation: the ring cloud at `cloud`, its
/// world, that world's moons and the ring's own largest members as the
/// sculptors (members big enough clear gaps; Cloud::resolve, the one method).
pub(crate) fn ring_ask_from_run(
    run: Option<&SystemRun>,
    cloud: &str,
    variant: u64,
    samples: Option<usize>,
    window: Option<[f64; 2]>,
) -> Result<sse_small::ring::RingAsk, Response> {
    use sse_small::ring::{RingAsk, RingHost, RingIn, Sculptor};
    let run = run.ok_or_else(|| {
        err(
            "no_formation",
            "this system has no formation run, so its rings have no makeup to read",
        )
    })?;
    let l = run
        .leftovers
        .as_ref()
        .ok_or_else(|| err("no_formation", "this system's formation made no rings"))?;
    let c = l
        .clouds
        .iter()
        .find(|c| c.address.to_string() == cloud && c.kind == "ring")
        .ok_or_else(|| err("not_found", format!("no ring {cloud} in this system")))?;
    let around = c.around.clone().unwrap_or_default();
    let w = l.worlds.iter().find(|w| w.id == around).ok_or_else(|| {
        err(
            "not_found",
            format!("the world {around} that ring {cloud} goes round is gone"),
        )
    })?;
    let au = sse_core::sheet::AU;
    let me = sse_core::sheet::M_EARTH;
    let radius_m = w.radius_earth() * sse_core::sheet::R_EARTH_MEAN;
    let mut sculptors: Vec<Sculptor> = l
        .moons
        .iter()
        .filter(|mo| mo.parent == around)
        .map(|mo| Sculptor {
            id: mo.address.to_string(),
            name: mo.address.to_string(),
            a_m: mo.a_planet_radii * radius_m,
            mass_kg: mo.mass_earth() * me,
        })
        .collect();
    let seed = seed_of(run);
    for k in 1..=3u64 {
        let b = c.resolve(&seed, k);
        sculptors.push(Sculptor {
            id: b.address.to_string(),
            name: b.address.to_string(),
            a_m: b.a_au * au,
            mass_kg: b.mass_kg,
        });
    }
    let s = c.solid.map(|x| x as f64);
    let t: f64 = s.iter().sum();
    let makeup = (t > 0.0).then(|| sse_small::Makeup {
        metal: s[0] / t,
        rock: s[1] / t,
        carbon: s[2] / t,
        ice: (s[3] + s[4] + s[5]) / t,
    });
    Ok(RingAsk {
        key: cloud.to_string(),
        variant,
        host: RingHost {
            mass_kg: w.mass_earth() * me,
            radius_m,
            ..Default::default()
        },
        sculptors,
        ring: RingIn {
            inner_m: c.a_min_au * au,
            outer_m: c.a_max_au * au,
            mass_kg: Some(c.mass_earth() * me),
            makeup,
            ..Default::default()
        },
        samples,
        window,
    })
}

/// `ring_structure`: one ring, whole (sse-small ring.rs).
pub(crate) fn ring_structure(
    ask: &sse_small::ring::RingAsk,
    seed: &Seed,
    sheet: &sse_reference::Reference,
) -> Result<Response, Response> {
    let s = settings();
    // The stack's sheet; a stack loaded without one (tests, toy stacks) reads the shipped sheet, as the settings do.
    let base;
    let sheet = if sheet.row("substances", &s.rings.ice).is_some() {
        sheet
    } else {
        base = sse_reference::Reference::base();
        &base
    };
    let r = sse_small::ring::structure(ask, &s.rings, &s.makeup.grain_density_kg_m3, seed, sheet)
        .map_err(|e| err("bad_request", e))?;
    Ok(Response::RingStructureIs {
        ring: serde_json::to_value(r).expect("plain"),
    })
}

use roster_proto::book::{ledger, Campaign};
use roster_proto::roster::{BandId, Roster};
use roster_proto::system::{Settings, System};
use sse_core::Seed;
use sse_formation::{disc, growth, hurley::Fits, leftovers, scenario, violent};

static mut SYS: Option<&'static System> = None;
static mut RO: Option<Roster<'static>> = None;
static mut IDS: Vec<String> = Vec::new();

#[allow(static_mut_refs)]
#[no_mangle]
pub extern "C" fn setup(n_exact: u32) -> u32 {
    let seed = Seed::from_text("cc-roster-1");
    let addr = scenario::system(16);
    let fits = Fits::new(0.02);
    let dsc = disc::Disc::new(disc::disc_params(&seed, &addr, 1.0, fits.l_zams(1.0), 0.0, 0.0, 0.55, 300.0, None));
    let g = growth::grow(&seed, &addr, &dsc);
    let u = violent::upheave(&seed, &addr, &dsc, &g);
    let l = leftovers::leftovers(&seed, &addr, &dsc, &u);
    let st = Settings { n_exact: n_exact as u64, ..Settings::default() };
    let sys: &'static System = Box::leak(Box::new(System::build(seed, addr, &u, &l, 4570.0, 1.0, &st)));
    unsafe {
        SYS = Some(sys);
        RO = Some(Roster::new(sys));
    }
    sys.reservoirs.len() as u32
}

#[allow(static_mut_refs)]
fn ro() -> &'static mut Roster<'static> {
    unsafe { RO.as_mut().unwrap() }
}

/// Plans for every band.
#[no_mangle]
pub extern "C" fn plans() -> u32 {
    let r = ro();
    let b = r.all_bands();
    for (id, _) in &b {
        r.plan(*id);
    }
    b.len() as u32
}

/// n forward reads in the largest comet band.
#[allow(static_mut_refs)]
#[no_mangle]
pub extern "C" fn forward(n: u32) -> u32 {
    let sys = unsafe { SYS.unwrap() };
    let r = ro();
    let (id, k0, k1) = r
        .all_bands()
        .into_iter()
        .filter(|(id, _)| sys.reservoirs[id.res].kind == "oort")
        .map(|(id, _)| {
            let (a, b) = sys.reservoirs[id.res].classes[id.class].bands[id.band];
            (id, a, b)
        })
        .max_by_key(|x| x.2 - x.1)
        .unwrap();
    let mut left = 0;
    for q in 0..n as u64 {
        let k = k0 + roster_proto::perm::mix(q) % (k1 - k0);
        left += r.fate(id, k).is_some() as u32;
    }
    left
}

/// The listed craters of every solid world (whole life), keeping their ids.
#[allow(static_mut_refs)]
#[no_mangle]
pub extern "C" fn lists() -> u32 {
    let sys = unsafe { SYS.unwrap() };
    let r = ro();
    let mut n = 0;
    for wi in 0..sys.worlds.len() {
        if !sys.worlds[wi].surface {
            continue;
        }
        let (c, _, _) = r.listed_craters(wi, (0.0, sys.horizon_myr), sys.age_myr, 2000);
        n += c.len() as u32;
        unsafe { IDS.extend(c.into_iter().map(|x| x.id)) };
    }
    n
}

/// Crater ids back to impactors.
#[allow(static_mut_refs)]
#[no_mangle]
pub extern "C" fn backward(n: u32) -> u32 {
    let r = ro();
    let ids = unsafe { &IDS };
    let mut s = 0u32;
    for i in 0..n as usize {
        s = s.wrapping_add(r.impactor_of(&ids[i % ids.len()]).rank as u32);
    }
    s
}

/// One class's exact books at the present (the debris belt's core class).
#[allow(static_mut_refs)]
#[no_mangle]
pub extern "C" fn books() -> u32 {
    let sys = unsafe { SYS.unwrap() };
    let deb = sys.reservoirs.iter().position(|r| r.kind == "debris-belt").unwrap();
    let lg = ledger(ro(), &Campaign::default(), deb, 0, sys.age_myr);
    let _ = BandId { res: 0, class: 0, band: 0 };
    lg.exact_members_walked as u32
}

/// FNV digest of every listed crater (id and diameter bits), in list order.
#[allow(static_mut_refs)]
#[no_mangle]
pub extern "C" fn digest() -> u64 {
    let sys = unsafe { SYS.unwrap() };
    let r = ro();
    let mut h: u64 = 0xcbf29ce484222325;
    for wi in 0..sys.worlds.len() {
        if !sys.worlds[wi].surface {
            continue;
        }
        let (c, _, _) = r.listed_craters(wi, (0.0, sys.horizon_myr), sys.age_myr, 2000);
        for x in c {
            for b in x.id.bytes().chain(x.d_km.to_bits().to_be_bytes()) {
                h ^= b as u64;
                h = h.wrapping_mul(0x100000001b3);
            }
        }
    }
    h
}

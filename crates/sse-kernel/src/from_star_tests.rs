//! `from_star` on run_formation and adopt (V3 gap 17 part 2b, "Add system
//! here"): a drawn star grows a real system that inherits its birth.

fn ask(k: &mut crate::Kernel, req: serde_json::Value) -> serde_json::Value {
    serde_json::from_str(&k.handle_json(&req.to_string())).unwrap()
}

fn open_with_a_star() -> (crate::Kernel, String) {
    let mut k = crate::Kernel::new();
    let r = ask(
        &mut k,
        serde_json::json!({"op":"open_universe","id":"g3","seed":"g3"}),
    );
    assert_eq!(r["type"], "universe_opened", "{r}");
    let sky = ask(
        &mut k,
        serde_json::json!({"op":"sky_from_here","limit_mag":3,"galaxies":false,
            "cone":{"l_deg":90,"b_deg":0,"half_angle_deg":60}}),
    );
    let stars = sky["sky"]["stars"].as_array().unwrap();
    let addr = stars[stars.len() / 2]["address"]
        .as_str()
        .unwrap()
        .to_string();
    (k, addr)
}

#[test]
fn a_drawn_star_grows_a_system_with_its_own_birth() {
    let (mut k, star) = open_with_a_star();
    let run = ask(
        &mut k,
        serde_json::json!({"op":"run_formation","seed":"here","from_star":star}),
    );
    assert_eq!(
        run["type"],
        "formation_ran",
        "{}",
        run.to_string().chars().take(400).collect::<String>()
    );
    // Promoted on the way, so its description is fixed.
    let p = ask(
        &mut k,
        serde_json::json!({"op":"promote_star","address":star}),
    );
    assert_eq!(p["already"], true, "{p}");
    // The run inherits the star's birth: the birth's address is the star's.
    let text = run["run"].to_string();
    let at = text
        .find("\"birth\"")
        .expect("the run keeps its source birth");
    eprintln!("{}", &text[at..(at + 400).min(text.len())]);
    assert!(
        text[at..].starts_with(&format!("\"birth\":{{\"address\":\"{star}\"")),
        "{}",
        &text[at..(at + 300).min(text.len())]
    );
    // The same star adopted becomes the system at its address.
    let a = ask(
        &mut k,
        serde_json::json!({"op":"adopt","formation":{"seed":"here","from_star":star}}),
    );
    assert_eq!(a["type"], "adopted", "{a}");
    let rs = ask(
        &mut k,
        serde_json::json!({"op":"resolved_system","system":star}),
    );
    assert_eq!(
        rs["type"],
        "system_resolved",
        "{}",
        rs.to_string().chars().take(300).collect::<String>()
    );
    // Twice is refused: the system is grown.
    let again = ask(
        &mut k,
        serde_json::json!({"op":"adopt","formation":{"seed":"here","from_star":star}}),
    );
    assert_eq!(again["code"], "already_adopted", "{again}");
    // Not with a birth of its own.
    let both = ask(
        &mut k,
        serde_json::json!({"op":"run_formation","seed":"x","from_star":star,"envelope":{}}),
    );
    assert_eq!(both["code"], "bad_request", "{both}");
}

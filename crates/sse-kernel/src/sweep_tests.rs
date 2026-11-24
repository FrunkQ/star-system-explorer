//! K1 `sweep`: Mars's orbit swept outwards on a what-if branch.

fn send(k: &mut crate::Kernel, req: serde_json::Value) -> serde_json::Value {
    serde_json::from_str(&k.handle_json(&req.to_string())).unwrap()
}

const SYS: &str = "u:sw/sys:0";

fn kernel() -> crate::Kernel {
    let mut k = crate::Kernel::new();
    send(
        &mut k,
        serde_json::json!({"op":"open_universe","id":"sw","seed":"sw"}),
    );
    let au = sse_core::sheet::AU;
    let r = send(
        &mut k,
        serde_json::json!({"op":"set_authored_batch","occurred_at_s":0,"edits":[
            {"address":SYS,"field":"system.age","value":1.45e17},
            {"address":"u:sw/sys:0/star:0","field":"body.mass","value":1.989e30},
            {"address":"u:sw/sys:0/star:0","field":"body.radius","value":6.957e8},
            {"address":"u:sw/sys:0/star:0","field":"star.luminosity","value":3.828e26},
            {"address":"u:sw/sys:0/star:0","field":"star.effective_temperature","value":5772.0},
            {"address":"u:sw/sys:0/body:0","field":"body.mass","value":1.898e27},
            {"address":"u:sw/sys:0/body:0","field":"body.radius","value":6.99e7},
            {"address":"u:sw/sys:0/body:0","field":"orbit.host","value":"u:sw/sys:0/star:0"},
            {"address":"u:sw/sys:0/body:0","field":"orbit.semi_major_axis","value":5.2 * au},
            {"address":"u:sw/sys:0/body:1","field":"body.mass","value":6.42e23},
            {"address":"u:sw/sys:0/body:1","field":"body.radius","value":3.39e6},
            {"address":"u:sw/sys:0/body:1","field":"orbit.host","value":"u:sw/sys:0/star:0"},
            {"address":"u:sw/sys:0/body:1","field":"orbit.semi_major_axis","value":1.52 * au}
        ]}),
    );
    assert_eq!(r["type"], "authored", "{r}");
    k
}

#[test]
fn a_sweep_finds_where_the_orbit_stops_lasting_and_moves_nothing() {
    let mut k = kernel();
    let au = sse_core::sheet::AU;
    let before = send(&mut k, serde_json::json!({"op":"state_hash","at_s":0}));
    let job = send(
        &mut k,
        serde_json::json!({"op":"start_job","job":{"kind":"sweep","system":SYS,
            "address":"u:sw/sys:0/body:1","field":"orbit.semi_major_axis",
            "from":1.0 * au,"to":5.0 * au,"count":9,"results":["orbit.semi_major_axis"]}}),
    );
    assert_eq!(job["type"], "job_started", "{job}");
    let id = job["job"].as_u64().unwrap();
    let mut last = serde_json::Value::Null;
    for _ in 0..100 {
        last = send(
            &mut k,
            serde_json::json!({"op":"step_job","job":id,"budget":3}),
        );
        if last["type"] == "job_done" {
            break;
        }
        assert_eq!(last["type"], "job_progress", "{last}");
    }
    assert_eq!(last["type"], "job_done", "{last}");
    let r = &last["result"];
    let steps = r["steps"].as_array().unwrap();
    assert_eq!(steps.len(), 9);
    assert_eq!(steps[0]["verdict"], "lasts", "{}", steps[0]);
    assert_ne!(
        steps[8]["verdict"], "lasts",
        "a Mars beside Jupiter does not last: {}",
        steps[8]
    );
    assert_eq!(steps[4]["results"]["orbit.semi_major_axis"], 3.0 * au);
    let tips = r["tipping_points"].as_array().unwrap();
    assert!(!tips.is_empty(), "{r}");
    let b = tips[0]["between"].as_array().unwrap();
    let (lo, hi) = (b[0].as_f64().unwrap(), b[1].as_f64().unwrap());
    assert!(
        hi - lo <= 0.5 * au / 16.0 + 1.0,
        "narrowed four times: {lo} {hi}"
    );
    eprintln!(
        "tipping {}: {:.3} to {:.3} au",
        tips[0]["to"],
        lo / au,
        hi / au
    );
    // The branch is gone and the main line never moved.
    let after = send(&mut k, serde_json::json!({"op":"state_hash","at_s":0}));
    assert_eq!(before, after);
    let again = send(
        &mut k,
        serde_json::json!({"op":"step_job","job":id,"budget":1}),
    );
    assert_eq!(again["code"], "unknown_job", "{again}");
}

#[test]
fn a_sweep_can_be_cancelled_and_refuses_bad_ranges() {
    let mut k = kernel();
    let bad = send(
        &mut k,
        serde_json::json!({"op":"start_job","job":{"kind":"sweep","system":SYS,
            "address":"u:sw/sys:0/body:1","field":"body.mass","from":1.0,"to":1.0,"count":4}}),
    );
    assert_eq!(bad["code"], "bad_job", "{bad}");
    let job = send(
        &mut k,
        serde_json::json!({"op":"start_job","job":{"kind":"sweep","system":SYS,
            "address":"u:sw/sys:0/body:1","field":"body.mass","from":1e22,"to":1e25,"count":4,"log":true}}),
    );
    let id = job["job"].as_u64().unwrap();
    send(
        &mut k,
        serde_json::json!({"op":"step_job","job":id,"budget":1}),
    );
    let c = send(&mut k, serde_json::json!({"op":"cancel_job","job":id}));
    assert_eq!(c["type"], "job_cancelled", "{c}");
}

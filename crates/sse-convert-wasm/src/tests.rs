use super::*;

fn root() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// The module returns exactly the kernel's `import_v3` body (without
/// reconstruction), so the converter and SSE4 read a save the same way.
#[test]
fn same_json_as_the_kernel_op() {
    let save =
        std::fs::read_to_string(root().join("tests/fixtures/v3/synthetic-sol.json")).unwrap();
    let mine: serde_json::Value = serde_json::from_str(&import_v3(&save, "sol")).unwrap();
    let req =
        serde_json::json!({"op": "import_v3", "save": save, "universe_id": "sol"}).to_string();
    let mut k: serde_json::Value =
        serde_json::from_str(&sse_kernel::Kernel::new().handle_json(&req)).unwrap();
    assert_eq!(k["type"], "v3_imported", "{k}");
    let o = k.as_object_mut().unwrap();
    o.remove("type");
    assert_eq!(o.remove("reconstructed"), Some(serde_json::json!([])));
    assert_eq!(serde_json::Value::Object(o.clone()), mine);
    assert!(mine["facts"].as_object().unwrap().len() > 5);
}

#[test]
fn a_bad_save_is_an_error_not_a_panic() {
    let v: serde_json::Value = serde_json::from_str(&import_v3("not json", "x")).unwrap();
    assert!(v["error"].is_string(), "{v}");
    let ver: serde_json::Value = serde_json::from_str(&importer_version()).unwrap();
    assert_eq!(ver["importer"], sse_import_v3::IMPORTER_VERSION);
}

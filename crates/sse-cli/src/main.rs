//! `sse` command-line tool.
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("harness") => {
            let r = sse_core::harness::run();
            print!("{}", sse_core::harness::report_json(&r));
            if r.encoding_failures == 0 {
                ExitCode::SUCCESS
            } else {
                ExitCode::FAILURE
            }
        }
        Some("import-v3") => import_v3(&args[1..]),
        Some("reconstruct-v3") => reconstruct_v3(&args[1..]),
        _ => {
            eprintln!(
                "usage: sse <command>\n  harness                      run the cross-device hash harness and print digests\n  import-v3 <file> [universe]  import a V3 starmap.json or system.json; print the report\n  reconstruct-v3 <file>        give each imported system a reconstructed formation history; print a summary"
            );
            ExitCode::from(2)
        }
    }
}

/// `sse import-v3 <file> [universe-id]`: the report goes to stdout, a summary
/// to stderr. A `.sse.zip` must be unpacked first (its `starmap.json` or
/// `system.json` is the save).
fn import_v3(args: &[String]) -> ExitCode {
    let Some(path) = args.first() else {
        eprintln!("usage: sse import-v3 <starmap.json | system.json> [universe-id]");
        return ExitCode::from(2);
    };
    if path.ends_with(".zip") {
        eprintln!("unpack the .sse.zip first and pass its starmap.json or system.json");
        return ExitCode::from(2);
    }
    let text = match std::fs::read_to_string(path) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("{path}: {e}");
            return ExitCode::FAILURE;
        }
    };
    let id = args.get(1).map(String::as_str).unwrap_or("v3-import");
    match sse_import_v3::import_json(&text, id) {
        Ok(im) => {
            let r = &im.report;
            print!("{}", r.to_json());
            eprintln!(
                "{}: {} systems, {} bodies, {} constructs, {} facts; {} derived values compared, {} outside tolerance; {} unclassified fields kept; state {}",
                path,
                r.counts.systems,
                r.counts.bodies,
                r.counts.constructs,
                r.counts.facts,
                r.differences.len(),
                r.differences_outside_tolerance(),
                r.unclassified.values().sum::<u32>(),
                r.state_hash
            );
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("{path}: {e}");
            ExitCode::FAILURE
        }
    }
}

/// `sse reconstruct-v3 <file>`: each system's star and planets become pins
/// and the formation suite finds a history for them (F9). JSON to stdout.
fn reconstruct_v3(args: &[String]) -> ExitCode {
    let Some(path) = args.first() else {
        eprintln!("usage: sse reconstruct-v3 <starmap.json | system.json>");
        return ExitCode::from(2);
    };
    let text = match std::fs::read_to_string(path) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("{path}: {e}");
            return ExitCode::FAILURE;
        }
    };
    let im = match sse_import_v3::import_json(&text, "v3-import") {
        Ok(im) => im,
        Err(e) => {
            eprintln!("{path}: {}", e.0);
            return ExitCode::FAILURE;
        }
    };
    let mut out = vec![];
    for k in sse_import_v3::reconstruct::known_systems(&im) {
        match sse_import_v3::reconstruct::reconstruct(&k, 16) {
            Ok(r) => out.push(sse_import_v3::reconstruct::summary(&k, &r)),
            Err(e) => out.push(serde_json::json!({"system": k.system.to_string(), "error": e})),
        }
    }
    eprintln!("{path}: {} systems reconstructed", out.len());
    println!("{}", serde_json::to_string_pretty(&out).expect("json"));
    ExitCode::SUCCESS
}

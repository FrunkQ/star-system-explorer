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
        Some("catalogue") => catalogue(&args[1..]),
        _ => {
            eprintln!(
                "usage: sse <command>\n  harness                      run the cross-device hash harness and print digests\n  import-v3 <file> [universe]  import a V3 starmap.json or system.json; print the report\n  reconstruct-v3 <file>        give each imported system a reconstructed formation history; print a summary\n  catalogue pack <in.json> <out.ssecat> [--sort] [--block-rows N]\n                               pack a text-form catalogue (sse.catalogue/1)\n  catalogue show <file> [--min-size S | --rank K] [--limit N]\n                               print the import preview of either form"
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

/// `sse catalogue pack|show`: the build-script end of RW3. Build scripts
/// (run where the catalogue hosts can be reached) write the text form; this
/// packs it into the byte-ranged form the store serves.
fn catalogue(args: &[String]) -> ExitCode {
    let usage = || {
        eprintln!(
            "usage: sse catalogue pack <in.json> <out.ssecat> [--sort] [--block-rows N]\n       sse catalogue show <file> [--min-size S | --rank K] [--limit N]"
        );
        ExitCode::from(2)
    };
    let flag = |name: &str| {
        args.iter()
            .position(|a| a == name)
            .and_then(|i| args.get(i + 1))
    };
    match (args.first().map(String::as_str), args.get(1)) {
        (Some("pack"), Some(input)) => {
            let Some(out) = args.get(2) else {
                return usage();
            };
            let text = match std::fs::read_to_string(input) {
                Ok(t) => t,
                Err(e) => {
                    eprintln!("{input}: {e}");
                    return ExitCode::FAILURE;
                }
            };
            let block_rows = match flag("--block-rows").map(|s| s.parse::<u32>()) {
                None => sse_catalogue::DEFAULT_BLOCK_ROWS,
                Some(Ok(n)) => n,
                Some(Err(_)) => return usage(),
            };
            let cat = if args.iter().any(|a| a == "--sort") {
                sse_catalogue::Catalogue::from_text_unsorted(&text).and_then(|mut c| {
                    c.sort_by_size()?;
                    Ok(c)
                })
            } else {
                sse_catalogue::Catalogue::from_text(&text).map(|(c, _)| c)
            };
            let packed = cat.and_then(|c| c.pack(block_rows));
            match packed {
                Ok(bytes) => {
                    if let Err(e) = std::fs::write(out, &bytes) {
                        eprintln!("{out}: {e}");
                        return ExitCode::FAILURE;
                    }
                    eprintln!("{out}: {} bytes", bytes.len());
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    eprintln!("{input}: refused: {e}");
                    ExitCode::FAILURE
                }
            }
        }
        (Some("show"), Some(input)) => {
            let bytes = match std::fs::read(input) {
                Ok(b) => b,
                Err(e) => {
                    eprintln!("{input}: {e}");
                    return ExitCode::FAILURE;
                }
            };
            let cut = match (flag("--min-size"), flag("--rank")) {
                (Some(s), None) => match s.parse() {
                    Ok(s) => sse_catalogue::Cut::Size(s),
                    Err(_) => return usage(),
                },
                (None, Some(k)) => match k.parse() {
                    Ok(k) => sse_catalogue::Cut::Rank(k),
                    Err(_) => return usage(),
                },
                (None, None) => sse_catalogue::Cut::All,
                _ => return usage(),
            };
            let limit = flag("--limit").and_then(|s| s.parse().ok()).unwrap_or(20);
            match sse_catalogue::read_bytes(&bytes, cut) {
                Ok((h, r, p)) => {
                    let v = sse_catalogue::preview(&h, &r, p.as_ref(), limit);
                    println!("{}", serde_json::to_string_pretty(&v).expect("json"));
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    eprintln!("{input}: refused: {e}");
                    ExitCode::FAILURE
                }
            }
        }
        _ => usage(),
    }
}

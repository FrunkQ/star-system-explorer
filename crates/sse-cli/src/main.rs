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
        _ => {
            eprintln!("usage: sse <command>\n  harness   run the cross-device hash harness and print digests");
            ExitCode::from(2)
        }
    }
}

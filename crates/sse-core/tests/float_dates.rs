//! TP1's ledger of dates held as doubles: it may only shrink.
//!
//! A date is integers (`sse_core::time::Instant`); a double cannot resolve
//! one second at a star's age. This counts, per file, the places that still
//! turn an absolute date into a double (`at_s as f64` and its kin) in the
//! Rust crates, and the screens' clock fields kept in years (`now_yr`,
//! `campaign_now_yr`, `campaign_epoch_yr`, `clock_yr`), and checks the counts
//! against `handover/packages/TP1-float-dates.txt`. A count above its line
//! fails: use `Instant::since` or `angle_since`, or a whole-second field. A
//! count below its line fails too, so the ledger is lowered in the same
//! change that removes a use (and a line at 0 is deleted).
//!
//! The screens' lines are also checked by `tests/ui/float-dates.test.ts` (in
//! `npm run ui:test`), because CI skips the Rust jobs for a screens-only
//! change; keep the two walks and word rules the same.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

const LEDGER: &str = "handover/packages/TP1-float-dates.txt";
/// Absolute dates by name (the last part of the cast expression).
/// A bare `t` counts too: it is how the kernel names a date most often
/// (`orbits.rs` held `t as f64` before TP1), at the price of a few lines
/// where `t` is something else (a temperature in `sse-sense/src/fog.rs`).
const DATE_NAMES: &[&str] = &[
    "t", "at_s", "now_s", "t_s", "epoch_s", "time_s", "date_s", "birth_s", "when_s", "depart_s",
    "arrive_s",
];
const YEAR_CLOCKS: &[&str] = &["now_yr", "campaign_now_yr", "campaign_epoch_yr", "clock_yr"];

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn walk(dir: &Path, exts: &[&str], out: &mut Vec<PathBuf>) {
    let Ok(rd) = fs::read_dir(dir) else { return };
    let mut entries: Vec<PathBuf> = rd.filter_map(|e| e.ok().map(|e| e.path())).collect();
    entries.sort();
    for p in entries {
        let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if p.is_dir() {
            if !matches!(name, "target" | "node_modules" | "dist" | "generated")
                && !name.starts_with('.')
            {
                walk(&p, exts, out);
            }
        } else if exts.iter().any(|e| name.ends_with(e)) {
            out.push(p);
        }
    }
}

fn ident_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

/// Casts of a date to a double on one line: `<name> as f64` where the
/// expression's last part is a date name, or the `.s` of an `Instant`.
fn date_casts(line: &str) -> usize {
    let mut n = 0;
    let mut rest = line;
    while let Some(i) = rest.find(" as f64") {
        let before = &rest[..i];
        let word: String = before
            .chars()
            .rev()
            .take_while(|&c| ident_char(c))
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect();
        let dot = before.len() > word.len() && before[..before.len() - word.len()].ends_with('.');
        if DATE_NAMES.contains(&word.as_str()) || (word == "s" && dot) {
            n += 1;
        }
        rest = &rest[i + 7..];
    }
    n
}

fn word_count(line: &str, words: &[&str]) -> usize {
    let mut n = 0;
    for w in words {
        let mut from = 0;
        while let Some(i) = line[from..].find(w) {
            let a = from + i;
            let b = a + w.len();
            let left = line[..a].chars().next_back().is_none_or(|c| !ident_char(c));
            let right = line[b..].chars().next().is_none_or(|c| !ident_char(c));
            if left && right {
                n += 1;
            }
            from = b;
        }
    }
    n
}

fn counts() -> BTreeMap<String, usize> {
    let root = root();
    let mut out = BTreeMap::new();
    let mut tally = |files: Vec<PathBuf>, f: &dyn Fn(&str) -> usize| {
        for p in files {
            let text = fs::read_to_string(&p).unwrap_or_default();
            let n: usize = text.lines().map(f).sum();
            if n > 0 {
                let rel = p.strip_prefix(&root).unwrap_or(&p);
                let key = rel.to_string_lossy().replace('\\', "/");
                out.insert(key, n);
            }
        }
    };
    let mut rs = vec![];
    walk(&root.join("crates"), &[".rs"], &mut rs);
    rs.retain(|p| !p.ends_with("sse-core/tests/float_dates.rs"));
    tally(rs, &date_casts);
    let mut ui = vec![];
    walk(&root.join("packages"), &[".ts", ".svelte"], &mut ui);
    tally(ui, &|l: &str| word_count(l, YEAR_CLOCKS));
    out
}

fn ledger() -> BTreeMap<String, usize> {
    let text = fs::read_to_string(root().join(LEDGER)).expect("the TP1 ledger");
    text.lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(|l| {
            let (n, path) = l.split_once(' ').expect("`count path`");
            (path.trim().to_string(), n.parse().expect("a count"))
        })
        .collect()
}

#[test]
fn dates_as_doubles_only_shrink() {
    let now = counts();
    let was = ledger();
    let mut problems = vec![];
    for (path, &n) in &now {
        match was.get(path) {
            None => problems.push(format!(
                "{path}: {n} new date(s) held as a double; keep dates as `Instant` and subtract with `since`"
            )),
            Some(&w) if n > w => problems.push(format!(
                "{path}: {n} dates held as doubles, the ledger allows {w}; use `Instant::since`"
            )),
            Some(&w) if n < w => problems.push(format!(
                "{path}: down to {n} from {w}, well done: lower its line in {LEDGER}"
            )),
            _ => {}
        }
    }
    for path in was.keys().filter(|p| !now.contains_key(*p)) {
        problems.push(format!("{path}: none left: delete its line in {LEDGER}"));
    }
    if !problems.is_empty() {
        let fresh: Vec<String> = now.iter().map(|(p, n)| format!("{n} {p}")).collect();
        panic!(
            "{}\n\nThe counts today, for the ledger:\n{}",
            problems.join("\n"),
            fresh.join("\n")
        );
    }
}

#[test]
fn the_counter_sees_what_it_should() {
    assert_eq!(date_casts("let t = w.at_s as f64;"), 1);
    assert_eq!(date_casts("(self.now.s as f64, x.t_s as f64)"), 2);
    assert_eq!(date_casts("let p = period_s as f64; let s = s as f64;"), 0);
    // The line TP1 fixed is seen.
    assert_eq!(
        date_casts("let mean = self.m0 + n * (t as f64 - self.epoch);"),
        1
    );
    assert_eq!(
        word_count("ws.campaign_now_yr = now_yr + 1", YEAR_CLOCKS),
        2
    );
    assert_eq!(word_count("snow_yr my_clock_yrs", YEAR_CLOCKS), 0);
}

//! The reference subsector (an invented one, "Lanternfall", subsector F of
//! the invented sector "Kestrel Reach") in every format.

use sse_sector_io::*;

const TAB: &str = include_str!("../../../tests/fixtures/sector/lanternfall.tab");
const SEC: &str = include_str!("../../../tests/fixtures/sector/lanternfall.sec");
const LINES: &str = include_str!("../../../tests/fixtures/sector/lanternfall.txt");
const MESSY: &str = include_str!("../../../tests/fixtures/sector/lanternfall-messy.txt");
const XML: &str = include_str!("../../../tests/fixtures/sector/kestrel-reach.xml");

/// The fields every world format carries (the T5 extensions and `raw` cleared).
fn core(w: &WorldLine) -> WorldLine {
    WorldLine {
        importance: None,
        economic: None,
        cultural: None,
        nobility: None,
        worlds: None,
        ru: None,
        raw: String::new(),
        ..w.clone()
    }
}

fn cores(f: &SectorFile) -> Vec<WorldLine> {
    f.worlds.iter().map(core).collect()
}

fn fields(f: &SectorFile) -> Vec<WorldLine> {
    f.worlds
        .iter()
        .map(|w| WorldLine {
            raw: String::new(),
            ..w.clone()
        })
        .collect()
}

#[test]
fn tab_reads_the_reference_subsector() {
    let f = parse_t5_tab(TAB);
    assert!(f.warnings.is_empty(), "{:?}", f.warnings);
    assert_eq!(f.worlds.len(), 12);
    assert_eq!(f.abbreviation.as_deref(), Some("Kstr"));
    let a = &f.worlds[0];
    assert_eq!(
        (a.hex.as_str(), a.name.as_str(), a.uwp.as_str()),
        ("0912", "Ashgrove", "A867975-9")
    );
    assert_eq!(a.bases, ["N", "S"]);
    assert_eq!(a.remarks, ["Hi", "Ga"]);
    assert_eq!((a.pbg.as_str(), a.allegiance.as_str()), ("712", "Kv"));
    assert_eq!(a.importance.as_deref(), Some("+3"));
    assert_eq!(a.economic.as_deref(), Some("D7E+2"));
    assert_eq!(a.cultural.as_deref(), Some("9A6C"));
    assert_eq!(
        (a.nobility.as_deref(), a.worlds, a.ru.as_deref()),
        (Some("BcC"), Some(9), Some("2156"))
    );
    let g = &f.worlds[6];
    assert_eq!(g.stars, ["F5 IV", "D"]);
    assert_eq!(f.worlds[2].zone, Some('A'));
    assert_eq!(f.worlds[4].zone, Some('R'));
    assert_eq!(f.worlds[4].stars, ["BD"]);
    assert_eq!(f.worlds[2].nobility, None);
    assert!(f.worlds.iter().all(|w| w.subsector() == Some('F')));
}

#[test]
fn every_format_gives_the_same_worlds() {
    let tab = parse_t5_tab(TAB);
    let want = cores(&tab);
    let sec = parse_sec(SEC);
    assert!(sec.warnings.is_empty(), "{:?}", sec.warnings);
    assert_eq!(cores(&sec), want);
    assert_eq!(sec.name.as_deref(), Some("Kestrel Reach"));
    assert_eq!(sec.subsectors[&'F'], "Lanternfall");
    // The legacy letters are expanded and the original kept in raw.
    assert_eq!(sec.worlds[3].bases, ["N", "W"]);
    assert!(sec.worlds[3].raw.contains("C766887-7 B "));

    let lines = parse_uwp_lines(LINES);
    assert!(lines.warnings.is_empty(), "{:?}", lines.warnings);
    assert_eq!(cores(&lines), want);
    assert_eq!(lines.name.as_deref(), Some("Kestrel Reach"));
}

#[test]
fn sec_without_its_ruler_reads_the_same() {
    let no_ruler: String = SEC
        .lines()
        .filter(|l| !l.starts_with("----"))
        .map(|l| format!("{l}\n"))
        .collect();
    let f = parse_sec(&no_ruler);
    assert!(f.warnings.is_empty(), "{:?}", f.warnings);
    assert_eq!(cores(&f), cores(&parse_t5_tab(TAB)));
}

#[test]
fn messy_paste_keeps_good_lines_and_warns_on_the_rest() {
    let f = parse_uwp_lines(MESSY);
    assert_eq!(cores(&f), cores(&parse_t5_tab(TAB)));
    let lines: Vec<usize> = f.warnings.iter().map(|w| w.line).collect();
    // Title line, column header, the unknown token "wet!", the line with no UWP.
    assert_eq!(lines, [1, 3, 11, 12], "{:?}", f.warnings);
    assert!(f.warnings[2].text.contains("wet!"));
    assert!(f.warnings[3].raw.starts_with("Moonless Shoal"));
}

#[test]
fn xml_metadata_and_merge() {
    let meta = parse_sector_xml(XML);
    assert!(meta.warnings.is_empty(), "{:?}", meta.warnings);
    assert_eq!(meta.name.as_deref(), Some("Kestrel Reach"));
    assert_eq!(meta.abbreviation.as_deref(), Some("Kstr"));
    assert_eq!(meta.subsectors.len(), 3);
    assert_eq!(meta.subsectors[&'K'], "Siltmoor & Vey");
    assert!(meta.worlds.is_empty());

    let merged = parse_uwp_lines(MESSY).with_metadata(meta.clone());
    assert_eq!(merged.name.as_deref(), Some("Kestrel Reach"));
    assert_eq!(merged.subsectors.len(), 3);
    assert_eq!(merged.worlds.len(), 12);

    let back = parse_sector_xml(&write_sector_xml(&meta));
    assert_eq!(
        (back.name, back.abbreviation, back.subsectors),
        (meta.name, meta.abbreviation, meta.subsectors)
    );
}

#[test]
fn detect_picks_each_format() {
    assert_eq!(detect(TAB), Format::T5Tab);
    assert_eq!(detect(SEC), Format::Sec);
    assert_eq!(detect(LINES), Format::UwpLines);
    assert_eq!(detect(MESSY), Format::UwpLines);
    assert_eq!(detect(XML), Format::Xml);
    let full = parse_t5_tab(TAB).with_metadata(parse_sector_xml(XML));
    for f in [Format::T5Tab, Format::Sec, Format::UwpLines, Format::Xml] {
        assert_eq!(detect(&write(f, &full)), f, "{f:?}");
    }
    assert_eq!(cores(&parse_any(SEC)), cores(&parse_any(TAB)));
}

#[test]
fn writers_round_trip_the_reference_subsector() {
    let full = parse_t5_tab(TAB).with_metadata(parse_sector_xml(XML));
    for f in [Format::T5Tab, Format::Sec, Format::UwpLines] {
        let text = write(f, &full);
        let back = parse_as(f, &text);
        assert!(
            back.warnings.is_empty(),
            "{f:?}: {:?}\n{text}",
            back.warnings
        );
        assert_eq!(fields(&back), fields(&full), "{f:?}\n{text}");
        // Writing what was read gives the same text again.
        assert_eq!(
            write(f, &back.clone().with_metadata(full.clone())),
            text,
            "{f:?}"
        );
    }
    let sec = write_sec(&full);
    let back = parse_sec(&sec);
    assert_eq!(
        (back.name, back.abbreviation, back.subsectors),
        (
            full.name.clone(),
            full.abbreviation.clone(),
            full.subsectors.clone()
        )
    );
}

#[test]
fn subsector_filter() {
    let f = parse_t5_tab(TAB).with_metadata(parse_sector_xml(XML));
    let s = f.subsector('f');
    assert_eq!(s.worlds.len(), 12);
    assert_eq!(s.subsectors.len(), 1);
    assert_eq!(s.subsectors[&'F'], "Lanternfall");
    let empty = f.subsector('A');
    assert!(empty.worlds.is_empty());
    assert_eq!(empty.subsectors[&'A'], "Corvane");
}

#[test]
fn bad_input_never_panics() {
    let samples = [
        "",
        "\n\n",
        "\u{feff}",
        "#",
        "\t\t\t",
        "UWP\n\t",
        "Hex\tUWP\n0101",
        "Hex\tUWP\n0101\tA867975-9\textra\tcells",
        "A867975-9",
        "- A867975-9 - - - - - -",
        "0101 A867975-9 { ( [",
        "0101 A867975-9 703 Im G2 V {x} (y) [z] q 99999999999 +",
        "ünïcode 0101 A867975-9 Ri 703 Im",
        "--- --- ---\nX 0101 A867975-9",
        "<Sector><Name>",
        "<",
        "x\r\ny A867975-9\r\n",
    ];
    for s in samples {
        for f in [Format::T5Tab, Format::Sec, Format::Xml, Format::UwpLines] {
            let parsed = parse_as(f, s);
            let _ = write(f, &parsed);
        }
        let _ = parse_any(s);
    }
    // Every prefix of every fixture.
    for text in [TAB, SEC, LINES, MESSY, XML] {
        for end in (0..text.len()).filter(|&i| text.is_char_boundary(i)) {
            let _ = parse_any(&text[..end]);
        }
    }
}

/// A small deterministic generator (xorshift64*), so the same worlds come
/// out on every run and every machine.
struct Gen(u64);

impl Gen {
    fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }
    fn below(&mut self, n: usize) -> usize {
        (self.next() % n as u64) as usize
    }
    fn pick<'a>(&mut self, items: &[&'a str]) -> &'a str {
        items[self.below(items.len())]
    }
    fn chance(&mut self, pct: usize) -> bool {
        self.below(100) < pct
    }
}

const EHEX: &str = "0123456789ABCDEFGHJKLMNPQRSTUVWXYZ?";

fn ehex(g: &mut Gen) -> char {
    let c: Vec<char> = EHEX.chars().collect();
    c[g.below(c.len())]
}

fn random_world(g: &mut Gen) -> WorldLine {
    let uwp: String = (0..9).map(|i| if i == 7 { '-' } else { ehex(g) }).collect();
    let syll = [
        "ka", "ren", "dor", "vel", "is", "thra", "mo", "quen", "ash", "lo", "zé",
    ];
    let mut name = String::new();
    for _ in 0..(1 + g.below(3)) {
        name.push_str(g.pick(&syll));
    }
    let mut name: String = name
        .chars()
        .enumerate()
        .map(|(i, c)| if i == 0 { c.to_ascii_uppercase() } else { c })
        .collect();
    if g.chance(20) {
        name.push_str("'s Rest");
    }
    if g.chance(15) {
        name = format!("New {name}");
    }
    let mut bases = Vec::new();
    for b in ["N", "S", "W", "D", "M", "K", "C", "E"] {
        if bases.len() < 3 && g.chance(15) {
            bases.push(b.to_string());
        }
    }
    if g.chance(5) {
        bases = vec!["S".to_string(), "N".to_string()];
    }
    let codes = [
        "Ag",
        "As",
        "Ba",
        "De",
        "Fl",
        "Ga",
        "Hi",
        "Ht",
        "Ic",
        "In",
        "Lo",
        "Lt",
        "Na",
        "Ni",
        "Po",
        "Ri",
        "Va",
        "Wa",
        "Cp",
        "Cs",
        "Cx",
        "Pz",
        "Mr",
        "RsA",
        "Fo",
        "(Droyne)",
        "(Droyne)W",
        "O:1910",
        "Di(Kebkh)",
    ];
    let remarks = (0..g.below(5))
        .map(|_| g.pick(&codes).to_string())
        .collect();
    let zone = [None, None, Some('A'), Some('R')][g.below(4)];
    let pbg: String = (0..3)
        .map(|i| {
            if i == 0 {
                char::from(b'0' + g.below(10) as u8)
            } else {
                ehex(g)
            }
        })
        .collect();
    let allegiance = g
        .pick(&["Kv", "Na", "KvTr", "Cs", "Zz9", "", "--"])
        .to_string();
    let types = ["O", "B", "A", "F", "G", "K", "M"];
    let lum = ["Ia", "Ib", "II", "III", "IV", "V", "VI", "D"];
    let mut stars = Vec::new();
    for _ in 0..g.below(4) {
        stars.push(match g.below(6) {
            0 => g.pick(&["D", "BD", "DA"]).to_string(),
            1 => format!("{}{}{}", g.pick(&types), g.below(10), g.pick(&lum)),
            _ => format!("{}{} {}", g.pick(&types), g.below(10), g.pick(&lum)),
        });
    }
    let opt = |g: &mut Gen, v: String| g.chance(60).then_some(v);
    let ix = format!("{}{}", g.pick(&["+", "-", ""]), g.below(6));
    let ex: String = (0..3).map(|_| ehex(g)).collect::<String>() + g.pick(&["+1", "-2", "+0"]);
    let cx: String = (0..4).map(|_| ehex(g)).collect();
    let nob = g.pick(&["B", "BcC", "Ef", "D"]).to_string();
    let ru = format!("{}", g.below(9000) as i64 - 300);
    let (x, y) = (1 + g.below(32) as u32, 1 + g.below(40) as u32);
    WorldLine {
        hex: xy_to_hex(x, y),
        name,
        uwp,
        bases,
        remarks,
        zone,
        pbg,
        allegiance,
        stars,
        importance: opt(g, ix),
        economic: opt(g, ex),
        cultural: opt(g, cx),
        nobility: opt(g, nob),
        worlds: g.chance(60).then(|| g.below(20) as u32),
        ru: opt(g, ru),
        raw: String::new(),
    }
}

#[test]
fn random_worlds_round_trip_through_every_writer() {
    let mut g = Gen(0x5EC7_0B10_C0DE_F00D);
    for batch in 0..40 {
        let file = SectorFile {
            worlds: (0..50).map(|_| random_world(&mut g)).collect(),
            ..Default::default()
        };
        for f in [Format::T5Tab, Format::Sec, Format::UwpLines] {
            let text = write(f, &file);
            let back = parse_as(f, &text);
            assert!(
                back.warnings.is_empty(),
                "batch {batch} {f:?}: {:?}",
                back.warnings
            );
            assert_eq!(back.worlds.len(), file.worlds.len());
            for (a, b) in back.worlds.iter().zip(&file.worlds) {
                assert!(
                    a.same_fields(b),
                    "batch {batch} {f:?}\n got {a:?}\nwant {b:?}"
                );
            }
        }
    }
}

#[test]
fn every_uwp_digit_survives() {
    // Each ehex digit (and '?') in each UWP position, through every writer.
    let digits: Vec<char> = EHEX.chars().collect();
    let mut worlds = Vec::new();
    for pos in [0, 1, 2, 3, 4, 5, 6, 8] {
        for &d in &digits {
            let mut u: Vec<char> = "A867975-9".chars().collect();
            u[pos] = d;
            worlds.push(WorldLine {
                hex: "0101".into(),
                name: "Probe".into(),
                uwp: u.into_iter().collect(),
                pbg: "100".into(),
                ..Default::default()
            });
        }
    }
    let file = SectorFile {
        worlds,
        ..Default::default()
    };
    for f in [Format::T5Tab, Format::Sec, Format::UwpLines] {
        let back = parse_as(f, &write(f, &file));
        let got: Vec<&str> = back.worlds.iter().map(|w| w.uwp.as_str()).collect();
        let want: Vec<&str> = file.worlds.iter().map(|w| w.uwp.as_str()).collect();
        assert_eq!(got, want, "{f:?}");
    }
}

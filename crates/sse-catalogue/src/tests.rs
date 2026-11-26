use crate::*;
use serde_json::json;

/// A small invented belt: every value is made up for the test and says so.
pub(crate) fn fixture_json() -> serde_json::Value {
    json!({
        "header": {
            "format": "sse.catalogue/1",
            "id": "test-belt",
            "title": "Test belt (invented values)",
            "version": "1",
            "level": "small_body",
            "frame": {"kind": "orbit", "plane": "ecliptic_j2000", "primary": "Sun"},
            "anchor": {"name": "Sun"},
            "epoch": {"kind": "jd", "jd": 2461000.5, "scale": "tdb"},
            "size": {"column": "diameter_km", "quantity": "diameter", "unit": "km"},
            "columns": [
                {"name": "id", "type": "str", "role": "id"},
                {"name": "name", "type": "str", "role": "value", "grade": "convention", "source": "author"},
                {"name": "diameter_km", "type": "f32", "role": "value", "unit": "km", "grade_column": "diameter_grade", "source_column": "diameter_source"},
                {"name": "diameter_grade", "type": "u8", "role": "meta"},
                {"name": "diameter_source", "type": "u8", "role": "meta"},
                {"name": "a_au", "type": "f64", "role": "value", "unit": "au", "grade": "measured", "source": "author"},
                {"name": "e", "type": "f32", "role": "value", "grade": "measured", "source": "author"},
                {"name": "i_deg", "type": "f32", "role": "value", "unit": "deg", "grade": "measured", "source": "author"},
                {"name": "node_deg", "type": "f32", "role": "value", "unit": "deg", "grade": "measured", "source": "author"},
                {"name": "peri_deg", "type": "f32", "role": "value", "unit": "deg", "grade": "measured", "source": "author"},
                {"name": "mean_anomaly_deg", "type": "f32", "role": "value", "unit": "deg", "grade": "measured", "source": "author"},
                {"name": "flags", "type": "u8", "role": "meta"}
            ],
            "sources": [
                {"id": "author", "name": "Test author", "version": "1", "kind": "authored",
                 "licence": "CC0-1.0", "credit": "Invented for the sse-catalogue tests."},
                {"id": "survey", "name": "Test survey", "version": "2", "kind": "downloaded",
                 "licence": "test only", "credit": "Sizes from the Test survey (a stand-in credit for tests)."}
            ],
            "completeness": {"size_edges": [100.0, 10.0], "fraction": [[1.0], [0.5]],
                             "grade": "estimated", "source": "author"},
            "steps": [{"size": 300.0, "label": "big"}, {"size": 30.0, "label": "middling"}]
        },
        "rows": [
            {"id": "t1", "name": "First", "diameter_km": 900.0, "diameter_grade": "measured", "diameter_source": "survey",
             "a_au": 2.77, "e": 0.08, "i_deg": 10.6, "node_deg": 80.3, "peri_deg": 73.6, "mean_anomaly_deg": 5.0, "flags": 1},
            {"id": "t2", "diameter_km": 500.0, "diameter_grade": "estimated", "diameter_source": "author",
             "a_au": 2.36, "e": 0.09, "i_deg": 7.1, "node_deg": 103.8, "peri_deg": 151.2, "mean_anomaly_deg": 20.0},
            {"id": "t3", "diameter_km": 300.0, "diameter_grade": 0, "diameter_source": 1,
             "a_au": 2.5, "e": 0.2, "i_deg": 3.0, "node_deg": 10.0, "peri_deg": 20.0, "mean_anomaly_deg": 30.0},
            {"id": "t4", "diameter_km": 120.0, "diameter_grade": "estimated", "diameter_source": "author",
             "a_au": 3.1, "e": 0.1, "i_deg": 1.0, "node_deg": 1.0, "peri_deg": 2.0},
            {"id": "t5", "diameter_km": 30.0, "diameter_grade": "estimated", "diameter_source": "author",
             "a_au": 2.2, "e": 0.15, "i_deg": 5.0, "node_deg": 50.0, "peri_deg": 60.0, "mean_anomaly_deg": 70.0},
            {"id": "t6", "diameter_km": 12.0, "diameter_grade": "estimated", "diameter_source": "author",
             "a_au": 2.9, "e": 0.05, "i_deg": 2.0, "node_deg": 5.0, "peri_deg": 6.0, "mean_anomaly_deg": 7.0},
            {"id": "t7", "diameter_km": 12.0, "diameter_grade": "estimated", "diameter_source": "author",
             "a_au": 2.8, "e": 0.06, "i_deg": 2.5, "node_deg": 5.5, "peri_deg": 6.5, "mean_anomaly_deg": 7.5},
            {"id": "t8", "diameter_km": 4.0, "diameter_grade": "estimated", "diameter_source": "author",
             "a_au": 2.6, "e": 0.07, "i_deg": 3.5, "node_deg": 8.0, "peri_deg": 9.0, "mean_anomaly_deg": 10.0}
        ]
    })
}

fn fixture() -> Catalogue {
    Catalogue::from_text(&fixture_json().to_string())
        .expect("fixture reads")
        .0
}

/// A source that records every range asked of it and refuses any byte at or
/// past `fence`.
struct Guard<'a> {
    bytes: &'a [u8],
    fence: u64,
    asked: Vec<(u64, u64)>,
}

impl ByteSource for Guard<'_> {
    fn read(&mut self, start: u64, end: u64) -> Result<Vec<u8>, Refusal> {
        self.asked.push((start, end));
        assert!(
            end <= self.fence,
            "read {start}..{end} crosses the fence at {}",
            self.fence
        );
        Ok(self.bytes[start as usize..end as usize].to_vec())
    }
}

/// A big invented list: sizes fall as 10,000 / (k + 1).
fn big(rows: usize) -> Catalogue {
    let mut v = fixture_json();
    v["header"]["steps"] = json!([{"size": 1000.0}, {"size": 100.0}, {"size": 10.0}]);
    let rows: Vec<serde_json::Value> = (0..rows)
        .map(|k| {
            json!({"id": format!("b{k}"), "diameter_km": 10_000.0 / (k as f64 + 1.0),
                   "diameter_grade": "estimated", "diameter_source": "author",
                   "a_au": 2.0 + (k % 100) as f64 * 0.01, "e": 0.1, "i_deg": 1.0,
                   "node_deg": 2.0, "peri_deg": 3.0, "mean_anomaly_deg": (k % 360) as f64})
        })
        .collect();
    v["rows"] = json!(rows);
    Catalogue::from_text(&v.to_string()).expect("big reads").0
}

#[test]
fn text_form_round_trips() {
    let a = fixture();
    let (b, _) = Catalogue::from_text(&a.to_text()).unwrap();
    assert_eq!(a, b);
}

#[test]
fn packed_form_round_trips_and_is_deterministic() {
    let a = fixture();
    let bytes = a.pack(2).unwrap();
    assert_eq!(bytes, a.pack(2).unwrap(), "same catalogue, same bytes");
    let (h, r, p) = read_bytes(&bytes, Cut::All).unwrap();
    assert_eq!(r.table, a.table);
    let mut h2 = h.clone();
    h2.packed = None;
    assert_eq!(h2, a.header);
    let p = p.unwrap();
    assert_eq!(p.index().rows, 8);
    // Steps at 300 and 30 km: three and five rows at or above them.
    assert_eq!(p.index().step_ranks, vec![3, 5]);
}

#[test]
fn a_prefix_read_never_touches_the_rest_of_the_file() {
    let cat = big(10_000);
    let bytes = cat.pack(256).unwrap();
    let p = Packed::open(&bytes).unwrap();
    for cut in [
        Cut::Size(100.0),
        Cut::Size(57.0),
        Cut::Rank(1),
        Cut::Rank(777),
    ] {
        let plan = p.plan(cut);
        // Asked of a source that refuses anything past the plan.
        let mut g = Guard {
            bytes: &bytes,
            fence: plan.end,
            asked: vec![],
        };
        let (_, r) = read_packed(&mut g, cut).unwrap();
        assert_eq!(g.asked.len(), 3, "preamble, header, blocks: {:?}", g.asked);
        assert_eq!(g.asked.iter().map(|a| a.1).max(), Some(plan.end));
        assert!(
            plan.end < bytes.len() as u64 / 2,
            "{cut:?} reads {} of {}",
            plan.end,
            bytes.len()
        );
        // A file truncated at the plan's end reads the same.
        let (_, r2, _) = read_bytes(&bytes[..plan.end as usize], cut).unwrap();
        assert_eq!(r.table, r2.table);
        // Garbage past the plan changes nothing.
        let mut spoiled = bytes.clone();
        for b in &mut spoiled[plan.end as usize..] {
            *b = 0xA5;
        }
        let (_, r3, _) = read_bytes(&spoiled, cut).unwrap();
        assert_eq!(r.table, r3.table);
        // The rows kept are exactly the cut's.
        let want = match cut {
            Cut::Size(s) => (0..cat.table.rows)
                .filter(|&k| cat.table.size(&cat.header, k).unwrap() >= s)
                .count(),
            Cut::Rank(k) => k as usize,
            Cut::All => cat.table.rows,
        };
        assert_eq!(r.table.rows, want, "{cut:?}");
        // At most one block past the cut is fetched.
        assert!(plan.rows_in_blocks - (want as u64) < 256, "{cut:?}");
    }
}

#[test]
fn a_cut_at_a_step_reads_exactly_to_it() {
    let cat = big(10_000);
    let bytes = cat.pack(256).unwrap();
    let p = Packed::open(&bytes).unwrap();
    for (s, &k) in p.header.steps.iter().zip(&p.index().step_ranks) {
        let plan = p.plan(Cut::Size(s.size));
        assert_eq!(plan.rows_in_blocks, k, "step {}", s.size);
        let (_, r) = p
            .read(&mut &bytes[..], Cut::Size(s.size))
            .map(|r| ((), r))
            .unwrap();
        assert_eq!(r.table.rows as u64, k);
    }
    // 10,000 / (k + 1) >= 100 for k < 100.
    assert_eq!(p.index().step_ranks, vec![10, 100, 1000]);
}

#[test]
fn one_piece_reads_on_its_own() {
    let cat = big(2_000);
    let bytes = cat.pack(300).unwrap();
    let p = Packed::open(&bytes[..Packed::header_end(&bytes).unwrap() as usize]).unwrap();
    let (s, e) = p.block_range(3).unwrap();
    let t = p.read_block(3, &bytes[s as usize..e as usize]).unwrap();
    assert_eq!(t.rows as u32, p.index().blocks[3].rows);
    assert_eq!(t.size(&p.header, 0), Some(p.index().blocks[3].size_max));
    // A piece from elsewhere is refused by its hash.
    let (s2, e2) = p.block_range(4).unwrap();
    let wrong = &bytes[s2 as usize..e2 as usize];
    if wrong.len() == (e - s) as usize {
        assert_eq!(p.read_block(3, wrong).unwrap_err().code, "bad_hash");
    }
}

#[test]
fn the_reader_asks_for_more_when_the_header_is_short() {
    let bytes = fixture().pack(4).unwrap();
    let e = Packed::open(&bytes[..5]).unwrap_err();
    assert_eq!((e.code.as_str(), e.need_bytes), ("need_more", Some(12)));
    let end = Packed::header_end(&bytes).unwrap();
    let e = Packed::open(&bytes[..20]).unwrap_err();
    assert_eq!(e.need_bytes, Some(end));
    assert!(Packed::open(&bytes[..end as usize]).is_ok());
}

#[test]
fn no_credit_no_reading() {
    let mut v = fixture_json();
    v["header"]["sources"][1]["credit"] = json!("  ");
    let e = Catalogue::from_text(&v.to_string()).unwrap_err();
    assert_eq!(e.code, "no_credit", "{e}");
    let mut v = fixture_json();
    v["header"]["sources"] = json!([]);
    assert_eq!(
        Catalogue::from_text(&v.to_string()).unwrap_err().code,
        "no_credit"
    );
    // And a packed header without credit is refused at open.
    let mut cat = fixture();
    let bytes = cat.pack(4).unwrap();
    let end = Packed::header_end(&bytes).unwrap() as usize;
    let mut h: Header = serde_json::from_slice(&bytes[12..end]).unwrap();
    h.sources[0].credit.clear();
    cat.header = h;
    let hb = serde_json::to_vec(&cat.header).unwrap();
    let mut forged = MAGIC.to_vec();
    forged.extend((hb.len() as u32).to_le_bytes());
    forged.extend(hb);
    forged.extend(&bytes[end..]);
    assert_eq!(Packed::open(&forged).unwrap_err().code, "no_credit");
}

#[test]
fn rows_out_of_size_order_are_refused_not_sorted() {
    let mut v = fixture_json();
    v["rows"][4]["diameter_km"] = json!(1000.0);
    let e = Catalogue::from_text(&v.to_string()).unwrap_err();
    assert_eq!(e.code, "not_sorted", "{e}");
    // A build tool may sort first.
    let mut c = Catalogue::from_text_unsorted(&v.to_string()).unwrap();
    c.sort_by_size().unwrap();
    assert_eq!(c.table.id(&c.header, 0), Some("t5"));
    assert!(c.check_whole().is_ok());
    // Ties are put in id order, so the sort is the same everywhere.
    let ids: Vec<_> = (0..c.table.rows)
        .map(|r| c.table.id(&c.header, r).unwrap())
        .collect();
    assert_eq!(ids, ["t5", "t1", "t2", "t3", "t4", "t6", "t7", "t8"]);
}

#[test]
fn a_damaged_block_is_refused() {
    let bytes = fixture().pack(2).unwrap();
    let mut bad = bytes.clone();
    let n = bad.len();
    bad[n - 3] ^= 0x40;
    assert_eq!(read_bytes(&bad, Cut::All).unwrap_err().code, "bad_hash");
    // Above the damage the cut still reads.
    assert!(read_bytes(&bad, Cut::Rank(2)).is_ok());
    let mut bad = bytes;
    bad[0] = b'X';
    assert_eq!(
        read_bytes(&bad, Cut::All).unwrap_err().code,
        "unknown_format"
    );
}

#[test]
fn every_value_says_where_it_came_from() {
    let c = fixture();
    let h = &c.header;
    let (dc, _) = h.column("diameter_km").unwrap();
    let (ac, _) = h.column("a_au").unwrap();
    let p0 = c.table.provenance(h, 0, dc).unwrap();
    assert_eq!(
        (p0.grade, h.sources[p0.source].id.as_str()),
        (Grade::Measured, "survey")
    );
    let p1 = c.table.provenance(h, 1, dc).unwrap();
    assert_eq!(
        (p1.grade, h.sources[p1.source].id.as_str()),
        (Grade::Estimated, "author")
    );
    assert_eq!(c.table.provenance(h, 1, ac).unwrap().grade, Grade::Measured);
    let row = c.table.row_json(h, 0);
    assert_eq!(
        row["_from"]["diameter_km"],
        json!({"grade": "measured", "source": "survey"})
    );
    assert!(row.get("name").is_some() && c.table.row_json(h, 1).get("name").is_none());
    // A value column with no source is refused.
    let mut v = fixture_json();
    v["header"]["columns"][5]["source"] = serde_json::Value::Null;
    assert_eq!(
        Catalogue::from_text(&v.to_string()).unwrap_err().code,
        "bad_column"
    );
    // A present value whose per-row source is absent has no provenance.
    let mut v = fixture_json();
    v["rows"][2]["diameter_source"] = serde_json::Value::Null;
    assert_eq!(
        Catalogue::from_text(&v.to_string()).unwrap_err().code,
        "no_provenance"
    );
}

#[test]
fn never_invent_unplaced_rows_are_kept_and_listed() {
    let (c, w) = Catalogue::from_text(&fixture_json().to_string()).unwrap();
    assert_eq!(c.table.rows, 8);
    assert_eq!(w.len(), 1, "{w:?}");
    assert_eq!(
        (w[0].id.as_str(), w[0].column.as_str()),
        ("t4", "mean_anomaly_deg")
    );
    let (mc, _) = c.header.column("mean_anomaly_deg").unwrap();
    assert!(
        c.table.columns[mc].number(3).is_none(),
        "absent stays absent"
    );
}

#[test]
fn undeclared_keys_and_bad_values_are_refused() {
    let mut v = fixture_json();
    v["rows"][0]["albedo"] = json!(0.1);
    assert_eq!(
        Catalogue::from_text(&v.to_string()).unwrap_err().code,
        "unknown_column"
    );
    let mut v = fixture_json();
    v["rows"][0]["flags"] = json!(300);
    assert_eq!(
        Catalogue::from_text(&v.to_string()).unwrap_err().code,
        "bad_value"
    );
    let mut v = fixture_json();
    v["rows"][0]["diameter_grade"] = json!("guessed");
    assert_eq!(
        Catalogue::from_text(&v.to_string()).unwrap_err().code,
        "bad_value"
    );
    let mut v = fixture_json();
    v["rows"][1]["id"] = json!("t1");
    assert_eq!(
        Catalogue::from_text(&v.to_string()).unwrap_err().code,
        "duplicate_id"
    );
    let mut v = fixture_json();
    v["header"]["format"] = json!("sse.catalogue/2");
    assert_eq!(
        Catalogue::from_text(&v.to_string()).unwrap_err().code,
        "unknown_format"
    );
    let mut v = fixture_json();
    v["header"]["frame"] = json!({"kind": "icrs"});
    assert_eq!(
        Catalogue::from_text(&v.to_string()).unwrap_err().code,
        "missing_column"
    );
}

#[test]
fn downloaded_data_may_not_carry_a_price() {
    let c = fixture();
    let why = c.header.may_carry_price().unwrap_err();
    assert!(
        why.contains("Test survey") && why.contains("cannot be sold"),
        "{why}"
    );
    let mut v = fixture_json();
    v["header"]["sources"][1]["kind"] = json!("authored");
    let (c, _) = Catalogue::from_text(&v.to_string()).unwrap();
    assert!(c.header.may_carry_price().is_ok());
}

#[test]
fn completeness_reports_the_stated_band_only() {
    let c = fixture();
    let k = &c.header.completeness;
    assert_eq!(k.stated(500.0, None), Some(1.0));
    assert_eq!(k.stated(100.0, None), Some(1.0));
    assert_eq!(k.stated(50.0, None), Some(0.5));
    assert_eq!(
        k.stated(5.0, None),
        None,
        "below the last edge nothing is stated"
    );
}

#[test]
fn the_motion_slot_is_checked() {
    let mut v = fixture_json();
    v["header"]["columns"]
        .as_array_mut()
        .unwrap()
        .push(json!({"name": "motion", "type": "u16", "role": "meta"}));
    v["header"]["motions"] = json!([{"id": "series", "model": "test_series", "table": "test.table",
        "span_jd": [2400000.5, 2500000.5], "scale": "tdb", "source": "author"}]);
    v["rows"][0]["motion"] = json!(0);
    assert!(Catalogue::from_text(&v.to_string()).is_ok());
    v["rows"][0]["motion"] = json!(1);
    assert_eq!(
        Catalogue::from_text(&v.to_string()).unwrap_err().code,
        "bad_motion"
    );
}

#[test]
fn base64_round_trips() {
    for n in 0..20usize {
        let b: Vec<u8> = (0..n as u32).map(|i| (i * 37 + 11) as u8).collect();
        assert_eq!(b64::decode(&b64::encode(&b)).unwrap(), b);
    }
    assert_eq!(b64::encode(b"Man"), "TWFu");
    assert_eq!(b64::decode("TWE=").unwrap(), b"Ma");
    assert!(b64::decode("TW!u").is_err());
}

#[test]
fn the_preview_shows_credits_steps_and_the_cut() {
    let bytes = fixture().pack(2).unwrap();
    let (h, r, p) = read_bytes(&bytes, Cut::Size(100.0)).unwrap();
    let v = preview(&h, &r, p.as_ref(), 2);
    assert_eq!(
        v["credits"][1]["credit"],
        "Sizes from the Test survey (a stand-in credit for tests)."
    );
    assert_eq!(v["price"]["may_sell"], false);
    assert_eq!(v["read"]["rows"], 4);
    assert_eq!(v["rows"].as_array().unwrap().len(), 2);
    assert_eq!(v["steps"][0]["rows"], 3);
    assert_eq!(v["total_rows"], 8);
}

/// The shared fixture file (the kernel's `catalogues` contract area reads
/// it): the same invented belt. Re-write with `SSE_BLESS=1`.
#[test]
fn the_fixture_file_is_current() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let path = root.join("tests/fixtures/catalogues/test-belt.catalogue.json");
    let want = fixture().to_text() + "\n";
    if std::env::var("SSE_BLESS").is_ok() {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, &want).unwrap();
    }
    let got = std::fs::read_to_string(&path).expect("fixture file; bless with SSE_BLESS=1");
    assert_eq!(got.replace("\r\n", "\n"), want, "re-bless with SSE_BLESS=1");
}

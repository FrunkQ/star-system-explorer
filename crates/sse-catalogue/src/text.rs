//! The text form: `{"header": {...}, "rows": [{...}, ...]}`, for authored
//! lists, tests and the output of build scripts. Small, readable, and the
//! same header as the packed form.
//!
//! In a row a key is a declared column; a missing key or `null` is absent.
//! A grade column may hold the grade's name ("measured") and a source
//! column the source's id, as well as their codes.

use crate::header::{ColType, Grade, Header};
use crate::table::{Table, Values, NULL_I32, NULL_U16, NULL_U32, NULL_U8};
use crate::{Catalogue, Refusal};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value as Json};
use std::collections::BTreeSet;

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct TextForm {
    header: Header,
    rows: Vec<Map<String, Json>>,
}

/// Parse the text form (header checked, rows taken as written).
pub(crate) fn parse(text: &str) -> Result<(Header, Table), Refusal> {
    let tf: TextForm = serde_json::from_str(text)
        .map_err(|e| Refusal::new("bad_text", format!("not a catalogue in text form: {e}")))?;
    let h = tf.header;
    if h.packed.is_some() {
        return Err(Refusal::new(
            "bad_text",
            "the text form carries no packed index".into(),
        ));
    }
    h.check()?;
    let grade_cols: BTreeSet<&str> = h
        .columns
        .iter()
        .filter_map(|c| c.grade_column.as_deref())
        .collect();
    let source_cols: BTreeSet<&str> = h
        .columns
        .iter()
        .filter_map(|c| c.source_column.as_deref())
        .collect();
    let mut t = Table::empty(&h);
    for (r, row) in tf.rows.iter().enumerate() {
        for k in row.keys() {
            if h.column(k).is_none() {
                return Err(Refusal::new(
                    "unknown_column",
                    format!("row {r} has {k:?}, which the header does not declare"),
                ));
            }
        }
        for (ci, c) in h.columns.iter().enumerate() {
            let v = row.get(&c.name).filter(|v| !v.is_null());
            let what = |want: &str| {
                Refusal::new(
                    "bad_value",
                    format!("row {r}, {}: {:?} is not {want}", c.name, v),
                )
            };
            let int = |lo: i64, hi: i64| -> Result<Option<i64>, Refusal> {
                match v {
                    None => Ok(None),
                    Some(Json::String(s)) if grade_cols.contains(c.name.as_str()) => Grade::ALL
                        .iter()
                        .find(|g| {
                            serde_json::to_value(g).ok().as_ref().and_then(Json::as_str) == Some(s)
                        })
                        .map(|g| Some(g.code() as i64))
                        .ok_or_else(|| what("a grade")),
                    Some(Json::String(s)) if source_cols.contains(c.name.as_str()) => h
                        .source(s)
                        .map(|(i, _)| Some(i as i64))
                        .ok_or_else(|| what("a listed source id")),
                    Some(x) => match x.as_i64() {
                        Some(i) if (lo..hi).contains(&i) => Ok(Some(i)),
                        _ => Err(what("an integer in range")),
                    },
                }
            };
            match (&mut t.columns[ci], c.ty) {
                (Values::F64(col), ColType::F64) => col.push(match v {
                    None => f64::NAN,
                    Some(x) => x
                        .as_f64()
                        .filter(|f| f.is_finite())
                        .ok_or_else(|| what("a finite number"))?,
                }),
                (Values::F32(col), ColType::F32) => col.push(match v {
                    None => f32::NAN,
                    Some(x) => {
                        let f = x
                            .as_f64()
                            .filter(|f| f.is_finite())
                            .ok_or_else(|| what("a finite number"))?;
                        let g = f as f32;
                        if !g.is_finite() {
                            return Err(what("within f32 range"));
                        }
                        g
                    }
                }),
                (Values::I32(col), ColType::I32) => col.push(
                    int(NULL_I32 as i64 + 1, i32::MAX as i64 + 1)?.map_or(NULL_I32, |i| i as i32),
                ),
                (Values::U32(col), ColType::U32) => {
                    col.push(int(0, NULL_U32 as i64)?.map_or(NULL_U32, |i| i as u32))
                }
                (Values::U16(col), ColType::U16) => {
                    col.push(int(0, NULL_U16 as i64)?.map_or(NULL_U16, |i| i as u16))
                }
                (Values::U8(col), ColType::U8) => {
                    col.push(int(0, NULL_U8 as i64)?.map_or(NULL_U8, |i| i as u8))
                }
                (Values::Str(col), ColType::Str) => col.push(match v {
                    None => None,
                    Some(Json::String(s)) if s.len() < u16::MAX as usize => Some(s.clone()),
                    Some(_) => return Err(what("a string shorter than 65535 bytes")),
                }),
                _ => unreachable!("Table::empty follows the header"),
            }
        }
        t.rows += 1;
    }
    Ok((h, t))
}

/// Write the text form, rows as stored, present values only.
pub(crate) fn write(cat: &Catalogue) -> String {
    let h = &cat.header;
    let mut header = h.clone();
    header.packed = None;
    let grade_cols: BTreeSet<&str> = h
        .columns
        .iter()
        .filter_map(|c| c.grade_column.as_deref())
        .collect();
    let source_cols: BTreeSet<&str> = h
        .columns
        .iter()
        .filter_map(|c| c.source_column.as_deref())
        .collect();
    let t = &cat.table;
    let rows = (0..t.rows)
        .map(|r| {
            let mut o = Map::new();
            for (ci, c) in h.columns.iter().enumerate() {
                let v = &t.columns[ci];
                if !v.is_present(r) {
                    continue;
                }
                let j = if let Some(s) = v.text(r) {
                    Json::String(s.to_string())
                } else if grade_cols.contains(c.name.as_str()) {
                    match v.code(r).and_then(Grade::from_code) {
                        Some(g) => serde_json::to_value(g).expect("grade"),
                        None => Json::from(v.code(r).unwrap_or(NULL_U8)),
                    }
                } else if source_cols.contains(c.name.as_str()) {
                    let i = v.code(r).unwrap_or(NULL_U8) as usize;
                    match h.sources.get(i) {
                        Some(s) => Json::String(s.id.clone()),
                        None => Json::from(i),
                    }
                } else if c.ty.is_float() {
                    Json::from(v.number(r).expect("present"))
                } else {
                    Json::from(v.number(r).expect("present") as i64)
                };
                o.insert(c.name.clone(), j);
            }
            o
        })
        .collect();
    serde_json::to_string_pretty(&TextForm { header, rows }).expect("text form serialises")
}

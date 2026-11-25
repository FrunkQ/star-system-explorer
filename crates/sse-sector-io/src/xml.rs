//! Sector metadata XML: `<Sector Abbreviation="..."><Name>..</Name>
//! <Subsectors><Subsector Index="A">..</Subsector></Subsectors>
//! <Routes><Route Start="0101" End="0203" .../></Routes></Sector>`.
//!
//! A small reader for just what that needs: elements, attributes, text,
//! entities, comments, CDATA and processing instructions. It never fails;
//! anything malformed becomes a warning and the reader carries on.

use crate::token::strip_bom;
use crate::{subsector_letter, Route, SectorFile};

#[derive(Debug, Default)]
struct Element {
    name: String,
    attrs: Vec<(String, String)>,
    children: Vec<Node>,
}

#[derive(Debug)]
enum Node {
    Elem(Element),
    Text(String),
}

impl Element {
    fn attr(&self, name: &str) -> Option<&str> {
        self.attrs
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case(name))
            .map(|(_, v)| v.as_str())
    }

    fn elems(&self) -> impl Iterator<Item = &Element> {
        self.children.iter().filter_map(|n| match n {
            Node::Elem(e) => Some(e),
            Node::Text(_) => None,
        })
    }

    fn child(&self, name: &str) -> Option<&Element> {
        self.elems().find(|e| e.name.eq_ignore_ascii_case(name))
    }

    fn text(&self) -> String {
        let mut s = String::new();
        for n in &self.children {
            if let Node::Text(t) = n {
                s.push_str(t);
            }
        }
        crate::token::collapse(&s)
    }
}

fn line_of(text: &str, pos: usize) -> usize {
    text[..pos].bytes().filter(|&b| b == b'\n').count() + 1
}

fn raw_line(text: &str, pos: usize) -> &str {
    let start = text[..pos].rfind('\n').map_or(0, |i| i + 1);
    let end = text[pos..].find('\n').map_or(text.len(), |i| pos + i);
    text[start..end].trim_end_matches('\r')
}

fn decode_entities(s: &str) -> String {
    let mut out = String::new();
    let mut rest = s;
    while let Some(amp) = rest.find('&') {
        out.push_str(&rest[..amp]);
        rest = &rest[amp..];
        let Some(semi) = rest.find(';').filter(|&i| i <= 12) else {
            out.push('&');
            rest = &rest[1..];
            continue;
        };
        let ent = &rest[1..semi];
        let ch = match ent {
            "amp" => Some('&'),
            "lt" => Some('<'),
            "gt" => Some('>'),
            "quot" => Some('"'),
            "apos" => Some('\''),
            _ => {
                let num = if let Some(h) = ent.strip_prefix("#x").or_else(|| ent.strip_prefix("#X"))
                {
                    u32::from_str_radix(h, 16).ok()
                } else {
                    ent.strip_prefix('#').and_then(|d| d.parse::<u32>().ok())
                };
                num.and_then(char::from_u32)
            }
        };
        match ch {
            Some(c) => {
                out.push(c);
                rest = &rest[semi + 1..];
            }
            None => {
                out.push('&');
                rest = &rest[1..];
            }
        }
    }
    out.push_str(rest);
    out
}

/// Finds the '>' that closes a tag, skipping quoted attribute values.
fn tag_end(s: &str) -> Option<usize> {
    let mut quote: Option<char> = None;
    for (i, c) in s.char_indices() {
        match quote {
            Some(q) if c == q => quote = None,
            Some(_) => {}
            None if c == '"' || c == '\'' => quote = Some(c),
            None if c == '>' => return Some(i),
            None => {}
        }
    }
    None
}

fn parse_tag(inner: &str) -> Element {
    let inner = inner.trim();
    let name_end = inner
        .find(|c: char| c.is_whitespace())
        .unwrap_or(inner.len());
    let mut e = Element {
        name: inner[..name_end].to_string(),
        ..Default::default()
    };
    let mut rest = inner[name_end..].trim_start();
    while !rest.is_empty() {
        let key_end = rest
            .find(|c: char| c == '=' || c.is_whitespace())
            .unwrap_or(rest.len());
        let key = rest[..key_end].to_string();
        rest = rest[key_end..].trim_start();
        let Some(after_eq) = rest.strip_prefix('=') else {
            if !key.is_empty() {
                e.attrs.push((key, String::new()));
            }
            continue;
        };
        rest = after_eq.trim_start();
        let value;
        if let Some(q) = rest.chars().next().filter(|c| *c == '"' || *c == '\'') {
            let body = &rest[1..];
            let end = body.find(q).unwrap_or(body.len());
            value = decode_entities(&body[..end]);
            rest = body.get(end + 1..).unwrap_or("").trim_start();
        } else {
            let end = rest.find(char::is_whitespace).unwrap_or(rest.len());
            value = decode_entities(&rest[..end]);
            rest = rest[end..].trim_start();
        }
        e.attrs.push((key, value));
    }
    e
}

/// Parses the whole document into its top-level elements.
fn parse_tree(text: &str, out: &mut SectorFile) -> Vec<Element> {
    let mut roots = Vec::new();
    let mut stack: Vec<Element> = Vec::new();
    let mut pos = 0;
    let close = |stack: &mut Vec<Element>, roots: &mut Vec<Element>| {
        if let Some(e) = stack.pop() {
            match stack.last_mut() {
                Some(parent) => parent.children.push(Node::Elem(e)),
                None => roots.push(e),
            }
        }
    };
    while pos < text.len() {
        let Some(lt) = text[pos..].find('<').map(|i| pos + i) else {
            if let Some(top) = stack.last_mut() {
                top.children.push(Node::Text(decode_entities(&text[pos..])));
            }
            break;
        };
        if lt > pos {
            if let Some(top) = stack.last_mut() {
                top.children
                    .push(Node::Text(decode_entities(&text[pos..lt])));
            }
        }
        let rest = &text[lt..];
        let skip_to = |marker: &str| rest.find(marker).map(|i| lt + i + marker.len());
        if rest.starts_with("<!--") {
            match skip_to("-->") {
                Some(p) => pos = p,
                None => {
                    out.warn(
                        line_of(text, lt),
                        "comment never closed",
                        raw_line(text, lt),
                    );
                    break;
                }
            }
        } else if let Some(body) = rest.strip_prefix("<![CDATA[") {
            match body.find("]]>") {
                Some(i) => {
                    if let Some(top) = stack.last_mut() {
                        top.children.push(Node::Text(body[..i].to_string()));
                    }
                    pos = lt + 9 + i + 3;
                }
                None => {
                    out.warn(line_of(text, lt), "CDATA never closed", raw_line(text, lt));
                    break;
                }
            }
        } else if rest.starts_with("<?") || rest.starts_with("<!") {
            let marker = if rest.starts_with("<?") { "?>" } else { ">" };
            match skip_to(marker) {
                Some(p) => pos = p,
                None => break,
            }
        } else {
            let Some(end) = tag_end(&rest[1..]).map(|i| lt + 1 + i) else {
                out.warn(line_of(text, lt), "tag never closed", raw_line(text, lt));
                break;
            };
            let inner = &text[lt + 1..end];
            pos = end + 1;
            if let Some(name) = inner.strip_prefix('/') {
                let name = name.trim();
                match stack
                    .iter()
                    .rposition(|e| e.name.eq_ignore_ascii_case(name))
                {
                    Some(i) => {
                        if i + 1 < stack.len() {
                            out.warn(
                                line_of(text, lt),
                                format!("</{name}> closes elements left open"),
                                raw_line(text, lt),
                            );
                        }
                        while stack.len() > i {
                            close(&mut stack, &mut roots);
                        }
                    }
                    None => out.warn(
                        line_of(text, lt),
                        format!("</{name}> has no matching open tag; ignored"),
                        raw_line(text, lt),
                    ),
                }
            } else if let Some(body) = inner.strip_suffix('/') {
                stack.push(parse_tag(body));
                close(&mut stack, &mut roots);
            } else {
                stack.push(parse_tag(inner));
            }
        }
    }
    if !stack.is_empty() {
        out.warn(
            line_of(text, text.len()),
            "document ends with elements still open",
            "",
        );
        while !stack.is_empty() {
            close(&mut stack, &mut roots);
        }
    }
    roots
}

fn find_sector(e: &Element) -> Option<&Element> {
    if e.name.eq_ignore_ascii_case("sector") {
        return Some(e);
    }
    e.elems().find_map(find_sector)
}

/// Sector metadata only: name, abbreviation (attribute or element) and the
/// subsector names. Worlds are not part of this file.
pub fn parse_sector_xml(text: &str) -> SectorFile {
    let mut out = SectorFile::default();
    let text = strip_bom(text);
    let roots = parse_tree(text, &mut out);
    let Some(sector) = roots.iter().find_map(find_sector) else {
        out.warn(
            1,
            "no <Sector> element found",
            text.lines().next().unwrap_or(""),
        );
        return out;
    };
    let names: Vec<&Element> = sector
        .elems()
        .filter(|e| e.name.eq_ignore_ascii_case("name"))
        .collect();
    let name = names
        .iter()
        .find(|e| e.attr("Lang").is_none())
        .or(names.first())
        .map(|e| e.text())
        .filter(|s| !s.is_empty());
    out.name = name;
    out.abbreviation = sector
        .attr("Abbreviation")
        .map(crate::token::collapse)
        .or_else(|| sector.child("Abbreviation").map(Element::text))
        .filter(|s| !s.is_empty());
    if let Some(subs) = sector.child("Subsectors") {
        for s in subs
            .elems()
            .filter(|e| e.name.eq_ignore_ascii_case("subsector"))
        {
            let index = s.attr("Index").unwrap_or("");
            match subsector_letter(index) {
                Some(c) => {
                    out.subsectors.insert(c, s.text());
                }
                None => out.warn(
                    1,
                    format!("subsector index '{index}' is not A to P; ignored"),
                    "",
                ),
            }
        }
    }
    if let Some(routes) = sector.child("Routes") {
        for r in routes
            .elems()
            .filter(|e| e.name.eq_ignore_ascii_case("route"))
        {
            let hex = |k: &str| r.attr(k).map(crate::token::collapse).unwrap_or_default();
            let (start, end) = (hex("Start"), hex("End"));
            if !crate::token::is_hex(&start) || !crate::token::is_hex(&end) {
                out.warn(
                    1,
                    format!("route {start:?} to {end:?}: each end is a four-digit hex; ignored"),
                    "",
                );
                continue;
            }
            let offset = |k: &str, out: &mut SectorFile| -> i32 {
                match r.attr(k).map(str::trim) {
                    None | Some("") => 0,
                    Some(v) => v.parse().unwrap_or_else(|_| {
                        out.warn(
                            1,
                            format!(
                                "route {start} to {end}: {k} {v:?} is not a whole number; 0 used"
                            ),
                            "",
                        );
                        0
                    }),
                }
            };
            let start_offset = (
                offset("StartOffsetX", &mut out),
                offset("StartOffsetY", &mut out),
            );
            let end_offset = (
                offset("EndOffsetX", &mut out),
                offset("EndOffsetY", &mut out),
            );
            let word = |k: &str| {
                r.attr(k)
                    .map(crate::token::collapse)
                    .filter(|s| !s.is_empty())
            };
            out.routes.push(Route {
                start,
                end,
                start_offset,
                end_offset,
                kind: word("Type"),
                allegiance: word("Allegiance"),
                style: word("Style"),
                colour: word("Color"),
            });
        }
    }
    out
}

fn escape(s: &str) -> String {
    let mut out = String::new();
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&apos;"),
            c => out.push(c),
        }
    }
    out
}

/// The metadata file: name, abbreviation and subsector names. Worlds are not
/// written (they live in the data file next to it).
pub fn write_sector_xml(file: &SectorFile) -> String {
    let mut out = String::from("<?xml version=\"1.0\" encoding=\"utf-8\"?>\n");
    match &file.abbreviation {
        Some(a) => out.push_str(&format!("<Sector Abbreviation=\"{}\">\n", escape(a))),
        None => out.push_str("<Sector>\n"),
    }
    if let Some(n) = &file.name {
        out.push_str(&format!("  <Name>{}</Name>\n", escape(n)));
    }
    if !file.subsectors.is_empty() {
        out.push_str("  <Subsectors>\n");
        for (k, v) in &file.subsectors {
            out.push_str(&format!(
                "    <Subsector Index=\"{k}\">{}</Subsector>\n",
                escape(v)
            ));
        }
        out.push_str("  </Subsectors>\n");
    }
    if !file.routes.is_empty() {
        out.push_str("  <Routes>\n");
        for r in &file.routes {
            out.push_str(&format!(
                "    <Route Start=\"{}\" End=\"{}\"",
                r.start, r.end
            ));
            for (k, v) in [
                ("StartOffsetX", r.start_offset.0),
                ("StartOffsetY", r.start_offset.1),
                ("EndOffsetX", r.end_offset.0),
                ("EndOffsetY", r.end_offset.1),
            ] {
                if v != 0 {
                    out.push_str(&format!(" {k}=\"{v}\""));
                }
            }
            for (k, v) in [
                ("Type", &r.kind),
                ("Allegiance", &r.allegiance),
                ("Style", &r.style),
                ("Color", &r.colour),
            ] {
                if let Some(v) = v {
                    out.push_str(&format!(" {k}=\"{}\"", escape(v)));
                }
            }
            out.push_str("/>\n");
        }
        out.push_str("  </Routes>\n");
    }
    out.push_str("</Sector>\n");
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_attributes_entities_and_comments() {
        let text = r#"<?xml version="1.0"?>
<!-- metadata -->
<Sector Tags="x" Abbreviation='Fsh'>
  <Name Lang="xx">Other</Name>
  <Name>Far &amp; Shelf</Name>
  <Subsectors>
    <Subsector Index="a">Low &#x52;oad</Subsector>
    <Subsector Index="Z">Bad</Subsector>
    <Subsector Index="B"><![CDATA[Cee <Dee>]]></Subsector>
  </Subsectors>
  <Empty/>
</Sector>"#;
        let f = parse_sector_xml(text);
        assert_eq!(f.name.as_deref(), Some("Far & Shelf"));
        assert_eq!(f.abbreviation.as_deref(), Some("Fsh"));
        assert_eq!(f.subsectors[&'A'], "Low Road");
        assert_eq!(f.subsectors[&'B'], "Cee <Dee>");
        assert_eq!(f.warnings.len(), 1);
    }

    #[test]
    fn abbreviation_element_and_broken_input() {
        let f = parse_sector_xml("<Sector><Abbreviation>Ab</Abbreviation><Name>N</Name>");
        assert_eq!(f.abbreviation.as_deref(), Some("Ab"));
        assert_eq!(f.name.as_deref(), Some("N"));
        assert!(!f.warnings.is_empty());
        for junk in [
            "<",
            "<Sector",
            "<!-- ",
            "<![CDATA[",
            "</x>",
            "<a b='>",
            "&#xFFFFFFFF;",
            "<Sector>&",
        ] {
            let _ = parse_sector_xml(junk);
        }
    }
}

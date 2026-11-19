# X3c (parsing half): sector data formats

Status: built (19 Nov 2026), new crate `crates/sse-sector-io`. Build plan row X3c, the importer and exporter part only. The hex map view, undoable import and the Travellermap fetch are not in this package.

## What was built
- Data model: `WorldLine` (hex, name, UWP, bases, remarks, zone, PBG, allegiance, stars, the T5 extensions `{Ix} (Ex) [Cx]`, nobility, W, RU, and the original line in `raw`), `SectorFile` (name, abbreviation, subsector names A to P, worlds, warnings) and `Warning` (line, text, raw). All serde, with BTreeMap only.
- Parsers: `parse_uwp_lines`, `parse_t5_tab`, `parse_sec` and `parse_sector_xml` (metadata only). Also `detect`, `parse_any`, `parse_as`.
- Writers: `write_t5_tab`, `write_sec` (aligned columns under a dash ruler), `write_uwp_lines` and `write_sector_xml`. The generic `write(format, file)` picks one.
- Merging and helpers: `SectorFile::with_metadata`, `SectorFile::subsector`, `hex_to_xy`, `xy_to_hex` and `subsector_of`.
- The rule is "never invent". Anything a parser cannot place becomes a warning. A line is kept whenever a UWP was found in it. Nothing panics on bad input.
- Dependencies: serde only (serde_json for tests). No new crates. The XML reader is hand-written.

## Formats and their quirks
- **UWP lines** (loose pastes). A line needs only a UWP: 7 ehex digits, '-', 1 ehex digit, with '?' allowed.
  - Before the UWP come the name and an optional hex, which may be first or last ("0101 Name" or "Name 0101").
  - After it, in this order: bases (capital letters), remarks, zone, PBG, allegiance (read only after a PBG), stars, then the T5 extensions.
  - A lone A, R or G right after the UWP is a zone when the PBG (or the end of the line) follows. Otherwise it is a base, and a single A or B there is the legacy pair (N+S, N+W).
  - Unknown tokens are warned about and dropped. Lines without a UWP are warned about and skipped.
  - The writer writes "-" for an empty allegiance when something follows it. It cannot write an allegiance for a world with no PBG.
- **T5 tab** (Travellermap `/data/<sector>/<subsector>/tab`). The header row names the columns, in any order, and any column may be missing. Case and a few aliases are accepted.
  - Bases are one T5 letter each, with no legacy expansion.
  - `{ +1 }`, `(A46+2)` and `[1716]` are read with or without their delimiters.
  - An empty cell or "-" reads as none.
  - A file whose rows all share one Sector value gets that value as its abbreviation. The writer writes the abbreviation as Sector and works SS out from the hex.
- **SEC** (legacy fixed columns). Lines are read by token shape: the name is everything before the hex, the hex is the four digits just before the UWP, and the PBG is the first three-character PBG-shaped token after it.
  - A lone letter between the UWP and the PBG is placed by the dash ruler when the line lines up with it. Without a ruler, a letter within 3 columns of the UWP is a base, and a lone letter at the end is a zone.
  - `# Sector:`, `# Abbreviation:` and `# Subsector X:` comments carry the metadata.
  - Header lines that mention "UWP" are skipped quietly.
  - **Extension**: legacy SEC has no T5 extension columns. When worlds have them, `write_sec` appends them after the stars (`{Ix} (Ex) [Cx] nobility W RU`, with `{}`/`()`/`[]`/`-` for empty ones), so the round trip keeps every field. Other tools would read those columns as part of the stars.
- **Sector XML** (metadata). Reads `<Sector Abbreviation=..>` (or an `<Abbreviation>` element), the first `<Name>` without a `Lang` attribute, and `<Subsectors><Subsector Index="A">`. Other elements are ignored.
  - The reader handles entities, CDATA, comments, processing instructions and self-closing tags. Malformed input becomes warnings.
  - The format does not carry worlds, so its round trip covers metadata only.
- **Stars**: a spectral type takes the following luminosity class with it ("G2 V", "M0 III", and the legacy "M8 D" meaning M8 dwarf). Compact "K7V", white dwarfs "D", "DA", and brown dwarfs "BD" are also read.
  - Because of this, a class-less "M8" followed by a white dwarf "D" reads back as one star "M8 D".
- **Remarks** are split on whitespace, except that a bracketed remark like "(Droyne Ancients)" stays one token.
- **Detect**: XML if the text starts with '<'. T5 tab if the first line with content has tabs and a UWP column. SEC if there is a dash ruler, or if there are '#' comments and world lines mostly have the hex just before the UWP. Otherwise UWP lines.

## Checks (`cargo test -p sse-sector-io`, 28 tests)
- Fixtures in `tests/fixtures/sector/` are an invented subsector, "Lanternfall" (subsector F of the invented "Kestrel Reach"), with 12 worlds. It comes as T5 tab, legacy SEC (with legacy A and B base letters), clean UWP lines, sector XML, and a messy paste.
  - The messy paste has odd spacing, tabs, the hex before or after the name, a title line, a column header, an unknown token and a line without a UWP.
- Every world format parses to the same worlds. SEC also reads the same with its ruler removed.
- The messy paste keeps all 12 worlds and gives exactly four warnings, on the right lines.
- `detect` picks the right format for every fixture and for every writer's output.
- Writers round trip the reference subsector, including the T5 extensions, through tab, SEC and UWP lines. Writing what was read gives the same text again. XML metadata also round trips.
- 2,000 deterministic random worlds (xorshift, fixed seed) round trip field for field through all three world writers. They cover random UWPs with '?', bases, remarks, zones, PBGs, allegiances, 0 to 3 stars, and every extension present or absent. Every ehex digit in every UWP position round trips as well.
- Bad input never panics: odd samples, plus every prefix of every fixture, go through every parser.
- Hex and subsector maths: corners (0101 A, 0810 A, 0901 B, 3240 P) and all 1,280 hexes.

## Limits
- Legacy SEC base letters are expanded only where certain: A = N+S, B = N+W, and C, D, E, K, M, N, S, W as themselves. The other letters (F, G, H, J, L, O, P, Q, R, T, U, V, X, Y, Z: Vargr, Aslan, Zhodani, Droyne and K'kree variants in the published table) are kept as written, with a warning. Confirm them against Travellermap's SEC notes before expanding them.
- A world with no allegiance whose first star is "BD" reads "BD" as the allegiance in SEC and UWP lines. The writers avoid this by writing "-".
- A PBG of "???" is accepted as a PBG.
- `SectorFile::subsector` keeps the whole file's warnings.
- No Travellermap fetch, no hex view, no undo. Those belong to the rest of X3c.

# X3c import: sector files into a campaign (19 Nov 2026)
- `sse_codes::import::import(codes, sheet, file, universe)`: every world line goes through the sheet's `import` hints (pack data: which columns are code lines, the profile line, listed codes to compare, and the referee's columns written as facts). Each line becomes a preview with its plan (X3b), system facts (`map.hex`, `map.sector`, `name`) and main-world facts at `<universe>/sys:<hex>/body:@1`. Impossible values are kept and marked with their reason, never altered. Guesses are said plainly (no or unreadable stellar data, giants and white dwarfs grown as dwarfs, '?' digits left to the physics). Listed trade codes that differ from those read back are reported. There is a one-line summary.
- Kernel op `import_sector {text, format?, sheet?, apply, at_s}`: a preview without `apply`; with it each world writes one authored event, plus one authored-impossible event when it has impossible facts. Undo and replay work as for any authored event.
- Checks: Lanternfall imports all 12 worlds and every UWP reads back unchanged. The tab, SEC and UWP-line files give the same facts. Zones and bases read back. The messy paste imports. A '?' line is refused with its reason. An impossible line (X266000-0) is kept and marked. Contract cases cover preview, apply, codes after the import, and a bad format.
- Found on Lanternfall: 6 of 12 worlds list trade codes that differ from the T5 rules read back (for example Ashgrove reads Pr as well). These are the fixture's own choices, and the report shows them.
- Not yet: the Travellermap fetch (needs the site Worker), growing each imported system (call `run_formation` with the plan's pins per system, or batch it), placing systems in 3D (handoff 7A part 3, "small deterministic offset"), and export from a campaign.

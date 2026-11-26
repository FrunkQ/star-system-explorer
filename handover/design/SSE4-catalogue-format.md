# The catalogue format `sse.catalogue/1` (track RW, package RW3)

Status: **built** (RW3, crate `crates/sse-catalogue`, op `read_catalogue`). This is the format RW4 (populations), RW7 (the Solar System catalogue), RW13 (star sources) and RW17 (measured motion) build on. Changes to it are agreed by message with the RW3 owner thread ("Build the catalogue format and reader") and land as a new format string, never as a silent change to `/1`.

Design behind it: `SSE4-real-catalogues-design.md` sections 3.1, 5.3, 5.3A, 6.3, 6A.2, 6A.3 and 7.5 (accepted 26 Nov 2026).

## 1. What it is

A catalogue is a list of measured or authored entries at one level (galaxy, star, world, small body), **stored biggest first and indexed by size**, so a reader takes a prefix and stops. Data below the Detail control's cut is never fetched, parsed or held (Alex, 26 Nov 17:29).

The reader refuses, before reading a row:
- a catalogue with no source, or a source with no credit text, name, version or licence (`no_credit`);
- a value column without a grade and a source (`bad_column`), or naming an unlisted source (`unknown_source`);
- an unknown format string, frame kind or field (`unknown_format`, `bad_header`);
- a frame whose placing columns are not declared (`missing_column`).

And, while reading rows:
- rows that rise in size (`not_sorted`; the reader never re-sorts), a row with no size (`missing_size`), no id or a repeated id (`missing_id`, `duplicate_id`);
- a present value whose per-row grade or source is absent (`no_provenance`), a grade code over 3 (`bad_grade`), a motion index not listed (`bad_motion`);
- a packed block whose SHA-256 differs (`bad_hash`).

**Never invent.** Absent values stay absent. A row missing a placing column (or its epoch, or its primary) is kept and listed as a warning `unplaced: no …`. The reader estimates nothing: sizes from brightness, masses, completeness curves and thinning are block flows (RW3b).

## 2. The header (both forms)

JSON, every field checked, unknown fields refused.

| Field | What |
|---|---|
| `format` | `"sse.catalogue/1"` |
| `id`, `title`, `version` | a lower-case slug, a title, a version string |
| `level` | `galaxy`, `star`, `world` (planet or moon), `small_body` |
| `frame` | `{"kind": "icrs"}` (`ra_deg`, `dec_deg`, `distance_pc`; optional `pm_ra_cosdec_mas_yr`, `pm_dec_mas_yr`, `rv_km_s`) · `{"kind": "galactic"}` (`l_deg`, `b_deg`, `distance_pc`) · `{"kind": "orbit", "plane": "ecliptic_j2000" \| "primary_equator" \| "laplace", "primary": "Sun"}` (`a_au` or `q_au`, `e`, `i_deg`, `node_deg`, `peri_deg`, `mean_anomaly_deg` or `tp_jd`; without `primary`, a `primary` str column) · `{"kind": "cartesian", "unit": "pc"\|"ly"\|"au"\|"km"\|"m", "axes": "words"}` (`x`, `y`, `z`) · `{"kind": "hex_grid", "grid": "traveller_sector", "parsecs_per_hex": 1.0}` (`hex`) |
| `anchor` | `{"name": "Sun", "id": "optional entry id"}`: what everything is placed from. When `id` is given it must be an entry (checked on a whole read) |
| `epoch` | `{"kind": "jd", "jd": 2461000.5, "scale": "tdb"\|"tt"\|"utc"}` · `{"kind": "per_row", "column": "epoch_jd", "scale": "tdb"}` · `{"kind": "universe", "s": 0}` (authored places) |
| `size` | `{"column": "diameter_km", "quantity": "diameter"\|"mass", "unit": …}`; units: mass `kg`, `msun`, `mearth`; diameter `km`, `m`. The column is a float value column |
| `columns` | see 2.1 |
| `sources` | `[{id, name, version, kind: "downloaded"\|"authored", licence, credit, link?, retrieved?}]`, at most 255. `credit` is the provider's text word for word |
| `completeness` | `{size_edges, distance_edges_pc?, fraction, grade, source, note?}`, see 2.2 |
| `steps` | `[{size, label?}]`, strictly falling: the named stops. In the packed form a block always ends at each |
| `motions` | `[{id, model, table, span_jd: [from, to], scale, source}]`: the RW17 slot, see 2.3 |
| `notes` | free text lines |
| `packed` | the packed form's index only (section 4) |

### 2.1 Columns

`{"name", "type": "f64"\|"f32"\|"i32"\|"u32"\|"u16"\|"u8"\|"str", "role": "id"\|"value"\|"meta", "unit"?, "grade"?, "grade_column"?, "source"?, "source_column"?, "doc"?}`

- Names are unique snake_case. Exactly one `id` column, type `str`, role `id`.
- A **value** column has exactly one of `grade` (the whole column) or `grade_column` (a `u8` meta column of codes), and exactly one of `source` (a source id) or `source_column` (a `u8` meta column of indices into `sources`). So every value carries its grade and the catalogue it came from.
- Grades and their codes: `measured` 0, `estimated` 1, `convention` 2, `hypothetical` 3.
- A **meta** column is bookkeeping and has no grade or source. Standard meta columns: `flags` (`u8`: 1 notable, shown whatever the cut; 2 hypothetical entry, off by default; other bits are warned and kept) and `motion` (`u16`, an index into `motions`).
- Absent values: floats NaN; `u8` 255, `u16` 65535, `u32` 4294967295, `i32` -2147483648; strings absent (in the text form, a missing key or `null`).

### 2.2 Completeness

Bands, not a curve. `size_edges` are lower bounds, biggest first: band `i` is sizes `>= size_edges[i]` and below `size_edges[i-1]`. Below the last edge nothing is stated. `distance_edges_pc` are upper bounds, nearest first (empty: one band for any distance). `fraction[i][j]` is in 0 to 1. The reader reports only the stated band (`Completeness::stated`); smoothing between bands is the thinning flow's.

### 2.3 Measured motion (for RW17)

A row may name a motion (`motion` column) that holds inside `span_jd`; outside it the body hands over to the kernel's orbit. The reader checks the slot (id unique, span real, source listed, index in range) and runs nothing. The model's terms live in the named pack `table`.

## 3. The text form

```json
{"header": { ... }, "rows": [{"id": "t1", "diameter_km": 900.0, "diameter_grade": "measured", "diameter_source": "survey", "a_au": 2.77, ...}]}
```

For authored lists, tests and build-script output. A key must be a declared column. A grade column may hold the grade's name, a source column the source's id. Rows must already be biggest first; `sse catalogue pack --sort` sorts (ties by id) for a build script. Example: `tests/fixtures/catalogues/test-belt.catalogue.json` (invented values).

## 4. The packed form (`.ssecat`)

```text
0        8 bytes   magic  "SSECAT", 0x00, 0x01
8        4 bytes   header length H, u32 little-endian
12       H bytes   the header JSON, with "packed": {rows, row_bytes, blocks, step_ranks}
12 + H   blocks, back to back
```

- A **block** is its rows, fixed width and little-endian with columns in header order (f64 8 bytes, f32/i32/u32 4, u16 2, u8 1, str 6 = u32 offset into the block's heap + u16 length; absent string offset 4294967295), then the UTF-8 heap of its strings.
- `blocks[i]` = `{rows, offset (from the end of the header), bytes, size_max, size_min, sha256}`. Offsets are contiguous; sizes never rise from block to block.
- `step_ranks[k]` = rows at or above `steps[k].size`; a block boundary always falls there.
- **Reading a cut** asks for three ranges: bytes 0 to 12, then 12 to 12 + H, then the body from its start to the end of the last block whose `size_max` is at or above the cut (or that holds rank k). A cut at a step reads exactly to it; any other cut reads at most one block past it and decodes no row of it below the cut. Nothing further is asked for (test `a_prefix_read_never_touches_the_rest_of_the_file`, which reads through a source that fails on any byte past the plan).
- **Pieces.** Each block can be fetched and checked on its own by its hash (`Packed::block_range`, `read_block`), so a store may serve a pack by byte range or in pieces. The header's SHA-256 (`header_sha256` in the preview) is the file's identity for a pack manifest.
- Writer default 1,024 rows a block; a 1.5 million row file then has an index of about 250 kB.

## 5. Selling and credits

- `Header::may_carry_price()`: any `downloaded` source means the catalogue (and anything holding its values) may be used and given away but not sold, with the reason in words naming the catalogues (Alex, 26 Nov 18:00). The Explorers shop calls the same rule (RW8).
- The preview carries every source's credit in full, for the first import and the Packs screen (Alex, 26 Nov 18:01).

## 6. Where it is used

| Use | How |
|---|---|
| Kernel op `read_catalogue` | `{text}` or `{packed_b64}`, cut `min_size` or `rank`, `limit` rows (default 20, at most 200). Returns the preview, or `catalogue_<code>` with the reason. Read-only, no universe. Contract area `catalogues` |
| Build scripts (RW7, RW13) | write the text form, then `sse catalogue pack in.json out.ssecat [--sort] [--block-rows N]`; `sse catalogue show <file> [--min-size S \| --rank K]` prints the preview |
| Populations (RW4) | `Table`: flat typed columns biggest first (`Values::F32` and so on), `Table::provenance(row, col)` for any value |
| Shell fetch (RW8) | `Packed::header_end(prefix)` says how many bytes the header needs; `Packed::plan(cut)` gives the byte range; `read_plan` decodes it |

## 7. Not in `/1`, on purpose

- No compression inside the file: a store may serve it gzip-free for byte ranges; whole-file downloads may be compressed in transit.
- No region tiling for stars (HEALPix by distance shell, design 5.4): when RW13 needs it, a tile is one catalogue file per cell with the same header, named in a pack's list; no format change.
- No estimates, joins, claims or thinning: those are RW3b flows and RW4, RW11.

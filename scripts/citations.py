#!/usr/bin/env python3
"""Master list of every paper SSE4 cites, for the reference run-through (track CR).

Gathers author-year citations, DOIs and arXiv ids from the packs (block and formula
sources, the reference sheet's per-value sources, flow tests), the kernel and UI code,
and the design notes, removes duplicates, and keeps one master list that is scored off
as each paper is checked (Alex, 24 Nov 2026: "construct a master list and score off as
we confirm").

    python3 scripts/citations.py --add               add new papers as unchecked, leaving every other row alone (builders)
    python3 scripts/citations.py --update            refresh every row's places and claims and MASTER.md (the run-through thread)
    python3 scripts/citations.py --check             fail if a pack or code citation is not on the list
    python3 scripts/citations.py --batches DIR       write daily Consensus briefs of the next unchecked papers
    python3 scripts/citations.py --score FILE        score a pasted Consensus reply back onto the list
    python3 scripts/citations.py --extra DIR         also read design notes from DIR (e.g. the project files)

The list is handover/citations/master.jsonl (one paper per line, the source of truth;
hand edits to status, checked, note and found are kept by --update). MASTER.md beside
it is written from it. The procedure is handover/citations/README.md.
"""

import argparse
import datetime
import json
import os
import re
import sys
import unicodedata

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
LIST_DIR = os.path.join(ROOT, "handover", "citations")
MASTER = os.path.join(LIST_DIR, "master.jsonl")
MASTER_MD = os.path.join(LIST_DIR, "MASTER.md")

STATUSES = ["unchecked", "confirmed", "wrong citation", "says something else", "not found"]

# Where citations are gathered. "checked" sources fail --check when a paper is missing;
# design notes are listed but only warned about (they arrive by docs-only pull requests,
# which do not run the Rust job, so a failure there would first show on main).
CODE_ROOTS = ["packs", "store-packs", "crates", "packages", "worker"]
NOTE_ROOTS = ["handover/design"]
GOLDEN = "tests/golden"
EARLIER = ["docs/citations.md", "docs/citations"]
SKIP_DIRS = {"node_modules", "target", "dist", ".git", "generated", "screens"}
CODE_EXT = {".rs", ".ts", ".js", ".mjs", ".svelte", ".json", ".py"}

# --- recognising a citation -------------------------------------------------------

UP = "A-ZÀ-ÖØ-ÞĀ-ſ"
LO = "a-zß-öø-ÿĀ-ſ"
# A surname: capitalised, may carry a particle (van, de, von, Mc, O') or a hyphen.
SUR = rf"(?:(?:van der |van den |van |von |de la |de |del |le |du |da |dos |di |ten |ter )?(?:O'|O’|Mc|Mac|D'|d')?[{UP}][{LO}]+(?:[-‐][{UP}][{LO}]+)?)"
INIT = rf"(?:,?\s(?:[{UP}]\.(?:[\s-]?[{UP}]\.)*))"
AUTH = rf"{SUR}(?:{INIT})?"
SEP = r"(?:,\s(?:and\s|&\s)?|\s(?:and|&)\s)"
LIST = rf"(?P<authors>{AUTH}(?:{SEP}{AUTH}){{0,7}}(?:,?\set\.?\sal\.?)?)"
# A year, not a date or a launch designation ("1961-01-12", "1985-056A").
YEAR = r"(?P<year>1[6-9]\d\d|20[0-2]\d)(?P<suffix>[a-z])?(?![\d]|[-‐/]\d)"
CITE_RE = re.compile(rf"(?<![{UP}{LO}\-])(?:{LIST})(?:,|\s)\s?\(?{YEAR}")
DOI_RE = re.compile(r"\b(10\.\d{4,9}/[^\s\"'<>\]\[{}|]+)")
ARXIV_RE = re.compile(r"(?:arxiv\.org/(?:abs|pdf)/|arXiv[:\s]\s?)((?:\d{4}\.\d{4,5})|(?:[a-z\-]+(?:\.[A-Z]{2})?/\d{7}))", re.I)
MARK_RE = re.compile(
    r"from memory|to verify|verify before|not read here|not yet checked|to check|reference to find|"
    r"unverified|check before relying|DOI to add|not checked",
    re.I,
)

# Words that look like a surname before a year but are not authors.
STOP = set(
    """
    January February March April May June July August September October November December
    Jan Feb Mar Apr Jun Jul Aug Sep Sept Oct Nov Dec Spring Summer Autumn Winter
    In Since Until From By Before After During The A An And Or Of For To On At As Up Is Was
    Version Edition Release Data Catalogue Catalog Table Figure Fig Section Chapter Page
    Year Years Earth Sun Moon Mars Venus Jupiter Saturn Uranus Neptune Pluto Mercury Titan
    Europa Io Ganymede Callisto Enceladus Triton Ceres Vesta Charon Halley
    Gaia Hipparcos Kepler Hubble Juno Cassini Voyager Galileo Viking Rosetta Dawn Messenger
    Apollo Luna Mariner Pioneer Magellan Huygens Planck Webb Spitzer Herschel Chandra Swift
    Curiosity Perseverance Insight Phoenix Opportunity Spirit Sputnik Vostok Mir Skylab
    Shoemaker Levy Comet Supernova Nova Storm Spot Event Impact Eruption Flare Burst
    Traveller Mongoose Marc Elite Star Wars Trek Dune Alien Aliens Expanse Firefly
    Since Around About Circa Approximately Updated Accessed Retrieved Published Revised
    New Old Late Early Mid Today Now Then Here There This That These Those
    North South East West Northern Southern Eastern Western Pacific Atlantic
    Claude Alex Orac Minerva Tars Fable Opus Sonnet Haiku
    Copyright Licence License Apache MIT GPL Formulation One Two Three Four Five Six Seven Eight Nine Ten Rust Node Svelte Python Vite WebGPU WebGL Chrome Firefox Safari Windows
    Astronomy Astrophysics Astrophysical Journal Review Reviews Letters Proceedings Annual Society Monthly
    Notices Geophysical Research Physics Physical Nature Science Sciences Icarus Academy Academies Press
    University Institute Survey Conference Symposium Meeting Workshop Report Bulletin Transactions
    Christmas Easter Monday Tuesday Wednesday Thursday Friday Saturday Sunday
    """.split()
)
# Words never allowed anywhere in the author run (they mean the match is prose).
STOP_ANY = {"The", "In", "Since", "From", "By", "Until", "Table", "Figure", "Version", "Edition"}


def fold(s):
    s = unicodedata.normalize("NFKD", s)
    return "".join(c for c in s if not unicodedata.combining(c)).lower()


def first_surname(authors):
    m = re.match(SUR, authors)
    return m.group(0) if m else authors.split()[0]


def surname_key(sur):
    s = fold(sur)
    s = re.sub(r"^(van der |van den |van |von |de la |de |del |le |du |da |dos |di |ten |ter )", "", s)
    return re.sub(r"[^a-z]", "", s)


def clean_doi(d):
    d = d.rstrip(".,;:)")
    # A DOI with an unbalanced closing bracket loses it ("(doi 10.x/y)").
    while d.endswith(")") and d.count("(") < d.count(")"):
        d = d[:-1]
    return d.lower()


def find_citations(text):
    """Yield (key, written, year) for each author-year citation in one piece of text."""
    for m in CITE_RE.finditer(text):
        authors = m.group("authors").strip()
        sur = first_surname(authors)
        if sur in STOP:
            continue
        words = re.findall(rf"[{UP}][{LO}]+", authors)
        if any(w in STOP_ANY for w in words):
            continue
        # A lone capitalised word right after a sentence full stop is usually prose ("Sunlight 2000").
        if len(sur) < 3 and "et al" not in authors:
            continue
        year = int(m.group("year"))
        if year > 2026:
            continue
        key = surname_key(sur) + str(year) + (m.group("suffix") or "")
        yield key, m.group(0), m.start(), m.end()


def snippet(text, start, end, width=260):
    if len(text) <= width:
        return text.strip()
    a = max(0, start - width // 3)
    b = min(len(text), a + width)
    s = text[a:b].strip()
    return ("…" if a > 0 else "") + s + ("…" if b < len(text) else "")


def written_form(text, start):
    """The citation as written: from its author list to the end of the reference (DOI, URL or clause)."""
    tail = text[start : start + 400]
    first = CITE_RE.match(tail)
    if first:
        nxt = CITE_RE.search(tail, first.end())
        if nxt:
            tail = tail[: nxt.start()]
    cut = len(tail)
    for stop in ["; ", " | ", "\n", ": ", " — "]:
        i = tail.find(stop)
        if 0 < i < cut:
            cut = i
    s = tail[:cut]
    # End at a closing bracket that was never opened, or at a new sentence.
    depth = 0
    for i, ch in enumerate(s):
        depth += ch == "("
        if ch == ")":
            depth -= 1
            if depth < 0:
                s = s[:i]
                break
    m = re.search(r"(?<=[a-z]{3})\. [A-Z][a-z]{3,}\s", s)
    if m:
        s = s[: m.start()]
    s = re.sub(r"\*\*|`|\[\[|\]\]", "", s)
    s = re.sub(r'"?\.to_string\(\)|",?$|^"', "", s.strip())
    return s.strip().rstrip(".,;( ")


# --- walking the sources ----------------------------------------------------------


def rel(path):
    return os.path.relpath(path, ROOT).replace(os.sep, "/")


def files_under(roots, exts):
    out = []
    for r in roots:
        top = os.path.join(ROOT, r) if not os.path.isabs(r) else r
        if os.path.isfile(top):
            out.append(top)
            continue
        for d, dirs, fs in os.walk(top):
            dirs[:] = sorted(x for x in dirs if x not in SKIP_DIRS)
            for f in sorted(fs):
                # A citation check's own report lists the evidence it read, not claims SSE4 rests on.
                if f.endswith("-citation-check.md"):
                    continue
                if os.path.splitext(f)[1] in exts:
                    out.append(os.path.join(d, f))
    return sorted(out)


def json_strings(o, path, ctx):
    """Yield (path, text, context) for every string in a JSON value, with its block id and any sibling value."""
    if isinstance(o, dict):
        here = ctx
        if isinstance(o.get("id"), str):
            here = o["id"]
        value = None
        if "value" in o and not isinstance(o["value"], (dict, list)):
            value = f"{o['value']}" + (f" {o['unit']}" if isinstance(o.get("unit"), str) else "")
        for k in o:
            v = o[k]
            p = f"{path}/{k}"
            if isinstance(v, str):
                c = here
                if value is not None and k in ("source", "text", "ref", "why", "note"):
                    c = f"{here} {path.rsplit('/', 1)[-1]} = {value}".strip()
                yield p, v, c
            else:
                yield from json_strings(v, p, here)
        # A references object {text, doi, url} reads as one citation.
        if isinstance(o.get("text"), str) and (isinstance(o.get("doi"), str) or isinstance(o.get("url"), str)):
            joined = o["text"] + (f" doi:{o['doi']}" if isinstance(o.get("doi"), str) else "") + (
                f" {o['url']}" if isinstance(o.get("url"), str) else ""
            )
            yield path, joined, here
    elif isinstance(o, list):
        for i, v in enumerate(o):
            yield from json_strings(v, f"{path}[{i}]", ctx)


def pieces(path):
    """Yield (where, text) pieces of one file: JSON strings, or lines of code and notes."""
    r = rel(path)
    if path.endswith(".json"):
        try:
            with open(path, encoding="utf-8") as fh:
                data = json.load(fh)
        except (ValueError, UnicodeDecodeError):
            return
        for p, text, ctx in json_strings(data, "", ""):
            if not re.search(r"\d{4}|10\.\d{4}|arxiv", text, re.I):
                continue
            where = f"{r} {ctx}".strip() if ctx else f"{r} {p}"
            yield where, text
        return
    try:
        with open(path, encoding="utf-8") as fh:
            lines = fh.read().split("\n")
    except UnicodeDecodeError:
        return
    if path.endswith(".md"):
        heading = ""
        for i, line in enumerate(lines, 1):
            if line.startswith("#"):
                heading = line.lstrip("# ").strip()
            if re.search(r"\d{4}|10\.\d{4}|arxiv", line, re.I):
                yield r + (f" ({heading[:60]})" if heading else ""), line
        return
    # Places carry no line numbers, so an edit elsewhere in a file never changes the list.
    for line in lines:
        if re.search(r"\d{4}|10\.\d{4}|arxiv", line, re.I):
            yield r, line.strip()


def gather(roots, exts):
    """Return {key: paper} gathered from the files under roots."""
    papers = {}
    for path in files_under(roots, exts):
        for where, text in pieces(path):
            found = list(find_citations(text))
            if not found:
                # A bare DOI or arXiv id still names a paper.
                for d in DOI_RE.findall(text):
                    d = clean_doi(d)
                    key = "doi:" + d
                    add(papers, key, d, None, where, text, 0, len(text), [d], [])
                continue
            dois = [clean_doi(d) for d in DOI_RE.findall(text)]
            arx = [a.lower() for a in ARXIV_RE.findall(text)]
            for key, written, s, e in found:
                # A DOI belongs to the citation it follows most closely.
                tail = text[e : e + 400]
                nxt = CITE_RE.search(tail)
                clause = tail[: nxt.start()] if nxt else tail
                my_dois = [clean_doi(d) for d in DOI_RE.findall(clause)] if len(found) > 1 else dois
                my_arx = [a.lower() for a in ARXIV_RE.findall(clause)] if len(found) > 1 else arx
                add(papers, key, written_form(text, s), int(re.search(YEAR, written).group("year")), where, text, s, e, my_dois, my_arx)
    return merge_dois(papers)


def merge_dois(papers):
    """Fold a bare-DOI entry into the author-year entry that carries the same DOI."""
    owner = {}
    for k, p in papers.items():
        if not k.startswith("doi:"):
            for d in p["dois"]:
                owner.setdefault(d, k)
    for k in [k for k in papers if k.startswith("doi:")]:
        o = owner.get(k[4:])
        if o:
            src = papers.pop(k)
            for f in ("places", "claims", "marks"):
                for v in src[f]:
                    if v not in papers[o][f]:
                        papers[o][f].append(v)
    return papers


def add(papers, key, written, year, where, text, s, e, dois, arx):
    p = papers.setdefault(
        key,
        {"key": key, "written": [], "dois": [], "arxiv": [], "places": [], "claims": [], "marks": []},
    )
    if written and written not in p["written"]:
        p["written"].append(written)
    for d in dois:
        if d not in p["dois"]:
            p["dois"].append(d)
    for a in arx:
        if a not in p["arxiv"]:
            p["arxiv"].append(a)
    if where not in p["places"]:
        p["places"].append(where)
    c = snippet(text, s, e)
    if c not in p["claims"]:
        p["claims"].append(c)
    for m in MARK_RE.findall(text):
        m = m.lower()
        if m not in p["marks"]:
            p["marks"].append(m)


def golden_keys():
    keys = set()
    for path in files_under([GOLDEN], {".json", ".jsonl"}):
        with open(path, encoding="utf-8") as fh:
            text = fh.read()
        for key, _, _, _ in find_citations(text):
            keys.add(key)
    return keys


def earlier_checks():
    """Statuses from the first citation pass (docs/citations*), by key: the strongest word found."""
    rank = {"verified": 4, "corrected": 3, "textbook": 2, "partly verified": 2, "metadata confirmed": 1, "unverified": 0}
    out = {}
    for path in files_under(EARLIER, {".md"}):
        with open(path, encoding="utf-8") as fh:
            for line in fh:
                if not line.startswith("|") or line.startswith("|---"):
                    continue
                low = line.lower()
                word = None
                for w in ["partly verified", "metadata confirmed", "unverified", "corrected", "verified", "textbook"]:
                    if w in low:
                        word = w
                        break
                if word is None:
                    continue
                for key, _, _, _ in find_citations(line):
                    old = out.get(key)
                    if old is None or rank[word] < rank[old[0]]:
                        # Keep the weakest: one unverified row is enough to want a check.
                        out[key] = (word, rel(path))
    return out


# --- the master list --------------------------------------------------------------


def load_master():
    rows = []
    if os.path.exists(MASTER):
        with open(MASTER, encoding="utf-8") as fh:
            for line in fh:
                line = line.strip()
                if line:
                    rows.append(json.loads(line))
    return rows


def write_master(rows):
    os.makedirs(LIST_DIR, exist_ok=True)
    with open(MASTER, "w", encoding="utf-8") as fh:
        for r in rows:
            fh.write(json.dumps(r, ensure_ascii=False, sort_keys=False) + "\n")


def tier_of(row):
    """Order of work: 1 marked from memory; 2 behind golden-tested numbers; 3 code and packs; 4 design notes only."""
    if row["marks"] and any(m in ("from memory", "to verify", "verify before", "check before relying") for m in row["marks"]):
        return 1
    if row.get("golden"):
        return 2
    if row.get("in_code"):
        return 3
    return 4


def best_written(written):
    """The fullest form that is still one reference: a journal and volume or a DOI, then a title, not prose."""
    if not written:
        return ""

    def rank(w):
        full = bool(re.search(r"\b\d{1,4}[,:(]\s?[A-Z]?\d|doi|10\.\d{4}|arxiv|https?://", w, re.I))
        prose = bool(re.search(r"\b(?:I|we|my|our|appear|found|used|gives|shows?|show)\b", w))
        return (not full, prose, len(w) > 240, -min(len(w), 240), w)

    return sorted(written, key=rank)[0]


def computed_rows(extra):
    """Every paper as gathered now, keyed, with no check results yet."""
    allp = gather(CODE_ROOTS + NOTE_ROOTS + extra, CODE_EXT | {".md"})
    code_tops = tuple(r + "/" for r in CODE_ROOTS)
    gold = golden_keys()
    early = earlier_checks()
    out = {}
    for key in sorted(allp):
        p = allp[key]
        row = {
            "key": key,
            "citation": best_written(p["written"]) or key,
            "doi": p["dois"],
            "arxiv": p["arxiv"],
            "marks": p["marks"],
            "golden": key in gold,
            "in_code": any(w.startswith(code_tops) for w in p["places"]),
            "earlier": "{} ({})".format(*early[key]) if key in early else "",
            "status": "unchecked",
            "checked": "",
            "note": "",
            "found": "",
            "places": p["places"],
            "claims": p["claims"][:6],
        }
        row["tier"] = tier_of(row)
        out[key] = row
    return out


RESULT_FIELDS = ("status", "checked", "note", "found")


def update(extra, only_add=False):
    """Refresh the list. only_add: leave every existing row untouched and add the new papers (builders use this)."""
    now = computed_rows(extra)
    old = {r["key"]: r for r in load_master()}
    rows = {}
    for key, row in now.items():
        prev = old.get(key)
        if prev is None:
            rows[key] = row
        elif only_add:
            rows[key] = prev
        else:
            for f in RESULT_FIELDS:
                row[f] = prev.get(f, row[f])
            rows[key] = row
    # A paper no longer cited anywhere stays on the list, marked, so a score is never lost.
    for key, prev in old.items():
        if key not in rows:
            # A hand-added row ("manual": true) names a source the gatherer cannot see (no author-year form).
            if not only_add and not prev.get("manual"):
                prev["places"] = []
                prev["gone"] = True
            rows[key] = prev
    out = [rows[k] for k in sorted(rows)]
    write_master(out)
    if not only_add:
        write_md(out)
    return out


def write_md(rows):
    live = [r for r in rows if not r.get("gone")]
    counts = {s: sum(1 for r in live if r["status"] == s) for s in STATUSES}
    tiers = {t: [r for r in live if r["tier"] == t] for t in (1, 2, 3, 4)}
    names = {1: "Marked from memory or to verify", 2: "Behind golden-tested numbers", 3: "Other packs and code", 4: "Design notes only"}
    out = [
        "# Citation run-through: master list",
        "",
        "Written by `scripts/citations.py --update` from `master.jsonl` (the source of truth; edit status there, or score a",
        "Consensus reply with `--score`). Procedure: [README.md](README.md). Do not edit this file by hand.",
        "",
        f"**{len(live)} papers.** " + ", ".join(f"{s}: {counts[s]}" for s in STATUSES) + ".",
        "",
        "| Order | Papers | Unchecked |",
        "|---|---|---|",
    ]
    for t in (1, 2, 3, 4):
        out.append(f"| {t}. {names[t]} | {len(tiers[t])} | {sum(1 for r in tiers[t] if r['status'] == 'unchecked')} |")
    out += ["", "Status words: " + ", ".join(f"*{s}*" for s in STATUSES) + ".", ""]
    for t in (1, 2, 3, 4):
        out += [f"## {t}. {names[t]}", "", "| Key | Citation | DOI | Cited in | Status | Checked | Note |", "|---|---|---|---|---|---|---|"]
        for r in sorted(tiers[t], key=order_key):
            cite = r["citation"].replace("|", "/")[:160]
            doi = ", ".join(r["doi"][:2])
            where = r["places"][0].replace("|", "/")[:70] + (f" (+{len(r['places']) - 1})" if len(r["places"]) > 1 else "")
            note = r["note"].replace("|", "/")[:120]
            out.append(f"| {r['key']} | {cite} | {doi} | {where} | {r['status']} | {r['checked']} | {note} |")
        out.append("")
    gone = [r for r in rows if r.get("gone")]
    if gone:
        out += ["## No longer cited", "", ", ".join(f"{r['key']} ({r['status']})" for r in gone), ""]
    with open(MASTER_MD, "w", encoding="utf-8") as fh:
        fh.write("\n".join(out))


def order_key(r):
    weak = 0 if r["earlier"].startswith(("unverified", "metadata", "partly")) or not r["earlier"] else 1
    return (r["tier"], weak, -len(r["places"]), r["key"])


def check(warn=False):
    rows = load_master()
    known = {r["key"] for r in rows} | {"doi:" + d for r in rows for d in r.get("doi", [])}
    bad = [r["key"] for r in rows if r.get("status") not in STATUSES]
    code = gather(CODE_ROOTS, CODE_EXT)
    notes = gather(NOTE_ROOTS, {".md"})
    missing = sorted(k for k in code if k not in known)
    missing_notes = sorted(k for k in notes if k not in known and k not in code)
    if missing_notes:
        print(
            f"note: {len(missing_notes)} design-note citations are not on the list yet (first: {missing_notes[0]}); "
            "run --add when convenient",
            file=sys.stderr,
        )
    if bad:
        print("citations: unknown status on " + ", ".join(bad) + f"; use one of {STATUSES}", file=sys.stderr)
    if missing:
        for k in missing:
            print(f"citations: {k} cited at {code[k]['places'][0]} is not on handover/citations/master.jsonl", file=sys.stderr)
        print("Run python3 scripts/citations.py --add and commit handover/citations/master.jsonl.", file=sys.stderr)
    if bad or missing:
        if warn:
            print(f"citations: warning only: {len(missing)} new papers to add", file=sys.stderr)
            return
        sys.exit(1)
    print(f"citations: all {len(code)} pack and code citations are on the master list")


# --- daily Consensus briefs -------------------------------------------------------

BRIEF_HEAD = """# Consensus batch {n}: {count} papers ({date})

Paste everything below the line into a Claude conversation that has the **Consensus** connector, then paste
Claude's whole reply back into the SSE4 citation thread (or save it as `results-{n:02d}.md` beside this file).

---

You are checking the references of a science-education program. For each numbered paper below, use Consensus
(search by title words, authors and year, or by DOI) to find the paper and read its abstract or text. For each:

1. Does the paper exist as cited? Give the full citation you found: authors, year, title, journal, volume, pages, DOI.
2. What is its key formula, number or finding that bears on the claim?
3. Does it say what we claim? Answer with exactly one status word:
   - `confirmed`: it exists as cited and supports the claim;
   - `wrong citation`: the paper exists but a detail is wrong (year, journal, DOI, author, equation number); say what;
   - `says something else`: the paper does not support the claim, or gives a different number or formula; say what it says;
   - `not found`: Consensus cannot find it (say what you searched).
   If you could only read the abstract and the claim is a detail inside the paper, say so in the note and use
   `confirmed` only when the abstract itself supports it; otherwise use `not found` with the note "abstract only".

Do not guess. Keep each note under 60 words.

Reply with one line per paper, in this exact form, and nothing between the lines:

`key | status | full citation found | key formula or figure | note`

Papers:
"""


def brief_item(r):
    claims = []
    for c in r["claims"]:
        c = re.sub(r"\s+", " ", c)
        if c not in claims:
            claims.append(c[:300])
        if len(claims) == 3:
            break
    lines = [f"**{r['key']}**: {r['citation']}"]
    if r["doi"]:
        lines.append(f"   DOI as cited: {', '.join(r['doi'][:3])}")
    if r["arxiv"]:
        lines.append(f"   arXiv as cited: {', '.join(r['arxiv'][:3])}")
    lines.append("   What we rest on it:")
    for c in claims:
        lines.append(f"   - {c}")
    return "\n".join(lines)


def batches(out_dir, size, count, start):
    rows = [r for r in load_master() if not r.get("gone") and r["status"] == "unchecked"]
    rows.sort(key=order_key)
    os.makedirs(out_dir, exist_ok=True)
    made = []
    for b in range(count):
        chunk = rows[b * size : (b + 1) * size]
        if not chunk:
            break
        n = start + b
        date = datetime.date.today().isoformat()
        text = BRIEF_HEAD.format(n=n, count=len(chunk), date=date) + "\n" + "\n\n".join(brief_item(r) for r in chunk) + "\n"
        path = os.path.join(out_dir, f"batch-{n:02d}.md")
        with open(path, "w", encoding="utf-8") as fh:
            fh.write(text)
        made.append((path, chunk[0]["key"], chunk[-1]["key"]))
    for p, a, z in made:
        print(f"{p}: {a} to {z}")


# --- scoring a reply --------------------------------------------------------------

LINE_RE = re.compile(r"^[\W_]*([a-z]+\d{4}[a-z]?|doi:\S+?)[*`:.\s]*\|\s*([a-z ]+?)\s*\|\s*(.*?)\s*\|\s*(.*?)\s*\|\s*(.*?)\s*\|?\s*`?$", re.I)


def score(path, date):
    rows = load_master()
    by_id = {r["key"]: r for r in rows}
    done, odd = [], []
    with open(path, encoding="utf-8") as fh:
        for line in fh:
            m = LINE_RE.match(line.strip())
            if not m:
                continue
            rid, status, found, figure, note = m.groups()
            status = status.lower().strip()
            if rid not in by_id or status not in STATUSES:
                odd.append(line.strip()[:120])
                continue
            r = by_id[rid]
            r["status"] = status
            r["checked"] = f"{date} Consensus"
            r["found"] = found.strip()
            r["note"] = (figure.strip() + ("; " if figure.strip() and note.strip() else "") + note.strip())[:600]
            done.append((rid, status))
    write_master(rows)
    write_md(rows)
    for rid, s in done:
        print(f"{rid}: {s}")
    for o in odd:
        print(f"not understood: {o}", file=sys.stderr)
    wrong = [d for d in done if d[1] != "confirmed"]
    print(f"scored {len(done)}; {len(wrong)} need a look (raise each with its owning thread)")


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--update", action="store_true")
    ap.add_argument("--add", action="store_true", help="add new papers only, touching no existing row")
    ap.add_argument("--check", action="store_true")
    ap.add_argument("--batches", metavar="DIR")
    ap.add_argument("--size", type=int, default=30)
    ap.add_argument("--count", type=int, default=1)
    ap.add_argument("--start", type=int, default=1, help="number of the first batch written")
    ap.add_argument("--warn", action="store_true", help="with --check: report missing papers but do not fail")
    ap.add_argument("--score", metavar="FILE")
    ap.add_argument("--date", default=datetime.date.today().isoformat())
    ap.add_argument("--extra", action="append", default=[], help="another folder of design notes to read")
    a = ap.parse_args()
    if a.update or a.add:
        rows = update(a.extra, only_add=a.add and not a.update)
        live = [r for r in rows if not r.get("gone")]
        print(f"{len(live)} papers on the list ({sum(1 for r in live if r['status'] == 'unchecked')} unchecked)")
    if a.score:
        score(a.score, a.date)
    if a.batches:
        batches(a.batches, a.size, a.count, a.start)
    if a.check:
        check(a.warn)
    if not (a.update or a.add or a.check or a.batches or a.score):
        ap.print_help()


if __name__ == "__main__":
    main()

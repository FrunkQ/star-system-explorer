#!/usr/bin/env python3
"""Tests for scripts/citations.py: what counts as a citation, how it is keyed, and how a reply is scored."""

import importlib.util
import json
import os
import sys
import tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
spec = importlib.util.spec_from_file_location("citations", os.path.join(HERE, "citations.py"))
c = importlib.util.module_from_spec(spec)
spec.loader.exec_module(c)


def keys(text):
    return [k for k, _, _, _ in c.find_citations(text)]


CASES = [
    ("Wright, N. J., Drake, J. J., Mamajek, E. E. and Henry, G. W. 2011, ApJ 743, 48", ["wright2011"]),
    ("the thin disc (Shakura and Sunyaev 1973), falling", ["shakura1973"]),
    ("Eker et al. 2018, MNRAS 479, 5491", ["eker2018"]),
    ("Iye, Tadaki and Fukumoto (2019, arXiv:1910.10926)", ["iye2019"]),
    ("von Zeipel 1924 and Lucy 1967 for gravity darkening", ["zeipel1924", "lucy1967"]),
    ("Sánchez-Lavega et al. 2011, Nature 475, 71", ["sanchezlavega2011"]),
    ("O'Neill, Emanuel and Flierl 2015", ["oneill2015"]),
    ("Kopparapu et al. 2013a and 2013b", ["kopparapu2013a"]),
    ("LaRowe, D. E. and Amend, J. P. 2015, Frontiers", ["larowe2015"]),
    ("Tegeler, Ch., Span, R. and Wagner, W. 1999, J. Phys. Chem.", ["tegeler1999"]),
    ("Millot, M., Coppari, F., Rygg, J. R., Correa Barrios, A., Hamel, S., Swift, D. C. and Eggert, J. H. 2019, Nature", ["millot2019"]),
    # Not citations: dates, launch designations, missions, months, journals.
    ("launched 1961 Feb 12; Venera (1961-003A)", []),
    ("Horizons, 2004-07-02", []),
    ("In 1999 the probe; Since 2005; January 2020; Voyager 1977", []),
    ("Astronomy & Astrophysics 2023", []),
]


def row(key, status="unchecked"):
    return {"key": key, "citation": key, "doi": [], "status": status, "places": []}


def split_list_tests():
    """One file per paper: names, round trip, the two failures, and moving a pre-split master.jsonl in."""
    bad = 0
    names = [c.paper_file(k) for k in ["shakura1973", "doi:10.1175/a_b", "doi:10.1175/a/b", "doi:10.1175/A-B"]]
    if names[0] != "shakura1973.json" or len(set(names)) != len(names):
        print(f"FAIL paper_file names: {names}")
        bad += 1
    with tempfile.TemporaryDirectory() as tmp:
        c.PAPERS = os.path.join(tmp, "papers")
        c.LEGACY = os.path.join(tmp, "master.jsonl")
        rows = [row("aarseth2001"), row("doi:10.1175/a_b")]
        c.write_master(rows)
        if [r["key"] for r in c.load_master()] != ["aarseth2001", "doi:10.1175/a_b"]:
            print("FAIL round trip")
            bad += 1
        # A key in a file of the wrong name (a hand-made copy) is refused, so a duplicate cannot hide.
        with open(os.path.join(c.PAPERS, "copy.json"), "w", encoding="utf-8") as fh:
            fh.write(json.dumps(row("aarseth2001")) + "\n")
        try:
            c.read_papers()
            print("FAIL a misnamed or duplicate paper file was accepted")
            bad += 1
        except c.ListError:
            pass
        os.remove(os.path.join(c.PAPERS, "copy.json"))
        # A branch from before the split: its master.jsonl is moved in, scores kept, and deleted.
        with open(c.LEGACY, "w", encoding="utf-8") as fh:
            for r in [row("aarseth2001", "confirmed"), row("zahn1977")]:
                fh.write(json.dumps(r) + "\n")
        c.absorb_legacy()
        got = {r["key"]: r["status"] for r in c.load_master()}
        if got != {"aarseth2001": "confirmed", "doi:10.1175/a_b": "unchecked", "zahn1977": "unchecked"} or os.path.exists(c.LEGACY):
            print(f"FAIL absorbing master.jsonl: {got}")
            bad += 1
    return bad


def main():
    bad = split_list_tests()
    for text, want in CASES:
        got = keys(text)
        if got != want:
            print(f"FAIL {text!r}: got {got}, want {want}")
            bad += 1
    if c.clean_doi("doi:10.1086/160960).") != "doi:10.1086/160960" and c.clean_doi("10.1086/160960).") != "10.1086/160960":
        print("FAIL clean_doi")
        bad += 1
    sici = c.DOI_RE.findall("doi:10.1175/1520-0426(1999)016<1854:ORODC>2.0.CO;2 and more")
    if sici != ["10.1175/1520-0426(1999)016<1854:ORODC>2.0.CO;2"]:
        print(f"FAIL SICI doi: {sici}")
        bad += 1
    m = c.LINE_RE.match("shakura1973 | confirmed | Shakura and Sunyaev 1973, A&A 24, 337 | T^4 law | read")
    if not m or m.group(1) != "shakura1973" or m.group(2) != "confirmed":
        print("FAIL reply line")
        bad += 1
    m = c.LINE_RE.match("`rhines1975 | says something else | Rhines 1975 | L | note`")
    if not m or m.group(2) != "says something else":
        print("FAIL reply line in backticks")
        bad += 1
    if bad:
        sys.exit(1)
    print(f"citations tests: {len(CASES) + 8} passed")


if __name__ == "__main__":
    main()

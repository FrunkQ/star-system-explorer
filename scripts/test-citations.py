#!/usr/bin/env python3
"""Tests for scripts/citations.py: what counts as a citation, how it is keyed, and how a reply is scored."""

import importlib.util
import os
import sys

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
    # Not citations: dates, launch designations, missions, months, journals.
    ("launched 1961 Feb 12; Venera (1961-003A)", []),
    ("Horizons, 2004-07-02", []),
    ("In 1999 the probe; Since 2005; January 2020; Voyager 1977", []),
    ("Astronomy & Astrophysics 2023", []),
]


def main():
    bad = 0
    for text, want in CASES:
        got = keys(text)
        if got != want:
            print(f"FAIL {text!r}: got {got}, want {want}")
            bad += 1
    if c.clean_doi("doi:10.1086/160960).") != "doi:10.1086/160960" and c.clean_doi("10.1086/160960).") != "10.1086/160960":
        print("FAIL clean_doi")
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
    print(f"citations tests: {len(CASES) + 3} passed")


if __name__ == "__main__":
    main()

# RS8: flight logs for every departed craft

Track RS (`handover/design/SSE4-real-spaceflight-feasibility.md`). Owner thread: "Build real spaceflight history pack".

## Part 1 (24 Nov 2026): every payload that left Earth orbit

`scripts/spaceflight/build_departed_logs.py <GCAT tsv dir>` reads GCAT's phase tables (deepcat, hcocat, lprcat,
landercat; the same download as RS1) and writes `store-packs/sol-today/logs/departed/<construct>.json`, one log
per payload in `catalogue.json`'s `departed` list: 462 logs. The ten showcase craft (Cassini, Huygens, Juno, Dawn,
Curiosity, JWST, Voyager 2, Columbia, Eagle) are skipped, as `build_logs.py` writes them in full; ids that would
clash with a showcase log get the piece designation added (`intrepid-1969-u02`).

How a phase becomes a piece:

| GCAT phase | Piece |
|---|---|
| Around a planet or moon, bound | orbit with GCAT's periapsis, apoapsis and inclination; node, periapsis and phase zero; epoch at the phase start |
| Around a planet or moon, escape or flyby (negative apoapsis) | hyperbola from the same figures; a flyby (EN) is also a `flyby` event at GCAT's periapsis time, with the periapsis distance |
| Around the Sun, ending at a planet the NAV fixture holds | course solved from its ends (Lambert), kept when the solved perihelion and aphelion agree with GCAT's within 20% (or GCAT gives none); 118 such legs |
| Around the Sun otherwise | orbit with GCAT's perihelion, aphelion and inclination, turned so it starts where the craft left its last body (node at that body's longitude, true anomaly from the distance) |
| Sun-Earth L1/L2, Earth-Moon L1/L2 | an orbit like the Earth's 1% inside or outside; a circle at the point's distance around the Earth |
| Straight-in fall (periapsis deep below the ground) | planned course "falling to ..." |
| On a surface (TX, DEP, LO, TO) | landed at landercat's site (else GCAT's Dest coordinates); LO and TO end with a `launch` event; TX ends with a stop "last signal" |
| Undocked from a carrier (UDK) | docked to the carrier's log |
| Carried by another craft (EAO, REL, AL, AR ...) | the carrier's own figures: GCAT's attached rows can be stale copies (Galileo's probe repeats its launch orbit at Venus) |

Endings: L, AL become landed (impacted if landercat says impact or failed); R, D, AR, S become gone (reentered in an
atmosphere, impacted on an airless body); F, C, E impacted; OX lost contact; DK docked.

Earth-centred inclinations are equatorial, turned to the ecliptic with the node at the equinox. GCAT dates are UTC
(turned to TT); a date with '?' or without an hour marks the piece estimated. Every orbit piece is estimated and says
why (GCAT gives no orientation).

Table slips handled: a phase that wholly holds another around a different body is dropped (Galileo's probe has an
Earth phase dated 1989 for 1990); a solved leg whose next phase starts before the meeting falls back to a planned course.

### Results

- 462 logs: 108 end on the ground, 133 gone, 221 still flying or docked.
- 21 heliocentric legs have no figures in GCAT (the Vega and Venera buses after their flybys, Giotto, Nozomi and
  others) and are planned courses "around the Sun".
- Measured against Horizons (`departed_cruise_against_horizons`, `scripts/spaceflight/horizons_departed.json`):

| Craft, date | Piece | Off by | Of distance |
|---|---|---|---|
| Mars Odyssey, 1 Jul 2001 | solved course | 2.6 million km | 1.4% |
| MRO, 1 Dec 2005 | solved course | 1.4 million km | 0.7% |
| MAVEN, 1 Mar 2014 | solved course | 2.5 million km | 1.5% |
| New Horizons, 1 Oct 2006 / 2009 / 2012 | turned hyperbola | 2.2 / 88 / 163 million km | 0.4 / 4.9 / 5.0% |
| Rosetta, Sep 2004 / Jun 2006 | turned orbit | 4.8 / 9.3 million km | 3.1 / 4.4% |
| MESSENGER, Jun 2005 / Mar 2007 | turned orbit | 32 / 21 million km | 23 / 19% |

The test holds the worst under 30%. Solved courses are as good as the showcase logs; turned orbits are right in size
and plane and drift along the orbit.

## Part 2 (24 Nov 2026): stations and crewed flights in Earth orbit

The same script writes `logs/earth-orbit/`, 770 logs: each station core in `stations.json` and each human
spaceflight vehicle record in `catalogue.json` (`crewed`), from GCAT's satcat (one phase per object): an orbit
around the Earth with GCAT's shape and inclination at its epoch (turned to the ecliptic), a launch event, and the
phase's end: 169 landed (sites from GCAT's coordinates, or KSC, Edwards, White Sands and the Kazakh steppe placed
from memory, or a stand-in), 60 gone (reentered or deorbited, Mir among them), 520 docked (a Soyuz or a Shuttle
whose first phase ends at a station; GCAT's public tables keep no later phase for these; auxcat.tsv, checked 24 Nov, lists only the objects attached to them), 21 still in
orbit. The coordinates reader now takes either order, so Apollo 13's Odyssey and Chang'e 6's return capsule land
at GCAT's coordinates. Test `stations_and_crewed_flights_have_logs` (Mir gone, STS-103 at KSC).

### Checks

`crates/sse-nav/tests/sol_today_logs.rs` now reads `logs/` and `logs/departed/`: every log runs in order with
estimates explained, every solved course meets its planet, every departed craft has a log
(`every_departed_craft_has_a_log`), and the cruise measurement above.

### Still to do

- Part 2: stations and crewed flights in Earth orbit (psatcat and the station cores), from GCAT's orbits.
- Splits between carriers and what they release (Viking orbiter and lander, Galileo and its probe): GCAT's REL
  phases give the times; each would need the child's piece to start at the split, as the showcase logs do.
- Moons and small bodies in the NAV fixture, so pieces around them get a position.
- Horizons state vectors for the craft Horizons holds, in place of turned orbits.
- Radii of the moons and small bodies are from memory, to verify against JPL SSD.

# RS3: flight logs for the showcase missions (part 1, 24 Nov 2026)

Track RS, design `handover/design/SSE4-real-spaceflight-feasibility.md` sections 3 and 6A. The log shape is the travel thread's `crates/sse-nav/src/flightlog.rs` (PR 377).

## What landed

- `scripts/spaceflight/build_logs.py` writes `store-packs/sol-today/logs/`, one flight log per construct, 30 in all. Run `python3 -I scripts/spaceflight/build_logs.py`; never edit the logs by hand.
- One split mechanism for every one-use part. A split names new constructs, and each has its own log from that moment:
  - **Saturn V / Apollo 11**: `apollo-11` (Columbia) splits off the S-IC at 2:42.30 and the S-II at 9:09.00. Both fall on ballistic arcs and end `gone: impacted` at the theoretical impact times. The S-IVB splits off at extraction and slings past the Moon into orbit about the Sun. Eagle undocks, lands at Tranquility Base, leaves `apollo-11-eagle-descent-stage` there (`landed`), docks again (join and a docked piece) and is jettisoned into lunar orbit. The service module ends `gone: reentered` at entry.
  - **Falcon 9 flight 20** (22 Dec 2015 UTC): the stage-two log splits off booster `falcon-9-b1019` at T+2:25. The booster's return piece names its three burns, and it ends `landed` on Landing Zone 1 at T+9:44. The eleven ORBCOMM satellites split off at their six release times, each into the 614 by 656 km orbit.
  - **STS-1**: the two SRBs (landed in the Atlantic, recovered) and the external tank (on the 13 by 80 nmi MECO orbit, `gone: reentered`). Columbia flies the five OMS burns (50.3, 41.8, 11.1, 11.4 and 91.1 m/s) and lands on Rogers Dry Lake at NSSDCA's flown duration.
  - **Cassini-Huygens** from JPL Horizons:
    - The flybys of Venus (twice), Earth and Jupiter, at Horizons' periapsis times and distances.
    - The deep space manoeuvre: 450 m/s on 3 Dec 1998, from the elements before and after.
    - Saturn orbit insertion: 576 m/s, with a range of 576 to 645 m/s depending on the window used.
    - One osculating Saturn orbit per year to 2017, then `gone: impacted` when Horizons' trajectory ends on 15 Sep 2017.
    - Huygens splits off on 25 Dec 2004 and lands on Titan.
  - **Voyager 2**: the travel thread's fixture, copied in with its Centaur and propulsion module given logs.
- `crates/sse-nav/tests/sol_today_logs.rs` (4 tests) checks the logs:
  - every log parses, and its pieces run with no gaps;
  - every estimate says why;
  - every split meets a log with a piece starting then, and every join meets a docked piece;
  - one-use parts end `gone`, and the reused booster and the SRBs end `landed`;
  - Cassini's solved arcs meet each flyby planet within 1,000 km;
  - Cassini's cruise is measured against Horizons.
- `scripts/spaceflight/horizons_cassini.json` keeps the Horizons extracts the builder reads.

## The big-burns fudge, measured

Cassini's log against Horizons' heliocentric positions:

| Date | Piece | Off by |
|---|---|---|
| 15 Jan 1998 | to Venus 1 | 359 thousand km (0.27%) |
| 1 Sep 1998 | orbit after Venus 1 | 2 thousand km (0.00%) |
| 15 Mar 1999 | to Venus 2 | 30 thousand km (0.01%) |
| 1 Mar 2000 | to Jupiter | 6.0 million km (1.37%) |
| 1 Oct 2001 | to Saturn | 8.1 million km (0.84%) |
| 1 Jan 2003 | to Saturn | 3.6 million km (0.30%) |
| 1 Jan 2004 | to Saturn | 5.9 million km (0.45%) |

The worst error is mid-way to Jupiter. The arcs are solved to the planets' centres, and the NAV fixture's planet orbits are its own Kepler approximations, so part of the error is the fixture's. The test holds it under 2%. Docs: the fudge register row "Every real craft's flight log" now carries these numbers.

## Corrections found

- Apollo 11 TEI is at 04:55:42 GMT (SP-4029). The fixture is right, and NSSDCA's 04:54:42 is a slip.
- The extraction time is 17:49:03, the ascent stage jettison 23:41:31 and the CM/SM separation 16:21:12. The pack log uses these; the travel thread's fixture `tests/fixtures/flightlog/apollo-11.json` keeps the rounded ones.

## Still marked from memory or estimated

- Apollo 11's orbit sizes and burn delta-v: the mission report is to be read. Horizons holds the Apollo 11 S-IVB (-399110) for orbit orientation.
- STS-1's times: these are the press kit's plan. The launch time is from memory, and the SRB splashdowns and the tank's breakup are estimated.
- Pad and landing-zone positions and Huygens' landing site: from memory, marked.
- No burn events for the Falcon 9 booster's return, because SpaceX publishes no delta-v. RS5's return physics flies them.

## Next (RS3 part 2)

- Logs for Dawn, Juno, Curiosity, JWST, the ISS, the V-2 and SNAP-10A, mostly from Horizons.
- Read the Apollo 11 and STS-1 mission reports for the flown figures.
- Measure low-orbit phase drift (STS-1).

# RS3 part 2 (24 Nov 2026): the rest of the showcase craft

- `build_logs.py` now writes 44 logs. It also reads `scripts/spaceflight/horizons_more.json`, which holds Horizons extracts for Juno, Dawn, MSL, JWST and the ISS.
  - **Juno**: the Earth departure, the two deep space manoeuvres (344.3 and 387.9 m/s, close to the published figures), the Earth flyby of 9 Oct 2013 at 6,916 km, and Jupiter orbit insertion (516 m/s). JOI is worked out from the speed at perijove using the far elements, because Jupiter's oblateness swings the near ones. After that come yearly Jupiter orbits to 2026. The Centaur splits off.
  - **Dawn**: no big burns, since it was on ion thrust. The log keeps Horizons' osculating orbit about the Sun every six months, the Mars flyby (3,937 km), and its orbits at Vesta and Ceres. Those are worked out from Horizons' vectors and match the survey, HAMO and LAMO orbits. It ends `lost_contact` on 1 Nov 2018. The Star 48 splits off.
  - **Curiosity**: the cruise is solved to arrive at Mars at entry interface. The cruise stage, heat shield, back shell and sky crane each split off and end `impacted`. The rover is `landed` at Bradbury Landing. The entry and landing times and the site are from memory, marked.
  - **JWST**: the Ariane 5 upper stage splits off at 27 min, then MCC-1a (20.0 m/s from Horizons), then osculating orbits about the Sun near L2 every three to six months.
  - **ISS**: Horizons' TLE-based orbit on 1 Jan each year from 1999 to 2026. The modules joining are RS8.
  - **SNAP-10A**: its 1,300 km polar orbit, from the secondary account.
- New tests in `sol_today_logs.rs`:
  - every solved course in every log arrives at its target, wherever the fixture holds it;
  - the ISS log's drift is measured against Horizons.
- Measured in low orbit (ISS), drift after a sample:
  - 1 hour: 41 km
  - 6 hours: 200 km
  - 1 day: 720 km
  - 1 week: 5,000 km
  - 1 month: 12,900 km
  
  So yearly samples give the size and plane of the orbit, not where along it the ISS is. RS8 samples low orbits every few hours for any time a game needs. The fudge register has these numbers.
- **V-2 MW 18014**, the first craft past 100 km: from GCAT's launch list. GCAT gives 14 Sep 1944 and 175 km, citing von Braun (1958), whereas the usual date is 20 Jun 1944. The log says so, and the hour and the fall are estimated.
- Apollo 11's orbit sizes and splashdown are now read from the mission report (sections 1 to 5) and marked recorded. Its burn delta-v stays marked from memory, because the report's trajectory section did not come through.

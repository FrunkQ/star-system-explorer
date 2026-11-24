# RK6 sources: the Paper spacecraft pack

Gathered on 24 Nov 2026 from fetched text of the primary documents (NASA, NTRS, arXiv, General Atomic). Most are OCR scans, so every value is **unverified** until someone checks it against the page image. "Summary only" means the value came from a model summary of a part of the document that was not returned as text; such values are **not used** in the pack. The Atomic Rockets site was used only as a map to these sources (plan section 7); nothing of its text, tables, selection or names is used.

Units are as written. Where the pack works a figure out, the working is in `scripts/build-paper-spacecraft.py` and the figure is marked `derived` in the pack.

## Discovery II (D-He3 spherical torus, crewed, Jupiter and Saturn)

Williams, C. H., Dudzinski, L. A., Borowski, S. K. and Juhasz, A. J. 2005, Realizing "2001: A Space Odyssey": piloted spherical torus nuclear fusion propulsion, NASA/TM-2005-213559 (AIAA-2001-3805). https://ntrs.nasa.gov/api/citations/20050160960/downloads/20050160960.pdf . Text OCR-noisy; Tables 1 and 2 read clean. US Government work.

| Quantity | Jupiter | Saturn | Where |
|---|---|---|---|
| IMLEO | 1,690 mt | 1,699 mt (Table 2; the text says mass properties were fixed for both, so 1,690 is used for both) | Tables 1, 2 |
| Payload | 172 mt | same | Table 2 |
| Structure (incl. reaction control propellant, reserves, residuals) | 646 mt | same | Table 1 |
| D-He3 fuel | 11 mt | same | Table 1 |
| Hydrogen | 861 mt: main impulse 807, reaction control 20, reserve 8, residuals 26 | same | Table 1 |
| Specific impulse | 35,435 lbf s/lbm | 47,205 lbf s/lbm | p. 7, Table 2 |
| Thrust | 6,250 lbf | 4,690 lbf | p. 7, Table 2 |
| Jet power | 4,830 MW | same | Table 2 |
| Jet efficiency | 0.8 ("assumed") | | p. 5 |
| Specific power | 8.62 kW/kg (reactor output over Ms) | ~ same | p. 5 |
| Initial thrust over mass | 1.68 milli-g | 1.25 milli-g | Table 2 |
| Trip time, one way | 118 days | 212 days | p. 7, Table 2 |
| Distance | 4.70 AU | 9.57 AU | Table 2 |
| Departure and arrival orbits | C3 ~ 0 | | Table 4 |
| Crew | six nominal (accommodation for twelve) | | p. 3 |
| Length | 240 m | | p. 3 |
| Shielding mass | 69 mt (in the payload) | | Table 1 |

Checks: ½ × 27,801 N × 347.5 km/s = 4.83 GW, and ½ × 20,862 N × 462.9 km/s = 4.83 GW: the two gears are one jet power. 27,801 N over 1,690 t is 1.68 milli-g. Summary only, not used: fusion power 7,895 MW, radiators 4,070 m² at 1,325 K, 1,119 MW waste heat, payload power 30 kWe.

## HOPE: crewed Callisto vehicles (magnetised target fusion and nuclear electric)

Adams, R. B., Alexander, R. A., Chapman, J. M. et al. 2003, Crewed mission to Callisto using advanced plasma propulsion systems, AIAA 2003-4527. https://ntrs.nasa.gov/api/citations/20030065984/downloads/20030065984_Update.pdf . Clean text. Table 8 prints no units; the timelines use mT and the masses read as kg (payload 163,933 against a 40 mT Transhab plus stores and lander).

MTF, D-He3, 180 days at Callisto (Tables 1, 2, 5, 8):
- IML1 (initial mass at Earth-Moon L1) 691,892; payload 163,933; total dry 515,216; main propellant 142,000; RCS propellant 34,674.
- Specific impulse 77,000 s; jet power 2.071 GW; propulsion specific mass 0.0193 kg/kW.
- Accelerations 0.0008 to 0.0013 g.
- Radiators: high temperature 1250 K, 176.5 MW, 1417.5 m²; medium 800 K, 33.9 MW, 1622.7 m².
- Timeline: total about 652 days, outbound about 249 days, 183 days at Callisto, departs 27 Apr 2045. Mission constrained to 650 days (p. 16).
- Crew six (p. 4).

Nuclear electric, MSR-LMR-MPD, piloted (Tables 6, 8):
- Total power 37 MW; jet power 24 MW; specific impulse 8,000 s; 5.36 kg/kW.
- Payload 57,885; dry 459,961; propellant 577,000; IML1 1,072,336 (the difference, 35,375, is taken as RCS and other propellant: derived).
- Flight about 832 days, thrusting about 812, about 120 days at Callisto, total about 1,661 days, departs 19 Nov 2044. Acceleration "0.0001 g" initial and final.

Not found: MTF thrust in newtons, yield and repetition rate (companion paper AIAA 2003-4526, not read).

## Project Longshot (D-He3 inertial fusion probe to Alpha Centauri)

Beals, K. A. et al. 1988, Project Longshot: an unmanned probe to Alpha Centauri, US Naval Academy, NASA CR-184718. https://ntrs.nasa.gov/api/citations/19890007533/downloads/19890007533.pdf . Heavily OCR-noisy.
- Specific impulse "1.02E6 seconds" (sec. 3.2.2.2).
- Weight table "Overall Total 396.414" metric tons; fuel "Total 264.276" (appendix, garbled).
- Thrust: "Thrust/m = 1.838e3 N / 89.345e3 kg" (appendix tank sizing, about p. 71). Read as a thrust of 1,838 N.
- Fission reactor "300 kilowatt"; "a minimum of 250 kW during the in-system phase"; total mass 6,400 kg (sec. 3.1).
- Transit "approximately 100 years"; cruise near 5% of c; a single impulse of 13,500 km/s (sec. 3.2.2.1).
- Lasers: 250 kW input each, 50 kW out at 20%, six of them; 1000 bit/s at full range.
Not found: pellet rate, fusion power, radiators, burn time.

## Orion: the 10 m nuclear pulse vehicle (General Atomic for NASA Marshall)

General Atomic 1964, Nuclear pulse vehicle study, condensed summary report, GA-4891 (contract NAS 8-11053), January 1964. https://www.classe.cornell.edu/~seb/celestia/orion/files/19770085619_1977085619.pdf . Read in full; OCR-noisy.
General Atomic 1964, Nuclear pulse space vehicle study, vol. III, conceptual vehicle designs and operational systems, GA-5009 (Shipps, Nance), 19 Sep 1964. https://www.classe.cornell.edu/~seb/celestia/orion/files/19760065935_1976065935.pdf . Front matter and section 2.1 read; the rest summary only.
Nance, J. C. 1964, Nuclear pulse propulsion, GA-5572 (IEEE 11th Nuclear Science Symposium), DTIC AD0450629. https://apps.dtic.mil/sti/pdfs/AD0450629.pdf .
- Propulsion module "200,000 lb", about 33 ft across (Nance); inert fraction 10 to 20%.
- Specific impulse "1,800 to 2,500 sec" (GA-4891).
- Earth-orbit departure weight 599,000 kg for the orbital-rendezvous Mars mission (GA-4891 Table 1, OCR-read).
- Mars round trip 450 days; "total mission velocity requirement is 72,850 fps" (GA-4891).
- Crew "8, nominally, or up to 20" (GA-4891).
- Start-up "0.3 to 0.5 g", final "3.0g or less" (GA-5009 vol. III sec. 2.1.2).
- Pulse interval 0.80 to 1.5 s (GA-5009 sec. 2.1.6); 900 pulse units stored internally (sec. 2.1.5).
- 20 m module: Jupiter, 192,000 fps, 900 days, 3,150 s (OCR-noisy). 4,000-ton class: 4,000 to 6,000 s.
Not found: pulse unit yield, pusher plate mass, the 10 m module's dry mass (summary only).

## Fission fragment rocket engine

Werka, R., Clark, R., Sheldon, R. and Percy, T. 2012, Concept assessment of a fission fragment rocket engine (FFRE) propelled spacecraft, NIAC phase I final report. https://www.nasa.gov/wp-content/uploads/2019/04/niac_2011_phasei_werka_ffre_tagged.pdf . Part 1 read as text.
- "almost 10 lbf of thrust at a delivered specific impulse of 527,000 seconds" (p. 8).
- "approximately 1000 MW thermal"; "about 700 MW of power is dumped to space" (p. 8); efficiency "about 11%" (p. 7).
- Masses with growth allowance (p. 12): FFRE 113.4 mT, structure 56.4, thermal 64.1, payload 58.0, inert 295.9, nuclear fuel 4.0, hydrazine 3.2, wet 303.1.
- Radiators (p. 11): 120 K, 0.05 MW, 2247 m², 22.9 mT; 500 K, 6 MW, 896 m², 10.7 mT; 1350 K, 699 MW, 1954 m², 29.2 mT; 400 K, 0.3 MW, 109 m², 1.4 mT.
- Mission: six crew to Callisto from Earth-Moon L1; "over 2100 days were needed to reach the orbit of Jupiter".
Check: ½ × 44.5 N × 5,168 km/s = 115 MW, 11.5% of 1000 MW. Part 2 (Mars, 32,000 s with a hydrogen afterburner) summary only, not used.

## Gaseous core engines

McLafferty, G. H. and Bauer, H. E. 1968, Studies of specific nuclear light bulb and open-cycle vortex-stabilized gaseous nuclear rocket engines, United Aircraft report F-910093-37, NASA CR-1030 (number read from a garbled cover). https://ntrs.nasa.gov/api/citations/19680012569/downloads/19680012569.pdf . Results item 1: cavity 500 atm; "Specific impulse -- 1870 sec"; flow 49.3 lb/sec; "Thrust, 92, 000 lb"; "Engine power -- 4600 megw"; "Engine weight -- 70,000 lb".
Rodgers, R. J., Latham, T. S. and Bauer, H. E. 1971, Analytical studies of nuclear light bulb engine radiant heat transfer and performance characteristics, K-910900-10. https://ntrs.nasa.gov/api/citations/19710028762/downloads/19710028762.pdf . Specific impulse 1100 to 3200 s with thrust-to-weight 0.4 to 6.9.
Ragsdale, R. 1991, Open cycle gas core nuclear rockets (NASA conference talk, N92). https://ntrs.nasa.gov/api/citations/19920001890/downloads/19920001890.pdf . Figure 6 baseline for a 5,000 MW reactor: "SPECIFIC IMPULSE 5200 SEC; THRUST 50,000 LB; ENGINE WEIGHT 250,000 LB". Half the engine weight is radiator.
Ragsdale, R. G. and Willis, E. A. 1971, Gas-core rocket reactors: a new look, NASA TM X-67823. https://ntrs.nasa.gov/api/citations/19710014431/downloads/19710014431.pdf . 2500 to 6500 s; 224,000 N chosen; radiator 1100 K at 140 kg per MW; uranium loss one per cent or less of the hydrogen flow.

**Finding:** Ragsdale's baseline does not close its books as printed: ½ × 222,411 N × 51.0 km/s is 5.67 GW of jet from a 5,000 MW reactor. The slide's thrust or power is rounded, or the reactor runs harder than its label. The pack carries the figures as written, and SSE4's balance check says so.

## Electric power and laser sails

Mason, L. S. 2003, A power conversion concept for the Jupiter Icy Moons Orbiter, NASA/TM-2003-212596. https://ntrs.nasa.gov/api/citations/20030105581/downloads/20030105581.pdf : two 100 kWe Brayton units, 496 kWt reactor, 21.7% cycle, 170 m² radiator at 200 K sink, 6800 s ion system. Masses summary only.
Johnson, S. 2003, Project Prometheus (workshop slides), NASA/TM-2003-212598. https://ntrs.nasa.gov/api/citations/20040034798/downloads/20040034798.pdf : "Up to 100 kWe", "specific mass (alpha) 50 kg/kWe" (slide 13).
Parkin, K. L. G. 2018, The Breakthrough Starshot system model, Acta Astronautica 152, 370, arXiv:1805.01306 (v3). https://arxiv.org/pdf/1805.01306 : 200 GW transmitted, 2.7 km optics, 1.06 µm, 4.1 m sail, 3.6 g with a 1 g payload, 0.2 c, 550 s of acceleration, 14,900 g peak, 63 GWh stored, 1.9 GWh kinetic, $8.0B beamer (Tables 1 and 2).

## Wanted, not yet read
- Project Daedalus (Bond, Martin et al. 1978, JBIS supplement): no open primary text found; the pack waits for it rather than using secondary figures.
- NASA DRA 5.0 Addendum (NASA-SP-2009-566-ADD) for the nuclear thermal Mars ship: the nasa.gov PDF refused the fetch.
- EMPIRE (1962 to 1963 Aeronutronic, General Dynamics and Lockheed studies), launch loops (Lofstrom 1985), skyhooks (Moravec 1977).

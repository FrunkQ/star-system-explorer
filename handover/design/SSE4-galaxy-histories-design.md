# SSE4 dark matter and galaxy histories (track GH)

Date 22 Nov 2026. Design thread "Design dark matter and galaxy histories". Status: **accepted by Alex, 22 Nov 2026 13:15** ("I agree with all the other recs"; his own rulings on questions 3 and 7 are written into sections 3.2 and 9.4). Section 14 holds the rulings. Companion to `SSE4-galaxy-design.md` (the cosmos chain C, W, H, Gx, R, P, S, E), `SSE4-night-sky-design.md` (section 6, galaxies in the sky), `SSE4-system-formation-design.md` (the pattern this design copies), `SSE4-matter-pack-design.md` (the master data sheet) and `SSE4-processes-not-classes.md`. Read against `main` at `bca853d` (G0 to G6, G nuclei, Y1 to Y5 and both parts of Y4 merged).

Alex, 22 Nov 12:39: "how can we mass and energy balance galactic filaments and galaxy formation without having dark matter as a theoretical particle that adds mass? ... some galaxies having very large amounts of dark matter ... stripped after collisions ... I was hoping that the Milky Way, once we see it, it looks like a battle-worn galaxy, not a pristine spiral ... we should be resolving galaxies a bit like planets ... patches of ancient stars ... patches of pristine space to form population 3 stars ... We need globular clusters. We need micro galaxies. We need dwarf galaxies. We need the ability for these galaxies to be orbiting the Milky Way ... this is just a repetition of work done before, but with just a slightly different lens on the front. How do you feel about that?" And 12:40: "dark matter should be listed as theoretical and have tweakable parameters like any other universal constant". And 12:47: "the galaxy effectively has its own accretion model". Yes: section 7 is that model, and it is the system accretion model's record and rules one level up.

---

## 1. Short answers

1. **Dark matter is already in the sums, but it has no name and it stops at the galaxy's edge.** The cosmology pack carries Ωm 0.3111 and Ωb 0.04897, so 84% of all matter in the web and in every halo is already dark. What is missing: it is not on the data sheet, nobody can see or change it, a galaxy does not keep a dark matter amount of its own, it cannot be stripped, and inside the galaxy it does nothing (the rotation speed is a typed 220 km/s). Section 3 puts it on the sheet as **theoretical**, section 4 gives every region and every galaxy a matter ledger, and section 5 makes it act inside the galaxy.
2. **Dark energy needs no frig.** It is already in the expansion and in the growth of structure, which is the honest place for it. It does not push on voids in particular. Voids grow because matter drains out of them, and dark energy slows that draining across the whole universe. What is missing is that the foam map is drawn in co-moving units, so nobody sees the universe getting bigger. Section 3.4 adds that as a switch, and two rows on the sheet.
3. **"Formation again through a different lens": mostly yes.** About two thirds of this design is reuse (section 11). The record, the pins, the fidelity ladder, "clouds until looked at", the budgets, the events on the ruler and the detail levels all carry over from system formation almost word for word. Three things are different in kind and need new blocks: galaxies are made of galaxies (so the history is a tree and the mechanism calls itself), stars and dark matter pass through each other (so the leftovers are orbits, not bodies), and the building has not finished (two galaxies are being eaten by ours today).
4. **One mechanism does all of it.** Every galaxy has one **assembly tree**: the list of smaller haloes it swallowed, each with its own small galaxy grown by the code we already have. Each swallowed galaxy then runs the same four processes (sinking, tidal stripping, gas stripping, heating). Satellites, dwarfs, ultra-faint dwarfs, streams, shells, ancient patches, accreted globular clusters, a thick disc, galaxies that lost their dark matter and galaxies that are nearly all dark matter are what that mechanism leaves behind. None is a special case, and each name is a label read off the result.
5. **It fits a browser.** No particle simulation. A Milky Way history at chart level is about 55 swallowed haloes and costs tens of milliseconds on top of what Y4 already does. The full survey (about 2,000 haloes down to the smallest that can hold stars) is a second or two, once, and then stored (estimates, to be measured in GH0). What a browser cannot do is listed in section 10 with what stands in for it.
6. **"Battle-worn" comes in two kinds.** Damage to the disc is bright and shows in true colour: the warp, ripples, driven arms, a punched ring, bursts, companions (section 9.4). The remains of old meals out in the halo are faint light, odd orbits and odd chemistry, and real images show them only at very long exposure. So Chart style inks those, and True style shows them when the exposure is deepened (section 9.3).

---

## 2. What the video and its sources say

The link is Anton Petrov, "Gaia Data Rewrites Everything We Knew About Galaxy History" (19 Aug 2026, 13 min). I could read its description, chapter list and sources. I could **not** read the spoken transcript, so the summary below comes from the chapters and the four sources, not from his words.

Chapters: Gaia data changed everything; what we thought before; the disc started to form earlier than thought; the Gaia Sausage collision was a lot more complex; Loki leftovers; hot and cold sides of the galaxy; overall history of the Milky Way.

| Source | What it gives the design |
|---|---|
| Belokurov, The Conversation, 10 Jun 2026 (doi:10.64628/AB.vujjdh5vc) | Stars not born here ("migrants") have plunging orbits and dwarf-galaxy chemistry. The largest ancient remnant is Gaia-Sausage-Enceladus, 8 to 11 Gyr ago. That collision splashed old disc stars into the halo, brought star clusters, and tilted the disc against the dark halo, which is itself stretched and still leaning. Nothing big since, which is unusual. The Large Magellanic Cloud is now pulling the galaxy about, and will merge |
| Xiang and Rix 2022, Nature (doi:10.1038/s41586-022-04496-5), MPG summary | 250,000 dated stars. The thick disc began forming 13 Gyr ago, 0.8 Gyr after the Big Bang. A burst of star formation 11 Gyr ago goes with the Gaia-Enceladus merger. The thin disc from about 8 Gyr ago. Stars born at one time share one metal content, so the gas was well mixed |
| Sestito et al. 2026, MNRAS 548, stag563 (doi:10.1093/mnras/stag563) | 20 very metal-poor stars within 2 kpc of the Sun, all orbiting in the plane, 11 forwards and 9 backwards, with dwarf-galaxy chemistry. Read as one small galaxy swallowed very early, tentatively named Loki. The authors say several events may have contributed |
| Oprea et al. 2026, MNRAS (doi:10.1093/mnras/stag319), RAS summary | The hot gas halo is up to 12% warmer in the south. The Large Magellanic Cloud pulls the disc towards it at about 40 km/s, squeezing the gas on that side. The difference grew in the last 100 Myr |

What this tells the design: a galaxy history is a short list of dated collisions, each leaving stars with their own orbits and chemistry, a changed disc, a changed dark halo and changed gas. That is the record in section 6.

---

## 3. Dark matter and dark energy on the data sheet

### 3.1 A fourth mark: theoretical

The sheet has three marks today: real, authored (real value beside it) and fictional. Dark matter and dark energy fit none of them. Their amounts are measured to about 1%, by gravity alone, and what they are is unknown. So a fourth mark, **theoretical**: "the amount is measured through gravity; no particle or field has been detected". It shows on the row at every depth, with one line saying what would change the picture (a detection, or a theory of gravity that needs none). It is not stamped on every galaxy. At Science and Engine depth the "Why?" of anything that reads a theoretical row says "rests on dark matter", with a link.

The star census already uses observed, inferred and hypothetical for kinds of star. Theoretical is the sheet's word for the same middle rung: strongly inferred, never seen directly.

### 3.2 The rows

A new page group on Constants, **Cosmos**, beside Nature and Reference objects. The measured parameters of the universe move here from `packs/base-cosmology` (they are facts about nature, so by the matter pack's own rule they belong on the sheet). Tuned model numbers stay in `base-cosmology`. With nothing edited, every hash stays the same (the MP1c test).

| Row | Default | Mark | Source |
|---|---|---|---|
| Hubble constant H0 | 67.66 km/s/Mpc | real | Planck 2018 VI, table 2 (doi:10.1051/0004-6361/201833910) |
| Ordinary matter, Ωb | 0.04897 | real | same |
| **Dark matter, Ωc** | 0.2621 (Ωc h² 0.1200 ± 0.0012) | **theoretical** | same |
| All matter, Ωm | worked out: Ωb + Ωc + neutrinos | | never typed |
| Dark matter to ordinary matter | worked out: 5.35 to 1 | | |
| Cosmic baryon share f_b | worked out: 0.157 | | the number every galaxy's gas supply reads |
| Fluctuation size σ8, tilt n_s | 0.8102, 0.9665 | real | same |
| Background temperature, neutrino count | 2.7255 K, 3.046 | real | Fixsen 2009; Mangano et al. 2005 |
| **Dark energy, ΩΛ** | worked out for a flat universe: 1 − Ωm − Ωr = 0.689 | **theoretical** | |
| **Dark energy pressure, w0 and wa** | −1 and 0 (a cosmological constant) | **theoretical** | form: Chevallier and Polarski 2001 (doi:10.1142/S0218271801000822); Linder 2003 (doi:10.1103/PhysRevLett.90.091301) |
| Dark energy density | worked out: 5.3 × 10⁻¹⁰ J/m³ | | |

**What dark matter is** (theoretical rows, the ones to play with). Three ideas are in use besides plain cold dark matter. All of them change only two things a galaxy can feel, so the kernel reads two worked-out numbers and nothing else. That keeps it one mechanism.

| Row | Default | What it changes | Limits today |
|---|---|---|---|
| Particle "warmth": thermal relic mass | cold (no cut-off) | a **smallest halo size**: below the half-mode mass, haloes are rare (Schneider et al. 2012, doi:10.1111/j.1365-2966.2012.21252.x). Fewer ultra-faint dwarfs, later first stars | heavier than about 5 to 6.5 keV (Iršič et al. 2017, doi:10.1103/PhysRevD.96.023522; Nadler et al. 2021, doi:10.1103/PhysRevLett.126.091101) |
| Self-interaction, cross-section per mass | 0 | a **core size**: the centre of a halo is flattened where particles have scattered about once (Rocha et al. 2013, doi:10.1093/mnras/sts514). Cored dwarfs strip more easily. In a collision the dark matter lags a little | under about 0.5 cm²/g in clusters (Harvey et al. 2015, doi:10.1126/science.1261381; Randall et al. 2008, doi:10.1086/587859); dwarfs allow more if it falls with speed (Kaplinghat, Tulin and Yu 2016, doi:10.1103/PhysRevLett.116.041302) |
| "Fuzzy" (ultra-light) particle mass | off | both: a cut-off and a solid core (Hui et al. 2017, doi:10.1103/PhysRevD.95.043541) | later; listed, not wired |
| Worked out: half-mode mass, core scale | 0, 0 | the two numbers G2 and DM4 read | |

Not on the sheet: annihilation and decay (nothing in the model reads them yet), and the local density near the Sun, which is a result to check (0.3 to 0.5 GeV/cm³; Read 2014, doi:10.1088/0954-3899/41/6/063101), not an input.

**The particle route, stated on the page (Alex, 22 Nov 13:10: "we dont even entertain MOND AND say we don't ... say we are going the particle route due to recent research").** SSE4 treats dark matter as matter: an unseen particle that adds mass. It does not offer gravity that changes at low acceleration (MOND and its relatives) as a mode, and the Cosmos page says so in two lines with the reasons:
- The web, the count of haloes and the growth of galaxies here are built on dark matter, and no agreed theory of altered gravity replaces all three. It has no accepted account of colliding clusters, where the mass sits apart from the gas (Clowe et al. 2006), and the whole class of theories in which gravity waves travel at a different speed from light was ruled out by the neutron-star merger of 2017 (Boran et al. 2018, doi:10.1103/PhysRevD.97.041501).
- The standard picture keeps passing new tests: counts of 16,000 galaxy clusters with lensing and clustering agree with the cosmic background's parameters (DES Collaboration 2025, Phys. Rev. D 112, 083535, doi:10.1103/3dzh-d8f5).
- Gravity's strength has not changed over time where it can be tested: neutron-star masses from a merger bound the change in G since then (Vijaykumar, Kapadia and Ajith 2021, doi:10.1103/PhysRevLett.126.141104), and red dwarfs, which would remember a jump in G, rule out the recent 4% and 11% steps proposed to ease the Hubble tension at 3.1 and 4.0 sigma (Sakstein and Desmond 2026, arXiv:2610.04048).

To be exact about what those last two show: they test G changing with **time**, which is a different idea from MOND (a different force law at low acceleration). They close a neighbouring door; the first bullet is what closes MOND's.

The one firm observation behind MOND is kept as a check: the tight link between ordinary matter and rotation speed (McGaugh, Lelli and Schombert 2016, doi:10.1103/PhysRevLett.117.201101), which DM4 must reproduce with haloes.

**The bone: constants with a history (package DM6).** A general mechanism, not a gravity special. Any row on the Constants page may carry a **time law** as well as a value: constant (the default for all), a steady drift (a rate per year), or a step at a date. G is the first to use it. Marked authored or fictional, never real, because the measured drift of G is zero within about one part in 10¹³ a year (lunar laser ranging; source to be checked in GH0). Reach, by the usual three kinds:
- **Follows:** every block that takes G as an input and is asked at a date now reads G at that date: orbits, periods, surface gravity, tides, Hill spheres, air scale height, the expansion rate.
- **Estimated:** stars, by the same homology scaling the sheet already uses for a changed G (brightness rises steeply with G), applied along the star's life. This is exactly the "memory" the red dwarf test looks for, so the sheet can show it.
- **Cannot follow:** how systems and galaxies formed, yields, and anything tuned to our universe.
So a GM can run "gravity was 10% stronger a billion years ago" as a what-if and watch orbits widen, years lengthen and stars dim to today's values. It needs the constants handle to take a date, which is a schema change in `sse-reference`; it is not free, and it is small.

### 3.3 Reach of a change

The cosmos chain is computed, not tabulated, so a changed Ωc really does follow a long way. This is better than changing G.

| Reach | Blocks |
|---|---|
| **Follows** | expansion and ages, growth of structure, the size of fluctuations at each mass, the web, halo counts per cell, halo growth histories, the baryon share of every galaxy's gas supply, halo profiles and rotation curves, the smallest-halo cut-off, the core size |
| **Estimated** (cited fit made in our universe, used unchanged, stamped) | the sub-halo counts and their sinking times (Giocoli et al. 2008; Boylan-Kolchin et al. 2008), the merger rate fit (Fakhouri et al. 2010), tidal stripping tracks, concentration against mass |
| **Cannot follow** | the galaxy regulator's tuned efficiencies (winds, depletion time, hot-halo cut), yields, anything tuned to the observed galaxy population |

### 3.4 Dark energy and the voids

What is true, and already built: `Cosmology` is flat with a cosmological constant. It sets how fast the universe expands and, through the growth factor, how fast matter can still gather. About 4 Gyr ago it began to win, and structure growth has been slowing since. In the far future it freezes the web and isolates each bound group (Nagamine and Loeb 2003, doi:10.1016/S1384-1076(03)00072-2; Busha et al. 2003, doi:10.1086/378043). That is G8's material.

What is true about voids: a void expands faster than the average, by roughly 10 to 15% for a deep one, because there is too little matter inside to slow it. Matter moves outward to the walls. Small voids merge into larger ones (Sheth and van de Weygaert 2004, doi:10.1111/j.1365-2966.2004.07661.x). The web's one Zel'dovich step already does this draining. Dark energy has the same density inside a void as anywhere else. Its part is that a void is the first place where it outweighs matter.

What is missing, and proposed (package DM2):
- The two pressure rows w0 and wa in the expansion rate, with the growth factor found by stepping its small differential equation when w is not −1 (cheap, and it then follows any change). Evolving dark energy is an open question: the DESI survey's 2024 and 2025 results lean that way at about 3 to 4 sigma with other data (DESI 2024 VI, doi:10.1088/1475-7516/2025/02/021; DR2, arXiv:2503.14738). The default stays −1 and 0.
- A **"real size"** switch on the foam map and cluster map: the same slice drawn in physical megaparsecs, so scrubbing time shows the universe growing, voids opening and, in the future, neighbours leaving. Today's map is co-moving only (V9 log, honest limits).
- A check: the density profile and outflow speed of the web's voids against the measured universal void profile (Hamaus, Sutter and Wandelt 2014, doi:10.1103/PhysRevLett.112.251302).
- Expanding the foam by hand would be a fudge, and a worse one than what is there. Nothing new goes on the register for dark energy. The lognormal density and the single Zel'dovich step are already on it.

---

## 4. The matter and energy ledger

Formation keeps budgets (disc gas, solids, angular momentum) and nothing forms from nowhere. The same rule, one level up.

### 4.1 Matter, per web cell (package DM3)

```
matter in the cell (from the web's density)
  dark matter      = in catalogued haloes  +  in haloes below the catalogue floor  +  never collapsed (diffuse)
  ordinary matter  = stars  +  cold gas  +  hot halo gas  +  thrown-out gas  +  black holes and remnants
                     +  gas between galaxies that never fell in
```

Each term is already computed somewhere (the web, G2's counts and their integral below the floor, G3's reservoirs). The ledger adds them up, and a test closes it: the parts sum to the cell's matter, and ordinary matter over all matter is f_b within the tolerance set by what galaxies threw out.

Checks: the share of mass and volume by environment (filaments hold about half the mass in about 6% of the volume, voids about 15% of the mass in about 77% of the volume; Cautun et al. 2014, doi:10.1093/mnras/stu768); the census of ordinary matter today, where stars are under a tenth and most is thin gas between galaxies (Fukugita and Peebles 2004, doi:10.1086/425155; Macquart et al. 2020, doi:10.1038/s41586-020-2300-2).

On screen: the foam and cluster maps get a "where the matter is" bar, and a dark matter layer.

### 4.2 Matter, per galaxy

Every halo carries, at every stored time: dark matter bound to it, dark matter it has lost to its host, and its ordinary matter by reservoir. Its **dark share** inside the visible galaxy and overall is then a plain number that the class atlas and the cards can read. A galaxy with very much or very little dark matter is the two ends of that number (section 7.4).

### 4.3 Energy

Alex asked for energy as well as mass. Three places where the galaxy code balances forces by a tuned curve today, and where an energy sum can replace the curve over time:

| Where | Today | The balance |
|---|---|---|
| Does infalling gas stay hot? | 1/(1 + (M/5×10¹¹)²), tuned | gas falling into a halo is shock-heated to the halo's virial temperature (from its depth); it feeds the galaxy only if it can cool within the halo's age (White and Frenk 1991) |
| How much gas do supernovae throw out? | (v/200)⁻², tuned | supernova energy per mass of stars formed against the binding energy per mass of gas, both known once the halo profile is known (DM4) |
| What does a merger do? | a burst and a bulge share by mass ratio | each merger has an orbital energy at infall. It goes to: the host's dark matter (the drag that makes the satellite sink), heating of the disc (which thickens it), unbinding of the satellite, and the gas. Section 7.5 uses these shares |

The first two are not rebuilt by this design. They are listed so the galaxy thread can move them when DM4 lands. The third is new here.

One limit to say plainly on the Cosmos page: energy is conserved for a bound object, not for the expanding universe as a whole. So the ledger is mass per co-moving region and energy per bound system.

---

## 5. Dark matter inside the galaxy (package DM4)

One block, `sse.cosmos.halo_structure`, gives each halo a density profile, and one block, `sse.cosmos.rotation_curve`, gives the circular speed at any radius from dark matter, disc, bulge and gas together.

- Profile: Navarro, Frenk and White (1997, doi:10.1086/304888), with the concentration from the halo's own mass and growth history (Ludlow et al. 2016, doi:10.1093/mnras/stw1046; scatter about 0.11 dex, Dutton and Macciò 2014, doi:10.1093/mnras/stu742). An early-forming halo is denser in the middle.
- Response to the galaxy, as two processes with strengths: ordinary matter settling in the middle pulls dark matter inward (Blumenthal et al. 1986, doi:10.1086/163867); bursts of star formation in a dwarf shake it outward into a core, most strongly where stars are about half a percent of the halo (Di Cintio et al. 2014, doi:10.1093/mnras/stt1891). Plus the sheet's core size if self-interaction is on.
- Everything that types a speed today reads the curve instead: G5's cell rotation (26.83 km/s/kpc), `origin.rs` (220 km/s), the bar test's unexplained × 1.1, encounter speeds, and later the widths of spectral lines.

Checks: the Milky Way's speed at the Sun 229 km/s, falling gently outward (Eilers et al. 2019, doi:10.3847/1538-4357/aaf648); local dark matter density 0.3 to 0.5 GeV/cm³; ordinary matter against flat rotation speed over five decades of mass (Lelli, McGaugh and Schombert 2016, doi:10.3847/2041-8205/816/1/L14).

On screen: a rotation curve card on the galaxy chart, with the dark and visible parts drawn apart. This is the classic picture of why dark matter was proposed, and it will be live.

---

## 6. A galaxy is a history: the record

Formation's principles carry over as they stand: generation is a timeline; one generator, parameterised by pins; layered rules, not simulation; budgets are conserved; provenance on everything; uncertainty is shown; clouds until looked at.

```
AssemblyHistory { galaxy address, seed, manifest ref,
  budgets:    dark matter, gas, stars, metals by group, black holes     with what claimed each part
  tree:       [{ progenitor address (itself a halo, with its own AssemblyHistory when looked at),
                 infall time, mass at infall, orbit at infall, came in with (group infall) }]
  bodies:     per progenitor, its fate so far:
                bound part (dark, stars, gas), sunk depth, orbit slices,
                events: [infall, each closest pass, half its dark matter lost, first stars torn off,
                         gas lost, stopped forming stars, broke up, reached the centre, collided with ...]
  components: what each progenitor left in the host (section 7.6): stars with their own ages and metals,
              an orbit family, a mixing state; globular clusters; gas delivered; a black hole
  host:       responses, dated: disc thickened, bulge grew, burst, bar formed or broke, disc tilted,
              arms disturbed, halo stretched
  slices:     stored times; the ruler interpolates, never recomputes
  evidence:   per value: calculated | tabulated | drawn (with spread) | authored | reconstructed
  level:      per part: sketch | standard | detailed, and which block }
```

All events are record events on the one universe clock, so the story sifter can find "the dwarf that passed through the disc 300 Myr before this world's extinction", and look-ahead can show the Large Magellanic Cloud's end.

---

## 7. The mechanism

### 7.1 One assembly tree (package GH1)

Today two separate draws describe what a halo swallowed: G2's major mergers (a Poisson process on the Fakhouri fit) and Y4 part 2's sub-haloes (the unevolved sub-halo mass function of Giocoli et al. 2008). They can count the same object twice, and neither knows about the other.

Proposed: one list. The **major mergers are pins** that claim their share of the accreted mass first (the house rule for pins), and the sub-halo mass function fills the rest below them, bin by bin, with the same keyed counts Y4 uses. So G2's merger events and Y4's satellites above the old floor stay as they are wherever the budget allows, and the lower bins are new.

- **Recursive.** A progenitor is a halo. Asked for its own tree, the same function answers, keyed by its address. So a large satellite brings its own satellites, which is what the Large Magellanic Cloud did (Kallivayalil et al. 2018, doi:10.3847/1538-4357/aadfee). Depth is only ever opened by looking.
- **Levels by mass floor.** Glance: summary numbers only. Chart: progenitors above 10⁻³ of the host (today's Y4 floor, about 55 for our galaxy). Survey: down to 10⁻⁵ (about 2,000, reaching the smallest haloes that could ever cool gas). Lower bins are drawn with their own keys, so going deeper adds members and never changes one already seen.
- **The fuller method** (a detailed block, later): a proper merger tree by the published algorithm of Parkinson, Cole and Helly (2008, doi:10.1111/j.1365-2966.2007.12517.x), written fresh. It gives every branch's own branches from the cosmology directly and follows a changed Ωc with no fit in the way.

### 7.2 Each progenitor's own galaxy

Already built: Y4 part 2 grows each satellite's galaxy with G3 on a halo that had its mass at infall. That run gives stars, gas, metals, a star formation history and an **age to metallicity track**. The track is what makes one swallowed galaxy's stars and clusters tell apart from another's. It is also how the real family tree was worked out (Kruijssen et al. 2020, doi:10.1093/mnras/staa2452; Aguado-Agelet et al. 2026, arXiv:2608.27611).

Cost is about 2 ms a galaxy (Y4 log). For the unseen thousands, the Sketch rung gives an envelope instead (stellar mass from halo mass with its scatter, metals from stellar mass; Kirby et al. 2013, doi:10.1088/0004-637X/779/2/102), shown as "about", and looking fixes it once.

Small haloes after reionisation cannot gather gas (built, in `epochs` and G3). Those that formed a few stars first are the **fossils**: the ultra-faint dwarfs, nearly all dark matter (Bovill and Ricotti 2009, doi:10.1088/0004-637X/693/2/1859; Brown et al. 2014, doi:10.1088/0004-637X/796/2/91). Those that formed none are **dark haloes**: real objects in the tree with an address and no light.

### 7.3 What happens after infall: four processes, no gates (package GH2)

Every progenitor runs all four, each with its own strength from its governing ratio. Fate is a label read afterwards.

| Process | Governing ratio | What it does | Source |
|---|---|---|---|
| **Sinking** (dynamical friction) | drag time over time since infall; strong only above a few percent of the host | the orbit shrinks; big ones reach the centre | Chandrasekhar 1943 (doi:10.1086/144517); times as Boylan-Kolchin et al. 2008 (used today) |
| **Tidal stripping** | host's tidal pull over the satellite's own gravity, at each closest pass | dark matter goes first, from the outside in. Stars start to leave only after about nine tenths of the dark matter has gone. A cored satellite strips much faster | Peñarrubia et al. 2008 (doi:10.1086/523686) and 2010 (doi:10.1111/j.1365-2966.2010.16762.x); Errani and Navarro 2021 (doi:10.1093/mnras/stab1215); Green, van den Bosch and Jiang 2021 (doi:10.1093/mnras/stab696) |
| **Gas stripping** | ram pressure of the host's hot gas over the satellite's grip on its own gas | gas is lost and star formation stops. Replaces today's fixed 3 Gyr delay, so small ones stop fast and big ones keep going, as observed | Gunn and Gott 1972 (doi:10.1086/151605); Fillingham et al. 2015 (doi:10.1093/mnras/stv2058) |
| **Shock heating** | energy of each fast pass through the disc or centre over binding energy | puffs the satellite up, which feeds stripping; the disc destroys the ones that pass close | Garrison-Kimmel et al. 2017 (doi:10.1093/mnras/stx1710) |

**Orbits.** Each progenitor is one test body moving in the host's smooth, slowly growing potential from DM4 (dark halo, disc, bulge), with the drag above. A fixed-step leapfrog in the kernel's pinned maths, about 14,000 steps a body for all of history. This is the method of the published satellite generator SatGen (Jiang et al. 2021, doi:10.1093/mnras/staa4034), written fresh from the paper. It is not a particle simulation: the bodies do not pull on each other, and there are tens to thousands of them, not millions. Positions are stored as slices, so "satellites orbiting the Milky Way" play forward and back on the ruler without recomputing, and look-ahead simply continues the same run.

**Labels read off** (class atlas rules, pack-editable): satellite (bound, hardly touched), being torn apart (bound core with a stream, like Sagittarius), stream, shell, mixed remains, merged. A thing can be two at once.

Checks: the count of our galaxy's satellites by brightness (about 60 known, about 220 expected in all; Drlica-Wagner et al. 2020, doi:10.3847/1538-4357/ab7eb9; Nadler et al. 2020, doi:10.3847/1538-4357/ab846a); how they are spread in distance; nearly all within the halo have stopped forming stars except the largest (Wetzel, Tollerud and Weisz 2015, doi:10.1088/2041-8205/808/1/L27); metals against stellar mass (Kirby et al. 2013).

### 7.4 Very little and very much dark matter

Nothing is added for these. They are where the processes above take a galaxy, plus two rare events in 7.7.

| What is seen | How it arises here | Grade |
|---|---|---|
| Ultra-faint dwarfs, mass-to-light in the hundreds or thousands | fossils of 7.2 (Simon 2019, doi:10.1146/annurev-astro-091918-104453) | well supported |
| Dark haloes, and starless gas clouds held by them | haloes below the gas threshold (Benítez-Llambay et al. 2017, doi:10.1093/mnras/stw2982) | predicted; candidates only |
| Dwarfs with far too little dark matter, by stripping | a cored satellite on a tight, plunging orbit loses its dark matter before its stars (Ogiya 2018, doi:10.1093/mnrasl/sly138; Jackson et al. 2021, doi:10.1093/mnras/stab093; Moreno et al. 2022, doi:10.1038/s41550-021-01598-4, who find about a third of Milky-Way-mass hosts have one) | well supported in simulations |
| NGC 1052-DF2 and DF4, almost none, with odd bright clusters | either stripping (Montes et al. 2020, doi:10.3847/1538-4357/abc340) or a head-on collision of two gas-rich dwarfs (7.7) | **argued** |
| Tidal dwarf galaxies, none by construction | made from a tail of a big merger (7.7) | well supported |
| Scatter in dark share among ordinary galaxies | scatter in concentration, in formation time and in how many stars a halo made | settled |

The first observations: van Dokkum et al. 2018 (doi:10.1038/nature25767) and 2019 (doi:10.3847/2041-8213/ab0d92). Gas-rich dwarfs that rotate too slowly for their mass may be a third route and are open (Mancera Piña et al. 2022, doi:10.1093/mnras/stab3491).

### 7.5 What a merger does to the host: by ratios, not by type (package GH3)

"Especially if those galaxies are of different types": the outcome does differ, and it is the physical ratios behind the types that decide it. Each row is a process with a strength. The inputs are the mass ratio, each side's gas share, the orbit (how head-on, how tilted to the disc, with or against its spin) and what the host is like then.

| Process | Strongest when | Leaves behind |
|---|---|---|
| Disc heating | heavy satellite, tilted orbit | a thick disc; old disc stars "splashed" into the halo (Toth and Ostriker 1992, doi:10.1086/171185; Belokurov et al. 2020, doi:10.1093/mnras/staa876) |
| Bulge growth | high mass ratio, little gas | a share of disc stars about equal to the mass ratio moves to the bulge, less when gas-rich because the disc regrows (Hopkins et al. 2009, doi:10.1088/0004-637X/691/2/1168) |
| Starburst and feeding the nucleus | gas-rich, close pass | a dated burst, clusters born (7.8), an active nucleus (built in G3 and G nuclei). Each close pass of a surviving satellite can do it again (Ruiz-Lara et al. 2020, doi:10.1038/s41550-020-1097-0) |
| Fresh gas | gas-rich, metal-poor satellite | the rings' gas is diluted and a second, lower-metal sequence of stars begins: the two families of disc stars seen near the Sun (Spitoni et al. 2019, doi:10.1051/0004-6361/201834188) |
| Tilt and stretch | massive satellite, any gas | disc tilted against the dark halo; halo stretched; a warp; the whole galaxy pulled off-centre (Conroy et al. 2021, doi:10.1038/s41586-021-03385-7; Han et al. 2023, doi:10.1038/s41550-023-02076-9) |
| Arms and bar | a close pass | driven arms, ripples through the disc, a bar started or weakened (Antoja et al. 2018, doi:10.1038/s41586-018-0510-7; Laporte et al. 2018, doi:10.1093/mnras/sty1574) |
| Hot gas | a massive companion | one side squeezed and warmer (Oprea et al. 2026) |
| Debris shape | head-on orbit gives **shells**; round orbit gives **streams**; backwards orbit gives counter-rotating stars; over the pole and gas-rich gives a **polar ring**; fast and straight through the disc gives a short-lived **ring galaxy** | section 7.6 (Hendel and Johnston 2015, doi:10.1093/mnras/stv2035; Lynds and Toomre 1976, doi:10.1086/154730) |
| Two spheroids, no gas | equal masses | a larger, rounder, slower-turning galaxy with shells and a scoured centre |

With this in, G3's stand-ins go: ex-situ stars as "ratio squared of the host's" and holes as a fixed thousandth of arriving stars (both on the register with exactly this replacement named). The stars, gas and hole that arrive are the progenitor's own.

### 7.6 Remains as places: streams, shells and ancient patches (package GH4)

Each progenitor that lost stars leaves a **component** in the host: a stellar mass, that galaxy's own spread of ages and metals, an orbit family, and a mixing state. A component is a density written as a formula, never a set of particles.

- **Stream** (young or small): a tube along the progenitor's stored orbit, ahead and behind. Its length grows in proportion to time since stripping began, and its width and spread in speed scale with the cube root of the mass ratio (Johnston 1998, doi:10.1086/305273). So a globular cluster gives a hair-thin stream and a dwarf galaxy a broad one.
- **Shells**: for head-on orbits, stars pile up at the turning points.
- **Mixed remains** (old or large): once the stream has wrapped many times it is a smooth cloud between the orbit's nearest and farthest distances, still marked by its motion and its chemistry. Gaia-Sausage-Enceladus is this.

The patch envelope gains one line: "plus every component that overlaps this patch". Then everything already built picks it up with no new mechanism:
- **Patches of ancient stars.** A G5 cell inside a component draws some of its fill stars from that component's ages, metals and motions. The star's card says where it was born: "in a dwarf galaxy that fell in 9.6 Gyr ago and was torn apart". `reconstruct_origin` reaches across galaxies.
- **Systems.** Formation inherits the patch as always, so a star from a swallowed dwarf gets that dwarf's low metals, few giant planets and little uranium.
- **The sky.** The count tree and the glow map include the components, so a stream is there in a deep enough exposure.
- **Hazards.** A patch inside a dense stream or a cluster has its own encounter rate.

Checks: the stellar halo's mass and metals rise with the mass of the biggest thing swallowed (Harmsen et al. 2017, doi:10.1093/mnras/stw2992; D'Souza and Bell 2018, doi:10.1093/mnras/stx3081); one or two progenitors supply most of it (Deason, Mao and Wechsler 2016, doi:10.3847/0004-637X/821/1/5); the halo's density falls off faster beyond the big progenitor's farthest reach (Deason et al. 2018, doi:10.3847/2041-8213/aad0ee); near the Sun about one star in a few hundred belongs to the halo.

### 7.7 Rare events that collisions bring (package GH6)

Each is a dated event with a cause, drawn from a rate the model already has, and each makes an object.

- **Two satellites collide.** The rate comes from the same block that gives stellar encounters (number density, cross-section, speed), fed with the satellites of one host. If both are gas-rich and the speed is several times their escape speed, the gas stops at the meeting point while both dark haloes and both sets of stars carry on. The gas makes new galaxies with no dark matter and unusually massive clusters, in a line (Silk 2019, doi:10.1093/mnrasl/slz090; Shin et al. 2020, doi:10.3847/1538-4357/aba434; Lee et al. 2021, doi:10.3847/2041-8213/ac16e0; van Dokkum et al. 2022, doi:10.1038/s41586-022-04665-6). Rare. **Argued.**
- **Tidal dwarf galaxies.** A gas-rich major merger on a forward orbit throws out long tails. Clumps in them become small galaxies with no dark matter. Most fall back within a few Gyr (Bournaud and Duc 2006, doi:10.1051/0004-6361:20065248).
- **Stripped nuclei.** A dwarf's central star cluster outlives the dwarf and passes for a very large globular cluster.
- **Pristine pockets and metal-free stars.** A halo that never formed a star holds gas with exactly no metals (7.2). If it is squeezed, by falling into a host or by a collision, it can make a short-lived cluster of metal-free stars long after the first ones. This was common in the first 2 Gyr, is predicted at a low rate later (Liu and Bromm 2020, doi:10.1093/mnras/staa2143), and has candidates at high redshift (Vanzella et al. 2023, doi:10.1051/0004-6361/202346981) and metal-free gas clouds seen 2 Gyr after the Big Bang (Fumagalli, O'Meara and Prochaska 2011, doi:10.1126/science.1213581). Inside a galaxy like ours today, truly pristine gas is not expected: the most metal-poor gas known nearby still has a few percent of the Sun's metals. So the pockets live in dark haloes on the outskirts and in voids. Marked plausible in the early universe and **speculative** today.

### 7.8 Globular clusters, and every other star cluster (package GH5)

One mechanism for all clusters. G5 already has stars born in clusters with a mass function. A globular cluster is the top end of that function, made where gas is dense and under high pressure (early discs, bursts, mergers), which then survived.

- **Birth.** Each galaxy, and each progenitor before it fell in, forms massive clusters at a rate set by its star formation and gas density (Kruijssen 2015, doi:10.1093/mnras/stv2026; Reina-Campos and Kruijssen 2017, doi:10.1093/mnras/stx790). Above about 10⁵ solar masses there are few enough to draw one by one, each with an address, a birth time and the metals of its galaxy at that time.
- **Loss.** Each loses stars by evaporation and by shocks, faster when small and when deep in the host (Baumgardt and Makino 2003, doi:10.1046/j.1365-8711.2003.06286.x; Lamers et al. 2005, doi:10.1051/0004-6361:20042241). A dissolved cluster leaves a thin stream.
- **Arrival.** A swallowed galaxy's clusters come with it and take up orbits in its component.

Checks: total cluster mass is a fixed share of halo mass, about one cluster per 5 × 10⁹ solar masses (Harris, Blakeslee and Harris 2017, doi:10.3847/1538-4357/836/1/67; Burkert and Forbes 2020, doi:10.3847/1538-3881/ab5b0e); about 160 for a galaxy like ours, with roughly four in ten brought in (Massari, Koppelman and Helmi 2019, doi:10.1051/0004-6361/201936135); two colour families; the usual peak near 2 × 10⁵ solar masses; two branches of age against metals (Forbes and Bridges 2010, doi:10.1111/j.1365-2966.2010.16373.x).

A cluster is a place. It resolves into stars through G5 like any dense patch: a hundred thousand old, metal-poor stars within a few parsecs, a sky full of bright neighbours, close passes every few million years, hardly any planets with rock. Open clusters, nuclear clusters and ultra-compact dwarfs are the same objects under other labels. So is the newest puzzle, objects of a few dozen solar masses that may be the smallest galaxies or dying clusters (Smith et al. 2024, doi:10.3847/1538-4357/ad0d9f). In this model they differ by one fact: whether a dark halo is there.

---

## 8. The Milky Way, and any authored galaxy (package GH7)

This is the parked "Milky Way mergers" question. The home galaxy's history is **pins**: the known events are authored into the tree with their evidence grade, they claim their budget first, and the mechanism fills in the dozens of small unknown ones around them. Each pin is editable, and "what if this never happened" is a what-if branch.

| When (Gyr ago) | Event | Stars brought | Grade |
|---|---|---|---|
| 13 to 12.5 | first, disordered stars; then the disc spins up and the thick disc forms | (born here) | well supported (Belokurov and Kravtsov 2022, doi:10.1093/mnras/stac1267; Xiang and Rix 2022) |
| about 11 or more | Kraken (also Heracles), near the centre; perhaps the largest by mass ratio | 2 to 5 × 10⁸ | argued: may be born here (Kruijssen et al. 2020; Horta et al. 2021, doi:10.1093/mnras/staa2987) |
| very early | Loki, in the plane | small | tentative (Sestito et al. 2026) |
| 8 to 11 | Gaia-Sausage-Enceladus, head-on: the thick disc heated, the Splash, about 20 clusters, the disc tilted | 3 × 10⁸ to 10⁹ | settled that it happened (Belokurov et al. 2018, doi:10.1093/mnras/sty982; Helmi et al. 2018, doi:10.1038/s41586-018-0625-x). Date argued: one group puts the debris under 3 Gyr old (Donlon et al. 2024, doi:10.1093/mnras/stae1264) |
| about 9 | Sequoia, on a backwards orbit | about 5 × 10⁷ | argued (Myeong et al. 2019, doi:10.1093/mnras/stz1770) |
| 5 to 8 | Helmi streams | about 10⁸ | well supported (Helmi et al. 1999, doi:10.1038/46980) |
| 5 to 6, still going | Sagittarius: three close passes, each with a burst of star formation in the disc; a stream right round the sky; its nucleus is the cluster M54 | about 10⁸ left | settled (Ibata, Gilmore and Irwin 1994, doi:10.1038/370194a0) |
| now | Large and Small Magellanic Clouds, on their first pass, with their own small satellites; the Large Cloud is about a tenth of our galaxy's mass | 3 × 10⁹ | well supported (Besla et al. 2007, doi:10.1086/521385; Erkal et al. 2019, doi:10.1093/mnras/stz1371) |
| in 2.5 | the Large Cloud merges | | predicted (Cautun et al. 2019, doi:10.1093/mnras/sty3084) |
| in 4 to 10, or never | Andromeda | | argued: once thought certain (van der Marel et al. 2012, doi:10.1088/0004-637X/753/1/9), now about an even chance in 10 Gyr (Sawala et al. 2025, doi:10.1038/s41550-025-02563-1) |

Reviews: Helmi 2020 (doi:10.1146/annurev-astro-032620-021917); Naidu et al. 2020 (doi:10.3847/1538-4357/abaef4); Malhan et al. 2022 (doi:10.3847/1538-4357/ac4d2a).

Also pinned: Andromeda and Triangulum as neighbours (this mends fudge F37, where home is a median halo with no Local Group); the named dwarf satellites (McConnachie 2012, doi:10.1088/0004-6256/144/1/4); the named globular clusters (Harris 1996, 2010 edition, doi:10.1086/118116).

**Licence.** A finding in a paper (a date, a mass, an order of events) is not Gaia's catalogue and can go in the base pack with its citation. A table of positions and motions taken from the Gaia catalogue falls under the existing ruling and belongs only in the optional real-sky pack. In the base pack, a named satellite has its measured position and distance from older public sources and an orbit the model draws to fit its history. This is my reading and I am not a lawyer (question 8).

**Any other galaxy** gets the same richness from the same mechanism with no pins. A fictional universe pack pins its own.

---

## 9. Resolving a galaxy like a planet

### 9.1 Detail levels

| Level | Seen from | What exists | Cost (estimate) |
|---|---|---|---|
| **Glance** | the sky, the cluster map | mass, size, shape, colour (built), plus from summary numbers alone: time of last big merger, a "disturbed" flag, expected satellites, dark share | none beyond today's |
| **Chart** | the galaxy chart | disc, bulge, bar, arms; the brighter satellites where they are now, with orbits; the largest streams and shells; the cluster system as a count and a few bright ones; rotation curve | tens of ms; stored |
| **Survey** | an explicit survey, or zooming into the halo | every progenitor to the smallest; every component with where it came from; each globular cluster; dark haloes (GM view); the family tree on the ruler | 1 to 3 s once; stored; served by the GM's machine to weak devices |
| **Site** | inside a satellite, a stream, a cluster or a patch | that object's own envelope and stars. A satellite is a galaxy, so the whole chain runs on it | as G4 and G5 today |

Same world at every level: deeper levels add lower mass bins with their own keys. No heavy work for a click: a click reads Glance.

### 9.2 Time

Everything in the record is dated, so the ruler scrubs it. A swallowed galaxy can be followed back to when it was a galaxy of its own, and forward as its stream lengthens and fades. Look-ahead continues the same orbit run. "Ink a merger" (G6) now adds a progenitor to the tree with a real orbit.

### 9.3 Looking the part (for U6 and V9's True makeover)

The kernel hands the renderer a data-only list: satellites, components with their density formula, clusters, warp, tilt, arm disturbance. No pictures.

- **Chart style** inks them: satellites with orbit lines, streams and shells as labelled arcs, clusters as marks, the family tree beside the ruler.
- **True style** shows what a camera would. Streams around real spirals appear near 28 magnitudes per square arcsecond and fainter, about a thousandth of the disc's brightness (Martínez-Delgado et al. 2010, doi:10.1088/0004-6256/140/4/962; Morales et al. 2018, doi:10.1051/0004-6361/201732271). So True gets an exposure control, and the scars come up as it deepens.
- **A dark matter layer**, at Science depth and on the GM view: the halo, its lumps, and the stretched shape.

### 9.4 Every collision leaves its mark (Alex, 22 Nov 13:14: "we will need some cool realism renderer work to show damaged galaxies - the fact the MW is twisted and has ripples along its disc ... each collision should leave its mark. High speed ones can literally punch a hole. slower ones will stay around forever orbiting like the LMC")

This corrects section 1 point 6, which was too gloomy. The **faint** scars are the old ones out in the halo (streams, shells, mixed remains). Damage to the **disc** is bright and shows in True style with no help: a warp, ripples, a ring, a burst, a bridge to a companion are all made of ordinary bright stars and gas.

Each merger response in 7.5 writes a **mark** into the record, and the renderer draws marks and nothing else:

```
Mark { kind, caused by (the event), began, strength 0 to 1, shape numbers, fades over }
```

| Mark | Made by | Looks like | Lasts |
|---|---|---|---|
| **Warp** | a massive companion pulling on the outer disc | the disc's edge bent up on one side and down on the other, an integral sign seen edge-on | while the companion is near, then about a Gyr |
| **Ripples** | a satellite passing through or close by the disc | waves up and down across the disc, a few hundred parsecs high and several kiloparsecs apart, growing outward; rings of slightly different height and brightness seen face-on | fade over about a Gyr as stars drift out of step; each new pass adds a set (Antoja et al. 2018; Laporte et al. 2018; Xu et al. 2015, doi:10.1088/0004-637X/801/2/105) |
| **Tilt and twist** | a heavy merger | disc plane at an angle to the outer halo and to the old remains; the outer disc turned against the inner | billions of years |
| **Thick disc** | early heavy mergers | a fat, old, redder layer around the thin one, plain edge-on | for good |
| **Driven arms, a bridge, a tail** | a close, slowish pass, forwards | two strong arms, one reaching to the companion, a long tail opposite | a few hundred Myr to a Gyr |
| **A punched hole and ring** | a fast, nearly straight pass through the disc | an expanding ring of new blue stars with an emptied middle, sometimes spokes, the culprit a little way off on the axis | 100 to 500 Myr (Lynds and Toomre 1976; Appleton and Struck-Marcell 1996) |
| **A small hole** | a dwarf or a dark halo through the gas layer | a gap in the gas with young stars round its rim | tens of Myr; argued as a cause of real gas holes (Bekki and Chiba 2006, doi:10.1086/499812) |
| **Burst knots** | gas squeezed by any pass | bright blue clumps and pink gas clouds along the disturbed side | tens of Myr each |
| **Lopsided disc, nucleus off-centre or double** | a recent meal | the disc heavier on one side; two bright points at the core | a Gyr or so |
| **Shells, streams, plumes** | 7.6 | faint arcs and loops in the halo | Gyrs, fading |
| **A gas trail** | a gas-rich satellite losing gas | a long ribbon of gas behind the companion, seen in radio | while it orbits |
| **A warm side** | a massive companion squeezing the hot halo | one half of the X-ray halo brighter | 100 Myr or so |

Notes for the renderer thread (GH9):
- **Satellites are small galaxies and glow as such.** Each is drawn from its own G3 result: a blue, clumpy one still forming stars (like the Large Cloud), a faint red smudge for a dead one, and nothing at all for a dark halo unless the dark matter layer is on.
- **Speed decides the mark, as Alex says.** Fast and straight through: a hole and a ring, and the intruder leaves. Slow: it is captured, sinks and keeps disturbing the disc at every pass. How long it lasts depends on its mass. A small satellite does orbit almost for ever. A heavy one is dragged in: the Large Cloud itself is predicted to merge in about 2.5 Gyr (Cautun et al. 2019), which look-ahead will show.
- Marks overlap and add. Strength falls with age by the "fades over" time, so scrubbing the ruler shows a galaxy being damaged and healing.
- Each mark is a formula over the disc (a bend, a wave, a ring), so it costs a few numbers, works at every detail level and needs no particles. The exact shapes are stand-ins for the disc's real response (section 10) and go on the register.
- Some marks belong to a view style other than visible light (gas trail in radio, warm side in X-rays), which fits the one-switch view styles design.

Citations in this section other than those already listed are from memory, for GH0.

A mockup of the galaxy chart with its history should come before GH8, as the Reference screen's did.

---

## 10. What fits a browser

| Piece | How | Verdict |
|---|---|---|
| Sheet rows, ledgers, halo profile, rotation curve | formulas | trivial |
| Assembly tree | keyed counts per mass bin | trivial at Chart; under 0.1 s at Survey |
| A galaxy per progenitor | G3, about 2 ms each; Sketch envelope until looked at | fine |
| Orbits and stripping | leapfrog, one test body each, 14,000 steps | about 1 million steps at Chart, 30 million at Survey: under 1 s compiled |
| Components in the envelope | a sum over the few that overlap a patch, indexed by radius | fine |
| Clusters one by one | a few hundred a galaxy | fine |
| Storage | about 30 kB a galaxy at Chart, about 1 MB at Survey | fine |

**Not practicable**, with what stands in (each goes on the register):

| Real physics | Why not | Stand-in |
|---|---|---|
| The true shape of tails and bridges in one collision | needs millions of mutually pulling particles | shape family chosen from the orbit; a detailed block or precomputed table later |
| The disc's own answer: bars, warps, ripples from first principles | same | strengths from published scalings |
| Gas dynamics of stripping and of dwarf collisions | hydrodynamics | pressure ratios; drawn outcomes for the rare collision |
| Streams bent by a lumpy, stretched, moving halo | needs the full potential in time | a tube around the orbit in a round halo |
| Gaps punched in thin streams by dark lumps | many fast encounters | a count of gaps from the encounter rate, later (Erkal et al. 2016, doi:10.1093/mnras/stw1957) |
| The real local universe's web | constrained initial conditions | pinned neighbours |

---

## 11. What is reused

| Already there | Used here for |
|---|---|
| Formation's record, pins, ladder, principles | the assembly record (section 6), nearly unchanged |
| G1 cosmology and web | follows the sheet's rows; the matter ledger |
| G2 haloes, growth, mergers | the tree's host and its pinned major mergers |
| Y4 part 2 sub-haloes, sinking times, satellite light | the tree's fill; becomes its Chart level |
| G3 regulator and rings | every progenitor's galaxy; fresh gas into rings; bursts |
| G nuclei | arriving holes are the progenitors' own |
| G4 envelope | one new line: overlapping components |
| G5 cells, stars, clusters, orbits, promotion | ancient patches; cluster birth; a cluster as a place |
| G6 events, encounter block, ink a merger, before and after | satellite collisions; edits to the tree |
| `reconstruct_origin` | origins across galaxies |
| Y1 count tree, Y3 glow, Y4 parts 1 and 3 | components in the sky; far galaxies unchanged |
| O6 and O7 light | each component's own colour |
| The F track's integrator know-how | the leapfrog (a far simpler job) |
| Class atlas (C track) | every label in this design |
| Data sheet marks and reach report | theoretical; follows, estimated, cannot follow |

**New blocks:** halo structure and rotation curve; the tree's pin-and-fill rule and recursion; the test-body orbit with the four processes; merger response shares; the component density formulas; cluster formation and loss; the two rare events. About a third of the work.

**Where the formation lens does not fit:**
1. Planets do not contain earlier planetary systems. Galaxies contain earlier galaxies, so the mechanism calls itself and must be kept lazy.
2. Planets collide and stick. Stars and dark matter pass through, so remains are orbits and chemistry, and the gas behaves differently from both. That difference is where galaxies without dark matter come from. Its nearest cousin in formation is a giant impact stripping a mantle.
3. Formation ends. Assembly has not, so "now" always holds things half eaten.

**Fudges this retires** (all on the register today): most of F39 (satellites with no orbits, no stripping, one quench delay, nothing small); part of F37; ex-situ stars and holes by ratio; the bar test's × 1.1; the typed 220 km/s; and it gives the stand-in arm crests their first real disturbance.

---

## 12. Research, graded

Settled: the measurement is secure and the reading is agreed. Well supported: agreed in outline, numbers still moving. Argued: live dispute. Open: not known.

| Topic | Grade | Note |
|---|---|---|
| Amount of dark matter and dark energy | settled | Planck 2018; supernovae (Riess et al. 1998, doi:10.1086/300499; Perlmutter et al. 1999, doi:10.1086/307221) |
| That dark matter is separate from gas | settled | colliding clusters (Clowe et al. 2006, doi:10.1086/508162) |
| What dark matter is | **open** | Bullock and Boylan-Kolchin 2017 (doi:10.1146/annurev-astro-091916-055313); Tulin and Yu 2018 (doi:10.1016/j.physrep.2017.11.004) |
| Whether dark energy changes with time | **argued** | DESI |
| The web, halo counts, growing by mergers | settled | Bond, Kofman and Pogosyan 1996 (doi:10.1038/380603a0); Lacey and Cole 1993 (doi:10.1093/mnras/262.3.627) |
| Halo profiles and concentrations | well supported | cusps against cores still argued in dwarfs (Pontzen and Governato 2012, doi:10.1111/j.1365-2966.2012.20571.x) |
| Stripping order: dark matter, then stars | well supported | idealised simulations; whether small haloes ever fully dissolve is argued (van den Bosch et al. 2018, doi:10.1093/mnras/stx2956) |
| Stellar haloes are swallowed galaxies | settled | Bullock and Johnston 2005 (doi:10.1086/497422); Cooper et al. 2010 (doi:10.1111/j.1365-2966.2010.16740.x) |
| The Milky Way's own list | well supported in outline | section 8 for each |
| Number of faint satellites | well supported | set by reionisation and stripping by the disc |
| Planes of satellites | **argued** | Pawlowski 2018 (doi:10.1142/S0217732318300045); not modelled, group infall gives a little of it |
| Globular clusters as the top of ordinary cluster formation | well supported | their odd chemistry is **open** and not modelled |
| Galaxies without dark matter | observed; cause **argued** | 7.4 |
| Late metal-free stars | **open** | 7.7 |
| Thick disc from early turbulence, from a merger, or both | **argued** | both processes run here and the result decides |

**Citations.** Read at source on 22 Nov: Jiang et al. 2021, van Dokkum et al. 2022, Kruijssen et al. 2020, Aguado-Agelet et al. 2026, Donlon et al. 2024 (through ESA's summary), Xiang and Rix 2022 (through MPG's summary), Sestito et al. 2026, Oprea et al. 2026 (through the RAS summary), Belokurov's article, and those already checked in the Y4 log (Giocoli et al. 2008, Boylan-Kolchin et al. 2008, Springel et al. 2008, Wetzel et al. 2013). **Every other reference and number in this document is written from memory** and must be checked at source by the citations thread in GH0 before a block relies on it (build plan section 1).

---

## 13. Packages (track GH, with DM1 to DM5 first)

Owners are suggestions for the coordinator. Each package adds its own fudge register rows.

| Id | Package | Needs | Owner | Acceptance checks |
|---|---|---|---|---|
| GH0 | Check every from-memory citation in section 12; time a throwaway tree and orbit run in WebAssembly on a phone and a laptop; walk a full catalogue of identified galaxy types (Hubble and de Vaucouleurs sequence, dwarf kinds, peculiar and interacting kinds, active kinds) and say for each which processes and Marks produce it, naming any the mechanism cannot yet produce (Alex, 22 Nov: "we will have every type of identified galaxy"); file the atom issues | none | citations, kernel | a checked source list; measured costs replace the estimates in 9.1 and 10; a type coverage table with every gap named |
| DM1 | Sheet: Cosmos group, the rows of 3.2, the theoretical mark, reach declarations; `base-cosmology` reads the sheet | MP1c | matter data sheet | nothing edited gives identical hashes; a changed Ωc gives a reach report; the mark shows at every depth, on phone and Night chart |
| DM2 | Dark energy: w0, wa, growth by its equation; "real size" switch on foam and cluster maps | DM1 | kernel; galaxy charts | growth matches today's closed form at w = −1 to 0.2%; void profile against Hamaus et al.; scrub shows expansion |
| DM3 | Matter ledger per cell and per halo; op `matter_budget`; "where the matter is" bar; dark matter layer | DM1 | kernel; galaxy charts | closure to rounding; environment shares and baryon census within stated bands |
| DM4 | Halo structure and rotation curve; retire the typed speeds | DM1 | kernel | Milky Way speed, local density, the baryon to speed relation; G5 and origin goldens re-blessed on purpose, reasons in the log |
| DM5 | Warm and self-interacting rows wired through the two numbers | DM4, GH2 | kernel | a 3 keV universe has fewer ultra-faint dwarfs by the published factor; cored dwarfs strip faster |
| DM6 | Constants with a history: a time law (constant, drift, step) on any Constants row, read by date; G first; the "particle route" lines on the Cosmos page | DM1 | matter data sheet; kernel | with every law constant, all hashes identical; a drifting G changes a year's length by the Kepler factor at each date; a stepped G shows a star's brightness change with the ESTIMATED stamp; reach report lists what cannot follow |
| GH1 | One assembly tree: pins, fill, recursion, levels; the record of section 6 | G2, Y4 | kernel | no double counting (accreted mass closes); deeper levels never change members already drawn; Y4's satellites above the old floor unchanged where the budget allows; order independence |
| GH2 | Orbits and the four processes; `satellites_at(time)`; labels | GH1, DM4 | kernel | section 7.3 checks; energy drift bound on the leapfrog; identical on rerun and across devices; retires F39's limits |
| GH3 | Progenitors feed the host: stars, gas, holes; merger responses, each writing a dated mark (9.4) | GH2 | kernel | stellar halo mass and metals against biggest progenitor; share of stars not born here against mass (Rodriguez-Gomez et al. 2016, used in G3); a Gaia-Sausage-like history gives a thick disc and two chemical families; G3 goldens re-blessed on purpose |
| GH4 | Components in the envelope, cells, origins, count tree and glow | GH3 | kernel; light and nebulae | section 7.6 checks; budget closure with components; a promoted star keeps its origin at any time read; G4, G5 and sky goldens re-blessed on purpose |
| GH5 | Clusters: birth, loss, arrival; a cluster as a place | GH3 | kernel | section 7.8 checks |
| GH6 | Satellite collisions, tidal dwarfs, stripped nuclei, pristine pockets; galaxy labels in the class atlas | GH2, GH5 | kernel | rates stated and bounded; each event reproduces from its manifest; honesty marks present |
| GH7 | Milky Way pack: pinned history, Local Group, named satellites and clusters, grades | GH3, GH5 | kernel; citations | pins survive and claim budget; fill adds no second copy of a pinned object; licence rule of section 8 kept; mends F37's placement |
| GH8 | Galaxy chart: satellites and orbits, streams, clusters, family tree on the ruler, rotation curve, dark matter layer, "where was this star born" | GH4, a mockup | galaxy charts | screenshots desk and phone, both schemes; no heavy work on a click |
| GH9 | Renderer: damaged galaxies from the mark list of 9.4 (warp, ripples, tilt, thick disc, driven arms and bridges, punched ring, burst knots, lopsidedness), satellites as small galaxies, halo remains with an exposure control in True | GH3, GH4, V9 | photoreal renderer | each mark kind has a fixture and a screenshot face-on and edge-on; scrubbing shows a mark appear and fade; the same galaxy in Chart and True; phone tier keeps warp, ring and satellites at least |
| GH10 | Sky: satellites move, streams in deep exposures, clusters as objects | GH4, Y5 | kernel; galaxy charts | no double counting in magnitude counts |
| GH11 | Story and search: sifters over assembly events; outcome-first queries ("a galaxy that lost its dark matter", "a world inside a globular cluster", "a stream crossing a habitable band") | GH6 | agents; kernel | hits carry evidence |
| GH12 | Later: stream gaps from dark lumps; cluster scale (gas torn from dark matter in colliding clusters, light between galaxies, galaxies stripped into "jellyfish"); the fuller merger tree; a stretched halo; far future with G8 | | | |

Order: GH0 and DM1 now. DM2, DM3 and DM4 are independent of each other. GH1 to GH4 in order. GH5 to GH7 after GH3. The switch-over packages (DM4, GH3, GH4) change goldens on purpose and each says why in its log.

**Fudge register rows to add as they land** (in `docs/fudges.md`, Galaxy and cosmology): the host halo is round and smooth for orbits; drag uses one Coulomb logarithm; stripping follows fitted tracks from idealised simulations; a stream is a tube round its progenitor's orbit; mixing state is a formula in wraps; merger response shares are published scalings; cluster formation efficiency is tuned to the cluster-to-halo relation; satellite collision and tidal dwarf outcomes are drawn; warm and self-interacting dark matter act only through two numbers; satellites do not pull on the host or on each other (so no reflex motion, except as a tilt strength); Sketch values are envelopes until looked at; pocket ignition is a drawn rate.

---

## 14. Rulings **[accepted by Alex, 22 Nov 2026; each was put as a question with a recommendation, and the recommendation is the ruling unless marked]**

1. **A fourth mark, "theoretical"**, for dark matter and dark energy rows: amount measured by gravity, nature unknown. Shown on the row at every depth; not stamped on every galaxy. **Yes.**
2. **Move the measured cosmic parameters onto the data sheet** as a Cosmos group (tuned model numbers stay in the cosmology pack). **Yes.**
3. **What dark matter is**: cold by default; warm (particle mass) and self-interacting (cross-section) as editable theoretical rows in 4.0, acting through a smallest-halo size and a core size; fuzzy listed for later. **Yes.** Decided by Alex 22 Nov 13:10: no altered-gravity (MOND) mode, and the page says SSE4 takes the particle route and why; as the "bone", a constant may carry a history, so G can vary through time as a what-if (package DM6).
4. **Dark energy**: no frig. It is already in. Add its two pressure rows and a "real size" switch so the expansion is seen. **Yes.**
5. **One assembly tree** replaces the separate major-merger and satellite draws, with major mergers as pins. Three packages will change galaxy and star goldens on purpose. **Yes.**
6. **Satellite orbits by stepping each one as a single test body** in a smooth potential, our own code, stored as slices. I read this as inside "no particle simulation". **Yes.**
7. **Battle-worn**: answered by Alex 22 Nov 13:14, see section 9.4. Every collision leaves a dated mark the renderer draws; disc damage shows in True style as it is; only the faint halo remains need the exposure control or Chart's ink.
8. **Milky Way pack**: pin the known history with grades. Where experts disagree (the date of the big collision, whether Kraken was a separate galaxy), take the majority view and offer the other as a one-click what-if. Published findings go in the base pack; Gaia catalogue values stay in the optional pack. **Yes.** The licence reading is Claude's, not a lawyer's.
9. **Pristine pockets and late metal-free stars**: on by default, rare, marked plausible early and speculative today, found only in dark haloes at the outskirts and in voids. **Yes.**
10. **Two readings to confirm.** "Micro galaxies" I have taken as the ultra-faint dwarfs, down to a few hundred stars. "Every mid-universe galaxy" I have taken as every ordinary galaxy anywhere, at any time between the first ones and the far future, not only ours. **Both confirmed.**
11. **A mockup next**: the galaxy chart with its history (satellites, streams, family tree, rotation curve, dark matter layer), by this thread, before GH8. **Yes.**
12. **Cluster scale** (colliding clusters, light between galaxies, stripped "jellyfish") waits in GH12. **Later.**

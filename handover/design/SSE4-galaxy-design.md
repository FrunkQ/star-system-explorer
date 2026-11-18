# SSE4 galaxy evolution design (cosmos suite)

Companion to `SSE4-design-handoff.md` (read that first; terms such as envelope, resolve, keyed draw, chronicle, fidelity contract are defined there). Date 17 Nov 2026.

Goal: a universe from the Big Bang to the far future, far richer than V3, with **no particle simulation**. Everything here is closed-form, table lookup, or a small ordinary differential equation stepped on a fixed grid. A full simulation can replace any box later as a heavy block.

Every model below is a **flow** of atoms that the user can open, read and change. Published fitting relations are named so builders can find them; builders must check each against its source paper and record the citation in the atom declaration. Where I give a number it is a default for a pack, not a constant in code.

---

## 1. The chain

```
C  cosmology            parameters, time, expansion, growth
W  cosmic web           one deterministic density field at any point and scale
H  haloes               which haloes exist in a region, their growth and mergers
Gx galaxy               gas, stars, metals, black hole, shape, over time
R  rings                radial structure and chemical history of a disc
P  patch                local conditions anywhere in a galaxy at any time: the envelope
S  populations          which stars exist: birth cohorts, masses, orbits, remnants
E  events               supernovae, bursts, encounters, mergers, active nuclei
->  physics suite        a star system formed from its patch's envelope
->  life suite           hazards and ingredients for biospheres
```

Each level is the envelope for the level below. Each level can be the starting point: a user who wants one system never touches C to R; defaults stand in (a Milky-Way-like patch).

---

## 2. C: cosmology

Pack parameters (defaults close to Planck 2018): H0, Ωm, ΩΛ, Ωb, σ8, ns. All editable; a fictional universe pack may change them and everything downstream follows.

Atoms:

- `cosmo.time_of_scale` : flat ΛCDM has a closed form, t(a) = (2 / (3 H0 √ΩΛ)) · asinh( √(ΩΛ/Ωm) · a^1.5 ). Inverse by the matching sinh form.
- `cosmo.growth` : linear growth factor D(a), Carroll, Press and Turner (1992) approximation.
- `cosmo.sigma_m` : rms fluctuation σ(M) from a fitted transfer function (Eisenstein and Hu 1998 no-wiggle form), tabulated once per parameter set.
- `cosmo.epochs` : recombination, dark ages, first stars, reionisation, peak star formation, now, last star formation, degenerate era, black hole era. Each is an event derived from the model, never a hard-coded date. Far-future eras follow Adams and Laughlin (1997) and are labelled speculative; proton decay is a pack switch.

Universe time is seconds since the origin. "Now" is a pack value (default 13.8 Gyr).

Reionisation: ionised fraction Q(t) from the collapsed fraction of haloes above the cooling threshold (from H) times an efficiency parameter. It feeds back as a filtering mass below which haloes cannot accrete gas afterwards. This is what makes some dwarf galaxies fossils and leaves primordial pockets.

---

## 3. W: the cosmic web as a function, not a grid

Requirement: evaluate density at any point, at any zoom, on any device, without storing a volume, and identically everywhere.

- The initial density contrast δ0(x) is a sum of band-limited noise octaves. Octave j has lattice spacing L0 / 2^j; each lattice point's value is a keyed draw `draw(seed, "web", j, ix, iy, iz)`; interpolation is a fixed smooth kernel. Octave amplitudes are set so the variance per octave matches σ(M) from C. Evaluating a point touches 64 lattice values per octave and as many octaves as the requested scale needs. Nothing is stored.
- Linear evolution: δ(x, t) = D(t) · δ0(x).
- Non-linear look: a lognormal transform of δ for density, plus a displacement of sample points along the gradient of the smoothed potential (one Zel'dovich step) so matter visibly drains from voids into walls, filaments and knots as time advances.
- Classification for the map and for halo bias: eigenvalues of the tidal tensor (second derivatives of the smoothed field, by finite differences of the same function). Count above a threshold gives void, sheet, filament or knot (the "T-web" method, Hahn et al. 2007).

Rendering: the foam is iso-surfaces and volume shading of this function, with gloss; on weak devices a pre-shaded slice. Scrubbing time changes D(t) and the displacement only, so the foam grows smoothly from near-uniform to today's web and beyond.

Check: measured two-point statistics of the generated field match the input spectrum within tolerance at three scales; identical hashes of sampled values across devices.

---

## 4. H: haloes

A region cell (default 8 Mpc comoving) has a mean overdensity from W.

- **How many haloes of each mass**: conditional mass function (Sheth and Tormen 1999 with peak-background split) gives expected counts per mass bin given the cell's overdensity. Counts are keyed Poisson draws per bin. Dense cells get more and bigger haloes; voids get few dwarfs. Budget rule: total halo mass may not exceed the cell's collapsed mass; the partition is drawn top-down from the largest bin.
- **Where**: positions at the highest peaks of the finer noise octaves inside the cell, so galaxies sit on the filaments you can see.
- **Growth history**: M(z) = M0 · (1+z)^β · exp(−γ z) (McBride et al. 2009). β and γ are drawn with scatter correlated with environment (denser means earlier formation).
- **Spin**: lognormal, median 0.035, width 0.5 dex (Bullock et al. 2001). Sets disc size later.
- **Major mergers**: a keyed Poisson process in time with the rate fit of Fakhouri, Ma and Boylan-Kolchin (2010) as a function of mass, mass ratio and redshift. Each major merger (ratio above 1:4) is an **event in the chronicle** with a time, a mass ratio and an orbit sketch. Minor mergers are part of smooth growth, except that the few largest leave **streams** (section 8).
- **Satellites and groups**: sub-haloes drawn from the sub-halo mass function; group membership is a relation between galaxy addresses, with a simple two-body orbit for the nearest large neighbour (this is what schedules a future Andromeda-style collision).

Because mergers are events, a GM can add, delete or move one with a click, and everything after it recomputes. This is the main lever for "rewrite a galaxy's history".

Check: halo counts against the Sheth-Tormen function over a large sample volume; merger counts per halo against the fit.

---

## 5. Gx: one galaxy as a regulated reservoir

A small system of rate equations per galaxy, stepped on a fixed grid (default 10 Myr, about 1,400 steps for the whole of history; cost is trivial). This is the "bathtub" or "regulator" approach (Bouché et al. 2010; Lilly et al. 2013; Davé et al. 2012).

Reservoirs: hot halo gas, cold gas M_g, stars M_*, metals by element group (section 6), central black hole M_bh.

- Gas supply = baryon fraction × halo growth rate × an efficiency that falls below the reionisation filtering mass and above about 10^12 solar masses (hot haloes, black hole heating).
- Star formation rate = M_g / t_dep, with depletion time about 1 to 2 Gyr today, shorter in the past.
- Outflow = η × star formation rate, with mass loading η falling with halo circular speed.
- Returned gas from dying stars with delay (section 6).
- Black hole grows mostly in mergers and instabilities; its output heats the halo above a mass threshold, shutting star formation down (quenching). Active-nucleus episodes are events.
- Shape: disc scale length from spin and halo size (Mo, Mao and White 1998); a major merger moves a mass-ratio-dependent fraction of disc stars into a bulge and triggers a starburst; a disc heavy enough for its halo grows a bar. Morphology class (disc, barred, lenticular, elliptical, irregular, dwarf) is a classification rule over these numbers, not a draw.

Outputs per time step are stored as the galaxy's history curves: this is what the time ruler shows.

Checks against published relations at the present day and at two earlier epochs: stellar mass against halo mass (Behroozi et al. 2019), star-forming main sequence, mass-metallicity relation, cosmic star formation history summed over a volume (Madau and Dickinson 2014), fraction of quenched galaxies with mass. Tolerances are loose (a factor of two); the aim is right shapes and trends.

---

## 6. R: rings and what died here before

This is where Alex's "systems form from the gas, dust and metals available in the patch, set by what died there before" becomes concrete. A disc is split into radial rings (default 24, log-spaced). Classic multi-zone chemical evolution (Matteucci and Francois 1989; Chiappini et al. 1997), with no particles.

Per ring, per time step: gas surface density, star formation by the Kennicutt-Schmidt law (rate ∝ gas density^1.4), infall with a timescale that grows with radius (inside-out formation), optional slow inward gas drift, and enrichment.

**Element groups tracked** (each a separate conserved budget):

| Group | Main source | Delay | Why it matters downstream |
|---|---|---|---|
| H, He, Li | Big Bang | none | baseline |
| O and α (Mg, Si, S, Ca) | core-collapse supernovae of massive stars | 3 to 40 Myr | rock; mantle Mg/Si ratio |
| Fe peak | type Ia supernovae (plus some core collapse) | 40 Myr to 10 Gyr, rate ∝ 1/t | iron cores; the [Fe/H] clock |
| C, N | dying intermediate stars (AGB) | 0.1 to 10 Gyr | C/O ratio sets carbide or silicate worlds; volatiles for life |
| s-process (Sr, Ba) | AGB stars | as above | tracer |
| r-process (Eu, Th, U) | neutron-star mergers, rare supernovae | from about 30 Myr, rate ∝ 1/t, rare and lumpy | radiogenic heat: how long a planet stays geologically alive |
| K-40 | core collapse | prompt | radiogenic heat |
| Al-26 (short-lived) | a supernova or massive-star wind near the birth cluster | must be within about 1 Myr | early melting and drying of planetesimals |

Enrichment at time t is the star formation history convolved with each channel's delay-time distribution and yield table. Yield tables are pack data (for example Nomoto et al. 2013 for massive stars, Karakas 2010 for AGB); stellar lifetimes are a fitted function of mass and metallicity.

Rare sources are **lumpy on purpose**: r-process events are few enough that the number in a ring per step is a keyed Poisson draw, so one neighbourhood can be uranium-rich and the next poor. That scatter is real, and it is story material.

What a ring knows at any time: gas mass and its abundances by group, the stars formed so far by age and abundance, the present rates of each kind of stellar death. That is the "what died here" ledger, and explain mode can show it as a stacked history.

Checks: radial metallicity gradient of a Milky-Way-like disc (about −0.05 to −0.07 dex per kpc), the [α/Fe] against [Fe/H] track with its knee, the metallicity distribution of long-lived stars near the Sun's radius (no excess of metal-poor stars), present supernova rates of each type.

---

## 7. P: the patch envelope

A patch is a cell of an octree in galactocentric coordinates, refined down to the neighbourhood scale (default leaf about 16 parsecs across). Its envelope at time t is computed, never stored unless edited:

```
from the ring(s) it overlaps:   gas density, abundances by group, star formation rate, death rates
× spiral arm and bar phase:     arms are a log-spiral density wave with a pattern speed; being in an arm multiplies gas density and star formation
× vertical position:            thin disc, thick disc, halo, each with its own age and abundance mix and scale height
× bulge / nuclear region terms
× local texture:                a lognormal turbulence factor from keyed noise (clouds and voids in the gas)
+ local events still in effect: supernova remnants, a star-forming complex, a stream passing through
+ authored edits
```

Derived hazards and conditions per patch: stellar density, encounter rate (density × cross-section × velocity dispersion), supernova rate within a harmful distance, gamma-ray burst exposure (rare, favoured at low metallicity), active-nucleus exposure, radiation background, time since metals passed the threshold for rocky planets.

**The green regions** are a classification over those: enough metals for rocky planets, not so metal-rich that migrating giant planets dominate, a low rate of sterilising events, low encounter rate, and (for complex life) enough quiet time. This is the galactic habitable zone idea (Lineweaver, Fenner and Gibson 2004) made editable. On the galaxy chart it is a hatched green band that visibly starts near the centre and moves outward over billions of years. Each criterion is a slider in a game or universe pack ("habitable" for a Force world may mean something else).

---

## 8. S: stars, identity, and the hard problem of time

### 8.1 Birth cohorts

Stars are born in a place and a time. A **cohort** is (birth patch, birth time step). Its total stellar mass comes from the patch's star formation rate; that is the budget. Within a cohort:

- Stars are born in **clusters** drawn from a cluster mass function (∝ M^−2). Cluster membership matters: it sets the chance of a nearby massive star (Al-26, harsh ultraviolet that strips discs), and siblings share abundances.
- Masses from the initial mass function (Kroupa 2001), drawn by a deterministic partition of the cluster's mass.
- Multiplicity (single, binary, triple) and orbital separations from fits that depend on primary mass (Raghavan et al. 2010; Moe and Di Stefano 2017).
- Abundances: the patch's gas at that time plus a small sibling-shared scatter.

Stellar evolution is a table lookup (pack data from published tracks such as MIST): for a mass and metallicity, radius, temperature and luminosity against age, the end state (white dwarf, neutron star, black hole, none yet), and what is returned to the gas. Remnants persist as objects.

### 8.2 Orbits without integration

A star's galactic orbit is described, not integrated: a guiding radius, a radial wobble and a vertical wobble with amplitudes drawn from the velocity dispersion for its birth population (which rises with age, roughly ∝ age^0.35), and phases. Position at any time is closed-form (the epicyclic approximation). Slow drift of the guiding radius (radial migration, which is why the Sun's neighbours have mixed birthplaces) is a short list of keyed step events per star. Halo stars and stream stars carry a simple eccentric orbit instead.

### 8.3 The rule that makes time work

Stars move tens of parsecs per million years relative to each other, so "the stars near here" is a different set at different epochs. A naive design either pops stars in and out or has to track a hundred billion orbits. The rule:

1. **A system that has been looked at is permanent.** Resolving a system records it. From then on it has one life line and one orbit, and it is the same system at every time anyone asks. It can be followed backwards to its birth cluster and forwards to its death.
2. **Everything not yet looked at is fill, valid for one time frame.** Time is divided into frames (default 1 Myr at neighbourhood scale, longer for coarser tiers). In a frame, the unresolved stars of a patch are drawn from the patch's envelope for that frame, keyed by (patch, frame). Inside the frame they move on their orbits. In the next frame the fill is drawn afresh.
3. **Permanent systems are pins in every frame.** They claim their share of the frame's budget first, so there is never double counting.
4. **Looking promotes.** Selecting, visiting, naming, charting or routing to a fill star makes it permanent.
5. **Encounters are events, not coincidences.** For a permanent system, close stellar passages over deep time are drawn as an event series from its patch's encounter rate along its orbit, keyed by the system's address. Each encounter creates a permanent passing star whose orbit is consistent with being there then.

What this guarantees: any identified system is identical no matter when or in what order it is examined; the statistics of any neighbourhood at any time match its envelope; campaigns (years to millennia) never cross a frame boundary in practice. What it does not claim: that the anonymous background a billion years from now is "the same stars moved". It is a consistent reconstruction and is labelled so. This replaces the looser wording of experiment 4 in the handoff.

Past edits: if a GM changes history earlier than a permanent system's birth, the system stays (it is a pin). If it no longer fits the new history (older than its galaxy now allows, say) a sifter flags it and it is stamped impossible; the GM may keep it as a mystery or release it.

### 8.4 From patch to planetary system

The physics suite forms a system from: stellar mass and multiplicity, abundances by group, birth cluster environment, age. Consequences it must honour: dust mass in the disc scales with metallicity; giant planet frequency rises steeply with [Fe/H] and with stellar mass (Fischer and Valenti 2005); C/O and Mg/Si set rock chemistry; r-process and K-40 set radiogenic heat; Al-26 sets early water loss; cluster ultraviolet truncates outer discs; close binaries truncate or exclude planet zones. A metal-free (Population III) star gets no rocky planets, and explain mode says why.

### 8.5 Special populations (each a rule set over the same machinery)

Open clusters and associations (young cohorts not yet dispersed), globular clusters (old, metal-poor, drawn with the halo), stellar streams (the stars of an accreted dwarf, spread along its orbit: an old, chemically distinct population threading the disc, good story material), the nuclear cluster and central black hole, giant molecular clouds and nebulae (objects in gas-rich patches with lifetimes of tens of Myr), compact remnants and their binaries, rogue planets (a density from ejection rates), **primordial pockets** (gas that stayed metal-free late: rare, in voids and small haloes, labelled plausible) and surviving low-mass first-generation stars (labelled speculative).

---

## 9. E: events

Events are how the galaxy becomes history rather than statistics. All are chronicle events with causes, reach and consequences, and all are editable.

| Event | Drawn from | Reach and effect |
|---|---|---|
| Core-collapse supernova | massive stars in cohorts (resolved ones individually; unresolved as a rate) | remnant bubble tens of parsecs for about 1 Myr; enrichment; harm to biospheres within about 10 parsecs; may trigger star formation; Al-26 for clusters forming nearby |
| Type Ia supernova | white dwarf binaries, delay distribution | iron; similar harm radius |
| Neutron-star merger | compact binaries | r-process lump; kilonova |
| Gamma-ray burst | rare massive-star deaths, low metallicity | beamed; harmful along the beam to about a kiloparsec |
| Stellar encounter | section 8.3 | perturbs outer planets and comet clouds; comet showers (an impact hazard handed to the physics and life suites) |
| Cloud passage | system orbit through gas texture | brief climate and radiation effects |
| Starburst | merger or bar inflow | a spike of everything above |
| Active nucleus episode | black hole accretion | radiation and wind along the axis; quenching |
| Major merger | section 4 | bulge growth, starburst, streams, orbit heating |
| Arm crossing | orbit versus pattern speed | higher hazard and encounter rates for tens of Myr |

Events cross patch boundaries by modifying neighbours' envelopes from `activated_at` onward (light and shock travel times respected).

---

## 10. The Milky Way and other authored galaxies

A universe pack can pin a galaxy: its parameters (masses, scale lengths, bar, arm geometry, pattern speeds), its merger history as authored events (for the real galaxy, the known ancient merger and the infalling Sagittarius dwarf and Magellanic Clouds), and catalogue stars as permanent pins at "now" with measured positions and velocities turned into orbits. The Sun sits at its measured radius and height. Fill supplies only what catalogues miss. Named nebulae, clusters and nearby galaxies are pins too. A fictional universe pack does the same with invented data.

---

## 11. What the user does with it

- **Chart a galaxy** (GM, chart room): face-on and edge-on charts with rings, arms, green band, hazards; a time ruler with the galaxy's history curves and events.
- **Rewrite history in a few clicks**: ink a merger, a starburst, a quiet nucleus, a passing satellite, an earlier or later reionisation, a different spin. Each is one authored event; the chart and every curve redraw from that time forward; a before/after ghost shows the difference.
- **Find a place**: outcome-first queries over patches ("calm, metal-rich, a K dwarf with a temperate rocky world, no supernova within 50 parsecs for 2 Gyr").
- **Survey**: resolve a patch into systems; unresolved space stays hatched with an expected count.
- **Read the ledger**: "what died here" for any patch or any star.
- **Look ahead**: the fate of a galaxy, a cluster, a neighbourhood.
- **See it**: the same data drives the sky backgrounds, the galaxy seen from inside, and the foam.

---

## 12. Stages and checks (cosmos track)

| Stage | Delivers | Verified by |
|---|---|---|
| G0 | Default patch envelope (Milky-Way-like) so the physics suite can form systems before any galaxy exists | system generation tests run from an envelope fixture |
| G1 | C and W: cosmology atoms, web function, foam view, epochs on the time ruler | closed-form checks against tabulated values; spectrum test; cross-device hashes |
| G2 | H: halo catalogue per cell, growth, spin, merger events; cluster and filament maps | mass function and merger rate tests over a sample volume |
| G3 | Gx and R: galaxy reservoirs, rings, element groups; galaxy chart with history curves; "what died here" | the published-relation checks in sections 5 and 6 |
| G4 | P and green regions; outcome-first queries at galaxy scale | envelope continuity across patch boundaries; habitable band position and its drift with time for a Milky-Way-like disc |
| G5 | S: cohorts, clusters, masses, multiplicity, evolution tables, orbits, frames, promotion; resolve to systems | order independence; budget closure with pins; local star counts and mass function near the Sun's radius against observation; identity test (a promoted system is bit-identical read at 20 different times in random order) |
| G6 | E: events, cross-patch effects, encounters; history editing with before/after | edit locality; event rates against section 6 rates; a scripted "ink a merger" scenario reproduces from its manifest |
| G7 | Milky Way pack, catalogue pins, sky and far-field rendering | pinned stars at catalogue positions; no double counting in magnitude counts; sky brightness sanity along three sight lines |
| G8 | Far future and group dynamics | scripted "age the galaxy 10 Gyr" run; labelled speculative where it is |

Each stage ships a fixture universe and a hash so later work cannot silently change it.

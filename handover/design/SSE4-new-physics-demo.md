# SSE4: the new-physics demo (track ND, video G)

Status: **accepted by Alex, 22 Nov 2026** (all seven questions of section 5 answered yes, 15:23 to 16:03 UTC).
Nothing here is built or filmed yet; the packages of section 3 are with their owner threads and the film
(ND10) is made near the end. Asked for by
Alex on 22 Nov: a short video, made near the end, that shows SSE4's core capability end to end. A user hands
Orac a paper, Orac works out how to express it as visible, editable, cited blocks, and the result is saved as a
pack that anyone can load, so only one person pays for the expensive model.

Alex's framing (22 Nov), which this plan follows:
- "No-one else can do this and it turns SSE from an RPG tool into something that could be used for academic
  what-ifs."
- "The physics chains being visible and editable is the killer blow: the paper is not disappearing into a hidden
  codebase, it is surfaced as simple blocks."

Checked against `main` at 2cef498 (22 Nov 2026).

## 1. The paper

Hollands, Gänsicke, Tremblay and Koester 2026, "Any colour you like: the diverse chromaticities of white
dwarfs", RASTI 5, 1 to 34, doi:10.1093/rasti/rzag069. Open access, CC BY 4.0, so its tables may be carried in a
pack with credit.

It has two halves.

**The method (its section 2 and 4).** Colour to the eye from a spectrum: integrate over the CIE 1931 colour
matching functions to get X, Y, Z; convert to linear sRGB; scale the brightest channel to 1; apply the sRGB
transfer curve. A correlated colour temperature from the 1960 u, v plane. A picture of the star's disc by
fitting a Claret four-term limb-darkening law to each of R, G and B, so the colour can shift towards the limb.

**The physics (its section 3).** Most white dwarfs look like blackbodies, and the median nearby white dwarf is
almost exactly white (6768 K, #F9F5FF). But the coolest have atmospheres as dense as a liquid (up to about
1 g/cm³), and there broad absorption eats whole parts of the visible spectrum:

| What is in the atmosphere | Where it comes from | What it does | Colour |
|---|---|---|---|
| Pure hydrogen or pure helium | the star itself | follows the blackbody track closely | orange, white, blue |
| Helium with a trace of hydrogen, below about 5000 K | the star itself | H2-He collision-induced absorption removes the red | yellow to cyan and blue |
| Carbon (C2 Swan bands), shifted and distorted above 0.15 g/cm³ (shift = 0.2 eV per g/cm³) | dredged up from the core | removes green | purple, pink, salmon |
| Sodium, calcium, magnesium, with lines broadened to hundreds of nm | rocky debris from its own planets falling in | Na removes orange, Ca removes violet | yellow, green, vivid blue |

Only about 3 in 100 nearby white dwarfs leave the blackbody track, so the pack must make these rare, not common.

**Why it is a good choice for SSE4.** The last row. SSE4 already dates the moment an aged system's asteroids
scatter inward and "rain metals onto the white dwarf" (`sse-time/src/epoch.rs`). With this paper the star's
colour becomes a consequence of the system's own history: "why is this star green?" is answered by "because its
asteroid belt fell into it 2 Gyr ago". That is the "why is this here?" idea applied to a star, and it gives the
academic what-if for free: change what fell in, and the colour changes.

**What would change on screen.** The white dwarf's dot on the system chart and in the sky, its disc on the Body
screen (with a colour shift at the limb), its swatch on cards and in Sizes, the Eye mode of the light switch,
and its spectrum chart (a wide trough at sodium instead of a smooth curve).

## 2. Feasibility, step by step

Short answer: **feasible, and a strong choice, but not with what is on main today, and not with zero new
blocks.** The colour half of the paper is already in SSE4. The physics half needs four general pieces that do
not exist yet. Three of them are already owed by planned work. If they are built first, then on camera Orac
truly adds no code: only wiring, data, citations and tests.

### 2.1 What already exists

| Need | State on main | Where |
|---|---|---|
| White dwarfs with mass, radius and cooling to a surface temperature | built | `sse-formation/src/lifeline.rs`, `sse-time` (cooling uses Mestel's law with A = 4, on the fudge register as F-S23) |
| Debris falling onto a white dwarf as a dated event | built, as a record event with sources | `sse-time/src/epoch.rs` |
| Light as a spectrum (thermal, reflected, power law, disc, named lines) | built | `sse-light`, pack `base-light`, op `read_spectrum` |
| Sodium D, calcium H and K, magnesium b in the line table | built, as narrow lines with a strength by temperature | `packs/base-light` (`lines`, `absorption`; a stand-in until O4) |
| Spectrum to CIE XYZ to sRGB, brightest channel scaled, transfer curve | built, and it is the paper's section 2 step for step | `packages/ui-contract/src/light.ts` (`xyz`, `linearSrgb`, `chromaticity`); TypeScript, not a block |
| H, He, Li, C, Na, Mg, Ca on the data sheet | built | `packs/base-reference` (118 elements) |
| Look up block onto the data sheet | built | `sse.reference.lookup` |
| Boxes (several blocks wired and used as one), with worked examples the kernel runs on load | built | `sse-flow/src/boxes.rs` |
| Logic blocks: add, subtract, multiply, divide, look up in a table, select, clamp, sum | built | `sse-logic/src/blocks.rs` |
| Orac drafts from a sentence, every draft checked by the kernel, nothing changes without Accept | built | `packages/agents` (A2 parts 1 to 4) |
| Add a pack from a file, with a preview of every value that changes | built | Packs screen (U5) |
| App fully usable with no key | built | A1 |

### 2.2 What is missing

| # | Gap | Why it matters | Kind of work | Already planned? |
|---|---|---|---|---|
| 1 | **A star has no atmosphere in the sense a planet has one.** Planets carry a gas mix, pressure and density; a white dwarf carries none of these. The star census says "Sketch (mass, cooling, no type)". | Everything in the paper hangs on gas mix, traces and density | kernel: the same atmosphere facts on any body (the one-body rule), not a white dwarf special | yes, B1 one body and B3 |
| 2 | **No gas has a colour yet, on any body.** This is the same mechanism as a planet's air having a colour: gas removes wavelengths, more widely as it gets denser. SSE4 does Rayleigh scattering per gas (`sse.air.*`) and narrow named lines, but no gas carries absorption bands, so neither Neptune's methane blue nor a white dwarf's sodium green can be computed today. | The whole effect is gas absorption | **one general piece** (section 2.6): absorption features as cited data sheet rows per gas, and one block that applies any gas mix to any light, reflected or the body's own | partly, O4 line-strength grid and the renderer's air |
| 3 | **A spectrum cannot travel on a wire.** Flows carry numbers, flags, lists and records; no block takes a spectrum in and gives one out. Stars on screen are a blackbody stub (`light.ts` says so). | Without it the chain cannot be shown as blocks, which is the central shot | kernel (a spectrum shape and three or four general light blocks: thermal continuum, absorb a broad feature, colour to the eye, limb law) | "colour to the eye" as a block is already a candidate on the fudge register |
| 4 | **No maths beyond arithmetic in a box.** No power, exponential or logarithm block. A relation like the Swan band shift (0.2 × density) can be wired today; a Planck curve or a line profile cannot. | Limits what any paper can become without Rust | kernel (design decision, see question 2) | no |
| 5 | **Orac cannot take a paper.** It takes one sentence. No file intake, no text extraction. | Step one of the video | agents (PDF text in the browser, sent as labelled data; the context guard already refuses a model that cannot hold it; this paper is about 45k tokens) | no |
| 6 | **Orac changes parameters; it cannot add or rewire nodes in a flow.** A2's log: "adding or rewiring nodes waits for the editor". It can draft a box, but not connect it to anything. | The new chain must feed the star's light | agents and chart room (flow editor) | yes, noted in A2 |
| 7 | **Orac drafts one kind of thing per card, into separate campaign packs.** It cannot plan one job as a set (boxes, table rows, sheet rows, a flow change) and it cannot draft light-table rows or data sheet rows. | A paper is always a set | agents | sheet rows are MP11; the rest is new |
| 8 | **A user's pack cannot carry its own fudges, citations register or a reproduction report.** Today those live in `docs/fudges.md` and `docs/citations.md` in the repo. | An academic needs these to trust a pack they did not write | kernel (pack manifest) and Packs screen | no |
| 9 | **Orac's work cannot be exported as one named pack.** It lives inside the campaign save. Only lenses export as a pack today. | "Save it as a pack" | chart room, small | no |
| 10 | **Explorers cannot publish or fetch V4 packs.** | "Other users load it" | Explorers thread | yes, X2 and TG15 pack shelves |
| 11 | **Limb darkening is one grey law** (Eddington) in the renderer. | The disc picture with a colour shift at the limb | renderer, small | no |

### 2.3 "No new blocks": the honest reading

Alex's belief was that this could be built from existing physics with no new blocks. Tested against main:

- **True for the colour science.** Nothing new is needed to turn a spectrum into a colour.
- **Not true for the physics today.** Gaps 1 to 4 are new blocks or new shapes. None is a white dwarf
  special: gap 2 is the same mechanism as a planet's air having a colour (section 2.6). But all four are general (they
  also give magenta brown dwarfs from the same sodium line, carbon stars, titanium oxide in red dwarfs, hazy
  planets), three are already owed by B3 and O4, and none is specific to this paper.
- **So the claim the video can honestly make is:** "no new code for this paper". By filming time the general
  pieces exist; Orac reads the paper and adds wiring, tables, sheet rows, citations, fudge entries and tests.
  That is a stronger and more checkable claim than "no new blocks", and a viewer can verify it: the pack is
  a data file.
- **One condition:** B3 must build the atmosphere facts (gas, traces, density) and stop there. If B3 also
  builds coloured white dwarfs, there is nothing left for Orac to do on camera.

### 2.4 How faithful can it be?

The paper's colours come from full model atmospheres (the Koester code, unified line-shape theory for the
sodium and calcium wings) and from observed spectra. SSE4 cannot run that in a browser, and the paper does not
publish the spectra ("available on reasonable request"), only colours and star parameters.

So the pack is a **reduced model**: a continuum at the star's temperature, with a few broad absorbers whose
width and depth follow density and abundance, set from the cited sources (Allard et al. for Na and Ca in
helium, Blouin et al. 2019 for the Swan band shift, Borysow et al. 2001 and Blouin et al. 2017 for
collision-induced absorption). Its worked examples are the paper's own stars:

| Check | Paper's value |
|---|---|
| Sun-like model, 5800 K | #FFF1EC (255, 241, 236) |
| Pure hydrogen white dwarf, 10 000 K | (200, 218, 255) |
| WD J1330+6435, 4240 K, log Na/He −8.7, log Ca/He −8.8 | green, #D0FFB4 at disc centre |
| WD J2356−2054, about 4000 K, more sodium | vivid blue, off the blackbody track entirely |
| WD J1011+2845, 4300 K, carbon | salmon pink, #FF8775 observed |
| Share of white dwarfs off the blackbody track in a volume | 3.4 % (+1.7, −1.3) |

The pack states on its listing how closely it reproduces them, and the reduced model goes on the fudge
register with what would replace it (tabulated model spectra). Expect the right hue and the right trend, not
the exact hex. Saying so on screen is part of the pitch to academics, not a weakness.

### 2.5 What an academic needs in order to trust a loaded pack

1. Every block opens to its formula and its source with a DOI (built: "Why?").
2. The pack's own fudge list and what each stands in for (gap 8).
3. A reproduction report: the paper's published numbers beside the pack's, run by the kernel on load, not
   claimed by the author (boxes already run worked examples on load; gap 8 adds the report).
4. A reach report before accepting: which stars in this universe change, before and after (built for packs
   as the Packs screen preview; the data sheet's reach report is MP1c).
5. The same answer on every machine: the pack's hash and the kernel's state hash (built).
6. Honesty marks: real, authored, fictional (built, GM view).
7. The what-if stays on a branch until kept (built: what-if branches).

### 2.6 One mechanism, thought wide (Alex, 22 Nov)

Alex asked whether a star's broad absorption is a different mechanism from a planet's air having a colour. It
is not. Both are gas removing wavelengths from light passing through it. Only two things differ, and both are
inputs, not mechanisms: where the light comes from (reflected starlight for a planet, the body's own glow for
a star or brown dwarf), and how dense the gas is (which widens the features). His ruling: build it as one
extensible capability under "processes, not classes", and in design think wide about other uses so that
nothing has to be reworked later.

So ND2 is one block, "light through gas", that takes any light, any gas mix and the conditions (temperature,
density, how much gas is in the way) and gives the light that comes out. The per-gas features are reference
rows on the data sheet, cited, editable, and addable by a pack. Nothing in it names a kind of body.

What the same piece serves, beyond this paper:

| Use | Gas and feature | Who reads it |
|---|---|---|
| Colour of a planet's disc | methane bands (Uranus, Neptune), sodium and potassium on hot giants, sulphur compounds | renderer, Eye mode, surface colour (S19) |
| Colour of the sky and the sun from the ground | the same rows, with the Rayleigh scattering already built | renderer air (R2b), `read_air_optics` |
| Brown dwarfs | the same broadened sodium line that makes a white dwarf green makes them magenta | one-body work (B1), light |
| Cool and carbon stars | titanium oxide in red dwarfs, the same C2 bands in carbon stars | O4, star disc |
| Dense hydrogen air | collision-induced absorption, which grows with density squared: giant planets, Titan, cool white dwarfs | climate (it is also a greenhouse term), light |
| Reading an air from afar | a transit shows which wavelengths the air removes | planet finding (O13), sensing (O9), the knowledge ladder |
| Fictional gases | a pack adds a gas with its own bands and every view shows it | universe packs, MP9 |
| Light through a nebula or dust lane | the same block with gas between source and observer | nebulae (O3 on), night sky |

Edge cases the design must hold from the start, so they are not bolted on: light passing through gas once
(a star's own glow) or twice (reflected off a surface or cloud deck); features that depend on a pair of gases,
not one (collision-induced); width and shift that follow density, not only pressure; gases with no known
features (say so, do not invent); and a body whose "surface" is a level in the gas (giants, stars).

## 3. Proposed packages (track ND)

One package each, with its owner thread (handed out 22 Nov). ND1 to ND3 are general kernel work that should ride with B3 and O4 when
those threads reach them; ND4 to ND8 are the capability itself; ND9 and ND10 are the proof.

| Id | Package | Owner | Needs | Acceptance |
|---|---|---|---|---|
| ND1 | Atmosphere facts on any body that holds gas, stars included (the one-body rule): gas mix, traces, pressure and density where the light leaves. For a white dwarf the traces are fed by the dated debris event and by core dredge-up. Facts only; no colour consequence | Build SSE4 proven core (inside B1 and B3) | B1 | a polluted white dwarf carries the elements of what fell in; a planet and a star answer the same atmosphere read; spectral type is a label read off the facts |
| ND2 | Gas absorption as one general mechanism (section 2.6): absorption features per gas as cited rows on the data sheet (bands, collision-induced pairs, how width grows with density); a spectrum on a wire; general light blocks: thermal continuum, "light through gas" (any mix, reflected light or the body's own glow), colour to the eye (CIE, moved from `light.ts` into the kernel as a fact), limb law per channel. No body type is named anywhere in it | Build light and nebulae (with O4 and the air optics), data rows with the matter data sheet thread | O1, O12, MP4 | the Sun and a 10 000 K hydrogen white dwarf reproduce the paper's section 2.4 values; the same block gives a methane-rich giant its blue-green; a star on screen reads its colour from the kernel, not the blackbody stub |
| ND3 | Maths in a box, exactly as `SSE4-equation-block-design.md`: power, exponential, logarithm, root and trigonometry blocks, and the Equation block (a formula as pack data, units checked on load, constants only through Look up, sourced, with worked examples) | Build kernel resolve and rules | J3, MP17a | that design's section 10: a Planck curve as an Equation matches `sse-light` to the last bit; a unit mismatch is refused at load in words |
| ND3b | Move the existing one-formula Rust atoms into named Equations in their base packs, family by family, as a shrinking list; ids, ports, results and golden hashes unchanged (equation design section 8A; accepted by Alex 22 Nov) | Build kernel resolve and rules, with each physics thread for its family | ND3 | every moved block gives identical bits; a keen amateur can follow a base flow from its block names |
| ND4 | Orac reads a paper: PDF or text dropped on the Orac tab, text extracted in the browser, sent as labelled data; Orac first answers with a plan in words (what the paper claims, what exists, what it will add, what is missing) before drafting | Build Orac, Minerva and Tars | A2 | nothing in the paper is treated as an instruction; a model too small for it is refused before sending; the plan names every missing piece |
| ND5 | Orac assembles one job as a set: boxes, a flow change that adds and wires nodes, light-table rows, data sheet rows (MP11), each kernel-checked, accepted together or not at all | Build Orac, Minerva and Tars, with the chart room's flow editor | ND4, MP11 | an invalid set never reaches Accept; one Undo withdraws the whole set |
| ND6 | Packs carry their own citations, fudge list and reproduction report; the Packs screen shows them before the pack is added | Build kernel resolve and rules, chart room | K5 | a pack whose worked examples fail is refused with the numbers side by side |
| ND7 | "Save as a pack": Orac's accepted set exported as one named, versioned pack file, with author, licence and what it cost to make | Rebuild the chart room's top level | ND5, ND6 | the file loads on a fresh device with no key and gives the same state hash |
| ND8 | Publish to and fetch from Explorers, hash verified (this is X2 and TG15; listed so the dependency is visible) | Integrate Explorers site with SSE4 | X2 | publish, fetch, hash verified |
| ND9 | Rehearsal on a different paper, to prove it is not a one-off; its gaps go back into ND2 to ND5 | Build Orac, Minerva and Tars | ND1 to ND7 | a second pack made with no kernel change |
| ND10 | Film video G on AC-ProArt with a real key: a real run, real cost on the ledger, time compressed with the clock visible | Capture footage for SSE4 trailers | ND1 to ND9, renderer star disc, spectrum chart | no shot faked; the pack used in the film is the one published |

Order: ND1 to ND3 whenever B3, O4 and the kernel thread reach them. ND4 to ND7 after MP11. ND8 with X2. ND9
and ND10 last, when most visual effects are in place (Alex's word).

## 4. Storyboard (video G, about 2 to 3 minutes)

Working title: **"Teach it new physics."** Pitch line: "From a paper to physics you can see, check and change."
The central shot is 5: the paper as a row of simple blocks.

| # | Shot | Caption idea | Depends on |
|---|---|---|---|
| 1 | An aged system. Its white dwarf is a plain blue-white dot. Why? says "a blackbody at 4200 K" | "SSE4 thinks this dead star glows like a hot coal. A paper from September 2026 says otherwise." | built (U4 look-ahead, Why?) |
| 2 | The paper's first page, then dropped onto Orac. One sentence typed: "Bring this in." | "Hand Orac the paper." | ND4 |
| 3 | Orac's plan in words: what the paper claims, the blocks it will use, the data it will add, what SSE4 lacks. The cost ticker runs | "It reads. It plans. It tells you what it cannot do." | ND4 |
| 4 | A draft comes back refused by the kernel (a unit that does not match, or a source with no DOI). Orac fixes it | "Every draft is checked by the engine, not by the AI." | built (A2 loop), ND5 |
| 5 | **The chain opens as blocks**: what fell in → atmosphere (gas, traces, density) → broad absorbers (sodium, calcium, carbon) → spectrum → colour to the eye. Click one: its formula, its inputs, the paper and DOI. Switch Curious, Science, Engine | "The paper has not vanished into a codebase. Here it is." | ND1, ND2, ND5, flow editor |
| 6 | The reproduction report: the paper's stars beside the pack's, swatch against swatch, with the stated tolerance and the fudge list | "It shows its working, and where it falls short." | ND6 |
| 7 | Accept. The reach preview: which stars in this universe change, before and after | "Nothing changes until you say so." | ND5, Packs preview |
| 8 | Payoff: the white dwarf turns green. Its disc, with the tint shifting at the limb; its spectrum with the sodium trough. Why? "Its asteroid belt fell into it 2.1 Gyr ago", and the ruler jumps to that event | "Why is this star green? Because it ate its planets." | ND1, ND2, renderer star disc, spectrum chart (O12 in the shell) |
| 9 | By hand, no AI: drag the sodium abundance up. Green slides to blue, the stars the paper predicts between its two examples. Then change what fell in | "Now ask your own what-if." | ND5, flow editor |
| 10 | Save as a pack. The listing: blocks added, tables, sources, fudges, and "cost to make: $X, once" | "One person paid for the thinking." | ND7 |
| 11 | A second browser with no key. Fetch from Explorers, add, same star, same green, same hash | "Everyone else just loads it." | ND8 |
| 12 | End card | "Star System Explorer 4. Physics you can read." | |

Filmable today: shot 1, and the refusal-and-fix half of shot 4 (as in clip 19). Everything else waits.

Rules that bind the film: nothing faked, so Orac really does the job on camera with a real key and the real
cost shown; the pack in the film is the pack published; the reduced model is named as one on screen.

## 5. Questions for Alex: all answered yes, 22 Nov

Kept as asked, with the answers. Questions 3 to 7 were accepted together at 16:03 ("agree to your proposals").

1. **Accept "no new code for this paper" in place of "no new blocks"?** The demo needs general pieces built
   first (ND1 to ND3). Recommended: yes. It is the honest claim and still one nobody else can make.
   **Accepted by Alex, 22 Nov: "Yeah! Perfect... blocks are NOT code."** The film's claim is "no new code
   for this paper"; say on screen that blocks are not code.
2. **A general formula block?** One block whose formula is pack data (an expression over its inputs, with
   units, a source, and the kernel's own deterministic maths), so a new relation never needs Rust. It changes
   the standing rule "atoms are Rust; new base blocks are raised as GitHub issues". Recommended: yes, as a
   short design first (how it shows its formula, how it is cited, cost limits), because without it "bring in
   any paper" quietly means "any paper whose maths we already shipped". This matters most for academic use.
   **Accepted by Alex, 22 Nov: "blocks should be able to do basic math! power, exponential or logarithm
   blocks are valid... or just have a general purpose equation, which is what I thought we had."** The design
   is `SSE4-equation-block-design.md`; ND3 builds both the maths blocks and the Equation block.
3. **Hold coloured white dwarfs out of B3** so that Orac does it on camera, with B3 building only the
   atmosphere facts? Recommended: yes.
4. **A reduced model with a stated tolerance, on the fudge register**, since the paper's full atmosphere
   models cannot run in a browser? Recommended: yes, with the paper's own stars as the worked examples and
   the hue and trend as the acceptance, not the exact hex.
5. **Rehearse on a second paper before filming (ND9)?** Recommended: yes. A capability shown once on a
   hand-picked paper will not convince an academic; twice does.
6. **Write to the authors near filming time?** They offer model spectra on request, which would tighten the
   reproduction, and a pack built from their paper is a courtesy to mention. Recommended: yes, from you, once
   the pack exists. Not before.
   **Accepted.** Alex also asked whether to write now; the advice given was to wait until there is a rough
   working result to show. **Licence point:** the paper and its tables are CC BY 4.0, but spectra the authors
   send on request are not covered by that licence. Anything derived from them may ship in a pack only with
   the authors' explicit permission, recorded with the pack's sources. Without it they may be used to check
   the pack privately, not to fill it.
7. **Packs carry their own fudges, sources and a reproduction report (ND6)?** Recommended: yes. It is the
   piece that turns a shared pack from "trust me" into something checkable, and it serves every pack, not
   only this one.

Questions 3, 4, 5 and 7: **accepted by Alex, 22 Nov**, as recommended.

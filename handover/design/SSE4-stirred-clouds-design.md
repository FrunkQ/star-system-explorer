# SSE4: clouds stirred by a giant's own winds (track WS)

Status: **ACCEPTED by Alex, 27 Nov 2026 ("agree with all", 10:45 UTC).** His answers and one change are in section 11A and win where they differ from the text. Written as a proposal at the coordinator's word for Alex's ask of 26 Nov ("use physics blocks and be able to say why does this look stormy?", with a Juno picture of Jupiter's pole). Design only; nothing here is built. WS is a code only, never shown in the product.

Read with: `handover/packages/VP-A2.md` (what PR 646 put on main), `SSE4-visual-polish-design.md` sections 5.4, 6 and 8 (the layers, the detail ladder, the tiers), `SSE4-weather-design.md` section 3 (giants), `crates/sse-weather/src/giants.rs`, `packages/renderer/src/giant-detail.ts` and `true-air.ts` (`flowFace`), `SSE4-edits-reach-design.md` (ER11a, baked tables), the picture clock (`packages/ui-contract/src/picture-clock.ts`).

**On sources.** The paper search is out until 1 Dec 2026. Every paper figure here is from memory and **unverified** (section 13). The hand checks in section 5 use the fixtures' own numbers plus figures quoted from memory.

---

## 1. The short answer

1. **Today's swirls are painted on.** PR 646 winds each storm round by one fixed angle and wrinkles the band edges with noise. Both are in the fudge register, and both name this package as the better way: "cloud tracers advected through the weather block's wind field" and "eddies resolved by the weather block".

2. **The kernel's winds cannot fold cloud yet, because they never change.** Jets plus a handful of steady storms shear cloud into straight threads and tidy spirals. The folds and filaments in Juno's pictures come from many small eddies that are born, drift and die. So the first job is in the kernel: give a giant its eddies.

3. **Eddies are the storms we already have, made small.** Same vortex, same shear law, same keyed draws by era window, so any date gives the same eddies. Their sizes run from the deformation radius up to the band's width. Their speed comes from the heat flowing through the air (one new block). An eddy turning against its band's shear is torn apart in one shear time, by the law PR 646 already uses for the ovals' shapes. One mechanism from the Great Red Spot's kind down to the smallest curl.

4. **Heat from below makes fresh cloud.** Small bright storms ("plumes") are numbered by the internal heat they must carry. By hand that gives Jupiter about 70 at a time, Saturn a few, Neptune one about a third of the time and Uranus almost never. That is the real difference between those four, from one rule.

5. **The picture is a pure function of the date.** Cloud lasts about as long as its particles take to fall out (a block; about two weeks on Jupiter by hand). Each pixel is followed backwards through the winds for that long and takes the cloud that was made where it came from. Nothing is stepped forward from frame to frame, so the clock can jump anywhere, run at any rate, or stop, and the picture is right for that instant.

6. **Kernel and renderer split as ruled.** The kernel gives the flow as a table (jets, storms, eddies, plumes, renewal time) and a reference answer for any small patch. The renderer follows the table on the GPU and looks colours up. A test holds the two together.

7. **"Why does it look like this here?"** is a kernel read. For any place on any giant it compares four rates (the band's shear, the eddies' turning, the cloud's renewal, the plumes from heat) plus the spin, and says which wins, in words, with the numbers and the blocks they came from. The same numbers drew the picture.

8. **Every device shows the stirred picture.** Low-power devices build it once as a still and do not animate it (section 11A). Colours are untouched: the stirring moves the colours the reference pack already gave.

9. **Six small packages** (section 9). **None moves `tests/golden/contract.jsonl`.** Three re-bless the giants' area goldens.

---

## 2. What the code does today (main at `64698389`, PR 646 merged)

| Piece | Where | State |
|---|---|---|
| Bands, jets, polar regime, polygon | `sse-weather/src/giants.rs` `layout`, `zonal_wind` | Jets from the Rhines scale with a pack eddy speed (17 m/s gas, 100 m/s ice, the second held so ice giants keep 3 jets: a registered fudge) |
| Storms | `giants.rs` `family`, `polar_systems` | Ovals, strings, dark spots, outbreaks, polar cyclones, drawn per era window by index, drifting with the local wind. A Gaussian vortex each. Rates are pack constants; internal heat only picks the polar regime |
| Oval shape | `sheared_aspect` | Moore and Saffman's ellipse in the band's shear |
| Wind at a place | `giant_cell` | Eastward jet wind plus each storm's turning. Near a polygon the cloud edge follows the polygon but **the wind stays a circle** |
| Eddies | nowhere in the kernel | Only as `eddy_m_s`, one number that sets the jet count |
| Cloud renewal | nowhere | Decks are steady by latitude |
| Drawn face | `giant-detail.ts` `detailGiant` | Tiles and patches at 1024 × 512; storms wound by one angle (fudge); band-edge curls from ladder noise (fudge) |
| Motion | `true-air.ts` `flowFace` | Rows slide at their jet's speed; storms turn with two blended phases. Off the campaign clock (CLK) |
| "Why" | `GiantLayout.why`, `WeatherSystem.why` | Good sentences for the bands, the poles and each storm. Nothing for the look of a place |

---

## 3. The model

Each numbered piece becomes one small block (section 8). Symbols: `a` radius, `Ω` spin rate, `f = 2Ω sin φ`, `H` scale height, `L_d = N H / f` the deformation radius, `g` gravity.

### 3.1 One stream function for the winds

All of a giant's winds come from one function ψ(place, date), with eastward wind `−∂ψ/∂y` and northward `∂ψ/∂x`. Winds made this way never pile cloud up or thin it out, so carrying cloud through them keeps its area.

- **Jets:** ψ is a function of latitude alone, giving today's `zonal_wind` unchanged.
- **Polygon jet:** ψ is a function of the distance from the polygon's edge (`polygon_edge_km`), not from the latitude circle. The wind then runs **along** the hexagon's sides. Today it runs in a circle, and stirring would smear the hexagon round in one turn. Saturn's real jet does follow its hexagon.
- **Vortices:** ψ is the weather block's Gaussian vortex written in the vortex's own ellipse, so the wind runs along the ellipse. Today the turning uses the round direction inside a stretched distance.

### 3.2 How fast the eddies are: the heat

```
power to eddies   ε = η · F · g / p₀                (W per kg of weather layer)
eddy speed        u(L) = C · (ε L)^(1/3),  capped at the speed of sound
```

`F` is all the heat the air sends out (the kernel's internal flux plus absorbed sunlight), `p₀` the weather layer's pressure (1 bar, a pack row), `L` the eddy's width. `η` is the share of that heat that ends up as eddy motion. The one-third law is the standard result for turbulence that passes energy from small eddies to large ones.

`η` and `C` go in as one fitted row: **set so Jupiter's eddies at band width run at the 17 m/s its bands already use** (about 4 per cent with C = 3). Measured eddy power on Jupiter is of that order (Salyk et al. 2006, from memory). Every other giant then follows from its own heat.

### 3.3 The eddies

Drawn exactly as the storm families are (per era window, keyed by the window's index and the world's address):

- **Size:** widths from `L_d` at that latitude up to the band's width, a fixed share of each band's area per doubling of size (pack row). Where `L_d` is wider than the band there is one size.
- **Speed:** 3.2 at its width.
- **Sense:** either, drawn.
- **Shape:** `sheared_aspect`, as the ovals.
- **Life:** a pack number of its own turns. **But** an eddy turning against the shear, past the edge where no steady shape exists (ζ/ω below 2√2 − 3, already in `sheared_aspect`), lives one shear time 1/|ζ| and is drawn stretching out as it dies.
- **Place and drift:** inside a band, as storms are placed; drifts with the wind at its centre, as storms do.
- **Fade:** strength rises and falls smoothly over its life, so nothing pops.

Eddies are too many and too brief to name. They are not in the `systems` list (the rule "special bodies tracked; gravel is a sum"). Grade: **drawn from the numbers**, like a storm's `evidence: "drawn"`.

Below `L_d` the air stops behaving as a flat sheet. That is the floor: nothing smaller is made, and the detail line says so in kilometres.

### 3.4 Fresh cloud: plumes numbered by the heat

Internal heat reaches the cloud tops through tall storms. If each carries a set power, the heat fixes how many are alive:

```
plumes alive   N = s · F_int · 4π a² / (F_p · π r_p²),     r_p = L_d / 2
```

`F_p` is the heat flux inside an active plume and `s` the share of internal heat that plumes carry: two pack rows (unverified; Gierasch et al. 2000 from memory). Each plume is a short-lived bright patch of the top deck, drawn by window as above, placed where the band's shear is cyclonic (where Jupiter's lightning is seen; Little et al. 1999, from memory), lasting one turn of an eddy its size. It adds cover exactly as today's `outbreak` does, at a smaller size. The winds then pull it into a tail.

The count's **ratio between worlds** does not depend on `F_p`: it follows from the internal heat and the spin.

### 3.5 How long cloud lasts

```
fall speed     v = 2 r² ρ_p g / (9 μ)            (Stokes, with the slip correction at low pressure)
renewal time   τ = deck thickness / v
```

`r` is the deck's particle radius (already in the pack per condensate), `ρ_p` its density and `μ` the gas viscosity (data sheet; rows to add if missing), thickness the pack's `thickness_h · H`. Where no radius is held, a registered fallback radius.

### 3.6 Carrying the cloud: backwards, for one renewal

For a pixel at place `x` and date `t`:

1. Follow the wind **backwards** from `(x, t)` to the date the cloud there was made, `t − age`. That gives where it came from, `X`.
2. The pixel takes the cloud made at `X`: the banded decks, the storms' cores, the plumes. That is today's 1024 face before any winding or curls, read at `X`.
3. Cloud is made in overlapping **generations** on a fixed lattice of dates, `τ` apart, anchored at the campaign epoch and offset from region to region by a keyed draw. A generation fades in from age 0, is strongest at age `τ`, and has faded out at `2τ`. Two are alive at any place, their weights summing to one. The mean age of the cloud on show is `τ`.

Because the lattice is anchored and the winds are known for any date, **the picture at a date depends only on the date.** Reading it at `t` directly, or playing the clock there, gives the same picture.

Two details the builder needs:

- **Storms keep their shape.** The source face is read for today's date, with the storms where they are now. A point that came from inside a storm is read where that storm stands now (shifted by the storm centre's drift over the age); a point outside is shifted by its own row's drift. The two are blended by the storm's Gaussian. Without this the Great Red Spot's kind would be sheared into a slab.
- **Threads thinner than a pixel** are read through the face's coarser levels, by how far the backward map has stretched there. Cloud stirred finer than the picture can show comes out as the local mean, which is what the eye would see.

Inside a big storm the same step winds the bands into a spiral by the storm's own winds for the cloud's real age. That replaces PR 646's fixed winding angle.

### 3.7 Why it looks the way it does

At a place, five numbers, all from the blocks above:

| Number | From | Says |
|---|---|---|
| Band shear rate, `|ζ|` | `zonal_vorticity` | how fast cloud is drawn into straight threads |
| Eddy turning rate, `u(L)/L`, and the share of eddies torn | 3.2, 3.3 | how fast cloud is folded and curled |
| Renewal rate, `1/τ` | 3.5 | how fresh and lumpy the cloud is before it is stirred |
| Plumes per million km², and internal heat | 3.4 | how much new bright cloud heat from below makes |
| Spin: `L_d`, band width, Rossby number `u / (f L)` | layout | how big the curls and bands are |

Plus, for the place itself, whether turning or stretching wins there (the Okubo-Weiss measure: stretching squared minus spin squared). Inside a storm turning wins and the cloud is held as a spiral; between storms stretching wins and it is drawn out.

The kernel builds the sentences, as `layout` and `family` already do. Two worked by hand from section 5:

> **Jupiter, a belt at 20°S.** "Stormy and folded. Heat of 13.6 W/m² drives eddies of 9 to 17 m/s, 1,400 to 8,500 km across. The belt's winds shear at one turn in 16 hours, faster than the eddies turn (5 to 18 days), so every eddy turning against the shear is torn into threads and those turning with it are stretched to twice their width. Cloud lasts about 15 days, long enough to be folded several times. About 70 bright plumes are rising at once, carrying the 5.4 W/m² that comes from inside."

> **Uranus.** "Calm. Only 0.7 W/m² flows through the air and 0.04 of it from inside, so eddies run at about 3 m/s under winds of 400, and a new bright plume appears about one week in fifty. What cloud there is, is drawn into long thin threads along the bands."

Each number in a sentence carries its block's id, so the card can open "Look up" on it, and an edited block or pack row marks the sentence as resting on an edit (ER).

---

## 4. The clock

- The one time source is the campaign clock (`ws.clock`, `clockAt`). No rate lives in the renderer. Stopped means a still picture.
- The kernel's flow table covers a span of dates. The renderer asks again when the clock leaves it, as it re-reads the face today when the day changes.
- At rates where a frame is longer than the cloud lasts, successive pictures share nothing. Each is still right for its instant, and the note says "the clouds change faster than the pictures", per the CLK rule. Nothing is slowed.
- Dates are read as the storms read them today (days as f64, about 30 s fine at a star's age). The time pipeline (TP) changes that for everyone together.

---

## 5. Hand checks

From the giant fixtures (`tests/fixtures/weather/giants.json`) and the pack's N = 0.0148 s⁻¹, at 45° latitude. Absorbed sunlight and the plume flux are from memory.

| | Jupiter | Saturn | Uranus | Neptune |
|---|---|---|---|---|
| Heat out, W/m² (internal + sunlight) | 5.4 + 8.2 | 2.0 + 2.6 | 0.04 + 0.65 | 0.43 + 0.27 |
| `L_d`, km | 1,430 | 2,960 | 2,830 | 1,990 |
| Eddy speed at band width, m/s | 17 (fitted) | about 8 | about 3 | about 3 |
| Jet scale, m/s (pack) | 150 | 150 | 400 | 400 |
| Plumes alive (`s` = 1, `F_p` = 3 kW/m²) | 69 | 4 | 0.02 | 0.35 |

Jupiter in more detail: band width 8,450 km (26 bands); eddies 9.4 m/s at `L_d`, turning in 5.5 days, and 17 m/s at band width, turning in 18 days; band shear about 1.8 × 10⁻⁵ s⁻¹ (16 hours); against that shear a band-wide eddy has ζ/ω = 1.4, so one turning with it is stretched to 2.9 times its width and one turning against it is torn; 5 µm ammonia ice falls at about 2 cm/s, so the deck renews in about 15 days.

What the checks say: Jupiter folded and busy; Saturn gentler with rare big plumes; the ice giants threaded, with Neptune now and then bright and Uranus bland. That is the right order for the four, from heat and spin alone.

---

## 6. Where it runs

| | Kernel (`sse-weather`, the `weather` op) | Renderer |
|---|---|---|
| Flow | `want: "flow"`: the jets finely by latitude, the polygon, every storm, eddy and plume alive in a span of dates (place, drift, size, shape, speed, sense, born, ends), the renewal time and lattice, as one table | Sums the table's vortices at a point; steps backwards along it |
| Truth | `want: "stirred"`: for a patch (as `patch`, up to 64 across) the place each cell's cloud came from and each generation's weight | A test holds the GPU's answer to this within a pixel |
| Source | `field` and `patch` as today, plus each plume's centre cell in the flow table | Composes colours with `composeAt` as today; lays each plume by its own footprint |
| Why | `want: "look"` at a place: the numbers, the winner, the sentences | Shows them |

The table uses `sse.table/1` and the `bake` op if ER11a is on main when WS2 starts; otherwise it is a plain weather response and moves to the bake door in ER11b's sweep.

The one sum the renderer holds is "add up the kernel's vortices and jets at a point". It is geometry over a kernel table, fenced by the test above, and is entered in the fudge register as the place where the two could drift.

The new `want` names are left out of the refusal's list, as `glint` is, so the core golden's `bad_want` line does not move.

### Tiers

| Tier | Stirring |
|---|---|
| `text` | none |
| `canvas` | the still of section 11A, built on the processor |
| **B** `gpu-basic` (phones) | **a stirred still**, built without being asked (section 11A). Not animated |
| **H** `gpu-high` | on. Face 1024 wide; eddies two pixels and wider (the ladder's rule), faded in over a factor of two |
| **U** `gpu-ultra` | face 2048 wide; every eddy down to `L_d` |
| **S** at rest | builds the finer face over 0.5 to 3 s |

The governor may drop the live pass under load; the picture falls back to the last stirred still, never to an unstirred one. The size comparison (every giant, each frame, 256 wide) stirs only at rest. PR 646's fixed winding and the ladder's `giant.curls` are switched off everywhere once WS5b lands; until a still is ready (a second or so) the unstirred face shows and the stirred one fades in.

Cost by estimate, to be measured in WS5: about 40 backward steps, some 8 vortices near each, two generations, half a million pixels: a few milliseconds on a desktop GPU, and only when the clock has moved.

---

## 7. What changes for whom

- **Every giant, any system:** eddies, plumes, renewal, and an answer to "why". Hot giants and brown dwarfs come out fully churned (large heat), slow spinners with a few wide eddies. Nothing is special-cased for Jupiter or Saturn.
- **Saturn:** the hexagon's wind follows its sides.
- **Explore:** the picture, and the "why" line on a giant's card: tap a place in True, read why. **Edit (Build):** the same, with the blocks and pack rows a click away; an edit marks the line. **Play (Run):** players see the picture and the plain sentence only.
- **No new screen, no tab, no press budget change.** The whole-world "why" sentence shows on the giant's card without a press (section 11A); the place-by-place detail sits under the card's extended details.
- **GM's "what is interesting here":** the count of plumes and any torn band are there to be listed when that list is built.

---

## 8. Numbers and blocks

Pack rows, `packs/base-weather` `weather_giants.stirring` (all `unverified` until the citations check):

| Row | Value | Source |
|---|---|---|
| `eddy_efficiency` (η, with C = 3) | 0.04, fitted to Jupiter's 17 m/s | Salyk et al. 2006; Kraichnan 1967 |
| `weather_layer_bar` | 1 | convention |
| `eddy_area_share_per_octave` | 0.25 | stand-in |
| `eddy_life_turns` | 2 | stand-in |
| `plume_flux_w_m2`, `plume_heat_share` | 3,000; 1 | Gierasch et al. 2000 (from memory; to be read) |
| `fallback_particle_um` | 5 | stand-in |
| gas viscosity, condensate densities | data sheet rows, added if missing | sheet |

Formula blocks, family `sse.weather`:

| Block | Does |
|---|---|
| `sse.weather.stream_function` | 3.1 |
| `sse.weather.eddy_power` | 3.2, first line |
| `sse.weather.cascade_speed` | 3.2, second line |
| `sse.weather.torn_by_shear` | 3.3: the limit and the one shear time (shares `sheared_aspect`'s law) |
| `sse.weather.plume_count` | 3.4 |
| `sse.weather.particle_fall_time` | 3.5 |
| `sse.weather.turning_or_stretching` | 3.7, Okubo-Weiss |
| `sse.weather.giant_stirring` (Big Block) | the eddy and plume draws and the backward carrying of 3.6; calls the blocks above. Its view: the four rates by latitude beside the globe |

New numbers are read through the physics in force (ER), never as typed constants; `giants.rs` adds no lines to the side-door ledger.

Fudge register, new: generations on a lattice in place of continuous renewal; all decks carried together by the top deck's wind; one renewal time per giant; eddy area share and life; plume flux and share; eddies drift with the wind only (no Rossby drift, as today's storms); ice giants carry two eddy speeds (3.2's for stirring, the pack's 100 m/s for the jet count) until WS8. Retired on H and above: "A giant's storms and poles read closer, and turned" (the winding half) and "Detail ladder: giant's curls".

---

## 9. Packages

Builder: the thread that built PR 646 ("Draw giant planet storms and hexagons"). Status rows belong in `handover/status/true-air.md`, added by its owner. Each leaves main green. **None moves `tests/golden/contract.jsonl`**: the only giant line there is Jupiter's `systems`, which no package changes. If the reach check shows otherwise for any package, stop and take a core slot.

| Package | What | Needs | Golden | Size |
|---|---|---|---|---|
| **WS1** winds from one stream function | 3.1. `giant_cell`'s winds read from ψ; the polygon jet follows its sides; vortex winds follow their ellipse | nothing | area only: `weather-giants.json` field digests re-blessed (Saturn's polygon, sheared ovals) | S |
| **WS2** eddies from the heat | 3.2, 3.3 and their blocks; the eddy draws; `want: "flow"` (jets, storms, eddies) | WS1; ER11a if landed | area only: new lines | M |
| **WS3** renewal and plumes | 3.4, 3.5; plumes in `giant_cell`'s cover and in the flow table | WS2 | area only: `weather-giants.json` field digests re-blessed (plume cover) | S |
| **WS4** the carrying, in the kernel | 3.6 as `want: "stirred"`: the reference answer for a patch | WS2 (WS3 for the lattice's τ; a pack τ until then) | area only: new lines | S |
| **WS5** the stirred face | Renderer: the backward pass on the GPU on H and above, held to WS4 by test; the clock; tiers; fixed winding and curls off where it runs; fudge register; screens on AC-ProArt | WS3, WS4 | none (renderer tests) | M |
| **WS5b** the still for low power | Section 11A: one stirred face per giant on tier B and `canvas`, built unasked, rebuilt as the date moves; the governor's fallback; PR 646's winding and curls retired | WS5 | none (renderer tests) | S |
| **WS6a** "why", kernel | 3.7 as `want: "look"` | WS3 | area only: new lines | S |
| **WS6b** "why", the card | The line under a giant's extended details, on the three desks | WS6a | none | S |

WS4 and WS6a can run beside each other after WS3. WS5 is the only one that needs a GPU.

**Later, not queued:**

| | What | Lane |
|---|---|---|
| WS7 | Each deck carried by its own winds (needs VP-A4 part 2); a longer memory for the chromophore; jets as mixing barriers | area |
| WS8 | One eddy speed: the jet count read from 3.2 in place of the pack's two numbers. Changes every giant's bands, and by hand gives Uranus 14 jets, so it needs a better law for ice giants first | **core, one slot** |
| WS9 | Rossby drift for every vortex, storms included; outbreaks as the largest plumes | **core** (Jupiter's `systems`) |
| WS10 | The same mechanism for brown dwarfs, Venus's cloud tops and hot giants' day-to-night flow | area |
| WS11 | Lightning where plumes are; U tier particles (VP-A7) | area |

Red-first tests (each seen red with its fix removed; at least one absolute):

- WS1: the wind's divergence over a patch is zero to rounding (absolute; red today inside a sheared oval); along a hexagon side the wind is parallel to the side (red today).
- WS2: Jupiter's eddy speed at band width is 17 m/s; Saturn, Uranus and Neptune's within a tenth of section 5's; an eddy against the shear past the limit ends within one shear time; the same eddies whichever date asks.
- WS3: plume counts over a long run of windows average section 5's row; none on a world with no internal heat.
- WS4: the stirred patch read at a date equals the one reached in two legs, to the bit; carrying keeps area (the backward map's stretch has unit determinant within 1 per cent, absolute); with eddies off, a band edge stays a straight line; a storm's outline after one renewal keeps the kernel's `aspect` within 5 per cent.
- WS5: the GPU's source places agree with WS4's within one pixel on a 64-cell patch; a stopped clock gives the same pixels twice; tier B draws exactly today's picture.
- WS5b: on tier B a giant shows a stirred face with no press (red before); that face equals the H tier's at the same date within the smaller face's pixel; a stopped clock builds once and never again.
- WS6a: Jupiter's belt says shear wins and Uranus says calm; every number in a sentence equals the block's output; an edited `eddy_efficiency` changes both the sentence and the flow table.

---

## 10. What could be wrong

1. **The eddy law is an order-of-magnitude law** fitted at one point (Jupiter). Saturn's 8 m/s and the ice giants' 3 m/s are predictions nobody has checked here.
2. **Ice giants now carry two eddy speeds** (3 m/s for stirring, 100 m/s for the jet count). The second is the older fudge. WS8 is the cure and it is not easy.
3. **The plume flux is a remembered figure.** It scales every count together; the ratios between worlds hold.
4. **Blending two generations** shows faint doubling where the two disagree most. Region offsets hide the beat; a third generation is the cheap fix if it shows.
5. **Real jets stop cloud crossing them.** Here eddies sit inside bands by placement, which gives scalloped edges and festoons but is not the real barrier.
6. **Saturn will look too crisp.** Its soft look is haze over the decks, which is cloud optics (TA3, MP33), not stirring.
7. **Close zoom** is still limited by the face's size (the U tier's 2048). Finer needs the patch route, a later package.
8. **Jupiter's bands are still twice too many** (13 jets a hemisphere-pair), so its eddies top out at 8,500 km. Not this track's fault or cure.

---

## 11. Questions for Alex

1. **Eddy speed from heat, fitted on Jupiter?** One block gives every giant its stirring from the heat it sends out, with one efficiency set so Jupiter matches. Ice giants then stir gently (3 m/s) while their band count still uses the old 100 m/s until a later core package. *Recommend yes, with WS8 noted as owed.*

2. **Plumes numbered by internal heat?** A new thing on every giant: Jupiter about 70 small bright storms at a time, Saturn a few, Neptune sometimes, Uranus almost never. Too many and too brief to name, so they are counted, not listed. *Recommend yes.*

3. **Phones: skip, or one still picture?** (Answered otherwise: always a still, section 11A.) Skipping keeps today's picture. Where a phone's measured speed allows, the at-rest tier could build one stirred picture when the view stops. *Recommend skip while moving, build at rest where the speed probe allows.*

4. **"Why does it look like this here?" as a line on the giant's card, by tapping a place in True?** No new screen and no change to press budgets. *Recommend yes.*

5. **Accept a too-crisp Saturn until the haze optics land?** *Recommend yes; it is said in the HUD's note.*

---

## 11A. Alex's answers, 27 Nov 2026 (10:45 UTC)

"agree with all - we wanna always have a pic - the click to get it is poor. Low power - run teh render bUT juyst a still. Should be doable?"

Questions 1, 2, 4 and 5: **yes.** Question 3 is changed:

- **There is always a stirred picture, on every device, with nothing to press.** No "skip" tier.
- **Low power means a still.** Tier B and `canvas` run the same backward pass once and keep the result; they do not run it every frame.
- **How.** It is doable because the picture depends only on the date. On a GPU the pass is run once at 512 wide (a quarter of the H tier's work, one frame's cost, spread over a few frames if the governor asks). With no GPU the same steps run on the processor from the same flow table, at 256 wide, in about a second by estimate (to be measured in WS5b). The still is cached by world and date.
- **The cache is in memory only** (Alex, 10:46 UTC: "and cache locally - dont save in file - but if mem exists it makes sense"). Stills are kept on the device while memory allows, oldest dropped first under a size limit taken from the device's measured memory; a device short of memory keeps only the one on show. Nothing is written to a file, a save, a pack or a postcard: a still can always be rebuilt from the date.
- **When the clock runs.** The still is carried row by row at the jets' speed as today (`flowFace`), so the bands still slide. It is rebuilt when the clock stops, and while playing each time the cloud has aged a tenth of its renewal time, as the budget allows. If the device cannot keep up it shows the last still, right for its own date, and the note says so. Nothing is slowed.
- **The design thread's reading of "the click to get it is poor":** nothing about the stirred look, or the first line of "why", may wait for a press. So the giant's card always shows the one-sentence "why" for the whole world; tapping a place in True only narrows it to that place.

---

## 12. Routine choices, for the coordinator

- Track code WS; rows in `status/true-air.md` by its owner.
- New `want`s unlisted in the refusal, so no core line moves.
- Area re-blesses in WS1 and WS3 are the work's own (Alex's rule on routine change).
- WS2 takes `sse.table/1` if ER11a is on main, else a plain response.
- WS5's screens are taken on AC-ProArt under the standing permission, to `sse4/handover-screens/WS5-screens/`.
- Every figure in section 13 goes through the citations check after 1 Dec; the pack rows stay `unverified` until then.

---

## 13. Sources, all unverified until 1 Dec 2026

- Rhines 1975, J. Fluid Mech. 69, 417: the scale where turbulence gives way to jets.
- Kraichnan 1967, Phys. Fluids 10, 1417: energy passed from small eddies to large in flat turbulence, speed as the cube root of size.
- Salyk, Ingersoll et al. 2006, Icarus 185, 430: eddies feeding Jupiter's jets, and the power in them.
- Galperin et al. 2014, Icarus 229, 295: Jupiter's eddy energy from Cassini.
- Moore and Saffman 1971; Marcus 1993, Annu. Rev. Astron. Astrophys. 31, 523: a vortex in shear (already used by PR 646).
- Okubo 1970, Deep-Sea Res. 17, 445; Weiss 1991, Physica D 48, 273: turning against stretching.
- Aref 1984, J. Fluid Mech. 143, 1: smooth changing winds fold a tracer into filaments.
- Staniforth and Côté 1991, Mon. Weather Rev. 119, 2206: following the wind backwards.
- Ackerman and Marley 2001, Astrophys. J. 556, 872: cloud as a balance of mixing and fall-out.
- Gierasch et al. 2000, Nature 403, 628: moist storms carrying Jupiter's internal heat.
- Little et al. 1999, Icarus 142, 306: Jupiter's lightning sits in the belts.
- Pearl and Conrath 1991, J. Geophys. Res. 96, 18921; Li et al. 2018, Nat. Commun. 9, 3709: the giants' heat budgets (Li et al. put Jupiter's internal heat higher, near 7.5 W/m²).
- Vasavada and Showman 2005, Rep. Prog. Phys. 68, 1935: the review the bands already cite.

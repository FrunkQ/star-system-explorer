# SSE4: melting inside worlds, and the heat it carries (track HF, package HF4b)

Status: **PROPOSED, 26 Nov 2026. Not yet accepted by Alex.** Design only; nothing here is built. Four questions for Alex are in section 13, each with a recommendation; routine choices for the coordinator are in section 14. HF is a code only, never shown in the product.

Asked for by the thread "Fix low heat flow in worlds" through the coordinator (26 Nov 2026, 20:46 UTC). That thread builds from this note.

Read with: `handover/status/heat-flow.md` and `handover/packages/HF1.md` (PR 660), `crates/sse-surface/src/core.rs` (`core_history`), `SSE4-time-scales-design.md` (PR 651, sections 6 to 8 and 10), `SSE4-edits-reach-design.md` (sections 3 to 5), `SSE4-magnetic-fields-design.md` (MF1), `docs/fudges.md` (the MF1 and HF1 rows).

**On sources.** The Consensus paper search is out until 1 Dec 2026. Every paper figure in this note is from memory and is marked **unverified** in section 15. None may go on the data sheet as verified until it has been read at source.

**On the scratch model.** I ported `core_history` (PR 660's head) to a small script, checked the port against the HF1 log (Mars 2,151 K against 2,150; Moon 2,046 against 2,047; Mercury 2,017; Earth 35 TW), then added this design's mechanism to it and ran Mars, the Moon, Mercury, Venus, Earth and Io. The numbers in section 5 come from that. It is in project files at `sse4/melt-proto/` (`base.py`, `melt.py`, `tide.py`). It is a design check, not kernel code: no inner core, no dynamo ledger, plain Python maths.

---

## 1. The short answer

1. **One mechanism does it, and most of it is bookkeeping the kernel already half does.** A solid layer that gets hotter than its solidus makes melt. The melt leaves by its own buoyancy and takes heat, water and the heat-making elements with it. What stays behind is softer while melt is in it and stiffer once the water has gone. Volcanism is the melt that leaves. This is the same for Mars's mantle, Io's mantle and, with ice's numbers, Europa's shell.

2. **The lid has to be its own thing.** Today the lid is implied (HF1 works its thickness out from the heat flow each step). In this design the lid's thickness is a kept state that grows by a freezing-front rule. A young lid then cools like a half-space without any special case, and the heat a young world loses comes from the lid's own cooling, **not** from the deep mantle. That is why the heat flow thread's trial killed Mars's field 4.5 Gyr ago: it took the half-space heat out of the whole mantle. With the lid as its own reservoir the scratch model puts the end of Mars's thermal dynamo 4.04 to 4.16 Gyr ago, against 3.98 for HF1 on the same measure (the kernel itself says about 3.9). The guarded range is 3.6 to 4.1. So it is close to the old edge, not broken, and section 11 says what to do if it lands just outside.

3. **Melt alone does not give Io its heat.** With melt in the mantle and the tidal response read from that state, a uniform Io comes out at 0.02 to 0.1 W/m² against a measured 2.2 (unverified). Three more things are needed, and this note says which are in reach:
   - an **anelastic response** (Andrade), not pure Maxwell: in reach, one more term in the HF4a block;
   - a **layered response** (liquid core, soft layer, lid): in reach as one heavy block, and it can be checked against seven measured Love numbers that have nothing to do with heat;
   - the **orbit side** (the host planet's own tide and the resonance partner): yes, an honest Io needs it, and it is a separate design (section 4.5). It is what holds Io's orbit stretched, and it sets a hard ceiling on the heat. For Io that ceiling is about 1.2 × 10¹⁴ W, which is the heat Io is measured to give off.

4. **Expected accuracy, said plainly** (section 5): Earth's heat loss and melt production come out right to about a third with nothing tuned. Lid worlds make the right amount of crust in their first 100 Myr and then too much, by 2 to 5 times, because the kernel's lid mantles run hot (two existing rows in the fudge register: the dry-mantle factor of 100 and the typed 1,950 K start). Io and Enceladus computed from their interiors alone will be low by one to two orders until the layered response and the orbit side exist. For real bodies with a measured heat flow, I recommend the measurement pins the heat and the model explains the inside (question 2).

5. **It is five packages, each leaving main green**, three of them in the core golden lane (section 9).

---

## 2. What the code does today (checked on PR 660's head, `42583ce`)

| Piece | Where | What it does | What is missing |
|---|---|---|---|
| Thermal history | `sse-surface/src/core.rs`, `core_history` | Two boxes, mantle and core, marched from the magma ocean's end in 20 Myr steps. Mantle loses heat by a plate scaling or a stagnant-lid scaling (Solomatov and Moresi 2000) | No lid state, no melt, no crust |
| Melt | `t_mantle_max_k` 2,200 K under plates; HF1's solidus at the lid's base under a lid | A clamp: heat that would lift the mantle past the limit is "vented" and counted in `q_surface_tw` | No melt fraction, no melt volume, nothing softened, nothing taken out |
| Lid | HF1's `hottest`: lid = k ΔT / q each step | Used only to find the solidus pressure | Fully grown from the first step |
| Water | `wet` 1 to 0 with a 1 Gyr e-folding under a lid; `dry_viscosity_factor` 100 | Stiffens lid mantles a hundredfold by about 2 Gyr | A typed time, not tied to melting. In the port this, not the core, is what dates Mars's field |
| Start | `t_mantle_start_k` 1,950 K for every world | | Typed, the same for the Moon and a super-Earth |
| Heat made | `Ctx::radiogenic` + `tidal_at` (`lib.rs:749`) | Radiogenic from the `planet-heat` flow, all of it in the mantle for all time; tides from one k2/Q = 0.3/100, or an authored `tidal_w_m2` | No world's own response; nothing moves into a crust |
| Regimes | `envelope` (`lib.rs:640`) | Worked out first, from heat made, by pack rules; the history then reads `mobile-lid` or not | Regimes and history do not march together (HF2's job) |
| Readers | `RegimeSpan.heat_flow_w_m2` (start and end of each span) | Plates, regimes' landforms, impacts' renewal, stains, plumes, wear, life all read heat **made** | HF2 |
| Tidal response | `sse.tides.maxwell_response` (HF4a, on the heat flow branch) | Im k2 and Re k2 of a uniform Maxwell sphere | Nothing reads it; no layers |
| Icy worlds | `shell_activity` (`lib.rs:631`): shell thickness = 651 ln(T_melt/T_s) / q | Instant conductive thickness from heat made | No shell state, no heat made in the shell |

The fixtures give Io, Europa, Enceladus and Triton an authored `tidal_w_m2` (2.5, 0.05, 0.02, 0.1). The 0.43 W/m² for Io is the Sol pack's Io, worked out from its orbit.

---

## 3. The mechanism

### 3.1 Layers and what is kept

A world's solid interior is a short stack of layers. Each layer names its material on the data sheet, and every number below is that material's row.

| Layer | Material today | Kept state (new in bold) |
|---|---|---|
| Core | `earth-core-liquid` and iron rows | core-top temperature, inner core radius (as now) |
| Rock mantle | `mantle-rock` | potential temperature `T_m` (as now, `t_u`); **fertile share `x`** (how much has never been through the melt zone); water share `w` (as now, `wet`, but now driven by melting); **retained melt `φ`** |
| Lid | the mantle's own rock, cold | **thickness `D_l`** |
| Crust | `basalt` | **mass `M_cr`**, **heat-making elements it holds** |
| Ocean, ice shell (icy worlds) | `h2o` forms, `surface-ice` | **shell thickness** (package HF4d, section 3.10) |

Seven numbers per rocky world where there are three today. All are kept at every grid point (section 6).

### 3.2 The lid as its own reservoir

The lid's base is where rock stops flowing with the mantle. Its temperature is a fixed step below the mantle's, set by how fast viscosity changes with temperature:

```
T_l = T_m − a_rh · R T_m² / E          (a_rh about 2.2; Davaille and Jaupart 1993)
```

The lid thickens when more heat conducts out of its base than the mantle delivers, and thins when less:

```
ρ c (T_m − T_l) · dD_l/dt = q_base − q_m − q_intruded
```

- `q_m` is the heat the convecting mantle delivers to the lid's base: the stagnant-lid scaling the code already has (`top`, unchanged).
- `q_base` is what conduction carries away from the base. For a lid with heat made inside it (`h`, W/m² of surface, from the crust) and nothing being buried: `q_base = k (T_l − T_s)/D_l − h/2`, and the surface gets `q_cond = k (T_l − T_s)/D_l + h/2`.
- `q_intruded` is the heat of melt that freezes inside the lid (section 3.5).

Three things follow with no special cases:

- **A young lid is a cooling half-space.** With `D_l` near zero, `q_base` is huge, the lid grows as √t, and the surface heat flow falls as 1/√t. This is the Turcotte and Schubert result the heat flow thread reached for, but the heat comes out of the rock that is joining the lid.
- **The mantle underneath only ever loses `q_m`.** Its energy sum does not see the lid's own cooling. This is the fix for the Mars field failure.
- **A steady old lid** settles at `q_base = q_m`, which is HF1's lid thickness.

**When lava is burying the surface** (a heat pipe), the lid is carried downward at the eruption speed `v` and conduction is choked. With `Pe = v D_l / κ`:

```
q_cond = k (T_l − T_s)/D_l · Pe / (e^Pe − 1)          at the surface
q_base = k (T_l − T_s)/D_l · Pe e^Pe / (e^Pe − 1)      at the base
```

(O'Reilly and Davies 1981.) At `Pe` → 0 both are plain conduction. At large `Pe` the surface conducts almost nothing and the lid is cold nearly to its base, which is how Io holds up mountains 17 km high on a world losing 2 W/m².

### 3.3 How much melts

Rock carried up under the lid follows the mantle's adiabat and crosses its solidus. Between solidus and liquidus the melt share is linear in temperature, and melting itself uses heat (the latent heat cools the parcel), so:

```
F(z) = (T_ad(z) − T_sol(P)) / (T_liq(P) − T_sol(P) + L/c)        clamped to 0..1
T_ad(z) = T_m · exp(α g z / c),   P = ρ g z
```

- `T_sol(P)` and `T_liq(P)` are fits on the sheet (dry peridotite: Hirschmann 2000 as HF1 already uses; liquidus from Katz et al. 2003).
- `L/c` is about 330 K for rock, so melting is about a third less productive than temperature alone says. Leaving it out raised young Mars's mean melt share from 0.17 to 0.34 in the scratch model.
- The **melt zone** is the depths from the lid's base down to where `F` reaches zero. Its thickness and its mean melt share `F̄` come from a fixed 48-point sum down the adiabat. Under moving plates the lid is the plate's own thin edge at the ridge, so the zone starts at the surface.
- Katz et al. give `F` a power of 1.5 and a kink where clinopyroxene runs out. I recommend linear with the latent term for the first build: it is what the parametrised Mars and Mercury models use (Morschhauser et al. 2011), and the exponent is one editable number on the block.
- Water lowers the solidus (Katz et al. 2003: ΔT = 43 · X^0.75 K, X in weight per cent). It goes in with the water budget (HF7), not now.

### 3.4 How fast melt is made

Two routes, added.

**By supply (the slow, usual route).** Convection carries fresh rock through the melt zone. The speed is the boundary-layer speed the heat flow already implies, with no new constant:

```
u = κ D / δ²            δ = boundary layer thickness = k ΔT_bl / q_m
Π_supply = u · F̄ · V_zone · x / D          (m³/s of melt)
```

`D` is the mantle's depth, `V_zone` the melt zone's volume, `x` the fertile share. This is the form Morschhauser et al. 2011 use for Mars (they divide by the planet's radius where I use the mantle's depth; noted in the fudge register as "the length the mantle turns over on").

**In place (the fast route, for heated worlds).** If the mantle is gaining heat, the share of that gain that lands inside the melt zone makes melt instead of raising the temperature:

```
N = heat in − q_m·A − heat carried by supply melt            (W; only if positive)
share = V_zone / V_mantle for radiogenic and core heat;  1 for tidal heat once a melt zone exists
Π_in_place = share · N / (ρ_melt (L + c (T_m − T_l)))
```

- This is HF1's "vent the excess at the solidus", graded by how much of the mantle is above its solidus. With the whole mantle above it, it **is** HF1.
- Tidal heat is made where the rock is soft, which is the melt zone, so its share is 1. (Once the layered response of section 4.2 exists it gives the true share per layer, and this line reads that.)
- Without this route the scratch model's Io had to heat to 2,700 K and melt entirely before supply could carry 100 TW. With it Io sits at 1,950 to 2,000 K.

**The basalt budget is finite.** A parcel that has melted once gives little more: `x` falls as rock is processed (section 3.7), and supply melting scales with `x`. This is a conservation rule, not a brake added for looks: without it the scratch model's Mars made more crust than its mantle holds basalt.

### 3.5 How melt leaves

**Darcy percolation, always** (the Bierson and Nimmo 2016 and Moore 2001 route), not an eruption-rate limit. Melt rises through the pores between grains at a speed set by its buoyancy:

```
flux = (b² φⁿ / C) · Δρ g / η_melt            (m/s through the top of the melt zone)
```

`b` grain size, `n` and `C` the permeability law (Miller et al. 2014: n = 2.6, C = 58), `Δρ` rock minus melt, `η_melt` the melt's viscosity.

Melt crosses the zone in far less than one step, so the melt present is the amount that lets the flux carry what is being made:

```
φ = ( Π/A · η_melt · C / (b² Δρ g) )^(1/n)          capped at the lock-up share φ_c (about 0.4)
```

This one rule covers both limits the brief asks about:

- **Weakly heated worlds (Mars, the Moon, Earth)** are supply-limited. Darcy could drain far more than is made, so `φ` stays at about 1 per cent and all the melt made leaves. "Eruption-limited" models are this limit.
- **Strongly heated worlds (Io)** hold more melt: 5 per cent in the scratch model at Io's measured heat with 1 cm grains, about 20 per cent with 1 mm grains. Grain size is the number this is most sensitive to and it is not known for Io.

**Up or down.** The sign of `Δρ` says which way melt goes. Rock melt at low pressure is lighter and rises. Water is denser than ice and drains down into the ocean (section 3.10). Same block.

**How much reaches the surface.** A share `f_ext` erupts; the rest freezes inside the lid as intrusions. First build: one editable number, 0.2 (about five parts intruded to one erupted; White et al. 2006), in the fudge register. The physics that should replace it is the magma's own head: melt erupts where the weight of the rock column it rose through can lift it to the surface, so a light crust (the Moon's) keeps dense melt down except where the crust is thin. That is listed as HF8.

- Erupted melt gives `L + c (T_m − T_s)` straight to the surface, and buries the lid at `v = f_ext · Π / A`.
- Intruded melt gives `L + c (T_m − T_l)` to the lid: `q_intruded` in section 3.2. It thins the lid. With no intrusions a heat-pipe lid has no steady thickness (Spencer et al. 2020 make the same point for Io).

### 3.6 What melt does to the rock left behind

```
η = η_dry_or_wet(T_m, w) · exp(−a_η φ)         for φ below φ_c        (a_η about 26; Mei et al. 2002)
μ = μ₀ · (1 − φ/φ_c)                             floor at 5 per cent of μ₀
```

- Above `φ_c` the grains no longer touch and the layer is a liquid with crystals in it. The first build caps `φ` at `φ_c` and says so in the event text; a true magma layer is the magma ocean the regimes already have.
- The rigidity law is the simplest that goes to zero at lock-up. Bierson and Nimmo 2016 use a measured form; it replaces this one when read at source. **Unverified, and flagged as a stand-in.**
- This softened `η` goes into the convection scaling (`q_m`, `u`) and into the tidal response (section 4). One viscosity, read in both places.

### 3.7 What leaves with the melt

One number does three jobs. The mass of mantle that passes through the melt zone in a step is the melt made over the mean melt share:

```
processed = ρ_melt · Π · dt / (F̄ · M_mantle)
x ← x · exp(−processed)          fertile share
w ← w · exp(−processed)          water (it prefers the melt almost wholly)
heat-making elements in the mantle ← the same
```

- **Water.** This replaces the typed 1 Gyr `degas_gyr`. A world that melts a lot dries fast; one that barely melts stays wet. The dry-mantle factor of 100 stays (it is a rheology number and moves to the sheet), but when it bites is now physics. The erupted share of the water is a dated amount handed to the air (the conserved chain's rule: an amount, never a description). Under moving plates sinking slabs carry water back down as now.
- **Heat-making elements.** U, Th and K go into the melt and so into the crust. The crust's share heats the lid, not the mantle. In the scratch model Mars's crust ends with about three quarters of them; estimates for the real Mars are about half to two thirds (unverified).
- **Crust.** `M_cr` grows by all melt made. **A crust cannot be thicker than its lid**: what is pushed below the lid's base goes back into the mantle with its elements and water. That is the whole of Io's cycle (its crust is its lid, turned over every few million years), and it is why Io's mantle never runs out of fertile rock.
- **Depletion raises the solidus** by up to about 150 K when a fifth of the mantle has been taken as melt (Morschhauser et al. 2011). One editable row; in the scratch model it cut the crust made on Mars by a quarter and on Mercury by two thirds.

### 3.8 Heat bookkeeping

Per step, in watts. The surface total is what HF2's readers take.

| | Mantle loses | Lid gains or loses | Surface loses |
|---|---|---|---|
| Convection under a lid | `q_m · A` | gains `q_m · A` | |
| Conduction | | loses `q_cond`, with its own cooling and the crust's heat | `q_cond · A` |
| Melt made | `ρ Π (L + c (T_m − T_l))` | | |
| Melt erupted | | is buried at `v` | `f_ext ρ Π (L + c (T_m − T_s))` |
| Melt intruded | | gains `(1 − f_ext) ρ Π (L + c (T_m − T_l))` | (conducts out later) |
| Crust pushed under | gains cold rock (already paid for in "melt made") | | |
| Moving plates | plate scaling (as now) + `ρ Π L` | none | the same |

Under moving plates only the latent heat is added: the plate scaling already counts the cooling of the new crust. In a steady heat pipe the surface total equals what the mantle loses, as it must.

A test in HF4b.2 adds up every row over a world's life and checks energy is conserved to 1 part in 10⁶. This is the "absolute, not a ratio" assertion the house rules ask for.

### 3.9 Moving plates (Earth)

Same blocks, with the lid replaced by the plate's edge at the ridge. Checked by hand and in the scratch model, with nothing fitted:

- Melt zone today: 53 km deep (real: 60 to 80 km).
- Mean melt share 0.15; speed 2.3 cm/yr (real plates average about 4 to 5).
- Melt made: 31 km³/yr (real: about 20 at ridges, 25 to 30 in all; Crisp 1984, unverified).
- Heat by melt today 1.1 TW of 34 TW. Early Earth: 45 TW of 160 at 0.1 Gyr, and a crust 30 to 40 km thick where today's is 7.

The 2,200 K cap is never reached and can go.

### 3.10 Ice shells (package HF4d)

An ice shell over an ocean is this mechanism with the roles turned over, and it is specified here only as far as the shared blocks need:

- The shell is the lid. Its thickness is a kept state. Its base is a true freezing front, so the lid rule gains the latent term: `ρ (L + c ΔT) dD/dt = q_base − q_from_below − heat made in the shell`.
- The melt is water. It is denser than ice Ih, so Darcy's sign sends it down and it joins the ocean. Nothing erupts by buoyancy; plumes stay the cracks-to-the-ocean physics `plumes.rs` has.
- Heat from below is the rock interior's own history (the same code, on the rock part).
- Heat made in the shell is the tidal response's share for the shell (section 4.2). Almost all of it is made in the warm bottom tenth, where ice's Maxwell time is near the orbital period.
- `shell_activity` then reads the kept thickness instead of an instant conductive one.

Ice's rheology rows (Goldsby and Kohlstedt 2001), rigidity and melting curve are on or go on the sheet under `h2o` and `surface-ice`. HF4d gets its own short design note from whoever builds it; nothing in HF4b or HF4c depends on it.

---

## 4. Tides

### 4.1 The loop

Each step: the state gives `η`, `μ` and the layer thicknesses; the response block gives Im k2; the existing `sse.tides.eccentricity_heating` block turns that into watts (its `k2_over_q` becomes the input port `tides.im_k2`, as the heat flow thread already told the small-moon thread); the watts go into the mantle's heat for the next step.

Tidal heating can run away (hotter is softer is hotter) on a time far shorter than a step. Section 6.2 says how the march stays stable and how a jump between branches becomes a dated event.

### 4.2 The response: three steps, each a real gain

**(a) Anelastic, not just Maxwell.** A Maxwell body only dissipates when its viscosity is within a factor of ten or so of μ/ω (about 10¹⁵ Pa s for Io). Real rock also creeps transiently, and loses energy at every viscosity (Andrade creep; Castillo-Rogez et al. 2011, Efroimsky 2012):

```
J = 1/μ − i/(η ω) + μ^(α−1) η^(−α) (iω)^(−α) Γ(1+α)          α about 0.3
μ* = 1/J
```

Written as real and imaginary parts it is plain arithmetic and fits a formula block. With α = 0 switched off it is HF4a's Maxwell block. For Io at 10¹⁹ Pa s: Maxwell gives 0.001 W/m², Andrade 0.12.

**(b) Layers.** A uniform sphere cannot describe a shell over an ocean at all, and it misses that a liquid core lets a mantle flex about four times more (Io's measured k2 is 0.125; a uniform solid Io gives 0.03). I recommend one heavy block, `sse.tides.layered_response`: the standard propagator method for a stack of uniform incompressible layers, each with its own complex rigidity (Sabadini, Vermeersen and Cambiotti 2016; Roberts and Nimmo 2008; Jara-Orué and Vermeersen 2011 for liquid layers). In: up to about six layers (outer radius, density, μ, η) and the tidal frequency. Out: Re k2, Im k2, h2, and each layer's share of the heat.

- It is a Big Block (6 by 6 complex matrices do not fit a formula) and says so in the fudge register. Its settings are editable; its working is fixed.
- Liquid layers are given a very small rigidity (a stand-in, registered) rather than their own boundary conditions.
- **It must reproduce HF4a's uniform block to 1 part in 10⁹** for a uniform body. Any slip in the matrices fails that.
- **It is checked against measured Love numbers**, which have nothing to do with heat and so cannot be tuned against it: Earth 0.30, Moon 0.024, Mars 0.17, Mercury 0.57, Venus 0.30, Io 0.125, Titan 0.62 (all unverified here; sources in section 15). Expect each within about 25 per cent, since the interiors are simple stacks.
- Cost: a few microseconds a call. It is called only for worlds whose tidal heat could matter (section 6.4).

The builder will not have the textbook to hand before 1 Dec. The two tests above are strong enough to write it from first principles; if that proves slow, HF4c.1 (Andrade, uniform) lands alone first and the layers follow.

**(c) HF4a's uniform block stays** for small moons and small bodies with no layers (Mimas, Miranda). The small-moon shape thread reads it as agreed. A small moon with an ocean (Enceladus) needs the layered block.

### 4.3 What to expect for Io, honestly

From the scratch model, with the tidal heat read back from the state each step:

| Response | Io's tidal heat | Against 2.2 W/m² |
|---|---|---|
| Uniform Maxwell (HF4a as built) | under 0.001 W/m² | nothing |
| Uniform Andrade, mantle dried by its own melting | 0.02 W/m² | 100 times low |
| Uniform Andrade, mantle kept wet | 0.11 W/m² | 20 times low |
| Layered, with the liquid core (estimate, not run) | 0.2 to 1 W/m² | 2 to 10 times low |

Why it stays low: Io needs an effective viscosity near 10¹⁶ Pa s. Darcy drains melt so well that the mantle holds only a few per cent, which softens it tenfold where ten-thousandfold is needed. Published models reach Io's heat with a thin, very soft, melt-rich layer or with rheology numbers chosen to fit (Segatz et al. 1988; Bierson and Nimmo 2016). So the mechanism gives the right kind of Io (melt-bearing mantle, heat-pipe lid, about a centimetre a year of resurfacing) at a heat that is too low, until the orbit side is in.

### 4.4 Measured heat pins the answer for real bodies

Where a real body has a measured heat flow (Io; Enceladus; the Moon's surface heat flow at the two Apollo sites), the measurement is a fact like a measured orbit. I recommend the same rule the real-catalogues design uses: **the measured value pins the heat, the model explains the inside, and the card shows the model's own number beside it** ("measured 2.2 W/m²; the interior alone gives 0.3: the rest is the orbit's doing, not yet modelled"). `Body.tidal_w_m2` already carries this for fixtures. Question 2.

Run this way at Io's measured 2.5 W/m², the scratch model gives a mantle at 1,950 to 2,000 K holding 5 per cent melt, 1,640 km³ of melt a year of which 330 erupt (0.8 cm a year over the whole surface; the measured resurfacing is about 1 cm a year, unverified), and almost all the heat leaving as lava. None of that was fitted.

### 4.5 Does an honest Io need the host's Q and the resonance partner? Yes

Two separate reasons.

1. **Why the orbit stays stretched.** A lone moon's eccentricity dies away in about 10⁵ to 10⁷ years as it heats. Io keeps 0.0041 only because Europa and Ganymede pump it. A generated moon with no partner and an eccentricity it was simply given is being heated by an orbit that could not last. The kernel does not hold resonances between moons (`sse-kernel/src/stability.rs` and formation know resonances for planets and belts only).
2. **Where the energy comes from, and its ceiling.** The heat is paid for by the host's spin, through the tide the moon raises on the host. That torque times the moon's orbital rate is the most a moon held in resonance can dissipate for long:

```
ceiling = n · (3/2) (k2/Q)_host · G m² R_host⁵ / a⁶
```

   For Io with Jupiter's measured k2/Q of 1.1 × 10⁻⁵ (Lainey et al. 2009) this is 1.2 × 10¹⁴ W. Io gives off about 0.9 to 1.0 × 10¹⁴ W. For Enceladus with Saturn's 1.6 × 10⁻⁴ (Lainey et al. 2017) it is 36 GW against a measured 10 to 16. So for a moon held in resonance the heat is set mostly by the **host**, and the moon's interior decides the eccentricity it settles at, not the other way round. This is the robust physics, and it needs no rock rheology at all.

What this note does about it:

- **Now (HF4c):** the ceiling is worked out and published as a fact (`tides.host_power_ceiling`) wherever the host's k2/Q is known. It is a check and an explanation ("Io gives off nine tenths of what Jupiter can feed it"), never a clamp. Jupiter's and Saturn's measured values go on the sheet as reference-object rows.
- **Logged, not faked:** a moon's eccentricity that nothing holds up is listed in the fudge register ("given, not explained"). No constant stands in.
- **Next (HF6, its own design):** resonance partners for moons from formation, free eccentricity damping, forced eccentricity, the host's dissipation for generated giants (a table of measured values first, per Alex's ruling on hard problems; resonance locking, Fuller et al. 2016, as the law to look at), and the coupled balance where heat is set by the host and eccentricity is the output. Question 3.

### 4.6 Enceladus and Europa

- **Enceladus.** Shell-only models with Maxwell or Andrade ice give 0.1 to 3 GW (Roberts and Nimmo 2008). The measured 10 to 16 GW is an open research question; the leading answer is friction in a loose, water-filled rock core (Choblet et al. 2017). Today's 4 GW comes from the shared k2/Q of 0.003 and is right by luck. **After HF4c and HF4d the computed Enceladus will be lower than today**, unless the loose core is modelled. That core is general physics the small-body thread is already building (a core that was never squeezed or heated shut is weak), so the two meet there. Until then: the measurement pins it (4.4), the ceiling brackets it (4.5).
- **Europa.** No measured heat flow. Published models span 0.02 to 0.1 W/m² and shells 3 to 30 km thick. It is a consistency check (chaos terrain, a shell thin enough to crack), not a number to hit.

---

## 5. What the scratch model gives, and what to expect from the build

Kernel numbers as they are (η_ref 10²¹ Pa s at 1,600 K, dry factor 100, start 1,950 K, 20.3 ppb U). New numbers at their cited values, none adjusted after seeing a result. "Start 1,780 K" is the derived start of question C3. Thermal dynamo end is "core heat flow drops below what the core's adiabat conducts", the measure on which HF1 gives Mars 3.98 Gyr ago where the kernel says about 3.9.

| Check | Measured (unverified) | HF1 today | Melt, start 1,950 K | Melt, start 1,780 K, with depletion |
|---|---|---|---|---|
| Earth heat loss | 46 to 47 TW | 35 | 34 | 34 |
| Earth melt made | 20 to 30 km³/yr | none | 31 | 31 |
| Mars thermal dynamo ends | 3.6 to 4.1 Gyr ago (guard) | 3.98 | 4.16 | 4.04 |
| Mars crust | 24 to 72 km (InSight) | none | 51 km at 0.1 Gyr, 320 km today | 2 km at 0.1 Gyr, 120 km today |
| Mars mantle today | about 1,600 to 1,750 K | 2,151 K | 2,048 K | 2,064 K |
| Mars lid today | 400 to 600 km | not kept | 486 km | 459 km |
| Moon crust and lavas | about 40 km crust (mostly from its magma ocean); lavas about 10⁷ km³ | none | 254 km | 107 km |
| Mercury crust | 35 ± 18 km | none | 239 km | 35 km |
| Venus melt made | 1 to 10 km³/yr (estimates) | none | 180 | 156 |
| Io, heat pinned at 2.5 W/m² | resurfacing about 1 cm/yr | 105 TW "vented", no state | 0.8 cm/yr erupted, 5% melt, 1,950 to 2,000 K | the same |
| Io, heat from its own interior | 2.2 W/m² | 0.43 (shared k2/Q) | 0.02 to 0.11 (uniform Andrade) | the same |

What I take from it:

1. **Earth is right for the right reason.** Melt production within a third, untuned. Its heat loss barely moves (the 12 TW gap to the measured 46 is mostly heat made in continental crust, which the kernel keeps in the mantle; HF4b.2's crust bookkeeping does not reach it because plates recycle their crust. Logged, not chased here).
2. **Mars's field survives.** The lid no longer robs the mantle. What moves the date now is how fast melting dries the mantle, which is physics where there was a typed 1 Gyr.
3. **Lid worlds make too much crust after the first 100 Myr**, by 2 to 5 times, and their mantles stay 300 K or more hotter than published models. The cause is upstream of melting: the kernel's lid mantles cannot lose heat (a wet reference viscosity fit to Earth, times 100 when dry) and all start at 1,950 K. Both are existing rows in the fudge register. The mechanism here turns that stored heat into lava, which is the honest consequence. **Do not tune the melt numbers to hide it.** The repairs are C3 (a derived start) and HF7 (a mantle water budget).
4. **The Moon is the hardest check.** Its real lavas are a thousandth of what a hot lunar mantle would erupt. Two pieces of real physics are missing and neither is in HF4b: a light flotation crust that dense melt cannot rise through (HF8), and a mantle left refractory by its magma ocean. Expect the Moon's lava plains to be dated about right (lavas for the first 1 to 2 Gyr, fading) and far too voluminous.
5. **Venus** comes out as a world that loses a third to a half of its heat by volcanism. So do published stagnant-lid models of Venus, which is one reason an episodic lid is argued for it (Armann and Tackley 2012).

**Expected accuracy after HF4b and HF4c, in one line each:**

| | Expect |
|---|---|
| Earth heat loss, melt production | within a third |
| Mars field date | within 0.2 Gyr of today's answer; may sit at the guard's edge |
| Lid thickness today (Mars, Moon, Mercury) | within a third |
| Dates volcanism fades (Mars, Moon) | right to about a factor of 2 |
| Crust thickness, lava volumes on lid worlds | 2 to 5 times high until C3 and HF7; the Moon far worse until HF8 |
| Measured Love numbers (layered block) | within about 25 per cent |
| Io's and Enceladus's heat from the interior alone | one to two orders low until HF6 |
| Io's inside, with the measured heat pinned | resurfacing rate, melt share and lava temperatures within a factor of 2 |

---

## 6. Time: a march kept as a line

Per `SSE4-time-scales-design.md` sections 6, 8 and 10. If that design changes before it is accepted, this section follows it.

### 6.1 Shape, anchor, grid

- **Shape:** march. **Anchor:** the body's birth. The march starts at the magma ocean's end, a function of the body alone.
- **Grid:** a function of the body alone, never of the date asked for. Geometric at first so the young lid's √t growth is resolved (first step 1 Myr, each step 1.25 times the last), capped at today's `step_myr` of 20. About 250 points to 4.5 Gyr.
- Today's last step is cut short to land on the body's age (`h = dt.min(age − t)`). That makes the grid depend on the date asked for, and it goes: the grid runs on fixed points and a read between two points interpolates the kept state linearly. The fixtures' ages fall on grid points today, so this alone moves no golden.
- **Kept:** the whole state at every grid point (section 3.1's seven numbers, the core's two, and the fluxes of section 7). A read at any date is a lookup. A look-ahead past the body's present continues the same grid and writes checkpoints.
- **Own time `tau`:** the mantle's cooling time, `M c T_m / heat lost`, published for the pace ladder. No absolute time in a double, no new `_yr` field: the line is held against `Instant` offsets from birth once TP1 lands, and against `t_gyr` offsets (differences, not dates) until then.
- **The two standing tests** (section 10 of the time design): read at a date equals stepped there in two legs, bit for bit; and one new one here, energy conserved over the life (section 3.8).

### 6.2 Stiff parts are solved, not stepped

Three pieces respond in far less than a step: the retained melt (thousands of years), a heat-pipe lid (the scratch model's Io lid swung between 3 and 120 km when stepped), and tidal runaway (a few Myr). The rule:

- Each fast state has its own time from its own physics. When that is shorter than a quarter of the step, the state is **set to its balance by bisection** on a fixed number of halvings, as HF1 already does for the solidus. Otherwise it is stepped.
- The mantle temperature itself is stepped implicitly (the new temperature is the one whose own fluxes give the change), again by fixed bisection. This is stable at any step length.
- Where tides give two possible balances (a cold stiff branch and a hot soft one), the march stays on the branch it is on. If that branch stops existing, it moves to the other and **that is a dated event** with a reason ("tidal heating ran away: the mantle began to melt"). Bisection brackets are searched outward from the last state in a fixed order, so the result is the same on every device.

### 6.3 Marks

The history's transitions are its marks for `happenings`, each with a `why` built from the numbers at that step, in the style `core.rs` already uses:

They are `CoreEvent`s, in the list the history already publishes, with these kinds:

| Kind | When |
|---|---|
| `melting-start`, `last-lavas` | melting starts; melting stops |
| `erupting`, `conducting` | the surface switches to erupting its heat (melt carries more than half), or back |
| `crust-recycling` | the crust reaches the lid's base and starts to be pushed under |
| `tidal-runaway`, `freeze-out` | tidal heat and softening feed each other; the interior stiffens and tidal heat dies away |
| `dynamo-start`, `inner-core-birth`, `dynamo-death` | as now |

A screen draws any kind it is given, with its `why`; the list is the kernel's and may grow.

"The last lavas" is the physical date for what `sse.heat.active_geology_lifetime` now gets from a calibrated threshold (`q_crit` 0.019 W/m², in the fudge register). Replacing that block's use is C5.

### 6.4 Cost

Today's history is under a millisecond a world. Budget for the new one: **5 ms a world on the bench device**, measured in each package log.

- The melt zone sum (48 points) and a few bisections a step are a few thousand floating-point operations: tens of microseconds a step.
- The layered response is called only if the tidal ceiling for the body (the heating block run with Im k2 at its largest possible value, 0.75) is more than 1 per cent of its radiogenic heat. Most worlds never call it.
- Blocks are read once at the top and run compiled inside the loop (edits-reach rule 1). If running formula blocks inside the march costs more than the budget, say so to the coordinator before working round it.
- The line is cached by the physics hash (ER7).

---

## 7. What readers read

The history publishes, at every grid point (W/m² of surface unless said):

| Field | Meaning |
|---|---|
| `heat_lost` | everything leaving the surface: conducted plus erupted. **The one heat number.** |
| `heat_conducted` | through the lid or plate |
| `heat_erupted` | carried by lava |
| `heat_made` | radiogenic (mantle and crust) plus tidal |
| `heat_radiogenic` | the radiogenic part, mantle and crust together |
| `heat_tidal` | the tidal part, and whether it was pinned by a measurement |
| `core_heat` | out of the core (as now) |
| `core_freezing` | the part of `core_heat` that comes from the inner core freezing (latent heat and the settling of the light element) |
| `mantle_cooling` | the fall in the mantle's own stored heat, the latent heat of its retained melt included. Negative while the mantle warms |
| `lid_cooling` | the fall in the lid's stored heat (3.2). Negative while the lid warms |
| `lid_km`, `crust_km` | |
| `melt_made`, `melt_erupted` | m of rock over the whole surface per Myr |
| `melt_retained` | share of the melt zone that is liquid |
| `mantle_k`, `fertile`, `water` | |

**One set of names, one unit.** These are the only names, published from the first package that has the number, in W/m² of surface. No reader, the heat screen included, keeps a second mapping; a screen that wants terawatts multiplies by the world's area. The fields close on each other at every grid point:

```
heat_lost = heat_conducted + heat_erupted
heat_made = heat_radiogenic + heat_tidal
heat_lost = heat_made + mantle_cooling + lid_cooling + core_heat
```

The third line is 3.8's bookkeeping read from outside, and the same test holds it to 1 part in 10⁶. So a flow diagram drawn from these fields balances with nothing left over. Before HF4b.1 there is no lid store and no eruption: `lid_cooling` and `heat_erupted` are published as zero, never left out.

Readers, and which package moves each:

| Reader | Reads today | Should read | Package |
|---|---|---|---|
| Regime rules (`q_ratio`, `heat_flow_w_m2`) | heat made | `heat_lost`, marched with the history | HF2 |
| Life's vents, air reductants (`world_life.rs`) | heat made | `heat_lost` | HF2 |
| Plates (speed, ridge height) | heat made over `q_ref` | `heat_lost` over `q_ref` now; the history's own speed `u` later | HF2, then HF5 |
| Plumes (rock) | core heat | `core_heat` (no change) | none |
| Lava plains, shields, flood lavas, tubes, paterae | (heat made / q_ref)³ | `melt_erupted` | HF5 |
| Crater renewal by lava (`impacts.rs`) | (q / q_ref)^n | `melt_erupted` over a flow's thickness | HF5 |
| Io's colours and stains | heat made | `melt_erupted`, `heat_erupted` | HF5 |
| Ice shell activity, chaos, plumes (ice) | instant conductive shell | kept shell thickness | HF4d |
| Small-moon shapes | none yet | the tidal power block with the body's own response | HF4c (other thread) |
| "Why does this look as it does" | none | the marks of 6.3 and the fields above | free, from the same numbers |

HF5 is where the calibrated `q_ref` family (0.06, 0.039, 2.5, 0.04, 0.007, 0.05 in `packs/base-surface`) leaves the fudge register: volcanic landforms then follow the lava that was actually erupted. It will move many surface goldens and some of the eleven tests HF2 found; those are tests of calibrated behaviour and are updated to follow this design once it is accepted, with each change listed in the package log. A test that guards a measured fact (Mars's field date, the Moon having lava plains at all) is a limit and stays.

---

## 8. Every number, and where it goes

Rule (edits-reach, section 14): `core.rs` is on the side-door ledger (4 sheet reads), and every `CoreSettings` number is a typed constant. Each package below moves the numbers it touches to the sheet or to a block's settings **in the same PR** and deletes its ledger lines. Compiled code reads them through `sse-physics` (`physics::sheet()`, `physics::run`), once, above the loop.

### 8.1 The data sheet (`packs/base-reference`, read with Look up)

All new rows carry `"check": "unverified"` until read at source after 1 Dec. Each new source goes on the master citation list (`scripts/citations.py --add`).

| Row | On | Value | Source | Now lives |
|---|---|---|---|---|
| `phases.solidus` (fit, °C, GPa, 0 to 10) | `mantle-rock` | −5.104 P² + 132.899 P + 1120.661 | Hirschmann 2000 | `CoreSettings::mantle_solidus_c` |
| `phases.liquidus` (fit, 0 to 10 GPa) | `mantle-rock` | −2.0 P² + 45.0 P + 1780.0 °C | Katz et al. 2003 | new |
| `phases.liquidus_deep` (Simon form) | `mantle-rock` | 1940 K (P/29 + 1)^(1/1.9) | Andrault et al. 2011 | `mantle_liquidus_*` |
| `phases.latent_fusion` | `mantle-rock` | 4.0 × 10⁵ J/kg (also quoted: 6 × 10⁵) | Turcotte and Schubert 2014 | new |
| `phases.solidus_rise_depleted` | `mantle-rock` | 150 K at 0.2 taken | Morschhauser et al. 2011 | new |
| `rigidity` | `mantle-rock` | 6.5 × 10¹⁰ Pa | Turcotte and Schubert 2014 | typed in HF4a's example |
| `creep.viscosity_ref`, `creep.t_ref`, `creep.activation` | `mantle-rock` | 10²¹ Pa s, 1,600 K, 3 × 10⁵ J/mol | Hirth and Kohlstedt 2003 (calibrated on Earth: stays in the register) | `eta_ref`, `t_ref_k`, `activation_j_mol` |
| `creep.dry_factor` | `mantle-rock` | 100 | Hirth and Kohlstedt 2003 | `dry_viscosity_factor` |
| `creep.melt_weakening` | `mantle-rock` | 26 | Mei et al. 2002 | new |
| `creep.andrade_exponent` | `mantle-rock` | 0.3 | Jackson and Faul 2010; Castillo-Rogez et al. 2011 | new |
| `melt.lockup_share` | `mantle-rock` | 0.4 | Costa et al. 2009 | new |
| `melt.grain_size` | `mantle-rock` | 1 × 10⁻² m (1 mm to 1 cm argued) | Bierson and Nimmo 2016 | new |
| `melt.permeability_exponent`, `melt.permeability_constant` | `mantle-rock` | 2.6, 58 | Miller et al. 2014 | new |
| `conductivity`, `diffusivity`, `heat_capacity`, `expansivity` | `mantle-rock` | 3, 10⁻⁶, 1,200, 1.5 × 10⁻⁵ | Turcotte and Schubert 2014 | `k_mantle`, `kappa_mantle`, `cp_mantle`, `alpha_mantle` |
| `density_liquid` | `basalt` | 2,800 kg/m³ | Stolper and Walker 1980 | new |
| `viscosity_liquid` | `basalt` | 10 Pa s (10 to 100) | Giordano et al. 2008 | new |
| `density` | `basalt` | 2,900 kg/m³ | Turcotte and Schubert 2014 | new |
| `rigidity`, `creep.*`, `creep.andrade_exponent` | `surface-ice` / `h2o` ice Ih | 3.5 × 10⁹ Pa; Goldsby and Kohlstedt 2001 | Gammon et al. 1983 | HF4d |
| `tides.k2_over_q` (reference object) | Jupiter, Saturn | 1.1 × 10⁻⁵, 1.6 × 10⁻⁴ | Lainey et al. 2009, 2017 | new |
| `tides.k2` (reference object, for checks) | Earth, Moon, Mars, Mercury, Venus, Io, Titan | 0.30, 0.024, 0.17, 0.57, 0.30, 0.125, 0.62 | section 15 | new |
| `heat.surface_loss` (reference object, for checks and pins) | Earth, Io, Enceladus, Moon | 47 TW; 2.2 W/m²; 15.8 GW; 0.014 to 0.021 W/m² (two Apollo sites) | section 15 | new |

### 8.2 Blocks (`packs/base-physics/formulas/`, new family `sse.melt` in `melt.json`)

Each is a formula block with its equation, cited source, worked example and a view. All are small enough to be formulas except the two marked.

| Block | Does | Equation |
|---|---|---|
| `sse.melt.fraction` | melt share of a parcel | 3.3, first line |
| `sse.melt.zone` (Rust: a 48-point sum) | zone thickness and mean share under a lid | 3.3 |
| `sse.heat.lid_base_temperature` | where the lid ends | 3.2, first line; `a_rh` a setting, 2.24 |
| `sse.heat.lid_growth` | dD/dt | 3.2 |
| `sse.heat.lid_conduction_with_burial` | surface and base conduction | 3.2, last pair |
| `sse.heat.boundary_layer_speed` | `u` | 3.4 |
| `sse.melt.supply` | melt made by convection | 3.4 |
| `sse.melt.in_place` | melt made by heating | 3.4 |
| `sse.melt.retained` | Darcy's melt share | 3.5 |
| `sse.melt.softening` | viscosity and rigidity factors | 3.6 |
| `sse.melt.processed` | what leaves with the melt | 3.7 |
| `sse.melt.heat_carried` | the rows of 3.8 | 3.8 |
| `sse.melt.erupted_share` | `f_ext`, one setting, 0.2 | 3.5; fudge register |
| `sse.tides.andrade_response` | uniform body, Andrade | 4.2a (supersedes HF4a's block, which becomes its α-off case) |
| `sse.tides.layered_response` (Big Block) | layered body | 4.2b |
| `sse.tides.host_power_ceiling` | the most a held moon can dissipate | 4.5 |

The march (`sse.core.history`) stays a Big Block that calls these. Its view is the thermal line itself: temperature, lid, crust and the heat split against time, with the marks on it. That chart is the "UI element that makes sense" for the whole family and is a screen package for later, not part of HF4b.

### 8.3 Fudge register (`docs/fudges.md`), by package

| Row | Change |
|---|---|
| `t_mantle_max_k` 2,200 | **removed** (HF4b.2) |
| HF1's "solidus at the lid's base" stand-in | **removed**, replaced by the melt model (HF4b.2) |
| `degas_gyr` 1 | **removed**: water leaves with melt (HF4b.2) |
| `dry_viscosity_factor` 100 | stays; moves to the sheet; note "when it bites is now physics" |
| `t_mantle_start_k` 1,950 | stays until C3 |
| `Tide { q: 100, k2: 0.3 }` | **removed** for heating (HF4c); stays for lock time until HF6 |
| `q_ref` family, `q_crit` | **removed** in HF5 and C5 |
| New: `erupted_share` 0.2 | stand-in until HF8 (magma's head) |
| New: melt-zone turnover length = mantle depth | scaling, permanent with reason |
| New: rigidity falls linearly to lock-up | stand-in until Bierson and Nimmo's form is read |
| New: liquid layers as very soft solids | stand-in in the layered block |
| New: tidal heat lands wholly in the melt zone | until the layered block gives shares |
| New: a moon's eccentricity is given, not explained | until HF6 |
| New: Earth's continental crust heat stays in the mantle | to review |
| New: intruded heat reaches the surface with no delay beyond the lid's own | permanent with reason (20 Myr steps) |

---

## 9. Packages

Each leaves main green. Sizes: S under a day, M one to two days. The heat flow thread owns all of them and keeps `handover/status/heat-flow.md`.

| Package | What | Needs | Golden | Size |
|---|---|---|---|---|
| **HF4b.0** numbers and blocks | Sheet rows of 8.1 (rock), the formula blocks of 8.2 with worked examples, `CoreSettings` reads moved to the door with **the same values** (so nothing moves). Nothing new is read yet | PR 660 merged; ER1 (PR 644) on main for the door, and if it is not yet, the rows and blocks land now and the reads move in HF4b.1 | reach lines only; UI reference rebuilt | S |
| **HF4b.1** the lid | `D_l` as a kept state with 3.2's rule; surface heat is the lid's conduction; HF1's clamp reads the kept lid; grid of 6.1; state kept at every point; the two-legs test | HF4b.0 | **core**, one slot (`surface.json`, the four core lines HF1 moved) | M |
| **HF4b.2** melt | 3.3 to 3.8: melt zone, supply and in-place melting, fertile share, Darcy's retained melt, softening, water and elements leaving, crust capped by the lid, the heat table; removes the 2,200 K cap, HF1's clamp and `degas_gyr`; the fields of section 7; the marks of 6.3; the energy test | HF4b.1 | **core**, one slot | M |
| **HF4c.1** tides read the state | Andrade response; the heating block takes `tides.im_k2`; tidal heat fed back each step with 6.2's rules; a measured `tidal_w_m2` pins it and the model's own number is published beside it (if question 2 is yes); the host ceiling; tells the small-moon thread | HF4b.2 | **core**, one slot (Sol's Io, Europa, Enceladus, Triton; fixtures with authored heat barely move) | M |
| **HF4c.2** layers | `sse.tides.layered_response`, the uniform-limit test, the seven Love numbers; the march passes its layers | HF4c.1 | core if any tidal world moves; else area | M |
| **HF2** readers read heat lost | As the heat flow thread planned it, on `heat_lost`, regimes marched with the history | HF4b.2 | surface goldens, core | M |
| **HF5** volcanism reads lava | The readers of section 7 move from `q_ref` to `melt_erupted` | HF2 | surface goldens (many) | M |
| **HF4d** ice shells | 3.10, from its own short note | HF4c.2 | surface goldens (icy moons) | M |
| **HF6** the orbit side | **Design first** (question 3) | HF4c.1 | | |
| **HF7** mantle water budget; **HF8** what erupts (magma's head) | Later; each replaces a registered stand-in | HF4b.2 | | |

Order: HF4b.0, HF4b.1, HF4b.2, HF4c.1, then HF2 and HF4c.2 side by side, then HF5, then HF4d.

**Red-first tests** (each seen to fail with its fix removed):

- HF4b.1: a young lid's surface heat falls as 1/√t while the mantle's own loss does not change (fails on today's code and on the half-space-on-the-mantle trial).
- HF4b.2: energy over a life is conserved to 10⁻⁶; a world with no heat makes no melt; crust never exceeds the mantle's basalt.
- HF4c.1: a uniform body at α = 0 equals HF4a's block; doubling the eccentricity quadruples the heat at fixed state.
- HF4c.2: the uniform limit to 10⁻⁹; a liquid sphere gives k2 = 1.5.

---

## 10. Goldens and guarded tests

**Will move, on purpose:**

- `tests/golden/surface.json`: every rocky fixture's envelope hash from HF4b.1 on (the history's steps are in it); icy fixtures at HF4d; sketches when regimes or landforms change (HF2, HF5).
- `tests/golden/contract.jsonl`: the same four core lines HF1 moved (two `formation_ran`, `system_resolved`, `surface_surveyed`), at HF4b.1, HF4b.2 and HF4c.1. Each is one turn in the core lane. HF4b.0 moves only manifest and stack hashes if any block is compiled in before ER3; say so when queueing.
- `tests/golden/contract-reach.txt`: new block lines at HF4b.0.
- Area goldens that hold a rocky world's heat or field: check `areas/kernel.jsonl` and the life and magnetic fixtures before queueing.

**Guards that are limits** (in `core_tests.rs` unless said):

| Guard | Scratch model says | If it fails |
|---|---|---|
| Mars's field dies 3.6 to 4.1 Gyr ago | 4.04 to 4.16 on a measure that reads 0.08 high, so about 3.96 to 4.08 | Stop. Do not move the range and do not tune. The date is now set by how fast early melting dries the mantle; report the dry-out curve and the core heat flow to the coordinator. The range itself looks sound to me (Lillis et al. 2013 put the end near 4.1 Gyr ago; Mittelholz et al. 2020 argue for a field as late as 3.7), so I am **not** asking Alex to change it |
| Earth's field never stops; started over 3.5 Gyr ago; inner core born 0.5 to 1.5 Gyr ago | Earth's mantle is 17 K cooler today; core heat flow up 3 per cent. Not run with the inner core | Report; do not adjust the nucleation fudge to compensate |
| Venus has no field because its core is not cooling | Venus's mantle ends near 2,190 K either way | Report |
| The Moon has lava plains and tubes; Mars has a rift (HF2's eleven) | Both erupt for their first 1 to 2 Gyr, too much | These pass or fail in HF2 and HF5, not here |

---

## 11. What could be wrong

- **I could not read a single paper.** Every figure in 8.1 is from memory. A wrong solidus or permeability number changes amounts, not the mechanism. The first job on 1 Dec is to read the fifteen or so sources of section 15 and correct the sheet.
- **The scratch model is not the kernel.** No inner core, no dynamo ledger. Its Mars date is a proxy that reads 0.08 Gyr earlier than the kernel on HF1.
- **Mars's field date is now near the guard's old edge**, and what sets it has changed from a typed time to the melting history. It may land at 4.1 to 4.2.
- **Lid mantles stay too hot**, so lava volumes are high. This design exposes that; it does not fix it (C3, HF7).
- **The supply rule is a scaling.** "The mantle turns over on its own depth" is good to a factor of 2.
- **The layered response is the riskiest code**: complex 6 by 6 matrices from first principles. The uniform-limit test and the Love numbers make an error visible, and HF4c.1 does not depend on it.
- **Grain size and the erupted share** are single numbers that Io's and the Moon's results lean on, and neither is well known.
- **Regime labels still feed the history** (`mobile-lid` or not). That is an existing exception to "a class is a label read off results"; a yield-stress rule for when a lid breaks would remove it and is not in this note.

---

## 12. What was tried and ruled out (from the heat flow thread, confirmed here)

- **Half-space cooling taken from the whole mantle**: kills Mars's field at once. Wrong reservoir (3.2).
- **Tides balanced against convection** (Fischer and Spohn style): Io at 22 W/m². Convection is not what carries Io's heat; melt is.
- **A uniform Maxwell Io at its solidus**: Im k2 about 3 × 10⁻⁶. Confirmed (4.3).
- **Eruption-limited melt removal**: it is Darcy's weakly heated limit, so it is not a separate choice (3.5).
- **One k2/Q for every world**: gives Enceladus about the right heat by luck and Io a fifth.

---

## 13. Questions for Alex

1. **Is this the mechanism you want: one melt model for every solid world, with the lid as its own kept state, and volcanism as the melt that leaves?** It means lid worlds will erupt 2 to 5 times too much lava for a while (the Moon far more), honestly logged, until the two older stand-ins upstream are replaced. *Recommend yes.* The other course is to hold it back until those are fixed too, which is a much bigger single change.

2. **For real bodies with a measured heat flow (Io, Enceladus), should the measurement set the heat, with the model's own figure shown beside it?** That is how the real-catalogues design treats measured orbits. It makes the Solar System's Io look and behave right now, and it keeps the gap in plain sight. Generated worlds always use the model. *Recommend yes.*

3. **Shall I design the orbit side next (HF6)?** That is the host planet's tide, resonance partners for moons, and eccentricity that dies away unless something holds it. It is what makes any heated moon honest, real or generated, and it is the robust route to Io's and Enceladus's heat. It touches formation (moons would need resonances) so it is a design of its own. *Recommend yes, after HF4c.1 lands, so it is written against real numbers.*

4. **"Heat pipe" as a label: should it mean "lava carries more than half the heat" in place of "more than 1 W/m²"?** It is the physical meaning (Moore and Webb 2013). It would also label the first few hundred million years of Mars, the Moon and Venus as heat-pipe worlds, which is what that paper argues but changes what a GM sees for young worlds. *Recommend yes, in HF5, not before.*

## 14. Routine choices, for the coordinator

- **C1** Sub-package names HF4b.0 to HF4c.2 as in section 9, three core-lane turns (HF4b.1, HF4b.2, HF4c.1). *Recommend as written.*
- **C2** The layered response as a Big Block. *Recommend yes;* if it is slow to get right, HF4c.1 ships alone.
- **C3** Replace the typed 1,950 K start with a derived one: the potential temperature at which the surface melt share equals the lock-up share, about 1,780 K for dry rock. In the scratch model it halves lid-world crust and moves Mars's date from 4.16 to 4.04. It also moves Earth's early history, so it is its own package in the core lane after HF4b.2, and Earth's guards are limits. *Recommend yes.*
- **C4** Tests of calibrated lid-world behaviour (HF2's eleven) are updated to follow this design once Alex accepts it, each change listed; tests of measured facts stay. *Recommend yes; this is the 26 Nov rule on tests.*
- **C5** `sse.heat.active_geology_lifetime` and its `q_crit` give way to the history's "last lavas" date, in HF5. *Recommend yes.*
- **C6** A thermal-line chart (the view for these blocks) goes to the screen thread that owns body cards, after HF2. *Recommend yes.*
- **C7** On 1 Dec, one pass by the citations thread over section 15 before any of these rows loses its `unverified` flag.

---

## 15. Sources, all unverified until 1 Dec 2026

Written from memory. Each needs reading at source, adding to the master list, and its number correcting on the sheet if wrong.

| Source | Used for |
|---|---|
| Andrault, D. et al. 2011, EPSL 304, 251 | deep liquidus (already cited in `core.rs`) |
| Armann, M. and Tackley, P. J. 2012, JGR 117, E12003 | stagnant-lid Venus loses heat by magmatism |
| Bierson, C. J. and Nimmo, F. 2016, JGR Planets 121, 2211 | Io: melt-dependent rheology, Darcy drainage, grain size |
| Castillo-Rogez, J. C., Efroimsky, M. and Lainey, V. 2011, JGR 116, E09008 | Andrade response |
| Choblet, G. et al. 2017, Nature Astronomy 1, 841 | Enceladus's heat from its loose core |
| Costa, A., Caricchi, L. and Bagdassarov, N. 2009, G-cubed 10, Q03010 | lock-up melt share |
| Crisp, J. A. 1984, J. Volcanol. Geotherm. Res. 20, 177 | Earth's magma output |
| Davaille, A. and Jaupart, C. 1993, J. Fluid Mech. 253, 141 | lid-base temperature step |
| Davies, J. H. and Davies, D. R. 2010, Solid Earth 1, 5 | Earth's 47 TW |
| Durante, D. et al. 2019, Icarus 326, 123 | Titan's k2 |
| Efroimsky, M. 2012, ApJ 746, 150 | Andrade tides |
| Fuller, J., Luan, J. and Quataert, E. 2016, MNRAS 458, 3867 | resonance locking in giant planets |
| Gammon, P. H. et al. 1983, J. Glaciol. 29, 433 | ice rigidity |
| Genova, A. et al. 2019, GRL 46, 3625 | Mercury's k2 |
| Giordano, D., Russell, J. K. and Dingwell, D. B. 2008, EPSL 271, 123 | melt viscosity |
| Goldsby, D. L. and Kohlstedt, D. L. 2001, JGR 106, 11017 | ice creep |
| Hirschmann, M. M. 2000, G-cubed 1, 1042 | solidus (already in `core.rs`, unverified there too) |
| Hirth, G. and Kohlstedt, D. 2003, AGU Geophys. Monogr. 138, 83 | rock creep, water, melt weakening |
| Howett, C. J. A. et al. 2011, JGR 116, E03003 | Enceladus's 15.8 GW |
| Jackson, I. and Faul, U. H. 2010, PEPI 183, 151 | Andrade exponent for olivine |
| Jara-Orué, H. M. and Vermeersen, B. L. A. 2011, Icarus 215, 417 | liquid layers in the propagator method |
| Katz, R. F., Spiegelman, M. and Langmuir, C. H. 2003, G-cubed 4, 1073 | liquidus, melt share law, water's effect |
| Khurana, K. K. et al. 2011, Science 332, 1186 | a melt-rich layer in Io |
| Knapmeyer-Endrun, B. et al. 2021, Science 373, 438 | Mars's crust thickness |
| Konopliv, A. S. and Yoder, C. F. 1996, GRL 23, 1857 | Venus's k2 |
| Konopliv, A. S. et al. 2020, GRL 47, e2020GL090568 | Mars's k2 |
| Lainey, V. et al. 2009, Nature 459, 957 | Io's k2/Q and heat; Jupiter's k2/Q (already cited) |
| Lainey, V. et al. 2017, Icarus 281, 286 | Saturn's k2/Q |
| Lillis, R. J. et al. 2013, JGR Planets 118, 1488 | when Mars's dynamo stopped |
| Mei, S. et al. 2002, EPSL 201, 491 | melt weakening factor |
| Miller, K. J. et al. 2014, EPSL 388, 273 | permeability law |
| Mittelholz, A. et al. 2020, Science Advances 6, eaba0513 | a late Martian dynamo |
| Moore, W. B. 2001, Icarus 154, 548 | Io's heat leaves by melt, not convection |
| Moore, W. B. and Webb, A. A. G. 2013, Nature 501, 501 | heat-pipe worlds (already cited) |
| Morschhauser, A., Grott, M. and Breuer, D. 2011, Icarus 212, 541 | the lid, melt zone and crust rules for Mars |
| O'Reilly, T. C. and Davies, G. F. 1981, GRL 8, 313 | the buried lid's temperature |
| Padovan, S. et al. 2015, GRL 42, 1029 | Mercury's crust |
| Park, R. S. et al. 2024, Nature (Io's tidal response from Juno) | Io's k2 0.125 |
| Roberts, J. H. and Nimmo, F. 2008, Icarus 194, 675 | layered tidal heating, Enceladus's shell |
| Sabadini, R., Vermeersen, B. and Cambiotti, G. 2016, Global Dynamics of the Earth, Springer | the propagator method |
| Segatz, M. et al. 1988, Icarus 75, 187 | Io's soft-layer models (already cited) |
| Solomatov, V. S. and Moresi, L.-N. 2000, JGR 105, 21795 | stagnant-lid scaling (already cited) |
| Spencer, D. C. et al. 2020, JGR Planets 125, e2020JE006443 | intrusions set Io's lid thickness |
| Stolper, E. and Walker, D. 1980, Contrib. Mineral. Petrol. 74, 7 | melt density |
| Turcotte, D. L. and Schubert, G. 2014, Geodynamics, 3rd edn | half-space cooling, rock properties (already cited) |
| Langseth, M. G., Keihm, S. J. and Peters, K. 1976, Proc. Lunar Sci. Conf. 7, 3143 | the Moon's heat flow at the Apollo 15 and 17 sites |
| Veeder, G. J. et al. 2012, Icarus 219, 701 | Io's heat output |
| White, S. M., Crisp, J. A. and Spera, F. J. 2006, G-cubed 7, Q03010 | intruded against erupted |
| Williams, J. G. et al. 2014, JGR Planets 119, 1546 | the Moon's k2 |

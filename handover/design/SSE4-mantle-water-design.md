# SSE4: water in the mantle, how it leaves and how it comes back (track HF, package HF7)

**Status: accepted by Alex, 28 Nov 2026 13:17 UTC** ("agree with all... is there any reason we dont have water from formation... fix upstream if needed"). See section 8A. Written 27 Nov 2026 by the thread "Fix low heat flow in worlds" at the coordinator's request, after HF4b.2 (melt) stopped at its guarded limits. Nothing here is built. Every paper figure below is from memory and unverified until the paper search returns (1 Dec 2026). Parent design: `SSE4-melt-and-heat-design.md` (sections 3.6, 3.7, 8.3, 10).

## 1. The short answer

HF4b.2 makes the mantle's water a result of melting, not a typed 1 Gyr drying time. It is still only a share running from 1 (wet) to 0 (dry), with a two-state viscosity (dry rock 100 times stiffer, `dry_factor` on the sheet, in the fudge register). That share now decides two guarded limits, and it decides them in opposite directions depending on one open rule:

| Water rule in HF4b.2 | Mars's field dies (guard 3.6 to 4.1 Gyr ago) | Venus (guard: no field) | Earth's inner core born (guard 0.5 to 1.5 Gyr ago) |
|---|---|---|---|
| A (ruled, 27 Nov): erupted water goes to the air, intruded water returns with sinking crust | 3.85 ✓ (mantle 35% wet today) | inner core born 0.57 Gyr ago, field restarts ✗ (mantle 31% wet) | 0.31 ✗ |
| B: no water returns with the crust | 4.17 ✗ (10% wet) | no field ✓ (2% wet) | 0.31 ✗ |
| B, with C3's derived start (1,780 K) | 4.15 ✗ | no field ✓ | 0.03 ✗ |
| HF4b.1 (water lost on a typed 1 Gyr e-folding) | 3.95 ✓ | no field ✓ | 0.91 ✓ |

HF7 replaces the share and the factor with a water **amount** and a flow law that reads it. It is the physics the two dates actually hang on. **It does not fix Earth's inner core** (section 5): Earth's mantle stays wet in every run, and with its water held at full the inner core is still born only 0.41 Gyr ago. That is a separate cause, so it gets a separate question (section 8, question 5).

## 2. What is kept

One more number per rocky world, replacing `wet`:

| State | Unit | Replaces |
|---|---|---|
| `W_m`, water in the mantle | kg | `wet`, the 0 to 1 share |
| `W_cr`, water held in the crust | kg | HF4b.2's crust water share |
| water handed to the air, dated | kg per step | nothing (HF4b.2 drops it) |

The concentration `C = W_m / M_mantle` (mass fraction, or ppm H₂O) is what the physics reads. The conserved chain's rule holds: water leaves the interior as an amount with a date, never as a description.

**Where the first amount comes from.** Formation's budget for the world, less what its magma ocean outgassed. This is `SSE4-system-formation-design.md` stage 8 ("magma ocean, outgassed second air"). Until formation hands over a mantle water amount, the start is the water a magma ocean's solid leaves behind, at the sheet's partition coefficient (question 1).

## 3. Viscosity reads the water (the dry factor goes)

Water-weakened olivine creep (Hirth and Kohlstedt 2003):

```
η(T, C) = η_ref · (C / C_ref)^(−r) · exp(E/R (1/T − 1/T_ref))        C floored at C_dry
```

- `r` is about 1 to 1.2 for dislocation creep and about 1 for diffusion creep.
- `C_ref` is the water of the mantle the reference viscosity was calibrated on (Earth's upper mantle, about 100 to 200 ppm H₂O). `C_dry` is the floor below which rock behaves as dry: nominally anhydrous minerals keep some water, about 10 ppm.
- At `C_ref` this is HF4b.2's wet rock exactly. At the floor it is `(C_ref / C_dry)^r` times stiffer: 10 to 20 for these numbers, against today's typed 100.

**Can `dry_factor` 100 leave the fudge register? Yes.** Once this law is in, the factor is no longer a number anyone types. It is what the sheet's `r`, `C_ref` and `C_dry` give, each a cited row on `mantle-rock`. `η_ref` stays in the register as a calibration on Earth, as it is now. Expect the stiff-dry limit to come out 5 to 10 times softer than 100. That alone makes dry lid worlds lose heat faster, which is the direction the "lid mantles too hot" fault needs (melt design section 5, point 3).

## 4. How water leaves and comes back

**Leaving with melt.** The water entering the melt is the processed mass times the mantle's concentration, enriched by batch partitioning. With `D ≈ 0.01` (Hirth and Kohlstedt 1996), melt at share `F̄` carries `C / (F̄ + D (1 − F̄))`, so almost all the water of the rock that melted. This is HF4b.2's `w ← w · exp(−processed)` written as an amount, and it reads the same block (`sse.melt.processed`).

**Where the melt's water goes.** Three places, and HF4b.2's open rule is the middle one:
- **Erupted share:** degasses to the air, dated (as rule A).
- **Intruded share:** the melt keeps the water it can dissolve at the pressure where it freezes, and loses the rest to the crust above it as gas. Water's solubility in basaltic melt rises with pressure (roughly `C_sat ≈ a √P`, a few wt% at 100 MPa; Dixon et al. 1995, Moore et al. 1998). So shallow intrusions degas and deep ones keep their water. That replaces rule A's "all intruded water stays" with physics. How deep the intrusions freeze is the lid's own base temperature against the melt's solidus, already in the march.
- **Bound in the crust:** what the intrusions keep. It goes back to the mantle when the crust sinks below the lid (rule A's return, now an amount).

**Coming back under plates.** Sinking slabs carry the water of hydrated crust back down. The amount is not a typed rate:

```
regassed = plate speed u × ridge length × hydrated thickness × crust water × χ(T_m)
```

- The hydrated thickness is set by how deep cracks let seawater in. It needs an ocean, so a dry plate world regasses nothing.
- `χ(T_m)` is the share that survives dehydration on the way down, and it falls as the mantle gets hotter (Crowley et al. 2011; Cowan and Abbot 2014).
- The plate speed is the march's own `u`. The ridge length is the plate model's.
- This replaces HF4b.2's "plates return everything as fast as the mantle turns over".

**Water lowers the solidus.** By ΔT = 43 · X^0.75 K, with X in wt% (Katz et al. 2003). The melt design deferred this to HF7 (section 3.3). A wet mantle melts more and dries faster; a dry one melts less. This is the feedback that should separate Venus from Mars.

## 5. What HF7 must decide, plainly

1. **Venus's field.** Under rule A, Venus keeps 31% of its water, its core cools, an inner core freezes 0.57 Gyr ago and its light elements restart the dynamo. With 2% (rule B) it does not. The threshold lies between those, and finding it with the scratch model is HF7's first job. If HF7's physics leaves Venus wetter than that threshold, Venus will have a field. That would be a real result, and I would bring it to Alex rather than tune it away. Real Venus's lack of a field is itself argued (a core with no inner core yet, or a stratified one; e.g. O'Rourke 2020), and the guard is a limit, so the sources need reading first.
2. **Mars's field.** It needs Mars's mantle to dry more slowly than rule B gives: 4.17 Gyr ago under B, against the guard's 4.1. Pressure-limited degassing of intrusions (section 4) is the mechanism that keeps a fraction of Mars's water. Mars's intrusions freeze under a lid hundreds of km thick, so they are deep and mostly keep their water.
3. **Earth's inner core.** **Not decided by HF7.** Earth stays 98% wet in HF4b.2, and holding it at 100% moves the birth only from 0.31 to 0.41 Gyr ago. The cause is that HF4b.2's melt carries 28% of young Earth's heat (as the melt design predicted, 45 of 160 TW), which cools the early mantle, stiffens the lower mantle, and slows the core's cooling (the core ends 28 K hotter). Candidates for what is missing, none of them to be tuned: the lower mantle's own viscosity law (a typed factor of 100, in the register); the core's conductivity (130 W/m/K, disputed from 30 to 150); and whether young Earth's ridge melt really leaves the mantle or is re-mixed. See question 5.
4. **The dry factor 100.** It leaves the register (section 3).

## 6. Blocks and sheet rows

- **Sheet rows, `mantle-rock`:** `creep.water_exponent` (r), `creep.water_ref` (C_ref), `creep.water_floor` (C_dry), `melt.water_partition` (D), `phases.solidus_water_drop` (43 K, 0.75). **On `basalt`:** `water_solubility` (a, and its pressure law). The rows `creep.dry_factor` and the fudge row for it are removed.
- **Blocks** (`melt.json`): `sse.melt.water_taken` (partitioning), `sse.melt.water_degassed` (the solubility split), `sse.heat.water_creep` (viscosity factor from C), and `sse.plates.regassing`. All four are formulas.
- **Fields** (melt design section 7): `water` becomes ppm H₂O. New fields: `water_crust` and `water_degassed` (kg per Myr, the air's input).

## 7. Packages

| Package | What | Golden |
|---|---|---|
| HF7.0 | Scratch check in `sse4/melt-proto/`: rules A and B against the section 3 and 4 physics, all three guards, numbers before code | none |
| HF7.1 | Water as an amount, the flow law, partitioning, solubility-limited degassing, the return with sinking crust; joins HF4b.2 in **one core-lane turn** (either alone breaks a guard) | core |
| HF7.2 | Regassing under plates; the dated water handed to the air (the air model and life's F2 read it) | core |
| HF7.3 | The solidus drop with water | core |

## 8. Questions for Alex

1. **The first water amount: formation's, or a stand-in until formation provides it?** *Recommend:* formation's, through the conserved chain. Until then, the stand-in is a magma ocean's leftover at the partition coefficient, in the fudge register.
2. **Degassing of intrusions by solubility (physics) rather than a share?** *Recommend yes.* It is the one mechanism that can tell Mars from Venus without a number chosen to do it.
3. **HF4b.2 and HF7.1 land together, in one core-lane turn?** *Recommend yes.* Either alone breaks a guarded limit, and never merging red rules out landing one first.
4. **The dry factor leaves the fudge register, derived from the sheet's water law?** *Recommend yes.* The stiff-dry limit then comes out 10 to 20, not 100. That moves every dry lid world, and the change is honest.
5. **Earth's inner core (0.31 Gyr ago against a guard of 0.5 to 1.5): a short investigation before HF4b.2 merges, checking the lower mantle's viscosity law and the core's conductivity against their sources once the paper search is back?** *Recommend yes.* Don't move the guard and don't touch the nucleation fudge. If a sourced lower-mantle law still gives an Earth core under 0.5 Gyr old, bring the numbers to you then.

## 8A. Alex's answers, 28 Nov 2026 (13:17 UTC)

All five recommendations of section 8 are rulings. On question 1 he added: "is there any reason we dont have water from formation... fix upstream if needed". What the code does today (checked on main 30574a2a):

- Formation (`crates/sse-formation/src/early.rs`, stage 8) knows each world's bulk water, keeps a typed `MANTLE_RETAINED` = 0.3 of it in the solidifying mantle (Elkins-Tanton 2008) and outgasses the rest. It publishes the bulk share (`interior.water_mass_fraction`) and the surface water, never the mantle's amount. Catalogued worlds carry no water amount at all.
- The 0.3 is the same for every world, including one whose magma ocean lasted about 100 Myr inside the runaway limit and lost its water to space (`desiccated`, Hamano et al. 2013). Venus is that world.

So the upstream fix is HF7's first slice (HF7.0b, below): formation publishes `interior.mantle_water` (kg); the share kept becomes a cited block reading how long the magma ocean lasted and whether its air escaped, in place of the typed 0.3; the thermal history starts from that amount; Sol's worlds get catalogued values, cited and marked unverified until read at source.

| Package | What | Golden |
|---|---|---|
| HF7.0 | Scratch check, as section 7 | none |
| HF7.0b | Formation hands the mantle's water over as an amount; the kept share as a block; catalogue values for Sol | core (formation lines) |

### 8B. HF7.0b: how much water a magma ocean leaves in the mantle (built from 28 Nov)

The fudge register already names the fix for the typed 0.3 (row `early.rs` `MANTLE_RETAINED`: "solubility laws for H2O and CO2 in magma"). One mechanism for every world, a Rust block (`sse.formation.mantle_water`, a sum over the ocean's freezing):

- The ocean freezes from the bottom up in 32 equal mass steps. Each step's new rock keeps the water of the melt it froze from at `D + φ_t`: the mineral partition (`melt.water_partition`, about 0.01) plus the melt trapped between grains (about 0.01; Elkins-Tanton 2008). Both are sheet rows on `mantle-rock`.
- The melt left over is in balance with the steam above it by water's solubility in magma, `x = a · p^b` (about 6.8e-8 Pa^-0.7 · p^0.7, mass fraction; Lebrun et al. 2013 after Carroll and Holloway), on `mantle-rock`'s melt row. The steam pressure is the air's water times gravity over area, found by halving at each step.
- **A world whose magma ocean is inside the runaway limit** (formation's `desiccated`, Hamano et al. 2013) loses the air's water to space as fast as it degasses. Each step's leftover melt then balances against an air with no water left, so the rock frozen late keeps almost nothing. Venus's mantle comes out drier than Earth's or Mars's because of how its ocean ended, not because of a number chosen for it.
- Published as `interior.mantle_water` (kg), read by the thermal history (HF7.1). The bulk share and the surface water stay as they are, now the remainder: water is conserved between mantle, air and space, and the conserved chain checks the sum.
- Catalogued worlds (Sol's) take the block's answer from their catalogued bulk water where there is one. Earth is checked against published mantle water estimates (about 1 to 3 oceans' worth, unverified), never tuned to it.

Every number here is from memory and unverified until 1 Dec 2026.

## 9. Sources (all unverified until 1 Dec 2026)

Hirth and Kohlstedt 1996 (EPSL 144, 93: water in the oceanic upper mantle) and 2003 (AGU monograph 138: rheology of the upper mantle); Katz, Spiegelman and Langmuir 2003 (G-cubed 4, 1073); Dixon, Stolper and Holloway 1995 (J. Petrology 36, 1607) and Moore, Vennemann and Carmichael 1998 (Am. Mineral. 83, 36), water solubility in basalt; Crowley, Gérault and O'Connell 2011 (EPSL 310, 380) and Cowan and Abbot 2014 (ApJ 781, 27), regassing; Morschhauser, Grott and Breuer 2011 (Icarus 212, 541), Mars's water; Sandu, Lenardic and McGovern 2011 (JGR 116, B12404), water and thermal histories; O'Rourke 2020 (GRL 47, e2019GL086126), Venus's dynamo; Elkins-Tanton 2008 (EPSL 271, 181), magma ocean volatiles; Lebrun et al. 2013 (JGR Planets 118, 1155), water solubility in magma oceans; Hamano, Abe and Genda 2013 (Nature 497, 607), magma oceans inside the runaway limit.

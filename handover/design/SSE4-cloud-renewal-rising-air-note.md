# Giant cloud renewal in rising air (WS3b): design note, DRAFT

Status: **draft, not accepted.** It was written while Alex's decision card on renewal time is open, at the coordinator's request. Nothing gets built from this note until Alex picks "add renewal by rising air" on that card. If he picks another option, the note is shelved.

Track WS (stirred clouds). Builds on design `SSE4-stirred-clouds-design.md` sections 3.4 to 3.6, WS3 (`renewal_s`, `sse.weather.particle_fall_time`, `sse.weather.plume_count`), and the WS4 finding in `handover/packages/WS4.md`.

All numbers below are worked by hand and are **unverified**. Sources marked "from memory" are to be checked when the paper search returns on 1 Dec 2026.

## 1. The problem in one paragraph

WS3 renews a giant's top deck only by its grains falling out. With the pack's 1 µm ammonia grains, Jupiter's deck lasts about 150 days. WS4 showed that cloud of that age has been stretched 2^30 to 2^54 by the eddies, so all of it reads as the local mean and the picture loses its detail. A real deck is also made and unmade by air moving up and down through it. Air rising through the deck brings vapour that condenses. Air sinking through it warms and the cloud evaporates. WS3 leaves this out, and leaving it out is a missing cause, not a wrong constant. This note adds that cause. It does not adjust grain size.

## 2. The physics

A parcel of the deck's air is replaced when air crosses the deck from below or from above. Heat flowing up through the deck sets how fast that happens. On a giant, that heat is carried upward by moist convection, the tall storms WS3 already counts as plumes. The rising air comes back down as slow sinking over the rest of the planet. Both halves of that turnover renew the deck: rising air makes fresh cloud, sinking air clears old cloud.

Each kilogram of rising air carries a set amount of energy, `Δh` (its moist static energy excess, mostly the latent heat of the water it condenses lower down). The air mass crossing a square metre each second is then:

```
mass flux        ṁ = s · F_int / Δh                                   kg m⁻² s⁻¹
deck's air       M = Δp / g = p_base (1 − e^(−thickness_h)) / g      kg m⁻²
overturn time    τ_up = M / ṁ = p_base (1 − e^(−thickness_h)) · Δh / (g · s · F_int)
renewal time     1/τ = 1/τ_fall + 1/τ_up
```

- `s` and `F_int` are the same plume heat share and internal heat that `sse.weather.plume_count` already uses. No new heat number is invented.
- `p_base`, `thickness_h` and `g` are the deck numbers `renewal_s` already reads.
- `Δh = L_v q`: the latent heat of the deepest condensate (water on Jupiter and Saturn) times its abundance below its cloud. The latent heat is already in `ClimateSettings` (`latent_j_kg`). The abundance comes from the composition the deck heights are built from.
- The two rates add because they are two independent ways for a grain to leave the deck.

This is one new block, `sse.weather.overturn_time`, with inputs (F_int, s, Δh, p_base, thickness_h, g). `renewal_s` then returns the combined τ. It stays one number per giant, as WS3 has it.

## 3. Worked by hand: Jupiter (unverified)

| Quantity | Value | Source |
|---|---|---|
| Internal heat `F_int` | 5.4 W/m² | Li et al. 2018 (from memory) |
| Plume share `s` | 1.0 | pack row `plume_heat_share` |
| Water abundance at depth | about 2 to 3 times solar, `q` about 0.01 to 0.02 by mass | Li et al. 2020, Juno MWR (from memory) |
| `L_v` | 2.5 × 10⁶ J/kg | data sheet |
| `Δh` | 2.5 × 10⁴ to 5 × 10⁴ J/kg | |
| Deck: base about 0.7 bar, `thickness_h` 0.5 | Δp ≈ 0.28 bar, M ≈ 1,100 kg/m² | pack, base-weather |
| `ṁ` | 1 × 10⁻⁴ to 2 × 10⁻⁴ kg m⁻² s⁻¹ | |
| `τ_up` | about 60 to 120 days | |
| `τ_fall` | about 150 days | WS3 |
| combined `τ` | about **40 to 70 days** | |

Rough check: a sinking speed of `ṁ / ρ` ≈ 0.5 to 1 mm/s near 0.5 bar. Cooling-rate estimates for Jupiter's upper troposphere (Conrath, Gierasch and Leroy 1990, from memory) give sinking of the same order, so the two routes roughly agree.

Scaling to other worlds: `τ_up` goes as `Δp Δh / (g F_int)`.

| World | `τ_up` | What renews the deck |
|---|---|---|
| Saturn (F_int about 2 W/m²) | a few times Jupiter's | mostly fall-out |
| Uranus (F_int under 0.1 W/m²) | about a hundred times Jupiter's | fall-out alone, and its deck is quiet, as seen |
| Neptune | between Saturn and Uranus | |
| Hot giants and brown dwarfs | hours to days | overturning, so they come out fully churned, as design section 7 expects |

No world is special-cased.

## 4. What it does and does not fix

- Even at 40 to 70 days, cloud of the mean age is still stretched about 2^8 to 2^25 on Jupiter, finer than the 1024 face's roughly 440 km pixel. **This note alone does not make the mean-age cloud visible.** It brings τ to the right order and puts the plumes and the turnover on the same heat budget.
- The detail the eye sees then comes from the young share of the cloud. With the design's two-generation lattice, cloud under 20 days old is a small share of the weight. See question 2.
- The cloud's fresh structure then has real causes: plumes, the storms' cores and the band edges. Each is stirred for the time it has actually lived.

## 5. Open questions, each with a recommendation

1. **One τ per giant, or one per band?** Rising air is not spread evenly. On Jupiter, zones are thought to rise and belts to sink in the upper troposphere (Ingersoll et al. 2004, from memory), and lightning sits in the belts. A τ per band would make zones fresh and belts cleared. **Recommend: one τ per giant now (WS3b), with a per-band τ recorded as a follow-up.** Per band needs the band's vertical motion, which no block gives yet, and the WS4 lattice already assumes one τ.
2. **Keep two generations, or move to an age spread that favours young cloud?** If cloud is made all the time and lost at rate 1/τ, its ages fall off exponentially. More of what is on show is then young (about 40 per cent under 20 days at τ = 45 days) than with the design's two generations peaking at τ. **Recommend: keep two generations for WS5, and measure on Sol's Jupiter whether the picture holds detail.** If it does not, a third, young generation at τ/4 is the cheapest step towards the exponential spread. It would be a design change, brought back to Alex first.
3. **Where does `Δh` come from when the deep water abundance is not held?** **Recommend: from the giant's composition through the same vapour curves the deck heights use. Where none is held, use a registered fallback (solar water) as a fudge row**, just as `fallback_particle_um` is today.
4. **Should absorbed sunlight below the deck count towards the heat that drives the turnover, as well as `F_int`?** Jupiter absorbs about 8 W/m² of sunlight, much of it below the ammonia deck (from memory). **Recommend: internal heat only for WS3b, and raise sunlight when the energy view (HF track) gives the absorbed profile by depth.** Counting it now would mean guessing the depth.
5. **Does the plume count change?** No. `plume_count` still says how many tall storms carry the heat. This note uses the same heat to say how fast the rest of the deck is replaced. Each plume is drawn as before.

## 6. Build package (only if Alex picks this option)

**WS3b, size S:**

- One new block, `sse.weather.overturn_time`, cited.
- `renewal_s` adds the two rates.
- A fudge row for the fallback water abundance, if one is used.
- Checks:
  - Jupiter's τ falls between 30 and 100 days (absolute).
  - Uranus's τ equals its fall time within 5 per cent (overturning is negligible there).
  - With `F_int` set to 0, τ equals τ_fall exactly.
  - Each check goes red with the overturn term removed.
- Goldens: the flow table's `renewal_days` and the stirred patches' ages move, as area-only re-blesses (`weather-giants` lines and the stirred lines). `contract.jsonl` is not touched.
- WS5 starts after it, as planned.

## Sources

- Stokes 1851, fall speed (on file, `handover/citations/papers/stokes1851.json`).
- Gierasch et al. 2000, Nature 403, 628, moist convection carrying the internal heat (on file through `sse.weather.plume_count`).
- Li et al. 2018, Nature Communications 9, 3709, Jupiter's internal heat (from memory, unverified).
- Li et al. 2020, Nature Astronomy 4, 609, Juno water abundance at the equator (from memory, unverified).
- Conrath, Gierasch and Leroy 1990, Icarus 83, 255, upper-troposphere circulation from temperatures (from memory, unverified).
- Ingersoll et al. 2004, in *Jupiter: The Planet, Satellites and Magnetosphere*, chapter 6, zones and belts rising and sinking (from memory, unverified).

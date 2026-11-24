# FT5 maturity: being short or ahead acts on the figure (design 5.2)

Accepted by Alex on 23 Nov (size rule 5.2, questions 15 to 18). This replaces FT2's flat size rule for every advance that has a part family. The flat rule stays only for the `fictional` family.

## What changed (`crates/sse-kernel/src/tech.rs`)
- **Families.** The kernel reads `families[]` from the pack stack. Each one has a maturity rule:
  - `worse_by {factor}`: the figure is divided by the factor per level short, or multiplied where lower is better. Engine 7, material 5, store 3.
  - `toward {share}`: `share`^n of the way from their older figure to the mature one. Near-ceiling devices use 2/3.
  - `same`: no change, because mass is mass.
  - `floor`: no change, and the advance states its floor.
  - `size {size_per_level_short, shrink_per_level_beyond, shrink_levels}`: the figure stays and the part's size scales. This is the old flat rule, kept for fictional and unknown devices.
  - An advance with no family is `fictional`. A family the stack doesn't define is a stack problem.
- **Short of home.** The first-generation figure is never worse than the best older advance they hold for the same figure and part. The reading names that advance (`floored_at`), in words such as "2 levels short: Woven carbon cable, no better yet than their PBO fibre (3718000 J/kg)".
- **Past home.** The figure improves by the ladder's `improve_per_level_beyond` (1.5) per level until it reaches the next cited `allowed` or `ceiling` mark, then stops. With no further mark it stays as it is. `same` and `floor` families never improve.
- **Fixed figures.** Figures an advance lists in `fixed` (physics, such as fuel energy) never move.
- **Size follows the figure.** As a first-order estimate, `size_factor` is the mature figure over the figure now: an engine at 1/7 of its exhaust is 7 times the part.
- **Fuel-specific marks.** A mark can carry `for` one fuel. It applies only to a part whose `fuel` matches, or to an advance burning it (`fuel` on the advance, else on the part in `tech_reading`). The 5.18 km/s hydrogen-oxygen ceiling no longer makes 6 km/s fictional for a nuclear rocket, and it caps the chemical rocket's improvement. This replaces RK2's rule (PR 361), which skipped every `for` ceiling; the RK2 thread agreed. `for` reads as "burning or made of", so the working_k ceilings (`uo2-fuel`, `tungsten`) match an advance's `fuel` once one sets working_k. RK2's ranking, the same part first, is kept.
- **New fields on each need:** `figure_now`, `maturity` (home, fixed, worse_by, toward, same, floor, size or improved), `family` and `floored_at`. Each chip on `leans_on` carries `family`, and `where` when a fictional advance gives one.
- **The ladder** no longer has `size_per_level_short`, `shrink_per_level_beyond` or `shrink_levels`. Those numbers live on the `fictional` family. `scripts/build-base-tech.py` drops them and gives the chemical rocket `"fuel": "hydrogen-oxygen"`.

## Checks (`sse-kernel/src/tech_tests.rs`)
- **Earth beanstalk at level 4.** Nanotube cable, 2 levels short, `worse_by` 5 per level, gives 1.9 MN·m/kg. That is worse than their PBO fibre, so the cable is floored at 3.718 MN·m/kg and the part is 12.6 times its size.
- **Fusion torch 1 level short.** Its exhaust is 347.5/7 km/s with a size factor of 7 and no floor. The fuel energy is `fixed`.
- **Fuel-specific ceiling.** 6 km/s reads as `can build` by the nuclear thermal rocket and is not fictional. With `fuel: hydrogen-oxygen` it is fictional.
- **Chemical rocket at level 6.** Three levels past home, it improves to the 5.18 km/s ceiling for its own fuel, not to the nuclear rocket's 8.58 mark. Its size factor is 4.565/5.18.
- **A setting's fictional reactionless drive** 3 levels short is 1000 times the size, and its chip carries `where`.

## Golden
Nothing moves. Physics doesn't read tech, and the contract has no tech_reading case whose answer changed.

## Next
- Section 15, "keeping the books": the `balances[]` as inspectable blocks fed by a design's quantities. A balance that doesn't close reads as fictional, never as a refusal. This also covers the Carnot heat pump, plus a real-hardware check on the `real_craft[]` rows that can be checked.

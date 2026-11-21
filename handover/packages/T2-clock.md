# T2 clock: one clock for the universe, and ageing one system

Status: done (21 Nov 2026). Kernel track. Brief: `handover/design/SSE4-top-level-ui-and-onboarding.md` sections 12.2 and 17.1 (Alex accepted both on 21 Nov). This replaces T2's per-system `system.now_year`.

## The model
- **One campaign now.** It is the `time.campaign_now` event (`set_campaign_now`). Every system's age is its own base age plus two terms:
  - the campaign time elapsed since the clock was first set (the latest `campaign.now_s` minus the first);
  - its own authored `system.age_shift_myr`.
  This is `crate::clock`. `worlds_at` applies it, and so does everything built on it: `resolved_system`, `plan_route`, `survey_surface` and so on. So `spec.age_myr` is the age at the campaign's now.
- **Reading "now".** Once the clock has moved a system (an age shift, or time elapsed), "now" is that system's present on its own dates, not the latest event. Without this, a kept look-ahead's future would read as the present. Systems that were never moved still read the latest event, as before, so no golden changed for them.
- `system.now_year` is still read on old saves, but it is never written again.

## Accepting a look-ahead (`commit_look_ahead`)
`accept` takes one of two values, at star age `move_now_myr` (default: the look-ahead's target).
- **`campaign_now`** ("make this the campaign's now"). The campaign clock moves on by the whole years between, so every system is that much older. The branch is kept as this system's future. If the clock was never set, its first reading is written at 0, which becomes the epoch. The answer carries `campaign_now_s`.
- **`system_age`** ("make this system this old, today"). The clock stays. The look-ahead is discarded and run again from a birth moved earlier, so its events fall before today. It is committed, then the same steps as `set_system_age` run with the commit as the cause. The answer carries `system_age`.
- **Neither.** The branch is kept as history and nothing moves.
- **Only `move_now_myr`** means `system_age`. That is the common GM meaning, and the existing UI call keeps working.

## `set_system_age {system, age_myr, composition?}`
- Makes the star older or younger at the campaign's now.
- Writes a timeless authored `system.age_shift_myr`, which is then reconstructed around (17.1):
  - **`sse_cosmos::origin::reconstruct`** gives one plausible past, deterministic and with no draws. The birth radius maximises `N(R; R_now, sigma(age)) x SFR(R, t_birth) x R`, and also `N(feh; gas feh(R, t_birth), 0.1)` when the composition is kept.
  - The drift spread is 3.6 kpc at 8 Gyr, growing as sqrt(age) (Frankel et al. 2018; churning, Sellwood and Binney 2002). The settings are in pack `cosmology.origin`.
  - The path runs from birth to now: the guide radius moves linearly in time, and the azimuth is integrated on a flat 220 km/s curve.
  - The why reads, for example: "Formed 9.10 Gyr ago at 6.7 kpc from the centre, 3.2 kpc nearer the centre than its orbit now, and has drifted outward since. ... One plausible past." That is the Sun's radius in the default galaxy.
  - It is written as a `system.origin` event with evidence resolved and facts `origin.*`: label, birth_gyr, age_gyr, birth_r_kpc, birth_phi_deg, now_r_kpc, migrated_kpc, feh_gas, feh, composition, why, unusual and path as JSON text. The age-shift event is its cause.
- **Composition.**
  - `follow` (the default): the star's `star.metallicity_z` becomes the birthplace gas. Formation-adopted systems read it too.
  - `keep`: birthplaces are weighed by the star's own [Fe/H]. If no birthplace of that age explains it, `origin.unusual` says why.
  - The planets are not re-derived yet. The answer says `rederive: true` when the Z changed.
- **IMPOSSIBLE** only when no origin fits: born before the Big Bang, before the disc formed stars, or when nowhere in the disc was forming stars. The shift is still written, as `authored-impossible`, with `system.age_impossible` giving the reason. A later possible age clears it.
- **A hand-pinned birthplace** (`origin.pinned` true) is never reconstructed over.

## Checks
- `sse-cosmos` `origin_tests` (4):
  - an old star beside the Sun came from further in, from poorer gas, and the spread grows as sqrt(age);
  - a kept rich composition comes from further in than a poor one, and an odd one is marked unusual;
  - IMPOSSIBLE only before the disc or before the Big Bang;
  - the pack carries the defaults.
- Kernel `one_clock_and_the_two_ways_to_accept`:
  - `system_age` moves only that system: its birth is 2 Gyr earlier, the other system and the clock stay, it reads at its present, and its history has `system.origin`;
  - `campaign_now` moves the clock by exactly 500 Myr, both systems age, and births stay;
  - keep, follow, impossible (stamped `authored-impossible`) and younger again all work;
  - refusals are covered, and a second kernel replays the whole session to the same bytes.
- Contract: 9 new cases, plus `set_system_age` in the op list. No engine golden changed.
- Existing look-ahead tests still pass unchanged; their `move_now_myr` calls are now `system_age`.

## Known limits and next
- The galaxy is the median Milky-Way-like stand-in, and a system with no galactic place sits at the Sun's radius, until G7. Both are in the fudge register.
- The references were written from memory and are flagged for the citations thread.
- Re-deriving planets after `follow`, as a pinned formation re-run with the changes listed, is the next step for the dialog.
- J6b (programs over a time jump, track J 6.2) builds on this clock: only moving the campaign's now resolves programs.
- The UI's `shiftCampaign` (moving the clock to another system's present) and the look-ahead dialog should move to the two named choices. That is a chart-room-thread change.

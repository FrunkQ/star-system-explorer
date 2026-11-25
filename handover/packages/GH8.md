# GH8: the leading arm, named in the galaxy chart's history (24 Nov 2026)

Owner: "Build galaxy charts and histories". Design: `SSE4-galaxy-histories-design.md` 9.4, "Which way arms wind"
(Alex, 24 Nov 2026): arms trail as a result of the disc's differential rotation; a leading arm is only ever the
dated mark of a pass or merger on a backwards orbit, fading as the rotation winds it back; rarity follows from the
drawn history, with the yardstick (at most about 1 in 100 spirals, Iye, Tadaki and Fukumoto 2019) as a pack
parameter checked by GH3.

## What was built (chart side, ahead of the kernel)
- `ui-contract/src/galaxy.ts` `leadingArmWords(marks, t_gyr, strengthAt)`: the latest `leading_arm` mark made by the
  chart's time as a sentence ("A leading arm, from a backwards pass 22 kpc out 300 Myr ago: it still winds against
  the spin."), how many were made, whether it still shows (the renderer's own fade, `markStrengthAt`, passed in).
  A mark from a meal reaching the centre (no `r_kpc`) reads "a backwards merger". Test `ui-contract/test/galaxy.test.ts`.
- Galaxy chart, "Built from other galaxies": the line above the mark list (`data-leading-arm` = showing or wound
  back), the rarity sentence with its citation, and "Leading arm (wound against the spin)" in the mark list.
- Citation row for Iye, Tadaki and Fukumoto 2019 in `docs/citations/galaxy-histories.md`.

## Depends on
The kernel's GH3 `leading_arm` mark (thread "Build galaxy arm fix and atmosphere advice"; kind `leading_arm`, shape
{r_kpc, prograde, speed_kms} from a pass or {prograde} from a meal, fades_gyr 0.9). It changes goldens, so it waits
for its slot in the golden lane. Until it merges no chart carries the mark and the line stays hidden. The kernel
thread notes most drawn leading arms are early and long wound back, so most charts will name one as past history.

## Next
When GH3's mark merges: an e2e that finds a halo with one (or inks a backwards merger once G6 edits can say so), and
the yardstick on the wire if the kernel sends `seen_share_max`.

## Part 2: with the kernel's mark (25 Nov 2026, after PR 463)
- The kernel makes a `leading_arm` mark from every backwards pass or meal, mostly weak. The chart's line now counts
  only arms strong enough to be seen: `LEADING_ARM_SEEN` = 0.25, kept in step with the kernel's
  `LeadingArmParams.seen_strength` (not on the wire). Weaker ones stay in the history list with the other marks.
- The line shows while the latest seen arm still winds against the spin, then "wound back" for one fading time, then
  goes; the history list keeps it.
- `data-leading-seen` on the chart lists when seen arms were made; `tests/ui/galaxy.e2e.mjs` scrubs to just after
  the latest and checks the line. In the default campaign the Milky-Way-like galaxy's latest is at 8.34 Gyr; at now
  no arm shows. Screen `GX-screens/gh8-leading-arm.png`.

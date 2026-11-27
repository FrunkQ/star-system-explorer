# TM9b: the phone's Run frame (track TM)

Status: done 27 Nov 2026, thread "Build the three desks". Design: `handover/design/SSE4-three-modes-design.md` section 8 (accepted 25 Nov 2026), on top of the phone frame of `SSE4-top-level-ui-and-onboarding.md` section 4.

## What it does (phone width, 760 px and under, Run desk only)

- **The stepper is a row of its own** under the bar. Every step is drawn in full and the row scrolls sideways. **Out** and **In** are thumb-sized (44 px) and held at the row's two ends while the steps scroll between them. A disabled one is drawn muted, not see-through.
- **The rail folds to its one instrument button** ("Chart ▾"), which heads the view controls on its own line where the row of chips was. The list opens as words in a column, as wide as the phone allows. Explore and Build keep the row of chips.
- **The clock is one row at the foot**: the date, one step chip (+ 1 hour) and "…". The other steps, Run and its rate, what is due and Jump to a date… come in under it on "…", and go again on ×.
- **The card is a sheet that comes up only on a press.** Nothing selected means no card. A body picked brings the card up from the foot (up to 60% of the height, scrolling inside), with **Close**. Esc also closes it, since Esc already clears the selection.
- The Run page fits a phone's screen without scrolling (390 × 664 checked).
- A phone on Run starts on Chart, as phones already do.

## Where

- `packages/shell/src/App.svelte`: `.app[data-desk]`, and the phone rules that put the stepper on its own row.
- `nav/PlaceLine.svelte`: the phone stepper row. `nav/Rail.svelte` and `nav/RailMenu.svelte`: the fold on a phone.
- `chart/Stage.svelte`: the view controls' column on a phone, and the card sheet with Close, asked of the new shell part `card-sheet` (desks: Run).
- `chart-grammar/src/CampaignClock.svelte`: the one-row clock with "…" (`[data-clock-more]`).

## Left for later

- Sound sits on its own short row under the clock on a phone; it could fold into "…" too.
- Tours and the start sheet's lines checked against the new Explore (TM9's last part).

## Checks

- `tests/ui/stepper.e2e.mjs` gains a phone section: Explore keeps the chips; on Run the stepper is a row under the bar with Out and In at its ends and at least 44 px; no chips, the one instrument button; the clock shows one step and "…"; no card until Tarn is pressed, then a sheet at the foot; Close puts it away; nothing overflows sideways. Seen red with the shell changes removed ("the stepper is a row of its own under the bar"), green with them.
- `tests/ui/time.e2e.mjs`: on a phone the clock shows one step and "…", Jump waits under "…", and the rest of its checks run after "…" is opened (follows design section 8).
- Screens: project files `sse4/handover-screens/TM9b-screens/`.

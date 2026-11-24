# AT15a: screen delivery, how a screen arrives and leaves

Status: part 1 done 24 Nov 2026 (thread "Build look transitions for screens", the navigation thread renamed). Design: atlas design `SSE4-atlas-and-reports-design.md` section 11A.4a; Playbook design 5.2 and PB10 ("deliveries on arrival and hold"). This is that one mechanism, filled in, not a second one.

## Alex's words (24 Nov)
- 20:08: "Dont forget to reuse some of the stuff we had around transitions... greenscreens building up rather than just flicking - the slow warm up and the disappear to a dot."
- 20:15: paper reports' pages flick ("replicate the paper experience").
- 20:15: "the guide is NOT paper - it is cheesy electronics built to a low cost" (so the Guide redraws in bands, a stepped wipe; Page turn is for paper editions only).

## What is built
- **Data and rules** `ui-contract/src/delivery.ts`: five moments (`power_on`, `arrive`, `turn`, `leave`, `power_off`); 14 deliveries with words, defaults and the scales each can act at (`DELIVERIES`); `resolveDelivery(look, moment, {first, reducedMotion, back})` gives the plan to play: full form for Power on and a send's first Arrive, the short form after, Turn always short and at most 400 ms, reduced motion a 150 ms fade, wrong-way choices fall back; `checkLookDelivery` says in words what is wrong (for the loader and Orac); `STARTER_DELIVERIES` for Hauler, Corporate, Deck, paper and the Guide.
- **Player** `chart-grammar/src/deliver.ts`: `playDelivery(node, plan)` (Web Animations; a tap or key anywhere finishes it; `data-delivering="<kind>"` while it plays), the Svelte action `use:deliver={plan}` (plays on mount and on each new plan) and the transition `out:deliverOut|global={plan}` (a leaving picture is held where it stood, above its replacement). Three scales: `screen`; `parts` (`[data-part]` children, else direct children, one after another); `inside` (`[data-deliver-item]`, else blocks of text, rows and figures; Type on uses `[data-deliver-text]`).
- **Looks** carry `delivery` (ThemeDoc in `skins/chart-room/src/themes.ts`): Phosphor = Hauler (Warm up then Build up inside, Build up, Type on, none, Collapse to a dot); Clean glass = Corporate; Deck panels = Deck; Paper = paper (Page turn); Field guide = the Guide (stepped wipe). Chart has none and shows the picture as before.
- **Players' screen** `shell/src/Players.svelte` (agreed with the Run desk thread): Power on when the picture first shows or a hold is released, Arrive for each new send, Power off as the GM holds the screen.

- **Turning by hand** `chart-grammar/src/page-turn.ts`: `use:pageTurn={{ onstep, sheet }}` on a page. A horizontal drag carries the sheet about its spine (left edge forward, right edge back) with a growing shadow; past a third, or flicked, it turns and calls `onstep(±1)`; short, it settles flat. Links, buttons and fields keep their presses; vertical drags still scroll; reduced motion steps without the sheet. Pure maths in `turnDrag` and `turnCommits`.

## Checks
`packages/ui-contract/test/delivery.test.ts`, `packages/chart-grammar/test/deliver.test.ts`, `tests/ui/page-turn.e2e.mjs` (a bare page, no build: drag forward turns about the left edge, short settles, back steps back, a button keeps its press), `tests/ui/delivery.e2e.mjs` (Hauler warms up, builds up; a key finishes; reduced motion fades; Chart plays nothing; held from the Switchboard it collapses to a dot, released it warms up). Screenshots: project files `sse4/handover-screens/AT15a-screens/`.

## Next (AT15a part 2)
- **Turn** on the Atlas page: the atlas thread calls `playDelivery(pageNode, resolveDelivery(look.delivery, "turn", { back }))` on each step (they asked to wire it themselves; the page is `article[data-atlas-page]`, blocks `data-deliver-item`, prose `data-deliver-text`). For paper editions add `use:pageTurn={{ sheet: true, onstep }}` on the same article so a swipe turns the page too.
- **Leave** is resolved but not wired: today a new send arrives over the old picture.
- The look card's "Screen delivery" row (PB10, Run desk thread) lists `DELIVERIES` and writes `delivery` on the look; Orac's look target should accept `delivery` and run `checkLookDelivery`.
- Parts declaring which scales they can do (11A.4 rule) is implicit today: a part that marks nothing gets its blocks animated, or a fade.

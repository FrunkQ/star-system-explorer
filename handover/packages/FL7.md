# FL7: warming up, the pure part

Status: built 28 Nov 2026, thread "Build the shared flourish system" (Opus). Design: `design/SSE4-visual-flourish-design.md`
section 6A (Alex's warm-up, 28 Nov 13:23 and 13:24 UTC; questions 8 to 10 "yes if practical") and 8A. Pure, in
`ui-contract/src/settle.ts`, tests only. The journey's use (the way down drawing itself in, Go turning to the accent,
the Run desk warming the next Show step) lands with FL5 and the Run desk; nothing on screen changes here.

## What it adds

- **The queue** (`WarmItem`, `warmOrder`): pictures known to be coming, each with its key, kind, seconds until it is
  needed, whether it is on screen, and its distance in levels. Built on screen first, then by when needed, then
  nearest first.
- **The hold-backs** (`warmPlan`): on a hot device, under data saver or past the memory limit, nothing ahead is warmed
  unless the viewer asked for the journey, and then only the next thing needed. What is on screen is never held back.
- **The time told** (`BuildTimes`, `readyLine`): each kind's build time remembered for this visit, smoothed; the first
  time the design's estimate, said with "about". "Getting the way down ready: about 20 seconds, 3 of 7 levels."
  Kept in memory, not in the device's storage: the speed probe keeps its timings, but these are cheap to learn again
  and the design keeps everything warm-up in memory (question 10).
- **The cache** (`WarmCache`): proper pictures in memory with their size, warm or on screen, and level. Under
  pressure (`shed`) the warm ones go first, whole levels at once, furthest from the viewer first (a picture that is not
  a level counts as furthest); what is on screen goes only when nothing warm is left.
- **What a level shows** (`levelPicture`): proper if kept, coarse if its coarse picture is ready, else hold, for FL5.

## Tests

`ui-contract/test/warm-up.test.ts` (6): order with the screen first (absolute); the hold-backs; the time told within
a factor of two on the second run (37 s then 33 s); the shedding order; all levels warmed never shows a stand-in;
nothing touches `localStorage`, `sessionStorage` or `indexedDB` (each trapped, any read or write fails the test).
Red first: the screen-first comparison removed, the hold-backs removed, the estimate never learning, warm and on
screen treated alike (red only once a test kept an on-screen picture that is not a level: added), and a write to
storage in `record`. Each went red.

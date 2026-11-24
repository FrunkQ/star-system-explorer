---
name: sse4-v3-gaps
description: The V3-to-SSE4 user-facing gap list (34 items, 23 Nov 2026): which are going ahead and which six questions wait on Alex
metadata:
  type: project
  modified: 2026-11-23T02:43:08.573Z
---
`/mnt/project-files/sse4/SSE4-v3-gaps.md` (repo `handover/design/`, PR 217) lists 34 V3.1 features that SSE4 lacks in the app, each with keep, drop or do-better and a coming-together stage. It complements docs/v3-parity.md (mechanism verdicts, K0).

Alex, 23 Nov 02:41: "the obvious ones you can push on and develop". Going ahead through the owning threads: 1-9, 11-14, 16, 18-23, 25-29, 31 (the coordinator was asked to route them).
All settled by Alex 23 Nov 02:51 (PR 218): class gallery on the classification pages with icon, still and animation rendered from the physics (item 35, replaces V3 type pictures); megastructures are constructs carrying physics blocks (autopilot, thrust blocks for stellar engines; design note first); Your systems map in stage 2; reports are purpose-built screens filtered by lens, not print-any-screen; colour-blind check yes; drops yes. Gas giant "rainbow" variety must come along, from cloud chemistry via ND2 (item 36), with V3's recipes as the target palette.

**How to apply:** when Alex answers, update the file's status line. See [[sse4-coming-together]].

Second audit 24 Nov 2026 (PR 393, appended to the same file): of items 1-36, 13 done, 7 in hand, 16 planned not started. New items 37-42 on finding your way (picker on the place line replacing Below and doubling as Find; press-and-hold menu on any object; system hover; zone lines with key; find by tag; campaign/system notes). Three questions to Alex: picker shape, context menu, navigation first in stage 2.

Navigation (24 Nov): items 37, 38, 41, 42 accepted by Alex ("yes to all"). Thread "Build object selector and menus" owns track NV (handover/status/navigation.md). Chart room built the selector's first cut (picker.ts, chart-grammar Picker.svelte, PR 399) then hands the place line to NV. NV1 object menu = PR 402: one action list ui-contract actions.ts; any element with data-address (or an sse:object-menu event from canvases) gets the menu. Commits in this env can't override the author to FrunkQ (blocked); squash merges carry FrunkQ.

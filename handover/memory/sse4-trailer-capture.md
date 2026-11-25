---
name: sse4-trailer-capture
description: SSE4 trailer footage: capture rig in repo scripts/trailer, clips and shot list in /mnt/project-files/sse4/trailer, daily re-shoot task
metadata:
  type: project
  modified: 2026-11-21T14:44:42.354Z
---
- Thread "Capture footage for SSE4 trailers" (started 21 Nov 2026, Alex's long-term ask): film each key feature for an intro trailer and a features trailer.
- Rig: repo `scripts/trailer/` (capture.mjs, director.mjs, shots/NN-*.mjs; PR 62). Films a local build of main (cloud sandbox cannot reach v4.starsystemx.com) via Chromium screencast + ffmpeg. Shots reuse tests/ui/nav.mjs.
- Output: /mnt/project-files/sse4/trailer/ (clips/, SHOTLIST.md with statuses captured/weak/needs GPU/not built, STORYBOARDS.md, rough cuts).
- No GPU in the cloud: WebGPU globe and System 3D film blank; those need `--gpu` on Alex's PC (AC-ProArt), open question.
- Daily scheduled re-shoot 21:23 UTC into the thread's session (trigger trig_01XZ4JxRi3Ey8yRdUCpptLLD).
- Videos wanted (Alex 21 Nov): intro trailer, features trailer, scale journey (filaments to a cave, hold until descent/surface/caves land), physics system demo (cut first), RPG short series. Storyboards A to E in STORYBOARDS.md.
- GPU takes filmed on AC-ProArt via Remote Control with --gpu --headed --channel chrome (Playwright's Chromium won't start there); thread folder tools can't stage files, so Alex uploads takes to the thread.
- Alex 21 Nov: GATHER clips only as features finish; no trailer cuts until more feature complete. Clips carry caption cards (d.caption) + .srt. Never call the V3 sample planet "Earth" (not modelled). Globe shot on hold until clouds, air, ice caps, cutaway.
- Videos F (system generation explainer, Alex 22 Nov) and G (Orac teaches new physics, from the ND demo thread; film near the end with a real key on AC-ProArt, real cost shown) are in STORYBOARDS.md; keep other threads' appended sections when rewriting it. Shots without `path` open "#chart" (start sheet is `path: ""`, PR 138). Earth through time (PR 140) waits on the chart room timeline, then GPU take.
- 22 Nov: shots 00 start sheet, 22 formation replay, 23 Reference elements, 24 Mag on catalogued Earth (hide navigator.gpu in the cloud). Scales above a system are place line steps (.place [data-place-step]), not Explore tabs. The catalogued Solar System's Earth (SOL1) has measured orbit and field but a generated surface: caption it so, and don't film its globe as Earth.
- 23 Nov: Scale (Real/Readable) and Band (incl. Mag) are in the one view controls. Land screen (SL1) held: works only on V3 sample / catalogued Earth with invented maps; Alex chose 23 Nov: wait for a grown world or a modelled Earth, then a GPU take. True views (V10a-e, V9t, GH9) await a GPU take.
- 23 Nov: Alex wanted a Discord preview: scripts/trailer/cut-preview.mjs cuts clean takes (--no-captions) with .srt captions burned as a dark strip (in-page cards hide under dialogs), <10 MB. Best run on AC-ProArt (C:\Development\sse4, branch trailer-gpu; playwright via NODE_PATH=C:\Development\dynamic-map-renderer-v2\node_modules). Alex wants True, HUD on, Mag lines shown; Wireframe only once on the charts.
- Never film or fake unbuilt features. Related: [[sse4-build-conventions]]
- 25 Nov: beauty shots H1-H4 (STORYBOARDS section H). Alex left AC-ProArt on for reshoots (01:28). Shots 25/27 provisional until giants get colour (climate PR 418, giant bands TA3a, renderer PR 488). Moon H1 waits on PR 467 deploying plus the renderer's close-up package. Local build can't run on the PC (wasm-bindgen Linux-only), so film against v4 live.
- 25 Nov 02:07: ON HOLD by Alex until all colours land and the renderer is better; daily task disabled; restart only on Alex's or the coordinator's word. State in repo handover/status/trailer.md.

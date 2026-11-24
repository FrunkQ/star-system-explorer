---
name: sse4-build-conventions
description: SSE4 build state and conventions for coding threads (repo, branch, checks, handover files)
metadata:
  type: project
---
- Repo https://github.com/FrunkQ/star-system-explorer, `main` created 19 Nov 00:20 UTC from the integration branch after merging PRs 1 to 3 (Alex approved). New work branches from main and arrives as PRs into main. Alex still to set main as GitHub default branch and Cloudflare production branch (asked 19 Nov).
- Resume state lives in the repo: `handover/START-HERE.md`, package logs `handover/packages/*.md`, rulings `handover/decisions.md`. Design copies in `handover/design/`, refreshed from /mnt/project-files/sse4/ by `handover/sync-design.sh` at each package boundary.
- M1 (K0 to K6) done 18 Nov; K0 docs await Alex's review.
- Alex (18 Nov 16:30): push hard through milestones, finish each cleanly; he comes back to start the next and to get V4 online.
- Commits as FrunkQ <frunk@frunk.net>; `scripts/check.sh` before commit. Never push to other branches without Alex's word.
- Kernel track: K7-K10, G0-G5, C1-C3, L0-L1d, T1/T2, ops through the API (see START-HERE).
- Physics track: P1, P3, P4, F0-F9, S0-S3; any orbit integrator written fresh, never from REBOUND (GPL).
- AC-ProArt (Alex's PC) builds and tests C:\Development\sse4; real V3 save copies in C:\Development\sse4-real-saves (results in handover/packages/P3.md).
- References on atoms are `{ text, doi?, url? }` (sse_flow::Reference), the shape the UI's "Why?" panel reads; every atom needs at least one linked reference (test). Heat-producing elements are atoms `sse.heat.*` with flows planet-heat and planetesimal-water (handover/packages/P1-heat.md).
- Live site (18 Nov, 23:00): https://v4.starsystemx.com, Cloudflare Worker `star-system-explorer` via Workers Builds, set up by Alex. Build `npm run build` → dist/, deploy `npx wrangler deploy`, production branch `claude/project-thread-h9y77o` until main exists. Pushing to that branch redeploys the live site. First build worked; Alex saw the harness PASS there (Chrome 157, 110 of 110 worker responses identical to native).
- 19 Nov 11:02: Alex said main is ours (merge PRs when green). PR 6 merged to main 0f1cf24, containing PRs 4 and 5 (all marked merged). Cloudflare production branch still the old physics branch: Alex to switch it to main (first prod deploy applies the SignalRoom DO migration). Sandbox cannot reach v4.starsystemx.com.
- UI track (19 Nov 04:05): on `claude/project-thread-adgixl` (PR #6, which now also carries PR 4 and PR 5 via PR 5 head 1cfdeb4): shell, views, players, p2p, seats (now via kernel grant op), lenses U9, sound N1, U4 what-if and look-ahead, O1 light switch (stubbed spectra), V13 part 1 make a system (V3 import, formation run, previews), A1 part 1 agents, V6 Cutaway on a Body screen (Tarn's stack from a fixture; ladder ported line for line from sse-world), V13 part 2 (adopt into the campaign, read back via resolved_system, look-ahead write-back on adopted systems). Cutaway now reads read_world_stack; adopted values use resolved_system evidence; list from systems op. Open: browser does not persist the chronicle across reloads, fixture Kestrel's kept look-ahead still does not rewrite orbits, p2p cannot carry kernel commits, A1 part 2 needs Ovid source.
- Routing (coordinator, 19 Nov 03:25): kernel API asks go straight to the owning thread ("Build kernel resolve and rules" or "Build SSE4 proven core") via message_thread; tell the coordinator only when something lands Alex should hear about, or when blocked.

---
name: explorers-site-repo
description: Explorers site (FrunkQ/starsystemx-creator-hub) facts and rules for the thread integrating it with SSE4
metadata:
  type: project
  modified: 2026-11-21T13:35:25.566Z
---
- Repo FrunkQ/starsystemx-creator-hub, live https://explorers.starsystemx.com. SvelteKit 2 + Svelte 5 on Cloudflare Workers (static assets), Supabase Postgres, private R2 (sshub-assets, sshub-bundles). v0.62.0 on 21 Nov 2026. Docs in docs/ (contract-with-sse.md, decisions.md D-numbers, sse-requirements.md R-numbers). Converter is FrunkQ/starsystem-convert (convert.starsystemx.com), own thread "Make converter a shared SSE4 module".
- Workers Builds deploys `main`: merging a PR ships to the live site. Migrations (db/migrations) are run by hand in Supabase.
- Merge rule (Alex, 21 Nov 16:07, replacing the earlier PRs-only rule): Explorers PRs merge themselves when checks pass (merge = live deploy); verify the live site after each. Anything with a DB migration, storage change, or effect on users' stored maps or accounts still needs Alex's own yes first.
- Related: [[sse4-build-conventions]]
- Skin PR #1 (0.63.0, D-93). Local preview: .dev.vars to a mock PostgREST; own node_modules/.vite per worktree. Live checks via built-in browser on Alex's machine (curl to explorers blocked by proxy).
- sseClip 2 (SSE4 PR 133) mirrored as C-08 in docs/contract-with-sse.md. Site still makes sseClip 1 only.
- Workers Builds gotcha (converter thread, 21 Nov): a production build started while a PR preview build is still running fails instantly; let the preview finish before merging.
- Pack shelves (Alex, 21 Nov): featured = original/generic; named-setting packs on a labelled fan-made unofficial shelf (plain names, no logos/art/copied text), reusing attestation + fan-work mechanism (D-61). Spec: SSE4-tags-design.md 5.7 (TG15).
- Paid packs (Alex, 21 Nov 16:43): build the full charging mechanism behind one admin master switch, off by default, invisible while off; free is always free; provider test mode. Design + packages in /mnt/project-files/sse4/SSE4-explorers-paid-packs.md by thread "Design paid packs on Explorers"; start when packages land. Migrations still need Alex's yes; payment keys from Alex only, never committed or logged.
- Flood (21 Nov): Workers free 100K/day is per ACCOUNT. PR #3 (0.64.0, D-94): scraper 403, PUBLIC_LIMIT 300/min/IP, edge-cached public GETs, robots AI block. Dashboard steps in docs/flood-protection.md, Alex's to do.
- Paid packs: PP2+PP3 (0.66.0/D-96, D-97, Stripe test mode only, PR #4) merged 21 Nov; Alex ran 0042/0043. Next for Alex: Stripe test account with Connect + Tax, keys into Worker secrets, flip switch. PP4 (seller onboarding) next; PP8/live keys need his explicit yes.
- Track ND (new-physics demo film, accepted 22 Nov; design /mnt/project-files/sse4/SSE4-new-physics-demo.md s3-4): ND8 = publish a V4 physics pack to Explorers, fetch on another device with no agent key, hash and state hash match. Pack listing should show from the manifest its sources + fudge list with reproduction report (ND6) and cost to make (ND7). Depends on this thread's X2 and TG15.
- V4 shelves (23 Nov): PR #8, 0.67.0, D-98, merged 23 Nov 02:56 with Alex's yes; migration 0044 (v4_items, v4_reports) for him to run. /shelves, /v/<slug>, /upload/v4, /api/v4, /api/v4/<slug>, /api/v4/<slug>/file (exact bytes, x-sse-sha256); contract C-09. Packs stored byte for byte at R2 bundles v4/<sha256>.json; campaign record/undo stripped. Not done: pack preview on a stock system, SSE4 side of ?explorers=<listing url> (agreed; button built in PR #8), V4 on Browse/front page, V4 reports in staff badge.
- Contains filter (23 Nov, example packs Q6): 0.68.0/D-99, PR #10 merged, no migration. Counts per content slug stored in existing v4_items.provides; /shelves "Contains" chips; /api/v4?contains=<slug>. CONTAINS table in src/lib/v4/read.ts. Rows before 0.68 used word keys (none existed).

# GZ0 browser timings (grand zoom, the travel thread's half), 26 Nov 2026

Thread "Build travel between places" (Opus). The kernel half is `GZ0-kernel-checks.md`. Design: section 10 of
`design/SSE4-grand-zoom-design.md`. Checks only: nothing changed, no test or golden reads this.

## How it was measured

`node scripts/gz0-timings.mjs --runs 2` on the cloud container (Intel Xeon 2.1 GHz, 4 cores), on main at d726d46 with
the release wasm. Three runners: Node, headless Chromium 158, and Chromium with its CPU slowed four times (the phone
stand-in the bench uses). Each read runs on a fresh kernel with the four base packs and a new universe ("gz0
timings"), cold, then again warm on the same kernel. Medians of two. Raw figures: `GZ0-timings-2026-11-26.json`.

The kernel runs on the page's own thread here so the slowdown reaches it. These are kernel reads only: no GPU is
involved, so Alex's GPU machine is not needed for them, but a faster desktop CPU will be quicker than this container.

## Results (cold)

| Read | Rung | Node | Chromium | Phone stand-in (4×) | Reply |
|---|---|---|---|---|---|
| Web cube, 48³ over 100 Mpc | web | 2.8 s | 2.2 s | 9.6 s | 11 MB |
| Web cube with galaxies, 48³ over 100 Mpc | web | 2.9 s | 2.7 s | 12.9 s | 2.5 MB |
| Web cube with galaxies, 48³ over 200 Mpc (the view's default) | web | 6.9 s | 6.3 s | 28.2 s | 3.3 MB |
| Web cube with galaxies, 32³ over 100 Mpc (the basic tier's plan) | web | 3.0 s | 2.6 s | 12.3 s | 2.5 MB |
| One halo cell, 8 Mpc (Galaxies nearby today) | cluster | 0.25 s | 0.24 s | 1.1 s | 15 kB |
| **The neighbourhood's galaxies and home's satellites** (`sky_from_here`, galaxies only) | cluster | **8.7 s** | **8.7 s** | **37.5 s** | 1.9 MB |
| The home galaxy's chart | galaxy | 0.25 s | 0.27 s | 1.0 s | 470 kB |
| One neighbour's full chart (biggest other halo in the home cell) | cluster | 0.31 s | 0.40 s | 1.5 s | 400 kB |
| The home star cell, 10 pc | stars | 0.23 s | 0.30 s | 0.85 s | 44 kB |

Setting up a kernel (packs and universe) is about 30 ms, 130 ms slowed.

## What it means for the journey

1. **The neighbourhood read is the problem: 8.7 s on desktop and 37 s on the phone stand-in, about nine times the
   1 s budget.** Design section 10 planned to read it during the web rung. Without stops, a stately grand zoom
   reaches the Galaxies nearby seam about 2 s after it starts (journeyPlan, GZ1), so the read cannot be hidden there
   on any device; with a stop at the web it is hidden only if the viewer lingers. It
   should be worked out once and kept (the gazetteer, GZ17, is the natural home), or read before the journey starts.
   Worth a look by the kernel thread: most of it is likely the dozens of cell catalogues behind `galaxies_seen`.
2. **The basic tier's 32³ cube saves nothing** (2.6 s against 2.7 s with galaxies): the halo catalogue costs more
   than the grid. A lighter journey on a phone needs a higher galaxy floor or a smaller extent, not a coarser grid.
   GZ11 should plan on that.
3. **Cold and warm are the same.** The kernel keeps nothing between calls, so a second journey pays again unless the
   client keeps the replies (design 10 already says the cube may be kept, as it is deterministic).
4. **The web cube at the view's default 200 Mpc takes 6 to 7 s** on desktop, inside the "under ten seconds before a
   cold grand zoom starts" of section 10, but 28 s on the phone stand-in.
5. Galaxy charts, a neighbour's full chart, the halo cell and the star cell are all a quarter to two fifths of a
   second on desktop and about 1 to 1.5 s slowed: comfortable to read ahead during the rung above.

## Not measured here

- A real phone. Only the stand-in. Someone with a phone should open the bench page (`/harness/?bench`) once these
  reads are in the bench, or run this script on a laptop with a phone attached.
- The system scene, the world's faces and tiles, and the site's ground: already in the bench's chunks, and inside a
  system the design's budgets are already met (`docs/minimum-spec.md`).
- The orientation chain against the sky (design 13, GZ0): not checked by this thread. It fits best with GZ14 (tilt
  from the web's tides), which makes the sky and Galaxies nearby read one axis.
- Drawing costs at a seam (two views alive): GZ3 and GZ11, with the drawing bench.

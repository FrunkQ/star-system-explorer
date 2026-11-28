# WS5: the stirred face

Design: `handover/design/SSE4-stirred-clouds-design.md`, sections 3.6, 6 and 11A. Needs WS3, WS3b and WS4. Status row: `handover/status/true-air.md`.

## What it costs (measured 28 Nov 2026, before building)

The design guessed about 40 backward steps a pixel, a few milliseconds a face. Measured on the kernel's own carrying (release build, fixtures, one cell):

| World | τ | Steps a cell (with halvings) | Wind sums | Eddies in reach a sum | Time a cell |
|---|---|---|---|---|---|
| Jupiter | 32 days | about 9,900 | 118,000 | about 1 | 430 ms |
| Neptune | 13 days | about 1,000 | 12,500 | about 3 | 90 ms |
| Saturn | 336 days | | | | 3.4 s |

- Most of the kernel's cost is the reference's own care: three paths a step for the Jacobian, and halving in strong strain.
- The picture needs neither. The stretch for reading coarser levels comes from neighbouring pixels' paths.
- Without them, the estimate is about 2 to 8 million operations a pixel on Jupiter. That is about 0.1 s for a 512-wide face on a desktop GPU and 1 to 2 s on a phone (unverified). Saturn takes about ten times that.
- Re-stirring every frame on the H tier, as the design has it, would cost about a tenth of a second a frame. Alex was asked on 28 Nov whether every device should build a still instead, rebuilt as time moves (recommended).

## WS5a: the carry table and the renderer's carrying

- **Kernel:** weather want `carry` (`stirred.rs` `CarryTable`). It holds everything the `Carrier` sums, over the two renewals and a day that a picture reaches back:
  - the bands' Ψ(ρ) tables and the eastward wind every 0.01°;
  - the poles' polygons;
  - every storm with its drift;
  - every eddy;
  - each band's generation offset;
  - the steps.
- **Fix:** the span's last instant now reads the last bin's eddies. They were dropped there, at the first stage of every step from the picture's own date.
- **Renderer:** `packages/renderer/src/stirred-carry.ts` sums the same winds and carries the same way. It has two modes:
  - the kernel's way (Jacobian, halving), used for the test;
  - fixed steps, the picture's way, which the GPU pass will run.
- **Held by test:**
  - `ws5_the_renderers_reference_is_current` keeps `tests/fixtures/weather/carry.json` current. It holds Neptune's whole table with a 4×4 stirred patch, and Saturn's table round day 1000 with the kernel's winds at 80 places, round its polygon and storms.
  - `test/stirred-carry.test.ts` checks four things:
    - winds within 1e-3 m/s (worst 6e-4);
    - the generations;
    - the kernel's way of carrying within a kilometre;
    - fixed steps within a kilometre per 64 times the stretch, far inside a 1024-wide face's pixel.
  - The wind check goes red with the storms' shield removed (seen).
- **Kernel test:** `a_giants_carry_table_holds_what_moves_its_cloud`.
- **Fudge row:** "The renderer's carrying of a giant's cloud".

## WS5c: the carrying on the GPU

- **Renderer:** `packages/renderer/src/stirred-gpu.ts`.
  - `packCarry` lays the carry table out as float textures. Eddies are filed by half day and on a 5° grid, each listed in every cell its reach touches.
  - `carryOnGpu` runs `sourcePlaces` (stirred-carry.ts, its processor twin) for every pixel at once. It is one three.js TSL source for WebGPU and WebGL2.
  - Each draw takes one fixed RK4 step. The state lives between draws in float targets, and every four draws one texel is read back, so the device never holds a long queue.
- **What the test machine taught** (all three.js r187 on SwiftShader, the software device in the cloud container):
  - A texture read inside a shader function of its own (`setLayout`) is named after another uniform on WebGL, and the shader fails to compile. Functions that read the table are written inline.
  - The device's own sin, cos, asin and atan are off by up to 1e-3. Carried over a thousand steps, that moved cloud by hundreds of kilometres. The carrying writes sine, cosine and arc tangent out as series. A place's frame and its neighbours a step away come from its own vector, with no angles.
  - A loop round the wind made the compiler take 13 GB. So there is one step a draw, and no loop wraps the wind.
  - A target reads back from the bottom row up on WebGL; `carryOnGpu` turns it over.
- **Held by test:** `tests/ui/stirred-gpu.e2e.mjs` carries an 8×8 patch on Neptune on the kernel's own table (`tests/fixtures/weather/carry.json`, 1,334 steps a pixel).
  - Younger generation: within 0.82 km of the processor (limit 2 km).
  - Older generation: within 27 km (limit a quarter of a 1024-wide face's pixel, 38 km). Its read place is its carried place drifted along the row for 21 days, so a float's 0.001° in latitude moves it by the jet's shear times the age.
  - Goes red with the eddies' winds weakened by a tenth (seen: 7,600 km off).
  - About 290 s on SwiftShader, most of it building the shader.
  - SwiftShader's WebGPU loses its device on this shader, so the WebGPU half is skipped here. On a real GPU it is unchecked.
  - `SSE_PROBE=probe` prints the places, winds and one step against the processor.
- **Next:** the stirred still. Compose today's face from the source map with the two generations' weights, wire it into the true face, keep it in memory only, and rebuild it as time moves.

## WS5d: the stirred still on the face

- **Composing** (`packages/renderer/src/stirred-still.ts`):
  - Each pixel takes the unstirred face's colour where each of its two generations was read, weighed as the kernel's `generations` weighs them.
  - Where the backward map stretches cloud (the larger spread of the read places of the pixels either side), the colour is read from a coarser level of the face, one level per doubling. Cloud stirred finer than a pixel comes out as its local mean.
  - The disc's mean colour is the face's own.
- **Source:** `detailGiant(..., { source: true })` is today's face with no winding and no curls. The carrying does both.
- **Built** (`stirred-runner.ts`):
  - It runs on a GPU of its own, off screen and made on first use, one still at a time, its draws in batches with a frame between (see the second round below).
  - Finished stills are kept in memory only, by world, date and size, up to 32.
- **Wired:** `readTrueFace` returns `stirred` for a giant: the kernel's `carry` table for the date, then the still. It does this only where a page can make a GPU, never in node. The system chart shows the unstirred face, then swaps in the still when it comes.
- **Held by test:** `test/stirred-still.test.ts`.
  - With no wind, the still is the face.
  - A pixel's shift reads a pixel west, with both generations weighed as the kernel weighs them.
  - A checkerboard stretched eightfold comes out as its mean. This goes red with the levels left out (seen).
- **Still to do:**
  - Rebuild the still as the date moves. Today it is built once per face read, at the date it is read.
  - The other views: body globe, size comparison, class pictures.
  - Retire PR 646's winding and curls wherever the still shows.
  - Screens on a real GPU (AC-ProArt).
  - The cost there, which is unmeasured.

### WS5d on a real GPU (AC-ProArt, 28 Nov), and what it changed

- **Twin test** with Chrome headed, `--force_high_performance_gpu`. Unless forced, Chrome picks the Radeon 890M.
  - RTX 4070: WebGL2 0.78 km and 24.3 km, in 36.8 s; WebGPU 0.97 km and 30.1 km, in 13.4 s.
  - Radeon 890M: WebGL2 0.71 km and 22.2 km, in 29.7 s; WebGPU 0.52 km and 17.4 km, in 8.0 s.
- **The product failed.**
  - Jupiter's eddy list needed a texture 12,923 tall, and Saturn's 29,123; a GPU allows 8,192. Every draw after that was invalid, the places came back unmoved, and the runner called the still built.
  - Building a still froze the tab for 1 to 7 s at a time. Jupiter took 40 s and Saturn 123 s on both GPUs, so the time went on the processor.
- **Fixes:**
  - The eddies are filed in each row's frame turning with that row's wind, in 2-day bins. In that frame an eddy sweeps only its drift against the row. Jupiter's list now needs 1,138 rows and Saturn's 4,373. `test/stirred-pack.test.ts` checks every eddy that reaches 4,000 random places and dates is in the list the shader reads, and goes red with the drift left out (seen).
  - `carryOnGpu` checks every texture against the device's limit, and refuses the result on any WebGPU validation or out-of-memory error.
  - Each material's shader is built with `compileAsync`.
  - The product took 4 steps a draw, with a frame between draws (undone in the second round, below).
  - Filing the table and composing the still run in a worker (`stirred-worker.ts`). The table filing is `stirred-pack.ts`, free of three.js.
- **How far cloud moves, at a 1024-wide face** (processor twin, median per row):
  - Neptune: 0 to 4 px north-south; along the bands, against the row's drift, 1 to 140 px.
  - Jupiter: 1 to 6 px north-south in the belts and 10 to 25 px towards the poles; along the bands, 4 to 250 px.
  - The unstirred face changes little along a band, so stirring shows at band edges, round storms and towards the poles, not as a whole-globe change.
- **Steps a leg:** Jupiter 955, Uranus 788, Saturn 11,820. Saturn's cloud lasts 336 days, so its still is about 25 times Neptune's work.

### The second round on AC-ProArt (28 Nov), and what it changed

- **What it found** (commit 2071fc7f, both GPUs alike):
  - Four steps a draw never built: three.js r187's node builder overflows its stack on a `Loop` round the step (`Maximum call stack size exceeded`), on WebGL2 and WebGPU. It only says so on the console; the shader read back zeros and the runner called Jupiter's and Neptune's stills built.
  - Saturn's list needed 9,099 rows of 2,048 on that date (18.6 million entries), past 8,192, and was refused.
  - Freezes were down to 1.2 s at worst. Jupiter took 65 s, from a frame's wait after every draw.
  - The globe never changed: `gpu.ts` keeps one texture a world and uploads again only on a new `version`, and the still came under the same key with none.
- **Fixes:**
  - One step a draw always, with no loop round it. The draws go in batches with a frame between, each batch waited on, doubled while it takes under 4 ms and halved over 8 ms (`carryOnGpu` `budgetMs`). The twin test runs the same way.
  - The eddy list is laid out as wide as it needs (2,048 to 16,384, never taller than wide), and where each cell's list starts is kept in two parts, because a float holds whole numbers exactly only to 2^24.
  - Each picture handed to the globe gets a version of its own (`true-body.ts`), so a new picture under the same key is uploaded; one of another size gets a new texture. `test/true-air.test.ts` goes red without it (seen).
  - Before a still is shown, six places are carried by the processor too, in the worker, while the GPU works. Latitude must agree to a pixel of the face, and the whole place to a twentieth of the way round (along the bands a float's last digit times the shear moves it by tens to hundreds of kilometres, on both). A shader that failed to build misses both by far.

- **Third round** (f7c5ecf0): the twin test passed on both GPUs and both backends, and Neptune's still was built and shown (52 s, freezes under 0.9 s). Jupiter's was refused by the new check, wrongly:
  - Jupiter's older generation is chaotic. On the processor alone, a nudge of 0.00001° to the start moves where the older generation was made by up to 3.5° of latitude, at the same places where the GPU and processor part (SwiftShader, Jupiter's whole table at day 1000, 64 places over the globe: younger within 0.013° everywhere, older up to 7.9°). Float32 and float64 are both right to their own last digit, and either is as good a picture.
  - So the check holds only the younger generation's latitude, to a pixel, with at most two of six places allowed to miss; a shader that failed to build misses at all six.
  - The batches never grew: reading one texel back takes most of a frame by itself, so every batch looked over budget. The budget now counts the wait less the least wait seen.

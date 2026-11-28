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

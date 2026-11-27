
### Why the bend table is Rust, not a flow

It is a table of about 65,000 numbers worked out by stepping one differential equation two million times: an Equation block has no loops by design. It is the "heavy physics ships as tables behind swappable blocks" pattern: the table is the design's block `sse.gravity.light_path` (published to the screen with the op `read_light_paths` in XO2), worked out once, hashed (`tests/golden/gravity.json`) and checked the same bits in the browser harness. Every size it is read with (horizon, photon orbit, last stable orbit, shadow) is a formula block in the base pack.

## Small bodies (track SB)


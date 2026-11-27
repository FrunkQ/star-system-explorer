### Why the disc trace is Rust, not a flow

The trace (`sse-light/src/disc_image.rs`, XO3) is a loop over about a hundred thousand rays of the picture, each read off the bend table: a loop, so not an Equation. Its physics is in blocks it runs for every ray: the shift factor is `sse.light.disc_shift`, the ring temperature is the thin-disc law, and the feed it draws (`sse-light/src/feed.rs`, XO2) only wires the seven `sse.feed.*` and gravity blocks together. The crossing angle and first-hit rule (`landing`) are geometry, stated in its words, and the renderer repeats them on the GPU.


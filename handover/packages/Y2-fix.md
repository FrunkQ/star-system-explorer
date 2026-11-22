# Y2 fix: massive stars and sky speed

Follow-up to Y2 (`Y2.md`), built 22 Nov 2026 by the kernel thread after the renderer thread's Y5 report: a 6.5 magnitude sky took 46 s in release wasm, and the brightest fill star was a 4 × 10⁷ L☉ "giant" at 4,500 K, far above any star ever measured.

## Massive stars (`crates/sse-cosmos/src/stars.rs`, fudge F-S11)
- **Main sequence above 31 M☉.** The Eker law was built on stars up to 31 M☉; above that L now rises as (M/31)^1.5 from the Eker value at 31 M☉ (`mlr_top_msun`, `mlr_top_slope`; Ekström et al. 2012, doi:10.1051/0004-6361/201117751). A 150 M☉ star is now about 2.5 × 10⁶ L☉ rather than 4 × 10⁷.
- **The cool limit.** A giant brighter than 6.3 × 10⁵ L☉ (`cool_limit_lsun`, Humphreys and Davidson 1979, doi:10.1086/157301) is a "hot supergiant" at its main-sequence temperature. No star that bright is seen cool.

## Sky speed (identical outputs except where noted)
- **Classes.** The 1 to 10 and 10 to 100 L☉ decades are split at 3 and 30 L☉, giving 14 classes. Those decades held most naked-eye stars, and each class searches only out to its own radius, so the split cuts the space searched.
- **Draws per class.** A leaf draws each class with its own keyed slot (`j × classes + c`, output sorted by slot), so one class no longer counts the others. This changes which stars are drawn (same statistics), so `tests/golden/stars.json` is re-blessed on purpose: the "before" count goes from 96 to 98 and both digests change.
- **class_grid.** Each IMF point's class per age bin is computed once per walk and reused by `class_mix`.
- **Draws prefix.** `sse_core::draw::Draws` keeps the hasher of its key prefix and clones it per counter. Values are unchanged (test `a_draws_object_gives_the_free_function_s_values`).

## Measured (default universe, from the Sun's place, test build)
- A 6.5 sky: 12.6 s, was 28 s.
- Stars brighter than V 0, 1, 2, 3, 4, 5, 6 and 6.5: 3, 17, 56, 165, 544, 1,843, 5,932 and 10,538. The real sky has about 4, 15, 48, 171, 513, 1,602, 4,800 and 9,000.
- The brightest fill star is a 2,500 L☉ giant at V −1.56.

## Note for other threads
The light thread's `sse_sky::glow::class_radii` reads class edges from params, so it follows the 14 classes without change. The renderer's sky slices by class index; any index it hard-codes should be read from the class edges instead.

## Follow-up: cut lists and the glow
`sky_from_here`'s `max_stars` now defaults to 20,000, the design's phone tier (ruling 12; desktops ask 200,000). When the list is cut it says `truncated` and gives `kept_to_mag`, the faintest star kept. Asking `sky_glow` with that as its limit puts the stars that were cut into the glow, so no light is lost. The renderer thread asked for this on 22 Nov. The Window's model line now says only moons are not drawn.

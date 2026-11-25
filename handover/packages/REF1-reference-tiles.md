# REF1: reference bodies show their own surface (`reference_tiles`)

This was asked by the coordinator for Alex's size comparison, 24 Nov. Every well-known Solar System body should look like its real counterpart in True.

## The cause

The kernel already gave every base-sol body its surface. This was checked on 24 Nov: with Sol opened from `packs/base-sol` the way the start sheet's door does it, `resolved_system` returns all six level-0 tiles for Mercury, Venus, Earth, the Moon, Mars, Jupiter, Saturn, Uranus, Neptune, Pluto and Ceres. Only the Sun is refused, as it should be.

The size comparison's reference bodies are not in any campaign system, so they never asked for tiles, and stayed grey.

## The op

`reference_tiles {body, tiles: [{face, level, x, y}]}`. `body` is the base-sol id (`mars`, `moon` and so on).

- **How it reads.** The kernel enters base-sol, as `packages/ui-contract/src/sol.ts` builds it, into a private reference universe (`u:reference/sys:sol`, seed `reference`). It then reads that body with `resolved_system`.
- **Answer.** `reference_tiles {body, address, tiles, refused, regime_label}`.
- **No campaign effect.** The campaign is never touched (its state hash is unchanged), and the op works with no campaign open.
- **Caching.** The reference universe is kept, so orbit runs are reused, until `load_stack`.
- **One world.** The reference Mars is the same world whatever campaign is open, because it has its own seed. It is not the campaign Sol's Mars, whose seed is the campaign's.
- **Session.** `session.referenceTiles(body)` gives the six level-0 tiles. Any seat may read them, since these are public facts.

## Checks

- `sol_tests::reference_bodies_have_surfaces_without_a_campaign`:
  - Mercury, Venus, the Moon and Mars get six tiles each;
  - the Moon is filed under Earth;
  - the campaign's state hash is unchanged;
  - a bare kernel gives the same Mars;
  - an unknown id lists the bodies there are.
- `perf-chunks.json`: the op is timed in the `resolve` chunk.

## Next

- Renderer: switch the Gallery's reference bodies to `referenceTiles`. The thread "Build photoreal renderer" was told.
- The big moons resolve the same way once base-sol carries them (the kernel thread's barycentre PR).
- The Kestrel sample worlds are fixtures, not base-sol. If they should get faces too, they need the same treatment from their own fixture sheet.

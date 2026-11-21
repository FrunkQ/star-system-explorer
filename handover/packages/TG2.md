# TG2: the physics wall

Tags design (`design/SSE4-tags-design.md`) sections 4 and 7, package TG2.
Built 21 Nov 2026 by the kernel thread, after TG1. It turns "tags never
reach physics" from a habit into a check.

## What it delivers
- `tags::physics_wall`: when a stack loads, every physics flow, and every box it uses (followed recursively), is searched for the blocks that read the game layer: `world.has_tag` and `world.read_kept`. A flow that uses one is refused, naming the pack, the flow, the node and the block. The refusal joins the stack's other flow problems.
- Tags and kept values still reach conditions in rules, triggers, programs and sifters, which are not physics. The only bridge from game to physics stays the record design's explicit, marked `set_pin`.

## Verified by
- `a_physics_flow_that_reads_a_tag_is_refused`: the toy physics flow loads. With a `world.has_tag` node added, the stack is refused with `sse.base-toy: flow toy.star-and-planet: node cursed reads world.has_tag ...`. The same happens for `world.read_kept`.
- `fifty_tags_on_sol_change_no_resolved_fact`: the synthetic Sol V3 save is adopted, then 50 tags are set across its bodies and taken off again. `resolved_system` (spec, every fact and its evidence, apart from the `tag.*` labels themselves) is identical before, with the tags on and with them off.
- `scripts/check.sh`: no golden changed.

## Goldens
None changed.

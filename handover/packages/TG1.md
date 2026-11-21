# TG1: tags are declared

Tags design (`design/SSE4-tags-design.md`) sections 4 and 7, package TG1.
Built 21 Nov 2026 by the kernel thread. The standing rule: a tag is only
a label, with no value and never an input to physics.

## What it delivers
- **`provides.other.tags`** in any pack (`crates/sse-kernel/src/tags.rs`):
  - `groups` entries carry `id`, `label`, `about?`, `colour?`, `applies_to?` and `exclusive?`;
  - `tags` entries carry `id`, `label`, `about?`, `group?` (default: the part before the first dot, if that group is declared), `source` (`authored`, `derived` or `either`, default either), `tier?` and `colour?`.
  - A tag's `tier` becomes its default disclosure tier wherever the stack's disclosure does not already list it. With no tier, a tag stays at the GM-only rung (ruling 4).
- **The load-time check.** Every tag named by a rule, sifter, template, trigger, program, data view, disclosure table, tag grant, lens, intent or code sheet must be declared somewhere in the stack. Otherwise the stack is refused with one message per reference, naming the pack, the reader and the item, for example `test.rpg-rules: trigger consequence names tag insult, ...`.
  - References are found in `tag` and `*_tag` texts, `tags` lists, views' `{mode, list}`, disclosure `tags` tables and `tag.<name>` fields and operands.
  - Grant patterns containing `{audience}` are templates, not names.
- **Declarations must agree.** Two packs may declare the same tag word for word (a shared vocabulary). A different declaration of the same id is refused, and so is a group a tag names that no pack declares.
- **Reserved for the 4.1 economy pack** (a pack whose id is `economy` or ends with `.economy`): the tag group `trade`, tags in it, and kept kinds with the prefix `economy.`.
- **The campaign's own tags.** When the GM sets `tag.<name> = true` on a name no pack declares, it joins the campaign's declarations. These are kept in the record at the universe root (`campaign.tag_decl.<name>`, event kind `tag.declared`), so they travel with a save. `set_authored` answers with `declared_tag`. The new op `campaign_tags` returns them in a pack's `tags` shape.
- **Exclusive groups.** `set_authored` refuses a second tag of an exclusive group on one object (`exclusive_tag`). Taking the first off makes room.
- **Declarations in the packs.**
  - The Traveller pack declares its 86 tags: template kinds and the code sheet's trade codes, bases, zones and gear, in a `traveller` group.
  - `traveller-generated` is declared for now with a note: it is a source mark and becomes a mark under ruling 9.
  - Test fixtures declare the tags they name.

## Verified by
- `an_undeclared_tag_is_refused_where_it_is_named`: one undeclared tag each in a rule, a template, a trigger and a view gives one message per place, naming pack, reader and item. The fixture's sifter, which reads the rule's tag, is the fifth.
- `the_traveller_pack_loads_clean`.
- `a_freely_typed_tag_joins_the_campaign_and_exclusive_groups_hold`: a freely typed tag joins the campaign once, as authored with no tier, and survives export and import. A second allegiance is refused; after the first is taken off it is allowed.
- `the_economy_names_are_reserved_and_declarations_must_agree`.

## Goldens
`tests/golden/contract.jsonl` changed on purpose. The fixture packs gained `tags` blocks, so their content hashes changed, and with them the stack hashes, manifest ids, state hashes and export bytes that include them (25 lines, ids and hashes only). The hello and bad_request lists gained `campaign_tags`, and one case was appended. `tests/golden/traveller-codes.json` changed on purpose too: only its `pack` hash moved, because the Traveller pack gained its `tags` block. Every code is identical. No other golden changed.

## Not done
- TG2, the physics wall: a flow reading `world.has_tag` refused at load.
- TG3: authored `false` holding a rule tag off.
- TG5: tier overrides and the undisclosed marker.
- Tag names are not yet forced to lowercase `group.name`. Traveller's trade codes keep their case (`traveller.trade.As`), and its template kinds are single words (`ship`, `construct`).
- Lenses live in the UI today. The check reads a pack's `lenses` block when one exists.

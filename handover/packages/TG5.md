# TG5: tags level with facts in data views

Tags design (`design/SSE4-tags-design.md`) section 4 ("Visibility"), questions 4 and 5, and section 7, package TG5. Built 21 Nov 2026 by the kernel thread. There is no separate visibility mechanism: tags pass through the same data view filter as facts.

## What it delivers
All of it is in `sse_observe::scene::subject`, the one redaction point. Every reply a player can be sent is built from a scene: `scene`, `subscribe`, `disclosures`, `sift` with an audience, and a program's senses.
- **Per-object tier.** The authored fact `observe.tag_tier.<tag>` (a tier name) moves one tag on one object to another tier, for example the syndicate that openly owns one station and secretly owns another. It is written with the ordinary `set_authored`, so it is chronicled and undoable like any GM edit. It is an `observe.*` field, so no scene shows it. A name that is not on the ladder is ignored, and the declared tier applies.
- **One marker.** When at least one tag sits exactly one rung above what the view shows, the subject carries `undisclosed: true`. Facts follow the same rule (`unknown_style: placeholder`, not the secret rung). The marker is a bare flag: no id, colour or count, so three hidden tags look like one. Tags two rungs up, GM-only tags, tags the view's tag filter leaves out and tags hidden by an override give nothing. The marker alone does not reveal that a subject exists, just as fact placeholders do not.
- **Note tags.** A note's tags in a scene pass the same filter as the subject's tags wherever the tag has a tier (declared in the disclosure, or overridden on the object). Before this, a player's scene showed every tag on a note it could see. A tag with no tier anywhere, such as `insult` typed by the crew's seat on its own note in the contract, goes where the note goes. Hiding a seat's own words from it would protect nothing, and the contract golden stays as it was.
- **Contract.** `undisclosed?: boolean` was added to `PlayerSubject` in `packages/ui-contract/src/player-scene.ts`.

## Default tier for a new GM tag (question 4)
A tag with no tier anywhere falls to the disclosure's `default_tier`, else the secret rung, so it is GM-only. That was already the case, and it matches the recommendation. Imported V3 tags carry their own tier through their declaration (TG6).

## Verified by
`hidden_tags_give_one_marker_and_a_gm_only_tag_reaches_no_player` (sse-kernel `tags_tests`):
- three tags at `surveyed` give one marker to a `near` view (obvious and scanned) and nothing to a `far` view (obvious only);
- the GM sees all five tags and no marker;
- per-object overrides move tags up and down, and the marker follows;
- for both audiences, `scene`, `disclosures` and `sift` with an audience, with and without a sifter on the GM-only tag, contain neither GM-only tag nor the override fields;
- the GM's own sift still finds the world;
- a note reaches players with only the tags they may see.

## Goldens
None changed. No golden scene has a tag one rung up under a placeholder view, or a note with tags.

## Not done
- `export_chronicle` and the campaign export are GM files, not player replies, and they hold every tag by design. Players get `disclosures`.
- The chart room's tier button and "as players see it" marker are TG10 (chart room thread).

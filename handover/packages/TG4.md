# TG4: seeded chance on rules

Tags design (`design/SSE4-tags-design.md`) question 1 and section 7, package TG4. Built 21 Nov 2026 by the kernel thread. Question 1 was accepted with the rest of the design.

## What it delivers
- A rule (`provides.other.rules`) may carry one optional `chance` between 0 and 1. Where the rule's condition holds, the tag also needs a roll under those odds.
- The roll is `sse_sift::chance_roll(seed, address, rule id)`, stream `sift.rule.chance.v1`. The counter is the first 8 bytes of a hash of the rule id. So the roll depends on the universe seed, the subject's address and the rule id only:
  - shuffling the rule list changes no tag;
  - adding a rule, or disabling a group, changes no other rule's tag;
  - it is the same roll at every date, so a rolled tag does not flicker while a world ages. The condition and its hysteresis still decide it alongside the roll.
- `chance: 0` is never and `chance: 1` is always, with no draw. Odds outside 0 to 1 fail the stack load.
- A rolled tag's "Why?" lists the facts its condition read, plus `chance` with the odds.
- `RuleSet::apply_authored` takes the universe seed. The kernel's `scene` and `sift` ops pass it. Without a seed (tests, `track`) nothing can be rolled, so a rule with odds strictly between 0 and 1 does not fire.

## Use it only where physics cannot decide
The design's rule applies: a rumour or a legend may have a chance; anything physics can decide carries none. Resource labels never carry a chance (design section 5.5, TG8).

## Verified by
`seeded_chance_depends_only_on_seed_address_and_rule` (sse-sift), with 500 bodies:
- reversing and swapping the rule list, inserting a rule and removing one leave every other tag the same;
- 0.3 odds give about 150 of 500;
- 0 and 1 behave as never and always, with and without a seed;
- another seed rolls differently;
- the "Why?" carries `chance`;
- odds of 1.5 are refused.

## Goldens
None changed. No shipped or fixture rule has a chance.

## Notes for other threads
- Orac's rule writer (`packages/agents/src/orac-rules.ts`) can now offer `chance` on hook-style rules only.
- The optional intrigue pack (TG8) is the first real user.

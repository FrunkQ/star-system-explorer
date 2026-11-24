---
name: traveller-pack-asks-23nov
description: Traveller pack thread's later asks and rulings of 23 Nov 2026 (ships done, licence, difficulty ladders, links, SV7, RK, manifest)
metadata:
  type: project
---
Continues [[traveller-pack-queue]]. WBH plan: [[traveller-wbh-generator]].

- 23 Nov: example packs 4.1 done in PR 289 (18 standard ships as templates and `nav_ships`, Mongoose notice, World hooks pack). Alex agreed 23 Nov: Traveller pack data alone is CC BY-NC 4.0 (packs/traveller/LICENCE.md; repo stays Apache-2.0); he emails Mongoose (sales@mongoosepublishing.com) himself, draft given. Travel thread asked to list packShips in voyage pickers.
- 23 Nov (Alex): game-pack difficulty ladders. A pack declares `difficulty` {dice, ladder of names and targets, mapping from SSE4 terms or odds, role to skill}; shown on the At world card and route console (travel thread) GM-only; SSE4 rolls nothing. Traveller first, checked against the Cepheus Engine SRD (OGL). Plan sent with 3 questions (odds through dice first, skill per role, GM-only). Also: standing links (travel thread) carry MP16 counted values; route import from travellermap XML is ours once their link op lands.
- Queue, last: SV7 (secrets and visibility design, handover/design/SSE4-secrets-and-visibility-design.md section 11): Traveller pack names its own rungs for GM only, heard of and known; waits on kernel SV1.
- Alex 23 Nov 19:50: game packs other than Traveller wait for 4.2 (Mothership game side, Eclipse Phase, 2300AD out of 4.0). Traveller is the showcase; Traveller work unchanged.
- Small follow-up: switch the ship's log (G16 shipLog) to ui-contract crewAboard(subject, roster) from PR 318 (manifest thread) once merged.
- Queue (Alex yes 23 Nov, Atomic Rockets plan track RK, SSE4-atomic-rockets-plan.md sections 12-15): RK11 our share: pack sensors move to one instrument shape with computed ranges; typed ranges import as marked authored ranges; range bands become setting measures (light thread owns sensing code). RK12 our share: Traveller weapons as 'effect at range' (in range/marginal/out of range ring; nothing fired), game table beside physics; waits on RK9 power ledger. Sensors must 'just work', tweakable by those who care. RK11 kernel side landed 23 Nov (PR 329): instruments name a method (image, spectrum, astrometry, active, magnetometer, gradiometer) with aperture and noise; detect takes range_bands; pack side DONE 24 Nov PR 366 (X3i): each kind has an instrument, detect computes reach, V3 range kept authored, fudge F51; next: triggers asking detect (kernel), fits per tech level (FT8).
- Late: RK16 pack side (Atomic Rockets plan section 17): map Traveller stealth and countermeasure gear onto the counters list; waits on RK14, RK15.
- Alex 23 Nov 22:12, difficulty ladders: yes to odds through the system's dice first (word table fallback), GM-only, and show the skill when the game pack knows the game's skills.

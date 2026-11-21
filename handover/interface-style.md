# Interface style: dynamic minimalism (standing)

Alex, 19 Nov 2026, on the chart room: "we have VAST amounts of data under the hood but it looks approachable ... I think of it as a kind of 'dynamic minimalism'. Keep it. It could maybe gain additional 'user tactility' with dynamic popups and animation flourishes."

Every interface thread holds to this, on GM and player screens alike (player themes change the look, not these rules).

## Minimalism: the surface stays calm

1. **Show what the question needs, and keep the rest a step away.** The depth sits behind lenses (Basic, Advanced, player), cards, "Why?" and Advanced disclosures, never in one crowded panel.
2. **Words before numbers. Numbers carry units and honest precision.** Say "about", not "~".
3. **Use the tokens only.** Colour, type, spacing and lines come from `packages/tokens`. One accent, used for the present, the campaign's now, authored ink and the action that matters.
4. **One mechanism per job.** One ruler for all of time, one card for any object, one lens switch. Don't add a second control for a special case.

## Dynamic: tactility without noise

5. **Every touch answers.** Hover, press and focus each give feedback within one frame, through a small change in tone, a 1 px lift or an inked edge. Never just a colour flash.
6. **Things open from what they describe.** A card, popup or dialog grows from the body, tick or button that opened it, and goes back to it when closed. Popups appear next to the pointer or the selection, never in a fixed corner far from it.
7. **Motion explains a change of state.** The ruler eases when it zooms, a tab slides its underline, and a value counts to its new figure. Motion is never decorative looping. It lasts 120 to 240 ms, eases out, and can be interrupted.
8. **Fast first.** Motion never delays input or a result. Animate only `transform` and `opacity`. Drop it on low-tier devices.
9. **`prefers-reduced-motion` is honoured everywhere.** It turns motion into instant changes or plain fades. Feedback on touch stays.
10. **Positions in time come from the kernel** (rendering rule 5). Animation between two states is presentation and is labelled as such where it could be mistaken for physics.

The shared helpers for this (`motion` tokens, an `opensFrom` action, and the reduced-motion switch) are in the UI track's tactility pass. Use them rather than writing your own.

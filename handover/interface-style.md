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

11. **Every diagram can be turned.** Any diagrammatic view of something with depth (a system, a body, a galaxy, a field, the web) offers zoom, slide and tilt. Its "Seen from" choice carries the fixed views that suit it (Above, Tilted, Edge-on or their like) and always a **Free** view: drag to turn and tilt, Shift-drag or two fingers to slide, wheel or pinch to zoom. Dragging with a mouse switches to Free by itself. Picking a fixed view puts everything back. On touch, one finger scrolls the page until Free is chosen, so a chart never traps a scroll. Flat charts (a ruler, a rotation curve, a ledger) zoom and slide only. Alex, 22 Nov 2026. Shown in the galaxy chart mockup.

12. **One set of view controls on every diagram.** Every diagram display has the same two switches, in the same place (the top of the stage, beside the tools), with the same words in the same order:
    - **View style:** Chart, 3D, True, Wireframe.
    - **Seen from:** Above, Tilted, Edge-on, Free.

    This covers the system chart, the body globe, the cutaway, Mag, the galaxy chart, the galaxies nearby, the cosmic web, the formation replay and the sky.
    - **An option a view or device cannot do stays where it is,** quiet, with the reason as its title (for example "True is coming: the photoreal views are held for now"). The control never rearranges itself from one screen to the next.
    - **A flat chart** (the system's log chart, a slice) is seen from above. Choosing Tilted, Edge-on or Free on it turns it into 3D where the view has 3D.
    - **Free** works the way rule 11 says. Dragging switches to Free by itself, and the hint line says how Free is worked.
    - **The choice is the viewer's,** for any seat. It is kept on their device, and later per panel, and never in the campaign.
    - **One mechanism.** A view declares what it draws (`ViewModule.controls`, `ui-contract/src/view-controls.ts`), and the shell, the players' page and any panel draw the one `ViewControls` component (`chart-grammar`). A view never draws its own 2D/3D or view switch.

    Alex, 22 Nov 2026.

The shared helpers for this (`motion` tokens, an `opensFrom` action, and the reduced-motion switch) are in the UI track's tactility pass. Use them rather than writing your own.

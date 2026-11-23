# Test drive (stage 1)

Stage 1 of `handover/design/SSE4-coming-together.md`. Alex accepted the plan on 23 Nov 2026, and set the guiding rule: "intuitive and approachable. Most common things a click away. Rarely touched things in menus that have relevant context."

The chart room thread keeps this file current. A fix to the walk goes ahead of any new screen.

## The walk

Open https://v4.starsystemx.com/ and try these steps in order. The check `tests/ui/walk.e2e.mjs` runs the same walk on a desktop and a phone for every pull request. It counts the presses each step takes, so a change that buries a common step fails.

| Step | What to do | Presses |
|---|---|---|
| Open the Solar System | On the start sheet, choose "Open the Solar System". | 1 |
| Pick a world | Press Earth on the chart. | 1 |
| Ask why | Press "why?" beside its orbit. It says the value comes from a real catalogue. | 1 |
| Change a world | Pick Mars. Choose the Build desk, then type a new orbit (1 AU) and press Enter. | 2, plus typing |
| See what followed | Mars's surface, which the physics works out, warms from about 206 K to about 255 K. Press its "why?". | 1 |
| Add a world | Pick the Sun, press "Add here" (the + in the Build tools, or A), then "Add this" on one of what forms there. Undo takes it back. | 3 (4 on a phone) |
| Open the globe | Pick Earth, then press "Open Earth ›". | 1 |
| Its magnetic field | Press "Mag" over the globe. | 1 |
| Cut it open | Choose "Cutaway" on the rail: crust, mantle, core and Movile Cave. | 1 |
| Run time | Press Play on the ruler. The ruler closes in so the days visibly pass. | 1 |
| Out to the cosmic web | Press "Web" at the left of the place line. | 1 (2 on a phone) |
| Straight back down | Press "‹ Back to Earth" on the rail, or Earth at the end of the place line. | 1 |
| Look up an element | Choose "Look up" on the rail, type its name (iron) and press Enter. A phone presses the Find box first. | 1 (2 on a phone) |

Undo (Ctrl+Z, or the arrow on the Build tools) takes the edit back. Nothing in the walk needs a key or an account.

## Known rough edges

These are unfinished on purpose, or known and queued. The owner thread is in brackets.

Physics:
1. **The Solar System's Earth has no greenhouse.** Its surface reads about −18 °C, its globe shows bare ground with "no surface liquid", and its regime reads "episodic lid", not plate tectonics. Worlds read back from the universe take their surface from the surface chain, which does not yet carry a catalogued atmosphere (kernel and proven core).
2. **The cutaway draws only worlds with a world stack.** Those are Earth, Mars, Venus, Europa and Titan in the Solar System, and Tarn in the sample. Any other world says "no cutaway yet" and lists the worlds that have one. Stacks built from each world's own history come later (proven core).
3. **Going up from a system opens the galaxy you last charted, not the system's own.** The kernel does not yet place systems in a galaxy (kernel, galaxy histories).

Presses:
4. **Find in Look up covers elements, constants and units.** Substances join it with their page (MP10 part 3), and it does not yet search names on the chart (chart room, Find in stage 2).
5. **Adding a world takes three presses** (pick the star, Add here, one of what forms there), and a fourth on a phone, where Add here is in More. With nothing selected it goes round the primary in two. Moons are not sampled from formation yet: "Something else" adds one, marked impossible (chart room; kernel for moon formation).
6. **On a phone, the cosmic web takes two presses.** The place line shows only the step above, so the way out is the list beside the name, under "Further out".
6a. **A world's magnetic field takes two presses:** the Show chip, then Mag. Band is folded with HUD because it is changed far less than View style, Seen from and Scale (chart room, 23 Nov).

Layout:
7. **The place name is cut to a letter or two on a narrow phone.** The bar holds the desk switch, "See as" and More beside it.
8. **On a phone, the view controls take two rows above every diagram** (HUD and Band are folded into one chip).
9. **The galaxy charts and the cosmic web are flat.** Their 3D, Tilted and Free options show as unavailable, with the reason (galaxy charts).
10. **Wireframe and in-universe looks are held** (Alex: design only for now). True ships on the system chart (V10a) and the globe.

Editing:
11. **A system read back from the universe (the Solar System, a grown or imported one) can be typed over in orbit, eccentricity, mass, radius and day,** and in a star's mass, radius, temperature and luminosity. Other fields and names are not yet editable there.
12. **"Why?" on a catalogued value says "a real catalogue" without naming it.** The source is kept with each value (U21); the card should name it (chart room).
13. **"Why?" on a value the kernel works out for such a system explains in words.** It does not yet open the flow, as the sample system's values do (kernel: flow references for read-back values).

## Changed for the walk (23 Nov 2026)

- The Solar System's worlds open in the cutaway by the catalogue's own ids, when the kernel has their stack.
- A system read back from the universe is edited in place on the Build desk. Each typed value goes to the fact the universe holds, in SI, so derived values follow and Undo works.
- Play closes the ruler in on a window that fits the step, so time visibly passes.
- "Look up" (Reference) is on the rail at every system and world, under Understand.
- The place line keeps inside its own space at every scale, and the trail gives way first. The system's name is its own switcher when the campaign holds more than one system. Below adds the campaign's other systems and "Further out".
- The phone's rail chips no longer squeeze into each other.
- Look up has one Find box: an element, constant or unit by name, symbol or number, so looking one up takes a press and a word.
- Add here (V3 gap 2): a world or moon by hand on the Build desk, from what formation runs make at that orbit, or anything, marked impossible.

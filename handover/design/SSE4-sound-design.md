# SSE4 sound design: hearing the universe

Written 18 Nov by the design thread at Alex's request. Authority for sound in SSE4. Read with the handoff (H) sections 8 (views, themes, Orac-first configuration), 9 (peer-to-peer), 14 (device edges) and the surface design (descent scene, atmosphere).

Alex's rulings (18 Nov): sound is wanted; "if sound can exist it should"; hearing the universe; procedural tonal background music derived from the physical characteristics of what you are looking at, simple tones and not composed music, **as background only**; and an atmosphere filter that plays a sample of speech or music as it would sound on that world, the audio twin of light dimming as you drop into a gas giant, useful at the RPG table.

---

## 1. Principles

1. **Sound is one more renderer.** It is driven by the same kernel facts as every picture, after the same player data-view filter, so players never hear what they are not allowed to know.
2. **Synthesised, never baked.** Sound is made in the browser from small data-only **sound recipes**. No audio files are shipped except a handful of short reference clips for the atmosphere filter. Recipes are deterministic, shareable in packs, and editable by Orac (H8.9).
3. **Honest about what you are hearing.** Every sound carries one of three labels, the audio version of the evidence labels:
   - **real sound**: pressure waves that a microphone there would record;
   - **translated signal**: something real that is not sound (radio, light, an oscillation too slow to hear), shifted into hearing by a stated rule;
   - **designed**: interface and instrument sounds, and the background music.
   Each has a "Why?" that states the physics or the mapping.
4. **Quiet by default.** Sound is off until the user turns it on (browsers also require a click before audio may start). The GM chart room is silent unless the GM opts in. Background music is a bed: slow, low, never demanding attention, and it ducks under voices.
5. **Never the only carrier.** Nothing is conveyed by sound alone; every sound has a visible or text equivalent. In the other direction, sound is offered as an accessibility aid, so charts can be heard.
6. **Device edges are clear.** Tiers `silent | basic | full`, declared and shown like render tiers.

## 2. Architecture

```
kernel facts -> data-view filter -> SoundScene (plain data) -> sound engine (browser) -> speakers
                                         ^ recipes, theme and mixer settings from packs and the user
```

- **SoundScene**: what is audible from this vantage now. Sources with position, kind, driving quantities (wind speed, gas density, storm rate, body periods, hazard level), the medium between source and listener, the listener's situation (open air, helmet, inside a hull, vacuum), and a deterministic **score** of timed events (a thunderclap at t, a note at t) drawn with keyed draws. The score is a kernel product and is identical on every device. The rendering of it into samples is the device's job and may differ in the last decimal, which is acceptable because nothing reads it back.
- **Sound recipe** (pack data, declarative): a small graph over a fixed set of nodes: oscillators, noise sources, filters, envelopes, a grain player, delay and reverb networks, a clip player, and mappings from scene quantities to node settings. Full parameter schema for Orac, three to five promoted controls for people.
- **Engine**: Web Audio API with an AudioWorklet for custom processing (written in Rust and compiled to WebAssembly where it matters). No server and no streaming: a GM who pushes ambience to the table sends a recipe id and settings, a few hundred bytes.
- **Mixer**: four layers the user balances: world, instruments, interface, music. Per-seat settings (H8.7), so the sensors station and the shared screen can differ.

## 3. The four uses already agreed

| Use | Examples | Label |
|---|---|---|
| **World sound** from physics | wind from real wind speed and gas density; thunder where the climate gives lightning; rain, methane rain, surf, volcanic rumble, ice cracking; and **silence in vacuum**, with only what comes through your boots and suit | real sound |
| **Translated signals** | a pulsar's spin as clicks; a magnetosphere's radio chorus; a star's oscillations sped up by a stated number of octaves; a passing ship's drive signature on sensors | translated signal |
| **Instrument sounds** | astrogation console, sensor contacts, approach and landing callouts, alarms tied to real thresholds | designed |
| **Interface and ambience** | per-theme interface sounds (green phosphor beeps, soft high-tech tones); GM-pushed ambience for a scene | designed |

## 4. The atmosphere filter: "what would we sound like here?"

Pick a world (and a height, for gas giants and thick atmospheres), pick a clip of speech or music, and hear it as it would sound there. It works on a live descent too: the same clip changes as the ship drops, in step with the light dimming.

**What changes, and from which facts** (all already published by the atmosphere flows):

| Effect | Physics | What you hear |
|---|---|---|
| Speed of sound | c = √(γ R T / M), from gas mix and temperature | resonances of the mouth and of wind instruments scale with c: voices go "helium-high" in light gases, deep in heavy ones. Strings and drums keep their pitch |
| Loading on the source | gas density against the vibrating thing | in very dense air, vocal folds and reeds are weighed down and pitch drops |
| How loud it starts | acoustic impedance ρc compared with Earth's | thin air barely carries sound (Mars is far quieter for the same effort); thick air carries it strongly |
| What survives distance | absorption that rises with frequency and depends on the gas; carbon dioxide is especially lossy | on a Mars-like world the high notes die within metres and distant sounds are dull thuds; on a Titan-like world sound carries far |
| Frequency-dependent speed | relaxation in some gases | on Mars, sounds above about 240 Hz travel measurably faster than lower ones, so a distant sharp sound smears |
| Bending and ducting | temperature and wind gradients | sound trapped near the ground, or lost upward |
| No medium | pressure below a floor | nothing through the air at all; only contact sound |

The listener's situation is a switch: bare ear (hypothetically), through a helmet microphone, or inside a pressurised hull.

Sources to build from, cited the way physics blocks are:
- Leighton, T. G. and Petculescu, A., "The sounds of music and voices in space", parts 1 and 2, *Acoustics Today* 5(3), July 2009. Theory and worked examples for voices and instruments on Mars, Venus and Titan. **[existence checked]**
- Maurice, S. et al., "In situ recording of Mars soundscape", *Nature* 605, 653 (2022). Perseverance microphones: sound speed near 240 m/s, two speeds either side of about 240 Hz, strong high-frequency loss. **[existence checked]**
- Petculescu, A. and Lueptow, R. M., "Atmospheric acoustics of Titan, Mars, Venus, and Earth", *Icarus* 186 (2007). Absorption and sound speed models by gas mix. **[from memory, verify before use]**
- Bass, H. E. and Chambers, J. P., "Absorption of sound in the Martian atmosphere", *J. Acoust. Soc. Am.* 109 (2001). **[from memory, verify before use]**
- For Earth-like air, the standard atmospheric absorption model (ISO 9613-1) as the golden case.

Implementation: gain from the impedance ratio, a filter bank shaped by absorption times distance, a delay from distance over c, and for voices and wind instruments a formant shift by the ratio of sound speeds, leaving pitch alone except for the dense-gas loading term. Tests: Earth gives the clip back unchanged; Mars reproduces the Perseverance figures; helium-oxygen reproduces the familiar diving-gas voice.

Clips: three or four short public-domain or specially recorded clips ship (a spoken sentence, a sung phrase, a flute, a guitar). A user may use their own clip or microphone; it is processed on their device and never leaves it.

## 5. Background music from what you are looking at

Not composition. A slow, tonal bed whose every property comes from a fact about the thing in view, so two different worlds *sound* different for real reasons, and a listener who learns the mapping can hear the system.

**The mapping** (a recipe, so all of it is adjustable):

| Fact | Becomes | Why this choice |
|---|---|---|
| Orbital periods of the bodies in view | the notes: each period is raised by a fixed number of octaves into hearing | period ratios become musical intervals by themselves. A resonant chain (3:2, 4:3) is literally a consonant chord; a system near but not on resonance beats slowly; an unrelated system is an open, unresolved cluster |
| Size and mass | register and weight: bigger is lower and fuller | matches how physical resonators behave |
| Star temperature | brightness of the whole bed | red dwarf systems sound dark and warm, hot stars thin and bright |
| Composition | timbre family: rock as soft mallets, ice as glass, gas as breath, metal as bell | recognisable after a few minutes of use |
| Atmosphere depth | how much the tone is softened and how long it rings | thick air blurs, vacuum is dry and exact |
| Rotation and eccentricity | slow pulse and slow swell | one cycle per day or per orbit, scaled |
| Activity (storms, volcanism, flares) | grain and movement in the texture | calm worlds are still |
| Life | one extra, gentle voice that is otherwise absent | a living world is audibly different, without saying how |
| Hazard and upheaval | roughness and dissonance | tension without melodrama |
| Scale of view | from single tones (a moon) to chords (a system) to slow clouds (a cluster, a galaxy) and very low drones (filaments) | zooming up is audible |
| Time scrubbing | the bed evolves with the system; chronicle events (an impact, a nova) are brief stingers | deep time can be heard passing |

- A dial runs from **true ratios** (exactly the physics, sometimes strange) to **settled** (each note pulled to the nearest pitch of a calm scale). The dial's position is shown, since "settled" is no longer exact.
- Note timing uses keyed draws, so the same view gives the same music for everyone at the table.
- Rules for staying in the background: slow attack and release, narrow loudness range, nothing rhythmic enough to tap to, no high sharp sounds, automatic ducking under speech, and silence is a valid result for an empty view.

Sources and prior work:
- Tamayo, D. et al. (2017), *Astrophysical Journal Letters*, on TRAPPIST-1's resonant chain, with the SYSTEM Sounds project (Matt Russo and colleagues) turning that chain into music by raising orbital frequencies into hearing. The model for the period-to-pitch mapping. **[project and paper existence checked; exact title to confirm]**
- Zanella, A. et al., "Sonification and sound design for astronomy research, education and public engagement", *Nature Astronomy* (2022), arXiv 2206.13536. A review of about a hundred astronomy sonification projects, with guidance on mappings and accessibility. **[existence checked]**
- Hermann, T., Hunt, A. and Neuhoff, J. (eds), *The Sonification Handbook* (2011), free online; and the ICAD conference proceedings. Parameter-mapping practice and its pitfalls. **[from memory, verify]**
- NASA Chandra sonifications (Arcand, Russo and others) as examples of translating images to sound with stated rules. **[from memory, verify]**
- Alex's own reference for the sound he has in mind (18 Nov): https://www.youtube.com/watch?v=WS5UxLHbUKc It is a SYSTEM Sounds video (Alex, 18 Nov), his example of "background sonification": the same project and the same period-to-pitch approach cited above, so the mapping in this section is the intended one. **[not viewable from the design session; the build session should watch it before tuning the music recipe]**
- The generative ambient tradition (slow independent loops that never repeat exactly) as the model for unobtrusiveness.

## 6. In the UI

- A small sound control on every screen: on or off, the four-layer mixer, and for music the true-to-settled dial. All sound sources in view can be listed with their labels and each answers "Why?".
- **Listen here**: on any world's card, opens the atmosphere filter with a clip picker, the listener switch, and a height slider where relevant.
- The Surface view's landing styles (surface design 7.1) each name a sound recipe, so the green-screen descent has its beeps and the photoreal one has wind rising with density.
- GMs can push an ambience or a single sound to players, per seat.
- Captions: an optional line of text describes current sounds ("wind rising, thunder to the north").
- Hearing the charts: any chart can be played as a sweep for users who cannot see it, following the review cited above.

## 7. Stages (track N)

| Stage | Contents | Needs | Evidence |
|---|---|---|---|
| N1 | Sound engine, SoundScene, recipe schema, tiers, mixer, off-by-default, start on user action, captions | V1, V11 | silent until enabled; a scene's score is identical on two devices; a recipe outside its schema is rejected; state hash unchanged by any sound setting |
| N2 | Interface and instrument sounds per theme; astrogation and sensor stations | N1, V2, V8 | every shipped theme has a set; alarms fire at the same thresholds as their visible warnings |
| N3 | Background music generator with the mapping above and the true-to-settled dial | N1, F0 | a resonant fixture system yields the expected intervals; the same view gives the same score for every peer; loudness stays inside the background limits |
| N4 | Atmosphere filter with shipped clips, own-clip option, listener switch, live on descent | N1, P1, S7 | Earth returns the clip unchanged; Mars matches the published speed and loss figures; vacuum is silent; a user's clip never leaves the device |
| N5 | World sound from physics: wind, weather, surf, volcanic and ice sounds, contact sound in vacuum, each with Why | N1, S7 | sources appear only where the facts support them; players do not hear filtered-out facts |
| N6 | Translated signals (pulsars, magnetospheres, stellar oscillations) and event stingers on the time ruler | N1, T1, G6 | each states its shift rule; pulsar click rate equals its spin |
| N7 | GM-pushed ambience per seat; hearing the charts | N1, V10 | a pushed ambience is a few hundred bytes; a chart sweep conveys a fixture trend to a listener test |

N1 to N3 follow the first player views. N4 and N5 follow surfaces. N6 follows the galaxy.

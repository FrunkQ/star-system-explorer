# L1c: boxes F and G: diversity, guilds, the four lenses, traits from the world

Status: done (19 Nov 2026). Kernel track, PR into `main`. Design: life design section 7 and section 14 row L1c.

## What was built
`sse-life::biosphere::resolve(graph, budget, history settings, capabilities, first ages, time since the last catastrophe, physics, seed, birth year)` returns a `Biosphere`:
- `resolved`: the fixed L0 `ResolvedBiosphere`, with guilds and relations.
- `guilds`: `GuildDetail` records with role, level, habitat kind, production and share, intake, physics and plain names, grade, flight and largest body.
- `diversity`, `traits`, the box B budget it used, and the capabilities.

**Budget.** `budget_with` applies box B for the metabolisms and media the capabilities unlock, by the same rule as the L1b history.

**Guilds** (aggregate populations, never individuals). Each guild's habitat occupancy is in the L0 `Guild` record.
- **Producers:** one per metabolism and habitat kind. Each habitat's production is attributed to its metabolisms in proportion to their energy caps (only the best light one, as box B counts it), so producer shares sum to 1 and their production to the budget's.
- **Decomposers** per living habitat kind take in 90% of its production.
- **Consumers** per feeding level, 2 to the habitat's level count, take in production x 0.1^(level-1).
- **Grades** follow the capabilities: microbial; protist with complex cells; animal with large bodies (on land only with land colonisation); plant producers on land with land colonisation.
- **Symbionts** (with complex cells) and **parasites** (with level 2 or more).
- Earth today has 14 guilds and Archean Earth 2.

**Relations, the four lenses:**
- resource: who eats whom, and decomposers on producers;
- habitat: producer to habitat address;
- capability: guild to the capability it rests on;
- ancestry: guild to lineage, and lineage chains such as oxygenic to anoxygenic to chemotrophy, or large bodies to multicellularity to complex cells.
- Relations are sorted, and habitats are sorted before resolving, so the result does not depend on input order.

**Diversity (F)**, a sketch: lineages = 0.05 x area^0.25 (species-area) x sqrt(living provinces) x sqrt(10 K climate bins) x recovery since the last catastrophe (10 Myr time constant) x a grade factor (animals 1, multicellular 0.3, complex cells 0.1, microbial 0.01).

**Traits from the world**, then keyed draws inside the limits:

| Trait | Rule |
|---|---|
| Producer colour | From the photon-flux peak 3.67e6 / Teff nm: pigments absorb where photons are plentiful (Kiang et al. 2007). Options by band; the draw picks one. Sun-like: green, yellow-green, red-brown. Hotter: blue-green, yellow, orange. Red dwarf: dark red, black, purple. Coolest: black, dark grey. Anoxygenic only: purple, brown, olive green. |
| Flight | Largest flyer = 250 kg x (air density / 1.225)^1.5 x (g / 9.81)^-3.5. 1 kg or more: feasible; 0.1 g to 1 kg: marginal; less, or no air: impossible. `not_evolved` without large bodies and land. Flying guilds are capped at the largest flyer. |
| Largest land animal | 7e4 kg x (g / 9.81)^-3, with large bodies and land. |
| Tallest plants | 130 m x 9.81 / g (hydraulic limit, Koch et al. 2004), with land and oxygenic photosynthesis. |
| Pace of life | Generation time against Earth's at 288 K, from the 0.65 eV activation energy. |
| Day strategies | Tidal locking (terminator dwellers), days over 100 h (long sleepers), obliquity over 40 degrees (migration, dormancy). |
| Senses | Chemical always. With multicellularity: vision where light reaches life, hearing in water and in air, an electric sense in salty water, "no vision" where life lives in darkness. |
| Biochemistry | Solvent; handedness is a keyed draw; own origin. |
| Use to visitors | Unrelated biochemistry: probably inert. |

**Vocabulary.** Every guild has a physics name ("anoxygenic photosynthesis on ferrous iron (ocean-sunlit)") and a plain name ("rusty purple mats", "hunters of the land, some flying"). Minerva's story layer comes later.

**Fixtures** (`scenario.rs`):
- `physics(name)`: gravity, air density, star temperature, day, obliquity and locking for the five worlds;
- `all_capabilities()`;
- `biospheres()`: Earth today, and Archean Earth with chemotrophy, dormancy and anoxygenic photosynthesis.

## Checks (`cargo test -p sse-life biosphere`, 6 tests)
- **Earth today:**
  - 14 guilds (8 to 40);
  - producer shares sum to 1 and production to box B's;
  - colour options include green;
  - flight feasible, with flyers up to 250 kg;
  - largest land animal 7e4 kg and tallest plants 130 m;
  - vision among the senses;
  - animal and plant guilds;
  - all four relation kinds present.
- **Archean Earth:**
  - fewer guilds, all microbial;
  - flight not evolved, no land animals;
  - purple, brown or olive producers;
  - fewer lineages.
- **Relations** point only at guilds, capabilities, lineages or habitat addresses.
- **Order independence:** reversed habitats and capabilities give an identical biosphere.
- **10,000 random worlds** (random gravity, air, star, day, tilt, locking, capability subsets, four habitat graphs):
  - no flyer where flight is impossible;
  - flyers never above the largest flyer, land animals never above the land limit;
  - consumer intake never above 10% of its producers;
  - producer shares sum to 1, at most 40 guilds, nothing NaN.
  - 2,018 flying guilds and 2,329 worlds where flight is impossible were exercised.
- **Golden:** `tests/golden/life-biosphere.json` (new), harness engine `life-biosphere`.

## Known limits
- **Light at the surface:** the producer colour uses the star's photon peak, not the spectrum after the atmosphere and water (the planet layer does not carry that yet).
- **Guilds are coarse:** per habitat kind, not per habitat or province. Body plans in oceans (viscosity), armour from dissolved minerals, limb proportions and flight by body mass are not separate traits yet.
- **Use to visitors** is a fixed sentence until shared origins (transfer, L2) exist.
- **Diversity** is a sketch with stated factors: an order-of-magnitude count, not a species number.
- **Physics fixtures:** `WorldPhysics` values are fixture data. The planet layer should supply them (gravity and air from P4 world stacks).

## Integration points
- L1d (signatures, views, sifters) reads `Biosphere`: guild names in both vocabularies, traits and the four lenses for the biosphere view.
- Minerva (story names) and the UI's life panel read `plain_name`, `traits` and `diversity`.
- Shared files touched: `crates/sse-wasm` (engine `life-biosphere`) and the harness lists.

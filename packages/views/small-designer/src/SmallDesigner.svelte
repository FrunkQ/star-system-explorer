<!--
  The small-body designer (RW16a, real catalogues design 6A.1). One screen over the kernel's small-body recipes: pick a
  kind, set its size and the handful of things that drive its look, and see it turn in True as the numbers change.
  Every control is a pin on the one resolve (`make_small_body`), so what turns is what the physics makes of the pins;
  "Let it decide" leaves a pin to the field. An implausible pin is never refused: the panel says what it would take.
  On every desk it is a design to look at; nothing is kept in the campaign (placing a design is the editing track's Add).
-->
<script lang="ts">
  import { untrack } from "svelte";
  import { constant, say, sayNumber, type Json, type SystemScene, type ViewContext } from "../../../ui-contract/src/index.ts";
  import { smallBodyColour, smallBodyMesh, trueTurn } from "../../../renderer/src/index.ts";
  import { DEFAULT_DESIGN, KINDS, MAKEUPS, designAsk, logPos, logVal, type Design } from "./design.ts";

  let { ctx, scene }: { ctx: ViewContext; scene: SystemScene } = $props();

  const depth = $derived(ctx.workspace.depth ?? "science");
  let d = $state<Design>({ ...DEFAULT_DESIGN });

  // Size: 10 m to 1,000 km, on a log slider.
  const D_LO = 0.01;
  const D_HI = 1000;
  // Spin: 0.2 h to 1,000 h.
  const S_LO = 0.2;
  const S_HI = 1000;

  type Body = {
    diameter_km: number;
    mass_kg: number;
    bulk_density_kg_m3: number;
    spin_hours: number;
    barrier_hours: number;
    type_label: string;
    type_why: string;
    structure: { kind: string };
    pair: { kind: string; origin?: string | null; size_ratio: number; separation_m: number };
    figure: { kind: string; axes_m: [number, number, number]; lobes: { axes_m: [number, number, number]; offset_m: number }[]; relief: number };
    albedo: number;
    weathering: number;
    makeup: { metal: number; rock: number; carbon: number; ice: number };
    history: { says: string }[];
    takes?: { about: string; says: string; needs_pa: number; holds_pa: number }[];
    activity?: { active: boolean; dominant?: string | null; gas_kg_s: number; coma_km: number; dust_tail_km: number; ion_tail_km: number; r_au: number };
    made_from?: string;
  };
  let body = $state<Body | null>(null);
  let problem = $state<string | null>(null);
  let asking = 0;

  // Ask the kernel whenever the design changes; a later answer wins.
  $effect(() => {
    const ask = designAsk(scene.system, $state.snapshot(d) as Design);
    const n = ++asking;
    untrack(() =>
      void ctx.session.makeSmallBody(ask).then((r) => {
        if (n !== asking) return;
        if (r.ok) ((body = r.body as unknown as Body), (problem = null));
        else problem = r.message;
      }),
    );
  });

  // The turning picture: a strip of one turn, drawn again (after a short pause) whenever the body changes.
  const TURN = { size: 280, frames: 48 };
  let strip = $state<{ sheet: HTMLCanvasElement; frames: number; size: number } | null>(null);
  let drawing = $state(false);
  let noGpu = $state(false);
  let drawFor = 0;
  let pause: ReturnType<typeof setTimeout> | null = null;
  $effect(() => {
    const b = body;
    if (!b) return;
    if (pause) clearTimeout(pause);
    const n = ++drawFor;
    const key = `${scene.system}/small:design-${d.variant}`;
    pause = setTimeout(() => {
      drawing = true;
      const figure = { ...b.figure, albedo: b.albedo, weathering: b.weathering };
      // The whole lump in the square, however long: its farthest point from the middle at 0.45 of the square.
      const m = smallBodyMesh(figure, key);
      let far = 1;
      for (let i = 0; i < m.positions.length; i += 3) far = Math.max(far, Math.hypot(m.positions[i]!, m.positions[i + 1]!, m.positions[i + 2]!));
      void trueTurn({ key, star: false, colour: smallBodyColour(figure), radius_m: (b.diameter_km * 1000) / 2, figure }, { size: TURN.size, frames: TURN.frames, tilt_rad: 0.35, passes: 2, fill: 0.45 / far }).then((t) => {
        if (n !== drawFor) return;
        drawing = false;
        if (!t || !t.lit) return void (noGpu = !t);
        strip = { sheet: t.sheet, frames: t.frames, size: t.size };
      });
    }, 220);
  });

  // Step through the strip: one turn every few seconds, or still under reduced motion.
  let canvas = $state<HTMLCanvasElement | null>(null);
  const reduced = typeof matchMedia === "function" && matchMedia("(prefers-reduced-motion: reduce)").matches;
  $effect(() => {
    const c = canvas;
    const s = strip;
    if (!c || !s) return;
    const g = c.getContext("2d");
    if (!g) return;
    let raf = 0;
    const t0 = performance.now();
    const paint = (now: number) => {
      const k = reduced ? 0 : Math.floor(((now - t0) / 6000) * s.frames) % s.frames;
      g.clearRect(0, 0, c.width, c.height);
      g.drawImage(s.sheet, k * s.size, 0, s.size, s.size, 0, 0, c.width, c.height);
      if (!reduced) raf = requestAnimationFrame(paint);
    };
    raf = requestAnimationFrame(paint);
    return () => cancelAnimationFrame(raf);
  });

  const lengthSaid = (km: number) => say(km * 1000, "length", { class: "small", what: "size" }, { depth }).text;
  const massSaid = (kg: number) => say(kg, "mass", { class: "small", what: "mass" }, { depth }).text;
  const timeSaid = (h: number) => say(h * 3600, "time", { class: "small", what: "day" }, { depth }).text;
  const densitySaid = (r: number) => say(r, "density", { class: "small" }, { depth }).text;
  const pct = (x: number) => `${sayNumber(100 * x, 2, { upTo: 0 })}%`;

  const STRUCTURES = [
    { id: null, words: "Let it decide" },
    { id: "rubble-pile", words: "Rubble pile" },
    { id: "monolith", words: "Solid" },
    { id: "relaxed", words: "Pulled round" },
  ] as const;
  const STRUCTURE_WORDS: Record<string, string> = { "rubble-pile": "a rubble pile", monolith: "solid", relaxed: "pulled round by its own gravity", "stripped-core": "a bared core", "ring-aggregate": "a loose clump" };
  const ICE_WORDS: Record<string, string> = { h2o: "water ice", co2: "carbon dioxide ice", co: "carbon monoxide ice" };
  const PAIR_WORDS: Record<string, string> = { "yorp-fission": "spun apart by sunlight", "gentle-merger": "a gentle merger in a cold disc", "impact-moon": "a moon thrown up by an impact" };
  const pairs = $derived(d.kind === "contact-binary" || d.kind === "binary-pair");
  const comet = $derived(d.kind === "comet-nucleus");

  const set = (patch: Partial<Design>) => (d = { ...d, ...patch });
  const shares = $derived(d.makeup ?? (body ? [body.makeup.metal, body.makeup.rock, body.makeup.carbon, body.makeup.ice] : [0, 1, 0, 0]));
  function share(i: number, v: number) {
    const s = [...shares] as [number, number, number, number];
    s[i] = v;
    set({ makeup: s });
  }
  const PARTS = ["Metal", "Rock", "Carbon", "Ice"];
  const sum = $derived(shares.reduce((a, b) => a + b, 0) || 1);
</script>

<section class="designer" data-small-designer aria-label="Small-body designer">
  <header>
    <h2>Design a small body</h2>
    <p class="lead">Asteroids, comet nuclei, contact binaries and pairs, at any size. Each choice goes into the same physics that makes the bodies in a belt, so what turns here is what those choices would make.</p>
  </header>

  <div class="kinds" role="radiogroup" aria-label="Kind">
    {#each KINDS as k}
      <button type="button" role="radio" aria-checked={d.kind === k.id} class:on={d.kind === k.id} title={k.hint} onclick={() => set({ kind: k.id, ...(k.id === "comet-nucleus" ? { r_au: Math.max(0.3, Math.min(d.r_au, 1.2)), diameter_km: Math.max(d.diameter_km, 1) } : {}) })}>{k.words}</button>
    {/each}
  </div>

  <div class="stage">
    <figure class="turn" aria-label="The body turning, drawn in True" data-turn>
      <canvas bind:this={canvas} width={TURN.size} height={TURN.size}></canvas>
      {#if noGpu}<figcaption>This device has no GPU for True, so the body is described in words only.</figcaption>
      {:else if drawing && !strip}<figcaption>Drawing…</figcaption>
      {:else}<figcaption>Drawn in True from its figure, lit from the upper left{drawing ? " · redrawing" : ""}.</figcaption>{/if}
      <button type="button" class="again" title="Another body made the same way: a new address, a new lump" onclick={() => set({ variant: d.variant + 1 })}>Another like it</button>
    </figure>

    <div class="controls">
      <label class="row">
        <span>Size</span>
        <input type="range" min="0" max="1" step="0.001" value={logPos(d.diameter_km, D_LO, D_HI)} oninput={(e) => set({ diameter_km: logVal(Number(e.currentTarget.value), D_LO, D_HI) })} aria-valuetext={lengthSaid(d.diameter_km)} />
        <output>{lengthSaid(d.diameter_km)} across</output>
      </label>

      <fieldset>
        <legend>Made of</legend>
        <div class="chips">
          <button type="button" class:on={d.makeup === null} onclick={() => set({ makeup: null })} title="What this system has at that distance">Let it decide</button>
          {#each MAKEUPS as m}
            <button type="button" class:on={d.makeup !== null && d.makeup.every((x, i) => x === m.shares[i])} onclick={() => set({ makeup: [...m.shares] as [number, number, number, number] })}>{m.words}</button>
          {/each}
        </div>
        <details>
          <summary>Fine shares</summary>
          {#each PARTS as p, i}
            <label class="row small">
              <span>{p}</span>
              <input type="range" min="0" max="1" step="0.01" value={shares[i]} oninput={(e) => share(i, Number(e.currentTarget.value))} />
              <output>{pct(shares[i]! / sum)}</output>
            </label>
          {/each}
        </details>
      </fieldset>

      <fieldset>
        <legend>Built as</legend>
        <div class="chips">
          {#each STRUCTURES as s}
            <button type="button" class:on={d.structure === s.id} onclick={() => set({ structure: s.id })}>{s.words}</button>
          {/each}
        </div>
      </fieldset>

      <label class="row">
        <span>Battered</span>
        <input type="range" min="0" max="1" step="0.01" value={d.battered ?? 0.5} oninput={(e) => set({ battered: Number(e.currentTarget.value) })} />
        <output>{d.battered === null ? "as it came" : d.battered < 0.34 ? "smooth" : d.battered < 0.67 ? "cratered" : "battered"}</output>
      </label>

      <label class="row">
        <span>Spin</span>
        <input type="range" min="0" max="1" step="0.001" value={logPos(d.spin_hours ?? body?.spin_hours ?? 6, S_LO, S_HI)} oninput={(e) => set({ spin_hours: logVal(Number(e.currentTarget.value), S_LO, S_HI) })} />
        <output>once in {timeSaid(d.spin_hours ?? body?.spin_hours ?? 6)}</output>
      </label>
      {#if d.spin_hours !== null || d.battered !== null}
        <button type="button" class="link" onclick={() => set({ spin_hours: null, battered: null })}>Let sunlight and collisions set the spin and scars</button>
      {/if}

      {#if pairs}
        <label class="row">
          <span>{d.kind === "contact-binary" ? "Second lobe" : "Its moon"}</span>
          <input type="range" min="0.05" max="1" step="0.01" value={d.size_ratio ?? body?.pair.size_ratio ?? 0.6} oninput={(e) => set({ size_ratio: Number(e.currentTarget.value) })} />
          <output>{pct(d.size_ratio ?? body?.pair.size_ratio ?? 0.6)} of the first, across</output>
        </label>
      {/if}

      <label class="row">
        <span>{comet ? "Closest to its star" : "From its star"}</span>
        <input type="range" min="0" max="1" step="0.001" value={logPos(d.r_au, 0.1, 100)} oninput={(e) => set({ r_au: logVal(Number(e.currentTarget.value), 0.1, 100) })} />
        <output>{say(d.r_au * constant("au"), "length", { class: "system", what: "orbit" }, { depth }).text}</output>
      </label>
    </div>
  </div>

  {#if problem}<p class="problem" role="alert">{problem}</p>{/if}

  {#if body}
    <div class="read" data-small-read>
      <p class="says">
        <strong>{lengthSaid(body.diameter_km)} across</strong>, {massSaid(body.mass_kg)}, {STRUCTURE_WORDS[body.structure.kind] ?? body.structure.kind}{#if body.pair.kind === "contact"}, two lobes touching{:else if body.pair.kind === "separated"}, with a moon {pct(body.pair.size_ratio)} its size{/if}.
        It turns once in {timeSaid(body.spin_hours)}{#if depth !== "curious"}; a rubble pile this dense flies apart faster than once in {timeSaid(body.barrier_hours)}{/if}.
      </p>
      <p>Type {body.type_label}: {body.type_why}.{#if depth !== "curious"} Bulk density {densitySaid(body.bulk_density_kg_m3)}, reflects {pct(body.albedo)} of the light.{/if}</p>
      {#if body.pair.origin}<p>Its pair: {PAIR_WORDS[body.pair.origin] ?? body.pair.origin}.</p>{/if}
      {#if body.activity}
        <p data-activity>{#if body.activity.active}At its closest it is awake: {ICE_WORDS[body.activity.dominant ?? ""] ?? "its ice"} boils off at {massSaid(body.activity.gas_kg_s)} a second, with a coma {lengthSaid(body.activity.coma_km)} across and a dust tail {lengthSaid(body.activity.dust_tail_km)} long.{:else}At its closest it stays asleep: too far from its star for its ice to boil.{/if}</p>
      {/if}
      {#if body.takes?.length}
        <div class="takes" data-takes>
          <h3>What it would take</h3>
          {#each body.takes as t}<p>{t.says}</p>{/each}
        </div>
      {/if}
      {#if body.history.length}
        <details>
          <summary>Its story</summary>
          {#each body.history as h}<p>{h.says}</p>{/each}
        </details>
      {/if}
      {#if body.made_from === "designed"}<p class="quiet">This system has no formation record, so the body is made against a Sun-like star with a main belt's history.</p>{/if}
    </div>
  {/if}
</section>

<style>
  .designer {
    display: grid;
    gap: var(--sse-gap, 12px);
    padding: var(--sse-pad, 16px);
    color: var(--sse-ink);
    max-width: 980px;
    min-width: 0;
    box-sizing: border-box;
    overflow-wrap: anywhere;
  }
  .stage > *,
  .controls,
  .row {
    min-width: 0;
  }
  .row input {
    width: 100%;
    min-width: 0;
  }
  h2 {
    margin: 0;
    font: var(--sse-font-title, 600 1.15rem/1.3 system-ui);
  }
  h3 {
    margin: 0 0 4px;
    font-size: 0.95rem;
  }
  .lead,
  .quiet {
    color: var(--sse-ink-muted);
    margin: 4px 0 0;
  }
  .kinds,
  .chips {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
  }
  button {
    font: inherit;
    color: inherit;
    background: var(--sse-ground);
    border: var(--sse-line, 1px) solid var(--sse-ink-faint);
    border-radius: 999px;
    padding: 4px 12px;
    cursor: pointer;
    transition: transform 120ms ease-out, border-color 120ms ease-out;
  }
  button:hover {
    border-color: var(--sse-ink);
    transform: translateY(-1px);
  }
  button.on,
  button[aria-checked="true"] {
    border-color: var(--sse-accent);
    box-shadow: inset 0 0 0 1px var(--sse-accent);
  }
  button.link {
    border: none;
    padding: 0;
    text-decoration: underline;
    color: var(--sse-ink-muted);
    justify-self: start;
  }
  .stage {
    display: grid;
    grid-template-columns: minmax(200px, 300px) 1fr;
    gap: 16px;
    align-items: start;
  }
  .turn {
    margin: 0;
    display: grid;
    gap: 6px;
    justify-items: center;
  }
  .turn canvas {
    width: 100%;
    max-width: 280px;
    aspect-ratio: 1;
    background: var(--sse-ground-raised);
    border-radius: 6px;
  }
  figcaption {
    font-size: 0.85rem;
    color: var(--sse-ink-muted);
    text-align: center;
  }
  .controls {
    display: grid;
    gap: 10px;
  }
  fieldset {
    border: none;
    padding: 0;
    margin: 0;
    display: grid;
    gap: 6px;
  }
  legend {
    font-weight: 600;
    margin-bottom: 4px;
  }
  .row {
    display: grid;
    grid-template-columns: 8.5em 1fr minmax(8em, auto);
    gap: 8px;
    align-items: center;
  }
  .row.small {
    grid-template-columns: 5em 1fr 4em;
    font-size: 0.9rem;
  }
  .row > span {
    font-weight: 600;
  }
  output {
    color: var(--sse-ink-muted);
    font-variant-numeric: tabular-nums;
  }
  .read p {
    margin: 4px 0;
  }
  .takes {
    border-left: 3px solid var(--sse-accent);
    padding: 4px 0 4px 10px;
    margin: 8px 0;
  }
  .problem {
    color: var(--sse-accent);
  }
  @media (max-width: 640px) {
    .stage {
      grid-template-columns: 1fr;
    }
    .row {
      grid-template-columns: 6.5em minmax(0, 1fr);
    }
    .row > output {
      grid-column: 2;
    }
  }
  @media (prefers-reduced-motion: reduce) {
    button {
      transition: none;
    }
  }
</style>

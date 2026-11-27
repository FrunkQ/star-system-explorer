<!--
  The ship builder (CM14a; compound ships design 8A.4, rulings 14A.2 to 14A.4, and Alex's two modes of 27 Nov 11:47):
  palette parts clipped at their connection points, stretched and widened, in plans and a freely turning 3D model over
  a fixed blueprint that builds up behind it. Two modes on one canvas: "Build it", where what the ship can do shows as
  each part goes on, and "Build to a plan", where a ship's basic capabilities set budgets that count down. It opens on a
  catalogue ship turned into parts, or on a saved build; it keeps a recipe, never a drawing. The sums are the parts'
  own figures (renderer ship-build.ts); every judgement of what it can do is the kernel's (`ship_at_world`).
-->
<script lang="ts">
  import { untrack } from "svelte";
  import { budgets, buildLayout, driveFacts, freePoints, fromLayout, designShip, readPropellants, shipViews, startShip, type BuildRecipe, type BuiltPart, type DesignRules, type DesignShip, type FreePoint, type Propellants, type ShipStyle, type ShipView, type ShipView3D } from "../../../renderer/src/index.ts";
  import { eraOf, paletteByEra, sayNumber, verdictWords, type NavShip, type Part, type PartsPalette, type ShipAtWorld, type ViewContext } from "../../../ui-contract/src/index.ts";

  let {
    ctx,
    ship = null,
    saved = null,
    onkeep,
    readonly = false,
    start = "plans",
  }: {
    ctx: ViewContext;
    /** The ship to open as parts when nothing is saved: a catalogue ship, or a construct's record. */
    ship?: DesignShip | null;
    /** A build kept before. */
    saved?: BuildRecipe | null;
    /** Keep the build with its ship (offered on the Build desk). */
    onkeep?: (r: BuildRecipe) => void;
    /** Draw only: a card off its edit desk. */
    readonly?: boolean;
    /** Open on the plans or the 3D model. */
    start?: "plans" | "model";
  } = $props();

  type Pack = { rules: DesignRules; styles: Record<string, ShipStyle> };
  let pack = $state<Pack | null>(null);
  let palette = $state<PartsPalette | null>(null);
  let missing = $state<string | null>(null);
  // Packs served beside the page (scripts/build-shell.sh).
  const SHIPS_PACK = "packs/base-ships.json";
  const PARTS_PACK = "packs/base-parts.json";
  const read = (path: string) => fetch(new URL(path, document.baseURI)).then((r) => r.json());
  $effect(() => {
    void Promise.all([read(SHIPS_PACK), read(PARTS_PACK)])
      .then(([s, p]: [{ provides: { other: { ship_design: DesignRules; ship_styles: Record<string, ShipStyle> } } }, { provides: { other: { palette: PartsPalette } } }]) => {
        pack = { rules: s.provides.other.ship_design, styles: s.provides.other.ship_styles };
        palette = p.provides.other.palette;
      })
      .catch((e) => (missing = `The parts could not be read (${e instanceof Error ? e.message : String(e)}).`));
  });

  const blank = (name: string): BuildRecipe => ({ format: "sse.build/1", name, parts: [] });
  let recipe = $state<BuildRecipe>(untrack(() => saved ?? blank(ship?.name ?? "New ship")));
  let opened = $state<string | null>(untrack(() => (saved ? "saved" : null)));
  let notes = $state<string[]>([]);
  let fuels = $state<Propellants>({});

  // Open the ship as parts once the palette is here (convert on open); a saved build wins.
  $effect(() => {
    const s = ship;
    if (!s || !pack || !palette || saved) return;
    const key = `${s.name}|${s.wet_t}|${s.dry_t}`;
    if (untrack(() => opened) === key) return;
    const style = Object.keys(pack.styles)[0]!;
    const old = designShip(s, untrack(() => fuels), pack.rules, style, pack.styles[style]!, 1);
    const got = fromLayout(old, s, palette);
    untrack(() => ((recipe = got.recipe), (notes = got.notes), (opened = key), (selected = (got.recipe.parts.find((b) => palette!.parts.find((x) => x.id === b.part)?.scale.stretch) ?? got.recipe.parts[0])?.key ?? null)));
  });
  $effect(() => {
    const f = recipe.fuels ?? [];
    void readPropellants(ctx.session, f).then((x) => (fuels = x)).catch(() => (fuels = {}));
  });

  let mode = $state<"adhoc" | "plan">(untrack(() => (saved?.plan ? "plan" : "adhoc")));
  let year = $state(2025);
  let view = $state<"plans" | "model">(untrack(() => start));
  let selected = $state<string | null>(null);
  let target = $state<number>(0);

  const W = 560;
  const H = 220;
  const style = $derived(pack ? (pack.styles.spine ?? Object.values(pack.styles)[0]!) : null);
  const layout = $derived(pack && palette ? buildLayout(recipe, palette, fuels, pack.rules) : null);
  const views = $derived(layout && layout.pieces.length ? shipViews(layout, [W, H]) : null);
  const free = $derived(layout ? freePoints(recipe, layout.placed) : []);
  // Clip targets: the chosen part's free points first.
  const targets = $derived([...free.filter((f) => f.key === selected), ...free.filter((f) => f.key !== selected)]);
  const groups = $derived(palette ? paletteByEra(palette, year) : []);
  const eraNow = $derived(palette?.eras?.find((e) => (e.from === undefined || year >= e.from) && (e.to === undefined || year <= e.to)));
  const chosen = $derived(layout?.placed.find((q) => q.key === selected) ?? null);
  const facts = $derived(layout ? driveFacts(layout.record) : null);
  const plan = $derived(recipe.plan ?? ship ?? null);
  const bars = $derived(mode === "plan" && plan && layout && pack ? budgets({ ...plan, ...(recipe.crew !== undefined ? { crew: recipe.crew } : {}), ...(recipe.cargo_t !== undefined ? { cargo_t: recipe.cargo_t } : {}) }, layout.totals, fuels, pack.rules) : []);

  let n = 0;
  const keyFor = (id: string) => {
    let k: string;
    do k = `${id.replace(/[^a-z0-9]+/gi, "-")}-${++n}`;
    while (recipe.parts.some((b) => b.key === k));
    return k;
  };
  function clip(p: Part) {
    if (readonly) return;
    const t: FreePoint | undefined = targets[Math.min(target, targets.length - 1)];
    const b: BuiltPart = { key: keyFor(p.id), part: p.id };
    if (recipe.parts.length && t) Object.assign(b, { on: t.key, place: t.place }, t.place === "hull" ? { along: t.along ?? 0.5, clock_deg: t.clock_deg ?? 0 } : {});
    else if (recipe.parts.length) return void (notes = [`Nowhere free to clip ${p.name}: every point is taken.`]);
    recipe = { ...recipe, parts: [...recipe.parts, b], ...(p.kind === "tank" && !recipe.fuels?.length && plan?.fuels ? { fuels: [...plan.fuels] } : {}) };
    report = null;
    // The part stays picked after clipping (the mock-up's default): the next part clips to it.
    selected = b.key;
    target = 0;
  }
  function edit(key: string, f: (b: BuiltPart, part: Part) => BuiltPart) {
    if (readonly || !palette) return;
    recipe = { ...recipe, parts: recipe.parts.map((b) => { const p = palette!.parts.find((x) => x.id === b.part); return b.key === key && p ? f(b, p) : b; }) };
    report = null;
  }
  // Stretch and widen by the part's own law; real hardware says it comes in its sizes (stretchPart's note).
  function resize(key: string, dl: number, dr: number) {
    edit(key, (b, p) => ({ ...b, ...(dl !== 1 ? { length_m: Math.max(0.2, (b.length_m ?? p.shape.length_m) * dl) } : {}), ...(dr !== 1 ? { radius_m: Math.max(0.05, (b.radius_m ?? p.shape.radius_m) * dr) } : {}) }));
  }
  function turn(key: string, by: number) {
    edit(key, (b) => (b.place === "hull" ? { ...b, clock_deg: (((b.clock_deg ?? 0) + by) % 360 + 360) % 360 } : { ...b, turn_deg: ((b.turn_deg ?? 0) + by) % 360 }));
  }
  function remove(key: string) {
    if (readonly) return;
    // A part comes off with everything clipped to it.
    const gone = new Set([key]);
    for (let grew = true; grew; ) {
      grew = false;
      for (const b of recipe.parts) if (b.on && gone.has(b.on) && !gone.has(b.key)) (gone.add(b.key), (grew = true));
    }
    recipe = { ...recipe, parts: recipe.parts.filter((b) => !gone.has(b.key)) };
    selected = recipe.parts[0]?.key ?? null;
    report = null;
  }
  function wheel(e: WheelEvent) {
    if (!selected || readonly) return;
    e.preventDefault();
    const k = Math.exp(-e.deltaY * 0.0015);
    if (e.shiftKey) resize(selected, 1, k);
    else resize(selected, k, 1);
  }

  // "What can it do?": the kernel judges the build as it stands at a few worlds; nothing is refused.
  const WORLDS = ["Earth", "Mars", "Venus"];
  let report = $state<{ ship: NavShip; worlds: ShipAtWorld[] } | null>(null);
  let asking = $state(false);
  let askFailed = $state<string | null>(null);
  async function whatCanItDo() {
    if (!layout) return;
    asking = true;
    askFailed = null;
    const r = await ctx.session.shipAtWorld({ fixture: "sol", ship: layout.record as unknown as NavShip, worlds: WORLDS });
    asking = false;
    if (r.ok) report = { ship: r.report.ship, worlds: r.report.worlds };
    else askFailed = r.message;
  }

  // The 3D model over the fixed blueprint: the model turns, the profiles behind it do not (ruling 14A.2).
  let canvas = $state<HTMLCanvasElement | null>(null);
  let model: ShipView3D | null = null;
  let yaw = $state(35);
  let pitch = $state(25);
  let zoom = $state(1);
  let noGpu = $state(false);
  $effect(() => {
    if (view !== "model" || !canvas || !style?.skin || !layout || !layout.pieces.length) return;
    const c = canvas;
    const l = layout;
    const skin = style.skin;
    let gone = false;
    void startShip(c, l, skin, { segments: 24, seed: 1, clear: true }).then((m) => {
      if (gone) return m?.dispose();
      model = m;
      noGpu = !m;
      redraw();
    });
    return () => {
      gone = true;
      model?.dispose();
      model = null;
    };
  });
  function redraw() {
    model?.draw({ yaw_deg: yaw, pitch_deg: pitch, zoom, width: W * 2, height: H * 2, ground: "transparent" });
  }
  $effect(() => {
    void [yaw, pitch, zoom];
    redraw();
  });
  let drag: { x: number; y: number } | null = null;
  function down(e: PointerEvent) {
    drag = { x: e.clientX, y: e.clientY };
    (e.currentTarget as Element).setPointerCapture(e.pointerId);
  }
  function move(e: PointerEvent) {
    if (!drag) return;
    yaw -= (e.clientX - drag.x) * 0.4;
    pitch = Math.max(-85, Math.min(85, pitch + (e.clientY - drag.y) * 0.4));
    drag = { x: e.clientX, y: e.clientY };
  }
  function modelWheel(e: WheelEvent) {
    // With a part picked the wheel stretches it; with none it zooms.
    if (selected && !readonly) return wheel(e);
    e.preventDefault();
    zoom = Math.max(0.3, Math.min(40, zoom * Math.exp(-e.deltaY * 0.0015)));
  }

  const t = (v: number) => (v >= 100 ? `${Math.round(v)} t` : `${sayNumber(v, 0, { decimals: v >= 10 ? 1 : 2 })} t`);
  const m3 = (v: number) => `${sayNumber(v, 0, { decimals: v >= 100 ? 0 : 1 })} m³`;
  const kms = (v: number) => `${sayNumber(v / 1000, 0, { decimals: 2 })} km/s`;
  const share = (b: { need: number; built: number }) => (b.need > 0 ? Math.max(0, Math.min(1, b.built / b.need)) : 1);
</script>

{#snippet plan2d(v: ShipView, title: string, cls = "")}
  <figure class={cls}>
    <figcaption>{title}</figcaption>
    <svg viewBox="0 0 {W} {H}" width="100%" role="img" aria-label="{title} of {recipe.name}">
      {#each v.paths as p, i (i)}
        <path class="{p.system}{p.member === selected ? ' picked' : ''}" d={p.d} data-part-key={p.member} onclick={() => p.member && !readonly && (selected = p.member)} role="presentation" />
      {/each}
    </svg>
  </figure>
{/snippet}

<section class="builder" data-builder aria-label="Ship builder">
  {#if missing}<p class="note">{missing}</p>{/if}
  {#if !readonly}
    <div class="bar">
      <div class="modes" role="group" aria-label="How to build">
        <button type="button" aria-pressed={mode === "adhoc"} onclick={() => (mode = "adhoc")} data-mode="adhoc">Build it</button>
        <button type="button" aria-pressed={mode === "plan"} onclick={() => (mode = "plan")} data-mode="plan" disabled={!plan} title={plan ? `Budgets from ${plan.name}` : "Open a catalogue ship to plan from it"}>Build to a plan</button>
      </div>
      <div class="modes" role="group" aria-label="View">
        <button type="button" aria-pressed={view === "plans"} onclick={() => (view = "plans")} data-show-plans>Plans</button>
        <button type="button" aria-pressed={view === "model"} onclick={() => (view = "model")} data-show-model>3D</button>
      </div>
      <button type="button" onclick={whatCanItDo} disabled={asking || !layout?.pieces.length} data-what-can-it-do>{asking ? "Asking..." : "What can it do?"}</button>
      {#if onkeep}<button type="button" onclick={() => onkeep?.($state.snapshot(recipe) as BuildRecipe)} disabled={!recipe.parts.length} data-keep-build>Keep with the ship</button>{/if}
    </div>
  {/if}

  <div class="canvas" onwheel={wheel} role="presentation">
    {#if views}
      {#if view === "model"}
        <!-- The blueprint stays put behind the model while it turns. -->
        <div class="stack">
          <div class="blueprint" aria-hidden="true">{@render plan2d(views.side, "Side", "faint")}{@render plan2d(views.top, "From above", "faint")}</div>
          <canvas bind:this={canvas} width={W * 2} height={H * 2} data-ship-model onpointerdown={down} onpointermove={move} onpointerup={() => (drag = null)} onpointercancel={() => (drag = null)} onwheel={modelWheel}></canvas>
        </div>
        {#if noGpu}<p class="note">This device cannot draw 3D; the plans are under the model.</p>{/if}
      {:else}
        <div class="pair">{@render plan2d(views.side, "Side")}{@render plan2d(views.top, "From above")}</div>
      {/if}
    {:else if layout}
      <p class="note">Nothing yet: pick a part below to start.</p>
    {/if}
  </div>

  {#if layout && facts}
    <dl class="totals" data-totals>
      <div><dt>Mass</dt><dd>{t(layout.record.wet_t)} full, {t(layout.record.dry_t)} empty</dd></div>
      <div><dt>Δv</dt><dd data-dv>{kms(facts.delta_v_m_s)}</dd></div>
      <div><dt>Push</dt><dd>{sayNumber(layout.record.accel_g, 0, { decimals: 3 })} g full</dd></div>
      <div><dt>Tanks</dt><dd>{m3(Object.values(layout.totals.tank_m3).reduce((a, v) => a + v, 0))}</dd></div>
      <div><dt>Room aboard</dt><dd>{m3(layout.totals.habitat_m3)}</dd></div>
      <div><dt>Size</dt><dd>{sayNumber(layout.length_m, 0, { decimals: 1 })} m long</dd></div>
    </dl>
  {/if}

  {#if bars.length}
    <ul class="budgets" data-budgets aria-label="Budgets from {plan?.name}">
      {#each bars as b (b.id)}
        <li data-budget={b.id} class:over={b.left < 0}>
          <span>{b.label}</span>
          <span class="track"><span class="fill" style="width: {share(b) * 100}%"></span></span>
          <span data-left={b.left}>{b.left < 0 ? `${sayNumber(-b.left, 0, { decimals: 0 })} ${b.unit} over` : `${sayNumber(b.left, 0, { decimals: 0 })} ${b.unit} to go`}</span>
        </li>
      {/each}
    </ul>
  {/if}

  {#if report}
    <ul class="verdicts" data-can-do>
      {#each report.worlds as w (w.body)}
        {@const can = w.checks.filter((c) => c.verdict !== "not-here")}
        {#if can.length}
          {#each can as c (c.id)}<li class={c.verdict}>{w.name}, {c.words.toLowerCase()}: {verdictWords(c.verdict)}</li>{/each}
        {:else}
          <li class="not-here">{w.name}: nothing it can do there as built{w.checks[0]?.why?.[0] ? ` (${w.checks[0].why[0]})` : ""}</li>
        {/if}
      {/each}
    </ul>
  {/if}
  {#if askFailed}<p class="note" data-ask-failed>{askFailed}</p>{/if}

  {#if !readonly && layout}
    {#if chosen}
      <div class="picked-bar" data-picked={chosen.key}>
        <strong>{chosen.part.name}</strong>
        <button type="button" onclick={() => resize(chosen.key, 1.25, 1)} disabled={!chosen.part.scale.stretch} data-longer>Longer</button>
        <button type="button" onclick={() => resize(chosen.key, 0.8, 1)} disabled={!chosen.part.scale.stretch} data-shorter>Shorter</button>
        <button type="button" onclick={() => resize(chosen.key, 1, 1.25)} disabled={!chosen.part.scale.widen} data-wider>Wider</button>
        <button type="button" onclick={() => resize(chosen.key, 1, 0.8)} disabled={!chosen.part.scale.widen} data-narrower>Narrower</button>
        {#if chosen.built.on}
          <button type="button" onclick={() => turn(chosen.key, 45)} data-turn>Turn</button>
          <button type="button" onclick={() => remove(chosen.key)} data-remove>Take off</button>
        {/if}
        <span class="hint">{chosen.part.scale.stretch || chosen.part.scale.widen ? "Wheel stretches, Shift-wheel widens" : "Comes in the sizes its pack offers"}</span>
      </div>
    {/if}
    {#if targets.length && recipe.parts.length}
      <label class="target">Clip the next part
        <select bind:value={target} data-clip-target>
          {#each targets as f, i (i)}<option value={i}>{f.label}</option>{/each}
        </select>
      </label>
    {/if}
    <div class="palette" data-palette>
      <label class="year">Parts flown by <input type="number" min="1940" max="2400" step="1" bind:value={year} data-year /> {#if eraNow}<span class="era">{eraNow.name}{eraNow.future ? " (future)" : ""}</span>{/if}</label>
      {#each groups as g (g.label)}
        <details open={g.label === "Shapes"}>
          <summary>{g.label}{g.era ? ` (${g.era.about})` : ""}</summary>
          <div class="parts">
            {#each g.parts as p (p.id)}
              <button type="button" onclick={() => clip(p)} data-palette-part={p.id} title={p.year ? `First flew ${p.year}` : p.advance ? `Figures from the tech advance ${p.advance}` : "A shape: geometry only"}>{p.name}</button>
            {/each}
          </div>
        </details>
      {/each}
    </div>
  {/if}
  {#if layout && (notes.length || layout.notes.length)}
    <ul class="notes">{#each [...notes, ...layout.notes] as w, i (i)}<li>{w}</li>{/each}</ul>
  {/if}
</section>

<style>
  .builder { display: grid; gap: 0.5em; min-width: 0; }
  .bar, .picked-bar { display: flex; gap: 0.5em; align-items: center; flex-wrap: wrap; }
  .modes { display: inline-flex; gap: 0; }
  .modes button[aria-pressed="true"] { background: var(--sse-accent); color: var(--sse-ground); }
  figure { margin: 0; }
  figcaption { color: var(--sse-ink-muted); font-size: 0.8em; }
  svg { display: block; height: auto; background: var(--sse-chart-ground); }
  path { stroke: var(--sse-ink); stroke-width: 1; fill: var(--sse-ground-raised); stroke-linejoin: round; cursor: pointer; }
  path.picked { stroke: var(--sse-accent); stroke-width: 2; }
  .spine { fill: var(--sse-ink-faint); }
  .radiators { fill: var(--sse-zone); fill-opacity: 0.35; stroke: var(--sse-zone-ink); }
  .wings { fill-opacity: 0.6; }
  .stack { position: relative; background: var(--sse-chart-ground); }
  .pair { display: grid; grid-template-columns: 1fr 1fr; gap: 0.5em; }
  .blueprint { position: absolute; inset: 0; display: flex; opacity: 0.25; pointer-events: none; overflow: hidden; }
  .blueprint figure { flex: 1 1 0; min-width: 0; }
  .blueprint svg { background: none; }
  canvas { position: relative; display: block; width: 100%; height: auto; aspect-ratio: 560 / 220; touch-action: none; }
  .totals { display: flex; flex-wrap: wrap; gap: 0.25em 1.25em; margin: 0; font-size: 0.9em; }
  .totals div { display: flex; gap: 0.4em; }
  .totals dt { color: var(--sse-ink-muted); }
  .totals dd { margin: 0; }
  .budgets { list-style: none; margin: 0; padding: 0; display: grid; gap: 0.25em; font-size: 0.85em; }
  .budgets li { display: grid; grid-template-columns: 12em 1fr 9em; gap: 0.5em; align-items: center; }
  .track { height: 0.5em; background: var(--sse-ground-raised); border-radius: 0.25em; overflow: hidden; }
  .fill { display: block; height: 100%; background: var(--sse-accent); }
  .over .fill { background: var(--sse-alert); }
  .over { color: var(--sse-alert); }
  .palette { display: grid; gap: 0.25em; }
  .parts { display: flex; flex-wrap: wrap; gap: 0.25em; }
  .parts button { font-size: 0.85em; }
  .era, .hint, .note, .notes { color: var(--sse-ink-muted); font-size: 0.85em; }
  .notes { margin: 0; padding-left: 1.2em; }
  .verdicts { margin: 0; padding-left: 1.2em; font-size: 0.85em; }
  .verdicts .dangerous { color: var(--sse-alert); }
  .year input { width: 5em; }
  @media (max-width: 700px) { .budgets li { grid-template-columns: 1fr; } .pair { grid-template-columns: 1fr; } }
</style>

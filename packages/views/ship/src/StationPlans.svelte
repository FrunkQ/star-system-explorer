<script lang="ts">
  import { untrack } from "svelte";
  // A station or land construct built from its description (renderer station-design.ts): plans from above and from the
  // side with call-outs, or the 3D model; a shape switch and a remix. Sizes come from the record and the base-ships
  // rules; the shape only arranges. The design is kept as a recipe ({ shape, seed, overrides }), reported through
  // `onchange` so the owner can save it on the construct as `construct.design`. `locked` (play desks) hides the shape
  // and remix, keeping turning and framing: play never changes a construct's look.
  import { designStation, narrateStation, pickShape, startStation, stationViews, type StationLayout, type StationLimits, type StationRecipe, type StationRecord, type StationRules, type StationShape, type StationSkin, type StationView, type StationView3D } from "../../../renderer/src/index.ts";
  import { DEFAULT_VIEW, SEEN_ELEVATION, WHY_TRUE_HELD, WHY_WIREFRAME_LATER, type ViewChoice, type ViewControlsSpec } from "../../../ui-contract/src/index.ts";
  import { ViewControls } from "../../../chart-grammar/src/index.ts";

  const CONTROLS: ViewControlsSpec = { styles: { chart: true, "3d": true, true: WHY_TRUE_HELD, wireframe: WHY_WIREFRAME_LATER }, flat: true };

  let {
    station,
    rules,
    shapes,
    limits,
    skin,
    recipe,
    locked = false,
    segments = 32,
    onchange,
  }: {
    station: StationRecord;
    rules: StationRules;
    shapes: Record<string, StationShape>;
    limits: StationLimits;
    skin?: StationSkin;
    /** Where the plans start; left out, the shape the description fits best. */
    recipe?: StationRecipe;
    locked?: boolean;
    segments?: number;
    onchange?: (r: StationRecipe) => void;
  } = $props();

  // Shapes for this construct's place only: a ring is not offered for a town on the ground.
  const offered = $derived(Object.entries(shapes).filter(([, s]) => s.place === (station.place ?? "space")));
  let shapeId = $state(untrack(() => recipe?.shape ?? pickShape(station, shapes)));
  let seed = $state(untrack(() => recipe?.seed ?? 1));
  const overrides = untrack(() => recipe?.overrides);
  // The station itself first: power arrays for a big port run to kilometres and would make it a dot.
  let framing = $state<"all" | "core">("core");
  let view = $state<ViewChoice>({ ...DEFAULT_VIEW });
  const W = 560;
  const H = 240;
  const layout = $derived<StationLayout>(designStation(station, rules, shapes, limits, { shape: shapeId, seed, ...(overrides ? { overrides } : {}) }));
  const views = $derived(stationViews(layout, [W, H], framing));
  const words = $derived(narrateStation(layout, shapes[layout.shape]!));
  const canClose = $derived(layout.parts.some((p) => p.surface === "solar" || p.surface === "radiators"));

  let canvas = $state<HTMLCanvasElement | null>(null);
  let model: StationView3D | null = null;
  let yaw = $state(35);
  let freePitch = $state(30);
  let zoom = $state(1);
  let pan = $state<[number, number]>([0, 0]);
  let failed = $state(false);
  const pitch = $derived(view.seen === "free" ? freePitch : (SEEN_ELEVATION[view.seen] * 180) / Math.PI);
  // Picking a fixed "Seen from" frames the model afresh (turn, zoom and slide), even the one already shown.
  $effect(() => {
    void view.framed;
    if (untrack(() => view.seen) !== "free") ((yaw = 35), (zoom = 1), (pan = [0, 0]));
  });
  $effect(() => {
    if (view.style !== "3d" || !canvas || !skin) return;
    const c = canvas;
    const l = layout;
    const k = skin;
    let gone = false;
    void startStation(c, l, k, { segments, seed }).then((m) => {
      if (gone) return m?.dispose();
      model = m;
      failed = !m;
      redraw();
    });
    return () => {
      gone = true;
      model?.dispose();
      model = null;
    };
  });
  function redraw() {
    model?.draw({ yaw_deg: yaw, pitch_deg: pitch, zoom, pan, frame: framing === "core" ? "core" : "all", width: W * 2, height: H * 2, ground: getComputedStyle(canvas!).getPropertyValue("--sse-chart-ground").trim() || "black" });
  }
  $effect(() => {
    void [yaw, pitch, zoom, pan, framing];
    redraw();
  });
  let drag: { x: number; y: number; slide: boolean } | null = null;
  const touches = new Map<number, [number, number]>();
  function down(e: PointerEvent) {
    if (view.seen !== "free") return;
    touches.set(e.pointerId, [e.clientX, e.clientY]);
    drag = { x: e.clientX, y: e.clientY, slide: e.shiftKey || touches.size > 1 };
    (e.currentTarget as Element).setPointerCapture(e.pointerId);
  }
  function move(e: PointerEvent) {
    if (!drag) return;
    const before = [...touches.values()];
    touches.set(e.pointerId, [e.clientX, e.clientY]);
    if (touches.size === 2) {
      const after = [...touches.values()];
      const span = (p: [number, number][]) => Math.hypot(p[0]![0] - p[1]![0], p[0]![1] - p[1]![1]);
      if (before.length === 2 && span(before) > 0) zoom = Math.max(0.3, Math.min(200, (zoom * span(after)) / span(before)));
    }
    const dx = e.clientX - drag.x;
    const dy = e.clientY - drag.y;
    if (drag.slide || touches.size > 1) pan = [pan[0] - (dx / W) * 2, pan[1] + (dy / H) * 2];
    else {
      yaw -= dx * 0.4;
      freePitch = Math.max(-85, Math.min(85, freePitch + dy * 0.4));
    }
    drag = { x: e.clientX, y: e.clientY, slide: drag.slide || touches.size > 1 };
  }
  function up(e: PointerEvent) {
    touches.delete(e.pointerId);
    if (!touches.size) drag = null;
  }
  function wheel(e: WheelEvent) {
    if (view.seen !== "free") return;
    e.preventDefault();
    zoom = Math.max(0.3, Math.min(200, zoom * Math.exp(-e.deltaY * 0.0015)));
  }
  function change(next: Partial<StationRecipe>) {
    if (next.shape !== undefined) shapeId = next.shape;
    if (next.seed !== undefined) seed = next.seed;
    onchange?.({ shape: shapeId, seed, ...(overrides ? { overrides } : {}) });
  }
</script>

{#snippet plan(v: StationView, title: string)}
  <figure>
    <figcaption>{title}</figcaption>
    <svg viewBox="0 0 {W} {H}" width="100%" role="img" aria-label="{title}: {words}">
      {#each v.paths as p, i (i)}<path class={p.surface} d={p.d} fill-rule="evenodd" />{/each}
      {#each v.callouts as c, i (i)}
        <path class="leader" d="M{c.anchor[0]} {c.anchor[1]} L{c.label[0]} {c.label[1] + (c.above ? 4 : -12)}" />
        <circle class="dot" cx={c.anchor[0]} cy={c.anchor[1]} r="2" />
        <text x={c.label[0]} y={c.label[1]} text-anchor="middle">{c.text}</text>
      {/each}
    </svg>
  </figure>
{/snippet}

<section class="plans" data-station-plans data-shape={layout.shape} data-mega={layout.mega ? (layout.mega.fictional ? "fictional" : "yes") : undefined}>
  <ViewControls spec={CONTROLS} choice={view} onchange={(c) => (view = c)} compact />
  <div class="bar">
    {#if !locked}
      <label>Shape
        <select value={shapeId} onchange={(e) => change({ shape: (e.currentTarget as HTMLSelectElement).value })} data-station-shape>
          {#each offered as [id, s] (id)}<option value={id}>{s.name}</option>{/each}
        </select>
      </label>
      <button type="button" onclick={() => change({ seed: seed + 1 })} data-remix>Remix</button>
    {/if}
    {#if canClose}
      <button type="button" aria-pressed={framing === "all"} onclick={() => (framing = framing === "core" ? "all" : "core")} data-station-arrays>Show the power arrays whole</button>
    {/if}
  </div>
  {#if view.style === "3d" && skin}
    <figure>
      <figcaption>{failed ? "This device cannot draw 3D; the plans are below." : "Model"}</figcaption>
      <canvas bind:this={canvas} width={W * 2} height={H * 2} data-station-model onpointerdown={down} onpointermove={move} onpointerup={up} onpointercancel={up} onwheel={wheel}></canvas>
    </figure>
  {/if}
  {#if view.style !== "3d" || failed || !skin}
    {@render plan(views.top, views.clipped ? "From above (power arrays left out)" : "From above")}
    {@render plan(views.side, views.clipped ? "Side (power arrays left out)" : "Side")}
  {/if}
  <p class="words">{words}</p>
  {#if layout.mega}
    <p class="mega" data-megastructure>
      <span class="mark">{layout.mega.fictional ? "Fictional" : "Megastructure"}</span>
      {layout.mega.why}.{#if layout.mega.needs} Its hull would need {layout.mega.needs}.{/if}
    </p>
  {/if}
  {#if layout.notes.length}
    <ul class="notes">
      {#each layout.notes as n, i (i)}<li>{n}</li>{/each}
    </ul>
  {/if}
</section>

<style>
  .plans { display: grid; gap: 0.5em; }
  .bar { display: flex; gap: 0.75em; align-items: center; flex-wrap: wrap; }
  figure { margin: 0; }
  figcaption { color: var(--sse-ink-muted); font-size: 0.8em; }
  svg { display: block; height: auto; background: var(--sse-chart-ground); }
  canvas { display: block; width: 100%; height: auto; aspect-ratio: 560 / 240; background: var(--sse-chart-ground); touch-action: none; }
  path { stroke: var(--sse-ink); stroke-width: 1; fill: var(--sse-ground-raised); stroke-linejoin: round; }
  .truss { fill: var(--sse-ink-faint); }
  .radiators { fill: var(--sse-zone); fill-opacity: 0.35; stroke: var(--sse-zone-ink); }
  .solar { fill: var(--sse-ink-muted); fill-opacity: 0.35; }
  .glass { fill: var(--sse-ink-muted); fill-opacity: 0.5; }
  .rock { fill: var(--sse-ink-faint); fill-opacity: 0.8; }
  .dock { fill: var(--sse-accent); fill-opacity: 0.5; stroke: var(--sse-accent); }
  .leader { fill: none; stroke: var(--sse-ink-muted); stroke-width: 0.6; }
  .dot { fill: var(--sse-ink); }
  text { fill: var(--sse-ink); font-size: 10px; }
  .words, .mega { color: var(--sse-ink-muted); font-size: 0.85em; line-height: 1.4; margin: 0; }
  .mega .mark { font-variant: small-caps; letter-spacing: 0.06em; color: var(--sse-accent-ink); border: 1px solid currentColor; padding: 0 0.3em; margin-right: 0.3em; }
  .notes { margin: 0; padding-left: 1.2em; font-size: 0.8em; color: var(--sse-ink-muted); }
</style>

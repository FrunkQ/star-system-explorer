<script lang="ts">
  import { untrack } from "svelte";
  // A ship designed from its parts: side and top views with call-outs, a style switch and a remix button. Sizes come
  // from the ship record and the base-ships rules; the style only arranges. The design is kept as a recipe
  // ({ style, seed, overrides }), reported through `onchange` so the owner can save it with the ship.
  import { designShip, fittedChecks, narrateDesign, shipViews, startShip, type FitResult, type DesignOverrides, type DesignRecipe, type DesignRules, type DesignShip, type Propellants, type ShipStyle, type ShipView, type ShipView3D } from "../../../renderer/src/index.ts";
  import { DEFAULT_VIEW, SEEN_ELEVATION, WHY_TRUE_HELD, WHY_WIREFRAME_LATER, verdictWords, type ViewChoice, type ViewControlsSpec } from "../../../ui-contract/src/index.ts";
  import { ViewControls } from "../../../chart-grammar/src/index.ts";

  // Chart is the plans (flat: side and top); any other "Seen from" turns it into the 3D model.
  const CONTROLS: ViewControlsSpec = { styles: { chart: true, "3d": true, true: WHY_TRUE_HELD, wireframe: WHY_WIREFRAME_LATER }, flat: true };

  let {
    ship,
    fuels = {},
    rules,
    styles,
    recipe = { style: Object.keys(styles)[0] ?? "spine", seed: 1 },
    fit = "all",
    segments = 32,
    abilities,
    pass,
    onchange,
  }: {
    ship: DesignShip;
    fuels?: Propellants;
    rules: DesignRules;
    styles: Record<string, ShipStyle>;
    recipe?: DesignRecipe;
    fit?: "all" | "hull";
    /** Sides round each body in 3D: the device tier sets it. */
    segments?: number;
    /** What its abilities fitted (renderer fitAbilities, run by the owner against the kernel) and the kernel's verdicts. */
    abilities?: FitResult;
    /** An aerobraking pass at its hottest (renderer passGlow): opens on the 3D model, radiators folded, the shield glowing. */
    pass?: { shield_k: number; standoff_rn: number; shock_curve_rn: number; glow: number };
    onchange?: (r: DesignRecipe) => void;
  } = $props();

  // The recipe and framing passed in are where the plans start; after that the reader's choices hold.
  let styleId = $state(untrack(() => recipe.style));
  let seed = $state(untrack(() => recipe.seed));
  let overrides = $state<DesignOverrides>(untrack(() => recipe.overrides ?? {}));
  let framing = $state<"all" | "hull">(untrack(() => (pass ? "hull" : fit)));
  let view = $state<ViewChoice>(untrack(() => (pass ? { ...DEFAULT_VIEW, style: "3d", seen: "free" } : { ...DEFAULT_VIEW })));
  const W = 560;
  const H = 220;
  const style = $derived(styles[styleId] ?? Object.values(styles)[0]!);
  const layout = $derived(designShip(ship, fuels, rules, styleId, style, seed, overrides, abilities?.fit));
  // One line per world and act the ship was fitted for: the kernel's verdict in words.
  const verdicts = $derived(abilities ? fittedChecks(abilities).map(({ world, check: c }) => ({ key: `${world}:${c.id}`, text: `${world}, ${c.words.toLowerCase()}: ${verdictWords(c.verdict)}`, verdict: c.verdict })) : []);
  const views = $derived(shipViews(layout, [W, H], framing));
  const words = $derived(narrateDesign(layout, style));
  // Worth closing in when radiators or a long spine dwarf the modules.
  const canClose = $derived(shipViews(layout, [W, H], "hull").clipped);

  // The 3D model: built once per design, redrawn as the camera turns. Free: drag to turn, wheel to zoom.
  let canvas = $state<HTMLCanvasElement | null>(null);
  let model: ShipView3D | null = null;
  let yaw = $state(35);
  let freePitch = $state(30);
  let zoom = $state(1);
  let pan = $state<[number, number]>([0, 0]);
  let failed = $state(false);
  const pitch = $derived(view.seen === "free" ? freePitch : (SEEN_ELEVATION[view.seen] * 180) / Math.PI);
  $effect(() => {
    if (view.style !== "3d" || !canvas || !style.skin) return;
    const c = canvas;
    const l = layout;
    let gone = false;
    const p = pass;
    void startShip(c, l, style.skin, { segments, seed, ...(p ? { pass: p } : {}) }).then((m) => {
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
    model?.draw({ yaw_deg: yaw, pitch_deg: pitch, zoom, pan, frame: pass && framing === "hull" ? "front" : framing, width: W * 2, height: H * 2, ground: getComputedStyle(canvas!).getPropertyValue("--sse-chart-ground").trim() || "black" });
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
  // Drag turns and tilts; Shift-drag or two fingers slide; two fingers apart or together zoom.
  function move(e: PointerEvent) {
    if (!drag) return;
    const before = [...touches.values()];
    touches.set(e.pointerId, [e.clientX, e.clientY]);
    if (touches.size === 2) {
      const after = [...touches.values()];
      const span = (p: [number, number][]) => Math.hypot(p[0]![0] - p[1]![0], p[0]![1] - p[1]![1]);
      if (before.length === 2 && span(before) > 0) zoom = Math.max(0.3, Math.min(40, (zoom * span(after)) / span(before)));
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
    zoom = Math.max(0.3, Math.min(40, zoom * Math.exp(-e.deltaY * 0.0015)));
  }

  function change(next: Partial<DesignRecipe>) {
    if (next.style !== undefined) styleId = next.style;
    if (next.seed !== undefined) seed = next.seed;
    onchange?.({ style: styleId, seed, ...(Object.keys(overrides).length ? { overrides } : {}) });
  }
</script>

{#snippet plan(v: ShipView, title: string)}
  <figure>
    <figcaption>{title}</figcaption>
    <svg viewBox="0 0 {W} {H}" width="100%" role="img" aria-label="{title}: {words}">
      {#each v.paths as p, i (i)}<path class={p.system} d={p.d} />{/each}
      {#each v.callouts as c, i (i)}
        <path class="leader" d="M{c.anchor[0]} {c.anchor[1]} L{c.label[0]} {c.label[1] + (c.above ? 4 : -12)}" />
        <circle class="dot" cx={c.anchor[0]} cy={c.anchor[1]} r="2" />
        <text x={c.label[0]} y={c.label[1]} text-anchor="middle">{c.text}</text>
      {/each}
    </svg>
  </figure>
{/snippet}

<section class="plans" data-ship-plans>
  <ViewControls spec={CONTROLS} choice={view} onchange={(c) => (view = c)} compact />
  <div class="bar">
    <label>Style
      <select value={styleId} onchange={(e) => change({ style: (e.currentTarget as HTMLSelectElement).value })}>
        {#each Object.entries(styles) as [id, s] (id)}<option value={id}>{s.name}</option>{/each}
      </select>
    </label>
    <button type="button" onclick={() => change({ seed: seed + 1 })} data-remix>Remix</button>
    {#if canClose || framing === "hull"}
      <button type="button" aria-pressed={framing === "hull"} onclick={() => (framing = framing === "hull" ? "all" : "hull")}>Close in on the hull</button>
    {/if}
  </div>
  {#if view.style === "3d"}
    <figure>
      <figcaption>{failed ? "This device cannot draw 3D; the plans are below." : "Model"}</figcaption>
      <canvas bind:this={canvas} width={W * 2} height={H * 2} data-ship-model onpointerdown={down} onpointermove={move} onpointerup={up} onpointercancel={up} onwheel={wheel}></canvas>
    </figure>
  {/if}
  {#if view.style !== "3d" || failed}
    {@render plan(views.side, "Side")}
    {@render plan(views.top, "From above")}
  {/if}
  <p class="words">{words}</p>
  {#if verdicts.length}
    <ul class="verdicts">
      {#each verdicts as v (v.key)}<li class={v.verdict}>{v.text}</li>{/each}
    </ul>
  {/if}
</section>

<style>
  .plans { display: grid; gap: 0.5em; }
  .bar { display: flex; gap: 0.75em; align-items: center; flex-wrap: wrap; }
  figure { margin: 0; }
  figcaption { color: var(--sse-ink-muted); font-size: 0.8em; }
  svg { display: block; height: auto; background: var(--sse-chart-ground); }
  canvas { display: block; width: 100%; height: auto; aspect-ratio: 560 / 220; background: var(--sse-chart-ground); touch-action: none; }
  path { stroke: var(--sse-ink); stroke-width: 1; fill: var(--sse-ground-raised); stroke-linejoin: round; }
  .plume { fill: var(--sse-accent); fill-opacity: 0.2; stroke: var(--sse-accent); }
  .spine { fill: var(--sse-ink-faint); }
  .radiators { fill: var(--sse-zone); fill-opacity: 0.35; stroke: var(--sse-zone-ink); }
  .leader { fill: none; stroke: var(--sse-ink-muted); stroke-width: 0.6; }
  .dot { fill: var(--sse-ink); }
  text { fill: var(--sse-ink); font-size: 10px; }
  .shield { fill: var(--sse-ink-muted); }
  .wings { fill-opacity: 0.6; }
  .legs { fill: var(--sse-ink-faint); }
  .verdicts { margin: 0; padding-left: 1.2em; font-size: 0.85em; color: var(--sse-ink-muted); }
  .verdicts .dangerous { color: var(--sse-alert); }
  .words { color: var(--sse-ink-muted); font-size: 0.85em; line-height: 1.4; margin: 0; }
</style>

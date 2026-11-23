<script lang="ts">
  import { untrack } from "svelte";
  // A ship designed from its parts: side and top views with call-outs, a style switch and a remix button. Sizes come
  // from the ship record and the base-ships rules; the style only arranges. The design is kept as a recipe
  // ({ style, seed, overrides }), reported through `onchange` so the owner can save it with the ship.
  import { designShip, narrateDesign, shipViews, type DesignOverrides, type DesignRecipe, type DesignRules, type DesignShip, type Propellants, type ShipStyle, type ShipView } from "../../../renderer/src/index.ts";

  let {
    ship,
    fuels = {},
    rules,
    styles,
    recipe = { style: Object.keys(styles)[0] ?? "spine", seed: 1 },
    fit = "all",
    onchange,
  }: {
    ship: DesignShip;
    fuels?: Propellants;
    rules: DesignRules;
    styles: Record<string, ShipStyle>;
    recipe?: DesignRecipe;
    fit?: "all" | "hull";
    onchange?: (r: DesignRecipe) => void;
  } = $props();

  // The recipe and framing passed in are where the plans start; after that the reader's choices hold.
  let styleId = $state(untrack(() => recipe.style));
  let seed = $state(untrack(() => recipe.seed));
  let overrides = $state<DesignOverrides>(untrack(() => recipe.overrides ?? {}));
  let framing = $state<"all" | "hull">(untrack(() => fit));
  const W = 560;
  const H = 220;
  const style = $derived(styles[styleId] ?? Object.values(styles)[0]!);
  const layout = $derived(designShip(ship, fuels, rules, styleId, style, seed, overrides));
  const views = $derived(shipViews(layout, [W, H], framing));
  const words = $derived(narrateDesign(layout, style));

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
  <div class="bar">
    <label>Style
      <select value={styleId} onchange={(e) => change({ style: (e.currentTarget as HTMLSelectElement).value })}>
        {#each Object.entries(styles) as [id, s] (id)}<option value={id}>{s.name}</option>{/each}
      </select>
    </label>
    <button type="button" onclick={() => change({ seed: seed + 1 })} data-remix>Remix</button>
    {#if views.clipped || framing === "hull"}
      <button type="button" aria-pressed={framing === "hull"} onclick={() => (framing = framing === "hull" ? "all" : "hull")}>Close in on the hull</button>
    {/if}
  </div>
  {@render plan(views.side, "Side")}
  {@render plan(views.top, "From above")}
  <p class="words">{words}</p>
</section>

<style>
  .plans { display: grid; gap: 0.5em; }
  .bar { display: flex; gap: 0.75em; align-items: center; flex-wrap: wrap; }
  figure { margin: 0; }
  figcaption { color: var(--sse-ink-muted); font-size: 0.8em; }
  svg { display: block; height: auto; background: var(--sse-chart-ground); }
  path { stroke: var(--sse-ink); stroke-width: 1; fill: var(--sse-ground-raised); stroke-linejoin: round; }
  .plume { fill: var(--sse-accent); fill-opacity: 0.2; stroke: var(--sse-accent); }
  .spine { fill: var(--sse-ink-faint); }
  .radiators { fill: var(--sse-zone); fill-opacity: 0.35; stroke: var(--sse-zone-ink); }
  .leader { fill: none; stroke: var(--sse-ink-muted); stroke-width: 0.6; }
  .dot { fill: var(--sse-ink); }
  text { fill: var(--sse-ink); font-size: 10px; }
  .words { color: var(--sse-ink-muted); font-size: 0.85em; line-height: 1.4; margin: 0; }
</style>

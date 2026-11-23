<!--
  The Light control (nebulae-and-light design 2.2, O1 UI side): which band
  recipe paints the marks. "Chart colours" leaves the symbology alone; every
  other recipe colours each body by its light and always shows a legend chip
  that says what the colours mean and how honest they are. The choice stays
  on this device and never reaches the session, so the state hash cannot
  change. Until the kernel publishes light facts, the light is stubbed from
  scene facts, and the panel says so body by body.
-->
<script lang="ts">
  import { untrack } from "svelte";
  import { HONESTY_WORDS, SYMBOLS, readLight, type BandRecipe, type LightDescription } from "../../ui-contract/src/index.ts";

  let { recipes, lights, missing, names, value = $bindable(SYMBOLS), key = "light" }: { recipes: readonly BandRecipe[]; lights: readonly LightDescription[]; missing: readonly string[]; names: ReadonlyMap<string, string>; value?: string; key?: string } = $props();

  const store = $derived(`sse4.${key}`);
  untrack(() => {
    try {
      value = readLight(localStorage.getItem(`sse4.${key}`), recipes.map((r) => r.id));
    } catch {
      /* private window: the default stands */
    }
  });
  $effect(() => {
    const v = JSON.stringify({ recipe: value });
    try {
      localStorage.setItem(store, v);
    } catch {
      /* settings last for this page only */
    }
  });

  const recipe = $derived(recipes.find((r) => r.id === value) ?? null);
  const stubbed = $derived(lights.filter((l) => l.stubbed));
  let open = $state(false);
  let why = $state<string | null>(null);
  const id = $props.id();
  // The panel opens above its button, never over it, and closes on ×, Escape or a press anywhere else.
  let root = $state<HTMLElement | null>(null);
  let toggle = $state<HTMLButtonElement | null>(null);
  let bottom = $state(44);
  function place() {
    const r = toggle?.getBoundingClientRect();
    if (r) bottom = Math.max(8, innerHeight - r.top + 8);
  }
  function close() {
    open = false;
    toggle?.focus();
  }
  const groups: [string, (r: BandRecipe) => boolean][] = [
    ["Seen as light", (r) => r.mode === "eye" || r.mode === "camera"],
    ["Astro palettes", (r) => r.mode === "composite"],
    ["One band", (r) => r.mode === "single"],
  ];
</script>

<svelte:window
  onkeydown={(e) => open && e.key === "Escape" && (e.stopPropagation(), close())}
  onpointerdown={(e) => open && !root?.contains(e.target as Node) && (open = false)}
  onresize={() => open && place()} />

<span class="light" bind:this={root} data-light={value} data-honesty={recipe?.honesty ?? "symbols"} data-stubbed={recipe ? stubbed.length : 0}>
  <button type="button" class="toggle" bind:this={toggle} aria-expanded={open} aria-controls="{id}-panel" onclick={() => (place(), (open = !open))} title="Which light the marks show">
    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" aria-hidden="true"><circle cx="12" cy="12" r="4" /><path d="M12 2v3M12 19v3M2 12h3M19 12h3M5 5l2 2M17 17l2 2M5 19l2-2M17 7l2-2" /></svg>
    Light: {recipe?.name ?? "chart colours"}
  </button>
  {#if recipe}
    <span class="chip" data-legend-chip title={recipe.legend}>
      {HONESTY_WORDS[recipe.honesty]}{#if stubbed.length}<span class="stub">&nbsp;· stubbed spectra</span>{/if}
    </span>
  {/if}
  {#if open}
    <div class="panel" id="{id}-panel" role="group" aria-label="Light" style:bottom="{bottom}px">
      <div class="phead"><span class="k">Light</span><button type="button" class="pclose" aria-label="Close light" data-panel-close onclick={close}>×</button></div>
      <fieldset>
        <legend>Marks show</legend>
        <label class="check"><input type="radio" name="{id}-r" value={SYMBOLS} bind:group={value} /> Chart colours (the symbology)</label>
        {#each groups as [g, f] (g)}
          <div class="k">{g}</div>
          {#each recipes.filter(f) as r (r.id)}
            <label class="check"><input type="radio" name="{id}-r" value={r.id} bind:group={value} /> {r.name} <span class="tag">{HONESTY_WORDS[r.honesty]}</span></label>
          {/each}
        {/each}
      </fieldset>
      {#if recipe}
        <div class="legend" aria-label="What the colours mean">
          <div class="k">What the colours mean</div>
          <p>{recipe.legend}</p>
          {#if recipe.channels.length}
            <ul class="channels">
              {#each recipe.channels as c (c.label)}<li><span class="sw" style="background:{c.colour}"></span>{c.label}</li>{/each}
            </ul>
          {/if}
          <p class="note">{recipe.why}</p>
          {#if recipe.mode !== "single"}<p class="note">Each mark shows the colour of its own light, not how bright it is beside its star.</p>{:else}<p class="note">Bodies fainter than the ramp reaches are drawn in ink.</p>{/if}
        </div>
        <div class="k">Where the light comes from</div>
        {#if stubbed.length}
          <p class="note" role="note"><b>Stubbed spectra.</b> Until the kernel publishes light facts, each body's light is built from facts this view already has. Ask any body why.</p>
          <ul class="sources">
            {#each stubbed as l (l.address)}
              <li>
                <span>{names.get(l.address) ?? l.address}</span>
                <button type="button" class="linkbtn" aria-label="Why is {names.get(l.address) ?? l.address} this colour?" onclick={() => (why = why === l.address ? null : l.address)}>why?</button>
                {#if why === l.address}<p class="why">{l.why}</p>{/if}
              </li>
            {/each}
          </ul>
        {/if}
        {#if missing.length}<p class="note">No light known for {missing.join(", ")}: drawn in ink.</p>{/if}
      {/if}
    </div>
  {/if}
</span>

<style>
  .light {
    position: relative;
    display: inline-flex;
    align-items: center;
    gap: 8px;
  }
  .toggle,
  .linkbtn {
    font: inherit;
    color: inherit;
    background: none;
    border: 0;
    padding: 0;
    cursor: pointer;
    display: inline-flex;
    gap: 4px;
    align-items: center;
  }
  .toggle {
    min-height: 24px;
  }
  .linkbtn {
    color: var(--sse-accent, currentColor);
    text-decoration: underline;
  }
  .chip,
  .tag {
    font-size: 10px;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    border: 1px solid currentColor;
    padding: 0 4px;
    white-space: nowrap;
  }
  .stub {
    color: var(--sse-accent, currentColor);
  }
  .phead {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }
  .pclose {
    height: auto;
    padding: 0 4px;
    border: none;
    background: none;
    color: inherit;
    font-size: 20px;
    line-height: 1;
    cursor: pointer;
  }
  .panel {
    position: fixed;
    bottom: 44px;
    right: 16px;
    z-index: 10;
    width: min(360px, calc(100vw - 32px));
    max-height: 70vh;
    overflow: auto;
    display: grid;
    gap: 8px;
    padding: 12px 14px;
    background: var(--sse-paper, var(--sse-ground, Canvas));
    color: var(--sse-ink, CanvasText);
    border: var(--sse-line, 1px) solid var(--sse-ink, CanvasText);
    box-shadow: 4px 4px 0 var(--sse-ink, CanvasText);
    font-family: var(--sse-font-body, inherit);
    font-size: 13px;
    text-align: left;
    white-space: normal;
  }
  fieldset {
    border: var(--sse-line, 1px) solid var(--sse-rule, currentColor);
    display: grid;
    gap: 4px;
    padding: 4px 8px 8px;
    margin: 0;
  }
  .check {
    display: flex;
    gap: 8px;
    align-items: center;
    flex-wrap: wrap;
  }
  input {
    accent-color: var(--sse-accent, auto);
  }
  legend,
  .k {
    font-size: 11px;
    color: var(--sse-ink-muted, currentColor);
    letter-spacing: 0.06em;
    text-transform: uppercase;
    margin-top: 4px;
  }
  .legend p {
    margin: 2px 0;
    line-height: 1.45;
  }
  .note {
    margin: 0;
    font-size: 12px;
    color: var(--sse-ink-muted, currentColor);
  }
  .channels,
  .sources {
    list-style: none;
    margin: 0;
    padding: 0;
    display: grid;
    gap: 4px;
  }
  .channels li,
  .sources li {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    align-items: center;
  }
  .sw {
    width: 12px;
    height: 12px;
    border: 1px solid var(--sse-ink, CanvasText);
  }
  .why {
    flex-basis: 100%;
    margin: 2px 0 6px;
    font-size: 12px;
    line-height: 1.45;
  }
</style>

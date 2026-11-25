<!-- The standard view controls (interface style rule 12): View style (Chart, 3D, True, Wireframe), Seen from (Above,
     Tilted, Edge-on, Free), Scale (Readable, Real), HUD (Off, On: orbit lines, names and outlines) and Band (Visible, Infrared, Ultraviolet, X-ray,
     Radio, Mag), the same words in the same order on every diagram, for every seat. Options a view or this
     device cannot do stay in place, quiet, with the reason as their title. Free shows how it is worked.
     How often each is used decides where it sits (Alex, 23 Nov: "how often it is used, easy flow and newbie friendly"):
     View style, Seen from and Scale are a press away; HUD and Band, changed far less, fold into one chip that always
     says what they are ("HUD on · Visible ▾") and opens the same two switches beside it.
     Composite (XO17, Alex 24 Nov: "everywhere"), the band switch's last entry, shows the last mix at once and opens a
     tick list under it: presets, the bands this view and seat can draw with their colours, at most two on a phone. -->
<script lang="ts">
  import Segmented from "./Segmented.svelte";
  import { opensFrom } from "./motion.ts";
  import { COMPOSITE, EXPOSURE_WORDS, COMPOSITE_WORD, DEFAULT_MIX, FREE_HINT, LAYER_COLOURS, VIEW_BANDS, VIEW_BAND_WORDS, compositeLegend, compositePresets, pickScale, pickSeen, pickStyle, presetMix, recolour, toggleLayer, viewOptions, type BandLimits, type BandPick, type CompositeMix, type ViewBand, type ViewExposure, type ViewScale, type SeenFrom, type ViewChoice, type ViewControlsSpec, type ViewStyle } from "../../ui-contract/src/index.ts";

  let { spec, choice, onchange, compact = false, limits = {} }: { spec: ViewControlsSpec; choice: ViewChoice; onchange: (c: ViewChoice) => void; compact?: boolean; limits?: BandLimits } = $props();
  const o = $derived(viewOptions(spec, choice, limits));
  // The mix the tick list edits: the viewer's kept one, cut to what can be drawn here.
  const mix = $derived<CompositeMix>({ ...(choice.composite?.preset ? { preset: choice.composite.preset } : {}), layers: o.composite ? o.layers : (choice.composite ?? DEFAULT_MIX).layers });
  const legend = $derived(o.composite ? compositeLegend({ layers: o.layers }, VIEW_BAND_WORDS) : null);
  let refused = $state<string | null>(null);
  function pickBand(b: BandPick) {
    refused = null;
    // A single band closes the fold, as a menu does; Composite keeps it open on its tick list.
    if (b !== COMPOSITE) open = false;
    onchange({ ...choice, band: b, ...(b === COMPOSITE && !choice.composite ? { composite: DEFAULT_MIX } : {}) });
  }
  function tick(b: ViewBand) {
    const limit = { can: o.drawable, ...(o.maxLayers !== undefined ? { max: o.maxLayers } : {}) };
    const next = toggleLayer(mix, b, limit);
    if ("refused" in next) return void (refused = next.refused);
    refused = null;
    onchange({ ...choice, band: COMPOSITE, composite: next });
  }
  function preset(id: string) {
    const m = presetMix(id);
    if (!m) return;
    refused = null;
    onchange({ ...choice, band: COMPOSITE, composite: m });
  }
  const uid = $props.id();
  let open = $state(false);
  let chip = $state<HTMLButtonElement | null>(null);
  let fold = $state<HTMLDivElement | null>(null);
  const summary = $derived(`HUD ${o.shown.hud ? "on" : "off"} · ${o.composite ? `${COMPOSITE_WORD} (${o.layers.length})` : VIEW_BAND_WORDS[o.shown.band]}${o.shown.style === "true" ? ` · ${EXPOSURE_WORDS[o.shown.exposure]}` : ""}`);
  // A choice closes it, as a menu does; so does a press off it, or Escape, which gives the keyboard back to the chip.
  $effect(() => {
    if (!open) return;
    const away = (e: PointerEvent) => {
      if (fold && !fold.contains(e.target as Node)) open = false;
    };
    const esc = (e: KeyboardEvent) => {
      if (e.key !== "Escape") return;
      e.stopPropagation();
      open = false;
      chip?.focus();
    };
    document.addEventListener("pointerdown", away, true);
    document.addEventListener("keydown", esc, true);
    return () => {
      document.removeEventListener("pointerdown", away, true);
      document.removeEventListener("keydown", esc, true);
    };
  });
</script>

<div class="vc" class:compact data-view-controls data-style={o.shown.style} data-seen={o.shown.seen} data-scale={o.shown.scale} data-hud={o.shown.hud ? "on" : "off"} data-band={o.composite ? COMPOSITE : o.shown.band} data-exposure={o.shown.exposure} data-layers={o.layers.map((l) => l.band).join(" ")}>
  <div class="field">
    <span class="k" aria-hidden="true">View style</span>
    <Segmented label="View style" value={o.shown.style} options={o.styles} onchange={(s: ViewStyle) => onchange(pickStyle(choice, s))} />
  </div>
  <div class="field">
    <span class="k" aria-hidden="true">Seen from</span>
    <Segmented label="Seen from" value={o.shown.seen} options={o.seen} onchange={(s: SeenFrom) => onchange(pickSeen(spec, choice, s))} />
  </div>
  <div class="field">
    <span class="k" aria-hidden="true">Scale</span>
    <Segmented label="Scale" value={o.shown.scale} options={o.scales} onchange={(s: ViewScale) => onchange(pickScale(spec, choice, s))} />
  </div>
  <div class="field fold" bind:this={fold}>
    <span class="k" aria-hidden="true">Show</span>
    <button type="button" class="chip" bind:this={chip} aria-expanded={open} aria-controls="{uid}-more" data-view-more title="HUD and Band: the overview drawn over the view, and the light it is seen in" onclick={() => (open = !open)}>{summary} <span aria-hidden="true">▾</span></button>
    {#if open}
      <div class="pop" id="{uid}-more" role="group" aria-label="Overview and light" use:opensFrom={() => chip}>
        <div class="field">
          <span class="k" aria-hidden="true">HUD</span>
          <Segmented label="HUD" value={o.shown.hud ? "on" : "off"} options={o.huds} onchange={(v: "off" | "on") => ((open = false), onchange({ ...choice, hud: v === "on" }))} />
        </div>
        <div class="field">
          <span class="k" aria-hidden="true">Band</span>
          <Segmented label="Band" value={o.composite ? COMPOSITE : o.shown.band} options={o.bands} onchange={pickBand} />
        </div>
        {#if o.composite}
          <div class="mix" role="group" aria-label="Composite: bands laid into one picture" data-composite>
            <div class="presets">
              {#each compositePresets() as p (p.id)}
                <button type="button" class="preset" class:on={choice.composite?.preset === p.id} title={p.why} data-preset={p.id} onclick={() => preset(p.id)}>{p.name}</button>
              {/each}
            </div>
            <ul class="ticks">
              {#each VIEW_BANDS as b (b)}
                {@const layer = mix.layers.find((l) => l.band === b)}
                {@const drawn = o.drawable.includes(b)}
                <li class:quiet={!drawn}>
                  <label title={drawn ? undefined : o.bands.find((x) => x.value === b)?.unavailable}>
                    <input type="checkbox" checked={!!layer} disabled={!drawn} data-tick={b} onchange={() => tick(b)} />
                    <span>{VIEW_BAND_WORDS[b]}</span>
                  </label>
                  {#if b === "visible"}
                    <span class="natural" title="Visible keeps its own colours">actual</span>
                  {:else}
                    <input type="color" class="swatch" aria-label="{VIEW_BAND_WORDS[b]} colour" value={layer?.colour ?? LAYER_COLOURS[b]} disabled={!layer} onchange={(e) => onchange({ ...choice, band: COMPOSITE, composite: recolour(mix, b, (e.currentTarget as HTMLInputElement).value) })} />
                  {/if}
                </li>
              {/each}
            </ul>
            {#if refused}<p class="why" role="status" data-composite-refused>{refused}</p>{/if}
          </div>
        {/if}
        <div class="field">
          <span class="k" aria-hidden="true">Exposure</span>
          <Segmented label="Exposure" value={o.shown.exposure} options={o.exposures} onchange={(e: ViewExposure) => ((open = false), onchange({ ...choice, exposure: e }))} />
        </div>
      </div>
    {/if}
  </div>
  {#if legend}<p class="legend" data-composite-legend data-mapped={legend.mapped ? "yes" : "no"}>{legend.text}</p>{/if}
  {#if o.shown.seen === "free" && o.turnable}<p class="hint" data-free-hint>{FREE_HINT}</p>{/if}
</div>

<style>
  .vc {
    display: flex;
    flex-wrap: wrap;
    align-items: flex-end;
    gap: 6px 14px;
    max-width: 100%;
  }
  .field {
    display: flex;
    flex-direction: column;
    gap: 3px;
    min-width: 0;
  }
  .k {
    font-family: var(--sse-font-body);
    font-size: 10px;
    letter-spacing: 0.1em;
    text-transform: uppercase;
    color: var(--sse-ink-muted);
  }
  .field :global(.seg) {
    background: var(--sse-ground-raised);
  }
  .field :global(.seg button) {
    height: 30px;
    padding: 0 10px;
    font-size: 12.5px;
  }
  .fold {
    position: relative;
  }
  .chip {
    height: 30px;
    padding: 0 10px;
    font: inherit;
    font-size: 12.5px;
    white-space: nowrap;
    border: var(--sse-line) solid var(--sse-ink);
    background: var(--sse-ground-raised);
    color: var(--sse-ink);
    cursor: pointer;
  }
  .chip:hover {
    background: var(--sse-ground);
  }
  .chip[aria-expanded="true"] {
    background: var(--sse-ink);
    color: var(--sse-on-ink);
  }
  .pop {
    position: absolute;
    top: calc(100% + 6px);
    left: 0;
    z-index: 6;
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding: 10px 12px;
    background: var(--sse-ground);
    border: var(--sse-line) solid var(--sse-ink);
    box-shadow: 4px 4px 0 var(--sse-ink);
    width: max-content;
    max-width: calc(100vw - 32px);
  }
  .pop .field {
    align-items: flex-start;
  }
  .mix {
    display: flex;
    flex-direction: column;
    gap: 6px;
    border-top: var(--sse-line) solid var(--sse-ink-faint, var(--sse-ink-muted));
    padding-top: 8px;
  }
  .presets {
    display: flex;
    flex-wrap: wrap;
    gap: 4px;
  }
  .preset {
    font: inherit;
    font-size: 12px;
    padding: 2px 8px;
    border: var(--sse-line) solid var(--sse-ink);
    background: var(--sse-ground-raised);
    color: var(--sse-ink);
    cursor: pointer;
  }
  .preset.on {
    background: var(--sse-ink);
    color: var(--sse-on-ink);
  }
  .ticks {
    list-style: none;
    margin: 0;
    padding: 0;
    display: grid;
    grid-template-columns: repeat(2, minmax(0, max-content));
    gap: 4px 16px;
    font-size: 12.5px;
  }
  .ticks li {
    display: flex;
    align-items: center;
    gap: 6px;
    justify-content: space-between;
  }
  .ticks li.quiet {
    color: var(--sse-ink-muted);
  }
  .ticks label {
    display: flex;
    align-items: center;
    gap: 5px;
    cursor: pointer;
  }
  .swatch {
    width: 22px;
    height: 18px;
    padding: 0;
    border: var(--sse-line) solid var(--sse-ink);
    background: none;
  }
  .natural {
    font-size: 11px;
    color: var(--sse-ink-muted);
  }
  .why,
  .legend {
    margin: 0;
    font-size: 12px;
    color: var(--sse-ink-muted);
    max-width: 44ch;
  }
  .legend {
    flex-basis: 100%;
  }
  .hint {
    flex-basis: 100%;
    margin: 0;
    font-size: 12px;
    color: var(--sse-ink-muted);
  }
  /* A phone: the labels go, the rows stay, and the two switches scroll sideways rather than wrap mid-word. */
  .compact .k {
    display: none;
  }
  @media (max-width: 760px) {
    .k {
      display: none;
    }
    /* The chip sits beside View style, so the bar is two rows: View style and Show, then Seen from and Scale. */
    .fold {
      order: -1;
    }
    .vc > .field:first-child {
      order: -2;
    }
    .pop .k {
      display: block;
    }
    /* The fold opens under the whole bar, edge to edge, so nothing in it runs off the side of a phone. */
    .vc {
      position: relative;
    }
    .fold {
      position: static;
    }
    .pop {
      left: 0;
      right: 0;
      width: auto;
      max-width: none;
    }
    .field :global(.seg) {
      overflow-x: auto;
      max-width: 100%;
    }
    .field :global(.seg button) {
      padding: 0 8px;
      flex: none;
    }
  }
</style>

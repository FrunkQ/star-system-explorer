<!-- The standard view controls (interface style rule 12): View style (Chart, 3D, True, Wireframe), Seen from (Above,
     Tilted, Edge-on, Free), Scale (Readable, Real), HUD (Off, On: orbit lines, names and outlines) and Band (Visible, Infrared, Ultraviolet, X-ray,
     Radio, Mag), the same words in the same order on every diagram, for every seat. Options a view or this
     device cannot do stay in place, quiet, with the reason as their title. Free shows how it is worked.
     How often each is used decides where it sits (Alex, 23 Nov: "how often it is used, easy flow and newbie friendly"):
     View style, Seen from and Scale are a press away; HUD and Band, changed far less, fold into one chip that always
     says what they are ("HUD on · Visible ▾") and opens the same two switches beside it. -->
<script lang="ts">
  import Segmented from "./Segmented.svelte";
  import { opensFrom } from "./motion.ts";
  import { EXPOSURE_WORDS, FREE_HINT, VIEW_BAND_WORDS, pickScale, pickSeen, pickStyle, viewOptions, type ViewBand, type ViewExposure, type ViewScale, type SeenFrom, type ViewChoice, type ViewControlsSpec, type ViewStyle } from "../../ui-contract/src/index.ts";

  let { spec, choice, onchange, compact = false }: { spec: ViewControlsSpec; choice: ViewChoice; onchange: (c: ViewChoice) => void; compact?: boolean } = $props();
  const o = $derived(viewOptions(spec, choice));
  const uid = $props.id();
  let open = $state(false);
  let chip = $state<HTMLButtonElement | null>(null);
  let fold = $state<HTMLDivElement | null>(null);
  const summary = $derived(`HUD ${o.shown.hud ? "on" : "off"} · ${VIEW_BAND_WORDS[o.shown.band]}${o.shown.style === "true" ? ` · ${EXPOSURE_WORDS[o.shown.exposure]}` : ""}`);
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

<div class="vc" class:compact data-view-controls data-style={o.shown.style} data-seen={o.shown.seen} data-scale={o.shown.scale} data-hud={o.shown.hud ? "on" : "off"} data-band={o.shown.band} data-exposure={o.shown.exposure}>
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
          <Segmented label="Band" value={o.shown.band} options={o.bands} onchange={(b: ViewBand) => ((open = false), onchange({ ...choice, band: b }))} />
        </div>
        <div class="field">
          <span class="k" aria-hidden="true">Exposure</span>
          <Segmented label="Exposure" value={o.shown.exposure} options={o.exposures} onchange={(e: ViewExposure) => ((open = false), onchange({ ...choice, exposure: e }))} />
        </div>
      </div>
    {/if}
  </div>
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

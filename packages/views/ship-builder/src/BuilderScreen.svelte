<!--
  The ship builder on the Workshop (CM14b): it opens on the first catalogue ship (the default, Alex 27 Nov 11:47:
  "generally they will be working from a catalogue of premade ships"), and "Example ships" in the builder opens any
  other, or a blank build. Each catalogue ship opens as palette parts (convert on open). Nothing is kept here: a build
  is kept with a ship on its card's Build desk pictures.
-->
<script lang="ts">
  import { readPropellants, type DesignShip } from "../../../renderer/src/index.ts";
  import { STOCK_SHIPS, shipRecordOf, type ViewContext } from "../../../ui-contract/src/index.ts";
  import { ShipBuilder } from "../../ship/src/index.ts";

  let { ctx }: { ctx: ViewContext } = $props();
  let pick = $state<string>(STOCK_SHIPS[0]?.id ?? "");
  let ship = $state<DesignShip | null>(null);
  let note = $state<string | null>(null);
  let gen = 0;
  $effect(() => {
    const id = pick;
    const g = ++gen;
    if (!id) return void (ship = null);
    void shipRecordOf(ctx.session, id).then(async (r) => {
      if (g !== gen) return;
      if (!r.ok) return void ((ship = null), (note = r.message));
      const s = r.ship as unknown as DesignShip;
      await readPropellants(ctx.session, s.fuels ?? []).catch(() => ({}));
      if (g === gen) ((note = null), (ship = s));
    });
  });

  // The builder is a working screen: it fills the area it scrolls in, to the bottom edge, as the mock-up does.
  let el = $state<HTMLElement | null>(null);
  let fillH = $state<number | null>(null);
  $effect(() => {
    if (!el) return;
    let box: HTMLElement | null = el.parentElement;
    while (box && !/(auto|scroll)/.test(getComputedStyle(box).overflowY)) box = box.parentElement;
    const host = box ?? document.documentElement;
    const fit = () => el && (fillH = Math.max(420, host.clientHeight - (el.getBoundingClientRect().top - (box ? box.getBoundingClientRect().top : 0)) - 4));
    const ro = new ResizeObserver(fit);
    ro.observe(host);
    fit();
    return () => ro.disconnect();
  });
</script>

<section class="screen" data-builder-screen data-opened={pick} bind:this={el} style={fillH ? `height: ${fillH}px` : ""}>
  {#if note}<p class="note">{note}</p>{/if}
  {#key pick}<ShipBuilder {ctx} {ship} catalogue={STOCK_SHIPS} onopen={(id) => (pick = id)} />{/key}
</section>

<style>
  .screen { display: grid; grid-template-rows: auto minmax(0, 1fr); padding: 0.5em; box-sizing: border-box; min-width: 0; }
  .screen > :global(.builder) { height: 100%; min-height: 0; }
  .note { color: var(--sse-ink-muted); font-size: 0.85em; }
</style>

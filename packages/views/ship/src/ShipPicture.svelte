<script lang="ts">
  // A ship's Chart side view and words. `ship` is the NavShip record with its base-nav fuels; `fuels` says how each
  // fuel is stored (from the master data sheet's substance rows). Inks come from the theme; the shape from physics.
  import { driveFacts, hullFromShip, narrateShip, shipPaths, type Propellants, type ShipRecord } from "../../../renderer/src/index.ts";

  let { ship, fuels = {}, height = 96, words = true }: { ship: ShipRecord; fuels?: Propellants; height?: number; words?: boolean } = $props();

  const width = $derived(height * 3);
  const hull = $derived(hullFromShip(ship, fuels));
  const drive = $derived(driveFacts(ship));
  const paths = $derived(shipPaths(hull, drive, [width, height]));
  const text = $derived(narrateShip(hull, drive));
</script>

<figure class="ship">
  <svg viewBox="0 0 {width} {height}" width="100%" style:max-width="{width}px" role="img" aria-label={text}>
    <path class="plume" d={paths.plume} />
    <path class="hull" d={paths.hull} />
    <path class="part" d={paths.dividers} />
  </svg>
  {#if words}<figcaption>{text}</figcaption>{/if}
</figure>

<style>
  .ship { margin: 0; }
  svg { display: block; height: auto; }
  .plume { fill: var(--sse-accent); fill-opacity: 0.22; stroke: var(--sse-accent); stroke-width: 1; }
  .hull { fill: var(--sse-ground-raised); stroke: var(--sse-ink); stroke-width: 1.25; stroke-linejoin: round; }
  .part { fill: none; stroke: var(--sse-ink-muted); stroke-width: 1; stroke-dasharray: 4 3; }
  figcaption { color: var(--sse-ink-muted); font-size: 0.85em; line-height: 1.4; margin-top: 0.35em; }
</style>

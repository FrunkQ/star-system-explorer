<!--
  The ship builder on the Workshop (CM14a): start from the catalogue (the default, Alex 27 Nov 11:47: "generally they
  will be working from a catalogue of premade ships") or from nothing, then build in the one builder. Each catalogue
  ship opens as palette parts (convert on open). Nothing is kept here: a build is kept with a ship on its card's Build
  desk pictures.
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
</script>

<section class="screen" data-builder-screen>
  <label>Start from
    <select bind:value={pick} data-catalogue>
      {#each STOCK_SHIPS as s (s.id)}<option value={s.id}>{s.name}</option>{/each}
      <option value="">Nothing: a blank build</option>
    </select>
  </label>
  {#if note}<p class="note">{note}</p>{/if}
  {#key pick}<ShipBuilder {ctx} {ship} />{/key}
</section>

<style>
  .screen { display: grid; gap: 0.75em; padding: 0.75em; max-width: 1100px; box-sizing: border-box; min-width: 0; }
  .note { color: var(--sse-ink-muted); font-size: 0.85em; }
</style>

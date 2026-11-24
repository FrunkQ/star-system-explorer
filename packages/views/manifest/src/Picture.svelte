<script lang="ts">
  // A construct's pictures on the Ships and stations card: the shared Pictures panel (views/pictures) with a ship's
  // own drawn views, its plans and 3D model designed from its parts (the renderer's ShipPlans, from the base-ships
  // pack). On the Build desk the design's style, remix and overrides are kept with the construct as a recipe; on
  // Explore and Run the look is fixed (Alex, 24 Nov: play changes what is aboard, never the ship). Stations and sites
  // without a drive have no drawn view yet (the renderer's station builder comes next), so they take own pictures.
  import { DESIGN_FIELD, designOf, factOf, shipRecordOf, type Json, type NavShip, type PlayerSubject, type ViewContext } from "../../../ui-contract/src/index.ts";
  import { readPropellants, type DesignRecipe, type DesignRules, type DesignShip, type Propellants, type ShipStyle } from "../../../renderer/src/index.ts";
  import { ShipPlans } from "../../ship/src/index.ts";
  import { Pictures } from "../../pictures/src/index.ts";

  let { ctx, construct, nav, editing = false, name }: { ctx: ViewContext; construct: PlayerSubject; nav: string | NavShip | null; editing?: boolean; name: string } = $props();
  const session = $derived(ctx.session);
  const ws = $derived(ctx.workspace);
  const design = $derived(designOf(factOf(construct, DESIGN_FIELD)));
  let note = $state<string | null>(null);

  // The base-ships pack's rules and styles, served beside the page (scripts/build-shell.sh).
  type Pack = { rules: DesignRules; styles: Record<string, ShipStyle> };
  let pack = $state<Pack | null>(null);
  const SHIPS_PACK = "packs/base-ships.json";
  $effect(() => {
    void fetch(new URL(SHIPS_PACK, document.baseURI))
      .then((r) => r.json())
      .then((p: { provides: { other: { ship_design: DesignRules; ship_styles: Record<string, ShipStyle> } } }) => (pack = { rules: p.provides.other.ship_design, styles: p.provides.other.ship_styles }))
      .catch(() => (pack = null));
  });

  // The ship's full numbers from the kernel.
  let ship = $state<DesignShip | null>(null);
  let fuels = $state<Propellants>({});
  let gen = 0;
  $effect(() => {
    const n = nav;
    if (!n) return void (ship = null);
    const g = ++gen;
    void shipRecordOf(session, n).then(async (r) => {
      if (g !== gen) return;
      if (!r.ok) return void ((ship = null), (note = r.message));
      const s = r.ship as unknown as DesignShip;
      const f = await readPropellants(session, (s as { fuels?: string[] }).fuels ?? []).catch(() => ({}));
      if (g === gen) ((fuels = f), (ship = s));
    });
  });
  const views = $derived(ship && pack ? [{ id: "plans", label: "Plans" }, { id: "model", label: "3D model" }] : []);

  // The plans report each change of style, remix or override (offered only on Build); the last one is kept, a moment after.
  let pending: ReturnType<typeof setTimeout> | null = null;
  function designChanged(r: DesignRecipe) {
    if (!editing) return;
    if (pending) clearTimeout(pending);
    const keepIt = { style: r.style, seed: r.seed, ...(r.overrides && Object.keys(r.overrides).length ? { overrides: r.overrides } : {}) } as unknown as Json;
    pending = setTimeout(async () => {
      pending = null;
      const s = await session.send({ kind: "set_value", address: construct.address, field: DESIGN_FIELD, value: keepIt, at_yr: ws.campaign_now_yr ?? ws.now_yr });
      if (s.status === "applied") ((note = "Design kept with the ship."), ws.revision++);
      else note = s.status === "refused" ? s.reason : "Sent to the game master.";
    }, 500);
  }
  $effect(() => () => pending && clearTimeout(pending));
</script>

<Pictures {ctx} subject={construct} {name} {editing} {views} hint={views.length ? "Style and remix are kept with the ship." : undefined}>
  {#snippet view(id, choose)}
    {#if ship && pack}
      <ShipPlans {ship} {fuels} rules={pack.rules} styles={pack.styles} segments={24} fit="hull" locked={!editing} start={id === "model" ? "model" : "plans"} onview={choose} {...design && pack.styles[design.style] ? { recipe: design as DesignRecipe } : {}} onchange={designChanged} />
    {/if}
  {/snippet}
</Pictures>
{#if note}<p class="small muted" aria-live="polite" data-design-note>{note}</p>{/if}

<style>
  .small { font-size: 12px; margin: 0; }
  .muted { color: var(--sse-ink-muted); }
</style>

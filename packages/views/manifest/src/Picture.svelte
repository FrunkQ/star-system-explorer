<script lang="ts">
  // A construct's pictures on the Ships and stations card: the shared Pictures panel (views/pictures) with a ship's
  // own drawn views, the ship from its parts (the ship builder, CM14b: a kept build, or its catalogue record opened as
  // parts), or a station's design built from its description (the renderer's StationPicture). On the Build desk the
  // picture is the builder itself and the build is kept with the construct; on Explore and Run the look is fixed
  // (Alex, 24 Nov: play changes what is aboard, never the ship). The old style-and-remix designer is gone (Alex,
  // 28 Nov). Sites with neither take own pictures.
  import { DESIGN_FIELD, buildOf, factOf, shipRecordOf, type Json, type NavShip, type PlayerSubject, type ViewContext } from "../../../ui-contract/src/index.ts";
  import { readPropellants, type BuildRecipe, type DesignRules, type DesignShip, type ShipStyle } from "../../../renderer/src/index.ts";
  import { ShipBuilder } from "../../ship/src/index.ts";
  import StationPicture from "../../ship/src/StationPicture.svelte";
  import { Pictures } from "../../pictures/src/index.ts";

  let {
    ctx,
    construct,
    nav,
    editing = false,
    name,
    station = null,
  }: {
    ctx: ViewContext;
    construct: PlayerSubject;
    nav: string | NavShip | null;
    editing?: boolean;
    name: string;
    /** A station: its template's words and tags pick the shape (renderer StationPicture); its recipe is kept by the card. */
    station?: { words: string; tags: string[]; ondesign: (r: unknown) => void } | null;
  } = $props();
  const session = $derived(ctx.session);
  const ws = $derived(ctx.workspace);
  // A ship built from parts (CM14a) is drawn from its build; on the Build desk the pictures are the builder itself.
  const build = $derived(buildOf(factOf(construct, DESIGN_FIELD)) as BuildRecipe | null);
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
  let gen = 0;
  $effect(() => {
    const n = nav;
    if (!n) return void (ship = null);
    const g = ++gen;
    void shipRecordOf(session, n).then(async (r) => {
      if (g !== gen) return;
      if (!r.ok) return void ((ship = null), (note = r.message));
      const s = r.ship as unknown as DesignShip;
      await readPropellants(session, (s as { fuels?: string[] }).fuels ?? []).catch(() => ({}));
      if (g === gen) ship = s;
    });
  });
  const views = $derived(station ? [{ id: "station", label: "Design" }] : ship && pack ? [{ id: "ship", label: "Ship" }] : []);

  async function keep(value: Json, kept: string) {
    const s = await session.send({ kind: "set_value", address: construct.address, field: DESIGN_FIELD, value, at_yr: ws.campaign_now_yr ?? ws.now_yr });
    if (s.status === "applied") ((note = kept), ws.revision++);
    else note = s.status === "refused" ? s.reason : "Sent to the game master.";
  }
  const keepBuild = (r: BuildRecipe) => keep({ build: r } as unknown as Json, "Build kept with the ship.");
</script>

<Pictures {ctx} subject={construct} {name} {editing} {views} hint={station ? "The shape and remix are kept with it." : views.length ? "Built from parts; the build is kept with the ship." : undefined}>
  {#snippet view(id)}
    {#if id === "station" && station}
      <StationPicture {construct} {name} words={station.words} tags={station.tags} locked={!editing} onchange={(r) => editing && station.ondesign(r)} />
    {:else if ship && pack}
      <ShipBuilder {ctx} {ship} saved={build} readonly={!editing} {...editing ? { onkeep: keepBuild } : {}} />
    {/if}
  {/snippet}
</Pictures>
{#if note}<p class="small muted" aria-live="polite" data-design-note>{note}</p>{/if}

<style>
  .small { font-size: 12px; margin: 0; }
  .muted { color: var(--sse-ink-muted); }
</style>

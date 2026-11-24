<script lang="ts" module>
  // What each construct shows in play this session (Explore and Run choose freely but keep nothing; Build keeps it).
  const chosenHere = new Map<string, string>();
</script>

<script lang="ts">
  // A construct's pictures on the Ships and stations card, with a selector: a ship designed from its own parts seen as
  // plans or as a 3D model (the renderer's ShipPlans, from the base-ships pack), and any pictures the GM gives it
  // (scaled down, raster only). On the Build desk the design's style, remix and overrides are kept as a recipe, pictures
  // are added and removed, and the one last chosen is kept as the one it opens on (Alex, 24 Nov). On Explore and Run
  // the look is fixed (play changes what is aboard, never the ship): any view can be chosen, nothing is kept.
  import { DESIGN_FIELD, PICTURES_FIELD, PICTURES_MAX, SHOWN_FIELD, checkPicture, designOf, factOf, fitSide, picturesOf, shownOf, type Json, type NavShip, type PlayerSubject, type PictureShown, type ViewContext } from "../../../ui-contract/src/index.ts";
  import { readPropellants, type DesignRecipe, type DesignRules, type DesignShip, type Propellants, type ShipStyle } from "../../../renderer/src/index.ts";
  import { ShipPlans } from "../../ship/src/index.ts";

  let { ctx, construct, nav, editing = false, name }: { ctx: ViewContext; construct: PlayerSubject; nav: string | NavShip | null; editing?: boolean; name: string } = $props();
  const session = $derived(ctx.session);
  const ws = $derived(ctx.workspace);

  const pictures = $derived(picturesOf(factOf(construct, PICTURES_FIELD)));
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

  // The ship's full numbers: the kernel resolves a stock id or a pack ship (ship_at_world with no worlds).
  let ship = $state<DesignShip | null>(null);
  let fuels = $state<Propellants>({});
  let gen = 0;
  $effect(() => {
    const n = nav;
    if (!n) return void (ship = null);
    const g = ++gen;
    void session.shipAtWorld({ fixture: "sol", ship: n, worlds: [] }).then(async (r) => {
      if (g !== gen) return;
      if (!r.ok) return void ((ship = null), (note = r.message));
      const s = r.report.ship as unknown as DesignShip;
      const f = await readPropellants(session, (s as { fuels?: string[] }).fuels ?? []).catch(() => ({}));
      if (g === gen) ((fuels = f), (ship = s));
    });
  });

  async function keep(field: string, value: Json): Promise<boolean> {
    const r = await session.send({ kind: "set_value", address: construct.address, field, value, at_yr: ws.campaign_now_yr ?? ws.now_yr });
    if (r.status !== "applied") return (note = r.status === "refused" ? r.reason : r.status === "error" ? r.message : "Sent to the game master."), false;
    ws.revision++;
    return true;
  }

  // The plans report each change of style, remix or override (offered only in edit mode, the Build desk: in play the
  // ship's look is fixed); the last one is kept, a moment after.
  let pending: ReturnType<typeof setTimeout> | null = null;
  function designChanged(r: DesignRecipe) {
    if (!editing) return;
    if (pending) clearTimeout(pending);
    const keepIt = { style: r.style, seed: r.seed, ...(r.overrides && Object.keys(r.overrides).length ? { overrides: r.overrides } : {}) } as unknown as Json;
    pending = setTimeout(() => {
      pending = null;
      void keep(DESIGN_FIELD, keepIt).then((ok) => ok && (note = "Design kept with the ship."));
    }, 500);
  }
  $effect(() => () => pending && clearTimeout(pending));

  // An own picture: scaled so its longer side fits, then WebP (or JPEG where WebP cannot be written), smaller until it fits.
  async function shrink(file: File): Promise<string> {
    const bmp = await createImageBitmap(file);
    const [w, h] = fitSide(bmp.width, bmp.height);
    const c = document.createElement("canvas");
    c.width = w;
    c.height = h;
    c.getContext("2d")!.drawImage(bmp, 0, 0, w, h);
    bmp.close();
    let out = "";
    for (const q of [0.85, 0.7, 0.55, 0.4]) {
      out = c.toDataURL("image/webp", q);
      if (!out.startsWith("data:image/webp")) out = c.toDataURL("image/jpeg", q);
      if (checkPicture(out) === null) break;
    }
    return out;
  }
  let busy = $state(false);
  async function give(file: File | undefined) {
    if (!file) return;
    busy = true;
    try {
      const uri = await shrink(file);
      const why = checkPicture(uri);
      if (why) return void (note = `Not kept: ${why}.`);
      const list = [...pictures, uri];
      if (await keep(PICTURES_FIELD, list as unknown as Json)) await choose(`picture:${list.length - 1}`, "Picture kept with it; it opens on this one.");
    } catch {
      note = "That file could not be read as a picture.";
    } finally {
      busy = false;
    }
  }
  async function takeAway(i: number) {
    const list = pictures.filter((_, j) => j !== i);
    if (!(await keep(PICTURES_FIELD, list as unknown as Json))) return;
    chosenHere.delete(construct.address);
    if (shown === `picture:${i}` || (shown?.startsWith("picture:") && Number(shown.slice(8)) > i)) await keep(SHOWN_FIELD, (designed ? "plans" : list.length ? "picture:0" : "") as Json);
    note = "Picture taken away.";
  }

  // Which one shows: this session's choice in play, else the one kept with the construct, else the plans or first picture.
  const designed = $derived(!!(nav && ship && pack));
  let tick = $state(0);
  const shown = $derived.by(() => {
    void tick;
    const here = chosenHere.get(construct.address);
    return (here && shownOf(here, pictures.length, designed)) || shownOf(factOf(construct, SHOWN_FIELD), pictures.length, designed);
  });
  async function choose(s: PictureShown, said?: string) {
    chosenHere.set(construct.address, s);
    tick++;
    if (editing && factOf(construct, SHOWN_FIELD) !== s && (await keep(SHOWN_FIELD, s))) note = said ?? "It opens on this one from now on.";
    else if (said) note = said;
  }
  const options = $derived<{ id: PictureShown; label: string; src?: string }[]>([
    ...(designed ? ([{ id: "plans", label: "Plans" }, { id: "model", label: "3D model" }] as const) : []),
    ...pictures.map((src, i) => ({ id: `picture:${i}` as PictureShown, label: `Picture ${i + 1}`, src })),
  ]);
  const shownPicture = $derived(shown?.startsWith("picture:") ? pictures[Number(shown.slice(8))] : undefined);
</script>

{#if shown || editing}
  <section class="picture" aria-label="Picture" data-picture={shown ?? "none"}>
    {#if options.length > 1}
      <div class="pick" role="group" aria-label="Show">
        {#each options as o (o.id)}
          <button type="button" class="chip" aria-pressed={shown === o.id} onclick={() => choose(o.id)} data-show={o.id} title={o.label}>
            {#if o.src}<img src={o.src} alt={o.label} />{:else}{o.label}{/if}
          </button>
        {/each}
      </div>
    {/if}
    {#if shownPicture}
      <img class="own" src={shownPicture} alt={`${name}, the game master's picture`} data-own-picture />
    {:else if (shown === "plans" || shown === "model") && ship && pack}
      {#key `${construct.address} ${shown}`}
        <ShipPlans {ship} {fuels} rules={pack.rules} styles={pack.styles} segments={24} fit="hull" locked={!editing} start={shown} onview={(v) => choose(v)} {...design && pack.styles[design.style] ? { recipe: design as DesignRecipe } : {}} onchange={designChanged} />
      {/key}
    {/if}
    {#if editing}
      <div class="row small">
        {#if pictures.length < PICTURES_MAX}<label class="quiet-btn file" class:busy>Add a picture…<input type="file" accept="image/png,image/jpeg,image/webp" disabled={busy} onchange={(e) => give(e.currentTarget.files?.[0])} data-picture-file /></label>{/if}
        {#if shownPicture}<button type="button" class="quiet-btn" onclick={() => takeAway(Number(shown!.slice(8)))} data-picture-clear>Remove this picture</button>{/if}
        <span class="muted">{designed ? "Style and remix are kept with the ship; it" : "It"} opens on the one last chosen here.</span>
      </div>
    {/if}
    {#if note}<p class="small muted" aria-live="polite" data-picture-note>{note}</p>{/if}
  </section>
{/if}

<style>
  .picture { margin: 8px 0; display: flex; flex-direction: column; gap: 6px; min-width: 0; }
  .pick { display: flex; flex-wrap: wrap; gap: 6px; }
  .chip { font: inherit; font-size: 12px; min-height: 34px; padding: 0 10px; border: 1px solid var(--sse-rule); background: transparent; color: var(--sse-ink); cursor: pointer; display: inline-flex; align-items: center; }
  .chip[aria-pressed="true"] { border-color: var(--sse-accent); box-shadow: inset 0 -2px 0 var(--sse-accent); }
  .chip img { height: 26px; width: auto; max-width: 48px; object-fit: cover; display: block; }
  img.own { display: block; max-width: 100%; max-height: 320px; object-fit: contain; border: 1px solid var(--sse-rule); background: var(--sse-ground-raised, var(--sse-ground)); }
  .row { display: flex; flex-wrap: wrap; gap: 8px 12px; align-items: center; }
  .quiet-btn { font: inherit; min-height: 34px; padding: 0 10px; border: 1px solid var(--sse-rule); background: transparent; color: var(--sse-ink); cursor: pointer; display: inline-flex; align-items: center; transition: transform var(--sse-motion-quick, 120ms) var(--sse-ease, ease-out); }
  .quiet-btn:hover { transform: translateY(-1px); border-color: var(--sse-ink); }
  .file input { position: absolute; width: 1px; height: 1px; opacity: 0; pointer-events: none; }
  .file { position: relative; }
  .busy { opacity: 0.5; }
  .small { font-size: 12px; margin: 0; }
  .muted { color: var(--sse-ink-muted); }
  @media (prefers-reduced-motion: reduce) {
    .quiet-btn { transition: none; }
  }
</style>

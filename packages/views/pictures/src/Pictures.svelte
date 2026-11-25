<script lang="ts" module>
  // What each subject shows in play this session (Explore and Run choose freely but keep nothing; Build keeps it).
  const chosenHere = new Map<string, string>();
</script>

<script lang="ts">
  // Pictures of anything with an address, with a selector (Alex, 24 Nov): the views its owner draws from data, passed
  // in as `views` and drawn by the `view` snippet (a ship's plans and 3D model, a world's chart and globe), and up to
  // four pictures the GM gives it (scaled down, raster only). On the Build desk (`editing`) pictures are added and
  // removed and the one last chosen is kept as the one it opens on, everywhere. On Explore and Run any of them can be
  // chosen, nothing is kept and nothing changes (play changes what is aboard, never the thing itself).
  import type { Snippet } from "svelte";
  import { PICTURES_FIELD, PICTURES_MAX, SHOWN_FIELD, checkPicture, fitSide, picturesOf, shownOf, type Json, type PictureShown, type ViewContext } from "../../../ui-contract/src/index.ts";

  let {
    ctx,
    subject,
    name,
    editing = false,
    views = [],
    view,
    hint,
  }: {
    ctx: ViewContext;
    /** The thing pictured: its address and facts (a construct or a body as the scene gives it). */
    subject: { address: string; facts: readonly { field: string; value?: unknown }[] };
    name: string;
    editing?: boolean;
    /** The drawn views, in the order the selector offers them; the first is the default. */
    views?: readonly { id: string; label: string }[];
    /** Draws one view; `choose` lets the view report a switch made inside it (a plans/model toggle of its own). */
    view?: Snippet<[string, (id: string) => void]>;
    /** A line beside the Build controls, about what else is kept (a ship's style and remix). */
    hint?: string;
  } = $props();
  const session = $derived(ctx.session);
  const ws = $derived(ctx.workspace);
  const factOf = (f: string) => subject.facts.find((x) => x.field === f)?.value;

  const pictures = $derived(picturesOf(factOf(PICTURES_FIELD)));
  const viewIds = $derived(views.map((v) => v.id));
  let note = $state<string | null>(null);

  async function keep(field: string, value: Json): Promise<boolean> {
    const r = await session.send({ kind: "set_value", address: subject.address, field, value, at_yr: ws.campaign_now_yr ?? ws.now_yr });
    if (r.status !== "applied") return (note = r.status === "refused" ? r.reason : r.status === "error" ? r.message : "Sent to the game master."), false;
    ws.revision++;
    return true;
  }

  // Which one shows: this session's choice in play, else the one kept with the subject, else the first view or picture.
  let tick = $state(0);
  const shown = $derived.by(() => {
    void tick;
    const here = chosenHere.get(subject.address);
    return (here && shownOf(here, pictures.length, viewIds)) || shownOf(factOf(SHOWN_FIELD), pictures.length, viewIds);
  });
  async function choose(s: PictureShown, said?: string) {
    chosenHere.set(subject.address, s);
    tick++;
    if (editing && factOf(SHOWN_FIELD) !== s && (await keep(SHOWN_FIELD, s))) note = said ?? "It opens on this one from now on.";
    else if (said) note = said;
  }
  const options = $derived([...views.map((v) => ({ id: v.id, label: v.label, src: undefined as string | undefined })), ...pictures.map((src, i) => ({ id: `picture:${i}`, label: `Picture ${i + 1}`, src }))]);
  const shownPicture = $derived(shown?.startsWith("picture:") ? pictures[Number(shown.slice(8))] : undefined);

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
    chosenHere.delete(subject.address);
    const was = shown;
    if (was?.startsWith("picture:") && Number(was.slice(8)) >= i) await keep(SHOWN_FIELD, (viewIds[0] ?? (list.length ? "picture:0" : "")) as Json);
    note = "Picture taken away.";
  }
</script>

{#if shown || editing}
  <section class="picture" aria-label="Picture" data-picture={shown ?? "none"}>
    {#if options.length > 1}
      <div class="pick" role="group" aria-label="Picture to show">
        {#each options as o (o.id)}
          <button type="button" class="chip" aria-pressed={shown === o.id} onclick={() => choose(o.id)} data-show={o.id} title={o.label}>
            {#if o.src}<img src={o.src} alt={o.label} />{:else}{o.label}{/if}
          </button>
        {/each}
      </div>
    {/if}
    {#if shownPicture}
      <img class="own" src={shownPicture} alt={`${name}, the game master's picture`} data-own-picture />
    {:else if shown && view && viewIds.includes(shown)}
      {#key `${subject.address} ${shown}`}{@render view(shown, (id) => choose(id))}{/key}
    {/if}
    {#if editing}
      <div class="row small">
        {#if pictures.length < PICTURES_MAX}<label class="quiet-btn file" class:busy>Add a picture…<input type="file" accept="image/png,image/jpeg,image/webp" disabled={busy} onchange={(e) => give(e.currentTarget.files?.[0])} data-picture-file /></label>{/if}
        {#if shownPicture}<button type="button" class="quiet-btn" onclick={() => takeAway(Number(shown!.slice(8)))} data-picture-clear>Remove this picture</button>{/if}
        <span class="muted">{hint ? `${hint} ` : ""}It opens on the one last chosen here.</span>
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

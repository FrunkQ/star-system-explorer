<script lang="ts">
  // One postcard (PC3, postcards design section 5): the front drawn live from its data, the back laid out like the back
  // of a postcard (left the caption and why it looks like this; right where, when, how, who, the message and what it
  // travels with; foot Go there, Send, Save picture). Presentation only.
  import { untrack } from "svelte";
  import { COMPOSITE, COMPOSITE_WORD, creditWords, goesWith, packsNeeded, sayNumber, SCALE_WORDS, VIEW_BAND_WORDS, VIEW_STYLE_WORDS, STAMP_SIDE, stampOf, type Postcard, type SystemScene, type ViewContext } from "../../../ui-contract/src/index.ts";
  import { dateWords, defaultCalendar, formatYears, fromPackCalendar } from "../../../chart-grammar/src/index.ts";
  import { drawStill, stillBlocker } from "./still.ts";
  import Send from "./Send.svelte";

  let {
    ctx,
    scene,
    card,
    stamp = null,
    visiting = false,
    reachable = true,
    newer = null,
    packs = [],
    shop = () => null,
    ongo,
    onkeep,
    onstamp,
    onpack,
  }: {
    ctx: ViewContext;
    scene: SystemScene;
    card: Postcard;
    /** A kept picture for this card: the envelope's, or one kept on this device for a campaign postcard. */
    stamp?: string | null;
    /** Opened from outside: read-only, and the back says whose it is. */
    visiting?: boolean;
    /** Go there can open its place here. */
    reachable?: boolean;
    /** The kernel version here, when it is newer than the one it was made with. */
    newer?: string | null;
    /** The packs in force here, for the list of packs it needs. */
    packs?: readonly { id: string; version: string }[];
    shop?: (id: string) => "free" | "paid" | null;
    ongo?: () => void;
    /** "Keep this system": the ordinary paste of its clip. */
    onkeep?: () => void;
    /** A stamp was drawn here (the wall keeps it on this device). */
    onstamp?: (uri: string) => void;
    /** The envelope to send: a campaign postcard is packed for sending first. */
    onpack?: () => Promise<Postcard | string>;
  } = $props();

  let side = $state<"front" | "back">("front");
  let canvasHost = $state<HTMLElement | undefined>();
  let progress = $state<string | null>(null);
  let drawn = $state<HTMLCanvasElement | null>(null);
  let note = $state<string | null>(null);
  let why = $state<string | null>(null);
  let sending = $state(false);
  const blocker = $derived(stillBlocker(card, scene));

  $effect(() => {
    const c = card;
    const host = canvasHost;
    if (!host) return;
    untrack(() => void draw(c, host));
  });
  let gen = 0;
  async function draw(c: Postcard, host: HTMLElement) {
    const g = ++gen;
    drawn = null;
    why = stillBlocker(c, scene);
    if (why) return;
    progress = "Drawing…";
    try {
      const r = await drawStill(ctx, scene, c, 1024, { onProgress: (d, n) => g === gen && (progress = `Refining ${d} of ${n}…`) });
      if (g !== gen) return;
      if (!r.ok) return void (why = r.why);
      r.photo.canvas.className = "front-canvas";
      r.photo.canvas.setAttribute("data-postcard-drawn", "");
      host.replaceChildren(r.photo.canvas);
      drawn = r.photo.canvas;
      note = r.photo.note;
      const s = stampOf(r.photo.canvas, STAMP_SIDE);
      if (s) onstamp?.(s);
    } catch (e) {
      if (g === gen) why = `No picture: ${e instanceof Error ? e.message : String(e)}`;
    } finally {
      if (g === gen) progress = null;
    }
  }

  const b = $derived(card.bookmark);
  const calendar = $derived(ctx.workspace.pack_calendar ? fromPackCalendar(ctx.workspace.pack_calendar, ctx.workspace.birth_s ?? null) : defaultCalendar(ctx.workspace.campaign_epoch_yr ?? ctx.workspace.campaign_now_yr ?? ctx.workspace.now_yr));
  const where = $derived.by(() => {
    const about = b.pose?.about ?? b.selected;
    const w = about ? scene.subjects.find((s) => s.address === about)?.label : null;
    return b.scope === scene.system ? (w ? `${w}, ${scene.title}` : scene.title) : (w ?? card.name);
  });
  // In this system, a date on its calendar; from elsewhere, the universe's own clock (its calendar is not ours).
  const when = $derived(b.scope === scene.system ? dateWords(b.at_yr, calendar, true) : `${formatYears(b.at_yr)} on its system's clock`);
  const view = $derived(b.view);
  const how = $derived.by(() => {
    const parts = [view ? VIEW_STYLE_WORDS[view.style] : null, `${view?.band === COMPOSITE ? COMPOSITE_WORD : VIEW_BAND_WORDS[view?.band ?? "visible"]} band`, `${SCALE_WORDS[view?.scale ?? "readable"]} scale`, b.pose ? `${sayNumber(b.pose.fov_deg, 2)}° across` : null].filter(Boolean);
    return parts.join(", ");
  });
  const exposure = $derived(b.pose?.exposure.mode === "eye" ? "As an eye would see it." : b.pose ? "A long exposure: to the eye the night side would be darker." : null);
  const goes = $derived(goesWith(card));
  const needed = $derived(packsNeeded(card, packs, shop));
  const credit = $derived(creditWords(card.credit));
  const STATE_WORDS: Record<string, string> = { have: "you have it", "other-version": "you have another version", free: "free to get", paid: "in the shop", unknown: "not found" };

  function savePicture() {
    const c = drawn;
    if (!c) return;
    c.toBlob((blob) => {
      if (!blob) return void (note = "The browser would not make the picture.");
      const a = document.createElement("a");
      a.href = URL.createObjectURL(blob);
      a.download = `${card.name.replace(/[^\w -]+/g, "").trim() || "postcard"}.png`;
      a.click();
      setTimeout(() => URL.revokeObjectURL(a.href), 1000);
    }, "image/png");
  }
</script>

<article class="card" data-postcard={card.id} data-side={side} aria-label="Postcard: {card.name}">
  <div class="face front" hidden={side !== "front"}>
    <div class="picture" bind:this={canvasHost}>
      {#if !drawn && stamp}<img class="stamp" src={stamp} alt="Kept picture of {card.name}" data-postcard-stamp />{/if}
    </div>
    {#if progress}<div class="refine" data-postcard-progress>{progress}</div>{/if}
    {#if why && !progress}<p class="why" data-postcard-why>{stamp ? "Kept picture. " : ""}{why}</p>{/if}
    <div class="corner">
      <button type="button" class="ctl" data-postcard-turn onclick={() => (side = "back")}>Turn over</button>
      {#if reachable && ongo}<button type="button" class="ctl" data-postcard-go onclick={ongo}>Go there</button>{/if}
      <button type="button" class="ctl" data-postcard-send onclick={() => (sending = true)}>Send</button>
    </div>
  </div>
  <div class="face back" hidden={side !== "back"} data-postcard-back>
    <div class="left">
      <h3>{card.name}</h3>
      {#if card.caption}<p class="caption">{card.caption}</p>{/if}
      <h4>Why it looks like this</h4>
      <ul class="reasons">
        {#if note}<li>{note}</li>{/if}
        {#if view?.scale === "readable"}<li>Sizes exaggerated: taken at Readable scale.</li>{/if}
        <li class="quiet">The reasons pass by pass, each with its why?, come with the renderer's reason lines.</li>
      </ul>
    </div>
    <div class="right">
      {#if visiting}<p class="visiting" data-postcard-visiting>Visiting: a postcard{credit ? ` from ${card.credit.at(-1)}` : ""}</p>{/if}
      {#if card.message}<p class="message" data-postcard-message>{card.message.text}{#if card.message.from}<br /><span class="from">{card.message.from}</span>{/if}</p>{/if}
      <dl>
        <dt>Where</dt><dd>{where}</dd>
        <dt>When</dt><dd>{when}</dd>
        <dt>How</dt><dd>{how}{exposure ? `. ${exposure}` : ""}</dd>
        {#if credit}<dt>Who</dt><dd>{credit}</dd>{/if}
        <dt>Travels as</dt><dd data-postcard-goes>{goes.words}</dd>
      </dl>
      {#if needed.length}
        <ul class="packs" data-postcard-packs>
          {#each needed as n (n.id)}<li data-pack-state={n.state}>{n.id} {n.version}: {STATE_WORDS[n.state]}{#if n.state !== "have"}{" "}<span class="quiet">(drawn without it, so it may differ)</span>{/if}</li>{/each}
        </ul>
      {/if}
      {#if newer}<p class="quiet" data-postcard-newer>Made with {card.world.how !== "here" ? card.world.made_with : ""}; the physics here is {newer}, so it may look different now. The kept picture shows how it looked when taken.</p>{/if}
    </div>
    <div class="foot">
      <button type="button" class="ctl" onclick={() => (side = "front")}>Turn over</button>
      {#if reachable && ongo}<button type="button" class="ctl" onclick={ongo}>Go there</button>{/if}
      {#if onkeep}<button type="button" class="ctl" data-postcard-keep onclick={onkeep}>Keep this system</button>{/if}
      <button type="button" class="ctl" onclick={() => (sending = true)}>Send</button>
      <button type="button" class="ctl" data-postcard-save disabled={!drawn} title={drawn ? "Save the picture as a PNG" : (blocker ?? "Still drawing")} onclick={savePicture}>Save picture</button>
    </div>
  </div>
  {#if sending}<Send {card} {stamp} {onpack} onclose={() => (sending = false)} />{/if}
</article>

<style>
  .card {
    position: relative;
    display: block;
    max-width: 960px;
    margin: 0 auto;
    color: var(--sse-ink);
    font-family: var(--sse-font-body);
  }
  .face {
    position: relative;
    border: var(--sse-line) solid var(--sse-ink);
    border-radius: var(--sse-radius);
    background: var(--sse-ground-raised);
    overflow: hidden;
  }
  .front {
    aspect-ratio: 1 / 1;
    max-height: 70vh;
    background: #000;
  }
  .picture {
    position: absolute;
    inset: 0;
    display: grid;
    place-items: center;
  }
  .picture :global(canvas.front-canvas),
  .picture img {
    width: 100%;
    height: 100%;
    object-fit: contain;
  }
  .refine {
    position: absolute;
    left: 0;
    right: 0;
    bottom: 0;
    padding: 2px 8px;
    font-size: 12px;
    color: #ddd;
    border-top: 2px solid var(--sse-accent);
    background: rgb(0 0 0 / 0.4);
  }
  .why {
    position: absolute;
    left: 12px;
    right: 12px;
    top: 40%;
    text-align: center;
    color: #eee;
    font-size: 14px;
  }
  .corner {
    position: absolute;
    right: 8px;
    top: 8px;
    display: flex;
    gap: 6px;
  }
  .corner .ctl {
    background: rgb(0 0 0 / 0.45);
    color: #fff;
    border-color: rgb(255 255 255 / 0.5);
  }
  .ctl {
    font: inherit;
    font-size: 13px;
    padding: 4px 10px;
    border: var(--sse-line) solid var(--sse-ink);
    border-radius: var(--sse-radius);
    background: transparent;
    color: inherit;
    cursor: pointer;
  }
  .ctl:disabled {
    opacity: 0.5;
    cursor: default;
  }
  .back {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 16px;
    padding: 16px;
    min-height: 320px;
    align-content: start;
  }
  .back .left {
    border-right: var(--sse-line) solid var(--sse-ink);
    padding-right: 16px;
  }
  h3 {
    margin: 0 0 6px;
    font-family: var(--sse-font-display, var(--sse-font-body));
  }
  h4 {
    margin: 12px 0 4px;
    font-size: 13px;
  }
  .caption {
    font-style: italic;
  }
  .reasons {
    padding-left: 18px;
    margin: 0;
    font-size: 13px;
  }
  .quiet {
    opacity: 0.7;
    font-size: 12px;
  }
  .message {
    font-family: var(--sse-font-display, var(--sse-font-body));
    font-size: 18px;
    margin: 0 0 12px;
    white-space: pre-wrap;
  }
  .from {
    font-size: 14px;
  }
  .visiting {
    font-weight: 600;
    margin: 0 0 8px;
  }
  dl {
    display: grid;
    grid-template-columns: auto 1fr;
    gap: 2px 10px;
    margin: 0;
    font-size: 13px;
  }
  dt {
    font-weight: 600;
  }
  dd {
    margin: 0;
  }
  .packs {
    font-size: 12px;
    padding-left: 18px;
  }
  .foot {
    grid-column: 1 / -1;
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    align-items: center;
    border-top: var(--sse-line) solid var(--sse-ink);
    padding-top: 10px;
  }
  @media (max-width: 640px) {
    .back {
      grid-template-columns: 1fr;
    }
    .back .left {
      border-right: none;
      padding-right: 0;
    }
  }
</style>

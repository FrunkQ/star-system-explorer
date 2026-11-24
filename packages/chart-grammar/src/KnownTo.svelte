<!--
  The "Known to" control (secrets and visibility design 4 and 5): one eye that is also the button. Press it and the
  table knows (or, on an open eye, it goes back to GM only); a slip says so, with Undo. While players are connected a
  reveal waits two seconds before it is sent, so Undo then means they never saw it. Hold the eye, right-click it, or
  press the caret beside it, and the Known to popup opens from the eye: one row of three steps per audience, the
  exceptions on this thing's facts, and how far a change reaches. On a phone the popup is a sheet from the foot.
  Everything is sent through `onset`, one set_known each; this component reads and holds, it never decides.
-->
<script lang="ts">
  import KnownEye from "./KnownEye.svelte";
  import { opensFrom } from "./motion.ts";
  import { LANDMARKS, LANDMARK_MEANS, LANDMARK_WORDS, fieldLandmark, holdReveal, lineChange, pressedLandmark, tableOf, type Held, type KnownChange, type KnownLandmark, type KnownTo } from "../../ui-contract/src/index.ts";

  let {
    read,
    label,
    field,
    fieldLabel,
    size = 14,
    hold = false,
    children = false,
    onset,
    onundo,
  }: {
    read: KnownTo;
    /** The thing's name, for the popup and the press target's label. */
    label: string;
    /** A card line's field: the eye then shows and sets that fact's exception. */
    field?: string;
    fieldLabel?: string;
    size?: number;
    /** Players are connected: hold a reveal two seconds. */
    hold?: boolean;
    /** The thing has things inside it, so "and what is inside it" is offered. */
    children?: boolean;
    /** Send one change; resolves to null when applied, or the reason it was not. */
    onset: (c: KnownChange) => Promise<{ ok: true; undo?: string } | { ok: false; why: string }>;
    /** Put back exactly what one applied change did (kernel undo_known), by the id `onset` gave. Without it, Undo
     * sends the opposite change. */
    onundo?: (undo: string) => Promise<string | null>;
  } = $props();

  const table = $derived(tableOf(read));
  const own = $derived(table ? (field ? fieldLandmark(table, field) : table.landmark) : "known");
  // A press shows at once (the eye opens, the thing inks in) and is sent when the hold ends.
  let shown = $state<KnownLandmark | null>(null);
  const eye = $derived<KnownLandmark>(shown ?? own);
  // A read that arrives while a press is held or on its way is older than the press: it must not put the eye back.
  let pending = 0;
  $effect(() => {
    void read;
    if (!pending) shown = null;
  });

  let slip = $state<{ words: string; undo: (() => void) | null } | null>(null);
  let held: Held | null = null;
  let slipTimer: ReturnType<typeof setTimeout> | undefined;
  function say(words: string, undo: (() => void) | null) {
    slip = { words, undo };
    clearTimeout(slipTimer);
    slipTimer = setTimeout(() => (slip = null), 6000);
  }
  const who = $derived(table ? (table.audience === "everyone" ? "the table" : table.label) : "the table");
  const what = $derived(field ? (fieldLabel ?? field).toLowerCase() : label);

  async function send(c: KnownChange): Promise<string | undefined> {
    pending++;
    const r = await onset(c).finally(() => pending--);
    if (!r.ok) {
      shown = null;
      say(`Not changed: ${r.why}`, null);
      return undefined;
    }
    return r.undo;
  }

  function press() {
    if (!table) return;
    const from = eye;
    const to = pressedLandmark(from);
    const c: KnownChange = { audience: table.audience, landmark: field ? lineChange(table, field, to) : to, ...(field ? { field } : {}), scope: "this" };
    shown = to;
    held?.flush();
    pending++;
    let sentAs: Promise<string | undefined> | null = null;
    const h = holdReveal(() => void ((sentAs = send(c)), pending--), { hold: hold && to !== "gm_only" });
    held = h;
    const back = () => {
      if (h.undo()) {
        pending--;
        shown = from;
        say(`Undone. ${who[0]!.toUpperCase()}${who.slice(1)} never saw it.`, null);
      } else {
        shown = from;
        const reverse = () => void send({ ...c, landmark: field ? lineChange(table!, field, from) : from });
        void (sentAs ?? Promise.resolve(undefined)).then((id) => (id && onundo ? onundo(id).then((why) => why && reverse()) : reverse()));
        say(from === "gm_only" ? `Hidden again. ${who[0]!.toUpperCase()}${who.slice(1)} has already seen it.` : `Put back as before.`, null);
      }
    };
    say(to === "known" ? `Shown to ${who}.` : to === "gm_only" ? `Hidden from ${who}.` : `${who[0]!.toUpperCase()}${who.slice(1)} ${who === "the table" ? "has" : "have"} heard of it.`, back);
  }

  // The popup.
  let open = $state(false);
  let btn = $state<HTMLButtonElement | null>(null);
  let root = $state<HTMLElement | null>(null);
  let scope = $state<"this" | "children">("this");
  function choose(audience: string, l: KnownLandmark) {
    shown = audience === table?.audience ? l : shown;
    void send({ audience, landmark: l, ...(field ? { field } : {}), scope });
  }
  function dropException(audience: string, f: string) {
    void send({ audience, landmark: null, field: f, scope: "this" });
  }
  function close() {
    open = false;
    btn?.focus();
  }
  let holdTimer: ReturnType<typeof setTimeout> | undefined;
  let longPressed = false;
  function down(e: PointerEvent) {
    if (e.button !== 0) return;
    longPressed = false;
    holdTimer = setTimeout(() => ((longPressed = true), (open = true)), 500);
  }
  const up = () => clearTimeout(holdTimer);
  const id = $props.id();
</script>

<svelte:window onkeydowncapture={(e) => open && e.key === "Escape" && (e.stopPropagation(), close())} onpointerdown={(e) => open && !root?.contains(e.target as Node) && (open = false)} />

<span class="known" bind:this={root} data-known={eye} data-known-field={field}>
  <button
    type="button"
    class="eyebtn"
    bind:this={btn}
    aria-label="{LANDMARK_WORDS[eye]}{field ? `: ${what}` : ''}. Press to {eye === 'known' ? `hide it from ${who}` : `show it to ${who}`}"
    title="Known to: {LANDMARK_WORDS[eye]}{table && table.rung_label && table.rung_label !== LANDMARK_WORDS[eye] ? ` (${table.rung_label})` : ''}"
    data-known-eye
    onclick={() => (longPressed ? (longPressed = false) : press())}
    onpointerdown={down}
    onpointerup={up}
    onpointerleave={up}
    oncontextmenu={(e) => (e.preventDefault(), up(), (open = true))}>
    <KnownEye landmark={eye} {size} />
  </button>
  <button type="button" class="caret" aria-label="Known to: who knows {what}" aria-expanded={open} aria-controls="{id}-pop" data-known-open onclick={() => (open = !open)}>
    <svg width="8" height="8" viewBox="0 0 8 8" aria-hidden="true"><path d="M1 2.5 4 5.5 7 2.5" /></svg>
  </button>
  {#if slip}
    <span class="slip" role="status" data-known-slip>
      {slip.words}
      {#if slip.undo}<button type="button" class="link" data-known-undo onclick={() => (slip?.undo?.(), slip && (slip.undo = null))}>Undo</button>{/if}
    </span>
  {/if}
  {#if open}
    <div class="pop" id="{id}-pop" role="dialog" aria-label="Known to: {what}" use:opensFrom={() => btn} data-known-pop>
      <div class="phead">
        <span class="k">Known to</span>
        <span class="what">{field ? `${label} · ${fieldLabel ?? field}` : label}</span>
        <button type="button" class="x" aria-label="Close Known to" onclick={close}>×</button>
      </div>
      {#each read.audiences as a (a.audience)}
        {@const at = a.audience === table?.audience && shown ? shown : field ? fieldLandmark(a, field) : a.landmark}
        <div class="row" role="radiogroup" aria-label="{a.label}" data-known-audience={a.audience}>
          {#if read.audiences.length > 1}<span class="who">{a.label}</span>{/if}
          {#each LANDMARKS as l (l)}
            <button type="button" role="radio" class="step" aria-checked={at === l} title={LANDMARK_MEANS[l]} data-known-step={l} onclick={() => choose(a.audience, l)}>
              <KnownEye landmark={l} size={11} />
              <span>{LANDMARK_WORDS[l]}</span>
            </button>
          {/each}
          {#if !field && a.rung_label && a.rung_label !== LANDMARK_WORDS[a.landmark]}<span class="rung" data-known-rung>{a.rung_label}</span>{/if}
        </div>
        {#if !field && a.exceptions.length}
          <div class="exc" data-known-exceptions>
            <span class="k">Exceptions</span>
            {#each a.exceptions as x (x.field)}
              <span class="x1">{x.field.replace(/[_.]/g, " ")}: {x.landmark === "gm_only" ? "kept back" : x.landmark === "known" ? "shown early" : "heard of"} <button type="button" class="link" onclick={() => dropException(a.audience, x.field)}>follow the {label.length > 18 ? "thing" : label}</button></span>
            {/each}
          </div>
        {/if}
        {#if a.cover.length}
          <div class="exc"><span class="k">Told instead</span>{#each a.cover as c (c.field)}<span class="x1">{c.field.replace(/[_.]/g, " ")}: "{c.told}"</span>{/each}</div>
        {/if}
      {/each}
      {#if !field && children}
        <div class="also" role="radiogroup" aria-label="How far it reaches">
          <span class="k">Also</span>
          <button type="button" role="radio" class="step" aria-checked={scope === "this"} onclick={() => (scope = "this")}>Just this</button>
          <button type="button" role="radio" class="step" aria-checked={scope === "children"} data-known-scope="children" onclick={() => (scope = "children")}>And what is inside it</button>
        </div>
      {/if}
    </div>
  {/if}
</span>

<style>
  .known {
    position: relative;
    display: inline-flex;
    align-items: center;
    vertical-align: middle;
  }
  /* The press target stays at least 24 px (44 px on touch) whatever the drawn size. */
  .eyebtn,
  .caret {
    display: inline-grid;
    place-items: center;
    min-width: 24px;
    min-height: 24px;
    height: auto;
    padding: 0;
    border: none;
    background: none;
    color: var(--sse-ink);
    cursor: pointer;
  }
  .caret {
    min-width: 14px;
    color: var(--sse-ink-muted);
    opacity: 0;
  }
  .caret svg {
    fill: none;
    stroke: currentColor;
    stroke-width: 1.4;
  }
  .known:hover .caret,
  .known:focus-within .caret,
  .caret[aria-expanded="true"] {
    opacity: 1;
  }
  @media (pointer: coarse) {
    .eyebtn {
      min-width: 44px;
      min-height: 44px;
    }
    .caret {
      opacity: 1;
      min-height: 44px;
    }
  }
  .eyebtn:focus-visible,
  .caret:focus-visible,
  .step:focus-visible {
    outline: 2px solid var(--sse-accent);
    outline-offset: 1px;
  }
  .slip {
    position: absolute;
    left: 0;
    top: 100%;
    z-index: 20;
    white-space: nowrap;
    padding: 3px 8px;
    border: var(--sse-line) solid var(--sse-ink);
    background: var(--sse-ground-raised);
    box-shadow: var(--sse-card-shadow);
    font: 12px var(--sse-font-body);
    color: var(--sse-ink);
  }
  .link {
    height: auto;
    padding: 0 0 0 6px;
    border: none;
    background: none;
    color: var(--sse-accent-ink);
    font: inherit;
    text-decoration: underline;
    cursor: pointer;
  }
  .pop {
    position: absolute;
    left: 0;
    top: calc(100% + 4px);
    z-index: 30;
    min-width: 280px;
    max-width: min(420px, calc(100vw - 32px));
    padding: 8px 10px 10px;
    border: var(--sse-line) solid var(--sse-ink);
    background: var(--sse-ground-raised);
    box-shadow: var(--sse-card-shadow);
    font: 13px var(--sse-font-body);
    color: var(--sse-ink);
    text-transform: none;
    letter-spacing: normal;
    font-weight: normal;
  }
  @media (max-width: 600px) {
    .pop {
      position: fixed;
      left: 0;
      right: 0;
      bottom: 0;
      top: auto;
      max-width: none;
      border-width: var(--sse-line) 0 0;
      padding: 12px 16px 20px;
    }
  }
  .phead {
    display: flex;
    align-items: baseline;
    gap: 8px;
    margin-bottom: 6px;
  }
  .k {
    font-family: var(--sse-font-mono);
    font-size: 11px;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: var(--sse-ink-muted);
  }
  .what {
    flex: 1;
    text-align: right;
    color: var(--sse-ink-muted);
    overflow-wrap: anywhere;
  }
  .x {
    height: auto;
    border: none;
    background: none;
    color: var(--sse-ink-muted);
    font-size: 18px;
    line-height: 1;
    cursor: pointer;
  }
  .row,
  .also {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 4px;
    margin: 4px 0;
  }
  .who {
    min-width: 7em;
  }
  .step {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    height: auto;
    min-height: 28px;
    padding: 2px 8px;
    border: var(--sse-line) solid var(--sse-rule);
    background: var(--sse-ground);
    color: var(--sse-ink);
    font: inherit;
    cursor: pointer;
  }
  .step[aria-checked="true"] {
    border-color: var(--sse-ink);
    box-shadow: inset 0 0 0 1px var(--sse-ink);
  }
  .rung {
    font-size: 12px;
    color: var(--sse-ink-muted);
  }
  .exc {
    display: flex;
    flex-wrap: wrap;
    gap: 2px 10px;
    margin: 6px 0 2px;
    font-size: 12px;
  }
  .x1 {
    color: var(--sse-ink-muted);
  }
</style>

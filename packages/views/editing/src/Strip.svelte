<!--
  The strip (ED1, editing design section 3): a number on a line. It opens from a handle's figure. The mark you hold is
  in the accent; reference worlds and this system's other worlds are dots; the figure top right takes typing in any
  unit of the kind. Dragging writes nothing: the figure follows in pencil and the rung is held so it does not jump.
  Letting go (or Enter after the arrow keys, or a typed figure) is one change. Esc puts it back.
-->
<script lang="ts">
  import { readFigure, say, stretchTo, stripDomain, stripPos, stripStep, stripValue, aboutFor, type Depth, type Handle, type PartMark, type StripDot, type StripDomain } from "../../../ui-contract/src/index.ts";

  let {
    handle,
    reading,
    value,
    dots,
    cls,
    depth,
    editable,
    marks = [],
    walls = [],
    fromTime,
    ontry,
    oncommit,
    onclose,
  }: {
    handle: Handle;
    /** The reading to say figures in (the remembered choice). */
    reading?: string;
    /** The value in SI, or null when the body has none yet. */
    value: number | null;
    dots: StripDot[];
    cls: string;
    depth: Depth;
    editable: boolean;
    /** Labelled places along the line ("past 90 is backwards"). */
    marks?: PartMark[];
    /** Real limits at the line's end ("leaves the system" at a shape of 1): typing may pass one, dragging stops short. */
    walls?: PartMark[];
    /** A typed time turned into this value (a year into an orbit's size), or null where it cannot be. */
    fromTime?: (si: number) => number | null;
    /** Read-only on Explore: "Try changing it" opens an example what-if where the strip moves (ED7). */
    ontry?: () => void;
    /** One change; resolves to null when it was applied, or the reason it was not. */
    oncommit: (si: number, how: { source: "computed" | "typed"; sig?: number }) => Promise<string | null>;
    /** Absent in a box, where the box closes as one. */
    onclose?: () => void;
  } = $props();
  const read = $derived(reading ?? handle.part.reading);

  const about = $derived(aboutFor(handle, cls));
  const sign = $derived(handle.part.signed && value !== null && value < 0 ? -1 : 1);
  // The strip's range follows the value and the dots; a typed value past the end stretches it.
  let stretched = $state<StripDomain | null>(null);
  const domain = $derived(stretched ?? stripDomain(handle, value, dots));
  $effect(() => {
    void handle.id;
    stretched = null;
  });
  // The pencil value while dragging or stepping; null when the strip shows the settled value.
  let pencil = $state<number | null>(null);
  let typed = $state<{ si: number; sig: number } | null>(null);
  let held = $state<string | undefined>(undefined);
  let busy = $state(false);
  let problem = $state<string | null>(null);
  let track: HTMLDivElement | undefined = $state();

  const shown = $derived(pencil ?? (value === null ? null : Math.abs(value)));
  const said = $derived(shown === null ? null : say(shown, handle.part.kind, about, { depth, reading: read, ...(held ? { hold: held } : {}), ...(typed && pencil === null ? { source: "typed" as const, sig: typed.sig } : {}) }));
  const markPos = $derived(shown === null ? null : stripPos(domain, shown));
  // Dots in order along the line; a name is written only where it does not run into the last one written.
  const placed = $derived.by(() => {
    const out = dots.map((d) => ({ ...d, pos: stripPos(domain, d.si), text: say(Math.abs(d.si), handle.part.kind, about, { depth, reading: read }).text, named: false })).sort((a, b) => a.pos - b.pos);
    let last = -1;
    for (const d of out) if (d.pos - last >= 0.1) ((d.named = true), (last = d.pos));
    return out;
  });

  function posAt(e: PointerEvent): number {
    const r = track!.getBoundingClientRect();
    return Math.min(1, Math.max(0, (e.clientX - r.left) / r.width));
  }
  // A wall at or past the line's end: dragging stops just short of it (typing may pass).
  const stopAt = (v: number) => walls.reduce((x, w) => (w.at > domain.lo && x >= w.at ? domain.lo + (w.at - domain.lo) * 0.999 : x), v);
  let dragging = false;
  function down(e: PointerEvent) {
    if (!editable || busy) return;
    dragging = true;
    track!.setPointerCapture(e.pointerId);
    held = shown !== null ? say(shown, handle.part.kind, about, { depth, reading: read }).rung : undefined;
    pencil = stopAt(stripValue(domain, posAt(e)));
    problem = null;
  }
  function move(e: PointerEvent) {
    if (dragging) pencil = stopAt(stripValue(domain, posAt(e)));
  }
  async function up() {
    if (!dragging) return;
    dragging = false;
    await settle();
  }
  async function settle(how: { source: "computed" | "typed"; sig?: number } = { source: "computed" }) {
    const v = pencil;
    held = undefined;
    if (v === null) return;
    busy = true;
    const why = await oncommit(sign * v, how);
    busy = false;
    problem = why;
    pencil = null;
  }
  function key(e: KeyboardEvent) {
    if (e.key === "Escape") {
      if (pencil !== null) {
        pencil = null;
        held = undefined;
      } else if (onclose) onclose();
      else return;
      // The strip's Esc is the strip's: it never also clears the selection behind it.
      e.preventDefault();
      e.stopPropagation();
      return;
    }
    if (!editable) return;
    if (e.key === "ArrowRight" || e.key === "ArrowUp" || e.key === "ArrowLeft" || e.key === "ArrowDown") {
      const from = pencil ?? shown ?? domain.lo;
      if (held === undefined && shown !== null) held = say(shown, handle.part.kind, about, { depth, reading: read }).rung;
      pencil = stripStep(domain, from, e.key === "ArrowRight" || e.key === "ArrowUp" ? 1 : -1, e.shiftKey);
      e.preventDefault();
    } else if (e.key === "Enter" && pencil !== null) {
      void settle();
      e.preventDefault();
    }
  }
  let entry = $state("");
  let editing = $state(false);
  function startTyping() {
    if (!editable) return;
    entry = said?.text.replace(/^about /, "") ?? "";
    editing = true;
  }
  async function typedIn(e: KeyboardEvent) {
    // The entry's own keys never reach the strip's (an Enter here would set the value twice).
    e.stopPropagation();
    if (e.key === "Escape") {
      editing = false;
      e.preventDefault();
      return;
    }
    if (e.key !== "Enter") return;
    e.preventDefault();
    const text = entry.trim();
    // A bare number is in the unit the figure shows (a shape has none).
    const bare = /^[-+−]?\d[\d.,]*(e[-+]?\d+)?$/i.test(text);
    let r: ReturnType<typeof readFigure> = readFigure(bare && said?.unit ? `${text} ${said.unit}` : text, handle.part.kind);
    if (!r.ok && bare && said && !said.unit) {
      const x = Number(text.replace("−", "-").replace(",", "."));
      if (Number.isFinite(x)) r = { ok: true, si: x, unit: "one", sig: text.replace(/^[-+−]?0*\.?0*/, "").replace(/[^\d]/g, "").length || 1, terms: 1 };
    }
    // An orbit's size takes a year too: "300 d" is the distance whose year that is.
    if (!r.ok && fromTime) {
      const t = readFigure(text, "time");
      const si = t.ok ? fromTime(Math.abs(t.si)) : null;
      if (t.ok && si !== null) r = { ...t, si };
    }
    if (!r.ok) {
      problem = r.as ? `${r.why}. Type a ${handle.part.kind}${fromTime ? " or a year" : ""}.` : r.why;
      return;
    }
    editing = false;
    // Typed values are taken exactly as typed, past the strip's end if need be: the strip stretches to hold them.
    stretched = stretchTo(domain, Math.abs(r.si));
    typed = { si: Math.abs(r.si), sig: r.sig };
    pencil = Math.abs(r.si);
    await settle({ source: "typed", sig: r.sig });
  }
  $effect(() => {
    void value;
    if (typed && value !== null && Math.abs(Math.abs(value) - typed.si) > Math.abs(typed.si) * 1e-9) typed = null;
  });
</script>

<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
<div class="strip" data-strip={handle.id} role="group" aria-label={`${handle.names.science}: ${said?.text ?? "not set"}`} onkeydown={key}>
  <div class="head">
    <span class="name">{depth === "curious" ? handle.names.curious : handle.names.science}</span>
    {#if editing}
      <input class="entry" data-strip-entry bind:value={entry} onkeydown={typedIn} onblur={() => (editing = false)} aria-label={`Type a ${handle.part.kind} for ${handle.names.science}`} />
    {:else}
      <button type="button" class="figure" class:pencil={pencil !== null} data-strip-figure disabled={!editable} onclick={startTyping} title={editable ? "Type a value, in any unit" : undefined}>
        {#if said}{said.number}{#if said.unit}<span class="unit">{said.unit.startsWith("°") || said.unit === "%" ? "" : " "}{said.unit}</span>{/if}{:else}none{/if}
      </button>
    {/if}
    {#if onclose}<button type="button" class="close" aria-label="Close the strip" onclick={onclose}>×</button>{/if}
  </div>
  <!-- svelte-ignore a11y_no_static_element_interactions (the mark inside is the slider; the line takes the pointer) -->
  <div
    class="track"
    class:live={editable}
    bind:this={track}
    onpointerdown={down}
    onpointermove={move}
    onpointerup={up}
    onpointercancel={() => {
      dragging = false;
      pencil = null;
      held = undefined;
    }}
  >
    <div class="line"></div>
    {#if domain.zero}<span class="none" style="left: 0">none</span>{/if}
    {#each placed as d (d.label)}
      <span class="dot" class:here={d.from === "here"} style="left: {d.pos * 100}%" title={`${d.label}: ${d.text}`} data-dot={d.label}></span>
      {#if d.named}<span class="dot-label" class:here={d.from === "here"} style="left: {d.pos * 100}%">{d.label}</span>{/if}
    {/each}
    {#each [...marks, ...walls] as m (m.says)}
      {@const wall = walls.includes(m)}
      {@const p = stripPos(domain, m.at)}
      <span class="tick" class:wall style="left: {p * 100}%" data-tick={m.says}></span>
      <span class="tick-label" class:wall style="left: {p * 100}%; transform: translateX({p < 0.1 ? 0 : p > 0.9 ? -100 : -50}%)">{m.says}</span>
    {/each}
    {#if markPos !== null}
      <span
        class="mark"
        class:pencil={pencil !== null}
        style="left: {markPos * 100}%"
        role="slider"
        tabindex={editable ? 0 : -1}
        aria-valuemin={domain.lo}
        aria-valuemax={domain.hi}
        aria-valuenow={shown ?? 0}
        aria-valuetext={said?.text}
        aria-readonly={!editable}
        data-strip-mark
      ></span>
    {/if}
  </div>
  <p class="verdict" aria-live="polite">
    {#if problem}<span class="problem">{problem}</span>
    {:else if !editable}Where this world sits among the reference worlds.{#if ontry}
        <button type="button" class="try" data-try onclick={ontry}>Try changing it</button>{:else} Change it on the Build desk.{/if}
    {:else if pencil !== null}Let go, or press Enter, to set it. Esc puts it back.
    {:else}Drag along the line, use the arrow keys (Shift for bigger steps), or press the figure to type a value in any unit.{/if}
  </p>
</div>

<style>
  .strip {
    min-width: 0;
    margin: 0;
    padding: 10px 18px 8px;
    background: var(--sse-ground);
    border-bottom: var(--sse-line) solid var(--sse-rule);
  }
  .head {
    display: flex;
    align-items: baseline;
    gap: 8px;
  }
  .name {
    font-size: 12px;
    letter-spacing: 0.04em;
    color: var(--sse-ink-muted);
  }
  .figure,
  .entry {
    margin-left: auto;
    font-family: var(--sse-font-mono);
    font-size: 15px;
    color: var(--sse-accent-ink);
  }
  .figure {
    border: 0;
    background: none;
    padding: 2px 4px;
    cursor: text;
    border-bottom: var(--sse-line) dotted var(--sse-accent-ink);
  }
  .figure:disabled {
    cursor: default;
    border-bottom-color: transparent;
    color: var(--sse-ink);
  }
  .figure.pencil {
    font-style: italic;
    color: var(--sse-ink-muted);
  }
  .entry {
    width: 12em;
    padding: 2px 4px;
    border: var(--sse-line) solid var(--sse-accent);
    background: var(--sse-ground-raised);
  }
  .unit {
    font-size: 12px;
  }
  .close {
    border: 0;
    background: none;
    color: var(--sse-ink-muted);
    font-size: 16px;
    min-width: 28px;
    min-height: 28px;
    cursor: pointer;
  }
  .track {
    position: relative;
    overflow: visible;
    height: 44px;
    margin: 4px 6px 0;
    touch-action: none;
  }
  .track.live {
    cursor: ew-resize;
  }
  .line {
    position: absolute;
    left: 0;
    right: 0;
    top: 14px;
    border-top: var(--sse-line-heavy) solid var(--sse-ink-faint);
  }
  .none {
    position: absolute;
    top: 22px;
    font-size: 10px;
    color: var(--sse-ink-faint);
  }
  .dot {
    position: absolute;
    top: 10px;
    width: 9px;
    height: 9px;
    margin-left: -4.5px;
    border-radius: 50%;
    background: var(--sse-ground-raised);
    border: var(--sse-line) solid var(--sse-ink-muted);
  }
  .dot.here {
    background: var(--sse-ink-muted);
  }
  .dot-label {
    position: absolute;
    top: 24px;
    transform: translateX(-50%);
    font-size: 10px;
    color: var(--sse-ink-faint);
    white-space: nowrap;
    pointer-events: none;
  }
  .dot-label.here {
    color: var(--sse-ink-muted);
  }
  .tick {
    position: absolute;
    top: 6px;
    height: 16px;
    border-left: var(--sse-line) solid var(--sse-ink-muted);
  }
  .tick.wall {
    border-left: var(--sse-line-heavy) solid var(--sse-alert);
  }
  .tick-label {
    position: absolute;
    top: 30px;
    font-size: 10px;
    color: var(--sse-ink-muted);
    white-space: nowrap;
    pointer-events: none;
  }
  .tick-label.wall {
    color: var(--sse-alert);
  }
  .mark {
    position: absolute;
    top: 4px;
    width: 14px;
    height: 22px;
    margin-left: -7px;
    background: var(--sse-accent);
    outline-offset: 2px;
    transition: left var(--sse-motion-quick) var(--sse-ease);
  }
  .mark.pencil {
    opacity: 0.7;
    transition: none;
  }
  .mark:focus-visible {
    outline: 2px solid var(--sse-focus);
  }
  .verdict {
    margin: 4px 0 0;
    font-size: 12px;
    line-height: 1.4;
    color: var(--sse-ink-muted);
  }
  .try {
    margin-left: 4px;
    padding: 2px 8px;
    border: var(--sse-line) solid var(--sse-accent);
    background: var(--sse-ground-raised);
    color: var(--sse-accent-ink);
    font: inherit;
    cursor: pointer;
  }
  .problem {
    color: var(--sse-accent-ink);
  }
  @media (max-width: 600px) {
    /* A thumb-sized mark on a phone: a little bigger to see, and a 44 px square to touch (the touch target
       guidelines' least) that leaves the reference names readable. */
    .track {
      height: 52px;
    }
    .mark {
      top: 1px;
      width: 18px;
      height: 26px;
      margin-left: -9px;
    }
    .mark::before {
      content: "";
      position: absolute;
      inset: -9px -13px;
    }
  }
  @media (prefers-reduced-motion: reduce) {
    .mark {
      transition: none;
    }
  }
</style>

<!--
  The strip (ED1, editing design section 3): a number on a line. It opens from a handle's figure. The mark you hold is
  in the accent; reference worlds and this system's other worlds are dots; the figure top right takes typing in any
  unit of the kind. Dragging writes nothing: the figure follows in pencil and the rung is held so it does not jump.
  Letting go (or Enter after the arrow keys, or a typed figure) is one change. Esc puts it back.
-->
<script lang="ts">
  import { readFigure, say, stretchTo, stripDomain, stripPos, stripStep, stripValue, aboutFor, type Depth, type Handle, type StripDot, type StripDomain } from "../../../ui-contract/src/index.ts";

  let {
    handle,
    reading,
    value,
    dots,
    cls,
    depth,
    editable,
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
    /** One change; resolves to null when it was applied, or the reason it was not. */
    oncommit: (si: number, how: { source: "computed" | "typed"; sig?: number }) => Promise<string | null>;
    onclose: () => void;
  } = $props();

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
  const said = $derived(shown === null ? null : say(shown, handle.part.kind, about, { depth, reading, ...(held ? { hold: held } : {}), ...(typed && pencil === null ? { source: "typed" as const, sig: typed.sig } : {}) }));
  const markPos = $derived(shown === null ? null : stripPos(domain, shown));
  // Dots in order along the line; a name is written only where it does not run into the last one written.
  const placed = $derived.by(() => {
    const out = dots.map((d) => ({ ...d, pos: stripPos(domain, d.si), text: say(Math.abs(d.si), handle.part.kind, about, { depth, reading }).text, named: false })).sort((a, b) => a.pos - b.pos);
    let last = -1;
    for (const d of out) if (d.pos - last >= 0.1) ((d.named = true), (last = d.pos));
    return out;
  });

  function posAt(e: PointerEvent): number {
    const r = track!.getBoundingClientRect();
    return Math.min(1, Math.max(0, (e.clientX - r.left) / r.width));
  }
  let dragging = false;
  function down(e: PointerEvent) {
    if (!editable || busy) return;
    dragging = true;
    track!.setPointerCapture(e.pointerId);
    held = shown !== null ? say(shown, handle.part.kind, about, { depth, reading }).rung : undefined;
    pencil = stripValue(domain, posAt(e));
    problem = null;
  }
  function move(e: PointerEvent) {
    if (dragging) pencil = stripValue(domain, posAt(e));
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
      } else onclose();
      e.preventDefault();
      return;
    }
    if (!editable) return;
    if (e.key === "ArrowRight" || e.key === "ArrowUp" || e.key === "ArrowLeft" || e.key === "ArrowDown") {
      const from = pencil ?? shown ?? domain.lo;
      if (held === undefined && shown !== null) held = say(shown, handle.part.kind, about, { depth, reading }).rung;
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
    const r = readFigure(entry, handle.part.kind);
    if (!r.ok) {
      problem = r.as ? `${r.why}. Type a ${handle.part.kind}${handle.id === "orbit" ? " (turning a year into a distance comes with the Orbit box)" : ""}.` : r.why;
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
    <button type="button" class="close" aria-label="Close the strip" onclick={onclose}>×</button>
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
    {:else if !editable}Where this world sits among the reference worlds. Change it on the Build desk.
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
  .problem {
    color: var(--sse-accent-ink);
  }
  @media (max-width: 420px) {
    .track {
      height: 52px;
    }
    .mark {
      width: 22px;
      height: 28px;
      margin-left: -11px;
    }
  }
  @media (prefers-reduced-motion: reduce) {
    .mark {
      transition: none;
    }
  }
</style>

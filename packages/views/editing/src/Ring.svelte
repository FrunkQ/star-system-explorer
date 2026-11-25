<!--
  The ring (editing design section 3B): an angle or a place round a circle. Drag the dot round; it catches at the named
  places (Alt lets it run free); the arrow keys step a degree (Shift ten); the figure takes typing in any angle unit,
  a bare number in degrees. Dragging writes nothing until you let go: one change, one Undo. Esc puts it back.
-->
<script lang="ts">
  import { readFigure, say, wrap, type Depth, type PartMark } from "../../../ui-contract/src/index.ts";

  let {
    id,
    name,
    value,
    catches = [],
    depth,
    editable,
    note,
    oncommit,
  }: {
    id: string;
    name: string;
    /** Radians, 0 at the right, counted anticlockwise; null when there is none yet. */
    value: number | null;
    catches?: PartMark[];
    depth: Depth;
    editable: boolean;
    /** A line under the figure ("from its closest point"). */
    note?: string;
    oncommit: (rad: number, how: { source: "computed" | "typed"; sig?: number }) => Promise<string | null>;
  } = $props();

  const DEG = Math.PI / 180;
  const CATCH = 4 * DEG;
  let pencil = $state<number | null>(null);
  let busy = $state(false);
  let problem = $state<string | null>(null);
  let svg: SVGSVGElement | undefined = $state();
  const shown = $derived(pencil ?? value);
  const said = $derived(shown === null ? null : say(shown, "angle", { class: "planet" }, { depth }));
  const caught = $derived(shown === null ? null : (catches.find((c) => Math.abs(wrap(shown - c.at + Math.PI) - Math.PI) < 1e-9) ?? null));
  const R = 34;
  const C = 44;
  const at = (a: number) => [C + R * Math.cos(a), C - R * Math.sin(a)] as const;

  function angleAt(e: PointerEvent): number {
    const r = svg!.getBoundingClientRect();
    const x = e.clientX - (r.left + r.width / 2);
    const y = r.top + r.height / 2 - e.clientY;
    let a = wrap(Math.atan2(y, x));
    if (!e.altKey) for (const c of catches) if (Math.abs(wrap(a - c.at + Math.PI) - Math.PI) < CATCH) a = wrap(c.at);
    return a;
  }
  let dragging = false;
  function down(e: PointerEvent) {
    if (!editable || busy) return;
    dragging = true;
    svg!.setPointerCapture(e.pointerId);
    pencil = angleAt(e);
    problem = null;
  }
  function move(e: PointerEvent) {
    if (dragging) pencil = angleAt(e);
  }
  async function settle(how: { source: "computed" | "typed"; sig?: number } = { source: "computed" }) {
    const v = pencil;
    if (v === null) return;
    busy = true;
    problem = await oncommit(v, how);
    busy = false;
    pencil = null;
  }
  async function up() {
    if (!dragging) return;
    dragging = false;
    await settle();
  }
  function key(e: KeyboardEvent) {
    if (e.key === "Escape") {
      if (pencil !== null) {
        pencil = null;
        e.preventDefault();
        e.stopPropagation();
      }
      return;
    }
    if (!editable) return;
    const step = (e.shiftKey ? 10 : 1) * DEG;
    if (e.key === "ArrowRight" || e.key === "ArrowUp" || e.key === "ArrowLeft" || e.key === "ArrowDown") {
      pencil = wrap((pencil ?? value ?? 0) + (e.key === "ArrowRight" || e.key === "ArrowUp" ? step : -step));
      e.preventDefault();
      e.stopPropagation();
    } else if (e.key === "Enter" && pencil !== null) {
      e.preventDefault();
      void settle();
    }
  }
  let entry = $state("");
  let editing = $state(false);
  async function typedIn(e: KeyboardEvent) {
    e.stopPropagation();
    if (e.key === "Escape") {
      editing = false;
      e.preventDefault();
      return;
    }
    if (e.key !== "Enter") return;
    e.preventDefault();
    const text = entry.trim();
    const r = readFigure(/^[-+−]?\d[\d.,]*$/.test(text) ? `${text}°` : text, "angle");
    if (!r.ok) {
      problem = `${r.why}. Type an angle, like 120° or 23d 26m.`;
      return;
    }
    editing = false;
    pencil = wrap(r.si);
    await settle({ source: "typed", sig: r.sig });
  }
</script>

<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
<div class="ring" data-ring={id} role="group" aria-label={`${name}: ${said?.text ?? "not set"}`} onkeydown={key}>
  <!-- svelte-ignore a11y_no_static_element_interactions (the dot inside is the slider; the ring takes the pointer) -->
  <svg
    viewBox="0 0 88 88"
    width="88"
    height="88"
    class:live={editable}
    bind:this={svg}
    onpointerdown={down}
    onpointermove={move}
    onpointerup={up}
    onpointercancel={() => ((dragging = false), (pencil = null))}
  >
    <circle cx={C} cy={C} r={R} class="track" />
    {#each catches as c (c.says)}
      {@const [x, y] = at(c.at)}
      {@const [x2, y2] = [C + (R + 6) * Math.cos(c.at), C - (R + 6) * Math.sin(c.at)]}
      <line x1={x} y1={y} x2={x2} y2={y2} class="tick"><title>{c.says}</title></line>
    {/each}
    {#if shown !== null}
      {@const [x, y] = at(shown)}
      <line x1={C} y1={C} x2={x} y2={y} class="spoke" />
      <circle
        cx={x}
        cy={y}
        r="7"
        class="dot"
        class:pencil={pencil !== null}
        role="slider"
        tabindex={editable ? 0 : -1}
        aria-label={name}
        aria-valuemin={0}
        aria-valuemax={360}
        aria-valuenow={Math.round(shown / DEG)}
        aria-valuetext={said?.text}
        aria-readonly={!editable}
        data-ring-dot
      />
    {/if}
  </svg>
  <div class="side">
    <span class="name">{name}</span>
    {#if editing}
      <input class="entry" data-ring-entry bind:value={entry} onkeydown={typedIn} onblur={() => (editing = false)} aria-label={`Type an angle for ${name}`} />
    {:else}
      <button type="button" class="figure" class:pencil={pencil !== null} data-ring-figure disabled={!editable} onclick={() => ((entry = said?.number ?? ""), (editing = true))} title={editable ? "Type an angle" : undefined}>
        {said?.text ?? "not set"}
      </button>
    {/if}
    {#if caught}<span class="caught" data-ring-caught>{caught.says}</span>{:else if note}<span class="note">{note}</span>{/if}
    {#if problem}<span class="problem">{problem}</span>{/if}
  </div>
</div>

<style>
  .ring {
    display: flex;
    align-items: center;
    gap: 10px;
    min-width: 0;
  }
  svg {
    flex: none;
    touch-action: none;
    overflow: visible;
  }
  svg.live {
    cursor: grab;
  }
  .track {
    fill: none;
    stroke: var(--sse-ink-faint);
    stroke-width: 2;
  }
  .tick {
    stroke: var(--sse-ink-muted);
    stroke-width: 1.5;
  }
  .spoke {
    stroke: var(--sse-ink-faint);
    stroke-width: 1;
  }
  .dot {
    fill: var(--sse-accent);
    outline: none;
  }
  .dot.pencil {
    opacity: 0.7;
  }
  .dot:focus-visible {
    stroke: var(--sse-focus);
    stroke-width: 2;
  }
  .side {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 2px;
    min-width: 0;
  }
  .name,
  .note,
  .caught {
    font-size: 11px;
    letter-spacing: 0.04em;
    color: var(--sse-ink-muted);
  }
  .caught {
    color: var(--sse-accent-ink);
  }
  .figure,
  .entry {
    font-family: var(--sse-font-mono);
    font-size: 14px;
    color: var(--sse-accent-ink);
  }
  .figure {
    border: 0;
    background: none;
    padding: 2px 0;
    cursor: text;
    border-bottom: var(--sse-line) dotted var(--sse-accent-ink);
  }
  .figure:disabled {
    cursor: default;
    color: var(--sse-ink);
    border-bottom-color: transparent;
  }
  .figure.pencil {
    font-style: italic;
    color: var(--sse-ink-muted);
  }
  .entry {
    width: 8em;
    padding: 2px 4px;
    border: var(--sse-line) solid var(--sse-accent);
    background: var(--sse-ground-raised);
  }
  .problem {
    font-size: 12px;
    color: var(--sse-accent-ink);
  }
</style>

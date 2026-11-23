<!--
  The Mag layer on the Body screen (magnetic fields design sections 8 and 9, MF14b part 1): the Chart cut, a plane
  through the magnetic axis and the wind's direction, with the field's lines inked by the theme (field-out where the
  field leaves the body, field-in where it enters), the wind's bubble, both axes and the tilt between them, and the
  words. Lines are a picture of the kernel's facts (read_field), traced by the renderer's shared tracer; nothing here
  computes physics. Every part has a why. A world with no field facts says so. It shows when the view controls'
  Band is Mag, in place of the globe.
-->
<script lang="ts">
  import { chartCut, cutPaths, FIELD_LINES, FIELD_REFS, fieldFromRead, narrateField, type FieldReadJson } from "../../../renderer/src/index.ts";
  import { showReference, type Depth, type ViewSession } from "../../../ui-contract/src/index.ts";

  let {
    session,
    system,
    name,
    tier = "gpu-basic",
    depth = "science",
    size = 300,
    on = false,
  }: { session: ViewSession; system: string; name: string; tier?: string; depth?: Depth; size?: number; on?: boolean } = $props();

  // The kernel's field facts. Until MF2 publishes fields as facts of every world, read_field's Solar System fixture
  // is the source: it covers the Solar System's own bodies, matched by name inside the campaign's Solar System.
  const SOL = /\/sys:sol$/;
  let busy = $state(false);
  let failed = $state<string | null>(null);
  let read = $state.raw<FieldReadJson[] | null>(null);
  async function load() {
    if (read || busy) return;
    busy = true;
    failed = null;
    const r = await session.readField({ fixture: "solar-system" });
    busy = false;
    if (!r.ok) return void (failed = r.message);
    read = r.bodies as unknown as FieldReadJson[];
  }
  $effect(() => {
    if (on && SOL.test(system)) void load();
  });

  const body = $derived(SOL.test(system) && read ? (read.find((b) => b.name === name) ?? null) : null);
  const field = $derived(body ? fieldFromRead(body) : null);
  const raw = $derived(body?.field as (FieldReadJson["field"] & { model?: string }) | undefined);
  const seeds = $derived(FIELD_LINES[tier as keyof typeof FIELD_LINES] ?? FIELD_LINES["gpu-basic"]);
  // Traced once per change of field, wind or tier, never per frame.
  const cut = $derived(field && seeds ? chartCut(field.model, field.bubble, { perHemisphere: seeds.perHemisphere }) : null);
  // A closed line leaves the body on one side and comes back in on the other: the field along it points away until
  // its farthest point and toward the body after, so each closed line is drawn in two halves, one in each ink.
  const inked = $derived.by(() => {
    if (!cut) return null;
    const lines = cut.lines.flatMap((l) => {
      if (l.kind !== "closed" || l.points.length < 3) return [l];
      let k = 0;
      let far = -1;
      l.points.forEach((p, i) => {
        const r = p.x * p.x + p.y * p.y;
        if (r > far) (far = r), (k = i);
      });
      return [
        { ...l, points: l.points.slice(0, k + 1) },
        { ...l, points: l.points.slice(k), polarity: (-l.polarity) as 1 | -1 },
      ];
    });
    return { ...cut, lines };
  });
  const extent = $derived(field?.bubble ? Math.min(90, Math.max(4, field.bubble.standoff_radii * 1.8)) : 6);
  const paths = $derived(inked ? cutPaths(inked, size, extent) : null);
  const words = $derived(field ? narrateField(field.model, field.bubble, cut?.trace) : "");

  type Part = "out" | "in" | "open" | "bubble" | "axes";
  let why = $state<Part | null>(null);
  const WHY: Record<Part, string> = {
    out: "Orange: where the field points away from the body. A line leaves from one magnetic pole's side and, if it closes, comes back in on the other.",
    in: "Blue: where the field points into the body. The two inks swap where a line turns round, at its farthest point.",
    open: "Fainter lines are open: they leave through the bubble into the wind's own field instead of landing back on the body. They come from the polar caps, where aurorae and the wind's particles get in.",
    bubble: "The bubble's edge is where the field's pressure meets the wind's (the magnetopause). Its nose sits at the stand-off on the side facing the star; its flanks flare back (Shue's surface).",
    axes: "The solid axis is the spin axis, the dashed one the magnetic axis; the angle between them is the field's tilt.",
  };
</script>

<section class="mag" aria-label="Mag layer" data-mag={on ? (field ? "lines" : busy ? "loading" : "none") : "off"}>
  {#if on}
    {#if busy}
      <p class="note" aria-busy="true">Reading the field…</p>
    {:else if failed}
      <p class="note" data-mag-error>Could not read the field: {failed}</p>
    {:else if !field}
      <p class="note" data-mag-none>No field facts for this world yet.{depth !== "curious" ? " Fields come from the facts of the Solar System's own bodies for now; other worlds get theirs once the kernel publishes a field for every world." : ""}</p>
    {:else}
      <div class="panel">
        {#if paths}
          <!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions (pointer shortcut; the legend below is the keyboard route to every why) -->
          <svg viewBox="0 0 {size} {size}" width={size} height={size} role="img" aria-label={`Mag layer of ${name}: ${words}`} data-mag-cut>
            {#if paths.bubble}<path class="bubble" d={paths.bubble} onclick={() => (why = "bubble")} />{/if}
            {#each paths.lines as l, i (i)}
              <path class="line" class:open={l.kind !== "closed"} d={l.d} stroke-width={Math.max(0.6, l.width * 1.6)} style:stroke={l.polarity > 0 ? "var(--sse-field-out)" : "var(--sse-field-in)"} data-kind={l.kind} onclick={() => (why = l.kind !== "closed" ? "open" : l.polarity > 0 ? "out" : "in")} />
            {/each}
            <circle class="world" cx={paths.body.cx} cy={paths.body.cy} r={Math.max(2, paths.body.r)} />
            <line class="spin" x1={paths.spinAxis[0]} y1={paths.spinAxis[1]} x2={paths.spinAxis[2]} y2={paths.spinAxis[3]} onclick={() => (why = "axes")} />
            <line class="magnetic" x1={paths.magneticAxis[0]} y1={paths.magneticAxis[1]} x2={paths.magneticAxis[2]} y2={paths.magneticAxis[3]} onclick={() => (why = "axes")} />
            <text class="tilt" x={paths.tilt.x} y={paths.tilt.y}>{paths.tilt.text}</text>
            <text class="sunward" x="6" y={size / 2 - 6}>◂ to the star</text>
          </svg>
        {:else}
          <p class="note">Lines are not drawn on this device; the words say what they would show.</p>
        {/if}
        <div class="side">
          <p class="words" data-mag-words>{words}</p>
          <ul class="legend" aria-label="What the lines mean">
            <li><button type="button" aria-pressed={why === "out"} onclick={() => (why = why === "out" ? null : "out")}><i class="sw" style:background="var(--sse-field-out)"></i>Field out · why?</button></li>
            <li><button type="button" aria-pressed={why === "in"} onclick={() => (why = why === "in" ? null : "in")}><i class="sw" style:background="var(--sse-field-in)"></i>Field in · why?</button></li>
            <li><button type="button" aria-pressed={why === "open"} onclick={() => (why = why === "open" ? null : "open")}><i class="sw faint"></i>Open lines · why?</button></li>
            {#if field.bubble}<li><button type="button" aria-pressed={why === "bubble"} onclick={() => (why = why === "bubble" ? null : "bubble")}><i class="sw dashed"></i>The wind's bubble · why?</button></li>{/if}
            <li><button type="button" aria-pressed={why === "axes"} onclick={() => (why = why === "axes" ? null : "axes")}><i class="sw axis"></i>Axes and tilt · why?</button></li>
          </ul>
          {#if why}<p class="why" data-mag-why={why} aria-live="polite">{WHY[why]}</p>{/if}
          {#if depth !== "curious"}
            <p class="source" data-mag-source>
              From {raw?.model ? `the ${raw.model} model` : "the kernel's field facts"}{raw?.grade ? ` (${raw.grade})` : ""}, by read_field; the wind is the kernel's at this world's distance.
            </p>
            {#if field.notes.length}<ul class="notes" data-mag-notes aria-label="Stand-ins">{#each field.notes as n (n)}<li>{n}</li>{/each}</ul>{/if}
          {/if}
          {#if depth === "engine" && cut}
            <p class="engine" data-depth="engine">
              Traced {cut.trace.counts.closed + cut.trace.counts.open} lines ({cut.trace.counts.closed} closed, {cut.trace.counts.open} open) with {seeds?.perHemisphere} seeds a hemisphere on this tier; g10 {raw?.g10_t?.toExponential(3)} T, g11 {raw?.g11_t?.toExponential(3)} T, h11 {raw?.h11_t?.toExponential(3)} T; drawn to {Math.round(extent)} radii.
            </p>
            <ul class="refs">{#each FIELD_REFS as ref (ref)}{@const r = showReference(ref)}<li>{#if r.href}<a href={r.href} target="_blank" rel="noopener">{r.text}</a>{:else}{r.text}{/if}</li>{/each}</ul>
          {/if}
        </div>
      </div>
    {/if}
  {/if}
</section>

<style>
  .mag {
    margin: 8px 0;
    font-size: 13px;
    max-width: 100%;
  }
  .note {
    margin: 8px 0 0;
    color: var(--sse-ink-muted);
  }
  .panel {
    display: flex;
    flex-wrap: wrap;
    gap: 12px 20px;
    margin-top: 10px;
    align-items: flex-start;
  }
  svg {
    max-width: 100%;
    height: auto;
    background: var(--sse-ground);
    border: 1px solid var(--sse-rule);
  }
  .line {
    fill: none;
    stroke-linecap: round;
    cursor: help;
  }
  .line.open {
    opacity: 0.45;
  }
  .bubble {
    fill: none;
    stroke: var(--sse-ink-muted);
    stroke-width: 1;
    stroke-dasharray: 4 3;
    cursor: help;
  }
  .world {
    fill: var(--sse-ink);
  }
  .spin,
  .magnetic {
    stroke: var(--sse-ink);
    stroke-width: 1;
    cursor: help;
  }
  .magnetic {
    stroke-dasharray: 3 2;
  }
  .tilt,
  .sunward {
    font-family: var(--sse-font-mono);
    font-size: 10px;
    fill: var(--sse-ink-muted);
  }
  .side {
    flex: 1 1 240px;
    min-width: 0;
  }
  .words {
    margin: 0 0 8px;
    color: var(--sse-ink);
  }
  .legend {
    list-style: none;
    margin: 0;
    padding: 0;
    display: grid;
    gap: 2px;
  }
  .legend button {
    font: inherit;
    border: 0;
    background: none;
    padding: 2px 0;
    color: var(--sse-ink);
    cursor: pointer;
    display: inline-flex;
    align-items: center;
    gap: 8px;
    text-align: left;
  }
  .legend button[aria-pressed="true"] {
    text-decoration: underline;
    text-underline-offset: 3px;
  }
  .sw {
    display: inline-block;
    width: 18px;
    height: 3px;
  }
  .sw.faint {
    background: var(--sse-ink-muted);
    opacity: 0.45;
  }
  .sw.dashed {
    border-top: 1px dashed var(--sse-ink-muted);
    height: 0;
  }
  .sw.axis {
    border-top: 1px solid var(--sse-ink);
    height: 0;
  }
  .why {
    margin: 8px 0 0;
    padding: 8px 10px;
    border-left: var(--sse-line-heavy) solid var(--sse-accent);
    background: var(--sse-ground-raised);
  }
  .source,
  .engine {
    margin: 8px 0 0;
    font-size: 12px;
    color: var(--sse-ink-muted);
  }
  .notes,
  .refs {
    margin: 4px 0 0;
    padding-left: 18px;
    font-size: 12px;
    color: var(--sse-ink-muted);
  }
  .refs a {
    color: var(--sse-ink-muted);
  }
</style>

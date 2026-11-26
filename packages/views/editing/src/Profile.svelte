<!--
  The Profile (RB3, ring and belt editing design sections 4.2, 5.1 and 7): an amount along a line, the editing
  grammar's fifteenth part. A ring or belt seen edge to edge: distance from the parent across, how much is there upward
  (density, on a log scale so a faint ring and a dense one both read), each zone a span with its name, its cause in
  words and the marks its sculptors make, drawn with a thin line down to the sculptor on the same axis.

  Shaping keeps the sum (design question 2): a join drags sideways to move the edge between two zones, a zone drags up
  or down to make it denser or thinner, and the rest give way in proportion. Every figure that drags can be typed, in
  any unit of its kind. A drag writes nothing: the shape is held in pencil; letting go (Enter after the arrow keys, or
  a typed figure) keeps it as one change through `oncommit`, with the zones in the ask's authored shape. Esc puts it
  back. The ring's own edges are Where's handle, not this part's.

  Desks (design section 7): on Explore it is shown and a change is a try (the host opens the what-if); on Build a change
  is kept, one Undo; on Run it is not shown. The host decides by passing `editable`; GM only, like every editing part.
-->
<script lang="ts">
  import { annulus, authoredZones, causeTag, edgesHeld, joins, lightThrough, markWords, moveJoin, profileSpans, readFigure, say, sayNumber, sculptorOf, setDensity, setShare, srgb, type AuthoredZone, type Depth, type ProfileSpan, type RingStructure, type Sculptor } from "../../../ui-contract/src/index.ts";

  let {
    ring,
    sculptors = [],
    parent = "the world",
    cls = "ring",
    depth,
    editable,
    oncommit,
    onreset,
  }: {
    /** The kernel's `ring_structure_is` answer (design section 9). */
    ring: RingStructure;
    /** The bodies that shape it (the ask's sculptors): moons round a world, planets round a star. */
    sculptors?: readonly Sculptor[];
    /** The parent's name, for the axis: "distance from Saturn". */
    parent?: string;
    /** The units engine's scale class (a ring or belt). */
    cls?: string;
    depth: Depth;
    editable: boolean;
    /** Keep a shape as one change: the zones in the ask's authored shape, and what changed in words. Resolves to null when kept, or the reason it was not. */
    oncommit?: (zones: AuthoredZone[], display: string) => Promise<string | null> | string | null;
    /** Let physics decide again: the host drops the authored zones. Resolves to null when done, or the reason it was not. */
    onreset?: () => Promise<string | null> | string | null;
  } = $props();

  // ------------------------------------------------------------------------------------------- the shape on show
  const base = $derived(profileSpans(ring));
  /** The last kept shape, ink until the kernel's answer changes. */
  let kept = $state<ProfileSpan[] | null>(null);
  /** A shape being dragged or stepped, pencil. */
  let held = $state<ProfileSpan[] | null>(null);
  $effect(() => {
    void ring;
    kept = null;
    held = null;
  });
  const spans = $derived(held ?? kept ?? base);
  const joinAt = $derived(joins(spans));

  let selected = $state<number | null>(null);
  let why = $state(false);
  let said = $state<string | null>(null);

  // ------------------------------------------------------------------------------------------------ the drawing
  let W = $state(640);
  const TOP = 18;
  const PLOT = 112;
  const BASE = TOP + PLOT;
  const SEEN = BASE + 6;
  const AXIS = SEEN + 14;
  const MARKS = AXIS + 26;
  const pad = 10;

  // The axis runs over the ring, and a little past its outer edge when sculptors sit just beyond it (shepherds, the
  // moons whose resonances fall inside); farther ones are named at the end of the moons' line.
  const lo = $derived(ring.inner_m);
  const hi = $derived.by(() => {
    const w = ring.outer_m - ring.inner_m;
    const near = sculptors.filter((s) => s.a_m > ring.outer_m && s.a_m <= ring.outer_m + 0.25 * w).map((s) => s.a_m);
    return near.length ? Math.max(...near) + 0.02 * w : ring.outer_m;
  });
  // Sculptors past the axis are named at the end of the moons' line, and the axis stops short of their names.
  const farText = $derived.by(() => {
    const f = sculptors.filter((s) => s.a_m > hi).sort((a, b) => a.a_m - b.a_m);
    return f.length ? `${f.map((s) => s.name).join(", ")} →` : "";
  });
  const right = $derived(pad + (farText ? farText.length * 6 + 10 : 0));
  const x = (m: number) => pad + ((m - lo) / (hi - lo)) * (W - pad - right);
  const xm = (px: number) => lo + ((px - pad) / (W - pad - right)) * (hi - lo);

  // Density on a log scale, fixed from the kernel's answer so the pointer maps steadily during a drag.
  const dens = $derived.by(() => {
    const s = base.map((z) => z.sigma_kg_m2).filter((v) => v > 0);
    const top = s.length ? Math.max(...s) : 1;
    const bot = s.length ? Math.max(Math.min(...s), top * 1e-5) : 1e-3;
    return { lo: Math.log10(bot) - 1, hi: Math.log10(top) + 0.6 };
  });
  const y = (sigma: number) => (sigma > 0 ? BASE - Math.min(1, Math.max(0, (Math.log10(sigma) - dens.lo) / (dens.hi - dens.lo))) * PLOT : BASE);
  const ys = (py: number) => 10 ** (dens.lo + ((BASE - py) / PLOT) * (dens.hi - dens.lo));

  const rgb = srgb;

  // Marks and their labels, staggered into rows so close ones do not overprint.
  const inside = (m: number) => m >= lo && m <= hi;
  const moonX = (ref: string) => {
    const s = sculptorOf(ref, sculptors);
    return s && inside(s.a_m) ? x(s.a_m) : W - right;
  };
  const marks = $derived.by(() => {
    const out: { at: number; px: number; row: number; label: string; words: string; to: number }[] = [];
    const ends = [-1e9, -1e9, -1e9];
    for (const m of [...ring.marks].filter((q) => inside(q.r_m)).sort((a, b) => a.r_m - b.r_m)) {
      const name = sculptorOf(m.sculptor, sculptors)?.name ?? m.sculptor;
      const px = x(m.r_m);
      const w = name.length * 6.5 + 8;
      const row = ends.findIndex((e) => px - w / 2 > e);
      if (row >= 0) ends[row] = px + w / 2;
      out.push({ at: m.r_m, px, row, label: name, words: markWords(m, sculptors), to: moonX(m.sculptor) });
    }
    return out;
  });
  // The sculptors on the axis, their names staggered into two rows so close ones (Atlas, Prometheus) both read.
  const near = $derived.by(() => {
    const ends = [-1e9, -1e9];
    return sculptors
      .filter((s) => inside(s.a_m))
      .sort((a, b) => a.a_m - b.a_m)
      .map((s) => {
        const px = x(s.a_m);
        const w = s.name.length * 6.5 + 6;
        // No room in either row: the dot keeps its name as a tooltip rather than overprinting another.
        const row = ends.findIndex((e) => px - w / 2 > e);
        if (row >= 0) ends[row] = px + w / 2;
        return { s, px, row };
      });
  });
  const H = $derived(MARKS + 3 * 12 + 42);

  // ------------------------------------------------------------------------------------------------ the words
  const dist = (m: number) => say(m, "length", { class: cls, what: "orbit" }, { depth });
  const share = (v: number) => say(v, "share", { class: cls }, { depth });
  const tauWords = (t: number) => (depth === "curious" ? `lets through about ${say(lightThrough(t), "share", { class: cls }, { depth, about: false }).text} of light` : `optical depth about ${sayNumber(t, 2)}`);
  const verdict = (i: number) => {
    const s = spans[i]!;
    return `${s.name} · ${share(s.share).text} of the ring · ${tauWords(s.tau)} · ${s.yours ? "shaped by you" : causeTag(s.cause, sculptors)}`;
  };
  const wordsOf = (s: readonly ProfileSpan[], i: number) => `${s[i]!.name} ${share(s[i]!.share).text}, ${dist(s[i]!.inner_m).text} to ${dist(s[i]!.outer_m).text}`;

  // ------------------------------------------------------------------------------------------------ moving it
  let svg = $state<SVGSVGElement | null>(null);
  let drag: { what: "join" | "zone"; i: number; from: ProfileSpan[]; x0: number; y0: number; moved: boolean; id: number } | null = null;

  const local = (e: PointerEvent): [number, number] => {
    const r = svg!.getBoundingClientRect();
    return [((e.clientX - r.left) / r.width) * W, ((e.clientY - r.top) / r.height) * H];
  };
  function down(e: PointerEvent, what: "join" | "zone", i: number) {
    if (what === "zone") selected = i;
    if (!editable || !svg) return;
    const [px, py] = local(e);
    drag = { what, i, from: spans.map((s) => ({ ...s })), x0: px, y0: py, moved: false, id: e.pointerId };
    svg.setPointerCapture(e.pointerId);
    e.preventDefault();
  }
  function move(e: PointerEvent) {
    if (!drag || e.pointerId !== drag.id) return;
    const [px, py] = local(e);
    if (!drag.moved && Math.hypot(px - drag.x0, py - drag.y0) < 3) return;
    drag.moved = true;
    held = drag.what === "join" ? moveJoin(drag.from, drag.i, xm(px)) : setDensity(drag.from, drag.i, ys(py));
    if (drag.what === "join") selected = drag.i - 1;
  }
  function up(e: PointerEvent) {
    if (!drag || e.pointerId !== drag.id) return;
    const was = drag;
    drag = null;
    svg?.releasePointerCapture(e.pointerId);
    if (was.moved) void keep(was.what === "join" ? `Moved the edge: ${wordsOf(held!, was.i - 1)}` : `Shaped ${wordsOf(held!, was.i)}`);
  }

  async function keep(display: string) {
    if (!held) return;
    const next = held;
    const refused = oncommit ? await oncommit(authoredZones(next), display) : null;
    if (refused) {
      said = refused;
      held = null;
      return;
    }
    kept = next;
    held = null;
    said = `${display}. The ring holds the same mass.`;
  }
  function cancel() {
    held = null;
    drag = null;
  }

  /** Arrow keys step a focused join or zone in pencil; Enter keeps it, Esc puts it back. */
  function key(e: KeyboardEvent, what: "join" | "zone", i: number) {
    if (e.key === "Escape" && held) return void (cancel(), e.preventDefault());
    if (e.key === "Enter" && held) return void (keep(what === "join" ? `Moved the edge: ${wordsOf(held, i - 1)}` : `Shaped ${wordsOf(held, i)}`), e.preventDefault());
    if (!editable) return;
    const now = held ?? spans;
    if (what === "join" && (e.key === "ArrowLeft" || e.key === "ArrowRight")) {
      const step = (ring.outer_m - ring.inner_m) * (e.shiftKey ? 5e-4 : 5e-3) * (e.key === "ArrowLeft" ? -1 : 1);
      held = moveJoin(now, i, now[i]!.inner_m + step);
      selected = i - 1;
      e.preventDefault();
    } else if (what === "zone" && (e.key === "ArrowUp" || e.key === "ArrowDown")) {
      const f = e.shiftKey ? 1.01 : 1.1;
      held = setDensity(now, i, now[i]!.sigma_kg_m2 * (e.key === "ArrowUp" ? f : 1 / f));
      selected = i;
      e.preventDefault();
    }
  }

  // Typing: the selected zone's edges (where they are joins) and its share, exactly as typed, in any unit of the kind.
  let typing = $state<{ what: "inner" | "outer" | "share"; text: string; why?: string } | null>(null);
  function startTyping(what: "inner" | "outer" | "share") {
    const s = spans[selected!]!;
    typing = { what, text: what === "share" ? share(s.share).text : dist(what === "inner" ? s.inner_m : s.outer_m).text };
  }
  function typed(e: KeyboardEvent) {
    if (!typing || selected === null) return;
    if (e.key === "Escape") return void (typing = null);
    if (e.key !== "Enter") return;
    const s = spans[selected]!;
    const kind = typing.what === "share" ? "share" : "length";
    const shown = typing.what === "share" ? share(s.share) : dist(typing.what === "inner" ? s.inner_m : s.outer_m);
    const r = readFigure(typing.text, kind, { unit: shown.unitId });
    if (!r.ok) return void (typing = { ...typing, why: r.why });
    held = typing.what === "share" ? setShare(spans, selected, r.si) : moveJoin(spans, typing.what === "inner" ? selected : selected + 1, r.si);
    const i = selected;
    typing = null;
    void keep(`Typed ${wordsOf(held, i)}`);
  }

  async function reset() {
    const refused = kept && onreset ? await onreset() : null;
    if (refused) return void (said = refused);
    held = null;
    kept = null;
    typing = null;
    said = "Back to the ring as the physics drew it.";
  }
  const shaped = $derived(!!kept || !!held);
  /** The selected zone's typeable figures: its edges where they are joins (the ring's own are Where's), and its share. */
  const figs = $derived(
    selected === null
      ? []
      : ([
          { what: "inner", word: "From", can: joinAt.includes(selected) },
          { what: "outer", word: "to", can: joinAt.includes(selected + 1) },
          { what: "share", word: "share", can: true },
        ] as const),
  );
</script>

<div class="profile" data-profile bind:clientWidth={W}>
  <p class="head">How much there is, across the ring{#if held}<span class="pencil"> · held, not kept</span>{/if}</p>
  <svg
    bind:this={svg}
    width={W}
    height={H}
    viewBox={`0 0 ${W} ${H}`}
    role="group"
    aria-label={`The ring's profile: ${spans.length} zones from ${dist(ring.inner_m).text} to ${dist(ring.outer_m).text}`}
    onpointermove={move}
    onpointerup={up}
    onpointercancel={cancel}
  >
    <text class="axis-label" x={pad} y={TOP - 6}>{depth === "curious" ? "more ↑" : "denser ↑"}</text>
    <!-- Zones: spans along the line, each as dense as its height. -->
    {#each spans as s, i (i)}
      {@const x0 = x(s.inner_m)}
      {@const x1 = x(s.outer_m)}
      {@const top = y(s.sigma_kg_m2)}
      <g class="zone" class:sel={selected === i} class:yours={s.yours} class:held={!!held} data-zone={s.name}>
        <rect
          class="body"
          x={x0}
          y={top}
          width={Math.max(1, x1 - x0)}
          height={BASE - top}
          fill={rgb(s.rgb)}
          tabindex="0"
          role="slider"
          aria-orientation="vertical"
          aria-label={`${s.name}: ${share(s.share).text} of the ring`}
          aria-valuenow={s.share}
          aria-valuemin={0}
          aria-valuemax={1}
          aria-valuetext={verdict(i)}
          aria-readonly={!editable}
          onpointerdown={(e) => down(e, "zone", i)}
          onkeydown={(e) => key(e, "zone", i)}
          onfocus={() => (selected = i)}
        ><title>{verdict(i)}</title></rect>
        <line class="lid" x1={x0} x2={x1} y1={top} y2={top} />
        {#if x1 - x0 > s.name.length * 6 + 8}<text class="name" x={(x0 + x1) / 2} y={Math.min(top, BASE - 14) - 4}>{s.name}</text>{/if}
      </g>
    {/each}
    <line class="base" x1={pad} x2={W - pad} y1={BASE} y2={BASE} />
    <!-- Joins: the edge between two touching zones, dragged sideways. -->
    {#if editable}
      {#each joinAt as j (j)}
        {@const jx = x(spans[j]!.inner_m)}
        <g class="join" data-join={j}>
          <line x1={jx} x2={jx} y1={TOP} y2={BASE} />
          <path d={`M ${jx} ${TOP - 4} l 4 4 l -4 4 l -4 -4 z`} />
          <rect
            class="grip"
            x={jx - 11}
            y={TOP - 6}
            width="22"
            height={PLOT + 6}
            tabindex="0"
            role="slider"
            aria-orientation="horizontal"
            aria-label={`Edge between ${spans[j - 1]!.name} and ${spans[j]!.name}`}
            aria-valuenow={spans[j]!.inner_m}
            aria-valuemin={spans[j - 1]!.inner_m}
            aria-valuemax={spans[j]!.outer_m}
            aria-valuetext={dist(spans[j]!.inner_m).text}
            onpointerdown={(e) => down(e, "join", j)}
            onkeydown={(e) => key(e, "join", j)}
          ><title>{`Edge at ${dist(spans[j]!.inner_m).text}: drag sideways`}</title></rect>
        </g>
      {/each}
    {/if}
    <!-- What it looks like: the kernel's fine profile, or the zones' own opacity once a shape is held. -->
    {#if shaped}
      {#each spans as s, i (i)}<rect class="seen" x={x(s.inner_m)} y={SEEN} width={Math.max(0.5, x(s.outer_m) - x(s.inner_m))} height="8" fill={rgb(s.rgb)} fill-opacity={1 - lightThrough(s.tau)} />{/each}
    {:else}
      {@const n = ring.profile.tau.length}
      {@const dx = (ring.profile.hi_m - ring.profile.lo_m) / Math.max(1, n)}
      {#each ring.profile.tau as t, k (k)}
        {@const z = ring.zones[ring.profile.zone[k] ?? -1]}
        {#if z}<rect class="seen" x={x(ring.profile.lo_m + k * dx)} y={SEEN} width={Math.max(0.5, x(ring.profile.lo_m + (k + 1) * dx) - x(ring.profile.lo_m + k * dx) + 0.3)} height="8" fill={rgb(z.rgb)} fill-opacity={1 - lightThrough(t)} />{/if}
      {/each}
    {/if}
    <text class="tick" x={x(ring.inner_m)} y={AXIS} text-anchor="start">{dist(ring.inner_m).text}</text>
    <text class="tick" x={x(ring.outer_m)} y={AXIS} text-anchor={hi > ring.outer_m ? "middle" : "end"}>{dist(ring.outer_m).text}</text>
    {#if selected !== null && spans[selected]}
      {@const s = spans[selected]!}
      {#if x(s.inner_m) - x(ring.inner_m) > 90 && x(ring.outer_m) - x(s.inner_m) > 60}<text class="tick sel" x={x(s.inner_m)} y={AXIS} text-anchor="middle">{dist(s.inner_m).text}</text>{/if}
    {/if}
    <!-- The sculptors' marks, each with a thin line down to its sculptor. -->
    {#each marks as m (m.at)}
      <line class="mark" x1={m.px} x2={m.px} y1={TOP} y2={BASE} />
      <line class="tie" x1={m.px} y1={MARKS - 4 + Math.max(0, m.row) * 12} x2={m.to} y2={H - 34} />
      {#if m.row >= 0}<text class="mark-label" x={m.px} y={MARKS - 8 + m.row * 12}><title>{m.words}</title>{m.label}</text>{/if}
    {/each}
    <line class="moons" x1={pad} x2={W - right} y1={H - 34} y2={H - 34} />
    <text class="axis-label" x={pad} y={H - 20}>{depth === "curious" ? "moons" : "sculptors"}</text>
    {#each near as n (n.s.id)}
      <circle class="moon" cx={n.px} cy={H - 34} r="3"><title>{`${n.s.name} at ${dist(n.s.a_m).text}`}</title></circle>
      {#if n.row >= 0}<text class="moon-label" x={n.px} y={H - 20 + n.row * 12}>{n.s.name}</text>{/if}
    {/each}
    {#if farText}<text class="moon-label far" x={W - pad} y={H - 30} text-anchor="end">{farText}</text>{/if}
  </svg>
  <p class="foot">distance from {parent} →</p>

  {#if selected !== null && spans[selected]}
    {@const s = spans[selected]!}
    <p class="verdict" class:pencil={!!held} data-profile-verdict>{verdict(selected)}</p>
    {#each edgesHeld(s, ring.marks, sculptors) as line (line)}<p class="verdict sub">{line}</p>{/each}
    {#if depth === "engine"}<p class="verdict sub engine">zones[{selected}] · sigma_kg_m2 {sayNumber(s.sigma_kg_m2, 3)} · mass_kg {sayNumber(s.mass_kg, 3)} · face {sayNumber(annulus(s.inner_m, s.outer_m), 3)} m²</p>{/if}
    {#if editable}
      <div class="figures" role="group" aria-label={`${s.name}'s figures`}>
        {#each figs as { what, word, can } (what)}
          <span class="lead">{word}</span>
          {#if typing?.what === what}
            <!-- svelte-ignore a11y_autofocus -->
            <input class="entry" data-profile-entry={what} bind:value={typing.text} onkeydown={typed} onblur={() => (typing = null)} autofocus aria-label={`Type ${s.name}'s ${what === "share" ? "share of the ring" : `${what} edge`}`} />
          {:else if can}
            <button type="button" class="figure" data-profile-figure={what} onclick={() => startTyping(what)}>{what === "share" ? share(s.share).text : dist(what === "inner" ? s.inner_m : s.outer_m).text}</button>
          {:else}
            <span class="figure fixed" title="The ring's own edge: set it with Where">{dist(what === "inner" ? s.inner_m : s.outer_m).text}</span>
          {/if}
        {/each}
      </div>
      {#if typing?.why}<p class="verdict warn" role="alert">{typing.why}</p>{/if}
    {/if}
  {:else}
    <p class="verdict muted">{editable ? "Press a zone to see it; drag a zone up or down, or an edge sideways. The ring keeps its mass." : "Press a zone to see it."}</p>
  {/if}
  {#if said}<p class="verdict note" role="status">{said}</p>{/if}
  <div class="menu" role="group" aria-label="More for the profile">
    {#if editable && shaped}<button type="button" class="link" data-profile-reset onclick={reset}>Reset</button>{/if}
    <button type="button" class="link" aria-pressed={why} data-profile-why onclick={() => (why = !why)}>Why?</button>
  </div>
  {#if why}
    <ul class="why">{#each ring.why as w (w)}<li>{w}</li>{/each}</ul>
  {/if}
</div>

<style>
  .profile {
    padding: 6px 18px 10px;
    border-bottom: var(--sse-line) solid var(--sse-rule);
    background: var(--sse-ground);
    min-width: 0;
  }
  .head,
  .foot {
    margin: 0 0 4px;
    font-size: 12px;
    color: var(--sse-ink-muted);
  }
  .foot {
    margin: -2px 0 6px;
    text-align: right;
  }
  svg {
    display: block;
    width: 100%;
    touch-action: none;
    user-select: none;
    overflow: visible;
  }
  .zone .body {
    fill-opacity: 0.55;
    stroke: none;
    cursor: default;
    outline: none;
  }
  .zone .body:focus-visible {
    stroke: var(--sse-accent);
    stroke-width: 2;
  }
  .zone .lid {
    stroke: var(--sse-ink);
    stroke-width: 1.5;
    pointer-events: none;
  }
  .zone.yours .lid {
    stroke: var(--sse-accent);
    stroke-width: 2.5;
  }
  .zone.held .lid {
    stroke-dasharray: 3 2;
  }
  .zone.sel .body {
    fill-opacity: 0.8;
  }
  .zone.sel .lid {
    stroke-width: 3;
  }
  .name {
    font-size: 11px;
    fill: var(--sse-ink);
    text-anchor: middle;
    pointer-events: none;
  }
  .base,
  .moons {
    stroke: var(--sse-ink-muted);
    stroke-width: 1;
  }
  .join line {
    stroke: var(--sse-ink-muted);
    stroke-width: 1;
    stroke-dasharray: 2 2;
    pointer-events: none;
  }
  .join path {
    fill: var(--sse-accent);
    pointer-events: none;
  }
  .grip {
    fill: transparent;
    cursor: ew-resize;
    outline: none;
  }
  .grip:focus-visible {
    stroke: var(--sse-accent);
    stroke-width: 1.5;
  }
  .seen {
    pointer-events: none;
  }
  .tick,
  .axis-label,
  .mark-label,
  .moon-label {
    font-size: 10.5px;
    fill: var(--sse-ink-muted);
  }
  .tick.sel {
    fill: var(--sse-ink);
  }
  .mark-label,
  .moon-label {
    text-anchor: middle;
  }
  .mark {
    stroke: var(--sse-ink-muted);
    stroke-width: 1;
    stroke-dasharray: 1 3;
    pointer-events: none;
  }
  .tie {
    stroke: var(--sse-ink-muted);
    stroke-width: 0.6;
    opacity: 0.7;
    pointer-events: none;
  }
  .moon {
    fill: var(--sse-ink);
  }
  .verdict {
    margin: 4px 0 0;
    font-size: 12px;
    color: var(--sse-ink);
  }
  .verdict.sub,
  .verdict.muted,
  .verdict.note {
    color: var(--sse-ink-muted);
  }
  .verdict.engine {
    font-family: var(--sse-mono, monospace);
    font-size: 11px;
  }
  .verdict.warn {
    color: var(--sse-warn, var(--sse-ink));
  }
  .pencil {
    font-style: italic;
    color: var(--sse-ink-muted);
  }
  .figures {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 6px;
    margin-top: 6px;
    font-size: 12px;
  }
  .lead {
    color: var(--sse-ink-muted);
  }
  .figure,
  .entry {
    min-height: 26px;
    padding: 2px 8px;
    border: var(--sse-line) solid var(--sse-ink-muted);
    border-radius: 4px;
    background: var(--sse-ground-raised);
    color: var(--sse-ink);
    font: inherit;
    font-size: 12px;
  }
  .figure {
    cursor: text;
  }
  .figure.fixed {
    border-style: dashed;
    color: var(--sse-ink-muted);
    cursor: default;
    display: inline-flex;
    align-items: center;
  }
  .entry {
    width: 12em;
  }
  .menu {
    display: flex;
    gap: 12px;
    margin-top: 8px;
    font-size: 12px;
  }
  .link {
    padding: 0;
    border: 0;
    background: none;
    color: var(--sse-ink-muted);
    font: inherit;
    text-decoration: underline;
    cursor: pointer;
  }
  .why {
    margin: 4px 0 0;
    padding-left: 18px;
    font-size: 12px;
    color: var(--sse-ink-muted);
  }
</style>

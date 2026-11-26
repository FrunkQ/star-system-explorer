<!--
  The star chooser (ED6 part 2, editing design section 9): what a star's Mass handle opens. The mass strip runs from
  the lightest brown dwarf to the heaviest star the fits cover, drawn in the colour of each mass's light at this
  system's present age, with the spectral letters under it and the walls where the answer changes kind. Under it, the
  life bar of the held mass with the present age marked. Kinds chips find the mass that is that kind now. Everything
  is read off the kernel's life line (describe_star); letting go writes the star's mass, radius, light and temperature
  as one change, kept on Build and a try on Explore (TM4).
-->
<script lang="ts">
  import { STAR_KINDS, constant, kindMass, massAt, massPos, readMassStrip, say, starEdits, birthMass, systemAgeYr, yearsToSeconds, type Depth, type Describe, type Json, type StarDescription, type StarKind, type StarSample, type ViewContext } from "../../../ui-contract/src/index.ts";

  let {
    ctx,
    facts,
    systemFacts,
    now_yr,
    depth,
    editable,
    trying = false,
    commitMix,
  }: {
    ctx: ViewContext;
    /** The star's own facts (the kernel's, or a what-if's over them). */
    facts: Record<string, Json>;
    /** The system's facts: its present age is `system.age`. */
    systemFacts: Record<string, Json>;
    /** The ruler's now, for a system whose record has no age. */
    now_yr: number;
    depth: Depth;
    editable: boolean;
    trying?: boolean;
    commitMix: (edits: { field: string; value: number; before: Json }[], display: string, before_display: string, label?: string) => Promise<string | null>;
  } = $props();

  const describe: Describe | null = $derived(ctx.session.describeStar ? (m, a, f) => ctx.session.describeStar!(m, a, f) : null);
  const age = $derived(systemAgeYr(systemFacts) ?? now_yr);
  const feh = $derived(typeof facts["star.feh"] === "number" ? (facts["star.feh"] as number) : undefined);
  const born = $derived(birthMass(facts));

  let strip = $state.raw<{ samples: StarSample[]; walls: StarDescription["walls"]; key: string } | null>(null);
  let problem = $state<string | null>(null);
  $effect(() => {
    const d = describe;
    const key = `${age}|${feh}`;
    if (!d || strip?.key === key) return;
    let live = true;
    void readMassStrip(d, age, feh, 64).then((r) => {
      if (!live) return;
      if (!r) problem = "The kernel could not describe stars here.";
      else strip = { ...r, key };
    });
    return () => (live = false);
  });
  const lo = $derived(strip ? strip.walls.deuterium_msun : 0.0124);
  const hi = $derived(strip ? strip.walls.fits_msun : 100);

  // The held mass (dragging, the arrow keys, a kind) in pencil, and what the life line says of it.
  let held = $state<number | null>(null);
  let heldSays = $state.raw<StarDescription | null>(null);
  let nowSays = $state.raw<StarDescription | null>(null);
  let busy = $state(false);
  const shown = $derived(held ?? born);
  $effect(() => {
    const d = describe;
    const m = born;
    if (!d || m === null) return;
    let live = true;
    void d(Math.min(hi * 0.999, Math.max(lo * 1.001, m)), age, feh).then((r) => live && r.ok && (nowSays = r.star));
    return () => (live = false);
  });
  let asked = 0;
  async function hold(m: number) {
    held = Math.min(hi * 0.999, Math.max(lo * 1.001, m));
    if (!describe) return;
    const n = ++asked;
    const r = await describe(held, age, feh);
    if (n === asked && r.ok) heldSays = r.star;
  }

  const sayMass = (m_msun: number) => say(m_msun * constant("M_sun"), "mass", { class: "star", what: "mass" }, { depth }).text;
  const sayAge = (yr: number) => say(Number(yearsToSeconds(yr)), "time", { class: "star", what: "age" }, { depth }).text;
  function line(d: StarDescription): string {
    const r = d.now;
    const remnant = r.phase === "white-dwarf" || r.phase === "neutron-star" || r.phase === "black-hole";
    const bits = [`${r.label}`];
    if (r.phase !== "black-hole") bits.push(say(r.teff_k, "temperature", { class: "star" }, { depth }).text, say(r.luminosity_w, "power", { class: "star", what: "light" }, { depth }).text);
    if (remnant) bits.push(`${sayMass(r.mass_msun)} left`);
    return bits.join(", ");
  }

  const lower = (t: string) => t.charAt(0).toLowerCase() + t.slice(1);
  async function choose(m: number, label?: string) {
    if (!describe || busy) return;
    busy = true;
    problem = null;
    const r = await describe(m, age, feh);
    if (!r.ok) {
      busy = false;
      problem = r.message;
      return;
    }
    const was = born === null ? "not set" : sayMass(born);
    const display = `${label ? `${label}: ` : ""}born at ${sayMass(r.star.mass_msun)}, ${lower(line(r.star))}`;
    problem = await commitMix(starEdits(r.star, facts), display, was, "Star");
    held = null;
    heldSays = null;
    busy = false;
  }

  // Kinds: the mass that is each kind at this age, worked out when the strip is read.
  let kinds = $state.raw<{ kind: StarKind; mass: number | null }[]>([]);
  $effect(() => {
    const s = strip;
    const d = describe;
    if (!s || !d) return;
    let live = true;
    void (async () => {
      const out: { kind: StarKind; mass: number | null }[] = [];
      for (const k of STAR_KINDS) out.push({ kind: k, mass: await kindMass(d, k, s.samples, age, feh) });
      if (live) kinds = out;
    })();
    return () => (live = false);
  });

  // The strip's paint: one stop per mass; clear where a black hole gives no light.
  const gradient = $derived(strip ? `linear-gradient(to right, ${strip.samples.map((s) => `${s.colour ?? "transparent"} ${massPos(s.mass_msun, lo, hi) * 100}%`).join(", ")})` : "none");
  // Each letter at the middle of its run along the strip.
  const letters = $derived.by(() => {
    if (!strip) return [];
    const runs: { letter: string; from: number; to: number }[] = [];
    for (const s of strip.samples) {
      const p = massPos(s.mass_msun, lo, hi);
      const last = runs.at(-1);
      if (s.letter && last?.letter === s.letter) last.to = p;
      else if (s.letter) runs.push({ letter: s.letter, from: p, to: p });
    }
    return runs.map((r) => ({ letter: r.letter, at: (r.from + r.to) / 2 }));
  });
  const walls = $derived(
    strip
      ? [
          { at: strip.walls.hydrogen_msun, says: "hydrogen fusion from" },
          { at: strip.walls.white_dwarf_msun, says: "ends as a neutron star from" },
          { at: strip.walls.black_hole_msun, says: "ends as a black hole from" },
        ]
      : [],
  );

  // The life bar of the held (or the star's own) mass: its phases along its life, the present age marked.
  const life = $derived(heldSays ?? nowSays);
  const lifeEnd = $derived.by(() => {
    if (!life) return 1;
    const last = life.phases.filter((p) => p.to_yr !== null).at(-1)?.to_yr ?? age;
    return Math.max(last, age) * 1.12;
  });

  let track: HTMLDivElement | undefined = $state();
  let dragging = false;
  const posOf = (e: PointerEvent) => {
    const b = track!.getBoundingClientRect();
    return (e.clientX - b.left) / b.width;
  };
  function down(e: PointerEvent) {
    if (!editable || busy || e.button !== 0) return;
    dragging = true;
    track!.setPointerCapture(e.pointerId);
    void hold(massAt(posOf(e), lo, hi));
  }
  function move(e: PointerEvent) {
    if (dragging) void hold(massAt(posOf(e), lo, hi));
  }
  function up() {
    if (!dragging) return;
    dragging = false;
    if (held !== null) void choose(held);
  }
  function key(e: KeyboardEvent) {
    if (!editable || busy) return;
    const m = held ?? born ?? 1;
    const step = e.shiftKey ? 1.25 : 1.04;
    if (e.key === "ArrowRight" || e.key === "ArrowUp") (e.preventDefault(), void hold(m * step));
    else if (e.key === "ArrowLeft" || e.key === "ArrowDown") (e.preventDefault(), void hold(m / step));
    else if (e.key === "Enter" && held !== null) (e.preventDefault(), void choose(held));
    else if (e.key === "Escape" && held !== null) (e.preventDefault(), e.stopPropagation(), (held = null), (heldSays = null));
  }
</script>

<div class="chooser" data-star-chooser>
  {#if !describe}
    <p class="note">This session cannot read stars off the life line.</p>
  {:else}
    {#if editable}
      <div class="kinds" role="group" aria-label="Kinds of star" data-star-kinds>
        {#each kinds as k (k.kind.id)}
          <button
            type="button"
            class="chip"
            data-star-kind={k.kind.id}
            disabled={busy || k.mass === null}
            title={k.mass === null ? `No star is a ${k.kind.label.toLowerCase()} at ${sayAge(age)}: change the system's age with Look ahead on the time ruler.` : `Born at ${sayMass(k.mass)}`}
            onclick={() => k.mass !== null && void choose(k.mass, k.kind.label)}>{k.kind.label}</button>
        {/each}
        {#if !kinds.length}<span class="note">Finding the kinds at {sayAge(age)}…</span>{/if}
      </div>
    {/if}
    <div class="head">
      <span class="name">{depth === "curious" ? "Born this heavy" : "Mass at birth"}</span>
      <span class="figure" class:pencil={held !== null} data-star-mass>{shown === null ? "not set" : sayMass(shown)}</span>
    </div>
    <div
      class="track"
      class:live={editable}
      bind:this={track}
      style="background: {gradient}"
      role="slider"
      tabindex={editable ? 0 : -1}
      aria-label="Mass at birth"
      aria-valuemin={lo}
      aria-valuemax={hi}
      aria-valuenow={shown ?? 0}
      aria-valuetext={shown === null ? "not set" : sayMass(shown)}
      aria-readonly={!editable}
      onpointerdown={down}
      onpointermove={move}
      onpointerup={up}
      onpointercancel={() => ((dragging = false), (held = null))}
      onkeydown={key}
      data-star-track
    >
      {#each walls as w (w.says)}<span class="wall" style="left: {massPos(w.at, lo, hi) * 100}%" title={w.says}></span>{/each}
      {#if shown !== null}<span class="mark" class:pencil={held !== null} style="left: {massPos(shown, lo, hi) * 100}%" data-star-mark></span>{/if}
    </div>
    <div class="letters" aria-hidden="true">
      {#each letters as l (l.letter + l.at)}<span style="left: {l.at * 100}%">{l.letter}</span>{/each}
    </div>
    <p class="note" data-star-walls>{walls.map((w) => `${w.says[0]!.toUpperCase()}${w.says.slice(1)} ${sayMass(w.at)}`).join("; ")}.</p>
    {#if life}
      <p class="now" aria-live="polite" data-star-now>{held !== null ? "Held here: " : "Now: "}{line(life)}{held !== null ? ". Let go, or press Enter, to set it. Esc puts it back." : "."}</p>
      <div class="life" aria-label="Its life" data-star-life>
        {#each life.phases as p (p.phase)}
          {@const to = p.to_yr ?? lifeEnd}
          <span class="phase {p.phase}" class:here={age >= p.from_yr && age < to} style="left: {(p.from_yr / lifeEnd) * 100}%; width: {(Math.max(0, Math.min(to, lifeEnd) - p.from_yr) / lifeEnd) * 100}%" title="{p.label}: from {sayAge(Math.max(p.from_yr, 1))}"></span>
        {/each}
        <span class="age" style="left: {Math.min(1, age / lifeEnd) * 100}%" title="This system's present age, {sayAge(age)}"></span>
      </div>
      <p class="note">{life.phases.filter((p) => (p.to_yr ?? lifeEnd) - p.from_yr > lifeEnd * 0.04).map((p) => p.label).join(", then ")}. This system is {sayAge(age)} old{editable ? "; change its age with Look ahead on the time ruler" : ""}.</p>
    {/if}
    {#if editable && trying && held === null}<p class="note" data-strip-trying>A try: it opens a what-if, and nothing is kept unless you press Keep it.</p>{/if}
    {#if problem}<p class="problem" role="status">{problem}</p>{/if}
  {/if}
</div>

<style>
  .chooser {
    display: flex;
    flex-direction: column;
    gap: 4px;
    min-width: 0;
    padding: 8px 18px 10px;
    border-bottom: var(--sse-line) solid var(--sse-rule);
    font-size: 12px;
  }
  .kinds {
    display: flex;
    flex-wrap: wrap;
    gap: 4px;
  }
  .chip {
    padding: 2px 8px;
    border: var(--sse-line) solid var(--sse-rule);
    background: var(--sse-ground-raised);
    color: var(--sse-ink);
    font: inherit;
    cursor: pointer;
  }
  .chip:hover:not(:disabled) {
    border-color: var(--sse-ink-muted);
  }
  .chip:disabled {
    color: var(--sse-ink-faint);
    cursor: default;
  }
  .head {
    display: flex;
    justify-content: space-between;
    align-items: baseline;
    margin-top: 4px;
  }
  .name {
    font-size: 11px;
    letter-spacing: 0.04em;
    color: var(--sse-ink-muted);
  }
  .figure {
    font-family: var(--sse-font-mono);
    font-size: 13px;
  }
  .figure.pencil {
    color: var(--sse-accent-ink);
  }
  .track {
    position: relative;
    height: 18px;
    border: var(--sse-line) solid var(--sse-rule);
    touch-action: none;
  }
  .track.live {
    cursor: pointer;
  }
  .track:focus-visible {
    outline: 2px solid var(--sse-accent);
    outline-offset: 2px;
  }
  .wall {
    position: absolute;
    top: -3px;
    bottom: -3px;
    width: 0;
    border-left: var(--sse-line) dashed var(--sse-ink-muted);
  }
  .mark {
    position: absolute;
    top: -5px;
    bottom: -5px;
    width: 4px;
    margin-left: -2px;
    background: var(--sse-accent);
  }
  .mark.pencil {
    opacity: 0.6;
  }
  .letters {
    position: relative;
    height: 14px;
    font-family: var(--sse-font-mono);
    font-size: 10px;
    color: var(--sse-ink-muted);
  }
  .letters span {
    position: absolute;
    transform: translateX(-50%);
    white-space: nowrap;
  }
  .now {
    margin: 4px 0 0;
  }
  .life {
    position: relative;
    height: 10px;
    background: var(--sse-ground);
    border: var(--sse-line) solid var(--sse-rule);
  }
  .phase {
    position: absolute;
    top: 0;
    bottom: 0;
    background: var(--sse-ink-faint);
    border-right: var(--sse-line) solid var(--sse-ground-raised);
  }
  .phase.main-sequence {
    background: var(--sse-ink-muted);
  }
  .phase.here {
    background: var(--sse-accent);
  }
  .age {
    position: absolute;
    top: -4px;
    bottom: -4px;
    width: 0;
    border-left: 2px solid var(--sse-ink);
  }
  .note {
    margin: 2px 0 0;
    color: var(--sse-ink-muted);
  }
  .problem {
    margin: 2px 0 0;
    overflow-wrap: anywhere;
    color: var(--sse-accent-ink);
  }
</style>

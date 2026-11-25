<!--
  A handle's box (ED12, editing design section 11F): what the Orbit and Spin handles open, under the row. The handle's
  own strip is on top; under it the rest of the subject as strips and rings (shape, where it is now, lean, the turn of
  the ellipse and where it crosses the plane; tilt), the chips (Retrograde, Locked, In step with), and the results the
  kernel gives back (the year, the stability verdict, whether tides would have locked it) with the orbit's geometry.
  Parts are declared in the handle sheet, so a pack's box draws the same way. Each move is one set_value, one Undo.
-->
<script lang="ts">
  import {
    boxAt,
    epochMeanFor,
    mirrorAngle,
    orbitShape,
    partHandle,
    resonances,
    resultsOf,
    RESULTS,
    say,
    sayNumber,
    solarDay,
    stabilityFor,
    outerWall,
    trueAnomalyFrom,
    yearsToSeconds,
    type Depth,
    type Handle,
    type Json,
    type OrbitPlaced,
    type SceneSubject,
    type StabilityCheck,
    type StripDot,
    type SystemScene,
    type ViewContext,
  } from "../../../ui-contract/src/index.ts";
  import Strip from "./Strip.svelte";
  import Ring from "./Ring.svelte";

  type How = { source: "computed" | "typed"; sig?: number };
  let {
    ctx,
    scene,
    subject,
    handle,
    facts,
    surfaces,
    cls,
    depth,
    editable,
    reading,
    dots,
    commit,
    ontry,
    onclose,
  }: {
    ctx: ViewContext;
    scene: SystemScene;
    subject: SceneSubject;
    handle: Handle;
    /** The kernel's facts for every body of the system. */
    facts: Record<string, Record<string, Json>>;
    surfaces: unknown;
    cls: string;
    depth: Depth;
    editable: boolean;
    reading?: string;
    dots: StripDot[];
    /** One set_value of `h`'s field to `si`; `shown` is what the change is called (the place now, not the epoch's angle). */
    commit: (h: Handle, si: number, how: How, shown?: string) => Promise<string | null>;
    ontry?: () => void;
    onclose: () => void;
  } = $props();

  const ws = $derived(ctx.workspace);
  const mine = $derived(facts[subject.address] ?? {});
  const num = (f: string): number | null => (typeof mine[f] === "number" ? (mine[f] as number) : null);
  const box = $derived(boxAt(handle, depth));
  const isOrbit = $derived(handle.part.field === "orbit.semi_major_axis");

  // The kernel's orbits now (orbits_at at the campaign's now) and, for the Orbit box, its stability checks.
  const now_s = $derived(Number(yearsToSeconds(ws.now_yr)));
  let placed = $state.raw<OrbitPlaced[]>([]);
  let checks = $state.raw<StabilityCheck[] | null>(null);
  $effect(() => {
    const sys = scene.system;
    const at = yearsToSeconds(ws.now_yr);
    const orbit = isOrbit;
    void ws.revision;
    if (!sys) return;
    let live = true;
    void ctx.session.orbitsAt(sys, [at]).then((r) => live && r.ok && (placed = r.bodies));
    if (orbit) void ctx.session.orbitStability(sys).then((r) => live && (checks = r.ok ? r.checks : null));
    return () => (live = false);
  });
  const me = $derived(placed.find((b) => b.address === subject.address && b.placed));
  const year = $derived(me?.period_s ?? null);
  const e = $derived(num("orbit.eccentricity") ?? 0);
  const angles = $derived({ inc: num("orbit.inclination") ?? 0, node: num("orbit.ascending_node") ?? 0, peri: num("orbit.argument_of_periapsis") ?? 0 });
  const nuNow = $derived(me?.at?.[0] ? trueAnomalyFrom(me.at[0].r_m, angles) : null);
  const epoch_s = $derived(num("orbit.epoch_s") ?? (num("orbit.epoch_v3_ms") ?? 0) / 1000);

  const said = (si: number, kind: string, what?: string) => say(si, kind, { class: cls, ...(what ? { what } : {}) }, { depth }).text;

  /** A part's value as the box shows it: the place round the orbit now, or the field itself. */
  function partValue(h: Handle): number | null {
    if (h.part.as === "true_anomaly_now") return nuNow;
    return num(h.part.field);
  }
  /** A ring's angle written to its field: the place now becomes the epoch's mean anomaly. */
  function ringCommit(h: Handle, rad: number, how: How): Promise<string | null> {
    if (h.part.as === "true_anomaly_now") {
      if (year === null) return Promise.resolve("Its year is not known yet, so where it is now cannot be set.");
      return commit(h, epochMeanFor(rad, e, year, now_s, epoch_s), how, `${said(rad, "angle")} from its closest point`);
    }
    return commit(h, rad, how);
  }

  // A typed year on the Size strip: the distance with that year round the same host (a ∝ P^2/3, the kernel's year as
  // the yardstick).
  const fromYear = $derived.by(() => {
    const a = num("orbit.semi_major_axis");
    const P = year;
    return isOrbit && a !== null && P ? (t: number) => a * (t / P) ** (2 / 3) : undefined;
  });

  // The widest orbit that lasts round the host (ED7, design section 11: a moon's strip runs to the Hill limit), from the
  // kernel's stability checks; dragging stops short of it, typing may pass.
  const wall = $derived(isOrbit && checks && !ws.branch ? outerWall(checks, subject.address) : null);

  // Chips.
  const lean = $derived(handle.box?.parts.find((p) => p.part.field === "orbit.inclination"));
  const steps = $derived.by(() => {
    const a = num("orbit.semi_major_axis");
    if (!isOrbit || a === null || year === null || !me) return [];
    const others = placed
      .filter((b) => b.placed && b.address !== subject.address && b.parent === me.parent && b.period_s)
      .map((b) => ({ label: scene.subjects.find((s) => s.address === b.address)?.label ?? b.address, a: facts[b.address]?.["orbit.semi_major_axis"] as number, year: b.period_s! }))
      .filter((o) => typeof o.a === "number");
    return resonances({ a, year }, others);
  });
  const rot = $derived(num("body.rotation_period"));
  const kernelSays = $derived(resultsOf(surfaces, subject.address));
  const lockRow = RESULTS.find((r) => r.key === "locked");
  let chipProblem = $state<string | null>(null);
  let chipSaid = $state<string | null>(null);
  async function chip(id: string, does: string) {
    chipProblem = null;
    chipSaid = null;
    if (does === "mirror" && lean) {
      const i = num("orbit.inclination") ?? 0;
      chipProblem = await commit(partHandle(handle, lean), mirrorAngle(i), { source: "computed" });
      if (!chipProblem) chipSaid = mirrorAngle(i) > Math.PI / 2 ? "Set to go round backwards: the lean is now past 90°." : "Set to go round forwards again.";
    } else if (does === "flip" && rot !== null) {
      chipProblem = await commit(handle, -rot, { source: "computed" });
      if (!chipProblem) chipSaid = -rot < 0 ? "Set to spin backwards: the sun rises in the west." : "Set to spin forwards: the sun rises in the east.";
    } else if (does === "lock" && year !== null) {
      chipProblem = await commit(handle, (rot !== null && rot < 0 ? -1 : 1) * year, { source: "computed" });
      if (!chipProblem) chipSaid = `Day set to its ${cls === "moon" ? "month" : "year"}: the same face always looks at the ${cls === "moon" ? "planet" : "star"}.`;
    }
    void id;
  }
  async function step(a: number, label: string) {
    chipProblem = await commit(handle, a, { source: "computed" });
    if (!chipProblem) chipSaid = `Size set for ${label}.`;
  }
  const lockedNow = $derived(year !== null && rot !== null && Math.abs(Math.abs(rot) / year - 1) < 1e-3);

  // Results under the box.
  const results = $derived.by(() => {
    const out: { key: string; label: string; text: string; tone?: "warn" }[] = [];
    const a = num("orbit.semi_major_axis");
    // On a what-if: the orbit's geometry, and the surface's results from the branch's own resolve (BC1, the card
    // reads it); the year and the verdict are main-line ops (orbits_at, orbit_stability) that do not read a branch.
    for (const k of box.results.filter((x) => !ws.branch || x === "closest" || x === "farthest" || x === "sunlight_swing" || x === "locked")) {
      if (k === "year" && year !== null) out.push({ key: k, label: cls === "moon" ? "Month" : "Year", text: said(year, "time", "year") });
      else if ((k === "closest" || k === "farthest") && a !== null && e > 0) out.push({ key: k, label: k === "closest" ? "Closest" : "Farthest", text: said(orbitShape(a, e)[k], "length", "orbit") });
      else if (k === "sunlight_swing" && a !== null && e > 0 && cls !== "moon") out.push({ key: k, label: "Sunlight", text: `${say(orbitShape(a, e).swing, "share", { class: cls }, { depth }).text} more at the closest point` });
      else if (k === "stability" && checks) {
        const v = stabilityFor(checks, subject.address);
        if (v) out.push({ key: k, label: "Lasts?", text: `${v.word}: ${v.says}`, ...(v.level > 0 ? { tone: "warn" as const } : {}) });
      } else if (k === "solar_day" && rot !== null && year !== null) {
        const d = solarDay(rot, year);
        out.push({ key: k, label: "Sun's day", text: d === null ? `none: the same face always looks at the ${cls === "moon" ? "planet" : "star"}` : said(d, "time", "day") });
      } else if (k === "days_in_year" && rot !== null && year !== null) {
        const d = solarDay(rot, year);
        if (d !== null) out.push({ key: k, label: cls === "moon" ? "Days a month" : "Days a year", text: sayNumber(year / d, 3) });
      } else if (k === "locked" && typeof kernelSays.locked === "boolean" && lockRow)
        out.push({ key: k, label: "Tides", text: kernelSays.locked ? `would have locked it by now (${lockRow.yes})` : lockedNow ? "would not have locked it by now: something else holds this day" : `would not have locked it by now (${lockRow.no})` });
    }
    return out;
  });
</script>

<div class="box" data-box={handle.id}>
  <Strip {handle} {reading} value={num(handle.part.field)} {dots} {cls} {depth} {editable} {ontry} walls={wall ? [wall] : []} fromTime={fromYear} oncommit={(si, how) => commit(handle, si, how)} {onclose} />
  {#if box.parts.length || box.chips.length}
    <div class="parts">
      {#each box.parts as p (p.id)}
        {@const ph = partHandle(handle, p)}
        {#if p.part.scale === "ring"}
          <div class="ring-part">
            <Ring id={p.id} name={depth === "curious" ? p.names.curious : p.names.science} value={partValue(ph)} catches={p.catches ?? []} {depth} {editable} note={p.part.as === "true_anomaly_now" ? "from its closest point" : undefined} oncommit={(rad, how) => ringCommit(ph, rad, how)} />
          </div>
        {:else}
          <Strip handle={ph} value={partValue(ph)} dots={[]} {cls} {depth} {editable} marks={p.marks ?? []} walls={p.walls ?? []} oncommit={(si, how) => commit(ph, si, how)} />
        {/if}
      {/each}
    </div>
    {#if box.chips.length}
      <div class="chips" role="group" aria-label="Shortcuts">
        {#each box.chips as c (c.id)}
          {#if c.does === "resonance"}
            {#each steps as r (r.label)}
              <button type="button" class="chip" class:on={r.on} data-chip="resonance" data-resonance={r.label} disabled={!editable || r.on} title={r.says} onclick={() => step(r.a, r.label)}>{r.label}</button>
            {/each}
          {:else}
            {@const on = c.does === "mirror" ? (num("orbit.inclination") ?? 0) > Math.PI / 2 : c.does === "flip" ? (rot ?? 0) < 0 : lockedNow}
            {@const name = c.does === "lock" ? (cls === "moon" ? "Locked to its planet" : "Locked to its star") : depth === "curious" ? c.names.curious : c.names.science}
            <button
              type="button"
              class="chip"
              class:on
              data-chip={c.id}
              aria-pressed={on}
              disabled={!editable || (c.does === "lock" && (year === null || on)) || (c.does === "mirror" && !lean) || (c.does === "flip" && rot === null)}
              onclick={() => chip(c.id, c.does)}>{name}</button
            >
          {/if}
        {/each}
      </div>
    {/if}
    {#if chipProblem || chipSaid}<p class="chip-said" class:problem={!!chipProblem} aria-live="polite" data-chip-said>{chipProblem ?? chipSaid}</p>{/if}
  {/if}
  {#if ws.branch && box.results.length}<p class="chip-said" data-box-branch>The year and the verdict follow once it is kept.</p>{/if}
  {#if results.length}
    <dl class="results" data-box-results>
      {#each results as r (r.key)}
        <div class="result" class:warn={r.tone === "warn"} data-result={r.key}><dt>{r.label}</dt><dd>{r.text}</dd></div>
      {/each}
    </dl>
  {/if}
</div>

<style>
  .box {
    min-width: 0;
    background: var(--sse-ground);
    border-bottom: var(--sse-line) solid var(--sse-rule);
  }
  .box :global([data-strip]) {
    border-bottom: 0;
  }
  .parts {
    display: grid;
    gap: 0;
    min-width: 0;
  }
  .ring-part {
    padding: 6px 18px;
  }
  .chips {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    padding: 6px 18px 4px;
  }
  .chip {
    min-height: 30px;
    padding: 2px 10px;
    border: var(--sse-line) solid var(--sse-ink-muted);
    border-radius: 15px;
    background: var(--sse-ground-raised);
    color: var(--sse-ink);
    font: inherit;
    font-size: 12px;
    cursor: pointer;
  }
  .chip.on {
    border-color: var(--sse-accent);
    color: var(--sse-accent-ink);
  }
  .chip:disabled {
    cursor: default;
    opacity: 0.75;
  }
  .chip-said {
    margin: 2px 18px 4px;
    font-size: 12px;
    color: var(--sse-ink-muted);
  }
  .chip-said.problem {
    color: var(--sse-accent-ink);
  }
  .results {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(150px, 1fr));
    gap: 4px 12px;
    margin: 0;
    padding: 6px 18px 10px;
  }
  .result {
    min-width: 0;
  }
  dt {
    font-size: 11px;
    letter-spacing: 0.04em;
    color: var(--sse-ink-muted);
  }
  dd {
    margin: 0;
    font-family: var(--sse-font-mono);
    font-size: 12px;
    overflow-wrap: anywhere;
  }
  .result.warn dd {
    color: var(--sse-alert);
  }
</style>

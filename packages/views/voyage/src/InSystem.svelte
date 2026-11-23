<!--
  Voyages in this system (V8 part 3b). The same finder as between the stars:
  places are the system's bodies and belts, legs are NAV's transfers with
  their launch windows (or torch flights), fuel is carried and refilled
  where a giant can be skimmed, ice cracked or fuel bought. The orbits lie
  along one line by size (log scale, so the Kuiper belt fits beside Mercury);
  each leg arcs above it. "Next stop" goes there and asks again.
  Presentation only: the kernel does every sum.
-->
<script lang="ts">
  import { untrack } from "svelte";
  import { IN_SYSTEM_SHIPS, MINIMISE_WORDS, YEAR_S, navFromScene, orbitAxis, voyageSpan, type Json, type Minimise, type ShipAtWorldAsk, type SystemScene, type SystemVoyagePlan, type ViewContext } from "../../../ui-contract/src/index.ts";
  import AtWorld from "./AtWorld.svelte";

  let { ctx, scene, params }: { ctx: ViewContext; scene: SystemScene; params: Record<string, Json> } = $props();
  const first = untrack(() => ({ ship: String(params.ship ?? "fusion-torch"), minimise: String(params.minimise ?? "home_time") }));
  let ship = $state(IN_SYSTEM_SHIPS.some((s) => s.id === first.ship) ? first.ship : "fusion-torch");
  let minimise = $state<Minimise>((MINIMISE_WORDS.some((m) => m.id === first.minimise) ? first.minimise : "home_time") as Minimise);
  let mode = $state<"explore" | "to">("to");
  const fromScene = $derived(navFromScene(scene));
  let home = $state("");
  let from = $state("");
  let to = $state("");
  let visited = $state<string[]>([]);
  let plan = $state<SystemVoyagePlan | null>(null);
  let failed = $state<string | null>(null);
  let busy = $state(false);

  // Start at the first world with a port, else the first world; head for the next one out.
  $effect(() => {
    const bs = fromScene.bodies;
    const nav = fromScene.nav;
    if (!bs.length || !nav) return;
    untrack(() => {
      if (bs.some((b) => b.address === from)) return;
      // A port if there is one, else the first world in the temperate zone (where people often live), else the first.
      const port = nav.bodies.find((b) => (b as { port?: boolean }).port)?.address;
      const zone = scene.subjects.find((x) => x.kind === "zone" && x.band_au);
      const inZone = zone ? nav.bodies.find((b) => b.a_au >= zone.band_au![0] && b.a_au <= zone.band_au![1])?.address : undefined;
      from = home = port ?? inZone ?? bs[0]!.address;
      visited = [from];
      const i = bs.findIndex((b) => b.address === from);
      to = bs[i + 1 < bs.length ? i + 1 : Math.max(0, i - 1)]!.address;
    });
  });

  let gen = 0;
  $effect(() => {
    const nav = fromScene.nav;
    const c = (ctx.workspace as { campaign_now_yr?: number }).campaign_now_yr;
    const depart = typeof c === "number" && Number.isFinite(c) ? { depart_s: c * Number(YEAR_S) } : {};
    const ask = { nav, from, ship, minimise, visited: [...visited], ...depart, ...(mode === "to" && to ? { to } : {}) };
    if (!nav || !from || (mode === "to" && (!to || to === from))) return void (plan = null);
    const g = ++gen;
    busy = true;
    ctx.session.planVoyage(ask).then((r) => {
      if (g !== gen) return;
      busy = false;
      if (!r.ok) return void ((failed = r.message), (plan = null));
      failed = null;
      plan = r.plan as SystemVoyagePlan;
    });
  });

  const v = $derived(plan?.voyage ?? null);
  const places = $derived(plan?.places ?? []);
  const W = 640;
  const H = 220;
  const AXIS_Y = 160;
  const x = $derived(orbitAxis(W, places));
  const at = (a: string) => places.find((p) => p.address === a);
  const arcs = $derived(
    (v?.legs ?? []).flatMap((l, i) => {
      const a = at(l.from);
      const b = at(l.to);
      if (!a || !b) return [];
      const [x1, x2] = [x(a.a_au), x(b.a_au)];
      const lift = Math.min(120, 24 + Math.abs(x2 - x1) * 0.35);
      return [{ i, l, d: `M ${x1} ${AXIS_Y} Q ${(x1 + x2) / 2} ${AXIS_Y - lift * 2} ${x2} ${AXIS_Y}` }];
    }),
  );
  const nameOf = (a: string) => at(a)?.name ?? a;
  const shipInfo = $derived(IN_SYSTEM_SHIPS.find((s) => s.id === ship));
  // What the ship can do where the voyage ends: an adopted system's own record on the GM's seat (its air), else the chart's system.
  const atAsk = $derived.by((): ShipAtWorldAsk | null => {
    const dest = v?.to;
    if (!dest || dest.startsWith("belt:")) return null;
    // The last leg's own speed over escape at the world (a torch course arrives braked, 0); a leg without one
    // (a belt hop) leaves the kernel's default, a Hohmann transfer from the next body in.
    const last = v?.legs.at(-1);
    const speed = last?.arrive_v_inf_m_s !== undefined ? { v_inf_m_s: last.arrive_v_inf_m_s } : last?.course === "torch" ? { v_inf_m_s: 0 } : {};
    if (!scene.fixture && ctx.session.seat.authority === "gm") return { system: scene.system, ship, worlds: [dest], ...speed };
    return fromScene.nav ? { nav: fromScene.nav, ship, worlds: [dest], ...speed } : null;
  });
  const arriving = $derived.by(() => {
    const l = v?.legs.at(-1);
    if (l?.arrive_v_inf_m_s === undefined) return undefined;
    return l.course === "lambert" ? "arriving on this voyage's transfer arc" : l.course === "hohmann" ? "arriving on this voyage's Hohmann transfer" : "arriving on this voyage's last leg";
  });
  const waited = $derived(v ? v.legs.reduce((t, l) => t + (l.wait_days ?? 0), 0) : 0);

  function pick(a: string) {
    if (mode === "to" && a !== from) to = a;
  }
  function nextStop() {
    if (!v) return;
    visited = [...visited, ...v.legs.map((l) => l.to).filter((s) => !visited.includes(s))];
    from = v.to;
    if (mode === "to") to = "";
  }
  function restart() {
    from = home;
    visited = [home];
  }
</script>

<div class="insys" data-voyage={v ? `${v.from}>${v.to}` : ""} data-legs={v?.legs.length ?? 0}>
  <div class="bar">
    <label class="lbl">Ship
      <select data-ship bind:value={ship}>
        {#each IN_SYSTEM_SHIPS as s (s.id)}<option value={s.id}>{s.name}</option>{/each}
      </select>
    </label>
    <span class="seg" role="group" aria-label="Goal">
      <button type="button" data-mode="to" aria-pressed={mode === "to"} onclick={() => (mode = "to")}>To a place</button>
      <button type="button" data-mode="explore" aria-pressed={mode === "explore"} onclick={() => (mode = "explore")}>Nearest unvisited</button>
    </span>
    <label class="lbl">Keep smallest
      <select data-minimise bind:value={minimise}>
        {#each MINIMISE_WORDS as m (m.id)}<option value={m.id}>{m.label}</option>{/each}
      </select>
    </label>
  </div>

  <div class="head">
    <h2>Voyages in {plan?.system ?? "this system"}</h2>
    {#if shipInfo}<span class="muted">{shipInfo.name}: {shipInfo.drive}</span>{/if}
    <p class="lede">Transfers wait for their windows; the computer refuels where the ship can use what it finds: a giant to skim, ice to crack or a port that stocks its fuel. Pick a world or a belt to go there.</p>
  </div>

  {#if fromScene.notes.length && !fromScene.nav}<p class="warn">{fromScene.notes[0]}</p>{/if}

  <svg class="axis" viewBox="0 0 {W} {H}" role="img" aria-label="The system's orbits by size, with the voyage's legs arcing between them">
    <line class="base" x1="16" y1={AXIS_Y} x2={W - 16} y2={AXIS_Y} />
    {#each places.filter((p) => p.belt) as p (p.address)}
      <rect class="belt" x={x(p.a_au) - 10} y={AXIS_Y - 6} width="20" height="12" rx="6" />
    {/each}
    {#each arcs as a (a.i)}<path class="arc" d={a.d} />{/each}
    {#each places as p, i (p.address)}
      {@const px = x(p.a_au)}
      <g class="place" class:here={p.address === from} class:dest={v && p.address === v.to} class:seen={visited.includes(p.address)}>
        {#if p.fuel.length}<circle class="fuel" cx={px} cy={AXIS_Y} r="8" />{/if}
        <circle class="dot" cx={px} cy={AXIS_Y} r={p.address === from || (v && p.address === v.to) ? 5 : 3} />
        <text x={px} y={AXIS_Y + (i % 2 ? 34 : 22)} text-anchor="middle">{p.name}</text>
        <circle class="hit" role="button" tabindex="-1" aria-label={p.name} data-place={p.address} cx={px} cy={AXIS_Y} r="12" onclick={() => pick(p.address)} onkeydown={(e) => e.key === "Enter" && pick(p.address)} />
      </g>
    {/each}
  </svg>

  <div class="ledger" data-voyage-ledger>
    {#if failed}<p class="warn" data-failed>{failed}</p>{/if}
    {#if busy && !plan}<p class="muted">Charting…</p>{/if}
    {#if plan && !v}
      <h3>No voyage</h3>
      {#each plan.refusals as r, i (i)}<p class="warn" data-refusal>{r}</p>{/each}
    {/if}
    {#if v}
      <h3>{nameOf(v.from)} to {nameOf(v.to)}</h3>
      <dl>
        <dt>Arrives after</dt><dd data-home-time>{voyageSpan(v.home_days)}</dd>
        {#if waited >= 1}<dt>Waiting for windows</dt><dd data-waited>{voyageSpan(waited)}</dd>{/if}
        <dt>Propellant</dt><dd>{v.propellant_t.toFixed(1)} t</dd>
        <dt>Legs</dt><dd>{v.legs.length}, refuelling {v.refuels} time{v.refuels === 1 ? "" : "s"}</dd>
      </dl>
      <h4>Legs</h4>
      <ol class="legs">
        {#each v.legs as l, i (i)}
          <li data-leg={l.course ?? l.means}>{l.words}. <span class="muted">Left on arrival: {l.propellant_left_t.toFixed(1)} t.</span></li>
        {/each}
      </ol>
      <button type="button" class="go" data-next onclick={nextStop}>{mode === "explore" ? "Next stop: go there" : "Arrive there"}</button>
      {#if visited.length > 1}<button type="button" class="quiet-btn" data-restart onclick={restart}>Back to the start</button>{/if}
      <AtWorld {ctx} ask={atAsk} arriving={arriving} />
    {/if}
  </div>
</div>

<style>
  .insys {
    color: var(--sse-ink);
    font-size: 13px;
    display: flex;
    flex-direction: column;
    gap: 12px;
    max-width: 1180px;
  }
  .bar {
    display: flex;
    flex-wrap: wrap;
    gap: 10px 12px;
    align-items: center;
  }
  .lbl {
    font-family: var(--sse-font-mono, monospace);
    display: inline-flex;
    gap: 6px;
    align-items: center;
  }
  select {
    font: inherit;
    min-height: 34px;
    color: var(--sse-ink);
    background: var(--sse-ground);
    border: 1px solid var(--sse-rule);
  }
  .seg {
    display: inline-flex;
    flex-wrap: wrap;
  }
  .seg button,
  .go,
  .quiet-btn {
    font: inherit;
    min-height: 34px;
    padding: 0 10px;
    border: 1px solid var(--sse-ink);
    background: transparent;
    color: var(--sse-ink);
    margin-right: -1px;
    cursor: pointer;
  }
  .seg button[aria-pressed="true"],
  .go {
    background: var(--sse-ink);
    color: var(--sse-ground);
  }
  .go {
    margin: 10px 8px 0 0;
  }
  .quiet-btn {
    margin-top: 10px;
    border-color: var(--sse-rule);
  }
  .head {
    display: flex;
    flex-wrap: wrap;
    gap: 8px 12px;
    align-items: baseline;
  }
  h2 {
    font-family: var(--sse-font-display, serif);
    font-size: 24px;
    font-weight: 600;
    margin: 0;
  }
  h3 {
    font-family: var(--sse-font-display, serif);
    font-size: 18px;
    font-weight: 600;
    margin: 0 0 4px;
  }
  h4 {
    font-size: 11px;
    letter-spacing: 0.12em;
    text-transform: uppercase;
    color: var(--sse-ink-muted);
    margin: 14px 0 4px;
    font-weight: 500;
  }
  .lede {
    margin: 0;
    flex-basis: 100%;
    font-size: 14px;
  }
  .muted {
    color: var(--sse-ink-muted);
  }
  .warn {
    color: var(--sse-accent-ink, var(--sse-accent));
    margin: 4px 0;
  }
  .axis {
    width: 100%;
    height: auto;
    border: 1px solid var(--sse-rule);
    background: var(--sse-ground);
    animation: ins-in 0.24s ease-out;
  }
  @keyframes ins-in {
    from {
      opacity: 0;
    }
  }
  .base {
    stroke: var(--sse-rule);
  }
  .belt {
    fill: var(--sse-rule);
    opacity: 0.7;
  }
  .arc {
    fill: none;
    stroke: var(--sse-accent);
    stroke-width: 2;
  }
  .place .dot {
    fill: var(--sse-ink-muted);
  }
  .place.seen .dot {
    fill: var(--sse-ink);
  }
  .place.here .dot,
  .place.dest .dot {
    fill: var(--sse-accent);
  }
  .place .fuel {
    fill: none;
    stroke: var(--sse-ink);
  }
  .place text {
    fill: var(--sse-ink-muted);
    font-size: 10px;
  }
  .hit {
    fill: transparent;
    cursor: pointer;
  }
  dl {
    display: grid;
    grid-template-columns: auto 1fr;
    gap: 2px 12px;
    margin: 0;
  }
  dt {
    color: var(--sse-ink-muted);
  }
  dd {
    margin: 0;
    font-family: var(--sse-font-mono, monospace);
  }
  .legs {
    margin: 0;
    padding-left: 18px;
  }
  .legs li {
    margin: 0 0 6px;
  }
</style>

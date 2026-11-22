<!--
  Between stars (V8 part 3). The crew asks the ship's computer for a voyage:
  a ship, a goal (a star picked on the map, or the nearest one not yet
  visited) and what to keep smallest. The kernel's plan_voyage charts the
  legs over the campaign's star cells. The map is seen from above (x across,
  y up, parsecs); legs are drawn star to star, refuel stops ringed, jumps
  dashed (a pack's rule, fictional) and flights solid (physics). "Next stop"
  takes the explorer there and asks again, so the path spirals outward.
  Presentation only: the kernel does every sum.
-->
<script lang="ts">
  import { untrack } from "svelte";
  import { MINIMISE_WORDS, VOYAGE_SHIPS, agesWords, systemShort, voyageFrame, voyageSpan, type Json, type Minimise, type StarCell, type SystemScene, type ViewContext, type VoyagePlan } from "../../../ui-contract/src/index.ts";

  let { ctx, params }: { ctx: ViewContext; scene: SystemScene; params: Record<string, Json> } = $props();
  const first = untrack(() => ({ ship: String(params.ship ?? "jump-courier"), minimise: String(params.minimise ?? "home_time"), gambles: params.gambles === true }));

  let ship = $state(VOYAGE_SHIPS.some((s) => s.id === first.ship) ? first.ship : "jump-courier");
  let minimise = $state<Minimise>((MINIMISE_WORDS.some((m) => m.id === first.minimise) ? first.minimise : "home_time") as Minimise);
  let gambles = $state(first.gambles);
  let mode = $state<"explore" | "to">("explore");
  let cell = $state<StarCell | null>(null);
  let home = $state<string>("");
  let from = $state<string>("");
  let to = $state<string>("");
  let visited = $state<string[]>([]);
  let knownFuel = $state<Record<string, string[]>>({});
  let picked = $state<string>("");
  let plan = $state<VoyagePlan | null>(null);
  let failed = $state<string | null>(null);
  let busy = $state(false);

  // The home cell: the one at the Sun's distance from the galaxy's centre.
  const session = untrack(() => ctx.session);
  session.starCell({}).then(async (now) => {
    if (!now.ok) return void (failed = now.message);
    // The cell as drawn at its frame's start (the positions plan_voyage uses).
    const r = await session.starCell({ cell: now.cell.cell, t_myr: now.cell.frame_start_myr });
    if (!r.ok) return void (failed = r.message);
    cell = r.cell;
    // Start from the system nearest the cell's middle, and say it has fuel (a home port).
    const mid = r.cell.systems.reduce<[number, number, number]>((a, s) => [a[0] + s.xyz_pc[0] / r.cell.systems.length, a[1] + s.xyz_pc[1] / r.cell.systems.length, a[2] + s.xyz_pc[2] / r.cell.systems.length], [0, 0, 0]);
    const d = (p: readonly number[]) => Math.hypot(p[0]! - mid[0], p[1]! - mid[1], p[2]! - mid[2]);
    // Prefer one with a neighbour within 2 pc, so a Jump-2 ship has somewhere to go.
    const near = (p: readonly number[]) => r.cell.systems.some((o) => o.xyz_pc !== p && Math.hypot(o.xyz_pc[0] - p[0]!, o.xyz_pc[1] - p[1]!, o.xyz_pc[2] - p[2]!) <= 2);
    const byMid = [...r.cell.systems].sort((a, b) => d(a.xyz_pc) - d(b.xyz_pc));
    const start = byMid.find((s) => near(s.xyz_pc)) ?? byMid[0];
    if (!start) return void (failed = "this cell holds no stars");
    home = from = start.address;
    visited = [start.address];
    knownFuel = { [start.address]: ["port"] };
  });

  let gen = 0;
  $effect(() => {
    const ask = { from, ship, minimise, gambles, home, visited: [...visited], known_fuel: { ...knownFuel }, ...(mode === "to" && to ? { to } : {}) };
    if (!ask.from || (mode === "to" && !to)) return void (plan = null);
    const g = ++gen;
    busy = true;
    ctx.session.planVoyage(ask).then((r) => {
      if (g !== gen) return;
      busy = false;
      if (!r.ok) return void ((failed = r.message), (plan = null));
      failed = null;
      plan = r.plan as VoyagePlan;
    });
  });

  const v = $derived(plan?.voyage ?? null);
  const stars = $derived.by(() => {
    const out = new Map<string, { address: string; xyz_pc: readonly number[]; fuel: string[]; odds?: number }>();
    if (!plan) for (const s of cell?.systems ?? []) out.set(s.address, { address: s.address, xyz_pc: s.xyz_pc, fuel: knownFuel[s.address] ?? [] });
    for (const s of plan?.systems ?? []) out.set(s.address, { address: s.address, xyz_pc: s.xyz_pc, fuel: s.fuel, ...(s.fuel_odds !== undefined ? { odds: s.fuel_odds } : {}) });
    return [...out.values()];
  });
  const W = 560;
  const H = 400;
  const frame = $derived(voyageFrame(W, H, stars.map((s) => s.xyz_pc)));
  const posOf = (a: string) => stars.find((s) => s.address === a)?.xyz_pc;
  const legs = $derived(
    (v?.legs ?? []).flatMap((l) => {
      const a = posOf(l.from);
      const b = posOf(l.to);
      return a && b ? [{ l, a: frame.at(a), b: frame.at(b) }] : [];
    }),
  );
  const trail = $derived(visited.map(posOf).filter((p): p is readonly number[] => !!p).map((p) => frame.at(p)));
  const pickedStar = $derived(stars.find((s) => s.address === picked) ?? null);
  const shipInfo = $derived(VOYAGE_SHIPS.find((s) => s.id === ship));
  const ages = $derived(v ? agesWords(v) : null);

  function pick(a: string) {
    picked = a;
    if (mode === "to" && a !== from) to = a;
  }
  function nextStop() {
    if (!v) return;
    const stops = v.legs.map((l) => l.to);
    visited = [...visited, ...stops.filter((s) => !visited.includes(s))];
    // A refuel stop the plan used is now known for certain.
    from = v.to;
    if (mode === "to") to = "";
  }
  function toggleFuel(a: string) {
    const k = { ...knownFuel };
    if (k[a]) delete k[a];
    else k[a] = ["skim"];
    knownFuel = k;
  }
  function restart() {
    from = home;
    visited = [home];
    to = "";
  }
  const name = (a: string) => systemShort(a);
</script>

<div class="voy" data-voyage={v ? `${v.from}>${v.to}` : ""} data-legs={v?.legs.length ?? 0}>
  <div class="bar">
    <label class="lbl">Ship
      <select data-ship bind:value={ship}>
        {#each VOYAGE_SHIPS as s (s.id)}<option value={s.id}>{s.name}</option>{/each}
      </select>
    </label>
    <span class="seg" role="group" aria-label="Goal">
      <button type="button" data-mode="explore" aria-pressed={mode === "explore"} onclick={() => (mode = "explore")}>Nearest unvisited</button>
      <button type="button" data-mode="to" aria-pressed={mode === "to"} onclick={() => (mode = "to")}>To a star</button>
    </span>
    <label class="lbl">Keep smallest
      <select data-minimise bind:value={minimise}>
        {#each MINIMISE_WORDS as m (m.id)}<option value={m.id}>{m.label}</option>{/each}
      </select>
    </label>
    <label class="tick"><input type="checkbox" data-gambles bind:checked={gambles} /> Allow informed gambles for fuel</label>
  </div>

  <div class="head">
    <h2>Between stars</h2>
    {#if shipInfo}<span class="muted">{shipInfo.name}: {shipInfo.drive}</span>{/if}
    <p class="lede">
      {#if mode === "to" && !to}Pick a star on the map to plan a voyage to it.{:else}The ship's computer charts the legs from what the crew knows. The crew asks; the computer plans.{/if}
    </p>
  </div>

  <div class="body">
    <svg class="map" viewBox="0 0 {W} {H}" role="img" aria-label="Star systems seen from above, with the voyage's legs">
      {#if trail.length > 1}<polyline class="trail" points={trail.map((p) => `${p.x},${p.y}`).join(" ")} />{/if}
      {#each legs as g, i (i)}
        <line class="leg" class:jump={g.l.means === "jump"} class:gamble={g.l.gamble} x1={g.a.x} y1={g.a.y} x2={g.b.x} y2={g.b.y} />
      {/each}
      {#each stars as s (s.address)}
        {@const p = frame.at(s.xyz_pc)}
        <g class="star" class:home={s.address === home} class:here={s.address === from} class:dest={v && s.address === v.to} class:seen={visited.includes(s.address)} class:picked={s.address === picked}>
          {#if s.fuel.length}<circle class="fuel" cx={p.x} cy={p.y} r="6" />{/if}
          <circle class="dot" cx={p.x} cy={p.y} r={s.address === from || (v && s.address === v.to) ? 4 : 2.4} />
          <circle class="hit" role="button" tabindex="-1" aria-label={name(s.address)} data-star={s.address} cx={p.x} cy={p.y} r="8" onclick={() => pick(s.address)} onkeydown={(e) => e.key === "Enter" && pick(s.address)} />
        </g>
      {/each}
      <g class="scale">
        <line x1="14" y1={H - 14} x2={14 + frame.scale} y2={H - 14} />
        <text x="14" y={H - 20}>1 pc</text>
      </g>
    </svg>

    <div class="ledger" data-voyage-ledger>
      {#if failed}<p class="warn" data-failed>{failed}</p>{/if}
      {#if busy && !plan}<p class="muted">Charting…</p>{/if}
      {#if plan && !v}
        <h3>No voyage</h3>
        {#each plan.refusals as r, i (i)}<p class="warn" data-refusal>{r}</p>{/each}
      {/if}
      {#if v}
        <h3>{name(v.from)} to {name(v.to)}</h3>
        <div class="stamps">
          {#if v.fictional}<span class="stamp" title="Jumps follow the pack's rule, not physics">Fictional: pack rule</span>{/if}
          {#if v.gambles}<span class="stamp risk" data-risk>{v.gambles} gamble{v.gambles === 1 ? "" : "s"}, {Math.round(v.odds * 100)}% all pay off</span>{/if}
          {#if v.needs}<span class="stamp quiet">needs {v.needs}</span>{/if}
        </div>
        <dl>
          <dt>Distance</dt><dd>{v.distance_pc.toFixed(2)} pc in {v.legs.length} leg{v.legs.length === 1 ? "" : "s"}</dd>
          <dt>Time at home</dt><dd data-home-time>{voyageSpan(v.home_days)}</dd>
          <dt>Time aboard</dt><dd data-ship-time>{voyageSpan(v.ship_days)}</dd>
          {#if v.jump_fuel_t > 0}<dt>Jump fuel</dt><dd>{v.jump_fuel_t.toFixed(0)} t</dd>{/if}
          {#if v.propellant_t > 0}<dt>Propellant</dt><dd>{v.propellant_t.toFixed(0)} t</dd>{/if}
          <dt>Refuel stops</dt><dd>{v.refuels}</dd>
        </dl>
        {#if ages}<p class="ages" data-ages>{ages}</p>{/if}
        <h4>Legs</h4>
        <ol class="legs">
          {#each v.legs as l, i (i)}
            <li data-leg={l.means}>
              <span>{l.words}.</span>
              <span class="muted"> Left on arrival: {l.means === "jump" ? `${l.jump_fuel_left_t.toFixed(0)} t of jump fuel` : `${l.propellant_left_t.toFixed(0)} t of propellant`}.</span>
            </li>
          {/each}
        </ol>
        <button type="button" class="go" data-next onclick={nextStop}>{mode === "explore" ? "Next stop: go there" : "Arrive there"}</button>
        {#if visited.length > 1}<button type="button" class="quiet-btn" data-restart onclick={restart}>Back to the start</button>{/if}
      {/if}
      {#if pickedStar}
        <h4>Picked</h4>
        <p data-picked>{name(pickedStar.address)}{#if pickedStar.odds !== undefined && !pickedStar.fuel.length}: {Math.round(pickedStar.odds * 100)}% odds of a gas giant from its star{/if}.</p>
        <button type="button" class="quiet-btn" data-fuel-toggle onclick={() => toggleFuel(pickedStar.address)}>{knownFuel[pickedStar.address] ? "Forget the fuel here" : "The crew knows of fuel here"}</button>
      {/if}
      {#if plan}<p class="muted small">Searched {plan.searched.toLocaleString("en-GB")} systems in {plan.cells} star cells. Visited so far: {visited.length}.</p>{/if}
    </div>
  </div>
</div>

<style>
  .voy {
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
  .seg button[aria-pressed="true"] {
    background: var(--sse-ink);
    color: var(--sse-ground);
  }
  .go {
    margin: 10px 8px 0 0;
    background: var(--sse-ink);
    color: var(--sse-ground);
  }
  .quiet-btn {
    margin-top: 10px;
    border-color: var(--sse-rule);
  }
  .tick {
    display: inline-flex;
    gap: 6px;
    align-items: center;
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
  .small {
    font-size: 12px;
    margin-top: 12px;
  }
  .warn {
    color: var(--sse-accent-ink, var(--sse-accent));
    margin: 4px 0;
  }
  .body {
    display: grid;
    grid-template-columns: minmax(0, 1.3fr) minmax(0, 1fr);
    gap: 16px;
    align-items: start;
  }
  @media (max-width: 760px) {
    .body {
      grid-template-columns: minmax(0, 1fr);
    }
  }
  .map {
    width: 100%;
    height: auto;
    border: 1px solid var(--sse-rule);
    background: var(--sse-ground);
    animation: voy-in 0.24s ease-out;
  }
  @keyframes voy-in {
    from {
      opacity: 0;
    }
  }
  .star .dot {
    fill: var(--sse-ink-muted);
    transition: r 0.18s ease-out;
  }
  .star.seen .dot {
    fill: var(--sse-ink);
  }
  .star.here .dot,
  .star.dest .dot {
    fill: var(--sse-accent);
  }
  .star.picked .dot {
    stroke: var(--sse-accent);
    stroke-width: 1.5;
  }
  .star .fuel {
    fill: none;
    stroke: var(--sse-ink);
    stroke-width: 1;
  }
  .star.home .fuel {
    stroke-width: 2;
  }
  .hit {
    fill: transparent;
    cursor: pointer;
  }
  .leg {
    stroke: var(--sse-accent);
    stroke-width: 2;
  }
  .leg.jump {
    stroke-dasharray: 5 4;
  }
  .leg.gamble {
    stroke-dasharray: 2 3;
  }
  .trail {
    fill: none;
    stroke: var(--sse-ink-muted);
    stroke-width: 1;
    opacity: 0.6;
  }
  .scale line {
    stroke: var(--sse-ink);
  }
  .scale text {
    fill: var(--sse-ink-muted);
    font-size: 10px;
  }
  .stamps {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    margin: 4px 0 8px;
  }
  .stamp {
    border: 1.5px solid var(--sse-accent);
    color: var(--sse-accent-ink, var(--sse-accent));
    font-size: 10px;
    letter-spacing: 0.14em;
    text-transform: uppercase;
    padding: 2px 8px;
  }
  .stamp.quiet {
    border-color: var(--sse-rule);
    color: var(--sse-ink-muted);
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
  .ages {
    font-style: italic;
    margin: 8px 0 0;
  }
  .legs {
    margin: 0;
    padding-left: 18px;
  }
  .legs li {
    margin: 0 0 6px;
  }
</style>

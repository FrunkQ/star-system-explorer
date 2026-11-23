<!--
  Route, the astrogation console (V8). The crew tells the ship's computer
  where to go and what matters most; the kernel's plan_route plots courses
  from the real orbits. Candidates side by side (time, fuel left, peak g,
  dose, difficulty, hazards), then the chosen course in the star's frame with
  its burns and hazards on the track, and the numbers that matter at the
  table. Nobody hand-pilots: the crew chooses, the computer flies.
  Presentation only: the kernel does every sum; this file lays them out.
-->
<script lang="ts">
  import { untrack } from "svelte";
  import { STOCK_SHIPS, shipChoices, shipToSend, aerobrakeOption, chanceWords, constant, verdictWords, worstDangers, type WorldCheck, daysWords, delayWords, difficultyPips, doseWords, dvWords, fromNowWords, fuelLeft, navFromScene, orbitEllipse, routeFrame, safestFirst, shortlist, trackIndexAt, trackPath, trackPoint, type Course, type Json, type Prefer, type RoutePlan, type SystemScene, type ViewContext, YEAR_S } from "../../../ui-contract/src/index.ts";

  const YEAR_S_NUM = Number(YEAR_S);
  let { ctx, scene, params }: { ctx: ViewContext; scene: SystemScene; params: Record<string, Json> } = $props();

  type Want = Prefer | "safest";
  const WANTS: { id: Want; label: string }[] = [
    { id: "fastest", label: "Get there fast" },
    { id: "save-fuel", label: "Save fuel" },
    { id: "balanced", label: "Balance the two" },
    { id: "safest", label: "Keep us safe" },
  ];
  const WINDOWS = [7, 30, 120, 365, 1000];
  const first = untrack(() => ({ ship: String(params.ship ?? "fusion-torch"), prefer: String(params.prefer ?? "balanced"), window: Number(params.window_days ?? 120), shown: Number(params.shown ?? 3) }));

  const from_scene = $derived(navFromScene(scene));
  const bodies = $derived(from_scene.bodies);
  let ship = $state(first.ship);
  // Ships the stacked packs list (the Traveller pack's standard ships) beside the stock ones; a pack ship goes whole.
  const choices = $derived.by(() => {
    void ctx.workspace.revision;
    const st = ctx.session.stack();
    return shipChoices(STOCK_SHIPS, [...st.base, ...(st.campaign ? [st.campaign] : [])] as unknown as Parameters<typeof shipChoices>[1]);
  });
  $effect(() => {
    if (!choices.some((c) => c.id === ship)) ship = "fusion-torch";
  });
  const shipName = $derived(choices.find((c) => c.id === ship) ?? choices.find((c) => c.id === "fusion-torch") ?? choices[0]);
  const sending = $derived(shipName ? shipToSend(shipName) : ship);
  let want = $state<Want>((WANTS.some((w) => w.id === first.prefer) ? first.prefer : "balanced") as Want);
  let windowDays = $state(WINDOWS.includes(first.window) ? first.window : 120);
  let fromAddr = $state<string>("");
  let toAddr = $state<string>("");

  // Defaults: from the first world in the temperate zone (where people live, often), to the next one out.
  $effect(() => {
    const bs = bodies;
    if (!bs.length) return;
    untrack(() => {
      if (!bs.some((b) => b.address === fromAddr)) {
        const zone = scene.subjects.find((s) => s.kind === "zone" && s.band_au);
        const inZone = zone ? bs.findIndex((b) => {
          const a = scene.subjects.find((s) => s.address === b.address)?.orbit?.a_au ?? 0;
          return a >= zone.band_au![0] && a <= zone.band_au![1];
        }) : -1;
        fromAddr = bs[inZone >= 0 ? inZone : 0]!.address;
      }
      if (!bs.some((b) => b.address === toAddr) || toAddr === fromAddr) {
        const i = bs.findIndex((b) => b.address === fromAddr);
        toAddr = bs[i + 1 < bs.length ? i + 1 : Math.max(0, i - 1)]!.address;
      }
    });
  });
  // A planet picked on the chart becomes the destination.
  $effect(() => {
    const sel = ctx.workspace.selection;
    if (sel && sel !== fromAddr && bodies.some((b) => b.address === sel)) untrack(() => (toAddr = sel));
  });

  let plan = $state<RoutePlan | null>(null);
  let failed = $state<string | null>(null);
  let busy = $state(false);
  let picked = $state<string | null>(null);
  let gen = 0;
  $effect(() => {
    const nav = from_scene.nav;
    // Leave from the campaign's "now" when the shell keeps one (players follow it), else the chart's.
    const campaignNow = (ctx.workspace as { campaign_now_yr?: number }).campaign_now_yr;
    const depart = typeof campaignNow === "number" && Number.isFinite(campaignNow) ? { depart_s: campaignNow * YEAR_S_NUM } : {};
    const ask = { ship: sending, from: fromAddr, to: toAddr, prefer: (want === "safest" ? "balanced" : want) as Prefer, window_days: windowDays, ...depart };
    if (!nav || !ask.from || !ask.to || ask.from === ask.to) return;
    const g = ++gen;
    busy = true;
    failed = null;
    ctx.session.planRoute({ nav, ...ask }).then((r) => {
      if (g !== gen) return;
      busy = false;
      if (!r.ok) return void ((failed = r.message), (plan = null));
      plan = r.plan;
      picked = null;
    });
  });

  const ordered = $derived(plan ? (want === "safest" ? safestFirst(plan.candidates) : plan.candidates) : []);
  const shown = $derived(shortlist(ordered, Math.max(1, Math.min(6, first.shown))));
  const best = $derived(shown[0] ?? null);
  const chosen = $derived(shown.find((c) => c.id === picked) ?? best);
  const nameOf = (addr: string) => bodies.find((b) => b.address === addr)?.label ?? addr;
  const now_s = $derived.by(() => {
    const c = (ctx.workspace as { campaign_now_yr?: number }).campaign_now_yr;
    return typeof c === "number" && Number.isFinite(c) ? c * YEAR_S_NUM : (from_scene.nav?.t0_s ?? 0);
  });

  // The risky option beside the safe one (Alex, 23 Nov: danger is drama, never forbidden): where the destination has
  // air, braking in it instead of the arrival burn, judged by the kernel's ship_at_world at this course's own speed.
  let aero = $state<{ course: string; check: WorldCheck; saves_m_s: number } | null>(null);
  let agen = 0;
  $effect(() => {
    const c = chosen;
    const nav = from_scene.nav;
    const arr = c?.burns.find((b) => b.what === "arrival burn");
    aero = null;
    if (!c || !nav || !arr || !(c.arrive_v_inf_m_s! > 0)) return;
    const g = ++agen;
    // An adopted system on the GM's seat answers from its own record (its air); a chart's inline system carries none.
    const where = !scene.fixture && ctx.session.seat.authority === "gm" ? { system: scene.system } : { nav };
    ctx.session.shipAtWorld({ ...where, ship: sending, worlds: [toAddr], v_inf_m_s: c.arrive_v_inf_m_s! }).then((r) => {
      if (g !== agen || !r.ok) return;
      const o = aerobrakeOption(c.burns, r.report.worlds[0]);
      if (o) aero = { course: c.id, ...o };
    });
  });

  function tag(c: Course, i: number): string {
    if (i === 0) return want === "safest" ? "safest" : want === "fastest" ? "fastest" : want === "save-fuel" ? "cheapest" : "best match";
    if (plan && c === safestFirst(plan.candidates)[0]) return "safest";
    if (plan && c === [...plan.candidates].sort((a, b) => a.days - b.days)[0]) return "fastest";
    if (plan && c === [...plan.candidates].sort((a, b) => a.dv_m_s - b.dv_m_s)[0]) return "cheapest";
    return "";
  }

  // Mini plot per card and the big chart: the star's frame, sized to the course.
  const MW = 300;
  const MH = 70;
  const BW = 640;
  const BH = 420;
  const bigFrame = $derived(routeFrame(BW, BH, from_scene.nav, plan, chosen));
  const ends = $derived(chosen ? { a: trackPoint(chosen.track[0]!, bigFrame), b: trackPoint(chosen.track.at(-1)!, bigFrame) } : null);
  const burnMarks = $derived(chosen ? chosen.burns.map((b) => ({ ...trackPoint(chosen.track[trackIndexAt(chosen, b.at_s)]!, bigFrame), b })) : []);
  const hazardMarks = $derived(chosen ? chosen.hazards.filter((h) => h.at_s !== undefined).map((h) => ({ ...trackPoint(chosen.track[trackIndexAt(chosen, h.at_s!)]!, bigFrame), h })) : []);
  const fuel = $derived(chosen && plan ? fuelLeft(chosen, plan.ship) : null);
  const AU_M = constant("au");
</script>

<div class="route" data-route={plan ? `${plan.from}>${plan.to}` : ""}>
  <aside class="ask" aria-label="Tell the computer">
    <h2>Astrogation</h2>
    <p class="hint">The ship's computer plots courses from the real orbits. The crew chooses; the computer flies.</p>
    <label>Ship
      <select bind:value={ship} data-ship>
        {#each choices as s (s.id)}<option value={s.id} data-pack={s.pack ?? null}>{s.name}{s.pack ? " (pack)" : ""}</option>{/each}
      </select>
    </label>
    {#if shipName}<p class="hint">{shipName.drive}</p>{/if}
    <label>From
      <select bind:value={fromAddr} data-from>
        {#each bodies as b (b.address)}<option value={b.address}>{b.label}</option>{/each}
      </select>
    </label>
    <label>To
      <select bind:value={toAddr} data-to>
        {#each bodies as b (b.address)}{#if b.address !== fromAddr}<option value={b.address}>{b.label}</option>{/if}{/each}
      </select>
    </label>
    <label>Leave within
      <select bind:value={windowDays}>
        {#each WINDOWS as w (w)}<option value={w}>{daysWords(w)}</option>{/each}
      </select>
    </label>
    <fieldset>
      <legend>What matters most?</legend>
      <div class="chips">
        {#each WANTS as w (w.id)}<button type="button" class:on={want === w.id} aria-pressed={want === w.id} onclick={() => (want = w.id)} data-want={w.id}>{w.label}</button>{/each}
      </div>
    </fieldset>
    {#if plan}
      <div class="knows">
        <span class="cap">What the ship has</span>
        <div class="row"><span>Δv aboard</span><span class="v">{dvWords(plan.dv_budget_m_s)}</span></div>
        <div class="row"><span>Usable, keeping the reserve</span><span class="v">{dvWords(plan.usable_dv_m_s)}</span></div>
        <div class="row"><span>Top acceleration</span><span class="v">{plan.ship.accel_g} g</span></div>
      </div>
    {/if}
    {#each from_scene.notes as n (n)}<p class="hint">{n}</p>{/each}
  </aside>

  <main class="answer">
    {#if !from_scene.nav}
      <p class="note">{from_scene.notes[0]}</p>
    {:else if failed}
      <p class="note" role="alert">The computer could not plot this: {failed}</p>
    {:else if plan}
      <header class="head">
        <h2>{shown.length === 1 ? "One way" : `${["", "One way", "Two ways", "Three ways", "Four ways", "Five ways", "Six ways"][shown.length]}`} to {nameOf(toAddr)}</h2>
        {#if busy}<span class="hint">plotting…</span>{/if}
        <span class="hint">{plan.candidates.length} course{plan.candidates.length === 1 ? "" : "s"} found{plan.refused.length ? `, ${plan.refused.length} beyond this ship` : ""}</span>
      </header>
      {#if !shown.length}
        <p class="note" data-none>No course this ship can fly. {plan.refused.map((r) => r.why).join(" ")}</p>
      {/if}
      <div class="split">
      {#if chosen}
        <section class="detail" aria-label="The chosen course" data-chosen={chosen.id}>
          <svg viewBox="0 0 {BW} {BH}" class="big" role="img" aria-label="{chosen.label} from {nameOf(plan.from)} to {nameOf(plan.to)}, drawn in {scene.title}'s frame">
            {#each from_scene.nav.belts as bt (bt.name)}
              {@const r0 = bt.a_in_au * AU_M * bigFrame.k}
              {@const r1 = bt.a_out_au * AU_M * bigFrame.k}
              {#if r0 < BW}<circle class="belt" cx={bigFrame.cx} cy={bigFrame.cy} r={(r0 + r1) / 2} stroke-width={Math.max(2, r1 - r0)}><title>{bt.name}</title></circle>{/if}
            {/each}
            {#each from_scene.nav.bodies as b (b.address)}
              {@const o = orbitEllipse(b, bigFrame)}
              {#if o}<ellipse class="orbit" class:end={b.address === plan.from || b.address === plan.to} cx={o.cx} cy={o.cy} rx={o.rx} ry={o.ry} transform="rotate({o.rot} {o.cx} {o.cy})" />{#if o.rx > 40 && b.address !== plan.from}<text class="olabel" x={o.cx + o.rx * 0.71 + 4} y={o.cy - o.ry * 0.71 - 4}>{b.name}</text>{/if}{/if}
            {/each}
            <circle class="star" cx={bigFrame.cx} cy={bigFrame.cy} r="6" />
            <path class="track big" d={trackPath(chosen.track, bigFrame)} />
            {#each burnMarks as m, j (j)}<circle class="burn" cx={m.x} cy={m.y} r="5"><title>{m.b.what}: {dvWords(m.b.dv_m_s)}</title></circle>{/each}
            {#each hazardMarks as m, j (j)}<circle class="hazmark" cx={m.x} cy={m.y} r="9"><title>{m.h.text}</title></circle>{/each}
            {#if ends}
              <circle class="endpt" cx={ends.a.x} cy={ends.a.y} r="6" /><text class="elabel" x={ends.a.x + 9} y={ends.a.y + 4}>{nameOf(plan.from)} at departure</text>
              <circle class="endpt" cx={ends.b.x} cy={ends.b.y} r="7" /><text class="elabel" x={ends.b.x + 10} y={ends.b.y + 4}>{nameOf(plan.to)} at arrival</text>
            {/if}
          </svg>
          <div class="strip">
            <div><span class="cap">Depart, arrive</span><span class="v">{fromNowWords(chosen.depart_s, now_s)}</span><span class="v">{fromNowWords(chosen.arrive_s, now_s)}</span></div>
            {#if fuel}
              <div class="fuel"><span class="cap">Fuel after arrival</span><span class="v" class:warn={fuel.under}>{Math.round(fuel.left * 100)}%</span>
                <span class="gauge" role="img" aria-label="{Math.round(fuel.left * 100)}% left; reserve {Math.round(fuel.reserve * 100)}%"><span class="gfill" class:warn={fuel.under} style="width: {fuel.left * 100}%"></span><span class="gres" style="left: {fuel.reserve * 100}%"></span></span>
                {#if fuel.under}<span class="hint warn">Below the {Math.round(fuel.reserve * 100)}% reserve.</span>{/if}
              </div>
            {/if}
            <div><span class="cap">Δv</span><span class="v">{dvWords(chosen.dv_m_s)}</span></div>
            <div><span class="cap">Peak</span><span class="v">{chosen.peak_g.toFixed(2)} g</span></div>
            <div><span class="cap">Dose</span><span class="v">{doseWords(chosen.dose_msv)}</span></div>
            <div><span class="cap">Light delay home</span><span class="v">{delayWords(chosen.delay_home_s)}</span></div>
            <div><span class="cap">Closest to {scene.subjects.find((s) => s.kind === "star" && !s.orbit)?.label ?? "the star"}</span><span class="v">{chosen.closest_star_au.toFixed(2)} AU</span></div>
          </div>
          <div class="cols">
            <div>
              <h4>Burns</h4>
              <ol class="burns">{#each chosen.burns as b, j (j)}<li><span class="v">{fromNowWords(chosen.depart_s + b.at_s, now_s)}</span> {b.what}: {dvWords(b.dv_m_s)}{b.duration_s > 3600 ? `, lasting ${daysWords(b.duration_s / constant("day"))}` : ""}</li>{/each}</ol>
              {#if aero && aero.course === chosen.id}
                {@const ds = worstDangers(aero.check)}
                <div class="aero" data-aerobrake={aero.check.verdict}>
                  <h4>Or brake in the air</h4>
                  <p>Skip the {dvWords(aero.saves_m_s)} arrival burn by aerocapture at {nameOf(toAddr)}: <strong data-level={aero.check.verdict}>{verdictWords(aero.check.verdict)}</strong>.</p>
                  {#each ds as d (d.id)}<p class="hint">{d.why}. If it goes wrong ({chanceWords(d.chance.p)}): {d.if_wrong}.</p>{/each}
                  {#if aero.check.would_pass?.length}<p class="hint">Safer with {aero.check.would_pass.join("; or ")}.</p>{/if}
                  <p class="hint">Offered, never forbidden: the GM weighs it.</p>
                </div>
              {/if}
              {#if chosen.difficulty.reasons.length}<h4>Why it is {chosen.difficulty.label}</h4><ul>{#each chosen.difficulty.reasons as r (r)}<li>{r}</li>{/each}</ul>{/if}
            </div>
            <div>
              <h4>How the computer worked it out</h4>
              <ul>{#each chosen.why as w, j (j)}<li>{w}</li>{/each}</ul>
              {#if plan.refused.length}<h4>Beyond this ship</h4><ul>{#each plan.refused as r, j (j)}<li>{r.why}</li>{/each}</ul>{/if}
              <p class="hint">Model: {plan.model.join("; ")}.</p>
            </div>
          </div>
        </section>
      {/if}
      <div class="cards">
        {#each shown as c, i (c.id)}
          {@const f = routeFrame(MW, MH, from_scene.nav, plan, c, 8)}
          {@const fl = fuelLeft(c, plan.ship)}
          <article class="card" class:chosen={chosen === c} data-course={c.id}>
            <div class="ctop"><h3>{c.label}</h3>{#if tag(c, i)}<span class="tag">{tag(c, i)}</span>{/if}</div>
            <svg viewBox="0 0 {MW} {MH}" aria-hidden="true" class="mini">
              {#each from_scene.nav.bodies as b (b.address)}
                {@const o = orbitEllipse(b, f)}
                {#if o}<ellipse class="orbit" class:end={b.address === plan.from || b.address === plan.to} cx={o.cx} cy={o.cy} rx={o.rx} ry={o.ry} transform="rotate({o.rot} {o.cx} {o.cy})" />{/if}
              {/each}
              <circle class="star" cx={f.cx} cy={f.cy} r="3" />
              <path class="track" d={trackPath(c.track, f)} />
            </svg>
            <div class="nums">
              <span><b>{daysWords(c.days)}</b></span>
              <span class:warn={fl.under}><b>{Math.round(fl.left * 100)}%</b> fuel left</span>
              <span><b>{c.peak_g < 0.01 ? c.peak_g.toFixed(3) : c.peak_g.toFixed(2)} g</b> peak</span>
            </div>
            <div class="diff"><span class="cap">Difficulty</span><span class="pips" aria-hidden="true">{#each [1, 2, 3, 4] as p (p)}<span class="pip" class:lit={difficultyPips(c.difficulty.label) >= p}></span>{/each}</span><span>{c.difficulty.label}</span></div>
            <ul>
              <li>Dose {doseWords(c.dose_msv)}{c.spe_chance > 0.01 ? `, ${Math.round(c.spe_chance * 100)}% chance of a solar storm` : ""}</li>
              {#each c.hazards.slice(0, 3) as h, j (j)}<li class="haz">{h.text}</li>{/each}
              {#if c.hazards.length > 3}<li class="hint">and {c.hazards.length - 3} more</li>{/if}
            </ul>
            <button type="button" class="pick" aria-pressed={chosen === c} onclick={() => (picked = c.id)}>{chosen === c ? "On the chart" : "Show on the chart"}</button>
          </article>
        {/each}
      </div>

      </div>
    {:else}
      <p class="note">Plotting…</p>
    {/if}
  </main>
</div>

<style>
  .aero {
    border-left: 2px solid var(--sse-accent);
    padding-left: 8px;
    margin: 8px 0;
  }
  .aero strong[data-level="dangerous"] {
    color: var(--sse-alert, var(--sse-accent));
  }
  .route {
    color: var(--sse-ink);
    font-size: 13px;
    display: grid;
    grid-template-columns: minmax(220px, 280px) minmax(0, 1fr);
    gap: 20px;
    align-items: start;
  }
  @media (max-width: 760px) {
    .route {
      grid-template-columns: minmax(0, 1fr);
    }
  }
  .ask {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  label {
    display: flex;
    flex-direction: column;
    gap: 4px;
    font-size: 12px;
    color: var(--sse-ink-muted);
  }
  select {
    font: inherit;
    font-size: 14px;
    min-height: 36px;
    background: var(--sse-ground);
    color: var(--sse-ink);
    border: 1px solid var(--sse-rule);
  }
  fieldset {
    border: 0;
    padding: 0;
    margin: 0;
  }
  legend {
    font-size: 12px;
    color: var(--sse-ink-muted);
    margin-bottom: 6px;
  }
  .chips {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
  }
  .chips button,
  .pick {
    font: inherit;
    min-height: 36px;
    padding: 0 12px;
    border: 1px solid var(--sse-ink);
    background: transparent;
    color: var(--sse-ink);
    cursor: pointer;
  }
  .chips button.on,
  .pick[aria-pressed="true"] {
    background: var(--sse-ink);
    color: var(--sse-ground);
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
    margin: 0;
  }
  h4,
  .cap {
    font-size: 11px;
    letter-spacing: 0.12em;
    text-transform: uppercase;
    color: var(--sse-ink-muted);
    font-weight: 500;
  }
  h4 {
    margin: 10px 0 4px;
  }
  .hint,
  .note {
    color: var(--sse-ink-muted);
    margin: 0;
  }
  .hint {
    font-size: 12px;
  }
  .knows {
    border: 1px solid var(--sse-rule);
    padding: 8px 10px;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  .row {
    display: flex;
    justify-content: space-between;
    gap: 8px;
  }
  .v {
    font-family: var(--sse-font-mono, monospace);
  }
  .warn {
    color: var(--sse-accent-ink);
  }
  .answer {
    display: flex;
    flex-direction: column;
    gap: 14px;
    min-width: 0;
  }
  .head {
    display: flex;
    flex-wrap: wrap;
    align-items: baseline;
    gap: 12px;
  }
  .split {
    display: grid;
    grid-template-columns: minmax(0, 1.5fr) minmax(260px, 1fr);
    gap: 16px;
    align-items: start;
  }
  @media (max-width: 1100px) {
    .split {
      grid-template-columns: minmax(0, 1fr);
    }
  }
  .cards {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }
  .card {
    background: var(--sse-panel-ground, var(--sse-ground));
    border: var(--sse-frame-width, 1px) solid var(--sse-frame-line, var(--sse-ink));
    box-shadow: var(--sse-frame-shadow, none);
    border-radius: var(--sse-frame-radius, 0);
    padding: 12px 14px;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .card.chosen {
    outline: 2px solid var(--sse-accent);
    outline-offset: 2px;
  }
  .ctop {
    display: flex;
    justify-content: space-between;
    align-items: baseline;
    gap: 8px;
  }
  .tag {
    font-size: 10px;
    letter-spacing: 0.12em;
    text-transform: uppercase;
    border: 1.5px solid var(--sse-accent);
    color: var(--sse-accent-ink);
    padding: 1px 6px;
    white-space: nowrap;
  }
  .mini {
    width: 100%;
    height: auto;
  }
  .nums {
    display: flex;
    flex-wrap: wrap;
    gap: 4px 12px;
    font-family: var(--sse-font-mono, monospace);
  }
  .diff {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .pips {
    display: inline-flex;
    gap: 3px;
  }
  .pip {
    width: 16px;
    height: 7px;
    border: 1px solid var(--sse-ink);
  }
  .pip.lit {
    background: var(--sse-ink);
  }
  ul,
  ol {
    margin: 0;
    padding-left: 18px;
    display: flex;
    flex-direction: column;
    gap: 3px;
    line-height: 1.4;
  }
  .haz {
    color: var(--sse-accent-ink);
  }
  svg .orbit {
    fill: none;
    stroke: var(--sse-line, var(--sse-rule));
    stroke-width: 1;
  }
  svg .orbit.end {
    stroke: var(--sse-ink);
  }
  svg .star {
    fill: var(--sse-ink);
  }
  svg .belt {
    fill: none;
    stroke: var(--sse-ink-muted);
    stroke-opacity: 0.35;
    stroke-dasharray: 1 4;
  }
  svg .track {
    fill: none;
    stroke: var(--sse-accent);
    stroke-width: 2;
  }
  svg .track.big {
    stroke-width: 2.5;
  }
  svg .burn {
    fill: var(--sse-ground);
    stroke: var(--sse-ink);
    stroke-width: 2;
  }
  svg .hazmark {
    fill: none;
    stroke: var(--sse-accent);
    stroke-dasharray: 3 2;
  }
  svg .endpt {
    fill: var(--sse-ink);
  }
  svg text {
    fill: var(--sse-ink);
    font-size: 12px;
  }
  svg .olabel {
    fill: var(--sse-ink-muted);
    font-family: var(--sse-title-font, serif);
    font-style: italic;
  }
  .detail {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }
  .big {
    width: 100%;
    max-height: 520px;
    background: var(--sse-chart-ground, transparent);
    border: 1px solid var(--sse-rule);
  }
  .strip {
    display: flex;
    flex-wrap: wrap;
    gap: 14px 26px;
    border: 1px solid var(--sse-rule);
    padding: 10px 14px;
  }
  .strip > div {
    display: flex;
    flex-direction: column;
    gap: 3px;
  }
  .fuel {
    min-width: 200px;
  }
  .gauge {
    display: block;
    width: 100%;
    position: relative;
    height: 9px;
    border: 1px solid var(--sse-ink);
  }
  .gauge .gfill {
    display: block;
    position: absolute;
    left: 0;
    top: 0;
    bottom: 0;
    background: var(--sse-ink);
  }
  .gauge .gfill.warn {
    background: var(--sse-accent);
  }
  .gauge .gres {
    display: block;
    position: absolute;
    top: -4px;
    bottom: -4px;
    width: 2px;
    background: var(--sse-accent);
  }
  .cols {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(260px, 1fr));
    gap: 16px;
  }
</style>

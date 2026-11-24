<!--
  Ship manifest and crew roster (track MC): who and what is aboard one ship, station or site. Every person and lot
  is a record actor with an `aboard.on` link (base manifest pack), so adding, editing and moving are kept values the
  ship's log already reads. `part` picks the panel: the manifest (what it can hold, its state, life support days and
  cargo) or the crew (people, roles, kind of being, awake or asleep, own age). Nothing names a game. GM only until
  the Known to eye can reveal entries to players.
-->
<script lang="ts">
  import { untrack } from "svelte";
  import { ABOARD, CARGO, PERSON, STATES, STATE_WORDS, aboard, ageNow, crewAboard, daysWords, entryFrom, entryId, lifeSupport, lifeSupportFrom, startingValues, secondsToYears, eyesShow, secrecyRelevant, tableOf, type AboardEntry, type AboardState, type IntentResult, type Json, type KnownChange, type KnownLandmark, type KnownTo as KnownRead, type PlayerSubject, type ViewContext } from "../../../ui-contract/src/index.ts";
  import { KnownTo } from "../../../chart-grammar/src/index.ts";
  import { fieldWords, valueWords } from "../../constructs/src/words.ts";

  let { ctx, construct, places, start = "crew" }: { ctx: ViewContext; construct: PlayerSubject; places: { address: string; label: string }[]; start?: "manifest" | "crew" } = $props();
  // One panel, two sides: who is aboard (Crew, first) and what (Manifest). The same for a ship, a station or a site.
  let part = $state<"manifest" | "crew">(untrack(() => start));
  const session = untrack(() => ctx.session);
  const ws = $derived(ctx.workspace);
  const depth = $derived(ws.depth ?? "science");
  const gm = session.seat.authority === "gm";

  let entries = $state<AboardEntry[]>([]);
  let recycled = $state(0);
  let readiness = $state<number | null>(null);
  let note = $state<string | null>(null);
  let now_s = $state(0);
  const here = $derived(aboard(entries, construct.address));
  const ls = $derived.by(() => {
    const st = session.stack();
    return lifeSupportFrom([...st.base, ...st.added, ...(st.campaign ? [st.campaign] : [])] as unknown as Json[]);
  });
  const support = $derived(lifeSupport(here.people, here.cargo, ls, recycled));
  const hasKinds = $derived(Object.keys(ls.rates).length > 0 || session.stack().base.some((p) => p.id === "sse.base-manifest"));

  const root = $derived(construct.address.split("/")[0]!);
  async function campaignAt(): Promise<number> {
    const now = await session.campaignNow();
    return now.ok && now.at_s !== undefined ? Number(now.at_s) : 0;
  }
  async function load() {
    if (!gm || !session.record) return;
    now_s = await campaignAt();
    const top = await session.record(root);
    if (!top.ok) return void (note = top.message);
    const mine = top.record.actors.filter((a) => a.kind === PERSON || a.kind === CARGO);
    const reads = await Promise.all(mine.map((a) => session.record!(a.address)));
    entries = mine.map((a, i) => entryFrom(a, reads[i]!.ok ? (reads[i] as { ok: true; record: import("../../../ui-contract/src/index.ts").RecordRead }).record : null));
    const own = await session.record(construct.address);
    if (own.ok) {
      const r = own.record.kept.find((k) => k.kind === ABOARD.recycled)?.value;
      recycled = typeof r === "number" ? r : 0;
      const rd = own.record.kept.find((k) => k.kind === "ship.readiness")?.value;
      readiness = typeof rd === "number" ? rd : null;
    }
  }
  $effect(() => {
    void ws.revision;
    void construct.address;
    untrack(() => void load());
  });

  // ---- Writing ----
  let busy = $state(false);
  async function keep(address: string, kind: string, value: Json | null, reason?: string): Promise<boolean> {
    const r = await session.setKept?.({ address, kind, value, ...(reason ? { reason } : {}), at_s: await campaignAt() });
    if (!r) return false;
    if (!r.ok) return (note = r.message), false;
    return true;
  }
  // ---- Known to (SV1, SV2): who at the table knows each person and lot aboard ----
  // Entries are record actors; the eye on each row reveals one, and "Show all to the table"
  // reveals everyone and everything on this side of the panel as one change that one Undo takes back. New entries are
  // as known as their place (Known to question 6).
  let known = $state.raw<Record<string, KnownRead>>({});
  $effect(() => {
    const list = [...here.people, ...here.cargo].map((e) => e.address);
    void ws.revision;
    if (!gm || (ws.desk !== "build" && ws.desk !== "run")) return void (known = {});
    let live = true;
    void Promise.all(list.map(async (a) => [a, await session.knownTo(a)] as const)).then((rs) => {
      if (live) known = Object.fromEntries(rs.flatMap(([a, r]) => (r.ok ? [[a, r.value]] : [])));
    });
    return () => (live = false);
  });
  // Who knows what is read at the campaign clock, so changes are made there too (the sky's time can run ahead).
  const knownAt = async () => secondsToYears(await campaignAt());
  const eyes = $derived(eyesShow(ws.desk, Object.values(known).some((k) => secrecyRelevant(k, !!ws.shared))));
  const landmarkOf = (a: string): KnownLandmark => tableOf(known[a])?.landmark ?? "known";
  const whyNot = (r: IntentResult) => (r.status === "refused" ? r.reason : r.status === "pending" ? "sent to the GM" : r.status === "error" ? r.message : "");
  const setKnownOf = (address: string) => async (c: KnownChange): Promise<{ ok: true; undo?: string } | { ok: false; why: string }> => {
    const r = await session.send({ kind: "set_known", address, ...c, at_yr: await knownAt() });
    if (r.status === "applied") return (ws.revision++, { ok: true, ...(r.event_id ? { undo: r.event_id } : {}) });
    return { ok: false, why: whyNot(r) };
  };
  async function undoKnown(undo: string): Promise<string | null> {
    const r = await session.send({ kind: "undo_known", undo, at_yr: await knownAt() });
    return r.status === "applied" ? (ws.revision++, null) : whyNot(r);
  }
  let lastShown = $state<string | null>(null);
  const hidden = $derived((part === "crew" ? here.people : here.cargo).filter((e) => landmarkOf(e.address) !== "known").map((e) => e.address));
  async function showAll() {
    if (!hidden.length) return;
    const r = await session.send({ kind: "set_known", address: hidden[0]!, audience: "everyone", landmark: "known", scope: { list: hidden }, at_yr: await knownAt() });
    if (r.status !== "applied") return void (note = whyNot(r));
    lastShown = r.event_id || null;
    void done(`${hidden.length === 1 ? "One entry is" : `${hidden.length} entries are`} now known to the table.`);
  }
  async function undoShowAll() {
    if (!lastShown) return;
    const why = await undoKnown(lastShown);
    lastShown = null;
    note = why ?? "Hidden again.";
  }

  async function done(msg: string | null) {
    note = msg;
    ws.revision++;
  }

  let adding = $state(false);
  let f = $state({ name: "", role: "", being: "", age: "" as string | number, state: "awake" as AboardState, what: "", amount: "" as string | number, unit: "t" });
  function openAdd() {
    adding = !adding;
    f = { name: "", role: "", being: "", age: "", state: "awake", what: "", amount: "", unit: "t" };
  }
  async function add() {
    const name = f.name.trim() || (part === "crew" ? "" : f.what.trim());
    if (!name || !session.addActor) return void (note = part === "crew" ? "A person needs a name." : "Say what it is.");
    busy = true;
    const kind = part === "crew" ? PERSON : CARGO;
    const at_s = await campaignAt();
    const id = entryId(name, entries.map((e) => e.address));
    const made = await session.addActor({ id, name, kind, at_s });
    if (!made.ok) {
      busy = false;
      return void (note = made.message);
    }
    // A number input binds a number, or "" when empty.
    const num = (v: string | number) => (typeof v === "number" ? v : v.trim() === "" ? undefined : Number(v));
    const vals = startingValues(kind, construct.address, { role: f.role, being: f.being, age_yr: num(f.age), state: f.state, what: f.what || name, amount: num(f.amount), unit: f.unit });
    for (const [k, v] of vals) if (!(await keep(made.address, k, v, k === ABOARD.on ? (kind === PERSON ? "came aboard" : "loaded") : undefined))) break;
    busy = false;
    adding = false;
    void done(`${name} ${kind === PERSON ? "is aboard" : "is loaded"}.`);
  }

  async function setState(e: AboardEntry, s: AboardState) {
    if (await keep(e.address, ABOARD.state, s)) void done(null);
  }
  async function setText(e: AboardEntry, kind: string, raw: string) {
    const v = raw.trim();
    if (await keep(e.address, kind, v ? v : null)) void done(null);
  }
  async function setNumber(address: string, kind: string, raw: string, why?: string) {
    const v = raw.trim() === "" ? null : Number(raw);
    if (v !== null && !Number.isFinite(v)) return void (note = `${raw} is not a number.`);
    if (await keep(address, kind, v, why)) void done(null);
  }
  // Move to another ship or place, or off (the place it is at). One dated change; the log keeps where it was.
  let moving = $state<string | null>(null);
  let moveTo = $state("");
  const parentOf = (a: string) => a.slice(0, a.lastIndexOf("/"));
  const targets = $derived(places.filter((p) => p.address !== construct.address));
  async function move(e: AboardEntry) {
    if (!moveTo) return;
    const to = places.find((p) => p.address === moveTo)?.label ?? moveTo;
    if (await keep(e.address, ABOARD.on, moveTo, `moved to ${to}`)) {
      moving = null;
      void done(`${e.name} moved to ${to}.`);
    }
  }

  // ---- Words ----
  const fact = (suffix: RegExp) => construct.facts.find((x) => suffix.test(x.field));
  const crewMax = $derived(fact(/(^|\.)crew_max$/));
  const berths = $derived(fact(/(^|\.)berths$/));
  const capacity = $derived(construct.facts.filter((x) => /(^|\.)(cargo|hold|tank|fuel)[a-z_]*$/.test(x.field) && typeof x.value === "number"));
  const counted = $derived(crewAboard(construct, entries) ?? 0);
  const amountWords = (e: AboardEntry) => (e.amount === undefined ? "amount not set" : `${valueWords(e.amount)}${e.unit ? ` ${e.unit}` : ""}`);
  const ageWords = (e: AboardEntry) => {
    const a = ageNow(e, now_s);
    return a === undefined ? "" : `${Math.floor(a)} yr`;
  };
  const sinceWords = (e: AboardEntry) => {
    const last = e.moves[e.moves.length - 1];
    return last ? `since ${Number(secondsToYears(last.at_s).toFixed(2))}` : "";
  };
</script>

<section class="aboard" aria-label="Aboard" data-aboard={part}>
  <div class="switch" role="group" aria-label="Aboard">
    {#each [["crew", "Crew"], ["manifest", "Manifest"]] as [k, label] (k)}
      <button type="button" aria-pressed={part === k} onclick={() => ((part = k as "crew" | "manifest"), (adding = false), (moving = null))} data-part={k}>{label}</button>
    {/each}
  </div>
  {#if eyes && (hidden.length || lastShown)}
    <div class="bar small" data-show-all-bar>
      {#if hidden.length}<button type="button" class="quiet-btn" onclick={showAll} data-show-all>Show {part === "crew" ? "the crew" : "the manifest"} to the table</button><span class="muted">{hidden.length} not known to them</span>{/if}
      {#if lastShown}<button type="button" class="quiet-btn" onclick={undoShowAll} data-show-all-undo>Undo</button>{/if}
    </div>
  {/if}
  {#if !hasKinds}
    <p class="small muted">The manifest pack is not on the stack, so nothing can be listed aboard.</p>
  {:else if part === "manifest"}
    <dl>
      <dt>People aboard</dt>
      <dd data-people-count>{counted}{crewMax ? ` of ${valueWords(crewMax.value)}` : ""}{#if berths}<span class="muted">{` · ${valueWords(berths.value)} berths`}</span>{/if}</dd>
      {#each capacity as c (c.field)}
        {@const w = fieldWords(c.field)}
        <dt>{w.label}</dt>
        <dd>{valueWords(c.value)}{#if w.unit}<span class="muted">{` ${w.unit}`}</span>{/if}<span class="muted small">{" (from its design)"}</span></dd>
      {/each}
      {#if readiness !== null}
        <dt>Readiness</dt>
        <dd>{Math.round(readiness * 100)}%{#if readiness < 1}<span class="muted"> · not fully ready</span>{/if}</dd>
      {/if}
      <dt>Recycled</dt>
      <dd>
        <!-- In play (Explore, Run) what is aboard changes; the ship's own fittings, like its recycling, only on Build. -->
        {#if gm && ws.mode === "edit"}
          <input type="number" min="0" max="100" step="1" value={Math.round(recycled * 100)} onchange={(e) => setNumber(construct.address, ABOARD.recycled, String(Number((e.currentTarget as HTMLInputElement).value) / 100), "life support changed")} aria-label="Share of oxygen and water recycled, in percent" data-recycled />%
        {:else}{Math.round(recycled * 100)}%{/if}
        <span class="muted small"> of oxygen and water</span>
      </dd>
    </dl>

    <h5>Life support</h5>
    {#if support.drawing === 0 && !support.unrated.length}
      <p class="small muted" data-life>Nobody aboard is drawing on the stores.</p>
    {:else}
      <p data-life>
        {#if support.first}
          {support.first.store === "oxygen" ? "Oxygen" : support.first.store === "water" ? "Water" : "Food"} runs out first, in <b data-first-days>{daysWords(support.first.days)}</b>, for {support.drawing} {support.drawing === 1 ? "person" : "people"} awake.
        {:else}
          No oxygen, water or food is listed aboard.
        {/if}
      </p>
      <ul class="stores">
        {#each support.stores as s (s.store)}
          <li data-store={s.store}><span>{s.store}</span> <span class="mono">{valueWords(Math.round(s.kg))} kg</span> <span class="muted">{s.days === null ? "not used" : daysWords(s.days)}</span>{#if depth !== "curious" && s.use_kg_day > 0}<span class="muted small"> · {Number(s.use_kg_day.toPrecision(3))} kg a day</span>{/if}</li>
        {/each}
      </ul>
      {#each support.unrated as u (u.being)}<p class="small muted" data-unrated>No life support figures for {u.being} ({u.count} aboard): they are left out of the days above.</p>{/each}
      {#if support.unweighed.length}<p class="small muted">Not weighed, so left out: {support.unweighed.join(", ")}. Give them a mass in kg or t.</p>{/if}
      {#if depth !== "curious"}<p class="small muted">Rates per person a day from the stack's packs ({Object.keys(ls.rates).join(", ") || "none"}); anyone whose kind of being is not stated is counted as human. People in cold sleep draw nothing.</p>{/if}
    {/if}

    <div class="bar"><h5>Cargo and stores</h5><span class="grow"></span>{#if gm}<button type="button" class="quiet-btn" aria-expanded={adding} onclick={openAdd} data-add-cargo>Load…</button>{/if}</div>
    {#if adding}
      <div class="form">
        <label>What <input type="text" bind:value={f.what} placeholder="water, food, oxygen, or any goods" data-what /></label>
        <label>Name <input type="text" bind:value={f.name} placeholder="the same as what it is" /></label>
        <label>Amount <input type="number" min="0" bind:value={f.amount} data-amount /></label>
        <label>Unit <input type="text" bind:value={f.unit} size="6" data-unit /></label>
        <button type="button" class="go" disabled={busy} onclick={add} data-load>Load it</button>
      </div>
    {/if}
    <ul class="list" data-cargo>
      {#each here.cargo as c (c.address)}
        <li class:pencil={eyes && landmarkOf(c.address) !== "known"} data-lot={c.address}>
          {#if eyes && known[c.address]}<KnownTo read={known[c.address]!} label={c.name} hold={(ws.players_online ?? 0) > 0} onset={setKnownOf(c.address)} onundo={undoKnown} />{/if}
          <span><b>{c.name}</b>{#if c.what && c.what !== c.name}<span class="muted">{` · ${c.what}`}</span>{/if}</span>
          {#if gm}
            <span><input type="number" min="0" value={c.amount ?? ""} onchange={(e) => setNumber(c.address, ABOARD.amount, (e.currentTarget as HTMLInputElement).value)} aria-label={`Amount of ${c.name}`} data-cargo-amount /> {c.unit ?? ""}</span>
          {:else}<span class="mono">{amountWords(c)}</span>{/if}
          {#if gm}<button type="button" class="quiet-btn" onclick={() => ((moving = moving === c.address ? null : c.address), (moveTo = parentOf(construct.address)))}>Move…</button>{/if}
          {#if moving === c.address}{@render mover(c)}{/if}
        </li>
      {:else}
        <li class="muted">Nothing loaded.</li>
      {/each}
    </ul>
  {:else}
    <div class="bar"><span class="muted small" data-crew-count>{counted}{crewMax ? ` of ${valueWords(crewMax.value)}` : ""} aboard</span><span class="grow"></span>{#if gm}<button type="button" class="quiet-btn" aria-expanded={adding} onclick={openAdd} data-add-person>Add someone…</button>{/if}</div>
    {#if adding}
      <div class="form">
        <label>Name <input type="text" bind:value={f.name} data-person-name /></label>
        <label>Role <input type="text" bind:value={f.role} placeholder="captain, pilot, passenger" data-role /></label>
        <label>Kind of being <input type="text" bind:value={f.being} placeholder="human" data-being /></label>
        <label>Age <input type="number" min="0" bind:value={f.age} size="4" data-age /></label>
        <label>Now <select bind:value={f.state}>{#each STATES as s}<option value={s}>{STATE_WORDS[s]}</option>{/each}</select></label>
        <button type="button" class="go" disabled={busy} onclick={add} data-add-confirm>Bring aboard</button>
      </div>
    {/if}
    <ul class="list" data-crew>
      {#each here.people as p (p.address)}
        <li data-person={p.address} class:gone={p.state === "dead" || p.state === "missing"} class:pencil={eyes && landmarkOf(p.address) !== "known"}>
          {#if eyes && known[p.address]}<KnownTo read={known[p.address]!} label={p.name} hold={(ws.players_online ?? 0) > 0} onset={setKnownOf(p.address)} onundo={undoKnown} />{/if}
          <span><b>{p.name}</b>{#if p.role && !gm}<span class="muted">{` · ${p.role}`}</span>{/if}</span>
          {#if gm}
            <input type="text" value={p.role ?? ""} placeholder="role" onchange={(e) => setText(p, ABOARD.role, (e.currentTarget as HTMLInputElement).value)} aria-label={`Role of ${p.name}`} />
          {/if}
          {#if gm}
            <select value={p.state} onchange={(e) => setState(p, (e.currentTarget as HTMLSelectElement).value as AboardState)} aria-label={`${p.name} is`} data-state>
              {#each STATES as s}<option value={s}>{STATE_WORDS[s]}</option>{/each}
            </select>
          {:else}<span>{STATE_WORDS[p.state]}</span>{/if}
          <span class="muted small">{[p.being ?? (depth === "curious" ? "" : "kind not stated"), ageWords(p)].filter(Boolean).join(" · ")}{#if depth === "engine"}<span class="mono">{` · ${p.address} ${sinceWords(p)}`}</span>{/if}</span>
          {#if gm}<button type="button" class="quiet-btn" onclick={() => ((moving = moving === p.address ? null : p.address), (moveTo = parentOf(construct.address)))} data-move>Move…</button>{/if}
          {#if moving === p.address}{@render mover(p)}{/if}
        </li>
      {:else}
        <li class="muted">Nobody aboard yet.</li>
      {/each}
    </ul>
    {#if depth !== "curious"}<p class="small muted">Ages move on while someone is awake; cold sleep stops the clock. Time dilation on fast voyages is not counted yet.</p>{/if}
  {/if}
  {#if note}<p class="small" aria-live="polite" data-aboard-note>{note}</p>{/if}
</section>

{#snippet mover(e: AboardEntry)}
  <div class="form move">
    <label>To <select bind:value={moveTo} data-move-to>{#each targets as t (t.address)}<option value={t.address}>{t.label}</option>{/each}</select></label>
    <button type="button" class="go" onclick={() => move(e)} data-move-confirm>Move</button>
  </div>
{/snippet}

<style>
  .aboard { margin-top: 10px; display: flex; flex-direction: column; gap: 6px; }
  .switch { display: inline-flex; align-self: flex-start; border: 1px solid var(--sse-rule); }
  .switch button { font: inherit; min-height: 28px; padding: 0 12px; border: none; background: transparent; color: var(--sse-ink); cursor: pointer; }
  .switch button[aria-pressed="true"] { background: var(--sse-ink); color: var(--sse-ground); }
  h5 { margin: 6px 0 0; font-size: 12px; text-transform: uppercase; letter-spacing: 0.06em; color: var(--sse-ink-muted); }
  dl { display: grid; grid-template-columns: max-content 1fr; gap: 4px 12px; margin: 4px 0; }
  dt { color: var(--sse-ink-muted); }
  dd { margin: 0; }
  .bar, .form { display: flex; flex-wrap: wrap; gap: 6px 10px; align-items: center; }
  .form label { display: inline-flex; gap: 4px; align-items: center; font-size: 12px; color: var(--sse-ink-muted); }
  .grow { flex: 1; }
  input, select { font: inherit; min-height: 28px; color: var(--sse-ink); background: var(--sse-ground); border: 1px solid var(--sse-rule); }
  input[type="number"] { width: 6em; }
  .list, .stores { list-style: none; margin: 0; padding: 0; display: flex; flex-direction: column; gap: 4px; }
  .list li { display: flex; flex-wrap: wrap; gap: 4px 10px; align-items: center; border-bottom: 1px solid var(--sse-rule); padding: 3px 0; }
  .list li.pencil b, .list li.pencil > span { color: var(--sse-ink-muted); font-style: italic; }
  .list li.pencil { border-bottom-style: dotted; }
  .list li.gone b { text-decoration: line-through; color: var(--sse-ink-muted); }
  .stores li { display: grid; grid-template-columns: 5em 7em 1fr; gap: 0 10px; }
  .move { width: 100%; }
  .quiet-btn, .go { font: inherit; min-height: 28px; padding: 0 10px; border: 1px solid var(--sse-rule); background: transparent; color: var(--sse-ink); cursor: pointer; transition: transform var(--sse-motion-quick, 120ms) var(--sse-ease, ease-out); }
  .quiet-btn:hover, .go:hover { transform: translateY(-1px); border-color: var(--sse-ink); }
  .go { background: var(--sse-ink); color: var(--sse-ground); border-color: var(--sse-ink); }
  button:disabled { opacity: 0.5; cursor: default; transform: none; }
  .mono { font-family: var(--sse-font-mono, monospace); }
  .small { font-size: 12px; margin: 0; }
  .muted { color: var(--sse-ink-muted); }
  @media (pointer: coarse) { .quiet-btn, .go, input, select, .switch button { min-height: 44px; } }
  @media (prefers-reduced-motion: reduce) { .quiet-btn, .go { transition: none; } }
</style>

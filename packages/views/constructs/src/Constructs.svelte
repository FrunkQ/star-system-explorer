<!--
  Ships and stations (V3 gap 13, J7 templates): the constructs in this system,
  each with its card at the depth in force, and "Add here" from the stacked
  packs' templates. Editing a fact is one set_value; taking one out of play is
  the kernel's retire (nothing is deleted). Every template, kind and word comes
  from the packs. The card's Log (V3 gap 16) is read from the history and the
  record, so it says only what happened.
-->
<script lang="ts">
  import { untrack } from "svelte";
  import { BOOKKEEPING, IN_SYSTEM_SHIPS, navShipOf, shipChoices, byKind, constructsIn, factOf, kindOf, placeHash, secondsToYears, yearsToSeconds, shipLog, templateOf, eyesShow, secrecyRelevant, tableOf, type IntentResult, type KnownChange, type KnownLandmark, type KnownTo as KnownRead, type ConstructTemplate, type Json, type ShipLogEntry, type PlayerSubject, type SystemScene, type ViewContext } from "../../../ui-contract/src/index.ts";
  import { KnownTo, dateWords, defaultCalendar, fromPackCalendar, opensFrom } from "../../../chart-grammar/src/index.ts";
  import { fieldWords, valueWords } from "./words.ts";
  import { Aboard } from "../../manifest/src/index.ts";
  import StationPicture from "../../ship/src/StationPicture.svelte";

  let { ctx, scene }: { ctx: ViewContext; scene: SystemScene; params: Record<string, Json> } = $props();
  const session = untrack(() => ctx.session);
  const ws = $derived(ctx.workspace);
  const gm = session.seat.authority === "gm";
  const depth = $derived(ws.depth ?? "science");
  const editing = $derived(gm && ws.mode === "edit");

  let templates = $state<ConstructTemplate[]>([]);
  let listed = $state<PlayerSubject[]>([]);
  let note = $state<string | null>(null);
  let picked = $state<string | null>(null);
  const kinds = $derived([...new Set(templates.map((t) => t.kind))]);
  const groups = $derived(byKind(templates));

  async function campaignAt(): Promise<bigint> {
    const now = await session.campaignNow();
    return now.ok && now.at_s !== undefined ? now.at_s : 0n;
  }
  async function load() {
    const t = await session.templates?.();
    if (t?.ok) templates = t.templates;
    const s = await session.scene(scene.system);
    if (!s.ok) return void (note = s.message);
    note = null;
    listed = constructsIn(s.scene, [...new Set((t?.ok ? t.templates : templates).map((x) => x.kind))]);
  }
  $effect(() => {
    void ws.revision;
    void scene.system;
    untrack(() => void load());
  });

  const names = $derived(Object.fromEntries(scene.subjects.map((s) => [s.address, s.label])));
  const nameOf = (s: PlayerSubject) => String(factOf(s, "name") ?? s.address.split("/").pop());
  const parentOf = (a: string) => a.slice(0, a.lastIndexOf("/"));
  const whereWords = (a: string) => {
    const p = parentOf(a);
    return p === scene.system ? `in ${scene.title}` : `at ${names[p] ?? p.split("/").pop()}`;
  };
  const current = $derived(listed.find((s) => s.address === picked) ?? null);
  const currentTemplate = $derived(current ? templateOf(current, templates) : undefined);
  const shownFacts = $derived.by(() => {
    if (!current) return [];
    const all = current.facts.filter((f) => f.field !== "name" && (depth === "engine" || !BOOKKEEPING.includes(f.field)));
    return depth === "curious" ? all.filter((f) => f.field.startsWith("construct.")).slice(0, 4) : all;
  });

  // ---- Add here ----
  let adding = $state(false);
  let addBtn = $state<HTMLElement | null>(null);
  let choice = $state<string | null>(null);
  let under = $state<string>("");
  let newName = $state("");
  let busy = $state(false);
  // Where it goes: the world in view when there is one, else the system itself.
  function openAdd() {
    adding = !adding;
    if (!adding) return;
    const sel = ws.selection && scene.subjects.some((s) => s.address === ws.selection) ? ws.selection : null;
    under = sel ?? scene.system;
    choice = null;
    newName = "";
  }
  const chosen = $derived(templates.find((t) => t.id === choice) ?? null);
  async function add() {
    if (!chosen) return;
    busy = true;
    const r = await session.createFrom?.({ template: chosen.id, under, ...(newName.trim() ? { name: newName.trim() } : {}), at_s: await campaignAt() });
    busy = false;
    if (!r) return;
    if (!r.ok) return void (note = r.message);
    note = `${newName.trim() || chosen.label} added ${whereWords(r.result.address)}.`;
    adding = false;
    picked = r.result.address;
    ws.revision++;
  }

  // ---- The log (V3 gap 16) ----
  let logOpen = $state(false);
  let logBtn = $state<HTMLElement | null>(null);
  let log = $state<ShipLogEntry[]>([]);
  const calendar = $derived(ws.pack_calendar ? fromPackCalendar(ws.pack_calendar) : defaultCalendar(ws.campaign_epoch_yr ?? ws.campaign_now_yr ?? ws.now_yr));
  async function loadLog(address: string) {
    const [h, r] = await Promise.all([session.history(address), session.record?.(address)]);
    if (picked !== address) return;
    log = shipLog(h, r?.ok ? r.record : null, (s) => secondsToYears(s), (k) => fieldWords(k).label).reverse();
  }
  $effect(() => {
    void ws.revision;
    const a = picked;
    if (a && logOpen) untrack(() => void loadLog(a));
  });

  // ---- Aboard (track MC): crew and manifest in one panel, opened from the card ----
  let aboardOpen = $state(false);
  const places = $derived([
    ...listed.map((s) => ({ address: s.address, label: nameOf(s) })),
    ...scene.subjects.map((s) => ({ address: s.address, label: s.address === scene.system ? `${s.label} (the system)` : s.label })),
  ]);

  // ---- Edit and retire ----
  let edits = $state<Record<string, string>>({});
  // A station's picture (renderer station-design.ts): built from its description, kept as `construct.design`.
  let pictureOpen = $state(false);
  const isStation = (c: PlayerSubject) => (templateOf(c, templates)?.kind ?? kindOf(c.address)) === "station";
  async function setDesign(r: unknown) {
    if (!current || !gm || ws.desk !== "build") return;
    const res = await session.send({ kind: "set_value", address: current.address, field: "construct.design", value: JSON.stringify(r), at_yr: ws.campaign_now_yr ?? ws.now_yr });
    note = res.status === "applied" ? null : res.status === "refused" ? res.reason : "Sent to the game master.";
    ws.revision++;
  }
  async function setFact(field: string, raw: string, was: unknown) {
    if (!current) return;
    const value: Json = typeof was === "number" ? Number(raw) : typeof was === "boolean" ? raw === "yes" || raw === "true" : raw;
    if (typeof was === "number" && !Number.isFinite(value as number)) return void (note = `${raw} is not a number.`);
    const r = await session.send({ kind: "set_value", address: current.address, field, value, at_yr: ws.campaign_now_yr ?? ws.now_yr });
    note = r.status === "applied" ? null : r.status === "refused" ? r.reason : "Sent to the game master.";
    delete edits[field];
    ws.revision++;
  }
  // Known to (SV2): each construct's eye in a narrow column on the left, sortable so "what the table does not know"
  // is one press; the eye shows on Build and Run once secrecy matters here. A GM-made base is where it most often does.
  let known = $state.raw<Record<string, KnownRead>>({});
  // Who knows what is read and changed at one exact second of the campaign's now (the story's date), never the sky's
  // time, as on the body card: a change made where the read does not reach would leave the eye where it was.
  function knownAt(): { at_yr: number; at_s: string } {
    const at_yr = ws.campaign_now_yr ?? ws.now_yr;
    return { at_yr, at_s: String(yearsToSeconds(at_yr)) };
  }
  $effect(() => {
    const list = listed.map((x) => x.address);
    void ws.revision;
    const at = BigInt(knownAt().at_s);
    if (!gm || (ws.desk !== "build" && ws.desk !== "run")) return void (known = {});
    let live = true;
    void Promise.all(list.map(async (a) => [a, await session.knownTo(a, undefined, at)] as const)).then((rs) => {
      if (live) known = Object.fromEntries(rs.flatMap(([a, r]) => (r.ok ? [[a, r.value]] : [])));
    });
    return () => (live = false);
  });
  const eyes = $derived(eyesShow(ws.desk, Object.values(known).some((k) => secrecyRelevant(k, !!ws.shared))));
  const landmarkOf = (a: string): KnownLandmark => tableOf(known[a])?.landmark ?? "known";
  let unknownFirst = $state(false);
  const RANK: Record<KnownLandmark, number> = { gm_only: 0, heard_of: 1, known: 2 };
  const rows = $derived(unknownFirst ? [...listed].sort((a, b) => RANK[landmarkOf(a.address)] - RANK[landmarkOf(b.address)]) : listed);
  const why = (r: IntentResult) => (r.status === "refused" ? r.reason : r.status === "pending" ? "sent to the GM" : r.status === "error" ? r.message : "");
  const setKnownOf = (address: string) => async (c: KnownChange): Promise<{ ok: true; undo?: string } | { ok: false; why: string }> => {
    const r = await session.send({ kind: "set_known", address, ...c, ...knownAt() });
    if (r.status === "applied") return (ws.revision++, { ok: true, ...(r.event_id ? { undo: r.event_id } : {}) });
    return { ok: false, why: why(r) };
  };
  async function undoKnown(undo: string): Promise<string | null> {
    const r = await session.send({ kind: "undo_known", undo, ...knownAt() });
    return r.status === "applied" ? (ws.revision++, null) : why(r);
  }

  let retiring = $state(false);
  let reason = $state("");
  async function retireIt() {
    if (!current) return;
    const r = await session.retire?.(current.address, reason.trim() || undefined, await campaignAt());
    if (!r) return;
    if (!r.ok) return void (note = r.message);
    note = `${nameOf(current)} is out of play. Its history stays.`;
    retiring = false;
    reason = "";
    picked = null;
    ws.revision++;
  }
  // The party's ship (landing run-through): a ship that names a nav ship can be the one the table travels in.
  const shipRows = $derived.by(() => {
    void ws.revision;
    const st = session.stack();
    return shipChoices(IN_SYSTEM_SHIPS, [...st.base, ...(st.campaign ? [st.campaign] : [])] as unknown as Parameters<typeof shipChoices>[1]);
  });
  const navOf = (s: PlayerSubject) => navShipOf(factOf(s, "construct.nav_ship"), shipRows);
  const isParty = (s: PlayerSubject) => ws.party?.ship === s.address;
  function makeParty(s: PlayerSubject) {
    const nav = navOf(s);
    if (!nav || !ws.setParty) return;
    ws.setParty(isParty(s) ? null : { ship: s.address, name: nameOf(s), nav, at: parentOf(s.address) });
    note = isParty(s) ? `${nameOf(s)} is the party's ship: Voyages starts where it is, and Land lands it.` : `The party has no ship.`;
  }
  const bodyAt = (a: string) => {
    const p = parentOf(a);
    return p !== scene.system && kindOf(p) === "body" ? p : null;
  };
</script>

<div class="desk" role="region" aria-label="Ships and stations">
  {#if !templates.length && !listed.length}
    <p class="lede">No pack in the stack gives ship or station templates. Add a game pack under Workshop, Packs, and its stock ships, stations and sites can be added here.</p>
  {:else}
    <div class="bar">
      <h3>Ships and stations in {scene.title}</h3>
      <span class="grow"></span>
      {#if gm && templates.length}<button type="button" class="go" bind:this={addBtn} aria-expanded={adding} onclick={openAdd} data-add-here>Add here…</button>{/if}
    </div>

    {#if adding}
      <section class="pop" aria-label="Add here" use:opensFrom={() => addBtn}>
        <div class="row">
          <label class="lbl">Where
            <select bind:value={under} data-under>
              <option value={scene.system}>{scene.title} (the system)</option>
              {#each scene.subjects.filter((s) => s.address !== scene.system) as s}<option value={s.address}>{s.label}</option>{/each}
            </select>
          </label>
          <label class="lbl">Name <input type="text" bind:value={newName} placeholder={chosen?.label ?? "its own name"} data-name /></label>
        </div>
        {#each groups as g}
          <h4>{g.words}</h4>
          <ul class="lib">
            {#each g.templates as t}
              <li>
                <button type="button" class="tpl" aria-pressed={choice === t.id} onclick={() => (choice = t.id)} data-template={t.id}>
                  <b>{t.label}</b>{#if t.explain}<span class="muted"> · {t.explain}</span>{/if}
                </button>
              </li>
            {/each}
          </ul>
        {/each}
        <div class="row">
          <button type="button" class="go" disabled={!chosen || busy} onclick={add} data-add>{chosen ? `Add ${chosen.label}` : "Pick one"}</button>
          <span class="small muted">{chosen ? `From ${chosen.pack}. Its facts start as the template gives them; edit any of them after.` : ""}</span>
        </div>
      </section>
    {/if}

    {#if note}<p class="small" aria-live="polite" data-note>{note}</p>{/if}

    <div class="body">
      <ul class="list" aria-label="Constructs" class:eyes>
        {#if eyes && listed.length > 1}<li class="eyehead"><button type="button" class="sort" aria-pressed={unknownFirst} data-known-sort onclick={() => (unknownFirst = !unknownFirst)} title="Put what the table does not know first">{unknownFirst ? "What the table does not know first" : "Sort by who knows"}</button></li>{/if}
        {#each rows as s (s.address)}
          <li class:pencil={eyes && landmarkOf(s.address) !== "known"}>
            {#if eyes && known[s.address]}<KnownTo read={known[s.address]!} label={nameOf(s)} hold={(ws.players_online ?? 0) > 0} onset={setKnownOf(s.address)} onundo={undoKnown} />{/if}
            <button type="button" class="row-btn" aria-pressed={picked === s.address} onclick={() => ((picked = picked === s.address ? null : s.address), (retiring = false))} data-construct={s.address}>
              <b>{nameOf(s)}</b>{#if isParty(s)} <span class="party" data-party title="The party's ship">party</span>{/if} <span class="muted">{templateOf(s, templates)?.label ?? kindOf(s.address)} · {whereWords(s.address)}</span>
            </button>
          </li>
        {:else}
          <li class="muted">None yet. "Add here" puts a stock ship, station or site in this system.</li>
        {/each}
      </ul>

      {#if current}
        <aside class="card" aria-label={nameOf(current)} use:opensFrom={() => document.querySelector(`[data-construct="${CSS.escape(current.address)}"]`)}>
          <h3>{nameOf(current)}</h3>
          <p class="small muted">{currentTemplate?.label ?? kindOf(current.address)} · {whereWords(current.address)}</p>
          {#if currentTemplate?.explain && depth !== "engine"}<p>{currentTemplate.explain}.</p>{/if}
          {#if depth === "engine"}<p class="mono small" data-depth="engine">{current.address}</p>{/if}
          <dl>
            {#each shownFacts as f (f.field)}
              {@const w = fieldWords(f.field)}
              <dt>{w.label}{#if f.marks?.includes("fictional")} <span class="mark" title="Fictional: a game's rule, not physics">fictional</span>{/if}</dt>
              <dd>
                {#if editing && !BOOKKEEPING.includes(f.field)}
                  <input type="text" value={edits[f.field] ?? valueWords(f.value)} oninput={(e) => (edits[f.field] = (e.currentTarget as HTMLInputElement).value)} onkeydown={(e) => e.key === "Enter" && edits[f.field] !== undefined && setFact(f.field, edits[f.field]!, f.value)} onblur={() => edits[f.field] !== undefined && setFact(f.field, edits[f.field]!, f.value)} aria-label={w.label} data-field={f.field} />
                {:else}
                  {valueWords(f.value)}
                {/if}
                {#if w.unit}<span class="muted"> {w.unit}</span>{/if}
                {#if depth === "engine"}<span class="mono muted small"> {f.field}</span>{/if}
              </dd>
            {/each}
          </dl>
          {#if depth === "curious" && current.facts.length > shownFacts.length}<p class="small muted">More at Science depth.</p>{/if}
          <nav class="links">
            {#if bodyAt(current.address)}<a href={placeHash({ scale: "world", system: scene.system, world: bodyAt(current.address)! })}>Open the world it is at</a>{/if}
            {#if gm && ws.setParty && navOf(current)}<button type="button" class="quiet-btn" aria-pressed={isParty(current)} onclick={() => makeParty(current!)} data-party-ship>{isParty(current) ? "The party's ship" : "Make this the party's ship"}</button>{/if}
            {#if isStation(current)}<button type="button" class="quiet-btn" aria-expanded={pictureOpen} onclick={() => (pictureOpen = !pictureOpen)} data-station-picture-toggle>Picture</button>{/if}
            {#if gm}<button type="button" class="quiet-btn" aria-expanded={aboardOpen} onclick={() => (aboardOpen = !aboardOpen)} data-aboard-toggle>Aboard</button>{/if}
            {#if gm}<button type="button" class="quiet-btn" bind:this={logBtn} aria-expanded={logOpen} onclick={() => (logOpen = !logOpen)} data-log-toggle>Log</button>{/if}
            {#if gm && current.facts.some((f) => f.field === "template.id")}<button type="button" class="quiet-btn" onclick={() => (retiring = !retiring)} data-retire>Take out of play…</button>{/if}
          </nav>
          {#if pictureOpen && isStation(current)}
            <div use:opensFrom={() => document.querySelector("[data-station-picture-toggle]")}>
              <StationPicture construct={current} name={nameOf(current)} words={currentTemplate?.explain ?? ""} tags={currentTemplate?.tags ?? []} locked={!(gm && ws.desk === "build")} onchange={(r) => setDesign(r)} />
            </div>
          {/if}
          {#if aboardOpen}
            <div use:opensFrom={() => document.querySelector("[data-aboard-toggle]")}>
              <Aboard {ctx} construct={current} {places} />
            </div>
          {/if}
          {#if logOpen}
            <section class="log" aria-label="Log" use:opensFrom={() => logBtn}>
              <h4>Log, newest first</h4>
              <ol data-log>
                {#each log as e, i (i)}
                  <li data-log-what={e.what}>
                    <span class="mono muted small">{dateWords(e.at_yr, calendar)}</span>
                    <span>{e.text}{#if e.why}<span class="muted"> · {e.why}</span>{/if}</span>
                    {#if depth !== "curious"}<span class="muted small">by {e.who}{#if depth === "engine"} · <span class="mono">{e.key}</span>{/if}</span>{/if}
                  </li>
                {:else}
                  <li class="muted">Nothing yet.</li>
                {/each}
              </ol>
              <p class="small muted">Read from the campaign's history and record: edit the ship, its kept values or its notes and the log follows.</p>
            </section>
          {/if}
          {#if retiring}
            <div class="row" use:opensFrom={() => document.querySelector("[data-retire]")}>
              <input type="text" bind:value={reason} placeholder="Why (lost with all hands, scrapped…)" aria-label="Why" />
              <button type="button" class="go" onclick={retireIt} data-retire-confirm>Take it out of play</button>
            </div>
            <p class="small muted">Nothing is deleted: it stays in the history, marked retired.</p>
          {/if}
        </aside>
      {/if}
    </div>
  {/if}
</div>

<style>
  .desk { color: var(--sse-ink); font-size: 13px; display: flex; flex-direction: column; gap: 12px; }
  .bar, .row { display: flex; flex-wrap: wrap; gap: 8px 12px; align-items: center; }
  .bar h3 { margin: 0; font-size: 15px; }
  .grow { flex: 1; }
  .lbl { font-family: var(--sse-font-mono, monospace); display: inline-flex; gap: 6px; align-items: center; }
  select, input[type="text"] { font: inherit; min-height: 34px; color: var(--sse-ink); background: var(--sse-ground); border: 1px solid var(--sse-rule); }
  dd input[type="text"] { min-height: 26px; width: 9em; }
  .party { font-size: 0.75em; padding: 0 5px; border: 1px solid currentColor; border-radius: 8px; color: var(--sse-accent, var(--sse-ink)); }
  .quiet-btn, .go, .tpl, .row-btn { font: inherit; min-height: 34px; padding: 0 10px; border: 1px solid var(--sse-rule); background: transparent; color: var(--sse-ink); cursor: pointer; text-align: left; transition: transform var(--sse-motion-quick, 120ms) var(--sse-ease, ease-out); }
  .quiet-btn:hover, .go:hover, .tpl:hover, .row-btn:hover { transform: translateY(-1px); border-color: var(--sse-ink); }
  .go { background: var(--sse-ink); color: var(--sse-ground); border-color: var(--sse-ink); }
  .tpl, .row-btn { width: 100%; border-color: transparent; border-bottom-color: var(--sse-rule); }
  .tpl[aria-pressed="true"], .row-btn[aria-pressed="true"] { border-color: var(--sse-accent); }
  button:disabled { opacity: 0.5; cursor: default; transform: none; }
  .pop { border: 1px solid var(--sse-rule); background: var(--sse-ground); padding: 12px; display: flex; flex-direction: column; gap: 8px; max-height: 60vh; overflow: auto; }
  h4 { margin: 6px 0 0; font-size: 12px; text-transform: uppercase; letter-spacing: 0.06em; color: var(--sse-ink-muted); }
  .lib, .list { list-style: none; margin: 0; padding: 0; }
  .list.eyes li { display: flex; align-items: center; gap: 2px; }
  .list.eyes .row-btn { flex: 1; min-width: 0; }
  /* Pencil: a construct the table has not been told of reads quieter, in italic, with a dotted rule. */
  .list li.pencil .row-btn { color: var(--sse-ink-muted); font-style: italic; border-bottom-style: dotted; }
  .eyehead { justify-content: flex-start; }
  .sort { font: inherit; font-size: 12px; min-height: 28px; padding: 0 6px; border: none; background: none; color: var(--sse-ink-muted); text-decoration: underline; cursor: pointer; }
  .body { display: grid; grid-template-columns: minmax(0, 1fr); gap: 16px; }
  .body:has(.card) { grid-template-columns: minmax(0, 2fr) minmax(260px, 3fr); }
  .card { border-left: 1px solid var(--sse-rule); padding-left: 16px; min-width: 0; }
  .card h3 { margin: 0; font-size: 15px; }
  dl { display: grid; grid-template-columns: max-content 1fr; gap: 4px 12px; margin: 8px 0; }
  dt { color: var(--sse-ink-muted); }
  dd { margin: 0; }
  .links { display: flex; gap: 12px; align-items: center; flex-wrap: wrap; }
  .links a { color: var(--sse-accent); }
  .mark { color: var(--sse-accent); font-size: 11px; }
  .log { margin-top: 10px; }
  .log ol { list-style: none; margin: 6px 0; padding: 0; display: flex; flex-direction: column; gap: 4px; max-height: 40vh; overflow: auto; }
  .log li { display: grid; grid-template-columns: max-content 1fr; gap: 0 10px; }
  .log li > :nth-child(3) { grid-column: 2; }
  .mono { font-family: var(--sse-font-mono, monospace); }
  .lede { font-size: 14px; max-width: 46em; }
  .small { font-size: 12px; margin: 0; }
  .muted { color: var(--sse-ink-muted); }
  @media (max-width: 720px) {
    .body:has(.card) { grid-template-columns: minmax(0, 1fr); }
    .card { border-left: none; border-top: 1px solid var(--sse-rule); padding: 12px 0 0; }
  }
  @media (prefers-reduced-motion: reduce) {
    .quiet-btn, .go, .tpl, .row-btn { transition: none; }
  }
</style>

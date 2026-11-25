<!--
  Roll call (SH2, design SSE4-spaceflight-history-design.md section 4): the date in view, one sentence, then who was in
  space that day grouped by place, craft and person. A look-up over flight logs and stays (ui-contract rollcall.ts); it
  writes nothing. Restraint (design 8.1, standing): names as the record gives them, plain words, no decoration.
-->
<script lang="ts">
  import { untrack } from "svelte";
  import InMemory from "./InMemory.svelte";
  import { countSteps, onThisDay, rollRecords, sayNumber, lossesOn, dateOf, epochOf, formatDate, parseDate, rollCall, rollSentence, secondsToYears, stepEvent, yearsToSeconds, type RollCall as Roll, type SystemScene, type ViewContext } from "../../../ui-contract/src/index.ts";
  import { openCampaign, openPack, packFolder, type RollFeed } from "./feed.ts";

  let { ctx, scene }: { ctx: ViewContext; scene: SystemScene } = $props();
  const ws = $derived(ctx.workspace);

  let feed = $state<RollFeed | null>(null);
  let problem = $state<string | null>(null);
  let revision = $state(0);
  let opened = false;

  $effect(() => {
    const sys = scene.system;
    untrack(() => void open(sys));
  });
  async function open(system: string) {
    const base = packFolder();
    const f = base ? await openPack(base) : await openCampaign(ctx, system);
    if ("why" in f) {
      problem = f.why;
      return;
    }
    feed = f;
    problem = null;
    // A pack opened on a day outside its span starts on its latest day, so the first look is never empty.
    if (!opened && f.span) {
      opened = true;
      const t = Number(yearsToSeconds(ws.now_yr));
      if (t < f.span.from_s || t > f.span.to_s + 365 * f.calendar.day_s) ws.setTime(secondsToYears(f.span.to_s));
    }
  }

  const t = $derived(Number(yearsToSeconds(ws.now_yr)));
  const anchor = $derived(feed ? epochOf(feed.calendar) : null);
  const dayFrom = $derived(feed && anchor ? t - dateOf(feed.calendar, t, anchor).second : t);
  const dayTo = $derived(feed ? dayFrom + feed.calendar.day_s : t);
  const names = $derived(new Map(scene.subjects.map((s) => [s.address, s.label])));
  const nameOf = (a: string) => names.get(a) ?? (a.split("/").pop()?.replace(/^[a-z]+:/, "").replace(/^./, (c) => c.toUpperCase()) ?? a);

  // Fetch the logs the day's craft need, then read again.
  $effect(() => {
    const f = feed;
    if (!f) return;
    const [a, b] = [dayFrom, dayTo];
    const ids = [...new Set(f.ix.stays.filter((s) => s.on && s.from_s < b && (s.to_s === null || s.to_s > a)).map((s) => s.on!))];
    if (!ids.length) return;
    void f.need(ids).then((ix) => ix && (revision += 1));
  });

  const roll = $derived.by((): Roll | null => {
    void revision;
    if (!feed) return null;
    return rollCall(feed.ix, dayFrom, dayTo, Math.min(Math.max(t, dayFrom), dayTo - 1), { nameOf });
  });
  const title = $derived(feed && anchor ? formatDate(feed.calendar, t, anchor) : "");
  const dateOfS = (s: number) => (feed && anchor ? formatDate(feed.calendar, s, anchor) : "");
  const sentence = $derived(roll ? rollSentence(roll) : "");
  const also = $derived(feed?.allLogs && roll ? roll.also : []);

  function go(s: number | null) {
    if (s === null) return;
    ws.setTime(secondsToYears(s));
  }
  const day = (n: number) => feed && go(t + n * feed.calendar.day_s);
  const event = (dir: 1 | -1) => feed && go(stepEvent(feed.ix, dir > 0 ? Math.max(t, dayFrom) : Math.min(t, dayFrom), dir));

  // In memory (SH6): the page of names, opened from here; the day of a loss names it under the roster (design 8.4).
  let page = $state<"roll" | "memory">("roll");
  const losses = $derived(feed?.memory ? lossesOn(feed.memory, dayFrom, dayTo) : []);

  // Records and On this day (SH5, design 4.3 and 4.6): questions over the same stays and count, each with its how.
  const steps = $derived(feed ? (feed.steps ?? countSteps(feed.ix)) : []);
  const records = $derived(feed ? rollRecords(feed.ix, steps, feed.calendar.day_s) : []);
  const thisDay = $derived(feed?.span ? onThisDay(feed.ix, steps, feed.calendar, t, feed.span) : []);
  const monthDay = $derived(feed && anchor ? ((d) => (d.month && d.mday !== undefined ? `${d.mday} ${d.month}` : ""))(dateOf(feed.calendar, t, anchor)) : "");
  const people = (n: number) => `${sayNumber(n, 6)} ${n === 1 ? "person" : "people"}`;
  const days = (s: number) => (feed ? `${sayNumber(Math.floor(s / feed.calendar.day_s), 6)} days` : "");
  const recordWords = (r: (typeof records)[number]) => (r.unit === "people" ? (r.key === "empty" ? dateOfS(r.at_s) : people(r.n)) : `${days(r.n)}, ${r.person?.name ?? ""}`);
  const yearWords = (y: (typeof thisDay)[number]) =>
    [y.up ? `${people(y.up)} up` : "", y.went ? `${sayNumber(y.went, 6)} went up` : "", y.came ? `${sayNumber(y.came, 6)} came down` : ""].filter(Boolean).join(", ");

  let typing = $state(false);
  let typed = $state("");
  let typedWrong = $state(false);
  function startTyping() {
    typed = title;
    typedWrong = false;
    typing = true;
  }
  function takeTyped() {
    if (!feed || !anchor) return;
    const s = parseDate(feed.calendar, typed, anchor);
    if (s === undefined) return void (typedWrong = true);
    typing = false;
    // Noon, so the day's flights are found whichever side of midnight they began.
    go(s + feed.calendar.day_s / 2);
  }
  const focus = (el: HTMLInputElement) => {
    el.focus();
    el.select();
  };
</script>

<section class="rollcall" data-rollcall aria-label="Roll call">
  {#if page === "roll"}<p class="eyebrow">Roll call · days in {feed?.calendar.name.startsWith("Gregorian") ? "UTC" : (feed?.calendar.name ?? "the campaign's calendar")}</p>{/if}
  {#if problem}
    <p class="words" data-rollcall-problem>{problem}</p>
  {:else if !feed}
    <p class="words muted">Reading who flew…</p>
  {:else if page === "memory" && feed.memory}
    <InMemory memory={feed.memory} carried={feed.carried ?? []} calendar={feed.calendar} onback={() => (page = "roll")} onday={(s) => ((page = "roll"), go(s))} />
  {:else}
    {#if typing}
      <form class="typing" onsubmit={(e) => (e.preventDefault(), takeTyped())}>
        <input aria-label="Go to a date" bind:value={typed} use:focus onkeydown={(e) => e.key === "Escape" && (typing = false)} data-rollcall-type />
        <button type="submit">Go</button>
        {#if typedWrong}<span class="muted" role="alert">That date could not be read. Write it the way the title is written, such as “{title}”.</span>{/if}
      </form>
    {:else}
      <h2><button class="date" onclick={startTyping} title="Type a date" data-rollcall-date>{title}</button></h2>
    {/if}
    <p class="sentence" data-rollcall-sentence>{sentence}</p>

    <div class="steps" role="group" aria-label="Move through time">
      <button onclick={() => event(-1)} data-rollcall-step="event-back">‹ Event</button>
      <button onclick={() => day(-1)} data-rollcall-step="day-back">‹ Day</button>
      <button onclick={() => day(1)} data-rollcall-step="day-on">Day ›</button>
      <button onclick={() => event(1)} data-rollcall-step="event-on">Event ›</button>
    </div>

    {#if !feed.ix.stays.length}
      <p class="words" data-rollcall-none>No flights are recorded here yet. People aboard a craft with a flight log are listed here on the days they flew.</p>
    {:else if roll?.empty}
      <p class="words" data-rollcall-empty>
        {#if roll.empty.before_first}
          Nobody had been to space yet.{#if roll.empty.next}
            The first was {roll.empty.next.name}, on <button class="link" onclick={() => go(roll.empty!.next!.at_s)}>{dateOfS(roll.empty.next.at_s)}</button>.{/if}
        {:else}
          Nobody was in space on this day.{#if roll.empty.last}
            The last person up was {roll.empty.last.name}, who came down on <button class="link" onclick={() => go(roll.empty!.last!.at_s - 1)}>{dateOfS(roll.empty.last.at_s)}</button>.{/if}{#if roll.empty.next}
            The next was {roll.empty.next.name}, on <button class="link" onclick={() => go(roll.empty!.next!.at_s)}>{dateOfS(roll.empty.next.at_s)}</button>.{/if}
        {/if}
      </p>
    {/if}

    {#each roll?.groups ?? [] as g (g.key)}
      <h3 class="eyebrow place" data-rollcall-place={g.label}>{g.label}</h3>
      {#each g.crafts as c (c.name + (c.on ?? ""))}
        <div class="craft" class:pencil={c.mark === "estimated"} data-rollcall-craft={c.name}>
          <div class="craft-line">
            <span class="craft-name">{c.name}</span>
            <span class="where" title={c.mark === "estimated" ? `Estimated: ${c.why ?? "the source gives no more"}` : undefined}>{c.where}</span>
          </div>
          <ul>
            {#each c.people as r (r.person)}
              <li class:pencil={r.mark === "estimated"} class:below={!!r.below_edge} data-rollcall-person={r.name}>
                <span class="who">{r.name}{#if r.being && r.being !== "human"}<span class="being">, {r.being}</span>{/if}</span>
                <span class="role">{r.role ?? ""}</span>
                <span class="day">day {r.day}{r.of !== null ? ` of ${r.of}` : ""}</span>
                {#if r.below_edge}<span class="note">{r.below_edge}</span>{/if}
                {#if r.height_unknown}<span class="note">its height is not recorded</span>{/if}
                {#if r.mark === "estimated" && r.why}<span class="note">{r.why}</span>{/if}
              </li>
            {/each}
          </ul>
        </div>
      {/each}
    {/each}

    {#each losses as l (l.id)}
      <p class="loss" data-rollcall-loss={l.id}>{l.sentence} <button class="link" onclick={() => (page = "memory")}>In memory</button></p>
    {/each}

    {#if also.length}
      <h3 class="eyebrow place">Also in flight, nobody aboard</h3>
      <ul class="also" data-rollcall-also>
        {#each also as a (a.key)}<li><b>{a.label}:</b> {a.count}</li>{/each}
      </ul>
    {/if}

    {#if records.length}
      <details class="fold" data-rollcall-records>
        <summary>Records</summary>
        <ul>
          {#each records as r (r.key)}
            <li data-rollcall-record={r.key}>
              <span class="rec-label">{r.label}</span>
              <button class="link" onclick={() => go(r.at_s)}>{recordWords(r)}</button>
              <span class="note">{r.how}</span>
            </li>
          {/each}
        </ul>
      </details>
    {/if}
    {#if thisDay.length && monthDay}
      <details class="fold" data-rollcall-this-day>
        <summary>On this day, {monthDay}</summary>
        <ul class="years">
          {#each thisDay as y (y.year)}
            <li><button class="link" onclick={() => go(y.at_s)}>{y.year}</button> <span>{yearWords(y)}</span></li>
          {/each}
        </ul>
      </details>
    {/if}

    {#if feed.memory}<p class="memory-door"><button class="link" onclick={() => (page = "memory")} data-rollcall-in-memory>In memory ›</button></p>{/if}
    <p class="foot muted" data-rollcall-credit>
      From {feed.from}.{#if feed.credit}{" "}{feed.credit}.{/if}
      {#if feed.calendar.name.startsWith("Gregorian")}Days are read on the pack's clock, about a minute from UTC.{/if}
    </p>
  {/if}
</section>

<style>
  .rollcall {
    padding: 16px 18px 20px;
    background: var(--sse-ground-raised);
    border: 1px solid var(--sse-line);
    border-radius: var(--sse-radius);
    box-shadow: var(--sse-card-shadow);
    color: var(--sse-ink);
    font-family: var(--sse-font-body);
  }
  .eyebrow {
    margin: 0;
    font-size: 0.75rem;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: var(--sse-ink-muted);
  }
  h2 {
    margin: 2px 0 6px;
  }
  .date {
    font-family: var(--sse-font-display);
    font-size: 2rem;
    font-weight: 600;
    background: none;
    border: 0;
    border-bottom: 1px dashed var(--sse-line);
    padding: 0;
    color: var(--sse-ink);
    cursor: text;
  }
  .typing {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    align-items: center;
    margin: 6px 0;
  }
  .typing input {
    font: inherit;
    font-size: 1.2rem;
    padding: 4px 8px;
    border: 1px solid var(--sse-line);
    border-radius: var(--sse-radius);
    background: var(--sse-ground);
    color: var(--sse-ink);
    min-width: 0;
    flex: 1;
  }
  .sentence {
    margin: 0 0 12px;
    line-height: 1.45;
  }
  .steps {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    margin-bottom: 14px;
  }
  .steps button,
  .typing button {
    font: inherit;
    padding: 5px 12px;
    border: 1px solid var(--sse-ink);
    border-radius: var(--sse-radius);
    background: var(--sse-ground-raised);
    color: var(--sse-ink);
    cursor: pointer;
    transition: background var(--sse-motion-quick) var(--sse-ease);
  }
  .steps button:hover,
  .typing button:hover {
    background: var(--sse-ground);
  }
  .place {
    margin-top: 14px;
    padding-bottom: 4px;
    border-bottom: 1px solid var(--sse-rule);
  }
  .craft {
    margin: 8px 0 4px 8px;
  }
  .craft-line {
    display: flex;
    justify-content: space-between;
    gap: 12px;
    align-items: baseline;
  }
  .craft-name {
    font-family: var(--sse-font-display);
    font-size: 1.15rem;
    font-weight: 600;
  }
  .where {
    color: var(--sse-ink-muted);
    font-size: 0.9rem;
    text-align: right;
  }
  ul {
    list-style: none;
    margin: 4px 0 0;
    padding: 0 0 0 16px;
  }
  li {
    display: grid;
    grid-template-columns: minmax(0, 1.4fr) minmax(0, 1fr) auto;
    gap: 2px 12px;
    padding: 3px 0;
    align-items: baseline;
  }
  .day {
    font-family: var(--sse-font-mono);
    font-size: 0.8rem;
    color: var(--sse-ink-muted);
    white-space: nowrap;
  }
  .role {
    color: var(--sse-ink-muted);
    font-size: 0.9rem;
  }
  .note {
    grid-column: 1 / -1;
    font-size: 0.85rem;
    color: var(--sse-ink-muted);
  }
  .being {
    color: var(--sse-ink-muted);
  }
  .pencil .who,
  .pencil .craft-name,
  .pencil .where {
    font-style: italic;
  }
  li.pencil {
    border-bottom: 1px dotted var(--sse-line);
  }
  li.below .who {
    color: var(--sse-ink-muted);
  }
  .also li {
    display: block;
  }
  .words {
    line-height: 1.5;
  }
  .link {
    font: inherit;
    background: none;
    border: 0;
    padding: 0;
    color: var(--sse-ink);
    text-decoration: underline;
    cursor: pointer;
  }
  .muted {
    color: var(--sse-ink-muted);
  }
  .loss {
    margin: 12px 0 0 24px;
    line-height: 1.45;
    color: var(--sse-ink-muted);
  }
  .fold {
    margin-top: 16px;
    font-size: 0.9rem;
  }
  .fold summary {
    cursor: pointer;
    color: var(--sse-ink-muted);
  }
  .fold ul {
    list-style: none;
    margin: 6px 0 0;
    padding: 0;
  }
  .fold li {
    display: block;
    margin: 0 0 8px;
  }
  .fold .rec-label {
    display: block;
    color: var(--sse-ink-muted);
  }
  .fold .note {
    display: block;
    font-size: 0.8rem;
    color: var(--sse-ink-muted);
  }
  .years li {
    margin: 0 0 2px;
  }
  .memory-door {
    margin: 18px 0 0;
  }
  .foot {
    margin-top: 22px;
    font-size: 0.8rem;
    line-height: 1.4;
  }
  @media (max-width: 560px) {
    li {
      grid-template-columns: minmax(0, 1fr) auto;
    }
    .role {
      grid-column: 1 / -1;
      order: 3;
    }
  }
</style>

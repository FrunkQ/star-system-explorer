<!--
  The start sheet (top-level design 3.1 and 16, step 3): what the app opens on.
  Begin (carry on or the sample campaign, grow a new system, open or bring a
  campaign), What it can do (ten verbs, each opening the real thing), one line
  of reassurance, and an opening line that changes by itself. Lines and verbs
  are data (start.lines.json); anything that names a feature not built yet is
  held back. Not modal: Escape or "Go to the chart" leaves it, and it can be
  reopened from More.
-->
<script lang="ts">
  import { untrack } from "svelte";
  import { fade } from "svelte/transition";
  import { opensKey, pickLine, shownLines, type StartLine, type StartLinesDoc, type StartOpens } from "../../ui-contract/src/index.ts";
  import { reducedMotion } from "../../chart-grammar/src/index.ts";
  import { loadPref, savePref } from "./prefs.ts";

  export type StartAction = StartOpens | { door: "sol" | "carry" | "grow" | "bring" | "chart" };
  interface AtlasEntry {
    verb: string;
    hint: string;
    opens: StartOpens;
    needs?: string[];
  }

  let {
    doc,
    built,
    carryOn,
    sample,
    ready,
    onopen,
  }: {
    doc: StartLinesDoc & { atlas: AtlasEntry[] };
    built: ReadonlySet<string>;
    carryOn: { title: string; when: string } | null;
    sample: string;
    ready: boolean;
    onopen: (a: StartAction) => void;
  } = $props();

  const readList = (k: string): string[] => {
    try {
      const v = JSON.parse(loadPref(k) ?? "[]");
      return Array.isArray(v) ? v.map(String) : [];
    } catch {
      return [];
    }
  };
  const visits = untrack(() => Number(loadPref("start.visits") ?? "0") || 0);
  let skip = $state(loadPref("start") === "skip");
  let line = $state<StartLine | null>(null);
  untrack(() => {
    line = pickLine(doc.lines, built, { visits, last: loadPref("start.last"), opened: readList("start.opened") }, Math.random());
    savePref("start.visits", String(visits + 1));
    if (line) savePref("start.last", line.id);
  });
  const atlas = $derived(doc.atlas.filter((a) => (a.needs ?? []).every((n) => built.has(n))));
  const many = $derived(shownLines(doc.lines, built).length > 1);

  // The line gives way to the next about every twelve seconds, holding still under the pointer or focus; with reduced
  // motion it changes per visit only.
  let held = $state(false);
  $effect(() => {
    if (reducedMotion() || !many) return;
    const t = setInterval(() => {
      if (held) return;
      const next = pickLine(doc.lines, built, { visits: Number.MAX_SAFE_INTEGER, last: line?.id, opened: readList("start.opened") }, Math.random());
      if (next) ((line = next), savePref("start.last", next.id));
    }, 12000);
    return () => clearInterval(t);
  });

  function open(a: StartAction) {
    if (!("door" in a && ["sol", "carry", "grow", "bring", "chart"].includes(a.door))) {
      const seen = readList("start.opened");
      const k = opensKey(a);
      if (!seen.includes(k)) savePref("start.opened", JSON.stringify([...seen, k]));
    }
    onopen(a);
  }
  function setSkip(v: boolean) {
    skip = v;
    savePref("start", v ? "skip" : "show");
  }
  const showable = (o: StartOpens) => !("link" in o);
</script>

<section class="start" aria-labelledby="start-title" data-start>
  <div class="sheet">
    <header class="head">
      <p class="mark" id="start-title">Star System Explorer</p>
      <div class="line" aria-live="polite" role="group" aria-label="About" onpointerenter={() => (held = true)} onpointerleave={() => (held = false)} onfocusin={() => (held = true)} onfocusout={() => (held = false)}>
        {#if line}
          {#key line.id}
            <div class="said" in:fade={{ duration: reducedMotion() ? 0 : 600 }} data-start-line={line.id}>
              <h1>{line.headline}</h1>
              <p>
                {line.sentence}
                {#if showable(line.opens)}<button type="button" class="show" disabled={!ready} onclick={() => open(line!.opens)}>Show me ›</button>
                {:else if "link" in line.opens}<a class="show" href={line.opens.link} target="_blank" rel="noopener">Show me ›</a>{/if}
              </p>
            </div>
          {/key}
        {/if}
      </div>
    </header>

    <div class="cols">
      <section class="begin" aria-labelledby="begin-h">
        <h2 id="begin-h">Begin</h2>
        {#if built.has("sol")}
          <button type="button" class="door" class:primary={!carryOn} data-door="sol" disabled={!ready} onclick={() => open({ door: "sol" })}><b>Open the Solar System</b><span>{#if !carryOn}<em class="here">Start here.</em> {/if}The Sun, its planets and belts, as measured</span></button>
        {/if}
        {#if carryOn}
          <button type="button" class="door primary" data-door="carry" onclick={() => open({ door: "carry" })}><b>Carry on: {carryOn.title}</b><span>kept {carryOn.when}</span></button>
        {:else}
          <button type="button" class="door" class:primary={!built.has("sol")} data-door="carry" onclick={() => open({ door: "carry" })}><b>Open the sample campaign</b><span>{sample}: a K dwarf with five worlds and their moons</span></button>
        {/if}
        <button type="button" class="door" data-door="grow" disabled={!ready} onclick={() => open({ door: "grow" })}><b>Grow a new system</b><span>from a cloud of gas, every stage kept</span></button>
        <button type="button" class="door" data-door="bring" disabled={!ready} onclick={() => open({ door: "bring" })}><b>Open or bring a campaign</b><span>a campaign file, or a V3 save</span></button>
        {#if !ready}<p class="wait" aria-busy="true">Opening the chart…</p>{/if}
      </section>

      <section class="atlas" aria-labelledby="atlas-h">
        <h2 id="atlas-h">What it can do</h2>
        <ol>
          {#each atlas as a (a.verb)}
            <li><button type="button" disabled={!ready} onclick={() => open(a.opens)} data-atlas={opensKey(a.opens)}><b>{a.verb}</b><span>{a.hint}</span></button></li>
          {/each}
        </ol>
      </section>
    </div>

    <footer class="foot">
      <p>Works in the browser, with nothing to install. Your work is kept on this device.</p>
      <label><input type="checkbox" checked={skip} onchange={(e) => setSkip((e.currentTarget as HTMLInputElement).checked)} /> Open straight to the chart next time</label>
      <button type="button" class="linkbtn" data-door="chart" onclick={() => open({ door: "chart" })}>Go to the chart ›</button>
    </footer>
  </div>
</section>

<style>
  .start {
    position: fixed;
    inset: 0;
    z-index: 40;
    overflow: auto;
    background: var(--sse-ground);
    color: var(--sse-ink);
    font-family: var(--sse-font-body);
  }
  .sheet {
    box-sizing: border-box;
    max-width: 1080px;
    margin: 0 auto;
    padding: 48px 32px 32px;
    display: flex;
    flex-direction: column;
    gap: 36px;
    min-height: 100%;
  }
  .mark {
    margin: 0 0 18px;
    font-family: var(--sse-font-mono);
    font-size: 12px;
    letter-spacing: 0.12em;
    text-transform: uppercase;
    color: var(--sse-ink-muted);
  }
  .line {
    min-height: 132px;
    display: grid;
  }
  .said {
    grid-area: 1 / 1;
  }
  h1 {
    margin: 0 0 10px;
    font-family: var(--sse-font-display);
    font-weight: 400;
    font-size: clamp(30px, 4.4vw, 48px);
    line-height: 1.08;
    max-width: 22ch;
  }
  .said p {
    margin: 0;
    font-size: 17px;
    max-width: 60ch;
    color: var(--sse-ink-muted);
  }
  .show {
    margin-left: 6px;
    padding: 0;
    border: 0;
    background: none;
    font: inherit;
    color: var(--sse-accent);
    cursor: pointer;
    text-decoration: underline;
    text-underline-offset: 3px;
  }
  .cols {
    display: grid;
    grid-template-columns: minmax(0, 5fr) minmax(0, 7fr);
    gap: 40px;
  }
  h2 {
    margin: 0 0 12px;
    font-family: var(--sse-font-mono);
    font-size: 12px;
    font-weight: 400;
    letter-spacing: 0.12em;
    text-transform: uppercase;
    color: var(--sse-ink-muted);
  }
  .begin {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  .door {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 2px;
    padding: 14px 16px;
    border: var(--sse-line) solid var(--sse-ink);
    background: var(--sse-ground-raised);
    color: var(--sse-ink);
    font: inherit;
    text-align: left;
    cursor: pointer;
    box-shadow: var(--sse-card-shadow);
  }
  .door b {
    font-family: var(--sse-font-display);
    font-weight: 400;
    font-size: 20px;
  }
  .door span {
    font-size: 13px;
    color: var(--sse-ink-muted);
  }
  .door.primary {
    background: var(--sse-ink);
    color: var(--sse-ground);
  }
  .here {
    font-style: normal;
    font-family: var(--sse-font-mono);
    font-size: 11px;
    letter-spacing: 0.08em;
    text-transform: uppercase;
  }
  .door.primary span {
    color: var(--sse-ground);
    opacity: 0.8;
  }
  .door:disabled,
  .atlas button:disabled {
    opacity: 0.5;
    cursor: progress;
  }
  .wait {
    margin: 0;
    font-size: 13px;
    color: var(--sse-ink-muted);
  }
  .atlas ol {
    margin: 0;
    padding: 0;
    list-style: none;
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    border-top: var(--sse-line) solid var(--sse-ink);
    counter-reset: atlas;
  }
  .atlas li {
    border-bottom: 1px solid var(--sse-ink-faint);
    counter-increment: atlas;
  }
  .atlas button {
    width: 100%;
    display: grid;
    grid-template-columns: 2.2em 1fr;
    gap: 0 6px;
    padding: 12px 8px 12px 0;
    border: 0;
    background: none;
    color: var(--sse-ink);
    font: inherit;
    text-align: left;
    cursor: pointer;
  }
  .atlas button::before {
    content: counter(atlas, decimal-leading-zero);
    grid-row: span 2;
    font-family: var(--sse-font-mono);
    font-size: 12px;
    color: var(--sse-ink-muted);
    padding-top: 3px;
  }
  .atlas button b {
    font-weight: 600;
  }
  .atlas button span {
    font-size: 13px;
    color: var(--sse-ink-muted);
  }
  .atlas button:hover b,
  .door:not(.primary):hover b {
    text-decoration: underline;
    text-underline-offset: 3px;
  }
  .foot {
    margin-top: auto;
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 10px 24px;
    padding-top: 16px;
    border-top: 1px solid var(--sse-ink-faint);
    font-size: 13px;
    color: var(--sse-ink-muted);
  }
  .foot p {
    margin: 0;
    flex: 1 1 320px;
  }
  .foot label {
    display: flex;
    align-items: center;
    gap: 6px;
  }
  .linkbtn {
    border: 0;
    background: none;
    padding: 0;
    font: inherit;
    color: var(--sse-ink);
    text-decoration: underline;
    text-underline-offset: 3px;
    cursor: pointer;
  }
  @media (max-width: 760px) {
    .sheet {
      padding: 28px 16px 20px;
      gap: 24px;
    }
    .line {
      min-height: 168px;
    }
    .cols {
      grid-template-columns: minmax(0, 1fr);
      gap: 24px;
    }
    .atlas ol {
      grid-template-columns: minmax(0, 1fr);
    }
  }
</style>

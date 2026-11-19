<!--
  The story finder (U8). Sifters are data (starters.json here, and any a
  pack brings); the kernel runs them over the whole campaign and returns
  hits best first with their evidence. This view lists them, says why each
  matters, picks the subject on the chart and copies a brief for a
  storyteller. Presentation only.
-->
<script lang="ts">
  import { untrack } from "svelte";
  import { SIGNIFICANCE_PARTS, fieldWords, storyBrief, valueWords, type Json, type SifterDoc, type SiftResult, type StoryHit, type SystemScene, type ViewContext } from "../../../ui-contract/src/index.ts";
  import starters from "./starters.json";

  let { ctx, scene, params }: { ctx: ViewContext; scene: SystemScene; params: Record<string, Json> } = $props();
  const minScore = untrack(() => Number(params.min_score ?? 0));
  const STARTERS = starters.sifters as SifterDoc[];
  const OFF_KEY = "sse.story.off";
  function loadOff(): string[] {
    try {
      const v = JSON.parse(localStorage.getItem(OFF_KEY) ?? "[]");
      return Array.isArray(v) ? v.map(String) : [];
    } catch {
      return [];
    }
  }
  let off = $state<string[]>(loadOff());
  function toggle(id: string) {
    off = off.includes(id) ? off.filter((x) => x !== id) : [...off, id];
    try {
      localStorage.setItem(OFF_KEY, JSON.stringify(off));
    } catch {
      /* private window: for this visit only */
    }
  }
  // The GM's own sifter, pasted as JSON (Orac can write these too).
  let own = $state("");
  const ownSifter = $derived.by((): { s: SifterDoc | null; err: string | null } => {
    if (!own.trim()) return { s: null, err: null };
    try {
      const v = JSON.parse(own) as SifterDoc;
      return v && typeof v === "object" && v.id && v.label ? { s: v, err: null } : { s: null, err: "a sifter needs an id, a label, a pattern and a significance" };
    } catch (e) {
      return { s: null, err: String((e as Error).message) };
    }
  });

  let result = $state<SiftResult | null>(null);
  let failed = $state<string | null>(null);
  let gen = 0;
  $effect(() => {
    void ctx.workspace.revision;
    const list = STARTERS.filter((s) => !off.includes(s.id)).concat(ownSifter.s ? [ownSifter.s] : []);
    const g = ++gen;
    ctx.session.sift({ sifters: list }).then((r) => {
      if (g !== gen) return;
      if (!r.ok) return void ((failed = r.message), (result = null));
      failed = null;
      result = r.result;
    });
  });

  const hits = $derived(result ? result.hits.filter((h) => h.significance.score >= minScore) : []);
  const nameOf = (a: string) => scene.subjects.find((s) => s.address === a)?.label ?? a.split("/").at(-1) ?? a;
  let copied = $state<string | null>(null);
  async function copy(h: StoryHit, i: number) {
    const text = storyBrief(h, nameOf);
    try {
      await navigator.clipboard.writeText(text);
      copied = `${i}`;
    } catch {
      copied = `${i}:shown`;
    }
  }
  let open = $state<number | null>(null);
</script>

<div class="story" data-story={result ? `${result.hits.length}` : ""}>
  <header class="head">
    <h2>Story finder</h2>
    <span class="stamp">GM only</span>
    <p class="lede" data-lede>
      {#if result}
        {hits.length ? `${hits.length} ${hits.length === 1 ? "thing" : "things"} worth a story in this campaign, the one that matters most first.` : "Nothing the sifters look for is in this campaign yet."}
      {:else if failed}
        {failed}
      {:else}
        Sifting the campaign…
      {/if}
    </p>
  </header>

  <div class="split">
    <section class="hits" aria-label="Hits">
      {#each hits as h, i (h.sifter + h.subject)}
        <article class="hit" data-hit={h.sifter}>
          <div class="top">
            <h3>{h.label}</h3>
            <span class="where">{nameOf(h.subject)}</span>
            <span class="score" title="How much it matters, 0 to 100">{Math.round(h.significance.score * 100)}</span>
          </div>
          <div class="parts" aria-label="Why it matters">
            {#each SIGNIFICANCE_PARTS as p, k (p)}
              <span class="part"><span class="meter"><span class="mfill" style:width="{Math.round(h.significance.components[k]! * 100)}%"></span></span>{p}</span>
            {/each}
          </div>
          <ul class="ev">
            {#each h.evidence as e, k (k)}
              <li>{e.address !== h.subject ? `${nameOf(e.address)}: ` : ""}{fieldWords(e.field)}: <strong>{valueWords(e.value)}</strong>{#if e.event}<span class="hint"> · set by event {e.event.slice(0, 8)}</span>{/if}</li>
            {/each}
          </ul>
          <div class="acts">
            <button type="button" onclick={() => ctx.workspace.select(h.subject)} data-pick>Pick on the chart</button>
            <button type="button" onclick={() => copy(h, i)} data-brief>Copy a brief for a storyteller</button>
            <button type="button" class="link" onclick={() => (open = open === i ? null : i)}>{open === i ? "Hide" : "Show"} the brief</button>
            {#if copied === `${i}`}<span class="hint" role="status">Copied.</span>{/if}
          </div>
          {#if open === i || copied === `${i}:shown`}<pre class="brief" data-brief-text>{storyBrief(h, nameOf)}</pre>{/if}
        </article>
      {/each}
    </section>

    <aside class="ledger">
      <h3>What it looks for</h3>
      <p class="hint">Sifters are data: patterns over facts and events, and how much a hit matters. Turn them on and off, or add your own.</p>
      <ul class="sifters">
        {#each STARTERS as s (s.id)}
          <li><label><input type="checkbox" checked={!off.includes(s.id)} onchange={() => toggle(s.id)} data-sifter={s.id} /> {s.label}</label></li>
        {/each}
        {#each (result?.sifters ?? []).filter((id) => !STARTERS.some((s) => s.id === id) && id !== ownSifter.s?.id) as id (id)}
          <li class="hint">{id} (from a pack)</li>
        {/each}
      </ul>
      <details>
        <summary>Add a sifter (JSON)</summary>
        <textarea bind:value={own} rows="8" spellcheck="false" aria-label="Your sifter as JSON" data-own></textarea>
        {#if ownSifter.err}<p class="hint" role="alert">{ownSifter.err}</p>{/if}
      </details>
      <h4>How much it matters</h4>
      <p class="hint">The mean of four parts, each 0 to 100: how long it lasts (up to 10 billion years), how far it reaches (up to 10,000 places), how hard it is to undo, and how rare it is (when the sifter asks).</p>
      <h4>Not yet</h4>
      <p class="hint">Sending a hit to Minerva waits for the agents; until then the brief copies as text. Sifters over events (impacts, flares, inked history) come as the campaign records them.</p>
    </aside>
  </div>
</div>

<style>
  .story {
    color: var(--sse-ink);
    font-size: 13px;
    display: flex;
    flex-direction: column;
    gap: 12px;
    max-width: 1180px;
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
    font-size: 17px;
    font-weight: 600;
    margin: 0;
  }
  h4 {
    font-size: 11px;
    letter-spacing: 0.12em;
    text-transform: uppercase;
    color: var(--sse-ink-muted);
    margin: 12px 0 4px;
    font-weight: 500;
  }
  .lede {
    margin: 0;
    flex-basis: 100%;
    font-size: 14px;
  }
  .stamp {
    border: 1.5px solid var(--sse-accent);
    color: var(--sse-accent-ink);
    font-size: 10px;
    letter-spacing: 0.14em;
    text-transform: uppercase;
    padding: 2px 8px;
  }
  .split {
    display: grid;
    grid-template-columns: minmax(0, 1fr) minmax(240px, 340px);
    gap: 20px;
    align-items: start;
  }
  .hits {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }
  .hit {
    border: 1px solid var(--sse-ink);
    padding: 12px 16px;
    background: var(--sse-paper, var(--sse-ground));
  }
  .top {
    display: flex;
    gap: 10px;
    align-items: baseline;
    flex-wrap: wrap;
  }
  .where {
    color: var(--sse-ink-muted);
  }
  .score {
    margin-left: auto;
    font-family: var(--sse-font-mono, monospace);
    font-size: 18px;
  }
  .parts {
    display: flex;
    flex-wrap: wrap;
    gap: 4px 14px;
    margin: 8px 0;
    color: var(--sse-ink-muted);
    font-size: 11px;
  }
  .part {
    display: inline-flex;
    gap: 6px;
    align-items: center;
  }
  .meter {
    display: block;
    width: 48px;
    height: 6px;
    border: 1px solid var(--sse-rule);
  }
  .mfill {
    display: block;
    height: 100%;
    background: var(--sse-ink);
  }
  .ev {
    margin: 0;
    padding-left: 18px;
  }
  .acts {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
    align-items: center;
    margin-top: 8px;
  }
  .acts button {
    font: inherit;
    min-height: 32px;
    padding: 0 10px;
    border: 1px solid var(--sse-ink);
    background: transparent;
    color: var(--sse-ink);
    cursor: pointer;
  }
  .acts button.link {
    border: none;
    text-decoration: underline;
    padding: 0;
  }
  .brief {
    white-space: pre-wrap;
    font-family: var(--sse-font-mono, monospace);
    font-size: 12px;
    border-left: 3px solid var(--sse-accent);
    padding: 6px 10px;
    margin: 8px 0 0;
  }
  .ledger {
    border: 1px solid var(--sse-ink);
    padding: 14px 18px;
  }
  .sifters {
    list-style: none;
    padding: 0;
    margin: 6px 0;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  label {
    display: inline-flex;
    gap: 6px;
    align-items: center;
  }
  textarea {
    width: 100%;
    box-sizing: border-box;
    font-family: var(--sse-font-mono, monospace);
    font-size: 12px;
    background: var(--sse-ground);
    color: var(--sse-ink);
    border: 1px solid var(--sse-rule);
  }
  .hint {
    color: var(--sse-ink-muted);
    font-size: 12px;
  }
  p {
    margin: 4px 0;
  }
  @media (max-width: 760px) {
    .split {
      grid-template-columns: 1fr;
    }
  }
</style>

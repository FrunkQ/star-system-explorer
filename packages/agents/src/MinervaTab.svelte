<!--
  Minerva (A3): narrative from what one audience knows. Pick who it is for;
  she is given only that audience's scene (through its data view), never the
  GM's. Her text is checked before you see it: every number must be in the
  facts she was given, and nothing they have not been shown may appear.
  A draft that fails is sent back to her, and never shown.
-->
<script lang="ts">
  import { asker, estimate, linked, type AgentsState, type LedgerRow } from "./agents.ts";
  import type { LoopResult } from "./loop.ts";
  import { aboutOne, factsBundle, hiddenFrom, hitsFor, minervaRequest, minervaSystem, runMinerva, type FactsBundle, type MinervaNotes, type Narrative } from "./minerva.ts";
  import type { PlayerScene, SiftResult, Vocab } from "../../ui-contract/src/index.ts";

  export interface MinervaHost {
    audiences: () => { id: string; label: string }[];
    /** The scene one audience is shown (K9), or the GM's with no audience. */
    scene: (audience?: string) => Promise<PlayerScene | null>;
    vocab: Vocab;
    title: string;
    /** The kernel's sifter hits over what one audience is shown (never the GM's), or null when it cannot say. */
    sift?: (audience: string) => Promise<SiftResult | null>;
    /** Keep a passed narrative as a subject's description (V3 gap 23): a note in the record for that audience. Null when kept, else why not. */
    keep?: (address: string, text: string, audience: string) => Promise<string | null>;
  }

  let { st, host, onrow, focus }: { st: AgentsState; host?: MinervaHost; onrow: (r: LedgerRow) => void; focus?: string | null } = $props();
  // Describe (V3 gap 23): a subject handed over from a card; her facts are that subject and its neighbours only.
  let focusLabel = $state("");
  let kept = $state<Record<number, string>>({});

  let audience = $state("everyone");
  let notes = $state<MinervaNotes>({ seed: "", style: "", length: "short" });
  let bundle = $state<FactsBundle | null>(null);
  let running = $state(false);
  let result = $state<LoopResult<Narrative> | null>(null);
  let problem = $state("");

  $effect(() => {
    const a = audience;
    void focus;
    bundle = null;
    kept = {};
    build(a).then((b) => {
      if (a === audience) bundle = b;
    });
  });
  const est = $derived(
    estimate(
      st,
      "describing",
      bundle
        ? [
            { role: "system", content: minervaSystem() },
            { role: "user", content: minervaRequest(bundle, notes) },
          ]
        : [],
      1500,
    ),
  );
  const money = (x: number) => `$${x < 0.01 ? x.toFixed(4) : x.toFixed(2)}`;

  /** The audience's facts, with the sifter hits over what they are shown. */
  async function build(a: string): Promise<FactsBundle | null> {
    if (!host) return null;
    const [all, r] = await Promise.all([host.scene(a), host.sift ? host.sift(a) : Promise.resolve(null)]);
    const s = all && focus ? aboutOne(all, focus) : all;
    if (all && focus) focusLabel = s ? (s.subjects.find((x) => x.address === focus)?.facts.find((f) => f.field === "name")?.value as string | undefined) ?? "" : "";
    return s ? factsBundle(s, host.vocab, { title: host.title, hits: focus ? [] : hitsFor(r, a, s) }) : null;
  }
  async function keepIt(i: number, text: string, a: string) {
    if (!host?.keep || !focus) return;
    const bad = await host.keep(focus, text, a);
    kept = { ...kept, [i]: bad ?? "kept" };
  }

  async function write(e: SubmitEvent) {
    e.preventDefault();
    if (!host) return;
    running = true;
    result = null;
    problem = "";
    const [theirs, gm, b] = await Promise.all([host.scene(audience), host.scene(), build(audience)]);
    if (!theirs || !gm || !b) {
      running = false;
      return void (problem = "The scene for that audience could not be built.");
    }
    const snap = JSON.parse(JSON.stringify(st)) as AgentsState;
    result = await runMinerva({ bundle: b, hidden: hiddenFrom(gm, theirs), notes, ask: asker(snap, "describing", { max_out: 1500, temperature: 0.7, onRow: onrow }) });
    running = false;
  }
</script>

<div role="tabpanel" aria-label="Minerva">
  <p>Minerva writes narrative from what one audience knows. She is given only their scene, through their data view; every number she writes must be in those facts, and anything they have not been shown stays out. A draft that fails the check goes back to her and is never shown.</p>
  {#if !host}
    <p class="hint" data-agent-off="minerva">Minerva works on the GM's chart room only.</p>
  {:else}
    {#if focus}<p class="meta" data-minerva-focus={focus}>Describing {focusLabel || "the body in focus"}: she is given it, what it orbits and what orbits it, as the audience knows them.</p>{/if}
    {#if !linked(st, "describing")}<p class="hint" data-agent-off="minerva">No model is linked for Describing. Link one under Link models.</p>{/if}
    <form class="ask" onsubmit={write}>
      <label>For <select bind:value={audience} aria-label="Audience">{#each host.audiences() as a (a.id)}<option value={a.id}>{a.label}</option>{/each}</select></label>
      <label>What about <input bind:value={notes.seed} aria-label="What about" placeholder="the approach to Tarn" /></label>
      <label>Voice <input bind:value={notes.style} aria-label="Voice" placeholder="ship's log, guidebook, briefing" /></label>
      <label>Length <select bind:value={notes.length} aria-label="Length"><option value="short">Short</option><option value="medium">Medium</option><option value="long">Long</option></select></label>
      {#if bundle}
        <details><summary>The facts she will be given</summary><pre class="facts" data-facts>{bundle.text}</pre></details>
      {/if}
      <p class="meta" data-minerva-estimate={est.tokens_in}>First try: about {est.tokens_in} tokens in, up to {est.max_out} out{est.usd !== undefined ? `, up to ${money(est.usd)}` : ""}.</p>
      <button type="submit" class="primary" disabled={running || !linked(st, "describing") || !bundle}>{running ? "Minerva is writing…" : "Write"}</button>
    </form>
    {#if problem}<p class="problem" role="alert">{problem}</p>{/if}
    {#if result}
      <p role="status" data-minerva-result={result.ok ? "ok" : "not"}>{result.message} {result.calls} {result.calls === 1 ? "call" : "calls"}{result.usd !== null ? `, ${money(result.usd)}` : ", price not given by the provider"}.</p>
      {#each result.passed as c, i (i)}
        <article class="narrative" data-narrative={c.value!.audience}>
          <p class="meta">Narrative · for {host.audiences().find((a) => a.id === c.value!.audience)?.label ?? c.value!.audience} · every number checked against their facts</p>
          {#each c.value!.text.split(/\n\s*\n/) as para, j (j)}<p>{para}</p>{/each}
          {#if focus && host.keep}
            {#if kept[i] === "kept"}<p class="meta" data-kept>Kept as its description, for {host.audiences().find((a) => a.id === c.value!.audience)?.label ?? c.value!.audience}.</p>
            {:else}<button type="button" data-keep-description onclick={() => keepIt(i, c.value!.text, c.value!.audience)}>Keep as its description</button>{#if kept[i]}<span class="problem" role="alert">{kept[i]}</span>{/if}{/if}
          {/if}
        </article>
      {/each}
      {#each result.failed as c, i (i)}
        <p class="meta" data-narrative-failed>Not shown: {c.problems.join("; ")}.</p>
      {/each}
    {/if}
  {/if}
</div>

<style>
  .facts {
    white-space: pre-wrap;
    font-size: 12px;
    max-height: 180px;
    overflow: auto;
  }
  .narrative {
    border-left: 3px solid var(--sse-accent, currentColor);
    padding: 2px 12px;
    margin: 8px 0;
    font-family: var(--sse-font-display);
  }
</style>

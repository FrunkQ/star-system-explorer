<!--
  Orac (A2): ask for a change in plain words. Orac drafts, SSE checks, Orac
  fixes (up to three times); only drafts that passed every check get an
  Accept button. Accepting goes through the GM's own path (a physics change
  is filed on the Physics screen for adoption; a lens joins your lenses; an
  atom issue opens GitHub's new-issue page for you to read and send).
-->
<script lang="ts">
  import { asker, estimate, linked, type AgentsState, type ChatMessage, type LedgerRow } from "./agents.ts";
  import type { LoopResult, Round } from "./loop.ts";
  import { oracRequest, oracSystem, runOrac, type Entry, type OracTarget } from "./orac.ts";
  import { draftFromPlanRequest, paperBlock, paperTokens, planPaper, planText, type Paper, type PaperPlan } from "./paper.ts";
  import { browserInflate, pdfText } from "./pdf-text.ts";

  export interface OracHost {
    targets: () => OracTarget[];
    catalogue: () => Entry[];
    /** What is open now (selected body, flow on the bench), as plain text. */
    here: () => string;
  }

  let { st, host, onrow, seed = "" }: { st: AgentsState; host?: OracHost; onrow: (r: LedgerRow) => void; seed?: string } = $props();

  // svelte-ignore state_referenced_locally
  let request = $state(seed);
  let running = $state(false);
  let rounds = $state<Round[]>([]);
  let result = $state<LoopResult<{ target: OracTarget; value: unknown }> | null>(null);
  let accepted = $state<Record<number, string>>({});
  // ND4: a paper handed to Orac. Read here, sent as labelled data; Orac plans in words first.
  let paper = $state.raw<Paper | null>(null);
  let paperProblem = $state("");
  let pasted = $state("");
  let plan = $state.raw<LoopResult<PaperPlan> | null>(null);
  async function takeFile(e: Event) {
    const f = (e.currentTarget as HTMLInputElement).files?.[0];
    (e.currentTarget as HTMLInputElement).value = "";
    if (!f) return;
    paperProblem = "";
    plan = null;
    if (/\.pdf$/i.test(f.name) || f.type === "application/pdf") {
      const r = await pdfText(new Uint8Array(await f.arrayBuffer()), browserInflate);
      if (r.ok) paper = { name: f.name, text: r.text, from: "pdf", pages: r.pages };
      else ((paper = null), (paperProblem = r.message));
    } else paper = { name: f.name, text: await f.text(), from: "text" };
  }
  function takePasted() {
    if (!pasted.trim()) return;
    paper = { name: "pasted text", text: pasted, from: "text" };
    pasted = "";
    plan = null;
  }

  const messages = $derived<ChatMessage[]>(host ? [{ role: "system", content: oracSystem(host.targets(), host.catalogue()) }, { role: "user", content: oracRequest(request, host.here()) + (paper ? `\n\n${paperBlock(paper)}` : "") }] : []);
  const est = $derived(estimate(st, "building", messages, 4000));
  const money = (x: number) => `$${x < 0.01 ? x.toFixed(4) : x.toFixed(2)}`;

  async function run(e: SubmitEvent) {
    e.preventDefault();
    if (!host || (!request.trim() && !paper)) return;
    running = true;
    rounds = [];
    result = null;
    accepted = {};
    const snap = JSON.parse(JSON.stringify(st)) as AgentsState;
    const ask = asker(snap, "building", { onRow: onrow });
    if (paper) {
      plan = null;
      plan = await planPaper({ paper, request, here: host.here(), targets: host.targets(), catalogue: host.catalogue(), ask, onRound: (r) => (rounds = [...rounds, r]) });
    } else result = await runOrac({ request, here: host.here(), targets: host.targets(), catalogue: host.catalogue(), ask, onRound: (r) => (rounds = [...rounds, r]) });
    running = false;
  }
  /** Step two for a paper: draft what the plan the GM has read says, with the paper as data. */
  async function draftPlan() {
    const p = plan?.passed[0]?.value;
    if (!host || !paper || !p) return;
    running = true;
    rounds = [];
    result = null;
    accepted = {};
    const snap = JSON.parse(JSON.stringify(st)) as AgentsState;
    result = await runOrac({ request: draftFromPlanRequest(request, JSON.parse(JSON.stringify(p)), paper), here: host.here(), targets: host.targets(), catalogue: host.catalogue(), ask: asker(snap, "building", { onRow: onrow }), onRound: (r) => (rounds = [...rounds, r]) });
    running = false;
  }
  const DONE: Record<string, string> = {
    lens: "Saved with your lenses: pick it from the Lens menu.",
    style: "Kept with your styles: name it on a screen item, or tweak it on the Styles screen.",
    screen: "Opened on the players page on this device. Open it in the Screens composer to share it.",
    symbology: "Kept with your symbols: choose it on the Symbols screen.",
    data_view: "Applied to the campaign's player data: it is on the Player data screen now.",
    sifter: "Added to the campaign's sifters: the story finder and Minerva use it now.",
    box: "Added to the campaign's boxes: any flow can use it as one block now.",
    table: "Added to the class atlas: the codes are worked out from each world's facts.",
    substance: "Added to your setting's data sheet: the editing screens offer it now.",
    trigger: "Added to the campaign's triggers: it fires from now on.",
    template: "Added to the campaign's templates: objects can be made from it.",
    reveal: "Sent: who knows it has changed, and the change undoes as one.",
    edition: "Kept in the campaign's own pack: choose it as a role's Atlas edition.",
    note_set: "Kept in the campaign's own pack: name it in an edition's margin to show it.",
    kit: "Kept in the campaign's own pack: choose it as a role's interface.",
    program: "Added to the campaign's programs. It runs once you attach it to an object whose template lists it.",
  };
  async function accept(i: number) {
    const c = result?.passed[i]?.value;
    if (!c) return;
    // The result lives in component state (deep proxies); a target gets plain data.
    let problem: string | null;
    try {
      problem = await c.target.accept(JSON.parse(JSON.stringify(c.value)));
    } catch (e) {
      problem = `Not applied: ${(e as Error).message}`;
    }
    accepted[i] = problem ?? (c.target.kind === "physics" ? "Filed on the Physics screen: adopt it there." : c.target.kind === "atom_issue" ? "Opened on GitHub for you to read and send." : (DONE[c.target.kind] ?? "Done."));
  }
</script>

<div role="tabpanel" aria-label="Orac">
  <p>Orac drafts changes from a sentence: physics, lenses, styles, player screens, symbols, data views, programs (autopilots), object templates, triggers, reveals (who knows what, for many things at once), whole player interfaces (kits), Atlas editions and margin notes, story sifters, boxes (blocks made of blocks), game tables as classes, missing gases and substances for the data sheet, and new physics blocks as issues. Hand it a paper (PDF or text) and it first answers with a plan in words: what the paper claims, what SSE has for it, what it would add and what is missing. SSE checks every draft and sends problems back to Orac, up to three times. Only drafts that pass can be accepted, and nothing changes until you accept.</p>
  {#if !host}
    <p class="hint" data-agent-off="orac">Orac works on the GM's chart room only.</p>
  {:else}
    {#if !linked(st, "building")}<p class="hint" data-agent-off="orac">No model is linked for Building. Link one under Link models; give it the strongest model you have.</p>{/if}
    <form class="ask" onsubmit={run}>
      <label>Ask Orac <textarea bind:value={request} rows="3" aria-label="Ask Orac" placeholder={paper ? "Bring this in." : "Tidally locked worlds should keep more heat on their day side."}></textarea></label>
      <div class="paper" data-orac-paper={paper ? paper.from : "none"}>
        {#if paper}
          <p class="meta">Paper: {paper.name}{paper.pages ? `, ${paper.pages} ${paper.pages === 1 ? "page" : "pages"}` : ""}, about {paperTokens(paper)} tokens. Orac reads it as data and plans in words before drafting anything. <button type="button" class="link" onclick={() => ((paper = null), (plan = null))}>Remove</button></p>
        {:else}
          <label class="meta">Hand Orac a paper (PDF or text) <input type="file" accept=".pdf,.txt,.md,application/pdf,text/plain,text/markdown" onchange={takeFile} aria-label="Hand Orac a paper" /></label>
          <details><summary class="meta">or paste its text</summary><textarea bind:value={pasted} rows="4" aria-label="Paste a paper's text"></textarea><button type="button" onclick={takePasted} disabled={!pasted.trim()}>Use this text</button></details>
        {/if}
        {#if paperProblem}<p class="hint" role="alert">{paperProblem}</p>{/if}
      </div>
      <p class="meta" data-orac-estimate={est.tokens_in}>First try: about {est.tokens_in} tokens in, up to {est.max_out} out{est.usd !== undefined ? `, up to ${money(est.usd)}` : ""}. Each fix sends the conversation again; up to four tries in all.{est.fits ? "" : " Too long for this model: nothing will be sent."}</p>
      <button type="submit" class="primary" disabled={running || !linked(st, "building") || (!request.trim() && !paper)}>{running ? (paper && !result ? "Orac is reading…" : "Orac is drafting…") : paper ? "Read and plan" : "Draft"}</button>
    </form>
    {#if rounds.length}
      <ol class="rounds" aria-label="Orac's tries">
        {#each rounds as r (r.round + (r.lookup ? "l" : "") + rounds.indexOf(r))}
          <li>{#if r.lookup}Read the catalogue{r.lookup.need.length ? `: ${r.lookup.need.join(", ")}` : ""}{r.lookup.find ? ` (searched "${r.lookup.find}")` : ""}{:else if r.problems.length}Try {r.round}: SSE found {r.problems.length} {r.problems.length === 1 ? "problem" : "problems"}, sent back{:else}Try {r.round}: passed every check{/if}</li>
        {/each}
      </ol>
    {/if}
    {#if plan && !result}
      {#if plan.passed[0]?.value}
        <article class="cand" data-orac-plan="ok">
          <h4>{plan.passed[0].made.title ?? "Orac's plan"} <span class="meta">· a plan in words · nothing drafted yet</span></h4>
          {#if plan.note}<p>{plan.note}</p>{/if}
          <pre class="plan">{planText(plan.passed[0].value)}</pre>
          <p class="meta">{plan.calls} {plan.calls === 1 ? "call" : "calls"}, {plan.tokens_in} tokens in, {plan.tokens_out} out{plan.usd !== null ? `, ${money(plan.usd)}` : ""}.</p>
          <button type="button" class="primary" onclick={draftPlan} disabled={running}>Draft it</button>
        </article>
      {:else}
        <p role="status" data-orac-plan="not">{plan.message}</p>
        {#each plan.failed as c, i (i)}<ul class="cand bad">{#each c.problems as p (p)}<li>{p}</li>{/each}</ul>{/each}
      {/if}
    {/if}
    {#if result}
      <p role="status" data-orac-result={result.ok ? "ok" : "not"}>{result.message} {result.calls} {result.calls === 1 ? "call" : "calls"}, {result.tokens_in} tokens in, {result.tokens_out} out{result.usd !== null ? `, ${money(result.usd)}` : ", price not given by the provider"}.</p>
      {#if result.note}<blockquote class="answer">{result.note}</blockquote>{/if}
      {#each result.passed as c, i (i)}
        <article class="cand" data-candidate="passed" data-kind={c.made.kind}>
          <h4>{c.made.title ?? c.made.kind} <span class="meta">· {c.made.kind} · passed SSE's checks</span></h4>
          <p>{c.value!.target.summary(c.value!.value)}</p>
          {#if c.made.rationale}<p class="meta">{c.made.rationale}</p>{/if}
          {#if accepted[i]}<p class="meta" role="status">{accepted[i]}</p>{:else}<button type="button" class="primary" onclick={() => accept(i)}>Accept</button>{/if}
        </article>
      {/each}
      {#each result.failed as c, i (i)}
        <article class="cand bad" data-candidate="failed" data-kind={c.made.kind}>
          <h4>{c.made.title ?? c.made.kind} <span class="meta">· {c.made.kind} · did not pass</span></h4>
          <ul>{#each c.problems as p (p)}<li>{p}</li>{/each}</ul>
        </article>
      {/each}
      {#if result.missing.length}
        <h4>What SSE does not have yet</h4>
        <ul>{#each result.missing as m (m.does)}<li>{m.does}{m.closest ? ` (closest: ${m.closest})` : ""}{m.why_not ? `: ${m.why_not}` : ""}</li>{/each}</ul>
      {/if}
    {/if}
  {/if}
</div>

<style>
  .rounds {
    font-size: 12px;
    color: var(--sse-ink-muted);
    padding-left: 18px;
  }
  .cand {
    border: var(--sse-line) solid var(--sse-rule);
    padding: 8px 10px;
    margin: 8px 0;
  }
  .cand.bad {
    border-style: dashed;
  }
  .plan {
    white-space: pre-wrap;
    font: inherit;
    font-size: 13px;
    margin: 4px 0 8px;
  }
  .paper details textarea {
    width: 100%;
  }
  .cand h4 {
    margin: 0 0 4px;
    font-weight: normal;
  }
</style>

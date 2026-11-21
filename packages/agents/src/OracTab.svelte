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

  export interface OracHost {
    targets: () => OracTarget[];
    catalogue: () => Entry[];
    /** What is open now (selected body, flow on the bench), as plain text. */
    here: () => string;
  }

  let { st, host, onrow }: { st: AgentsState; host?: OracHost; onrow: (r: LedgerRow) => void } = $props();

  let request = $state("");
  let running = $state(false);
  let rounds = $state<Round[]>([]);
  let result = $state<LoopResult<{ target: OracTarget; value: unknown }> | null>(null);
  let accepted = $state<Record<number, string>>({});

  const messages = $derived<ChatMessage[]>(host ? [{ role: "system", content: oracSystem(host.targets(), host.catalogue()) }, { role: "user", content: oracRequest(request, host.here()) }] : []);
  const est = $derived(estimate(st, "building", messages, 4000));
  const money = (x: number) => `$${x < 0.01 ? x.toFixed(4) : x.toFixed(2)}`;

  async function run(e: SubmitEvent) {
    e.preventDefault();
    if (!host || !request.trim()) return;
    running = true;
    rounds = [];
    result = null;
    accepted = {};
    const snap = JSON.parse(JSON.stringify(st)) as AgentsState;
    result = await runOrac({ request, here: host.here(), targets: host.targets(), catalogue: host.catalogue(), ask: asker(snap, "building", { onRow: onrow }), onRound: (r) => (rounds = [...rounds, r]) });
    running = false;
  }
  const DONE: Record<string, string> = {
    lens: "Saved with your lenses: pick it from the Lens menu.",
    style: "Kept with your styles: name it on a screen item, or tweak it on the Styles screen.",
    screen: "Opened on the players page on this device. Open it in the Screens composer to share it.",
    symbology: "Kept with your symbols: choose it on the Symbols screen.",
    data_view: "Applied to the campaign's player data: it is on the Player data screen now.",
    sifter: "Added to the campaign's sifters: the story finder and Minerva use it now.",
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
  <p>Orac drafts changes from a sentence: physics, lenses, styles, player screens, symbols, data views, programs (autopilots), story sifters, and new physics blocks as issues. SSE checks every draft and sends problems back to Orac, up to three times. Only drafts that pass can be accepted, and nothing changes until you accept.</p>
  {#if !host}
    <p class="hint" data-agent-off="orac">Orac works on the GM's chart room only.</p>
  {:else}
    {#if !linked(st, "building")}<p class="hint" data-agent-off="orac">No model is linked for Building. Link one under Link models; give it the strongest model you have.</p>{/if}
    <form class="ask" onsubmit={run}>
      <label>Ask Orac <textarea bind:value={request} rows="3" aria-label="Ask Orac" placeholder="Tidally locked worlds should keep more heat on their day side."></textarea></label>
      <p class="meta" data-orac-estimate={est.tokens_in}>First try: about {est.tokens_in} tokens in, up to {est.max_out} out{est.usd !== undefined ? `, up to ${money(est.usd)}` : ""}. Each fix sends the conversation again; up to four tries in all.{est.fits ? "" : " Too long for this model."}</p>
      <button type="submit" class="primary" disabled={running || !linked(st, "building") || !request.trim()}>{running ? "Orac is drafting…" : "Draft"}</button>
    </form>
    {#if rounds.length}
      <ol class="rounds" aria-label="Orac's tries">
        {#each rounds as r (r.round + (r.lookup ? "l" : "") + rounds.indexOf(r))}
          <li>{#if r.lookup}Read the catalogue{r.lookup.need.length ? `: ${r.lookup.need.join(", ")}` : ""}{r.lookup.find ? ` (searched "${r.lookup.find}")` : ""}{:else if r.problems.length}Try {r.round}: SSE found {r.problems.length} {r.problems.length === 1 ? "problem" : "problems"}, sent back{:else}Try {r.round}: passed every check{/if}</li>
        {/each}
      </ol>
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
  .cand h4 {
    margin: 0 0 4px;
    font-weight: normal;
  }
</style>

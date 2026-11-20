<!--
  Tars (A4): support from SSE's own help passages and the "why?" of what is
  selected. Every answer says where it came from. With no model linked he
  answers from the best passage alone; with one, the model's answer must cite
  a passage he was given and keep to its numbers, or he falls back.
-->
<script lang="ts">
  import { asker, estimate, linked, type AgentsState, type LedgerRow } from "./agents.ts";
  import { askTars, findPassages, tarsRequest, tarsSystem, SHELF, type Passage, type TarsAnswer } from "./tars.ts";

  export interface TarsHost {
    /** The "why?" passages for what is selected now (the kernel's explanation bundles). */
    why: () => Promise<Passage[]>;
    here: () => string;
  }

  let { st, host, onrow, onhumour }: { st: AgentsState; host?: TarsHost; onrow: (r: LedgerRow) => void; onhumour: (h: number) => void } = $props();

  let question = $state("");
  // svelte-ignore state_referenced_locally
  let humour = $state(st.humour);
  let asking = $state(false);
  let answer = $state<TarsAnswer | null>(null);
  let note = $state("");

  const est = $derived(estimate(st, "support", [{ role: "system", content: tarsSystem(humour) }, { role: "user", content: tarsRequest(question, findPassages(question, SHELF), host?.here() ?? "") }], 600));
  const money = (x: number) => `$${x < 0.01 ? x.toFixed(4) : x.toFixed(2)}`;

  async function ask(e: SubmitEvent) {
    e.preventDefault();
    if (!question.trim()) return;
    asking = true;
    note = "";
    const passages = [...SHELF, ...((await host?.why()) ?? [])];
    const snap = JSON.parse(JSON.stringify(st)) as AgentsState;
    const r = await askTars({ question, passages, here: host?.here() ?? "", humour, ...(linked(st, "support") ? { ask: asker(snap, "support", { max_out: 600, temperature: 0.5, onRow: onrow }) } : {}) });
    answer = r.answer;
    asking = false;
    if (r.run && !r.run.ok) note = r.run.calls ? `The model's answer did not pass its checks (${r.run.message}); this is Tars's answer from the passage itself.` : r.run.message;
    else if (r.run) note = `${r.run.tokens_in} tokens in, ${r.run.tokens_out} out${r.run.usd !== null ? `, ${money(r.run.usd)}` : ", price not given by the provider"}.`;
  }
</script>

<div role="tabpanel" aria-label="Tars">
  <p>Tars answers questions about SSE and about why a value of the selected body is what it is. Read only, allowed to be funny, and every answer says where it came from.</p>
  {#if !linked(st, "support")}
    <p class="hint" data-agent-off="tars">No model is linked for Support, so Tars answers from SSE's own help passages alone. Link a model under Link models for fuller answers.</p>
  {/if}
  <label class="slider">Humour <input type="range" min="0" max="1" step="0.05" bind:value={humour} onchange={() => onhumour(humour)} aria-label="Humour" /></label>
  <form class="ask" onsubmit={ask}>
    <label>Ask Tars <textarea bind:value={question} rows="3" aria-label="Ask Tars"></textarea></label>
    {#if linked(st, "support")}<p class="meta" data-estimate={est.tokens_in}>Before sending: about {est.tokens_in} tokens in, up to {est.max_out} out{est.usd !== undefined ? `, up to ${money(est.usd)}` : ""}{est.context ? `, the model fits ${est.context}` : ""}.{est.fits ? "" : " Too long for this model."}</p>{/if}
    <button type="submit" class="primary" disabled={asking || !question.trim()}>{asking ? "Asking…" : "Ask"}</button>
  </form>
  {#if answer}
    <blockquote class="answer" data-answer data-by={answer.by}>{answer.words}</blockquote>
    {#if answer.sources.length}
      <p class="meta" data-sources>From: {#each answer.sources as p, i (p.id)}{i ? "; " : ""}<span data-source={p.id}>{p.title}</span>{p.place ? ` (${p.place})` : ""}{/each}{answer.by === "rules" ? " · Tars's own pages" : ""}</p>
      {#each answer.sources.flatMap((p) => p.references ?? []) as r (r.text)}
        <p class="meta">{r.text}{#if r.href} <a href={r.href} target="_blank" rel="noopener">open</a>{/if}</p>
      {/each}
    {/if}
  {/if}
  {#if note}<p class="meta" role="status">{note}</p>{/if}
</div>

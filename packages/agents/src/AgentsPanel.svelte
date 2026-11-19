<!--
  Agents (A1): link your own models, keep the keys in this browser, and meet
  Tars, Orac and Minerva. Everything works with no key; each agent says what
  it is waiting for. Every run shows its estimate first and its cost after.
-->
<script lang="ts">
  import { JOBS, PROVIDER_KINDS, chat, memoryStore, estimate, linked, listModels, loadAgents, newProvider, readLoaded, saveAgents, tarsSystem, type AgentsState, type ChatMessage, type JobId, type ModelInfo, type Provider, type ProviderKind } from "./agents.ts";

  let { context = "", onchange }: { context?: string; onchange?: (s: AgentsState) => void } = $props();

  // Settings in "sse4.agents", each key in its own entry (Ovid saves its links without keys).
  const store = (() => {
    try {
      return localStorage ?? memoryStore();
    } catch {
      return memoryStore();
    }
  })();
  let st = $state<AgentsState>(loadAgents(store));
  function save(removed: string[] = []) {
    const snap = $state.snapshot(st) as AgentsState;
    saveAgents(store, snap, removed);
    onchange?.(snap);
  }

  let tab = $state<"link" | "tars" | "orac" | "minerva">("link");

  // Adding a provider.
  let kind = $state<ProviderKind>("openrouter");
  let label = $state("");
  let base = $state(PROVIDER_KINDS.openrouter.base);
  let key = $state("");
  let problem = $state("");
  $effect(() => {
    base = PROVIDER_KINDS[kind].base;
    label = PROVIDER_KINDS[kind].label;
  });
  let models = $state<Record<string, ModelInfo[]>>({});
  let checking = $state<string | null>(null);
  async function check(p: Provider) {
    checking = p.id;
    const r = await listModels(p);
    checking = null;
    if (r.ok) {
      models[p.id] = r.models;
      problem = r.models.length ? "" : `${p.label} answered but lists no models.`;
    } else problem = `${p.label}: ${r.message}`;
  }
  async function add(e: SubmitEvent) {
    e.preventDefault();
    const made = newProvider($state.snapshot(st) as AgentsState, kind, label, base, key);
    if ("problem" in made) return void (problem = made.problem);
    const p: Provider = made;
    st.providers.push(p);
    key = "";
    save();
    await check(p);
  }
  function remove(id: string) {
    st.providers = st.providers.filter((p) => p.id !== id);
    for (const j of Object.keys(st.links) as JobId[]) if (st.links[j]?.provider === id) delete st.links[j];
    save([id]);
  }
  async function link(job: JobId, provider: string, model: string) {
    if (!provider || !model) {
      delete st.links[job];
    } else {
      const m = models[provider]?.find((x) => x.id === model);
      st.links[job] = { provider, model, ...(m?.context ? { context: m.context } : {}), ...(m?.price ? { price: m.price } : {}), ...(m?.varies ? { varies: true as const } : {}) };
      // A local model plans with what it is loaded with, never the most it could take.
      const p = st.providers.find((x) => x.id === provider);
      const loaded = p ? await readLoaded($state.snapshot(p) as Provider, model) : null;
      if (loaded && st.links[job]?.model === model) st.links[job]!.loaded = loaded;
    }
    save();
  }
  const today = new Date().toISOString().slice(0, 10);
  const spent = $derived(st.ledger.filter((r) => r.day === today).reduce((a, r) => a + (r.usd ?? 0), 0));
  const runs = $derived(st.ledger.filter((r) => r.day === today).length);
  // Money a provider did not report and no listed price covers stays unknown, never counted as zero.
  const unpriced = $derived(st.ledger.filter((r) => r.day === today && r.ok !== false && r.usd === undefined).length);

  // Tars.
  let question = $state("");
  let answer = $state("");
  let asking = $state(false);
  let note = $state("");
  const messages = $derived<ChatMessage[]>([
    { role: "system", content: tarsSystem(st.humour, context || "(nothing selected)") },
    { role: "user", content: question },
  ]);
  const est = $derived(estimate(st, "support", messages, 600));
  async function ask(e: SubmitEvent) {
    e.preventDefault();
    if (!question.trim()) return;
    asking = true;
    note = "";
    const r = await chat($state.snapshot(st) as AgentsState, "support", messages);
    asking = false;
    if (!r.ok) {
      if (r.row) (st.ledger.push(r.row), save());
      return void (note = r.message);
    }
    answer = r.text;
    st.ledger.push(r.row);
    save();
    note = `${r.row.tokens_in} tokens in, ${r.row.tokens_out} out${r.row.usd !== undefined ? `, $${r.row.usd.toFixed(4)}` : ", price not given by the provider"}.`;
  }
  const money = (x: number) => `$${x < 0.01 ? x.toFixed(4) : x.toFixed(2)}`;
</script>

<section class="ag" aria-labelledby="ag-title">
  <h2 id="ag-title">Agents</h2>
  <p class="hint">Bring your own key. Keys stay in this browser: they go only to the provider you name, never to SSE's servers or to players' devices. The app works fully with no agent linked.</p>
  <div class="tabs" role="tablist" aria-label="Agents">
    {#each [["link", "Link models"], ["tars", "Tars"], ["orac", "Orac"], ["minerva", "Minerva"]] as [t, name] (t)}
      <button type="button" role="tab" aria-selected={tab === t} onclick={() => (tab = t as typeof tab)}>{name}</button>
    {/each}
  </div>

  {#if tab === "link"}
    <div role="tabpanel" aria-label="Link models">
      {#if st.providers.length}
        <ul class="providers">
          {#each st.providers as p (p.id)}
            <li data-provider={p.id}>
              <span class="name">{p.label}</span>
              <span class="meta">{p.base}{p.key ? " · key kept in this browser only" : ""}</span>
              <span class="row">
                <button type="button" class="linkbtn" onclick={() => check(p)} disabled={checking === p.id}>{checking === p.id ? "Checking…" : models[p.id] ? `${models[p.id]!.length} models` : "Check"}</button>
                <button type="button" class="linkbtn" onclick={() => remove(p.id)}>Remove</button>
              </span>
            </li>
          {/each}
        </ul>
      {/if}
      <form class="add" onsubmit={add}>
        <h3>Add a provider</h3>
        <label>Kind <select bind:value={kind} aria-label="Provider kind">{#each Object.entries(PROVIDER_KINDS) as [k, v] (k)}<option value={k}>{v.label}</option>{/each}</select></label>
        <p class="hint">{PROVIDER_KINDS[kind].note}</p>
        <label>Name <input bind:value={label} aria-label="Provider name" /></label>
        <label>Address <input bind:value={base} aria-label="Provider address" placeholder="https://…/v1" /></label>
        {#if PROVIDER_KINDS[kind].key !== "none"}
          <label>Key <input type="password" bind:value={key} aria-label="Provider key" autocomplete="off" placeholder={PROVIDER_KINDS[kind].key === "optional" ? "if it needs one" : ""} /></label>
        {/if}
        <button type="submit" class="primary">Add and check</button>
      </form>

      <h3>Jobs</h3>
      <p class="hint">Each job links one model. Measured recommendations per job: not measured yet.</p>
      <table class="jobs">
        <tbody>
          {#each Object.entries(JOBS) as [j, info] (j)}
            {@const l = st.links[j as JobId]}
            <tr data-job={j}>
              <th scope="row">{info.label}{info.agent ? ` (${info.agent})` : ""}</th>
              <td>
                <select aria-label="{info.label} provider" value={l?.provider ?? ""} onchange={(e) => link(j as JobId, e.currentTarget.value, models[e.currentTarget.value]?.[0]?.id ?? "")}>
                  <option value="">Not linked</option>
                  {#each st.providers as p (p.id)}<option value={p.id}>{p.label}</option>{/each}
                </select>
              </td>
              <td>
                {#if l}
                  <input aria-label="{info.label} model" list="models-{l.provider}" value={l.model} onchange={(e) => link(j as JobId, l.provider, e.currentTarget.value)} />
                  <datalist id="models-{l.provider}">{#each models[l.provider] ?? [] as m (m.id)}<option value={m.id}></option>{/each}</datalist>
                {/if}
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
      <p class="row">
        <label>Daily cap, US dollars <input type="number" min="0" step="0.5" value={st.cap_usd ?? ""} aria-label="Daily cap" onchange={(e) => ((st.cap_usd = e.currentTarget.value === "" ? null : Number(e.currentTarget.value)), save())} /></label>
        <span class="meta" data-spent={spent}>Today: {runs} {runs === 1 ? "run" : "runs"}, {money(spent)}{unpriced ? ` (and ${unpriced} with no price given)` : ""}</span>
      </p>
      {#if problem}<p class="problem" role="alert">{problem}</p>{/if}
    </div>
  {:else if tab === "tars"}
    <div role="tabpanel" aria-label="Tars">
      <p>Tars answers questions about SSE and the body you have selected. Read only, and allowed to be funny.</p>
      {#if !linked(st, "support")}
        <p class="hint" data-agent-off="tars">No model is linked for Support. Link one under Link models; until then Tars stays quiet and nothing else changes.</p>
      {/if}
      <label class="slider">Humour <input type="range" min="0" max="1" step="0.05" bind:value={st.humour} onchange={() => save()} aria-label="Humour" /></label>
      <form class="ask" onsubmit={ask}>
        <label>Ask Tars <textarea bind:value={question} rows="3" aria-label="Ask Tars"></textarea></label>
        <p class="meta" data-estimate={est.tokens_in}>Before sending: about {est.tokens_in} tokens in, up to {est.max_out} out{est.usd !== undefined ? `, up to ${money(est.usd)}` : ""}{est.context ? `, the model fits ${est.context}` : ""}.{est.fits ? "" : " Too long for this model."}</p>
        <button type="submit" class="primary" disabled={asking || !linked(st, "support") || !question.trim()}>{asking ? "Asking…" : "Ask"}</button>
      </form>
      {#if answer}<blockquote class="answer" data-answer>{answer}</blockquote>{/if}
      {#if note}<p class="meta" role="status">{note}</p>{/if}
    </div>
  {:else if tab === "orac"}
    <div role="tabpanel" aria-label="Orac">
      <p>{JOBS.building.what}</p>
      <p class="hint" data-agent-off="orac">{linked(st, "building") ? `Linked: ${st.links.building!.model}.` : "No model is linked for Building."} Orac's change-request loop (plan, draft, check, fix up to three times) arrives with package A2, built on Ovid's agent loop once its source is read. Until then the change requests on the Physics screen are fixtures.</p>
    </div>
  {:else}
    <div role="tabpanel" aria-label="Minerva">
      <p>{JOBS.describing.what}</p>
      <p class="hint" data-agent-off="minerva">{linked(st, "describing") ? `Linked: ${st.links.describing!.model}.` : "No model is linked for Describing."} Minerva's narrative arrives with package A3, after the story finder (sifter hits) she reads from.</p>
    </div>
  {/if}
</section>

<style>
  .ag {
    font-family: var(--sse-font-body);
    font-size: 14px;
    color: var(--sse-ink);
    max-width: 640px;
  }
  h2,
  h3 {
    margin: 0 0 8px;
    font-family: var(--sse-font-display);
    font-weight: normal;
  }
  h3 {
    margin-top: 16px;
    font-size: 17px;
  }
  .hint,
  .meta {
    color: var(--sse-ink-muted);
    font-size: 12px;
  }
  .tabs {
    display: flex;
    border-bottom: var(--sse-line) solid var(--sse-rule);
    margin: 10px 0 12px;
  }
  .tabs button {
    border: 0;
    background: transparent;
    min-height: 34px;
    padding: 0 12px;
    font: inherit;
    color: var(--sse-ink-muted);
    cursor: pointer;
  }
  .tabs button[aria-selected="true"] {
    color: var(--sse-ink);
    box-shadow: inset 0 -2px 0 var(--sse-accent);
  }
  .providers {
    list-style: none;
    margin: 0;
    padding: 0;
  }
  .providers li {
    display: grid;
    gap: 2px;
    padding: 6px 0;
    border-bottom: var(--sse-line) solid var(--sse-rule);
  }
  .name {
    font-family: var(--sse-font-display);
    font-style: italic;
    font-size: 16px;
  }
  .row {
    display: flex;
    gap: 14px;
    align-items: center;
    flex-wrap: wrap;
  }
  .add,
  .ask {
    display: grid;
    gap: 6px;
  }
  input:not([type="range"]),
  select,
  textarea {
    display: block;
    width: 100%;
    margin-top: 3px;
    min-height: 34px;
    padding: 0 8px;
    border: var(--sse-line) solid var(--sse-ink);
    background: var(--sse-ground);
    color: var(--sse-ink);
    font: inherit;
    box-sizing: border-box;
  }
  textarea {
    padding: 6px 8px;
  }
  input[type="range"] {
    accent-color: var(--sse-accent);
  }
  .jobs {
    width: 100%;
    border-collapse: collapse;
  }
  .jobs th,
  .jobs td {
    text-align: left;
    font-weight: normal;
    padding: 4px 8px 4px 0;
  }
  .linkbtn {
    color: var(--sse-accent);
    font: inherit;
    background: none;
    border: 0;
    padding: 0;
    cursor: pointer;
    text-decoration: underline;
  }
  .primary {
    justify-self: start;
    height: 38px;
    padding: 0 16px;
    border: var(--sse-line) solid var(--sse-ink);
    background: var(--sse-ink);
    color: var(--sse-on-ink);
    font: inherit;
    cursor: pointer;
  }
  .primary:disabled {
    opacity: 0.45;
    cursor: default;
  }
  .answer {
    margin: 10px 0;
    padding: 8px 12px;
    border-left: 2px solid var(--sse-accent);
    font-family: var(--sse-font-display);
    font-size: 15px;
    line-height: 1.5;
    white-space: pre-wrap;
  }
  .problem {
    color: var(--sse-accent-ink);
  }
</style>

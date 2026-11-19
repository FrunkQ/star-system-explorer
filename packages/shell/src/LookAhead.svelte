<!--
  Look ahead (U4 part 2, T2): age the system the chart shows, on a branch of
  its own. The chart and card follow a scrubber through the snapshots; the
  look-ahead's events go on the ruler as future events. Nothing moves the
  main line until the GM keeps it (as history, or moving "now" too).
  Ages past the predictability horizon are one plausible future and say so.
-->
<script lang="ts">
  import { KIND_WORDS, fmtAge, specFromScene, subjectNames, type LookedAhead, type SystemScene, type ViewSession } from "../../ui-contract/src/index.ts";

  let {
    session,
    scene,
    ahead = $bindable(null),
    at = $bindable(0),
    onkept,
  }: { session: ViewSession; scene: SystemScene; ahead?: LookedAhead | null; at?: number; onkept: (note: string, now_myr: number | null, state_hash: string) => void } = $props();

  let open = $state(false);
  let busy = $state(false);
  let problem = $state<string | null>(null);
  const prepared = $derived(specFromScene(scene));
  const nowGyr = $derived(scene.now_yr / 1e9);
  let target = $state(0);
  $effect(() => {
    if (!target) target = Math.round((nowGyr + 5) * 100) / 100;
  });
  let why = $state<number | null>(null);
  const id = $props.id();

  async function run() {
    if ("error" in prepared) return void (problem = prepared.error);
    busy = true;
    problem = null;
    if (ahead) await session.discardBranch(ahead.branch);
    const r = await session.lookAhead(prepared.spec, target * 1000);
    busy = false;
    if (!r.ok) return void ((problem = r.code === "bad_target" ? `Pick an age after now (${fmtAge(scene.now_yr / 1e6)}).` : r.message), (ahead = null));
    ahead = r.result;
    at = 0;
  }
  async function drop() {
    if (ahead) await session.discardBranch(ahead.branch);
    ahead = null;
    at = 0;
  }
  async function keep(moveNow: boolean) {
    if (!ahead) return;
    const snap = ahead.snapshots[at];
    const r = await session.commitLookAhead(ahead.branch, moveNow && snap ? snap.age_myr : undefined);
    if (!r.ok) return void (problem = r.message);
    const n = ahead.events.length;
    onkept(moveNow && snap ? `Kept: ${n} future ${n === 1 ? "event" : "events"} on the chronicle, and now is ${fmtAge(snap.age_myr)}.` : `Kept: ${n} future ${n === 1 ? "event" : "events"} on the chronicle.`, moveNow && snap ? snap.age_myr : null, r.state_hash);
    ahead = null;
    at = 0;
    open = false;
  }
  const snap = $derived(ahead?.snapshots[at]);
  const beyond = $derived(!!ahead && !!snap && snap.age_myr > ahead.now_myr + ahead.horizon_myr);
  function jump(age: number) {
    if (!ahead) return;
    let best = 0;
    ahead.snapshots.forEach((s, i) => Math.abs(s.age_myr - age) < Math.abs(ahead!.snapshots[best]!.age_myr - age) && (best = i));
    at = best;
  }
</script>

<svelte:window onkeydown={(e) => open && e.key === "Escape" && !ahead && (open = false)} />

<span class="ahead" data-ahead={ahead ? "open" : undefined} data-ahead-at={snap?.age_myr}>
  <button type="button" class="linkbtn quiet" aria-expanded={open} aria-controls="{id}-panel" onclick={() => (open = !open)} title="Age this system forward on a branch; nothing changes until you keep it">Look ahead…</button>
  {#if open}
    <div class="panel" id="{id}-panel" role="group" aria-label="Look ahead">
      <div class="row">
        <label>Age the star to (Gyr)<input type="number" step="0.5" min={Math.ceil(nowGyr * 10) / 10} bind:value={target} aria-label="Target age in Gyr" /></label>
        <span class="presets">
          {#each [1, 5, 20] as d (d)}<button type="button" class="chip" onclick={() => (target = Math.round((nowGyr + d) * 100) / 100)}>+{d} Gyr</button>{/each}
        </span>
        <button type="button" class="go" onclick={run} disabled={busy || "error" in prepared}>{busy ? "Ageing…" : ahead ? "Run again" : "Look ahead"}</button>
      </div>
      <p class="note">
        Now {fmtAge(scene.now_yr / 1e6)}. One run of the ageing model (medium cost, about a second). {#if !("error" in prepared)}Assumed: {prepared.assumed.join("; ")}.{/if}
      </p>
      {#if problem}<p class="problem" role="alert">{problem}</p>{/if}
      {#if ahead && snap}
        <label class="scrub">
          <span>Showing <b>{fmtAge(snap.age_myr)}</b>: {snap.star.phase}, {snap.star.colour}{#if beyond}<span class="pf">one plausible future</span>{/if}</span>
          <input type="range" min="0" max={ahead.snapshots.length - 1} step="1" bind:value={at} aria-label="Age shown" aria-valuetext={fmtAge(snap.age_myr)} />
        </label>
        <p class="note">Beyond {fmtAge(ahead.now_myr + ahead.horizon_myr)} the details are one plausible future, not a prediction: small differences now grow too large to follow.</p>
        <ol class="events" aria-label="What happens">
          {#each ahead.events as e, i (i)}
            <li data-kind={e.kind}>
              <button type="button" class="linkbtn" onclick={() => jump(e.age_myr)}>{fmtAge(e.age_myr)}</button>
              {KIND_WORDS[e.kind] ?? e.kind}{#if e.kind === "engulfed" || e.kind === "oceans_lost"}: {subjectNames(scene, e).filter((n) => n !== scene.subjects[0]?.label).join(", ")}{/if}
              {#if e.plausible_future}<span class="pf">plausible</span>{/if}
              <button type="button" class="linkbtn" aria-label="Why: {KIND_WORDS[e.kind] ?? e.kind} at {fmtAge(e.age_myr)}" onclick={() => (why = why === i ? null : i)}>why?</button>
              {#if why === i}<p class="why">{e.why}</p>{/if}
            </li>
          {/each}
        </ol>
        <div class="row actions">
          <button type="button" class="linkbtn" onclick={drop}>Throw away</button>
          <button type="button" class="linkbtn" onclick={() => keep(false)} title="The future events join the chronicle; now stays where it is">Keep as history</button>
          <button type="button" class="linkbtn" onclick={() => keep(true)} title="Keep it and move the system's now to the age shown">Keep and move now to {fmtAge(snap.age_myr)}</button>
        </div>
      {/if}
    </div>
  {/if}
</span>

<style>
  .ahead {
    position: relative;
    display: inline-flex;
  }
  .linkbtn {
    font: inherit;
    color: var(--sse-accent);
    background: none;
    border: 0;
    padding: 0;
    text-decoration: underline;
    cursor: pointer;
  }
  .quiet {
    color: var(--sse-ink-muted);
  }
  .panel {
    position: fixed;
    bottom: 44px;
    left: 16px;
    z-index: 10;
    width: min(460px, calc(100vw - 32px));
    max-height: 70vh;
    overflow: auto;
    display: grid;
    gap: 8px;
    padding: 12px 14px;
    background: var(--sse-ground-raised, var(--sse-ground, Canvas));
    color: var(--sse-ink, CanvasText);
    border: var(--sse-line, 1px) solid var(--sse-ink, CanvasText);
    box-shadow: 4px 4px 0 var(--sse-ink, CanvasText);
    font-family: var(--sse-font-body, inherit);
    font-size: 13px;
    text-align: left;
    white-space: normal;
  }
  .row {
    display: flex;
    flex-wrap: wrap;
    gap: 8px 12px;
    align-items: end;
  }
  label {
    display: grid;
    gap: 2px;
  }
  input[type="number"] {
    font: inherit;
    width: 7em;
    padding: 3px 6px;
    background: transparent;
    color: inherit;
    border: 1px solid var(--sse-rule, currentColor);
  }
  .presets {
    display: flex;
    gap: 4px;
  }
  .chip {
    font: inherit;
    font-size: 12px;
    padding: 2px 6px;
    background: transparent;
    color: inherit;
    border: 1px solid var(--sse-rule, currentColor);
    cursor: pointer;
  }
  .go {
    font: inherit;
    padding: 4px 12px;
    background: var(--sse-ink);
    color: var(--sse-on-ink);
    border: 0;
    cursor: pointer;
  }
  .note {
    margin: 0;
    font-size: 12px;
    color: var(--sse-ink-muted);
  }
  .problem {
    margin: 0;
    color: var(--sse-accent-ink, var(--sse-accent));
  }
  .scrub input {
    width: 100%;
    accent-color: var(--sse-accent);
  }
  .pf {
    margin-left: 6px;
    font-size: 10px;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    border: 1px dashed currentColor;
    padding: 0 4px;
  }
  .events {
    margin: 0;
    padding-left: 18px;
    display: grid;
    gap: 3px;
  }
  .why {
    margin: 2px 0 6px;
    font-size: 12px;
    line-height: 1.45;
  }
  .actions {
    justify-content: space-between;
  }
</style>

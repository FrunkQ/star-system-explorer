<!--
  The journey player (grand zoom GZ3 and GZ8, first part; design/SSE4-grand-zoom-design.md sections 5.3 to 5.7). It
  plays a journey laid out by journey.ts: one picture always aimed at the target, the width shrinking at a steady rate
  in powers of ten, each level drawn by its own view (the same reads its screen makes), and at a change of level the
  two views stacked as two canvases, the outgoing one fading over the incoming one while both keep zooming (fudge
  GZ-F1). The place line across the top is the progress bar; Go on (Space or Enter) leaves a stop, Back up returns to
  the one before, Escape leaves the journey. The journey's time is camera travel, not the campaign's clock: the
  universe does not move while it plays (5.7). If a level below has not finished reading when its seam comes, the
  journey holds just before it with one quiet line, never half-faded (5.4).

  Levels drawn so far: the cosmic web and the galaxies nearby. The galaxy, local stars, system, world and site follow
  in later parts; a level not drawn here is passed over and says why on the place line.
-->
<script lang="ts">
  import { onDestroy } from "svelte";
  import Nearby from "../../../views/cluster-map/src/Nearby.svelte";
  import JourneyWeb from "./JourneyWeb.svelte";
  import { SPACE_CSS } from "../../../renderer/src/index.ts";
  import { ZOOM_IN_FROM, journeyAt, journeyHold, journeyPlan, journeySteps, nearbyAim, readyHold, stopBefore, widthLine, RUNG_WORDS, NEARBY_REACHES, sayNumber, type Approach, type JourneyCamera, type LadderRung, type ViewSession } from "../../../ui-contract/src/index.ts";

  let { session, to = "cluster", approach = ZOOM_IN_FROM, onclose }: { session: ViewSession; to?: LadderRung; approach?: Approach; onclose: () => void } = $props();

  /** The levels this player can draw today. */
  const DRAWS: readonly LadderRung[] = ["web", "cluster"];
  const reduced = typeof matchMedia === "function" && matchMedia("(prefers-reduced-motion: reduce)").matches;
  const planned = $derived(journeyPlan({ to: DRAWS.includes(to) ? to : "cluster", approach: { ...approach, from: "web" }, drawn: (r) => DRAWS.includes(r), reduced_motion: reduced }));
  const plan = $derived(planned.ok ? planned.plan : null);

  let t = $state(0);
  let released = $state(0);
  let playing = $state(true);
  let ready = $state<boolean[]>([]);
  let notes = $state<string[]>([]);
  const readyOf = (i: number) => (r: { ok: boolean; note?: string }) => {
    const next = [...ready];
    next[i] = r.ok;
    ready = next;
    if (r.note) notes = [...notes, r.note];
    if (!r.ok) failed = r.note ?? "a level could not be read";
  };
  let failed = $state<string | null>(null);

  // The clock of the journey: real seconds, camera travel only (never the campaign's clock).
  let raf = 0;
  let last = 0;
  let waiting = $state<number | null>(null);
  function tick(now: number) {
    if (plan && playing) {
      const dt = last ? Math.min(0.1, (now - last) / 1000) : 0;
      const want = journeyHold(plan, t + dt, released);
      const held = readyHold(plan, want, ready);
      waiting = held.waiting;
      t = held.t;
    }
    last = now;
    raf = requestAnimationFrame(tick);
  }
  raf = requestAnimationFrame(tick);
  onDestroy(() => cancelAnimationFrame(raf));

  const frame = $derived(plan ? journeyAt(plan, t) : null);
  const steps = $derived(plan && frame ? journeySteps(plan, frame) : []);
  const camOf = (entry: number): JourneyCamera | null => frame?.views.find((v) => v.entry === entry) ?? null;
  const shown = (entry: number) => !!camOf(entry);
  const layerOrder = (entry: number) => frame?.views.findIndex((v) => v.entry === entry) ?? -1;
  // A view keeps its last camera while hidden, so it is already where the journey will want it.
  const kept: Record<number, JourneyCamera> = {};
  const camFor = (entry: number): JourneyCamera | null => {
    const c = camOf(entry);
    if (c) kept[entry] = c;
    return c ?? kept[entry] ?? null;
  };
  // The web past the galaxies nearby's reach stays behind them (so the level below is as filled in as the one above):
  // from the seam on, the web is drawn at the bottom with full light beyond that reach, and only the light within it
  // fades as the catalogue's galaxies fade in over it. Same cross-fade, the other way up (fudge GZ-F1).
  const NEARBY_MPC = NEARBY_REACHES.find((r) => r.id === "neighbourhood")!.mpc;
  const clusterAt = $derived(plan ? plan.path.findIndex((p) => p.rung === "cluster") : -1);
  const webBehind = $derived(clusterAt > 0 && plan?.path[clusterAt - 1]?.rung === "web" && shown(clusterAt));
  const layerShown = (i: number) => shown(i) || (webBehind && i === clusterAt - 1);
  const layerOpacity = (i: number) => {
    if (webBehind && i === clusterAt - 1) return 1;
    if (webBehind && i === clusterAt) return 1 - (camOf(clusterAt - 1)?.opacity ?? 0);
    return camOf(i)?.opacity ?? 0;
  };
  const layerZ = (i: number) => (webBehind && i === clusterAt - 1 ? 0 : layerOrder(i) + 1);
  const atStop = $derived(!!frame?.stop && released <= (plan?.stops.filter((s) => s.hold_s === null).indexOf(frame.stop!) ?? -1));
  const done = $derived(!!frame?.done);
  const width = $derived(frame ? widthLine(frame.width_m) : null);

  let box = $state(0);
  let boxH = $state(0);
  const side = $derived(Math.max(0, Math.min(box, boxH)));

  function goOn() {
    if (!plan) return;
    if (done) return void ((t = 0), (released = 0), (playing = true));
    if (atStop) released += 1;
    playing = true;
  }
  function backUp() {
    if (!plan) return;
    const b = stopBefore(plan, t);
    t = b.t;
    released = b.released;
  }
  function jumpTo(entry: number) {
    if (!plan) return;
    const e = plan.path[entry]!;
    const stop = plan.stops.find((s) => s.entry === entry);
    t = stop ? stop.t : entry === 0 ? 0 : e.t_in + (e.t_out - e.t_in) / 2;
    released = plan.stops.filter((s) => s.hold_s === null && s.t < t - 1e-6).length;
  }
  function key(e: KeyboardEvent) {
    if (e.key === "Escape") return void (e.preventDefault(), onclose());
    if (e.key === " " || e.key === "Enter") {
      if ((e.target as HTMLElement).closest("button")) return;
      e.preventDefault();
      goOn();
    }
  }
  const readingWords = (entry: number) => `Reading ${RUNG_WORDS[plan!.path[entry]!.rung].toLowerCase()}…`;
</script>

<svelte:window onkeydown={key} />

<section class="journey" aria-label="Zoom in from the cosmic web" data-journey-player data-journey-t={sayNumber(t, 3, { decimals: 2 })} data-journey-rung={frame?.rung ?? ""} data-journey-waiting={waiting ?? ""} data-journey-done={done ? "" : undefined}>
  <nav class="line" aria-label="Where the journey is">
    {#each steps as s (s.rung)}
      <button
        type="button"
        class="step {s.state}"
        disabled={s.entry === undefined}
        aria-current={s.state === "current" ? "step" : undefined}
        title={s.why ?? (s.entry !== undefined ? `Go to ${s.words}` : `${s.words}: not on this journey`)}
        onclick={() => s.entry !== undefined && jumpTo(s.entry)}
        data-journey-step={s.rung}
        data-state={s.state}>{s.words}</button
      >
    {/each}
    <span class="gap"></span>
    <button type="button" class="quiet" onclick={backUp} data-journey-back>Back up</button>
    <button type="button" class="go" onclick={goOn} data-journey-go>{done ? "Again" : atStop ? (t === 0 ? "Zoom in" : "Go on") : playing ? "Zooming…" : "Go on"}</button>
    <button type="button" class="quiet" onclick={onclose} aria-label="Leave the journey" data-journey-leave>Leave</button>
  </nav>

  <div class="room" bind:clientWidth={box} bind:clientHeight={boxH}>
    {#if !planned.ok}
      <p class="say" role="alert">{planned.why}</p>
    {:else if plan}
      <div class="square" style:width="{side}px" style:height="{side}px" style:background={SPACE_CSS}>
        {#each plan.path as e, i (i)}
          <div class="layer" class:hidden={!layerShown(i)} style:opacity={layerOpacity(i)} style:z-index={layerZ(i)} data-journey-layer={e.rung} data-ready={ready[i] ? "" : undefined}>
            {#if e.rung === "web"}
              <JourneyWeb {session} cam={webBehind ? (camOf(i + 1) ?? camFor(i)) : camFor(i)} onready={readyOf(i)} inner={webBehind ? { radius_mpc: NEARBY_MPC, alpha: camOf(i)?.opacity ?? 0 } : null} />
            {:else if e.rung === "cluster"}
              {@const c = camFor(i)}
              <Nearby {session} seen="free" scale="real" hud={false} look="true" onchart={() => {}} aim={c ? nearbyAim(c, 560) : nearbyAim({ width_m: e.stop_m, eye_dir: [0.85, 0, 0.52] }, 560)} onread={(ok) => readyOf(i)({ ok, note: ok ? "" : "the galaxies nearby could not be read" })} />
            {/if}
          </div>
        {/each}
        {#if frame?.stop && atStop}
          <div class="caption" data-journey-caption>
            <b>{RUNG_WORDS[frame.rung]}</b>
            <span>{t === 0 ? "Press Zoom in, or Space, to travel down to the galaxies around our own." : "Press Go on to travel on."}</span>
          </div>
        {/if}
        {#if waiting !== null}<p class="reading" data-journey-reading>{readingWords(waiting)}</p>{/if}
        {#if width && (frame?.width_line || atStop)}
          <p class="width" data-journey-width={width.exp}>{width.words} · {width.power}</p>
        {/if}
      </div>
    {/if}
    {#if failed}<p class="say" role="alert">{failed}</p>{/if}
  </div>
</section>

<style>
  .journey {
    position: fixed;
    inset: 0;
    z-index: 60;
    display: flex;
    flex-direction: column;
    background: var(--sse-ground);
    color: var(--sse-ink);
  }
  .line {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 4px 2px;
    padding: 8px 12px;
    border-bottom: 1px solid var(--sse-rule);
  }
  .step {
    font: inherit;
    min-height: 34px;
    padding: 0 10px;
    border: 0;
    border-bottom: 2px solid transparent;
    background: transparent;
    color: var(--sse-ink);
    cursor: pointer;
  }
  .step.passed {
    color: var(--sse-ink);
  }
  .step.current {
    color: var(--sse-accent);
    border-bottom-color: var(--sse-accent);
  }
  .step.coming,
  .step.held,
  .step.off {
    color: var(--sse-ink-faint, var(--sse-ink-muted));
  }
  .step.held {
    font-style: italic;
  }
  .step:disabled {
    cursor: default;
  }
  .gap {
    flex: 1;
  }
  .go,
  .quiet {
    font: inherit;
    min-height: 34px;
    padding: 0 12px;
    border: 1px solid var(--sse-ink);
    cursor: pointer;
  }
  .go {
    background: var(--sse-accent);
    border-color: var(--sse-accent);
    color: var(--sse-ground);
  }
  .quiet {
    background: transparent;
    color: var(--sse-ink);
  }
  .room {
    position: relative;
    flex: 1;
    min-height: 0;
    display: grid;
    place-items: center;
    padding: 8px;
  }
  .square {
    position: relative;
    overflow: hidden;
  }
  .layer {
    position: absolute;
    inset: 0;
    will-change: opacity;
  }
  .layer.hidden {
    visibility: hidden;
  }
  .caption {
    position: absolute;
    left: 16px;
    top: 16px;
    max-width: 320px;
    display: flex;
    flex-direction: column;
    gap: 4px;
    padding: 10px 12px;
    background: color-mix(in srgb, var(--sse-ground) 88%, transparent);
    border: 1px solid var(--sse-rule);
    z-index: 10;
  }
  .width,
  .reading {
    position: absolute;
    left: 16px;
    bottom: 12px;
    margin: 0;
    font-size: 13px;
    color: #e8e8e8;
    text-shadow: 0 0 3px #000;
    z-index: 10;
  }
  .reading {
    bottom: 36px;
  }
  .say {
    margin: 12px;
  }
  @media (prefers-reduced-motion: reduce) {
    .layer {
      will-change: auto;
    }
  }
</style>

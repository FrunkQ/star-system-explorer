<!--
  The journey player (grand zoom GZ3 and GZ8, first part; design/SSE4-grand-zoom-design.md sections 5.3 to 5.7). It
  plays a journey laid out by journey.ts: one picture always aimed at the target, the width shrinking at a steady rate
  in powers of ten, each level drawn by its own view (the same reads its screen makes), and at a change of level the
  two views stacked as two canvases, the outgoing one fading over the incoming one while both keep zooming (fudge
  GZ-F1). The place line across the top is the progress bar; Go on (Space or Enter) leaves a stop, Back up returns to
  the one before, Escape leaves the journey. The journey's time is camera travel, not the campaign's clock: the
  universe does not move while it plays (5.7). If a level below has not finished reading when its seam comes, the
  journey holds just before it with one quiet line, never half-faded (5.4).

  Warming up (visual flourish FL5 with FL7's journey use, SSE4-visual-flourish-design.md sections 6 and 6A): the
  levels read one after another in the order the journey needs them, the one on screen first, and are kept in memory
  for the visit only (each view's own cache; never a file). By default the journey waits at its start, the web turning
  slowly and the place line inking in level by level, and Go says "Getting ready…" with the time left until every level
  is ready. A press before that goes early: a level with a coarse picture that keeps the filled-in rule is travelled on
  and settles at the next rest; a level without one holds before its seam as before. Played for film (`film`, or
  `film=1` in the link) it never shows a coarse picture and holds until each level is proper.

  Levels drawn so far: the cosmic web and the galaxies nearby. The galaxy, local stars, system, world and site follow
  in later parts; a level not drawn here is passed over and says why on the place line.
-->
<script lang="ts" module>
  import { BuildTimes } from "../../../ui-contract/src/index.ts";
  // How long each level's reads took on this device, for this visit only (design 6A: "it knows how long").
  const times = new BuildTimes();
</script>

<script lang="ts">
  import { onDestroy } from "svelte";
  import Nearby from "../../../views/cluster-map/src/Nearby.svelte";
  import JourneyWeb from "./JourneyWeb.svelte";
  import { SPACE_CSS } from "../../../renderer/src/index.ts";
  import { ZOOM_IN_FROM, journeyAt, journeyHold, journeyPlan, journeySteps, nearbyAim, readyHold, stopBefore, widthLine, journeyReadNext, journeyWaitLine, levelShows, startTurn, turnedAboutUp, type LevelAnswer, RUNG_WORDS, NEARBY_REACHES, sayNumber, type Approach, type JourneyCamera, type LadderRung, type ViewSession } from "../../../ui-contract/src/index.ts";

  let { session, to = "cluster", approach = ZOOM_IN_FROM, film = false, onclose }: { session: ViewSession; to?: LadderRung; approach?: Approach; film?: boolean; onclose: () => void } = $props();

  /** The levels this player can draw today. */
  const DRAWS: readonly LadderRung[] = ["web", "cluster"];
  const reduced = typeof matchMedia === "function" && matchMedia("(prefers-reduced-motion: reduce)").matches;
  const planned = $derived(journeyPlan({ to: DRAWS.includes(to) ? to : "cluster", approach: { ...approach, from: "web" }, drawn: (r) => DRAWS.includes(r), reduced_motion: reduced }));
  const plan = $derived(planned.ok ? planned.plan : null);

  let t = $state(0);
  let released = $state(0);
  let playing = $state(true);
  /** Each level's answer: true once its proper picture is in (a coarse one is a `LevelAnswer`; none declares one yet). */
  let ready = $state<(boolean | LevelAnswer)[]>([]);
  let notes = $state<string[]>([]);
  // The levels whose reads have been started, in order, and when each began (for its remembered time).
  let opened = $state<number[]>([]);
  const began: Record<number, number> = {};
  const readyOf = (i: number) => (r: { ok: boolean; note?: string }) => {
    const next = [...ready];
    next[i] = r.ok;
    ready = next;
    const took = performance.now() - (began[i] ?? performance.now());
    // A level answered from memory at once is not a build; only real reads teach the device its times.
    if (r.ok && plan && took > 100) times.record(`journey.${plan.path[i]!.rung}`, took);
    if (r.note) notes = [...notes, r.note];
    if (!r.ok) failed = r.note ?? "a level could not be read";
  };
  const proper = (i: number) => ready[i] === true;
  const allReady = $derived(!!plan && plan.path.every((_, i) => proper(i)));
  // Went early: Go pressed at the start before every level was ready (never for film).
  let early = $state(false);
  const how = $derived({ early: early && !film, film });
  // What each level last showed, so one travelled into on its coarse picture settles only at rest.
  const showed: Record<number, "proper" | "coarse" | null> = {};
  const host = { hot: false, data_saver: typeof navigator !== "undefined" && !!(navigator as { connection?: { saveData?: boolean } }).connection?.saveData, memory_over: false, asked: true };
  let failed = $state<string | null>(null);

  // The clock of the journey: real seconds, camera travel only (never the campaign's clock).
  let raf = 0;
  let last = 0;
  let waiting = $state<number | null>(null);
  // The start level's slow turn while the way down gets ready, radians about the page's up (not under reduced motion).
  let turn = $state(0);
  let clock = $state(0);
  function tick(now: number) {
    const dt = last ? Math.min(0.1, (now - last) / 1000) : 0;
    if (plan && playing) {
      const want = journeyHold(plan, t + dt, released);
      const held = readyHold(plan, want, ready, how);
      waiting = held.waiting;
      t = held.t;
    }
    const resting = t === 0 && !reduced && !film && released === 0;
    if (resting || turn !== 0) turn = startTurn(turn, dt, resting);
    if (now - clock > 250) clock = now;
    last = now;
    raf = requestAnimationFrame(tick);
  }
  raf = requestAnimationFrame(tick);
  onDestroy(() => cancelAnimationFrame(raf));

  const frame = $derived(plan ? journeyAt(plan, t) : null);
  // The warm-up: the next level to read, one at a time in the order the journey needs them.
  $effect(() => {
    if (!plan || !frame) return;
    if (opened.some((i) => ready[i] === undefined)) return;
    const next = journeyReadNext(plan, { entry: frame.entry, t }, ready, host);
    if (next === null || opened.includes(next)) return;
    began[next] = performance.now();
    opened = [...opened, next];
  });
  const mayRead = (i: number) => opened.includes(i);
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
  const restsNow = $derived(!playing || !!frame?.done || (!!frame?.stop && released <= (plan?.stops.filter((s) => s.hold_s === null).indexOf(frame.stop!) ?? -1)));
  const picture = (i: number): "proper" | "coarse" | "hold" => {
    const p = levelShows(ready[i], how, showed[i] ?? null, restsNow);
    showed[i] = p === "hold" ? (showed[i] ?? null) : p;
    return p;
  };
  const atStop = $derived(!!frame?.stop && released <= (plan?.stops.filter((s) => s.hold_s === null).indexOf(frame.stop!) ?? -1));
  const done = $derived(!!frame?.done);
  const width = $derived(frame ? widthLine(frame.width_m) : null);

  let box = $state(0);
  let boxH = $state(0);
  // The room's padding (8 px a side) is inside its client size: the square fits within it, never past the window.
  const side = $derived(Math.max(0, Math.min(box, boxH) - 16));

  function goOn() {
    if (!plan) return;
    if (done) return void ((t = 0), (released = 0), (playing = true));
    if (atStop && t === 0) early = !allReady;
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
  const reading = $derived(opened.find((i) => ready[i] === undefined));
  const waitLine = $derived(plan && !allReady ? (void clock, journeyWaitLine(plan, ready, times, reading !== undefined ? performance.now() - (began[reading] ?? performance.now()) : 0)) : null);
  const gettingReady = $derived(!allReady && atStop && t === 0 && !done);
  // The web's camera turned by the start's slow turn.
  const turned = (c: JourneyCamera | null): JourneyCamera | null => (c && turn !== 0 ? { ...c, eye_dir: turnedAboutUp(c.eye_dir, turn) } : c);
</script>

<svelte:window onkeydown={key} />

<section class="journey" aria-label="Zoom in from the cosmic web" data-journey-player data-journey-t={sayNumber(t, 3, { decimals: 2 })} data-journey-rung={frame?.rung ?? ""} data-journey-waiting={waiting ?? ""} data-journey-opened={opened.join(" ")} data-journey-film={film ? "" : undefined} data-journey-early={how.early ? "" : undefined} data-journey-done={done ? "" : undefined}>
  <nav class="line" aria-label="Where the journey is">
    {#each steps as s (s.rung)}
      <button
        type="button"
        class="step {s.state}"
        class:ready={s.entry !== undefined && proper(s.entry)}
        disabled={s.entry === undefined}
        aria-current={s.state === "current" ? "step" : undefined}
        title={s.why ?? (s.entry !== undefined ? `Go to ${s.words}` : `${s.words}: not on this journey`)}
        onclick={() => s.entry !== undefined && jumpTo(s.entry)}
        data-journey-step={s.rung}
        data-state={s.state}
        data-ready={s.entry !== undefined && proper(s.entry) ? "" : undefined}>{s.words}</button
      >
    {/each}
    <span class="gap"></span>
    <button type="button" class="quiet" onclick={backUp} data-journey-back>Back up</button>
    <button type="button" class="go" class:waits={gettingReady} onclick={goOn} data-journey-go title={gettingReady ? (film ? "Filming waits until every level is ready" : "Go now, on plainer pictures where a level has one") : undefined}
      >{done ? "Again" : gettingReady ? "Getting ready…" : atStop ? (t === 0 ? "Zoom in" : "Go on") : playing ? "Zooming…" : "Go on"}</button
    >
    <button type="button" class="quiet" onclick={onclose} aria-label="Leave the journey" data-journey-leave>Leave</button>
  </nav>

  <div class="room" bind:clientWidth={box} bind:clientHeight={boxH}>
    {#if !planned.ok}
      <p class="say" role="alert">{planned.why}</p>
    {:else if plan}
      <div class="square" style:width="{side}px" style:height="{side}px" style:background={SPACE_CSS}>
        {#each plan.path as e, i (i)}
          <div class="layer" class:hidden={!layerShown(i)} style:opacity={layerOpacity(i)} style:z-index={layerZ(i)} data-journey-layer={e.rung} data-ready={proper(i) ? "" : undefined} data-picture={layerShown(i) ? picture(i) : undefined}>
            {#if !mayRead(i)}
              <!-- Not read yet: its turn in the warm-up comes after the levels needed before it. -->
            {:else if e.rung === "web"}
              <JourneyWeb {session} cam={turned(webBehind ? (camOf(i + 1) ?? camFor(i)) : camFor(i))} onready={readyOf(i)} inner={webBehind ? { radius_mpc: NEARBY_MPC, alpha: camOf(i)?.opacity ?? 0 } : null} />
            {:else if e.rung === "cluster"}
              {@const c = camFor(i)}
              <Nearby {session} seen="free" scale="real" hud={false} look="true" onchart={() => {}} aim={c ? nearbyAim(c, 560) : nearbyAim({ width_m: e.stop_m, eye_dir: [0.85, 0, 0.52] }, 560)} onread={(ok) => readyOf(i)({ ok, note: ok ? "" : "the galaxies nearby could not be read" })} />
            {/if}
          </div>
        {/each}
        {#if frame?.stop && atStop}
          <div class="caption" data-journey-caption>
            <b>{RUNG_WORDS[frame.rung]}</b>
            <span>{t === 0 ? (gettingReady ? (film ? "Filming starts once every level is ready." : "The way down is getting ready, and the journey goes smoothly once it is. Press Space to go now on whatever is ready.") : "Press Zoom in, or Space, to travel down to the galaxies around our own.") : "Press Go on to travel on."}</span>
            {#if waitLine && t === 0}<span class="wait" data-journey-wait>{waitLine}</span>{/if}
          </div>
        {/if}
        {#if waiting !== null && t > 0}<p class="reading" data-journey-reading>{readingWords(waiting)}{waitLine ? ` ${waitLine}` : ""}</p>{/if}
        {#if width && (frame?.width_line || atStop || done)}
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
  /* The way down inks in as each level becomes ready (design 6A: the place line is the progress bar). */
  .step.coming.ready {
    color: var(--sse-ink-muted, var(--sse-ink));
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
  .go.waits {
    background: transparent;
    border-color: var(--sse-ink-muted, var(--sse-ink));
    color: var(--sse-ink);
  }
  .wait {
    color: var(--sse-ink-muted, var(--sse-ink));
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

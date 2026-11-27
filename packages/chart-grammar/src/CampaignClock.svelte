<!--
  The campaign clock (three desks design 7.3): on the Run desk it takes the ruler's place. One line: the campaign's
  date and time in the pack's calendar, large; step chips from the calendar's units (+ 1 hour, + 1 day, + 1 week);
  and Run ▸ with a rate, which lets human time flow (worlds turn, ships move along their courses). No deep time,
  no What if, no Look ahead: ageing a star is one press on the desk switch away. Above the system nothing moves in
  human time, so the clock shows the date and has no steps.
  What is due (TM3b): the next few dated things as ticks on a short line a few days wide; a press on a tick offers
  "Go to then". "Jump to a date…" opens the ordinary jump with a dialog that says where it lands and what it passes
  over. Both read the kernel's `happenings` reply (`ui-contract/src/due.ts`, package TP6), handed in by the shell;
  until that op exists the line says "Nothing due" and the dialog says nothing is marked on the way.
  Presentation only: every step, every tick of Run ▸ and every jump is handed to `onstep` with the new campaign time.
-->
<script lang="ts">
  import { untrack } from "svelte";
  import { motionMs } from "./motion.ts";
  import { DAY_S, YEAR_S, dateWords, defaultCalendar, spanWords, steps, type Calendar } from "./calendar.ts";
  import { NO_HAPPENINGS, passedWords, type DueTick, type Happenings } from "../../ui-contract/src/index.ts";

  let {
    campaignNow,
    calendar,
    moving = true,
    due = [],
    dueWindowS = 3n * BigInt(DAY_S),
    passing,
    onstep,
    onrate,
  }: {
    /** The campaign's now, in years. */
    campaignNow: number;
    calendar?: Calendar;
    /** False above the system: the date only. */
    moving?: boolean;
    /** The next few marks ahead (from `dueAhead` over the kernel's happenings); empty until TP6. */
    due?: DueTick[];
    /** How far ahead the due line looks, in seconds. */
    dueWindowS?: bigint;
    /** What a jump of this many seconds would pass over (the kernel's happenings for the stretch); absent until TP6. */
    passing?: (span_s: bigint) => Happenings;
    onstep?: (yr: number) => void;
    /** How fast time passes while Run ▸ runs, campaign seconds per real second (0 when stopped): the picture's clock (CLK1) runs at it. */
    onrate?: (s_per_s: number) => void;
  } = $props();

  const cal = $derived(calendar ?? defaultCalendar(campaignNow));
  const UNITS = ["hour", "day", "week"];
  const chips = $derived(steps(cal).filter((s) => UNITS.includes(s.id)));
  const hourYr = $derived(steps(cal).find((s) => s.id === "hour")!.yr);
  // Run ▸'s rates, as the time that passes in the campaign each real second.
  const RATES = [
    { id: "real", label: "real time", hours: 1 / 3600 },
    { id: "minute", label: "a minute a second", hours: 1 / 60 },
    { id: "hour", label: "an hour a second", hours: 1 },
    { id: "day", label: "a day a second", hours: 24 },
  ];
  let rateId = $state("minute");
  let running = $state(false);
  const rate = $derived(RATES.find((r) => r.id === rateId) ?? RATES[1]!);

  // Human time flows a few times a second: smooth enough for a turning world, gentle on the kernel. Reduced motion
  // still lets the clock run (it is a clock, not an animation); it just ticks once a second.
  let clockEl: HTMLElement | undefined = $state();
  // The picture's clock (CLK1) runs at Run ▸'s rate, and stops with it, so the worlds move smoothly between ticks.
  $effect(() => {
    const r = running && moving ? rate.hours * 3600 : 0;
    untrack(() => onrate?.(r));
  });
  $effect(() => () => untrack(() => onrate?.(0)));
  $effect(() => {
    if (!running || !moving) return;
    const every = motionMs(clockEl ?? null) > 0 ? 250 : 1000;
    const first = performance.now();
    const from = untrack(() => campaignNow);
    const per_ms = (rate.hours * hourYr) / 1000;
    // The date is the start plus the time run, worked out afresh at each tick: adding a quarter second at a time to
    // a year near a star's age is lost to rounding (a year there resolves about 30 s), so real time never moved on.
    const id = setInterval(() => onstep?.(from + (performance.now() - first) * per_ms), every);
    return () => clearInterval(id);
  });
  $effect(() => {
    if (!moving) running = false;
  });

  const words = $derived(dateWords(campaignNow, cal, moving));

  // The due line. A tick pressed offers "Go to then"; the offset is whole seconds, turned into the clock's years
  // only at the hand-over (TP3 moves the clock itself to seconds).
  let picked: DueTick | null = $state(null);
  const toYr = (s: bigint) => Number(s) / YEAR_S;
  const goTo = (d: DueTick) => ((picked = null), onstep?.(campaignNow + toYr(d.ahead_s)));

  // "Jump to a date…": an amount of a calendar unit, the date it lands on, and what lies on the way.
  const JUMP_UNITS = ["day", "week", "month", "year"];
  let jumpDialog: HTMLDialogElement | undefined = $state();
  let jumpN = $state(1);
  let jumpUnit = $state("week");
  const jumpYr = $derived((steps(cal).find((s) => s.id === jumpUnit)?.yr ?? 0) * (Number.isFinite(jumpN) && jumpN > 0 ? jumpN : 0));
  const jumpS = $derived(BigInt(Math.round(jumpYr * YEAR_S)));
  const onTheWay = $derived(jumpS > 0n ? passedWords(passing?.(jumpS) ?? NO_HAPPENINGS) : "");
  const jump = () => {
    if (jumpYr <= 0) return;
    jumpDialog?.close();
    onstep?.(campaignNow + jumpYr);
  };
</script>

<div class="clock" bind:this={clockEl} data-campaign-clock={campaignNow} data-running={running ? "yes" : "no"}>
  <div class="readout">
    <div class="date" data-clock-date>{words}</div>
    <div class="sub"><span class="stamp">Now</span> in the campaign{cal.id !== "sse.calendar.standard" ? ` · ${cal.label}` : ""}</div>
  </div>
  {#if moving}
    <div class="bar" role="toolbar" aria-label="Campaign clock">
      {#each chips as s (s.id)}
        <button type="button" class="chip" data-clock-step={s.id} onclick={() => onstep?.(campaignNow + s.yr)} title="Move the campaign on {spanWords(s.yr, cal)}; players' screens follow">+ 1 {s.label}</button>
      {/each}
      <span class="run">
        <button type="button" class="chip" class:on={running} aria-pressed={running} data-clock-run onclick={() => (running = !running)} title={running ? "Stop the clock" : "Let time pass in the campaign"}>{running ? "Stop ❚❚" : "Run ▸"}</button>
        <select aria-label="Rate" bind:value={rateId} data-clock-rate>
          {#each RATES as r (r.id)}<option value={r.id}>{r.label}</option>{/each}
        </select>
      </span>
      <div class="due" data-clock-due aria-label="What is due">
        <span class="due-label">Due</span>
        <span class="due-line" title="The next {spanWords(toYr(dueWindowS), cal)}">
          {#each due as d (d.mark.source + d.mark.at_s)}
            <button type="button" class="tick" style:left="{d.x * 100}%" data-due-tick={d.mark.kind} aria-label="{d.mark.why}, in {spanWords(toYr(d.ahead_s), cal)}" title="{d.mark.why}, in {spanWords(toYr(d.ahead_s), cal)}" onclick={() => (picked = picked === d ? null : d)}></button>
          {/each}
          {#if picked}
            <span class="offer" role="group" aria-label="Due tick">
              <span>{picked.mark.why}</span>
              <button type="button" class="chip" data-due-go onclick={() => picked && goTo(picked)}>Go to then</button>
            </span>
          {/if}
        </span>
        {#if !due.length}<span class="none" data-due-none>Nothing due</span>{/if}
      </div>
      <button type="button" class="chip" data-clock-jump onclick={() => jumpDialog?.showModal()} title="Move the campaign on by weeks or years: says what it will do first">Jump to a date…</button>
    </div>
  {/if}
</div>

{#if moving}
  <dialog bind:this={jumpDialog} class="jump" aria-label="Jump to a date" data-jump-dialog>
    <form method="dialog" onsubmit={(e) => (e.preventDefault(), jump())}>
      <h2>Jump to a date</h2>
      <label class="amount">
        <span>Move the campaign on</span>
        <input type="number" min="1" step="1" bind:value={jumpN} data-jump-n aria-label="How many" />
        <select bind:value={jumpUnit} data-jump-unit aria-label="Unit">
          {#each steps(cal).filter((s) => JUMP_UNITS.includes(s.id)) as s (s.id)}<option value={s.id}>{s.label}{jumpN === 1 ? "" : "s"}</option>{/each}
        </select>
      </label>
      <p data-jump-lands>Lands on <strong>{dateWords(campaignNow + jumpYr, cal, true)}</strong>, {spanWords(jumpYr, cal)} from now. Players' screens follow.</p>
      <p data-jump-passes>{onTheWay ? `On the way: ${onTheWay}.` : "Nothing is marked on the way."}</p>
      <div class="acts">
        <button type="button" class="chip" onclick={() => jumpDialog?.close()}>Cancel</button>
        <button type="submit" class="chip on" data-jump-go disabled={jumpYr <= 0}>Jump</button>
      </div>
    </form>
  </dialog>
{/if}

<style>
  .clock {
    display: flex;
    align-items: center;
    gap: 28px;
    height: 100%;
    padding: 0 24px;
    background: var(--sse-ground-raised);
    color: var(--sse-ink);
    font-family: var(--sse-font-body);
  }
  .date {
    font-family: var(--sse-font-display);
    font-size: 24px;
    line-height: 1.2;
  }
  .sub {
    font-size: 11px;
    color: var(--sse-ink-muted);
    margin-top: 2px;
  }
  .stamp {
    display: inline-block;
    padding: 0 4px;
    margin-right: 4px;
    border: var(--sse-line) solid var(--sse-accent);
    color: var(--sse-accent);
    font-family: var(--sse-font-mono);
    font-size: 9px;
    letter-spacing: 0.1em;
    text-transform: uppercase;
    transform: rotate(-2deg);
  }
  .bar {
    display: flex;
    align-items: center;
    gap: 8px;
    flex-wrap: wrap;
  }
  .chip {
    height: 30px;
    padding: 0 12px;
    font: inherit;
    font-size: 13px;
    border: var(--sse-line) solid var(--sse-ink);
    border-radius: var(--sse-radius);
    background: transparent;
    color: var(--sse-ink);
    cursor: pointer;
  }
  .chip:hover {
    background: var(--sse-ground);
  }
  .chip.on {
    background: var(--sse-ink);
    color: var(--sse-on-ink);
  }
  .run {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    margin-left: 8px;
  }
  .due {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    margin-left: 8px;
    font-size: 11px;
    color: var(--sse-ink-muted);
  }
  .due-label {
    font-family: var(--sse-font-mono);
    letter-spacing: 0.08em;
    text-transform: uppercase;
  }
  .due-line {
    position: relative;
    width: 140px;
    height: 18px;
    border-bottom: var(--sse-line) solid var(--sse-ink-muted);
  }
  .tick {
    position: absolute;
    bottom: -1px;
    width: 10px;
    height: 14px;
    margin-left: -5px;
    padding: 0;
    border: 0;
    border-left: 2px solid var(--sse-accent);
    background: transparent;
    cursor: pointer;
  }
  .offer {
    position: absolute;
    bottom: calc(100% + 6px);
    left: 0;
    z-index: 5;
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 6px 8px;
    white-space: nowrap;
    background: var(--sse-ground-raised);
    color: var(--sse-ink);
    border: var(--sse-line) solid var(--sse-ink);
    border-radius: var(--sse-radius);
  }
  .jump {
    border: var(--sse-line) solid var(--sse-ink);
    border-radius: var(--sse-radius);
    background: var(--sse-ground-raised);
    color: var(--sse-ink);
    font-family: var(--sse-font-body);
    max-width: min(420px, calc(100vw - 32px));
  }
  .jump h2 {
    font-family: var(--sse-font-display);
    font-size: 18px;
    margin: 0 0 12px;
  }
  .amount {
    display: flex;
    align-items: center;
    gap: 8px;
    flex-wrap: wrap;
  }
  .amount input {
    width: 72px;
    height: 30px;
    font: inherit;
  }
  .acts {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
    margin-top: 12px;
  }
  select {
    height: 30px;
    font: inherit;
    font-size: 12px;
  }
  @media (max-width: 760px) {
    .clock {
      gap: 12px;
      padding: 8px 12px;
      flex-wrap: wrap;
    }
    .date {
      font-size: 18px;
    }
  }
</style>

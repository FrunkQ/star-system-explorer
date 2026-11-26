<!--
  The campaign clock (three desks design 7.3): on the Run desk it takes the ruler's place. One line: the campaign's
  date and time in the pack's calendar, large; step chips from the calendar's units (+ 1 hour, + 1 day, + 1 week);
  and Run ▸ with a rate, which lets human time flow (worlds turn, ships move along their courses). No deep time,
  no What if, no Look ahead: ageing a star is one press on the desk switch away. Above the system nothing moves in
  human time, so the clock shows the date and has no steps.
  Presentation only: every step and every tick of Run ▸ is handed to `onstep` with the new campaign time.
-->
<script lang="ts">
  import { untrack } from "svelte";
  import { motionMs } from "./motion.ts";
  import { dateWords, defaultCalendar, spanWords, steps, type Calendar } from "./calendar.ts";

  let {
    campaignNow,
    calendar,
    moving = true,
    onstep,
    onrate,
  }: {
    /** The campaign's now, in years. */
    campaignNow: number;
    calendar?: Calendar;
    /** False above the system: the date only. */
    moving?: boolean;
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
    </div>
  {/if}
</div>

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

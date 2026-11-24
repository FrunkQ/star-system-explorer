<!--
  The performance readout (PF1, design/SSE4-performance-and-minimum-spec.md).
  A small card that opens from More while "Performance readout" is on: each
  big chunk's calls, median and 95th percentile against its budget, the
  frame time, start-up, memory and long tasks. A row opens to its ops. The
  report is made only when the person copies or saves it; nothing is sent.
-->
<script lang="ts">
  import { describeDevice, jsHeapMb, perf, perfReport, watchPage, type ChunkSummary, type DeviceDescription } from "../../ui-contract/src/index.ts";
  import { opensFrom } from "../../chart-grammar/src/index.ts";
  import { loadPref, savePref } from "./prefs.ts";
  import { deviceReadout, measureAgain, quality, setQuality } from "../../renderer/src/runtime.ts";
  import { QUALITY_WORDS, type ProbeResult, type QualityChoice } from "../../renderer/src/probe.ts";

  let { kernel = "", edge, from, onclose }: { kernel?: string; edge?: { tier: string; unitsPerSecond: number }; from?: () => Element | null; onclose: () => void } = $props();

  let chunks = $state<ChunkSummary[]>([]);
  let frame = $state<number | null>(null);
  let longTasks = $state(0);
  let wasmMb = $state<number | null>(null);
  let heapMb = $state<number | null>(null);
  let open = $state<string | null>(null);
  let folded = $state(loadPref("perf.folded") === "1");
  let label = $state(loadPref("perf.label") ?? "");
  let note = $state("");
  let device: DeviceDescription | null = null;

  // VP4: drawing quality. The device's measured speed (the probe), the person's choice, and "Measure again".
  let q = $state<QualityChoice>(quality());
  let speed = $state<ProbeResult | null>(null);
  let measuring = $state(false);
  const QUALITIES = Object.entries(QUALITY_WORDS) as [QualityChoice, string][];
  const speedWords = (r: ProbeResult | null) =>
    r ? `Graphics measured on ${r.backend === "webgpu" ? "WebGPU" : "WebGL2"}: fill ${r.factors.fill.toFixed(1)}×, march ${r.factors.march.toFixed(1)}×, geometry ${r.factors.geometry.toFixed(1)}× the reference's time${r.steady ? "" : " (unsteady: the device paused)"}.` : "Graphics speed not measured yet: it is measured with the first view drawn on the GPU.";
  $effect(() => void deviceReadout().then((d) => (speed = d.probe)).catch(() => undefined));
  function pickQuality(v: QualityChoice) {
    q = v;
    setQuality(v);
  }
  async function again() {
    measuring = true;
    speed = await measureAgain().catch(() => null);
    measuring = false;
  }

  const ms = (x: number) => (x >= 1000 ? `${(x / 1000).toFixed(x >= 10_000 ? 0 : 1)} s` : `${x >= 100 ? Math.round(x) : x.toFixed(1)} ms`);
  const fill = (c: ChunkSummary) => Math.min(1, c.p95_ms / c.budget_ms);
  const STAGES: [string, string][] = [
    ["startup.packs", "packs"],
    ["startup.kernel", "kernel"],
    ["startup.session", "universe"],
    ["startup.measure", "measured"],
    ["startup.first_chart", "first chart"],
  ];
  let stages = $state<string[]>([]);

  let seen = -1;
  function refresh() {
    if (perf.version === seen) return;
    seen = perf.version;
    const s = perf.summary();
    const d = s.find((c) => c.id === "drawing")?.sources.find((x) => x.source === "page.frame");
    frame = d ? d.median_ms : null;
    if (!folded) chunks = s;
    longTasks = perf.longTasks;
    wasmMb = perf.wasmBytes ? Math.round(perf.wasmBytes / 1e5) / 10 : null;
    heapMb = jsHeapMb() ?? null;
    const st = perf.stamps();
    stages = STAGES.filter(([k]) => st[k] !== undefined).map(([k, w]) => `${w} ${ms(st[k]!)}`);
  }

  $effect(() => {
    const stop = watchPage();
    // Once a second is plenty for a readout, and costs nothing when nothing changed.
    const t = setInterval(refresh, 1000);
    refresh();
    return () => (clearInterval(t), stop());
  });
  $effect(() => savePref("perf.folded", folded ? "1" : "0"));

  function fold() {
    folded = !folded;
    seen = -1;
    refresh();
  }

  async function report(): Promise<string> {
    device ??= await describeDevice(edge);
    savePref("perf.label", label);
    return JSON.stringify(perfReport({ device, label: label.trim(), kernel, site: location.host }), null, 2);
  }
  async function copy() {
    const text = await report();
    try {
      await navigator.clipboard.writeText(text);
      note = "Copied. Paste it wherever you keep reports; nothing was sent.";
    } catch {
      note = "This browser would not copy; use Save instead.";
    }
  }
  async function save() {
    const text = await report();
    const a = document.createElement("a");
    a.href = URL.createObjectURL(new Blob([text], { type: "application/json" }));
    a.download = `sse4-perf-${(label.trim() || "device").replace(/[^A-Za-z0-9-]+/g, "-").toLowerCase()}.json`;
    a.click();
    setTimeout(() => URL.revokeObjectURL(a.href), 1000);
    note = "Saved to your downloads; nothing was sent.";
  }
  function clear() {
    perf.clear();
    open = null;
    seen = -1;
    refresh();
    note = "";
  }
</script>

<section class="perf" class:folded aria-label="Performance readout" data-perf use:opensFrom={from}>
  <header>
    <h2>Performance</h2>
    <span class="live" title="The page's frame interval (median of the last few seconds)">{frame === null ? "measuring…" : `${ms(frame)} a frame`}</span>
    <button type="button" class="ic" aria-expanded={!folded} onclick={fold} title={folded ? "Show the chunks" : "Fold to one line"}>{folded ? "▴" : "▾"}</button>
    <button type="button" class="ic" onclick={onclose} title="Switch the readout off">×</button>
  </header>
  {#if !folded}
    {#if chunks.length === 0}
      <p class="m">Nothing measured yet. Use the app as usual: every chart, card and screen you open is timed from now.</p>
    {:else}
      <ul class="chunks">
        {#each chunks as c (c.id)}
          <li data-chunk={c.id} data-verdict={c.verdict}>
            <button type="button" class="row" aria-expanded={open === c.id} onclick={() => (open = open === c.id ? null : c.id)} title={c.meaning}>
              <span class="nm">{c.name}</span>
              <span class="fig">{ms(c.median_ms)}<small> · 95% {ms(c.p95_ms)}</small></span>
              <span class="bar" aria-hidden="true"><span class="in" style="transform: scaleX({fill(c)})"></span></span>
              <span class="vd">{c.verdict === "over" ? `over ${ms(c.budget_ms)}` : c.verdict === "near" ? `near ${ms(c.budget_ms)}` : `within ${ms(c.budget_ms)}`}</span>
            </button>
            {#if open === c.id}
              <table>
                <thead><tr><th>Source</th><th>Calls</th><th>Median</th><th>95%</th><th title="The kernel's own time, without the trip to its worker">Kernel</th></tr></thead>
                <tbody>
                  {#each c.sources as s (s.source + (s.detail ?? ""))}
                    <tr><td class="op">{s.source}{#if s.detail}<small> {s.detail}</small>{/if}</td><td>{s.calls}</td><td>{ms(s.median_ms)}</td><td>{ms(s.p95_ms)}</td><td>{s.kernel_median_ms === undefined ? "" : ms(s.kernel_median_ms)}</td></tr>
                  {/each}
                </tbody>
              </table>
            {/if}
          </li>
        {/each}
      </ul>
    {/if}
    <p class="m line">
      {#if stages.length}<span>Start-up: {stages.join(" · ")}</span>{/if}
      {#if wasmMb !== null}<span>kernel memory {wasmMb} MB</span>{/if}
      {#if heapMb !== null}<span>page memory {heapMb} MB</span>{/if}
      <span title="Main-thread tasks over 50 ms since the readout started">{longTasks} long {longTasks === 1 ? "task" : "tasks"}</span>
    </p>
    <p class="m line" data-perf-drawing>
      <label title="Auto places this device by its measured speed; the others are your choice, and views say so">Quality <select value={q} onchange={(e) => pickQuality(e.currentTarget.value as QualityChoice)}>{#each QUALITIES as [v, w] (v)}<option value={v}>{w}</option>{/each}</select></label>
      <span>{measuring ? "Measuring the graphics (about half a second)…" : speedWords(speed)}</span>
      <button type="button" class="quiet" onclick={again} disabled={measuring} data-perf-measure>Measure again</button>
    </p>
    <div class="acts">
      <label class="lab"><span>This device</span><input type="text" bind:value={label} placeholder="e.g. my phone" maxlength="60" /></label>
      <button type="button" onclick={copy} data-perf-copy>Copy report</button>
      <button type="button" onclick={save}>Save</button>
      <button type="button" class="quiet" onclick={clear}>Clear</button>
    </div>
    <p class="m">{note || "Kept on this device. A report holds timings and what this device says about itself, never your universe."}</p>
  {/if}
</section>

<style>
  .perf {
    position: fixed;
    right: 12px;
    bottom: 44px;
    z-index: 30;
    width: min(380px, calc(100vw - 24px));
    max-height: min(70vh, 640px);
    overflow: auto;
    padding: 10px 12px;
    border: var(--sse-line) solid var(--sse-ink);
    border-radius: var(--sse-radius);
    background: var(--sse-ground-raised);
    color: var(--sse-ink);
    box-shadow: var(--sse-card-shadow);
    font-family: var(--sse-font-body);
    font-size: 12.5px;
  }
  .perf.folded {
    width: auto;
    min-width: 220px;
  }
  header {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  h2 {
    margin: 0;
    font-family: var(--sse-font-display);
    font-size: 14px;
    font-weight: 600;
  }
  .live {
    flex: 1;
    font-family: var(--sse-font-mono);
    font-size: 11.5px;
    color: var(--sse-ink-muted);
  }
  button {
    font: inherit;
    color: inherit;
    background: transparent;
    border: var(--sse-line-hair, 1px) solid var(--sse-rule);
    border-radius: var(--sse-radius);
    min-height: 28px;
    padding: 0 8px;
    cursor: pointer;
    transition: transform var(--sse-motion-quick) var(--sse-ease);
  }
  button:hover {
    border-color: var(--sse-ink);
  }
  button:active {
    transform: translateY(1px);
  }
  button:focus-visible {
    outline: 2px solid var(--sse-focus);
    outline-offset: 1px;
  }
  .ic {
    min-width: 28px;
    padding: 0;
  }
  .quiet {
    border-color: transparent;
    color: var(--sse-ink-muted);
  }
  .chunks {
    list-style: none;
    margin: 8px 0 0;
    padding: 0;
  }
  .row {
    display: grid;
    grid-template-columns: minmax(0, 1fr) auto;
    gap: 2px 8px;
    width: 100%;
    padding: 5px 4px;
    border: none;
    border-top: var(--sse-line-hair, 1px) solid var(--sse-rule);
    border-radius: 0;
    text-align: left;
  }
  .nm {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .fig {
    font-family: var(--sse-font-mono);
    font-size: 11.5px;
    text-align: right;
  }
  .fig small,
  .op small {
    color: var(--sse-ink-muted);
  }
  .bar {
    grid-column: 1;
    height: 3px;
    align-self: center;
    background: var(--sse-rule);
    overflow: hidden;
  }
  .in {
    display: block;
    height: 100%;
    background: var(--sse-zone-ink);
    transform-origin: left;
    transition: transform var(--sse-motion) var(--sse-ease);
  }
  [data-verdict="near"] .in {
    background: var(--sse-ink-muted);
  }
  [data-verdict="over"] .in {
    background: var(--sse-accent);
  }
  .vd {
    font-size: 11px;
    color: var(--sse-ink-muted);
    text-align: right;
  }
  [data-verdict="over"] .vd {
    color: var(--sse-accent-ink);
  }
  table {
    width: 100%;
    margin: 2px 0 6px;
    border-collapse: collapse;
    font-family: var(--sse-font-mono);
    font-size: 11px;
  }
  th {
    font-weight: 500;
    color: var(--sse-ink-muted);
    text-align: right;
  }
  th:first-child,
  td:first-child {
    text-align: left;
    overflow-wrap: anywhere;
  }
  td {
    padding: 2px 0 2px 6px;
    text-align: right;
    border-top: var(--sse-line-hair, 1px) solid var(--sse-rule);
  }
  .m {
    margin: 8px 0 0;
    font-size: 11.5px;
    color: var(--sse-ink-muted);
  }
  .line {
    display: flex;
    flex-wrap: wrap;
    gap: 2px 10px;
  }
  .acts {
    display: flex;
    flex-wrap: wrap;
    align-items: end;
    gap: 6px;
    margin-top: 8px;
  }
  .lab {
    display: flex;
    flex-direction: column;
    flex: 1 1 140px;
    gap: 2px;
    font-size: 11px;
    color: var(--sse-ink-muted);
  }
  input {
    min-height: 28px;
    min-width: 0;
    padding: 0 6px;
    border: var(--sse-line) solid var(--sse-ink);
    background: var(--sse-ground);
    color: var(--sse-ink);
    font: inherit;
    font-size: 12.5px;
  }
  /* A phone: a sheet along the foot of the screen. */
  @media (max-width: 600px) {
    .perf {
      right: 0;
      left: 0;
      bottom: 0;
      width: auto;
      max-height: 60vh;
      border-radius: var(--sse-radius) var(--sse-radius) 0 0;
    }
  }
  @media (prefers-reduced-motion: reduce) {
    button,
    .in {
      transition: none;
    }
  }
</style>

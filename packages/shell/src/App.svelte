<!--
  The GM shell: a fixed, curated chart-room layout (H8.8) whose areas are
  filled from a screen document with view modules. The shell reaches the
  kernel only through the session, the same client API a player app uses.
-->
<script lang="ts">
  import { checkScreen, gmSeat, markFor, type CheckedScreen, type ScreenDoc, type Symbology, type SystemScene, type ViewModule, type ViewSession } from "../../ui-contract/src/index.ts";
  import { openBrowserSession } from "../../ui-contract/src/host-browser.ts";
  import { themeCss, THEMES } from "../../skins/chart-room/src/index.ts";
  import { Segmented, ToolButton, formatYears } from "../../chart-grammar/src/index.ts";
  import workspaceDoc from "./documents/workspace.gm.json";
  import sceneDoc from "./documents/kestrel.scene.json";
  import symbologyDoc from "./documents/diagrammatic.stub.json";
  import { REGISTRY } from "./registry.ts";
  import { createWorkspace } from "./workspace.svelte.ts";
  import { loadPref, savePref } from "./prefs.ts";
  import type { Json } from "../../ui-contract/src/index.ts";

  const scene = sceneDoc as unknown as SystemScene;
  const symbology = symbologyDoc as unknown as Symbology;
  const ws = createWorkspace(scene.now_yr);

  const checked = checkScreen(workspaceDoc as unknown as ScreenDoc, REGISTRY);
  const screen: CheckedScreen | null = checked.ok ? checked.screen : null;
  const mountFor = (area: string) => screen?.mounts.find((m) => m.area === area);

  let theme = $state(loadPref("theme") === "night-chart" ? "night-chart" : "chart-room");
  $effect(() => {
    const root = document.documentElement;
    root.setAttribute("style", themeCss(theme));
    root.style.colorScheme = THEMES.find((t) => t.id === theme)?.mode ?? "light";
    root.dataset.theme = theme;
    savePref("theme", theme);
  });

  let session = $state<ViewSession | null>(null);
  let failure = $state<string | null>(null);
  let status = $state({ kernel: "", tier: "", hash: "" });
  openBrowserSession({ base: document.baseURI, seat: gmSeat(), universe: scene.universe })
    .then(async (s) => {
      session = s;
      status = { kernel: await s.kernelVersion(), tier: s.device()?.tier ?? "unmeasured", hash: await s.stateHash() };
    })
    .catch((e) => (failure = String(e)));

  $effect(() => {
    void ws.revision;
    const s = session;
    if (s) s.stateHash().then((h) => (status = { ...status, hash: h }));
  });

  function view(node: HTMLElement, m: { module: ViewModule; params: Record<string, Json> }) {
    const h = m.module.mount(node, { session: session!, workspace: ws, params: m.params, mark: (s) => markFor(s, symbology) }, scene);
    return { destroy: () => h.dispose() };
  }

  let help: HTMLDialogElement | undefined = $state();
  const bodies = scene.subjects.filter((s) => s.kind === "star" || s.orbit).sort((a, b) => (a.orbit?.a_au ?? 0) - (b.orbit?.a_au ?? 0));
  function cycle(by: number) {
    const i = bodies.findIndex((b) => b.address === ws.selection);
    ws.select(bodies[(i + by + bodies.length) % bodies.length]!.address);
  }

  function onkeydown(e: KeyboardEvent) {
    const t = e.target as HTMLElement;
    if (t.closest("input, textarea, select, dialog") || e.ctrlKey || e.metaKey || e.altKey) return;
    const keys: Record<string, () => void> = {
      e: () => (ws.mode = "edit"),
      v: () => (ws.mode = "view"),
      n: () => (theme = theme === "chart-room" ? "night-chart" : "chart-room"),
      t: () => document.querySelector<HTMLElement>("[role=slider]")?.focus(),
      c: () => document.querySelector<HTMLElement>(".area-chart [role=button][aria-pressed=true], .area-chart [role=button]")?.focus(),
      "]": () => cycle(1),
      "[": () => cycle(-1),
      "?": () => help?.showModal(),
      Escape: () => ws.select(null),
    };
    const f = keys[e.key];
    if (f) {
      e.preventDefault();
      f();
    }
  }
</script>

<svelte:window {onkeydown} />

<div class="app">
  <header class="bar">
    <h1 class="title">{scene.title}</h1>
    <div class="kicker">{scene.subtitle} · {formatYears(ws.now_yr)}</div>
    <div class="spacer"></div>
    <Segmented
      label="Mode"
      value={ws.mode}
      options={[
        { value: "edit", label: "Edit", key: "E" },
        { value: "view", label: "View", key: "V" },
      ]}
      onchange={(v) => (ws.mode = v)}
    />
    <div class="themes">
      <Segmented
        label="Chart style"
        value={theme}
        options={[
          { value: "chart-room", label: "Chart room", key: "N" },
          { value: "night-chart", label: "Night chart", key: "N" },
        ]}
        onchange={(v) => (theme = v)}
      />
    </div>
    <button type="button" class="night" aria-label="Switch between Chart room and Night chart" onclick={() => (theme = theme === "chart-room" ? "night-chart" : "chart-room")}>
      <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" aria-hidden="true"><path d="M20 14.5A8 8 0 0 1 9.5 4a8 8 0 1 0 10.5 10.5z" /></svg>
    </button>
  </header>

  {#if !screen}
    <main class="problem">
      <h2>The workspace document is not valid</h2>
      <ul>{#each checked.ok ? [] : checked.errors as e (e)}<li>{e}</li>{/each}</ul>
    </main>
  {:else if failure}
    <main class="problem" role="alert">
      <h2>The kernel did not start</h2>
      <p>{failure}</p>
    </main>
  {:else if !session}
    <main class="problem" aria-busy="true"><p>Opening the chart…</p></main>
  {:else}
    <main class="stage">
      <div class="area-chart">{#if mountFor("chart")}<div class="fill" use:view={mountFor("chart")!}></div>{/if}</div>
      <nav class="tools" aria-label="Tools">
        <ToolButton label="Select" icon="M5 3l14 8-6 2-2 6z" pressed={true} />
        <ToolButton label="Keyboard shortcuts" icon="M3 7h18v10H3zM7 11h1M11 11h1M15 11h2M7 14h10" keyhint="?" onclick={() => help?.showModal()} />
      </nav>
      <aside class="area-card" aria-label="Selected body">{#if mountFor("card")}<div use:view={mountFor("card")!}></div>{/if}</aside>
    </main>
    <footer class="area-ruler" aria-label="Time">{#if mountFor("ruler")}<div class="fill" use:view={mountFor("ruler")!}></div>{/if}</footer>
    <div class="status" aria-label="Session">
      <span>Seat: {session.seat.label}</span>
      <span>kernel {status.kernel}</span>
      <span title="Measured capability tier of this device">tier {status.tier}</span>
      <span title="State hash: {status.hash}" data-state-hash={status.hash}>state {status.hash.slice(0, 10)}</span>
    </div>
  {/if}
</div>

<dialog bind:this={help} class="help" aria-labelledby="help-title">
  <h2 id="help-title">Keyboard</h2>
  <dl>
    <dt>Tab</dt><dd>Move between the chart's bodies, the card and the ruler</dd>
    <dt>Arrows</dt><dd>On the chart, the next or previous body; on the ruler, 1 % of the span</dd>
    <dt>Enter</dt><dd>Select the focused body; on the card, ink the typed value</dd>
    <dt>Esc</dt><dd>Put a typed value back, or clear the selection</dd>
    <dt>[ and ]</dt><dd>Previous or next body</dd>
    <dt>C, T</dt><dd>Go to the chart, go to the time ruler</dd>
    <dt>Page Up, Page Down, Home, End</dt><dd>Larger steps on the ruler</dd>
    <dt>E, V</dt><dd>Edit or view</dd>
    <dt>N</dt><dd>Chart room or Night chart</dd>
  </dl>
  <form method="dialog"><button type="submit">Close</button></form>
</dialog>

<style>
  :global(html, body) {
    margin: 0;
    background: var(--sse-ground);
    color: var(--sse-ink);
    font-family: var(--sse-font-body);
  }
  :global(*:focus-visible) {
    outline: 2px solid var(--sse-focus);
    outline-offset: 2px;
  }
  .app {
    display: grid;
    grid-template-rows: 64px minmax(0, 1fr) 76px 28px;
    height: 100vh;
    height: 100dvh;
    overflow: hidden;
  }
  .bar {
    display: flex;
    align-items: center;
    gap: 20px;
    padding: 0 24px;
    border-bottom: var(--sse-line) solid var(--sse-ink);
    background: var(--sse-ground);
    min-width: 0;
  }
  .title {
    margin: 0;
    font-family: var(--sse-font-display);
    font-size: 26px;
    font-weight: 600;
  }
  .kicker {
    font-size: 12px;
    color: var(--sse-ink-muted);
    letter-spacing: 0.08em;
    text-transform: uppercase;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .spacer {
    flex: 1;
  }
  .night {
    display: none;
  }
  .stage {
    position: relative;
    min-height: 0;
  }
  .area-chart,
  .fill {
    position: absolute;
    inset: 0;
  }
  .area-chart .fill,
  .area-ruler .fill {
    position: relative;
    height: 100%;
  }
  .tools {
    position: absolute;
    left: 24px;
    top: 24px;
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .area-card {
    position: absolute;
    right: 24px;
    top: 24px;
    width: 380px;
    max-height: calc(100% - 48px);
    overflow: auto;
    padding: 0 6px 6px 0;
  }
  .area-ruler {
    border-top: var(--sse-line) solid var(--sse-ink);
    min-width: 0;
  }
  .status {
    display: flex;
    gap: 16px;
    align-items: center;
    padding: 0 24px;
    border-top: var(--sse-line) solid var(--sse-rule);
    background: var(--sse-ground-raised);
    font-family: var(--sse-font-mono);
    font-size: 11px;
    color: var(--sse-ink-muted);
    white-space: nowrap;
    overflow: hidden;
  }
  .problem {
    padding: 48px 24px;
    max-width: 640px;
  }
  .problem h2 {
    font-family: var(--sse-font-display);
  }
  .help {
    max-width: 520px;
    border: var(--sse-line) solid var(--sse-ink);
    box-shadow: var(--sse-card-shadow);
    background: var(--sse-ground-raised);
    color: var(--sse-ink);
    padding: 20px 24px;
  }
  .help::backdrop {
    background: rgba(0, 0, 0, 0.25);
  }
  .help h2 {
    margin: 0 0 12px;
    font-family: var(--sse-font-display);
  }
  .help dl {
    display: grid;
    grid-template-columns: auto 1fr;
    gap: 6px 16px;
    margin: 0 0 16px;
    font-size: 14px;
  }
  .help dt {
    font-family: var(--sse-font-mono);
    font-size: 13px;
  }
  .help dd {
    margin: 0;
  }
  .help button {
    height: 40px;
    padding: 0 16px;
    border: var(--sse-line) solid var(--sse-ink);
    background: var(--sse-ink);
    color: var(--sse-on-ink);
    font-family: var(--sse-font-body);
  }

  /* Phone: the same card and ruler without the wide chart (H8.5). */
  @media (max-width: 760px) {
    .app {
      grid-template-rows: 56px auto auto 64px auto;
      height: auto;
      min-height: 100dvh;
      overflow: visible;
    }
    .bar {
      gap: 10px;
      padding: 0 12px;
    }
    .title {
      font-size: 22px;
    }
    .kicker,
    .themes,
    .tools {
      display: none;
    }
    .night {
      display: flex;
      align-items: center;
      justify-content: center;
      width: 44px;
      height: 44px;
      border: var(--sse-line) solid var(--sse-ink);
      background: transparent;
      color: var(--sse-ink);
    }
    .stage {
      display: contents;
    }
    .area-chart {
      position: relative;
      height: 44vh;
      border-bottom: var(--sse-line) solid var(--sse-ink);
    }
    .area-chart .fill {
      position: absolute;
    }
    .area-card {
      position: static;
      width: auto;
      max-height: none;
      overflow: visible;
      margin: 16px 16px 24px;
      padding: 0 6px 6px 0;
    }
    .area-ruler {
      position: sticky;
      bottom: 0;
      z-index: 2;
    }
    .status {
      flex-wrap: wrap;
      gap: 4px 12px;
      padding: 6px 12px;
    }
  }
</style>

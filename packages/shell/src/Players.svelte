<!--
  Player screen host (V1 preview). Mounts the player view stack from a screen
  document onto a scene built for the "everyone" audience. Until the kernel
  exposes K9's scene op, the scenes are K9's own verified outputs for Kestrel
  before and after one press of Scanned, and the page says so. V3 turns this
  into the general screens runtime; V4 brings scenes over the session.
-->
<script lang="ts">
  import { checkPlayerScene, checkScreen, subjectName, type PlayerContext, type PlayerScene, type ScreenDoc, type Vocab, type ViewHandle } from "../../ui-contract/src/index.ts";
  import { PLAYER_STACK } from "../../views/player-stack/src/index.ts";
  import { themeCss, THEMES } from "../../skins/chart-room/src/index.ts";
  import { Segmented } from "../../chart-grammar/src/index.ts";
  import screenDoc from "./documents/players.screen.json";
  import vocabDoc from "./documents/vocab.plain.json";
  import { loadPref, savePref } from "./prefs.ts";

  const vocab = vocabDoc as unknown as Vocab;
  const checked = checkScreen(screenDoc as unknown as ScreenDoc, PLAYER_STACK);

  // Scenes are data fetched like any other (scenes/ is filled at build from K9's verified outputs).
  let scenes = $state<Record<string, PlayerScene> | null>(null);
  let sceneErrors = $state<string[]>([]);
  Promise.all(["before", "after"].map(async (k) => [k, await (await fetch(new URL(`scenes/tarn-${k}.json`, document.baseURI))).json()] as const))
    .then((pairs) => {
      sceneErrors = pairs.map(([, s]) => checkPlayerScene(s)).filter((e): e is string => !!e);
      scenes = Object.fromEntries(pairs);
    })
    .catch((e) => (sceneErrors = [String(e)]));

  let which = $state<"before" | "after">("before");
  const scene = $derived(scenes?.[which]);
  let selection = $state<string | null>(null);
  let theme = $state(loadPref("theme") === "night-chart" ? "night-chart" : "chart-room");
  $effect(() => {
    const root = document.documentElement;
    root.setAttribute("style", themeCss(theme));
    root.style.colorScheme = THEMES.find((t) => t.id === theme)?.mode ?? "light";
    root.dataset.theme = theme;
    savePref("theme", theme);
  });

  const handles: ViewHandle<PlayerScene>[] = [];
  $effect(() => {
    const s = scene;
    if (s) for (const h of handles) h.update(s);
  });

  function view(node: HTMLElement, m: { module: (typeof PLAYER_STACK)[number]; params: PlayerContext["params"] }) {
    const ctx: PlayerContext = { params: m.params, vocab, selection: () => selection, select: (a) => (selection = a) };
    const h = m.module.mount(node, ctx, scene!);
    handles.push(h);
    return { destroy: () => (handles.splice(handles.indexOf(h), 1), h.dispose()) };
  }
  const star = $derived(scene?.subjects.find((s) => s.address === scene.scope));
</script>

<div class="page">
  <header class="bar">
    <h1>{star ? subjectName(star) : "System"}</h1>
    <span class="kicker">{screenDoc.label} · audience {scene?.audience ?? "…"}</span>
    <span class="spacer"></span>
    <Segmented
      label="Sensor data"
      value={which}
      options={[
        { value: "before", label: "Long range" },
        { value: "after", label: "Scanned" },
      ]}
      onchange={(v) => (which = v)}
    />
    <Segmented
      label="Theme"
      value={theme}
      options={[
        { value: "chart-room", label: "Chart" },
        { value: "night-chart", label: "Night" },
      ]}
      onchange={(v) => (theme = v)}
    />
  </header>
  {#if !checked.ok || sceneErrors.length}
    <main class="problem" role="alert"><h2>This screen cannot be shown</h2><ul>{#each [...(checked.ok ? [] : checked.errors), ...sceneErrors] as e (e)}<li>{e}</li>{/each}</ul></main>
  {:else if !scene}
    <main class="problem" aria-busy="true"><p>Waiting for the scene…</p></main>
  {:else}
    <main class="grid">
      {#each checked.screen.mounts as m (m.area)}
        <section class="area area-{m.area}" data-view={m.module.id}><div class="fill" use:view={m}></div></section>
      {/each}
    </main>
  {/if}
  <footer class="status">Scene: K9 scene builder output for Kestrel (fixture until the kernel's scene op is wired) · the Scanned control stands in for the GM's button</footer>
</div>

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
  .page {
    min-height: 100dvh;
    display: grid;
    grid-template-rows: auto 1fr auto;
  }
  .bar {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 10px 20px;
    padding: 12px 24px;
    border-bottom: var(--sse-line) solid var(--sse-ink);
  }
  h1 {
    margin: 0;
    font-family: var(--sse-font-display);
    font-size: 26px;
    font-weight: 600;
  }
  .kicker {
    font-size: 12px;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: var(--sse-ink-muted);
  }
  .spacer {
    flex: 1;
  }
  .grid {
    display: grid;
    gap: 20px;
    padding: 20px 24px;
    grid-template-columns: minmax(0, 1.4fr) minmax(0, 1fr) minmax(0, 1fr);
    grid-template-areas:
      "a a b"
      "a a b"
      "c d b"
      "e e e"
      "f f f";
    align-items: start;
  }
  .area-a { grid-area: a; height: 460px; border: var(--sse-line) solid var(--sse-ink); }
  .area-b { grid-area: b; }
  .area-c { grid-area: c; }
  .area-d { grid-area: d; }
  .area-e { grid-area: e; }
  .area-f { grid-area: f; }
  .area-a .fill {
    height: 100%;
  }
  .status {
    padding: 8px 24px;
    border-top: var(--sse-line) solid var(--sse-rule);
    font-family: var(--sse-font-mono);
    font-size: 11px;
    color: var(--sse-ink-muted);
  }
  .problem {
    padding: 24px;
  }
  @media (max-width: 760px) {
    .bar {
      padding: 10px 12px;
    }
    .grid {
      grid-template-columns: minmax(0, 1fr);
      grid-template-areas: "a" "b" "c" "d" "e" "f";
      padding: 12px;
      gap: 14px;
    }
    .area-a {
      height: 320px;
    }
  }
</style>

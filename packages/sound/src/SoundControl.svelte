<!--
  The small sound control on every screen (N1, sound design 6). Off by
  default; the engine is made only when the user turns sound on, which is
  also the user action browsers require before audio may start. Settings stay
  on this device: they never reach the session, so the state hash cannot
  change. Every source shows its label (real, translated, designed) and its
  Why; captions say in words what is sounding.
-->
<script lang="ts">
  import { SOUND_LABELS, SOUND_LAYERS, readSound, score, scoreHash, type SoundScene, type SoundSettings } from "../../ui-contract/src/index.ts";
  import { createEngine, loadRecipes, type Engine } from "./engine.ts";
  import orbitTones from "./recipes/orbit-tones.json";

  let { scene, deviceTier, seed = "", key = "sound" }: { scene: SoundScene; deviceTier?: string; seed?: string; key?: string } = $props();

  const store = $derived(`sse4.${key}`);
  function load(): SoundSettings {
    try {
      return readSound(localStorage.getItem(store), deviceTier);
    } catch {
      return readSound(null, deviceTier);
    }
  }
  let st = $state<SoundSettings>(load());
  $effect(() => {
    const s = JSON.stringify($state.snapshot(st));
    try {
      localStorage.setItem(store, s);
    } catch {
      /* private window: settings last for this page only */
    }
  });

  const { recipes, problems } = loadRecipes([orbitTones]);
  let engine: Engine | null = null;
  let silent = $state<string | null>(null);
  let started = $state(false);
  let open = $state(false);
  let why = $state<string | null>(null);

  // Off by default. Even a saved "on" waits for a user action here before any audio is made.
  function turn(on: boolean) {
    st.on = on;
    if (on && !engine && st.tier !== "silent") {
      const e = createEngine(recipes, $state.snapshot(st) as SoundSettings, seed);
      if ("silent" in e) silent = e.silent;
      else ((engine = e), (started = true));
    }
    engine?.set($state.snapshot(st) as SoundSettings);
    if (on) engine?.play(scene);
  }
  $effect(() => {
    const s = $state.snapshot(st) as SoundSettings;
    engine?.set(s);
  });
  // A saved "on" starts at the user's first touch or key on the page, never by itself.
  $effect(() => {
    if (!st.on || engine) return;
    const start = () => turn(true);
    window.addEventListener("pointerdown", start, { once: true });
    window.addEventListener("keydown", start, { once: true });
    return () => {
      window.removeEventListener("pointerdown", start);
      window.removeEventListener("keydown", start);
    };
  });
  $effect(() => {
    const sc = scene;
    if (engine && st.on) engine.play(sc);
  });
  $effect(() => () => engine?.close());

  // The score this device would play first: the same on every device for the same view and dial.
  const first = $derived(scoreHash(score(scene, 0, st.dial, seed)));

  // Captions: what is sounding now, in words.
  let now = $state<string>("");
  $effect(() => {
    if (!st.on || !st.captions) return void (now = "");
    const t = setInterval(() => {
      const playing = engine?.sounding() ?? [];
      const names = [...new Set(playing.map((n) => scene.sources.find((s) => s.id === n.source)?.name).filter(Boolean))];
      now = names.length ? `Tones of ${names.join(", ")}` : scene.caption;
    }, 500);
    return () => clearInterval(t);
  });
  const id = $props.id();
</script>

<svelte:window onkeydown={(e) => open && e.key === "Escape" && (open = false)} />

<span class="sound" data-sound={st.on ? "on" : "off"} data-started={started} data-score-hash={first} data-sources={scene.sources.length}>
  <button type="button" class="toggle" aria-expanded={open} aria-controls="{id}-panel" onclick={() => (open = !open)} title="Sound (off by default)">
    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" aria-hidden="true"><path d="M4 9h4l5-4v14l-5-4H4z" />{#if st.on}<path d="M16 9a4 4 0 0 1 0 6M18.5 6.5a8 8 0 0 1 0 11" />{:else}<path d="M17 9l5 6M22 9l-5 6" />{/if}</svg>
    Sound {st.on ? (started ? "on" : "on (starts at your first tap)") : "off"}
  </button>
  {#if st.on && st.captions && now}<span class="caption" role="status" aria-live="polite">{now}</span>{/if}
  {#if open}
    <div class="panel" id="{id}-panel" role="group" aria-label="Sound">
      <label class="check"><input type="checkbox" checked={st.on} onchange={(e) => turn(e.currentTarget.checked)} /> Sound on</label>
      {#if silent}<p class="note" role="alert">{silent}</p>{/if}
      {#each problems as p (p)}<p class="note" role="alert">Recipe refused: {p}</p>{/each}
      <label>Device sound tier
        <select bind:value={st.tier} aria-label="Device sound tier">
          <option value="silent">Silent</option>
          <option value="basic">Basic (6 voices, no echo)</option>
          <option value="full">Full</option>
        </select>
      </label>
      <fieldset>
        <legend>Mixer</legend>
        {#each SOUND_LAYERS as l (l)}
          <label class="slider">{l[0]!.toUpperCase() + l.slice(1)}<input type="range" min="0" max="1" step="0.05" bind:value={st.mixer[l]} aria-label="{l} level" /></label>
        {/each}
      </fieldset>
      <label class="slider">Music: true ratios to settled<input type="range" min="0" max="1" step="0.05" bind:value={st.dial} aria-label="True ratios to settled" /></label>
      <p class="note">{st.dial === 0 ? "True ratios: exactly the physics, sometimes strange." : `Settled ${Math.round(st.dial * 100)} %: notes pulled toward a calm scale, so no longer exact.`}</p>
      <label class="check"><input type="checkbox" bind:checked={st.captions} /> Captions</label>
      <div class="k">Sources in view</div>
      {#if scene.sources.length}
        <ul class="sources">
          {#each scene.sources as s (s.id)}
            <li>
              <span>{s.name}</span>
              <span class="tag">{SOUND_LABELS[s.label]}</span>
              <button type="button" class="linkbtn" aria-label="Why does {s.name} sound like this?" onclick={() => (why = why === s.id ? null : s.id)}>why?</button>
              {#if why === s.id}<p class="why">{s.why}</p>{/if}
            </li>
          {/each}
        </ul>
      {:else}
        <p class="note">{scene.caption}</p>
      {/if}
    </div>
  {/if}
</span>

<style>
  .sound {
    position: relative;
    display: inline-flex;
    align-items: center;
    gap: 10px;
  }
  .toggle,
  .linkbtn {
    font: inherit;
    color: inherit;
    background: none;
    border: 0;
    padding: 0;
    cursor: pointer;
    display: inline-flex;
    gap: 4px;
    align-items: center;
  }
  .toggle {
    min-height: 24px;
  }
  .linkbtn {
    color: var(--sse-accent, currentColor);
    text-decoration: underline;
  }
  .caption {
    font-style: italic;
    color: var(--sse-ink-muted, currentColor);
  }
  .panel {
    position: fixed;
    bottom: 44px;
    right: 16px;
    z-index: 10;
    width: min(340px, calc(100vw - 32px));
    max-height: 70vh;
    overflow: auto;
    display: grid;
    gap: 8px;
    padding: 12px 14px;
    background: var(--sse-paper, var(--sse-ground, Canvas));
    color: var(--sse-ink, CanvasText);
    border: var(--sse-line, 1px) solid var(--sse-ink, CanvasText);
    box-shadow: 4px 4px 0 var(--sse-ink, CanvasText);
    font-family: var(--sse-font-body, inherit);
    font-size: 13px;
    text-align: left;
    white-space: normal;
  }
  .check {
    display: flex;
    gap: 8px;
    align-items: center;
  }
  .slider {
    display: grid;
    gap: 2px;
  }
  input {
    accent-color: var(--sse-accent, auto);
  }
  select {
    display: block;
    margin-top: 2px;
    font: inherit;
    background: transparent;
    color: inherit;
  }
  fieldset {
    border: var(--sse-line, 1px) solid var(--sse-rule, currentColor);
    display: grid;
    gap: 4px;
    padding: 4px 8px 8px;
  }
  legend,
  .k {
    font-size: 11px;
    color: var(--sse-ink-muted, currentColor);
    letter-spacing: 0.06em;
    text-transform: uppercase;
  }
  .note {
    margin: 0;
    font-size: 12px;
    color: var(--sse-ink-muted, currentColor);
  }
  .sources {
    list-style: none;
    margin: 0;
    padding: 0;
    display: grid;
    gap: 4px;
  }
  .sources li {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    align-items: baseline;
  }
  .tag {
    font-size: 10px;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    border: 1px solid currentColor;
    padding: 0 4px;
  }
  .why {
    flex-basis: 100%;
    margin: 2px 0 6px;
    font-size: 12px;
    line-height: 1.45;
  }
</style>

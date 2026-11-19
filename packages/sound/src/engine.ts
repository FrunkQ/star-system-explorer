// The sound engine (N1): Web Audio in the browser. It turns a SoundScene's
// score into sound through data-only recipes, behind a four-layer mixer.
// Nothing is created until the user turns sound on (browsers also require a
// user action before audio may start), and nothing it makes is read back.
import { SOUND_LAYERS, checkRecipe, score, voiceLimit, type ScoreNote, type SoundLayer, type SoundRecipe, type SoundScene, type SoundSettings } from "../../ui-contract/src/index.ts";

export interface Engine {
  /** Start or follow a scene. Notes already scheduled ring out. */
  play(scene: SoundScene): void;
  /** Apply new settings: mixer, dial, tier. Off stops everything. */
  set(settings: SoundSettings): void;
  /** The notes sounding now, newest first, for captions. */
  sounding(): ScoreNote[];
  /** Why the engine is silent, or null when it is not. */
  silentBecause(): string | null;
  close(): void;
}

type Ctx = AudioContext;

/** Recipes the engine may play; a recipe outside its schema is refused here, before any sound. */
export function loadRecipes(docs: readonly unknown[]): { recipes: Map<string, SoundRecipe>; problems: string[] } {
  const recipes = new Map<string, SoundRecipe>();
  const problems: string[] = [];
  for (const d of docs) {
    const e = checkRecipe(d);
    if (e.length) problems.push(`${(d as { id?: string })?.id ?? "a recipe"}: ${e.join("; ")}`);
    else recipes.set((d as SoundRecipe).id, d as SoundRecipe);
  }
  return { recipes, problems };
}

/**
 * Create the engine on a user action. Returns null with a reason when this
 * browser cannot make sound (the "silent" tier, shown to the user).
 */
export function createEngine(recipes: Map<string, SoundRecipe>, settings: SoundSettings, seed = ""): Engine | { silent: string } {
  const AC = (globalThis as { AudioContext?: typeof AudioContext; webkitAudioContext?: typeof AudioContext }).AudioContext ?? (globalThis as { webkitAudioContext?: typeof AudioContext }).webkitAudioContext;
  if (!AC) return { silent: "This browser has no Web Audio, so sound is off (silent tier)." };
  const ctx: Ctx = new AC();
  void ctx.resume();
  const master = ctx.createGain();
  master.gain.value = 0.9;
  master.connect(ctx.destination);
  const layers = Object.fromEntries(
    SOUND_LAYERS.map((l) => {
      const g = ctx.createGain();
      g.connect(master);
      return [l, g];
    }),
  ) as Record<SoundLayer, GainNode>;

  let st = settings;
  let scene: SoundScene | null = null;
  let started = ctx.currentTime;
  let window = -1;
  let timer: ReturnType<typeof setInterval> | null = null;
  const live: { note: ScoreNote; from: number; until: number; nodes: AudioNode[] }[] = [];

  function applyMixer() {
    for (const l of SOUND_LAYERS) layers[l].gain.setTargetAtTime(st.on ? st.mixer[l] : 0, ctx.currentTime, 0.2);
  }
  applyMixer();

  function noiseBuffer(colour: string): AudioBuffer {
    const n = ctx.sampleRate * 2;
    const b = ctx.createBuffer(1, n, ctx.sampleRate);
    const d = b.getChannelData(0);
    // Keyed, not random: the same noise on every run (nothing reads it back, but there is no reason to differ).
    let x = 0x9e3779b9;
    let last = 0;
    for (let i = 0; i < n; i++) {
      x ^= x << 13;
      x ^= x >>> 17;
      x ^= x << 5;
      const white = ((x >>> 0) / 4294967296) * 2 - 1;
      last = colour === "white" ? white : colour === "pink" ? 0.97 * last + 0.03 * white * 4 : 0.995 * last + 0.005 * white * 12;
      d[i] = colour === "white" ? white : last;
    }
    return b;
  }

  /** Build one note of a recipe at absolute time `at`. */
  function voice(r: SoundRecipe, note: ScoreNote, at: number): AudioNode[] {
    const made = new Map<string, AudioNode>();
    const sources: AudioScheduledSourceNode[] = [];
    const skip = new Set(st.tier === "full" ? [] : (r.full_only ?? []));
    let release = 1;
    for (const n of r.nodes) {
      if (skip.has(n.id)) continue;
      const s = n as Record<string, unknown>;
      switch (n.type) {
        case "osc": {
          const o = ctx.createOscillator();
          o.type = (s.wave as OscillatorType) ?? "sine";
          o.frequency.value = note.freq * ((s.ratio as number) ?? 1);
          o.detune.value = (s.detune_cents as number) ?? 0;
          sources.push(o);
          made.set(n.id, o);
          break;
        }
        case "noise": {
          const o = ctx.createBufferSource();
          o.buffer = noiseBuffer((s.colour as string) ?? "white");
          o.loop = true;
          sources.push(o);
          made.set(n.id, o);
          break;
        }
        case "filter": {
          const f = ctx.createBiquadFilter();
          f.type = (s.kind as BiquadFilterType) ?? "lowpass";
          f.frequency.value = (s.freq_hz as number) ?? 1000;
          f.Q.value = (s.q as number) ?? 0.7;
          made.set(n.id, f);
          break;
        }
        case "env": {
          const g = ctx.createGain();
          const a = (s.attack_s as number) ?? 0.01;
          release = (s.release_s as number) ?? 1;
          const hold = Math.max(at + a, at + note.dur - release);
          g.gain.setValueAtTime(0, at);
          g.gain.linearRampToValueAtTime(note.level, at + a);
          g.gain.setValueAtTime(note.level, hold);
          g.gain.linearRampToValueAtTime(0, hold + release);
          made.set(n.id, g);
          break;
        }
        case "gain": {
          const g = ctx.createGain();
          g.gain.value = (s.gain as number) ?? 1;
          made.set(n.id, g);
          break;
        }
        case "delay": {
          const d = ctx.createDelay(4);
          d.delayTime.value = (s.time_s as number) ?? 0.5;
          const fb = ctx.createGain();
          fb.gain.value = (s.feedback as number) ?? 0;
          d.connect(fb);
          fb.connect(d);
          made.set(n.id, d);
          break;
        }
      }
    }
    for (const [a, b] of r.wires) {
      const from = made.get(a);
      const to = b === "out" ? layers[r.layer] : made.get(b);
      if (from && to) from.connect(to);
    }
    const end = at + Math.max(note.dur, 0.1) + release + 0.5;
    for (const s of sources) {
      s.start(at);
      s.stop(end);
    }
    sources[0]?.addEventListener("ended", () => {
      for (const n of made.values()) n.disconnect();
    });
    return [...made.values()];
  }

  function schedule(w: number) {
    if (!scene || !st.on) return;
    const notes = score(scene, w, st.dial, seed);
    const limit = voiceLimit(st.tier);
    const base = started + w * scene.window_s;
    for (const n of notes) {
      const src = scene.sources.find((s) => s.id === n.source);
      const r = src && recipes.get(src.recipe);
      if (!r) continue;
      const at = base + n.t;
      if (at < ctx.currentTime) continue;
      // The tier's voice limit, counted over notes sounding at once.
      const busy = live.filter((x) => x.from <= at && x.until > at).length;
      if (busy >= limit) continue;
      live.push({ note: n, from: at, until: at + n.dur, nodes: voice(r, n, at) });
    }
  }

  function tick() {
    if (!scene || !st.on) return;
    const now = ctx.currentTime - started;
    const w = Math.floor(now / scene.window_s);
    // Keep the next window scheduled a little ahead.
    while (window < w + 1) schedule(++window);
    for (let i = live.length - 1; i >= 0; i--) if (live[i]!.until + 10 < ctx.currentTime) live.splice(i, 1);
  }

  function stopAll() {
    for (const x of live) for (const n of x.nodes) n.disconnect();
    live.length = 0;
  }

  return {
    play(next) {
      const changed = !scene || scene.vantage !== next.vantage || scene.sources.map((s) => s.id + s.freq_true).join() !== next.sources.map((s) => s.id + s.freq_true).join();
      scene = next;
      if (!changed) return;
      stopAll();
      started = ctx.currentTime + 0.1;
      window = -1;
      tick();
      timer ??= setInterval(tick, 1000);
    },
    set(next) {
      const was = st;
      st = next;
      applyMixer();
      if (!next.on) stopAll();
      if (next.on && (!was.on || was.dial !== next.dial || was.tier !== next.tier)) {
        stopAll();
        started = ctx.currentTime + 0.1;
        window = -1;
        void ctx.resume();
        tick();
      }
    },
    sounding() {
      const now = ctx.currentTime;
      return live.filter((x) => x.from <= now && x.until > now).map((x) => x.note).reverse();
    },
    silentBecause() {
      if (!st.on) return "Sound is off.";
      if (st.tier === "silent") return "Sound tier is silent on this device.";
      if (ctx.state !== "running") return "The browser has not started audio yet.";
      return null;
    },
    close() {
      if (timer) clearInterval(timer);
      stopAll();
      void ctx.close();
    },
  };
}

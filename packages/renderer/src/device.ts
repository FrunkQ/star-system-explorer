// The device as the renderer sees it (VP4): its capabilities and measured speed, read once a page and shared by
// every view; the visit's heat watch; the person's Quality choice. Views read placement from here through probe.ts
// `place()`. Presentation only: nothing here reaches the kernel, and nothing is sent anywhere.
import { HeatWatch, loadProbe, loadQuality, probeKey, saveProbe, saveQuality, type Capabilities, type ProbeResult, type QualityChoice } from "./probe.ts";

/** The device's capabilities and measured speed (VP4 probe.ts): read once a page, the speed cached on the device. */
let capsRead: Promise<Capabilities> | null = null;
let speedProbe: Promise<ProbeResult | null> | null = null;
export const speedListeners = new Set<() => void>();
/** The heat watch is the visit's, not one view's: a hot device stays lowered on every view. */
export const heat = new HeatWatch();
let lastSpeed: ProbeResult | null = null;

const storage = (): Storage | null => {
  try {
    return typeof localStorage !== "undefined" ? localStorage : null;
  } catch {
    return null;
  }
};

export async function deviceCaps(): Promise<Capabilities> {
  capsRead ??= import("./probe-gpu.ts").then((m) => m.readCapabilities());
  return capsRead;
}

/** The measured speed: the cached probe for this adapter and browser, else null while one is measured in the background. */
export async function deviceSpeed(caps: Capabilities): Promise<ProbeResult | null> {
  const key = probeKey(caps, typeof navigator !== "undefined" ? navigator.userAgent : "");
  const kept = loadProbe(storage(), key);
  if (kept) return (lastSpeed = kept);
  if (!speedProbe) {
    speedProbe = import("./probe-gpu.ts")
      .then((m) => m.measureProbe(caps))
      .then((r) => {
        if (r) (saveProbe(storage(), { ...r, key }), (lastSpeed = r));
        for (const l of speedListeners) l();
        return r;
      })
      .catch(() => null);
  }
  return null;
}

/** Forget the measured speed and measure again (More › Performance › Measure again). Resolves with the new result. */
export async function measureAgain(): Promise<ProbeResult | null> {
  const caps = await deviceCaps();
  try {
    storage()?.removeItem("sse4.render.probe");
  } catch {
    // Nothing kept.
  }
  speedProbe = null;
  await deviceSpeed(caps);
  return (await speedProbe) ?? null;
}

/** The quality choice (More › Performance): kept on the device, and every open view is placed again at once. */
export function setQuality(q: QualityChoice): void {
  saveQuality(storage(), q);
  for (const l of speedListeners) l();
}

/** The Quality choice now (Auto unless the person chose). */
export const quality = (): QualityChoice => loadQuality(storage());

/** The device's capabilities and its cached speed, for the performance readout. */
export async function deviceReadout(): Promise<{ caps: Capabilities; probe: ProbeResult | null }> {
  const caps = await deviceCaps();
  return { caps, probe: loadProbe(storage(), probeKey(caps, typeof navigator !== "undefined" ? navigator.userAgent : "")) };
}

/** The speed now, if it has been measured (the readout's estimated shares use it). */
export const speedNow = (): ProbeResult | null => lastSpeed;

/** Told when the speed is measured or the Quality choice changes: a view places itself again. Returns the unsubscribe. */
export function onPlacementChange(fn: () => void): () => void {
  speedListeners.add(fn);
  return () => speedListeners.delete(fn);
}

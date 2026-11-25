// Motion for dynamic minimalism (handover/interface-style.md). Motion explains
// a change of state and never delays input: durations come from the theme's
// tokens (--sse-motion-quick, --sse-motion, --sse-ease), and a reduced-motion
// setting makes all of them zero. Only transform and opacity are animated.
import { drawn } from "../../ui-contract/src/index.ts";

/** True when the person has asked for less motion. */
export function reducedMotion(): boolean {
  return typeof matchMedia === "function" && matchMedia("(prefers-reduced-motion: reduce)").matches;
}

/** A motion token in milliseconds, read where `el` sits (themes can differ by panel); 0 under reduced motion. */
export function motionMs(el: Element | null, token: "motion" | "motion-quick" = "motion"): number {
  if (!el || reducedMotion() || typeof getComputedStyle !== "function") return 0;
  const v = getComputedStyle(el).getPropertyValue(`--sse-${token}`).trim();
  const n = parseFloat(v);
  if (!Number.isFinite(n)) return 0;
  return v.endsWith("ms") ? n : v.endsWith("s") ? n * 1000 : n;
}

/** The theme's easing where `el` sits, as a CSS timing function. */
export function easing(el: Element | null): string {
  const v = el && typeof getComputedStyle === "function" ? getComputedStyle(el).getPropertyValue("--sse-ease").trim() : "";
  return v || "cubic-bezier(0.2, 0, 0, 1)";
}

type Source = Element | DOMRect | null | undefined | (() => Element | DOMRect | null | undefined);
const rectOf = (s: Source): DOMRect | null => {
  const v = typeof s === "function" ? s() : s;
  if (!v) return null;
  return v instanceof Element ? v.getBoundingClientRect() : v;
};

/**
 * Where an element seems to come from: it starts a little way towards its
 * source (30 % of the distance), a touch smaller and transparent, and settles.
 * Restrained on purpose: a cue of where it came from, not a flight across the
 * screen. Returns the keyframes, or null when there is nothing to animate.
 */
export function fromKeyframes(node: DOMRect, source: DOMRect | null): Keyframe[] | null {
  const dx = source ? (source.x + source.width / 2 - (node.x + node.width / 2)) * 0.3 : 0;
  const dy = source ? (source.y + source.height / 2 - (node.y + node.height / 2)) * 0.3 : 8;
  return [
    { transform: `translate(${drawn(dx)}px, ${drawn(dy)}px) scale(0.94)`, opacity: 0 },
    { transform: "none", opacity: 1 },
  ];
}

/**
 * Svelte action: the element opens from its source when it mounts (a card
 * from the body it describes, a popup from the button that opened it).
 * `use:opensFrom={() => sourceEl}`.
 */
export function opensFrom(node: HTMLElement, source: Source) {
  const ms = motionMs(node);
  if (ms > 0 && typeof node.animate === "function") {
    const kf = fromKeyframes(node.getBoundingClientRect(), rectOf(source));
    if (kf) node.animate(kf, { duration: ms, easing: easing(node) });
  }
  return {};
}

/**
 * Dialogs open from what opened them. Installs one listener on the document:
 * a click remembers the control pressed; when a <dialog> gains `open`, it
 * grows from that control. Closing is instant (a dialog's close cannot be
 * held back without delaying the person). Returns an uninstaller.
 */
export function tactileDialogs(doc: Document = document): () => void {
  let last: Element | null = null;
  const remember = (e: Event) => {
    const t = e.target as Element | null;
    last = t?.closest?.("button, [role=button], a") ?? t;
  };
  doc.addEventListener("click", remember, true);
  const seen = new WeakSet<Element>();
  const mo = new MutationObserver((records) => {
    for (const r of records) {
      const d = r.target as HTMLDialogElement;
      if (d.tagName !== "DIALOG") continue;
      if (!d.open) {
        seen.delete(d);
        continue;
      }
      if (seen.has(d)) continue;
      seen.add(d);
      const ms = motionMs(d);
      const source = last && !d.contains(last) && last.isConnected ? last.getBoundingClientRect() : null;
      const kf = fromKeyframes(d.getBoundingClientRect(), source);
      if (ms > 0 && kf && typeof d.animate === "function") d.animate(kf, { duration: ms, easing: easing(d) });
    }
  });
  mo.observe(doc.body, { subtree: true, attributes: true, attributeFilter: ["open"] });
  return () => (doc.removeEventListener("click", remember, true), mo.disconnect());
}

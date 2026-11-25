<!--
  One control, drawn in the kit's control set (track AT, AT15; atlas design 11A.4 rule 5). A part declares a control as
  a kind (AT13); the kit's set says how that kind is drawn and lit in its look: Hauler's `[P]RINT` on a command line, a
  glass pill, a console cap, a gadget key, an inked word. One component draws every kind for every part, so controls
  look and behave the same across a kit. Theme-free: only the look's tokens.

  Fixed minimums no set can go under: a 44 px press target on touch (32 px with a mouse), the focus ring, and no motion
  when the reader asks for reduced motion.
-->
<script lang="ts">
  import { controlWords, drawingOf, type ControlSet, type PartControlKind } from "../../ui-contract/src/index.ts";

  let {
    words,
    kind = "button",
    set,
    sets,
    hotkey,
    on = false,
    disabled = false,
    label,
    onclick,
  }: {
    /** The control's plain words ("Print", "Next page"). */
    words: string;
    kind?: PartControlKind;
    /** The kit's control set, by id or as given; absent draws the plain set. */
    set?: string | ControlSet;
    /** Control sets from the pack stack, for a set named by id. */
    sets?: readonly ControlSet[];
    /** The key that presses it, when the set shows keys; else the first letter. */
    hotkey?: string;
    /** Lit: a switch that is on, the chosen choice or tab. */
    on?: boolean;
    disabled?: boolean;
    /** What a screen reader hears, when the words alone are not enough ("Next page: Tarn a"). */
    label?: string;
    onclick?: (e: MouseEvent) => void;
  } = $props();

  const d = $derived(typeof set === "object" ? (set.kinds[kind] ?? drawingOf(set.extends, kind, sets)) : drawingOf(set, kind, sets));
  const shown = $derived(controlWords(words, d, hotkey));
  const pressable = $derived(kind === "switch" || kind === "choice" || kind === "tab");
</script>

<button
  type="button"
  class="kc frame-{d.frame} lit-{d.lit ?? 'fill'}"
  class:on
  data-kit-control={kind}
  data-frame={d.frame}
  aria-pressed={pressable ? on : undefined}
  aria-label={label ?? (shown.text !== words ? words : undefined)}
  aria-keyshortcuts={shown.key}
  {disabled}
  {onclick}>{shown.text}</button>

<style>
  .kc {
    font: inherit;
    min-height: 32px;
    min-width: 32px;
    padding: 4px 12px;
    color: var(--sse-ink);
    background: var(--sse-ground-raised, var(--sse-ground));
    border: 1px solid var(--sse-ink);
    border-radius: var(--sse-radius, 4px);
    cursor: pointer;
    transition:
      background-color var(--sse-motion, 160ms) var(--sse-ease, ease),
      color var(--sse-motion, 160ms) var(--sse-ease, ease),
      box-shadow var(--sse-motion, 160ms) var(--sse-ease, ease);
  }
  @media (pointer: coarse) {
    .kc {
      min-height: 44px;
      min-width: 44px;
    }
  }
  .kc:focus-visible {
    outline: 2px solid var(--sse-focus, var(--sse-accent));
    outline-offset: 2px;
  }
  .kc:disabled {
    opacity: 0.45;
    cursor: default;
  }

  /* Frames: the outline a control wears. */
  .frame-bracket {
    font-family: var(--sse-font-mono, monospace);
    letter-spacing: 0.06em;
    border-color: transparent;
    background: transparent;
    border-radius: 0;
    padding: 4px 8px;
  }
  .frame-pill {
    border-radius: 999px;
    border-color: color-mix(in srgb, var(--sse-accent) 45%, transparent);
    background: color-mix(in srgb, var(--sse-accent) 10%, var(--sse-ground-raised, var(--sse-ground)));
    padding: 4px 16px;
  }
  .frame-cap {
    border: 0;
    border-radius: 999px 4px 4px 999px;
    background: var(--sse-accent);
    color: var(--sse-ground);
    font-weight: 600;
    letter-spacing: 0.08em;
    padding: 6px 18px 6px 20px;
    text-align: right;
  }
  .frame-key {
    border-radius: 6px;
    box-shadow: inset 0 -2px 0 color-mix(in srgb, var(--sse-ink) 35%, transparent);
  }
  .frame-ink {
    border-color: transparent;
    background: transparent;
    padding: 4px 4px;
    text-decoration: underline;
    text-decoration-color: color-mix(in srgb, var(--sse-ink) 35%, transparent);
    text-underline-offset: 3px;
  }

  /* Lit: on, chosen, or held down. */
  .lit-fill.on,
  .lit-fill:active:not(:disabled) {
    background: var(--sse-ink);
    color: var(--sse-ground);
  }
  .frame-cap.lit-fill.on,
  .frame-cap.lit-fill:active:not(:disabled) {
    background: var(--sse-ink);
    color: var(--sse-ground);
  }
  .lit-invert.on,
  .lit-invert:hover:not(:disabled),
  .lit-invert:active:not(:disabled) {
    background: var(--sse-ink);
    color: var(--sse-ground);
  }
  .lit-glow.on,
  .lit-glow:active:not(:disabled) {
    background: var(--sse-accent);
    color: var(--sse-ground);
    box-shadow: 0 0 0 3px color-mix(in srgb, var(--sse-accent) 30%, transparent);
  }
  .lit-underline.on {
    text-decoration-color: var(--sse-accent);
    text-decoration-thickness: 2px;
  }
  .lit-bold.on {
    font-weight: 700;
  }
  .lit-bold:active:not(:disabled) {
    box-shadow: inset 0 1px 0 color-mix(in srgb, var(--sse-ink) 35%, transparent);
    transform: translateY(1px);
  }

  @media (prefers-reduced-motion: reduce) {
    .kc {
      transition: none;
    }
    .lit-bold:active:not(:disabled) {
      transform: none;
    }
  }
</style>

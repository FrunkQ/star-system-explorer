import type { LookDelivery } from "../../../ui-contract/src/index.ts";
// The two GM modes (H8.5). Night chart inherits Chart room and changes only
// the colours, so the structure is the same in both.
export interface ThemeDoc {
  schema: "sse.theme/1";
  id: string;
  name: string;
  /** One line for people choosing a theme. */
  description?: string;
  extends?: string;
  mode: "light" | "dark";
  /** What the theme can carry (H8.4A): symbologies degrade by a stated rule where it cannot. */
  supports: { colours: number | "full"; patterns: boolean; shapes: boolean };
  tokens: Record<string, string | number>;
  /** Post-process filters drawn over the whole screen, in order (V2). Presentation only. */
  filters?: Filter[];
  /** The vocabulary layer a screen in this theme uses unless it names one. */
  vocab?: string;
  /** A palette offered to the colour channel when a symbology asks for "theme palette" (H8.4A). */
  palette?: string[];
  /** How a screen in this look arrives and leaves, one delivery per moment (atlas design 11A.4a). Presentation only. */
  delivery?: LookDelivery;
}

/** One step of a theme's filter chain. Strength is 0 to 1. */
export type Filter =
  | { kind: "scanlines"; strength: number; spacing?: number }
  | { kind: "glow"; strength: number }
  | { kind: "grain"; strength: number }
  | { kind: "vignette"; strength: number }
  | { kind: "tint"; colour: string; strength: number }
  | { kind: "flicker"; strength: number };

export const CHART_ROOM: ThemeDoc = {
  schema: "sse.theme/1",
  id: "chart-room",
  name: "Chart room",
  mode: "light",
  supports: { colours: "full", patterns: true, shapes: true },
  tokens: {
    ground: "#EFEAE0",
    "ground-raised": "#F7F3EA",
    ink: "#1E2A33",
    "ink-muted": "#56626C",
    "ink-faint": "#7D8790",
    grid: "#D9D1C2",
    rule: "#D9D1C2",
    accent: "#B4471F",
    "accent-ink": "#8A3612",
    "on-ink": "#EFEAE0",
    "field-out": "#B35A12",
    "field-in": "#2D64A0",
    "origin-big-bang": "#BFD4E6",
    "origin-cosmic-rays": "#8877DD",
    "origin-low-mass-stars": "#E3B23C",
    "origin-massive-stars": "#2F6F8F",
    "origin-white-dwarfs": "#7FA06A",
    "origin-neutron-star-mergers": "#6B2A52",
    "origin-human-made": "#C9C1B2",
    // The map palette (territory overlays OV4): eight colours for map layers, tuned to this ground, each kept ΔE 10
    // or more from every other for every colour vision (colour-vision.test) and at least 3:1 against the ground.
    "map-1": "#CB383C",
    "map-2": "#1C55A4",
    "map-3": "#437962",
    "map-4": "#87820B",
    "map-5": "#795482",
    "map-6": "#329290",
    "map-7": "#630A70",
    "map-8": "#416019",
    zone: "rgba(127, 168, 142, 0.32)",
    "zone-ink": "#3F6B50",
    hatch: "#1E2A33",
    focus: "#1F5FA8",
    "font-display": "'Newsreader', 'Iowan Old Style', Georgia, serif",
    "font-body": "'IBM Plex Sans', system-ui, sans-serif",
    "font-mono": "'IBM Plex Mono', ui-monospace, monospace",
    "line-hair": "0.75px",
    line: "1px",
    "line-heavy": "1.6px",
    "card-shadow": "6px 6px 0 #1E2A33",
    radius: "0px",
    "grid-step": 160,
    "motion-quick": "120ms",
    motion: "200ms",
    ease: "cubic-bezier(0.2, 0, 0, 1)",
    "panel-ground": "#F7F3EA",
    "panel-ink": "#1E2A33",
    "panel-muted": "#56626C",
    "frame-line": "#1E2A33",
    "frame-width": "1px",
    "frame-radius": "0px",
    "frame-shadow": "6px 6px 0 #1E2A33",
    "cap-ground": "#F7F3EA",
    "cap-ink": "#1E2A33",
    "cap-rule": "#1E2A33",
    "cap-end": "0px",
    "cap-end-colour": "#B4471F",
    "cap-end-radius": "0px",
    "title-font": "'Newsreader', 'Iowan Old Style', Georgia, serif",
    "title-style": "normal",
    "title-case": "none",
    "title-size": "24px",
    "label-case": "uppercase",
    "label-tracking": "0.1em",
    "value-font": "'IBM Plex Mono', ui-monospace, monospace",
    "value-ink": "#1E2A33",
    "unknown-style": "italic",
    "row-rule": "#D9D1C2",
    "row-rule-style": "dotted",
    "tag-ground": "#F7F3EA",
    "tag-ink": "#8A3612",
    "tag-radius": "0px",
    "text-glow": "none",
    "chart-ground": "#EFEAE0",
    "bench-ground": "#16181C",
    "bench-raised": "#1F2228",
    "bench-raised-2": "#262A31",
    "bench-ink": "#ECEFF4",
    "bench-ink-muted": "#A6AEBB",
    "bench-ink-faint": "#6B7380",
    "bench-line": "#2E333B",
    "bench-accent": "#7FB2FF",
    "bench-accent-ink": "#A9CBFF",
    "bench-on-accent": "#0E1420",
    "bench-good": "#6FD39A",
    "bench-warn": "#F2B35B",
    "bench-bad": "#FF7A7A",
    "bench-violet": "#C3A6FF",
  },
};

export const NIGHT_CHART: ThemeDoc = {
  schema: "sse.theme/1",
  id: "night-chart",
  name: "Night chart",
  extends: "chart-room",
  mode: "dark",
  supports: { colours: "full", patterns: true, shapes: true },
  tokens: {
    ground: "#0B1117",
    "ground-raised": "#111A22",
    ink: "#DCE3EA",
    "ink-muted": "#9AA8B6",
    "ink-faint": "#6C7A88",
    grid: "#18232E",
    rule: "#22303D",
    accent: "#5EB8FF",
    "accent-ink": "#8CCBFF",
    "on-ink": "#0B1117",
    "field-out": "#F0A050",
    "field-in": "#6CB4F0",
    "origin-big-bang": "#BFD4E6",
    "origin-cosmic-rays": "#8877DD",
    "origin-low-mass-stars": "#E3B23C",
    "origin-massive-stars": "#4F9FC4",
    "origin-white-dwarfs": "#7FA06A",
    "origin-neutron-star-mergers": "#C45A9A",
    "origin-human-made": "#5A6672",
    "map-1": "#E39B41",
    "map-2": "#57AEFF",
    "map-3": "#6CC48A",
    "map-4": "#EECF46",
    "map-5": "#DB6FE4",
    "map-6": "#4ED7D0",
    "map-7": "#F47FAB",
    "map-8": "#CCD186",
    zone: "rgba(62, 140, 106, 0.22)",
    "zone-ink": "#7CC4A0",
    hatch: "#9AA8B6",
    focus: "#F2A93B",
    "card-shadow": "6px 6px 0 #000000",
    "panel-ground": "#111A22",
    "panel-ink": "#DCE3EA",
    "panel-muted": "#9AA8B6",
    "frame-line": "#DCE3EA",
    "frame-shadow": "6px 6px 0 #000000",
    "cap-ground": "#111A22",
    "cap-ink": "#DCE3EA",
    "cap-rule": "#DCE3EA",
    "cap-end-colour": "#5EB8FF",
    "value-ink": "#DCE3EA",
    "row-rule": "#22303D",
    "tag-ground": "#111A22",
    "tag-ink": "#8CCBFF",
    "chart-ground": "#0B1117",
  },
};

export const THEMES: readonly ThemeDoc[] = [CHART_ROOM, NIGHT_CHART];

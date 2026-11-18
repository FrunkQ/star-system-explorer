// The two GM modes (H8.5). Night chart inherits Chart room and changes only
// the colours, so the structure is the same in both.
export interface ThemeDoc {
  schema: "sse.theme/1";
  id: string;
  name: string;
  extends?: string;
  mode: "light" | "dark";
  /** What the theme can carry (H8.4A): symbologies degrade by a stated rule where it cannot. */
  supports: { colours: number | "full"; patterns: boolean; shapes: boolean };
  tokens: Record<string, string | number>;
}

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
    zone: "rgba(62, 140, 106, 0.22)",
    "zone-ink": "#7CC4A0",
    hatch: "#9AA8B6",
    focus: "#F2A93B",
    "card-shadow": "6px 6px 0 #000000",
  },
};

export const THEMES: readonly ThemeDoc[] = [CHART_ROOM, NIGHT_CHART];

// Evidence and honesty stamps (H8.5). Honesty marks (authored, impossible,
// fictional) always show on the GM view, whatever the lens.
export const STAMP_LABELS: Record<string, string> = {
  resolved: "Resolved",
  authored: "Authored",
  impossible: "Impossible",
  fictional: "Fictional",
  "one-plausible-future": "One plausible future",
  unsurveyed: "Unsurveyed",
  fixture: "Fixture data",
};

export const HONESTY = new Set(["authored", "impossible", "fictional"]);

export function stampLabel(kind: string): string {
  return STAMP_LABELS[kind] ?? kind.replace(/-/g, " ");
}

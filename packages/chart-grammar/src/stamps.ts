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
  catalogued: "Catalogued",
  unverified: "Unverified",
  "out-of-range": "Out of range",
  "changed-physics": "Changed physics",
  validated: "Validated",
  rejected: "Rejected",
};

// "changed-physics" is an honesty mark too: the value is not the base physics as shipped.
export const HONESTY = new Set(["authored", "impossible", "fictional", "changed-physics"]);

export function stampLabel(kind: string): string {
  return STAMP_LABELS[kind] ?? kind.replace(/-/g, " ");
}

// Words for a construct's facts (V3 gap 13): a field's last part made plain,
// with the unit its suffix names. Presentation only; a pack's vocabulary
// terms win where it has them.

const UNITS: Record<string, string> = { dt: "dT", g: "g", m: "m", kg: "kg", t: "t", mw: "MW", km: "km", s: "s", pc: "pc" };

/** "traveller.cargo_dt" is "cargo" in dT; "construct.crew_max" is "crew max". */
export function fieldWords(field: string, terms: Record<string, string> = {}): { label: string; unit?: string } {
  if (terms[field]) return { label: terms[field]! };
  const last = field.split(".").pop() ?? field;
  const parts = last.split("_");
  const u = parts.length > 1 ? UNITS[parts[parts.length - 1]!] : undefined;
  const words = (u ? parts.slice(0, -1) : parts).join(" ");
  const range = field.startsWith("sensor.") ? `${field.split(".")[1]!.replace(/_/g, " ")} ${words}` : words;
  return u ? { label: range, unit: u } : { label: range };
}

/** A value in words: numbers with honest precision, big ones as powers of ten. */
export function valueWords(v: unknown): string {
  if (typeof v === "number") {
    if (!Number.isFinite(v)) return String(v);
    const a = Math.abs(v);
    if (a !== 0 && (a >= 1e7 || a < 1e-3)) {
      const e = Math.floor(Math.log10(a));
      return `${Number((v / Math.pow(10, e)).toPrecision(3))} × 10^${e}`;
    }
    return Number.isInteger(v) ? v.toLocaleString("en-GB") : String(Number(v.toPrecision(4)));
  }
  if (typeof v === "boolean") return v ? "yes" : "no";
  return v === undefined || v === null ? "unknown" : String(v);
}

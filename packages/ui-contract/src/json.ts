/** Plain JSON data (no NaN, no undefined). */
export type Json = null | boolean | number | string | Json[] | { [k: string]: Json };

/**
 * Encode a kernel request. The kernel's integer times are i64; JavaScript
 * carries them as bigint, which JSON.stringify refuses, so they are written
 * here as exact decimal digits.
 */
export function encodeRequest(request: unknown): string {
  const marks: string[] = [];
  const text = JSON.stringify(request, (_k, v) => {
    if (typeof v === "bigint") {
      marks.push(v.toString());
      return `\u0000big${marks.length - 1}\u0000`;
    }
    if (typeof v === "number" && !Number.isFinite(v)) throw new Error("non-finite number in a kernel request");
    return v;
  });
  return text.replace(/"\\u0000big(\d+)\\u0000"/g, (_m, i: string) => marks[Number(i)]!);
}

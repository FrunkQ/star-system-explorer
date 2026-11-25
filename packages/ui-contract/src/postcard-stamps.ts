// Stamps (PC3, postcards design section 2): the small kept picture of a postcard, made from a drawn still. A campaign
// postcard is a bookmark, which carries no picture, so its stamp is kept on this device (a per-viewer convenience):
// the wall shows something at once, and the card always draws live. An envelope that travels carries its own.
const KEY = "sse-postcard-stamps";
const MOST = 48;

function read(): [string, string][] {
  try {
    const v = JSON.parse(localStorage.getItem(KEY) ?? "[]");
    return Array.isArray(v) ? v.filter((x): x is [string, string] => Array.isArray(x) && typeof x[0] === "string" && typeof x[1] === "string") : [];
  } catch {
    return [];
  }
}

export function stampFor(scope: string, id: string): string | null {
  return read().find(([k]) => k === `${scope}|${id}`)?.[1] ?? null;
}

export function keepStamp(scope: string, id: string, uri: string): void {
  const k = `${scope}|${id}`;
  const first: [string, string] = [k, uri];
  const next: [string, string][] = [first, ...read().filter(([x]) => x !== k)].slice(0, MOST);
  try {
    localStorage.setItem(KEY, JSON.stringify(next));
  } catch {
    // Full or blocked: the wall draws without it.
  }
}

/** A small kept picture from a drawn still: WebP (PNG where the browser has no WebP), at most STAMP_SIDE across. */
export function stampOf(canvas: HTMLCanvasElement, side: number): string | null {
  const k = Math.min(1, side / Math.max(canvas.width, canvas.height, 1));
  const c = document.createElement("canvas");
  c.width = Math.max(1, Math.round(canvas.width * k));
  c.height = Math.max(1, Math.round(canvas.height * k));
  const g = c.getContext("2d");
  if (!g) return null;
  g.drawImage(canvas, 0, 0, c.width, c.height);
  for (const [type, q] of [["image/webp", 0.8], ["image/jpeg", 0.8], ["image/png", undefined]] as const) {
    const uri = c.toDataURL(type, q);
    if (uri.startsWith(`data:${type}`)) return uri;
  }
  return null;
}

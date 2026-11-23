// Show a player view inside a virtual tabletop (V3 gap 26): `?embed=1` on a
// seat's join link drops the page's own chrome, and a small message bridge
// talks to the tabletop that frames it. Both only for a tabletop whose web
// address the GM listed on that seat, matched exactly: scheme, host and port,
// no wildcards, no paths. Anything else is refused, and says why.
//
// The bridge carries only what the seat already sees: the subject in focus
// (its address and the name this seat's scene gives it) and that the view is
// ready. Inbound, it takes one ask, "focus this subject", from the allowed
// tabletop only, and the view still shows only what its scene holds.

/** The page was opened to sit inside a tabletop (`?embed=1`). */
export function embedAsked(search: string): boolean {
  const v = new URLSearchParams(search).get("embed");
  return v === "1" || v === "true";
}

/**
 * A tabletop's web address as an exact origin ("https://app.example.com" or with a port), or why it cannot be one.
 * Only https, or http on this machine (localhost, 127.0.0.1) for testing. No path, query, user or wildcard.
 */
export function embedOrigin(raw: string): { ok: true; origin: string } | { ok: false; reason: string } {
  const s = raw.trim();
  if (!s) return { ok: false, reason: "empty" };
  if (s.includes("*")) return { ok: false, reason: `${s}: no wildcards; list each tabletop's exact address` };
  let u: URL;
  try {
    u = new URL(s);
  } catch {
    return { ok: false, reason: `${s}: not a web address (it should start https://)` };
  }
  const local = u.hostname === "localhost" || u.hostname === "127.0.0.1";
  if (u.protocol !== "https:" && !(u.protocol === "http:" && local)) return { ok: false, reason: `${s}: only https addresses (http only on this machine, for testing)` };
  if (u.username || u.password) return { ok: false, reason: `${s}: no user name or password in the address` };
  if ((u.pathname !== "/" && u.pathname !== "") || u.search || u.hash) return { ok: false, reason: `${s}: just the site, with no path (${u.origin})` };
  return { ok: true, origin: u.origin };
}

/** The listed origins, each checked; problems one line each. */
export function embedOrigins(lines: readonly string[]): { origins: string[]; problems: string[] } {
  const origins: string[] = [];
  const problems: string[] = [];
  for (const l of lines) {
    if (!l.trim()) continue;
    const r = embedOrigin(l);
    if (r.ok) {
      if (!origins.includes(r.origin)) origins.push(r.origin);
    } else problems.push(r.reason);
  }
  return { origins, problems };
}

/** The origin of the page framing this one: the browser's own ancestor list where it has one, else the referrer. */
export function framingOrigin(w: { self: unknown; top: unknown; location: { ancestorOrigins?: { length: number; [i: number]: string } } }, referrer: string): string | null | "unframed" {
  if (w.self === w.top) return "unframed";
  const a = w.location.ancestorOrigins;
  if (a && a.length > 0) return a[0] ?? null;
  try {
    return referrer ? new URL(referrer).origin : null;
  } catch {
    return null;
  }
}

export type EmbedMode = { mode: "page" } | { mode: "embedded"; origin: string } | { mode: "refused"; reason: string };

/** What this page does: a page of its own, a view inside an allowed tabletop, or a refusal saying why. */
export function embedMode(asked: boolean, framing: string | null | "unframed", allowed: readonly string[] | undefined): EmbedMode {
  if (!asked) return { mode: "page" };
  if (framing === "unframed") return { mode: "page" };
  if (!allowed?.length) return { mode: "refused", reason: "The game master has not allowed this seat inside a tabletop. They can list the tabletop's web address on the seat." };
  if (framing === null) return { mode: "refused", reason: "This browser does not say which site is showing this view, so it cannot be checked against the game master's list." };
  if (!allowed.includes(framing)) return { mode: "refused", reason: `${framing} is not one of the tabletops the game master allowed for this seat.` };
  return { mode: "embedded", origin: framing };
}

/** Messages to the tabletop. Every one carries `source: "sse4"` and the bridge version. */
export type EmbedOut =
  | { source: "sse4"; v: 1; type: "ready"; seat: string; audience: string }
  | { source: "sse4"; v: 1; type: "focus"; address: string | null; name: string | null };

/** The one ask a tabletop may send: focus a subject the view already shows. Anything else is ignored. */
export function embedAsk(data: unknown): { type: "focus"; address: string } | null {
  if (!data || typeof data !== "object") return null;
  const d = data as { source?: unknown; type?: unknown; address?: unknown };
  if (d.source !== "vtt" || d.type !== "focus" || typeof d.address !== "string" || d.address.length > 200) return null;
  return { type: "focus", address: d.address };
}

/** The bridge: posts only to the allowed origin, hears only from it and only from the framing window. */
export class EmbedBridge {
  readonly #origin: string;
  readonly #parent: { postMessage(m: unknown, origin: string): void };
  readonly #listen: (f: (e: { origin: string; source: unknown; data: unknown }) => void) => () => void;
  #stop: (() => void) | null = null;
  constructor(origin: string, parent: { postMessage(m: unknown, origin: string): void }, listen: (f: (e: { origin: string; source: unknown; data: unknown }) => void) => () => void) {
    this.#origin = origin;
    this.#parent = parent;
    this.#listen = listen;
  }
  send(m: EmbedOut): void {
    this.#parent.postMessage(m, this.#origin);
  }
  onAsk(f: (a: { type: "focus"; address: string }) => void): void {
    this.#stop?.();
    this.#stop = this.#listen((e) => {
      if (e.origin !== this.#origin || e.source !== this.#parent) return;
      const a = embedAsk(e.data);
      if (a) f(a);
    });
  }
  close(): void {
    this.#stop?.();
    this.#stop = null;
  }
}

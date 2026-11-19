// Pieces lifted from Ovid Next (FrunkQ's own code, relicensed by its author
// as Apache-2.0 for SSE4; see NOTICE). Each carries the Ovid file and function
// it came from, so the two can be compared. They are trimmed to what SSE4
// needs: Ovid's provider list is longer (drawing, voices, its own PC server)
// and SSE4 has no household server, so the parts about those are left out.
//
// Read with docs/ovid-agents.md. Nothing here calls a model.

// ---------------------------------------------------------------- addresses

/** ovid: browser/src/models/links.ts (ENDPOINT_TAIL, AT_HOME_HOST): an endpoint pasted in place of the address, and an address at home. */
const ENDPOINT_TAIL = /\/(chat\/completions|completions|models|embeddings)$/i;
const AT_HOME_HOST = /^(127\.\d+\.\d+\.\d+|localhost|10\.\d+\.\d+\.\d+|192\.168\.\d+\.\d+|172\.(1[6-9]|2\d|3[01])\.\d+\.\d+|[a-z0-9-]+\.local|\[::1\])$/i;

/**
 * ovid: browser/src/models/links.ts::normalBase. An address as a person typed
 * or pasted it, in the one form SSE calls: http:// added for an address at
 * home and https:// for any other when no scheme is given, a slip such as
 * "http:/" or backslashes mended, an endpoint pasted in place of the address
 * taken off, the trailing slash dropped, and /v1 added when no path is given
 * (a path it has, such as /api/v1, is kept; Ollama's /api becomes /v1). What
 * cannot be read as an address is left as typed.
 */
export function normalBase(typed: string, ollama = false): string {
  let t = typed.trim().replace(/\\/g, "/");
  if (!t) return "";
  t = t.replace(/^(https?):\/*(?=[^/])/i, "$1://");
  if (!/^[a-z][a-z0-9+.-]*:\/\//i.test(t)) t = (AT_HOME_HOST.test(t.replace(/[:/].*$/, "")) || /^[\d.]+(:\d+)?(\/|$)/.test(t) ? "http://" : "https://") + t;
  let u: URL;
  try {
    u = new URL(t);
  } catch {
    return typed.trim();
  }
  if (u.protocol !== "http:" && u.protocol !== "https:") return typed.trim();
  let path = u.pathname.replace(/\/+$/, "");
  for (let was = ""; was !== path; ) {
    was = path;
    path = path.replace(ENDPOINT_TAIL, "").replace(/\/+$/, "");
  }
  if (!path || (ollama && /^\/api$/i.test(path))) path = "/v1";
  return u.origin + path;
}

// ---------------------------------------------------------------- prices

/**
 * ovid: browser/src/models/links.ts::listedPrice. A negative listed price is
 * "the price varies" (OpenRouter lists a router's price as -1), never a
 * price. Prices are a catalogue estimate, never the charge.
 */
export function listedPrice(p: { prompt: number; completion: number } | undefined): { price?: { prompt: number; completion: number }; varies?: true } {
  if (!p || !Number.isFinite(p.prompt) || !Number.isFinite(p.completion)) return {};
  if (p.prompt < 0 || p.completion < 0) return { varies: true };
  return { price: p };
}

// ---------------------------------------------------------------- loaded context

/**
 * ovid: browser/src/models/loaded-context.ts. A model on this computer plans
 * with the context it is loaded with, not the most it could take. Where the
 * server does not say, 4096: the length LM Studio and Ollama load with unless
 * told otherwise, never their listed maximum.
 */
export const UNSAID = 4096;

const pos = (x: unknown): number | undefined => (typeof x === "number" && Number.isFinite(x) && x > 0 ? x : undefined);
// A link's model and LM Studio's key name the same model with or without the publisher.
const same = (a: unknown, b: string) => typeof a === "string" && (a === b || a.split("/").pop() === b.split("/").pop());

/** ovid: browser/src/models/loaded-context.ts::fromLmStudioV1. The loaded length in LM Studio's GET /api/v1/models answer. */
export function fromLmStudioV1(json: unknown, model: string): { tokens: number; most?: number } | null {
  const j = json as { models?: unknown[]; data?: unknown[] } | null;
  const list = (Array.isArray(j?.models) ? j!.models : Array.isArray(j?.data) ? j!.data : []) as Record<string, unknown>[];
  const loaded = (m: Record<string, unknown>) => (Array.isArray(m?.loaded_instances) ? (m.loaded_instances as Record<string, unknown>[]) : []);
  const m = model ? list.find((x) => same(x?.key, model) || same(x?.id, model) || loaded(x).some((i) => same(i?.id, model))) : list.find((x) => loaded(x).length);
  if (!m) return null;
  const inst = loaded(m).find((i) => !model || same(i?.id, model)) ?? loaded(m)[0];
  const tokens = pos((inst?.config as Record<string, unknown> | undefined)?.context_length);
  const most = pos(m.max_context_length);
  return tokens ? { tokens, ...(most ? { most } : {}) } : null;
}

/** ovid: browser/src/models/loaded-context.ts::fromOllamaPs. A running model's context in Ollama's /api/ps answer. */
export function fromOllamaPs(json: unknown, model: string): number | undefined {
  const list = (Array.isArray((json as { models?: unknown[] } | null)?.models) ? (json as { models: Record<string, unknown>[] }).models : []) as Record<string, unknown>[];
  const m = model ? list.find((x) => x?.name === model || x?.model === model) : list[0];
  return pos(m?.context_length);
}

// ---------------------------------------------------------------- keys out of anything sent

/**
 * ovid: browser/src/play/support-report.ts (KEY_RES) and play/support-ask.ts::clean.
 * Key shapes taken out of anything sent to a model (a person may paste one
 * into a question). Ovid's last shape, any run of 32 or more base-64
 * characters, is left out here: SSE's text is full of state hashes and
 * addresses that look like that. The keys actually stored in this browser
 * are taken out by value instead, which catches any shape.
 */
export const KEY_SHAPES: readonly RegExp[] = [/\bsk-or-[A-Za-z0-9_-]{6,}/g, /\bsk-ant-[A-Za-z0-9_-]{6,}/g, /\bsk-[A-Za-z0-9_-]{8,}/g, /\bBearer\s+\S+/g, /\bghp_[A-Za-z0-9]{6,}/g, /\bgithub_pat_[A-Za-z0-9_]{6,}/g];

export function clean(text: string, known: readonly string[] = []): string {
  let t = text;
  for (const k of known) if (k && k.length >= 6) t = t.split(k).join("[a key]");
  for (const re of KEY_SHAPES) t = t.replace(new RegExp(re.source, re.flags), "[a key]");
  return t;
}

// ---------------------------------------------------------------- replies

/**
 * ovid: browser/src/agent/builder.ts::readReply. The JSON object in a model's
 * reply: inside a ```json fence if there is one, else from the first { to the
 * last }. Null when there is none, so the loop can say "your answer was not
 * one JSON object" and try again.
 */
export function readJson(text: string): Record<string, unknown> | null {
  const fence = /```(?:json)?\s*\n([\s\S]*?)\n```/.exec(text);
  const body = fence ? fence[1]! : text.slice(text.indexOf("{"), text.lastIndexOf("}") + 1);
  try {
    const v = JSON.parse(body) as unknown;
    return v && typeof v === "object" && !Array.isArray(v) ? (v as Record<string, unknown>) : null;
  } catch {
    return null;
  }
}

/**
 * ovid: browser/src/agent/builder.ts::readLookup. A lookup reply: the catalogue
 * entries a model asks to read in full before it drafts ({"need": [...]}), or
 * a search by plain words ({"find": "..."}). Null when the reply is not a
 * lookup. A lookup does not use up a try.
 */
export function readLookup(text: string): { need: string[]; find: string | null } | null {
  const v = readJson(text);
  if (!v || "make" in v || "request" in v || "requests" in v) return null;
  const need = Array.isArray(v.need) ? v.need.map(String).slice(0, 40) : [];
  const find = typeof v.find === "string" && v.find.trim() ? v.find.trim().slice(0, 200) : null;
  return need.length || find ? { need, find } : null;
}

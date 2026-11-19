// Agents, bring your own key (A1, H13, docs/ovid-agents.md), in the shape of
// Ovid Next's set-up: the same three provider kinds plus "another provider by
// its /v1 address"; jobs that each link one model; a content-free usage
// ledger with an estimate before every run; never send more than the model is
// loaded with. Part 2 reconciled this with Ovid's source (models/links.ts,
// ledger.ts, loaded-context.ts, play/support-ask.ts): the pieces lifted from
// it are in lifted.ts, each naming the Ovid function it came from.
//
// Rules this module keeps:
// - Keys live in this browser only, each under its own entry
//   ("sse4.agents.key.<provider>"), apart from the settings ("sse4.agents"),
//   as Ovid saves its link list without keys. So the settings can be shown,
//   exported or reset without carrying a key. Keys are sent only to the
//   provider's own address, in its Authorization header, and taken out of
//   anything sent to a model (lifted.ts clean). Nothing
//   here is imported by the peer-to-peer code, the share, the players' page or
//   any session, so a key cannot reach another device or a state hash
//   (tests/ui/boundary.test.ts checks the imports).
// - LLMs never compute physics and never sit in a deterministic path.
// - The app is fully usable with no key: nothing else depends on this module.

import { UNSAID, clean, fromLmStudioV1, fromOllamaPs, listedPrice, normalBase } from "./lifted.ts";

export type ProviderKind = "openrouter" | "lmstudio" | "ollama" | "other";

export const PROVIDER_KINDS: Record<ProviderKind, { label: string; base: string; key: "required" | "optional" | "none"; note: string }> = {
  openrouter: { label: "OpenRouter", base: "https://openrouter.ai/api/v1", key: "required", note: "One key, many models. Prices come with the model list." },
  lmstudio: { label: "LM Studio on this computer", base: "http://localhost:1234/v1", key: "none", note: "Start LM Studio's local server. Nothing leaves this computer." },
  ollama: { label: "Ollama on this computer", base: "http://localhost:11434/v1", key: "none", note: "Run Ollama with OLLAMA_ORIGINS set to allow this page. Nothing leaves this computer." },
  other: { label: "Another provider", base: "", key: "optional", note: "Any OpenAI-compatible /v1 address." },
};

export interface Provider {
  id: string;
  kind: ProviderKind;
  label: string;
  /** The /v1 address. */
  base: string;
  /** Stays in this browser. */
  key?: string;
}

/** The jobs agents do. Each job links one model (Ovid's "jobs", SSE4's agents). */
export type JobId = "support" | "building" | "describing" | "deciding";
export const JOBS: Record<JobId, { agent: string; label: string; what: string; package: string }> = {
  support: { agent: "Tars", label: "Support", what: "Answers questions about SSE from the docs and this universe's facts. Read only. Allowed to be funny.", package: "A4" },
  building: { agent: "Orac", label: "Building", what: "Proposes change requests (flows, packs, screens, data views, sifters), each validated before you see it. Never changes anything itself.", package: "A2" },
  describing: { agent: "Minerva", label: "Describing", what: "Writes narrative from what the players may know, labelled narrative, never a number not in the facts.", package: "A3" },
  deciding: { agent: "", label: "Deciding", what: "Small cheap checks for the others, so the expensive models do only the work that needs them.", package: "A2" },
};

export interface ModelInfo {
  id: string;
  /** Tokens the model holds, when the provider says. */
  context?: number;
  /** US dollars per token, when the provider says. */
  price?: { prompt: number; completion: number };
  /** The provider lists the price as varying (a router): no estimate is made. */
  varies?: true;
}

export interface LedgerRow {
  day: string;
  job: JobId;
  model: string;
  tokens_in: number;
  tokens_out: number;
  /** As the provider charged it, else from its listed price; missing when neither is known (never shown as zero). */
  usd?: number;
  /** False for a call that failed: it is kept too, as Ovid's ledger keeps them. */
  ok?: boolean;
  /** Milliseconds the call took. */
  ms?: number;
}

export interface AgentsState {
  schema: "sse.agents/0";
  providers: Provider[];
  links: Partial<Record<JobId, { provider: string; model: string; context?: number; price?: ModelInfo["price"]; varies?: true; loaded?: number }>>;
  /** Daily spending cap in US dollars; null for none. */
  cap_usd: number | null;
  /** 0 (plain) to 1 (dry humour, the default), for Tars. */
  humour: number;
  ledger: LedgerRow[];
}

export const STORE_KEY = "sse4.agents";
/** Where one provider's key is kept: its own entry, never inside the settings. */
export const keyEntry = (provider: string) => `${STORE_KEY}.key.${provider}`;

/** The part of localStorage the agents use (a Map in tests). */
export interface KeyStore {
  getItem(k: string): string | null;
  setItem(k: string, v: string): void;
  removeItem(k: string): void;
}

export function emptyAgents(): AgentsState {
  return { schema: "sse.agents/0", providers: [], links: {}, cap_usd: 1, humour: 0.6, ledger: [] };
}

export function readAgents(raw: string | null): AgentsState {
  if (!raw) return emptyAgents();
  try {
    const o = JSON.parse(raw) as Partial<AgentsState>;
    if (o.schema !== "sse.agents/0") return emptyAgents();
    return { ...emptyAgents(), ...o, providers: Array.isArray(o.providers) ? o.providers : [], ledger: Array.isArray(o.ledger) ? o.ledger.slice(-500) : [] };
  } catch {
    return emptyAgents();
  }
}

/** A store that lasts for this page only (a private window, tests). */
export function memoryStore(): KeyStore {
  const m = new Map<string, string>();
  return { getItem: (k) => m.get(k) ?? null, setItem: (k, v) => void m.set(k, v), removeItem: (k) => void m.delete(k) };
}

/** The settings as stored: every provider without its key (Ovid's saveLinks). */
export function settingsText(s: AgentsState): string {
  return JSON.stringify({ ...s, providers: s.providers.map(({ key: _k, ...p }) => p), ledger: s.ledger.slice(-500) });
}

/**
 * Settings and keys from the store. A key saved inside the settings by part 1
 * is moved to its own entry the first time it is read.
 */
export function loadAgents(store: KeyStore): AgentsState {
  let raw: string | null = null;
  try {
    raw = store.getItem(STORE_KEY);
  } catch {
    return emptyAgents();
  }
  const s = readAgents(raw);
  let moved = false;
  for (const p of s.providers) {
    if (p.key) {
      try {
        store.setItem(keyEntry(p.id), p.key);
        moved = true;
      } catch {
        /* keep it for this page */
      }
    } else {
      try {
        const k = store.getItem(keyEntry(p.id));
        if (k) p.key = k;
      } catch {
        /* none */
      }
    }
  }
  if (moved) saveAgents(store, s);
  return s;
}

/** Save the settings and each key in its own entry; a removed provider's key is deleted with it. */
export function saveAgents(store: KeyStore, s: AgentsState, removed: readonly string[] = []): void {
  try {
    store.setItem(STORE_KEY, settingsText(s));
    for (const p of s.providers) {
      if (p.key) store.setItem(keyEntry(p.id), p.key);
      else store.removeItem(keyEntry(p.id));
    }
    for (const id of removed) if (!s.providers.some((p) => p.id === id)) store.removeItem(keyEntry(id));
  } catch {
    /* private window: settings last for this page only */
  }
}

/** A new provider from what was typed: the address mended as Ovid mends it (normalBase). */
export function newProvider(s: AgentsState, kind: ProviderKind, label: string, typedBase: string, key: string): Provider | { problem: string } {
  const base = normalBase(typedBase, kind === "ollama");
  if (!/^https?:\/\//.test(base)) return { problem: "That is not an address SSE can call. It usually looks like https://example.com/v1." };
  if (PROVIDER_KINDS[kind].key === "required" && !key.trim()) return { problem: `${PROVIDER_KINDS[kind].label} needs a key.` };
  let id: string = kind;
  for (let n = 2; s.providers.some((p) => p.id === id); n++) id = `${kind}-${n}`;
  return { id, kind, label: label.trim() || PROVIDER_KINDS[kind].label, base, ...(key.trim() ? { key: key.trim() } : {}) };
}

const isLocal = (p: Provider) => p.kind === "lmstudio" || p.kind === "ollama";

/**
 * The context a prompt to this link must fit. For LM Studio and Ollama, what
 * the model is loaded with (read by readLoaded), else 4096; never the most it
 * could take (Ovid's loaded-context rule). For others, the listed context.
 */
export function contextOf(s: AgentsState, job: JobId): number | undefined {
  const l = s.links[job];
  const p = l && s.providers.find((x) => x.id === l.provider);
  if (!l || !p) return undefined;
  if (isLocal(p)) return l.loaded ?? UNSAID;
  return l.context;
}

/**
 * Ask a local server what the model is loaded with (LM Studio's /api/v1/models,
 * Ollama's /api/ps). Null for a provider that is not local or does not say.
 */
export async function readLoaded(p: Provider, model: string, fetcher: typeof fetch = fetch): Promise<number | null> {
  if (!isLocal(p)) return null;
  const root = p.base.replace(/\/+$/, "").replace(/\/v1$/, "");
  try {
    const r = await fetcher(p.kind === "lmstudio" ? `${root}/api/v1/models` : `${root}/api/ps`);
    if (!r.ok) return null;
    const j = (await r.json()) as unknown;
    return p.kind === "lmstudio" ? (fromLmStudioV1(j, model)?.tokens ?? null) : (fromOllamaPs(j, model) ?? null);
  } catch {
    return null;
  }
}

/** Whether any agent can run. With none linked, every agent panel says so and the app carries on. */
export function linked(s: AgentsState, job: JobId): boolean {
  const l = s.links[job];
  return !!l && s.providers.some((p) => p.id === l.provider);
}

// ------------------------------------------------------------ the adapter

/** Headers for one provider's own address. A key goes nowhere else. */
function headers(p: Provider): Record<string, string> {
  const h: Record<string, string> = { "content-type": "application/json" };
  if (p.key) h.authorization = `Bearer ${p.key}`;
  if (p.kind === "openrouter") h["x-title"] = "Star System Explorer";
  return h;
}

const url = (p: Provider, path: string) => `${p.base.replace(/\/+$/, "")}${path}`;

export async function listModels(p: Provider, fetcher: typeof fetch = fetch): Promise<{ ok: true; models: ModelInfo[] } | { ok: false; message: string }> {
  try {
    const r = await fetcher(url(p, "/models"), { headers: headers(p) });
    if (!r.ok) return { ok: false, message: r.status === 401 || r.status === 403 ? "The provider refused the key." : `The provider answered ${r.status}.` };
    const body = (await r.json()) as { data?: { id: string; context_length?: number; pricing?: { prompt?: string; completion?: string } }[] };
    const models = (body.data ?? []).map((m) => {
      const listed = m.pricing && m.pricing.prompt !== undefined && m.pricing.completion !== undefined ? { prompt: Number(m.pricing.prompt), completion: Number(m.pricing.completion) } : undefined;
      return { id: m.id, ...(m.context_length ? { context: m.context_length } : {}), ...listedPrice(listed) };
    });
    return { ok: true, models };
  } catch {
    return { ok: false, message: p.kind === "lmstudio" || p.kind === "ollama" ? "Nothing answered on this computer at that address. Is the local server running and allowing this page?" : "Could not reach the provider." };
  }
}

export interface ChatMessage {
  role: "system" | "user" | "assistant";
  content: string;
}

/** A rough token count: about four characters per token. Shown as an estimate, never as a fact. */
export function estimateTokens(messages: readonly ChatMessage[]): number {
  return Math.ceil(messages.reduce((n, m) => n + m.content.length + 4, 0) / 4);
}

/** The estimate before a run (Ovid's ledger rule: show it first). */
export function estimate(s: AgentsState, job: JobId, messages: readonly ChatMessage[], max_out: number): { tokens_in: number; max_out: number; usd?: number; fits: boolean; context?: number; today_usd: number; over_cap: boolean } {
  const l = s.links[job];
  const tokens_in = estimateTokens(messages);
  const usd = l?.price ? tokens_in * l.price.prompt + max_out * l.price.completion : undefined;
  const today = new Date().toISOString().slice(0, 10);
  const today_usd = s.ledger.filter((r) => r.day === today).reduce((a, r) => a + (r.usd ?? 0), 0);
  const context = contextOf(s, job);
  return { tokens_in, max_out, ...(usd !== undefined ? { usd } : {}), fits: context === undefined || tokens_in + max_out <= context, ...(context ? { context } : {}), today_usd, over_cap: s.cap_usd !== null && usd !== undefined && today_usd + usd > s.cap_usd };
}

/** Ask the job's model. Refuses before sending when the prompt would not fit or the day's cap would be passed. */
export async function chat(
  s: AgentsState,
  job: JobId,
  messages: readonly ChatMessage[],
  opts: { max_out?: number; fetcher?: typeof fetch; temperature?: number } = {},
): Promise<{ ok: true; text: string; row: LedgerRow } | { ok: false; message: string; row?: LedgerRow }> {
  const l = s.links[job];
  const p = l && s.providers.find((x) => x.id === l.provider);
  if (!l || !p) return { ok: false, message: `No model is linked for ${JOBS[job].label}. Everything else works without one.` };
  const max_out = opts.max_out ?? 600;
  // Keys out of anything sent, by value and by shape (a person may paste one into a question).
  const keys = s.providers.map((x) => x.key ?? "").filter(Boolean);
  const sent = messages.map((m) => ({ role: m.role, content: clean(m.content, keys) }));
  const e = estimate(s, job, sent, max_out);
  if (!e.fits) return { ok: false, message: `This would need about ${e.tokens_in + max_out} tokens and ${l.model} ${isLocal(p) ? "is loaded with" : "holds"} ${e.context}. Nothing was sent.${isLocal(p) ? " Load the model with a longer context, then check it again." : ""}` };
  if (e.over_cap) return { ok: false, message: `This would pass today's cap of $${s.cap_usd!.toFixed(2)}. Nothing was sent.` };
  const day = new Date().toISOString().slice(0, 10);
  const t0 = Date.now();
  const failed = (message: string) => ({ ok: false as const, message, row: { day, job, model: l.model, tokens_in: 0, tokens_out: 0, ok: false, ms: Date.now() - t0 } });
  try {
    const r = await (opts.fetcher ?? fetch)(url(p, "/chat/completions"), { method: "POST", headers: headers(p), body: JSON.stringify({ model: l.model, messages: sent, max_tokens: max_out, ...(opts.temperature !== undefined ? { temperature: opts.temperature } : {}) }) });
    if (!r.ok) return failed(r.status === 401 || r.status === 403 ? "The provider refused the key." : `The provider answered ${r.status}.`);
    const body = (await r.json()) as { choices?: { message?: { content?: string } }[]; usage?: { prompt_tokens?: number; completion_tokens?: number; cost?: number } };
    const text = body.choices?.[0]?.message?.content ?? "";
    const tokens_in = body.usage?.prompt_tokens ?? e.tokens_in;
    const tokens_out = body.usage?.completion_tokens ?? Math.ceil(text.length / 4);
    const usd = body.usage?.cost ?? (l.price ? tokens_in * l.price.prompt + tokens_out * l.price.completion : undefined);
    const row: LedgerRow = { day, job, model: l.model, tokens_in, tokens_out, ...(usd !== undefined ? { usd } : {}), ok: true, ms: Date.now() - t0 };
    return { ok: true, text, row };
  } catch {
    return failed(isLocal(p) ? "Nothing answered on this computer at that address. Is the local server running and allowing this page?" : "Could not reach the provider.");
  }
}

/** Tars's standing instructions. The humour setting turns the jokes down. */
export function tarsSystem(humour: number, context: string): string {
  const tone = humour >= 0.75 ? "Be dryly funny, briefly." : humour >= 0.35 ? "A light, dry remark is welcome now and then." : "Be plain and direct; no jokes.";
  return [
    "You are Tars, the support agent of Star System Explorer (SSE), an astrophysics and role-playing toolkit.",
    "You are read only: you explain, you never change anything. You never compute physics: every number you give must come from the facts below or the user's message, and you say where it came from.",
    "If you are not sure, say so and point to the place in the app that answers it (a value's \"why?\", the Physics screen, the Player data screen).",
    tone,
    "Facts in view:",
    context,
  ].join("\n");
}

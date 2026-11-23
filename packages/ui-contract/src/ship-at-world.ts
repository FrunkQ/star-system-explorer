// A ship at a world (aerobraking, landing and take-off on the physics): the
// kernel's `ship_at_world` (sse-nav `entry`) works each act out from the
// ship's own numbers and the world's air and gravity, and answers with figures,
// dangers, rough chances and what would make it safe. Nothing is refused and
// nothing is rolled: the GM judges (Alex, 23 Nov 2026). This file only holds
// the shapes and the words a card puts them in.
import type { NavShip, NavSystem } from "./route.ts";

export interface ShipAtWorldAsk {
  /** An inline system, a NAV fixture (`sol`), or an adopted system's address (GM only). */
  nav?: NavSystem;
  fixture?: string;
  system?: string;
  at_s?: number;
  ship: string | NavShip;
  /** Body names or addresses. */
  worlds: string[];
  /** Speed arriving from far away; default a Hohmann transfer from the next body in. */
  v_inf_m_s?: number;
}

export interface Chance { p: number; basis: string }
export interface ShipEffect { kept: string; op: "cap" | "set"; value: number }

export interface Danger {
  id: string;
  subject: "crew" | "hull" | "shield" | "drive" | "ship" | string;
  role: "pilot" | "navigator" | "engineer" | string;
  figure: number;
  limit: number;
  unit: string;
  ratio: number;
  level: "ok" | "caution" | "danger";
  chance: Chance;
  if_wrong: string;
  effects?: ShipEffect[];
  why: string;
}

export type Verdict = "safe" | "risky" | "dangerous" | "not-here";

export interface WorldCheck {
  id: "aerocapture" | "entry" | "runway-landing" | "powered-landing" | "take-off" | string;
  words: string;
  verdict: Verdict;
  figures?: Record<string, number>;
  dangers?: Danger[];
  would_pass?: string[];
  why: string[];
}

export interface WorldAir { pressure_pa: number; temperature_k: number; molar_kg: number; scale_height_m?: number; gas: string; ground: boolean }

export interface ShipAtWorld {
  body: string;
  name: string;
  gravity_m_s2: number;
  orbit_m_s: number;
  escape_m_s: number;
  v_inf_m_s: number;
  arrival: string;
  air?: WorldAir;
  checks: WorldCheck[];
}

export interface ShipAtWorldReport { system: string; ship: NavShip; worlds: ShipAtWorld[]; model: string[] }

/** A verdict in a few words for a card. */
export function verdictWords(v: Verdict): string {
  switch (v) {
    case "safe": return "within limits";
    case "risky": return "close to the limits";
    case "dangerous": return "past a limit";
    default: return "not possible here";
  }
}

/** A rough chance in words, never a dice target: "about 1 in 4". */
export function chanceWords(p: number): string {
  if (!(p > 0.005)) return "very unlikely";
  if (p > 0.995) return "near certain";
  if (p >= 0.5) return `about ${Math.round(p * 100)} in 100`;
  return `about 1 in ${Math.max(2, Math.round(1 / p))}`;
}

/** The dangers of a check that matter, worst first (caution and danger only). */
export function worstDangers(c: WorldCheck): Danger[] {
  const rank = { danger: 2, caution: 1, ok: 0 } as const;
  return (c.dangers ?? []).filter((d) => d.level !== "ok").sort((a, b) => rank[b.level] - rank[a.level] || b.chance.p - a.chance.p || a.id.localeCompare(b.id));
}

/** The risky option beside a course's arrival burn (Alex, 23 Nov: offered, never forbidden): braking in the
 * destination's air instead, when the kernel finds air there to brake in. The burn it saves and the kernel's check. */
export function aerobrakeOption(burns: readonly { what: string; dv_m_s: number }[], world: ShipAtWorld | undefined): { check: WorldCheck; saves_m_s: number } | null {
  const arr = burns.find((b) => b.what === "arrival burn");
  const check = world?.checks.find((c) => c.id === "aerocapture");
  return arr && check && check.verdict !== "not-here" ? { check, saves_m_s: arr.dv_m_s } : null;
}

// V3 gap item 14, stage 4: what a ship can do shapes how it looks. A ship that must aerobrake gets a heat shield, a
// ship that lands where there is air gets wings or a lifting body, landers get legs, a parachute where the air can
// carry one, and a ship that refuels by skimming a giant gets a scoop (Alex, 23 Nov 2026: "aerobraking ships need to
// look like they can"; "ships that can land will have aero").
//
// The designer does not judge the physics of entry itself: the kernel's `ship_at_world` (sse-nav `entry`) does. The
// designer gives the kernel a ship with parts sized from its layout, reads back each danger's figure and limit, and
// scales the part by the law the figure follows until the figure sits a margin inside its limit:
//   stagnation heating q ∝ 1/√r_n (Sutton and Graves), so r_n' = r_n (q / (margin q_max))²;
//   stall speed v_s ∝ 1/√S (lift equals weight), so S' = S (v_s / (margin v_max))²;
//   a parachute's terminal speed v = √(2 m g / (ρ C_d A)), so A = 2 m g / (ρ C_d v²).
// Then it asks once more and keeps the kernel's answer for the words. Nothing is refused: a part that cannot be
// made big enough is drawn at its largest and the words say how far short it falls (danger is drama).
import { constant } from "../../ui-contract/src/index.ts";
import type { NavShip, ShipAtWorld, ShipAtWorldAsk, ShipAtWorldReport, WorldCheck } from "../../ui-contract/src/index.ts";
import type { ShipLayout } from "./ship-design.ts";

/** Where the ship must be able to aerobrake and land: body names in the system asked about. */
export interface Abilities {
  aerobrake?: string[];
  land?: string[];
}

export interface ShieldKind {
  kind: "reusable" | "ablative";
  name: string;
  max_heat_flux_w_m2: number;
  /** Tonnes per m² of a reusable shield, whatever the pass. */
  areal_kg_m2?: number;
  /** An ablator's effective heat of ablation, J per kg burnt away. */
  heat_of_ablation_j_kg?: number;
  min_areal_kg_m2?: number;
}

/** base-ships `ship_design.abilities`. */
export interface AbilityRules {
  /** A part is sized so its figure sits at this share of its limit (the kernel's own "would pass" mark). */
  margin: number;
  /** Lightest first; the first that does the job within `shield_base_max_r` is fitted. */
  shields: ShieldKind[];
  /** A blunt sphere-cone's nose radius over its base radius, at the bluntest. */
  nose_over_base: number;
  /** Largest shield base, in hull radii, before it would have to be a deployable. */
  shield_base_max_r: number;
  /** Drag coefficients of the shapes the designer makes. */
  cd: { shield: number; open: number; skin: number };
  wing: { cl_max: number; aspect: number; sweep_deg: number; l_over_d: number; max_span_over_length: number };
  lifting_body: { cl_max: number; l_over_d: number };
  legs: { max_touchdown_m_s: number };
  chute: { cd: number; max_mach: number; max_q_pa: number; fill_s: number; max_area_over_front: number };
  scoop: { skim_density_kg_m3: number; skim_speed_m_s: number; fill_days: number };
  /** The shield face's emissivity, for the pass view's glow. */
  shield_emissivity?: number;
}

/** What the designer fitted, for the drawing. */
export interface Fitted {
  shield?: { kind: string; name: string; base_r: number; nose_r: number; mass_t: number };
  lift?: { form: "wings" | "lifting body"; area_m2: number; span_m: number; chord_m: number; sweep_deg: number };
  legs?: { gear: "legs" | "wheels" };
  chute?: { area_m2: number; diameter_m: number };
  scoop?: { area_m2: number; radius_m: number };
  /** Worlds where it lands on a runway. */
  runway?: string[];
  /** What the designer did and could not do, in words (the layout carries them). */
  notes?: string[];
}

export interface FitResult {
  /** What was asked of it. */
  abilities: Abilities;
  /** The ship with its shield, aero and landing fields written, as NAV reads it. */
  ship: NavShip;
  fit: Fitted;
  /** The kernel's last answer for the fitted ship. */
  report: ShipAtWorldReport | null;
  /** What the designer did and could not do, in words. */
  notes: string[];
}

export type AskShipAtWorld = (a: ShipAtWorldAsk) => Promise<{ ok: true; report: ShipAtWorldReport } | { ok: false; code: string; message: string }>;

/** The ship's outline as the air sees it: largest radius off the axis, length, and the body's planform. */
export function outlineOf(l: ShipLayout): { front_r: number; length_m: number; planform_m2: number } {
  const bodies = l.pieces.filter((p) => p.shape === "body" && p.system !== "spine" && p.system !== "plume");
  const front_r = Math.max(...bodies.map((p) => Math.max(p.r0, p.r1) + Math.hypot(p.y, p.z)));
  // Seen from above, each body on the axis is a trapezium.
  const planform_m2 = bodies.filter((p) => p.y === 0 && p.z === 0).reduce((a, p) => a + (p.r0 + p.r1) * (p.x1 - p.x0), 0);
  return { front_r, length_m: l.length_m, planform_m2 };
}

const danger = (c: WorldCheck | undefined, id: string) => c?.dangers?.find((d) => d.id === id);
const check = (w: ShipAtWorld, id: string) => w.checks.find((c) => c.id === id);
const hasAir = (w: ShipAtWorld) => w.air !== undefined && w.air.pressure_pa > 0;

/** Size the ship's shield, lift, legs, parachute and scoop from what it must do, asking the kernel at each step. */
export async function fitAbilities(ask: AskShipAtWorld, where: Pick<ShipAtWorldAsk, "fixture" | "nav" | "system" | "at_s" | "v_inf_m_s">, base: NavShip & { refuel?: string[] }, l: ShipLayout, hull: "open" | "skin", abilities: Abilities, rules: AbilityRules): Promise<FitResult> {
  const o = outlineOf(l);
  const notes: string[] = [];
  const fit: Fitted = {};
  const { refuel, ...nav } = base;
  const front = Math.PI * o.front_r * o.front_r;
  let ship: NavShip = { ...nav, aero: { ref_area_m2: front, cd: hull === "skin" ? rules.cd.skin : rules.cd.open } };

  // Skimming a giant fills the tanks through an intake: area from the mass to gather, the air's density and speed.
  if (refuel?.includes("scoop")) {
    const s = rules.scoop;
    const kg = ((base.propellant_t ?? base.wet_t - base.dry_t) || 0) * 1e3;
    const area = kg / (s.skim_density_kg_m3 * s.skim_speed_m_s * s.fill_days * constant("day"));
    fit.scoop = { area_m2: area, radius_m: Math.sqrt(area / Math.PI) };
  }

  const brake = [...new Set(abilities.aerobrake ?? [])];
  const land = [...new Set(abilities.land ?? [])];
  const worlds = [...new Set([...brake, ...land])];
  if (!worlds.length) return { abilities, ship, fit, report: null, notes };

  const run = async (s: NavShip) => {
    const r = await ask({ ...where, ship: s, worlds });
    if (!r.ok) throw new Error(`ship_at_world: ${r.message}`);
    return r.report;
  };

  // Landers stand on legs; a ship that lands on a runway rolls on wheels, which take the same sink rate.
  if (land.length) {
    fit.legs = { gear: "legs" };
    ship = { ...ship, landing: { legs: true, max_touchdown_m_s: rules.legs.max_touchdown_m_s } };
  }

  // 1. Lift: a trial wing as big as the hull's planform, then scaled to land at a margin under the runway's speed.
  let report = await run(land.length ? { ...ship, aero: { ...ship.aero!, lift_area_m2: o.planform_m2, cl_max: rules.wing.cl_max } } : ship);
  const byName = (r: ShipAtWorldReport, n: string) => r.worlds.find((w) => w.name === n || w.body === n);
  let wingNeed = 0;
  const thin: string[] = [];
  const runway: string[] = [];
  const powered: string[] = [];
  for (const n of land) {
    const w = byName(report, n);
    if (!w || !hasAir(w) || !w.air!.ground) continue;
    // A drive strong enough to hover sets the ship down itself: no wings to carry (its shield and legs still).
    if (danger(check(w, "powered-landing"), "powered-landing.thrust")?.level === "ok") {
      powered.push(n);
      continue;
    }
    const d = danger(check(w, "runway-landing"), "runway.speed");
    if (!d) continue;
    const need = o.planform_m2 * (d.figure / (rules.margin * d.limit)) ** 2;
    const span = Math.sqrt(rules.wing.aspect * need);
    if (span > rules.wing.max_span_over_length * o.length_m) thin.push(n);
    else {
      wingNeed = Math.max(wingNeed, need);
      runway.push(n);
    }
  }
  if (powered.length) notes.push(`Its drive is strong enough to hover and set it down at ${powered.join(" and ")}, so it carries no wings for ${powered.length > 1 ? "them" : "it"}.`);
  for (const n of thin) notes.push(`The air at ${n} is too thin for wings to land this ship: it comes down on its drive and legs.`);
  if (wingNeed > 0) {
    // A lifting body if the hull's own planform carries it at the lifting body's lower lift coefficient; else wings.
    const asBody = (wingNeed * rules.wing.cl_max) / rules.lifting_body.cl_max <= o.planform_m2;
    if (asBody) {
      fit.lift = { form: "lifting body", area_m2: o.planform_m2, span_m: 0, chord_m: 0, sweep_deg: 0 };
      ship = { ...ship, aero: { ...ship.aero!, lift_area_m2: o.planform_m2, cl_max: rules.lifting_body.cl_max, l_over_d: rules.lifting_body.l_over_d } };
    } else {
      const span = Math.sqrt(rules.wing.aspect * wingNeed);
      fit.lift = { form: "wings", area_m2: wingNeed, span_m: span, chord_m: wingNeed / span, sweep_deg: rules.wing.sweep_deg };
      ship = { ...ship, aero: { ...ship.aero!, lift_area_m2: wingNeed, cl_max: rules.wing.cl_max, l_over_d: rules.wing.l_over_d } };
    }
    fit.legs = { gear: "wheels" };
    fit.runway = runway;
  }

  // 2. The shield: wherever the ship meets air at speed. A blunt nose first, as wide as the hull, then as wide as it
  // must be for the lightest material, then a stronger material.
  const hot = (r: ShipAtWorldReport) => {
    const out: { q: number; load: number; where: string }[] = [];
    for (const w of r.worlds) {
      if (!hasAir(w)) continue;
      if (brake.includes(w.name) || brake.includes(w.body)) {
        const c = check(w, "aerocapture");
        const d = danger(c, "aerocapture.heating");
        if (d) out.push({ q: d.figure, load: c?.figures?.heat_load_j_m2 ?? 0, where: w.name });
      }
      if (land.includes(w.name) || land.includes(w.body)) {
        const c = check(w, "entry");
        const d = danger(c, "entry.heating");
        if (d) out.push({ q: d.figure, load: c?.figures?.heat_load_j_m2 ?? 0, where: w.name });
      }
    }
    return out;
  };
  for (const n of brake) {
    const w = byName(report, n);
    if (w && !hasAir(w)) notes.push(`${n} has no air to brake in: the ship burns to stop there.`);
  }
  const maxR = rules.shield_base_max_r * o.front_r;
  const withShield = (k: ShieldKind, baseR: number, loadLimit?: number, mass = 0): NavShip => ({
    ...ship,
    aero: { ...ship.aero!, ref_area_m2: Math.max(front, Math.PI * baseR * baseR), cd: Math.max(ship.aero!.cd, rules.cd.shield) },
    shield: { nose_radius_m: baseR * rules.nose_over_base, max_heat_flux_w_m2: k.max_heat_flux_w_m2, kind: k.kind, ...(loadLimit ? { max_heat_load_j_m2: loadLimit } : {}), ...(mass ? { mass_t: mass } : {}) },
  });
  if (hot(report).length) {
    let chosen: { k: ShieldKind; baseR: number; short?: number } | null = null;
    for (const k of rules.shields) {
      // The trajectory moves a little as the shield widens (more drag, a higher pass): settle in a few rounds.
      let baseR = o.front_r;
      for (let round = 0; round < 3; round++) {
        const r = await run(withShield(k, baseR));
        const rn = baseR * rules.nose_over_base;
        const need = Math.max(...hot(r).map((h) => rn * (h.q / (rules.margin * k.max_heat_flux_w_m2)) ** 2)) / rules.nose_over_base;
        baseR = Math.max(o.front_r, need);
        if (baseR > maxR) break;
      }
      if (baseR <= maxR) {
        chosen = { k, baseR };
        break;
      }
      chosen = { k, baseR: maxR, short: baseR };
    }
    const { k, baseR, short } = chosen!;
    // Mass: a reusable shield weighs its areal density over the cap; an ablator burns away the pass's heat load (with
    // the margin), and never less than its thinnest layer. A sphere-cone cap's area is taken as its base disc's.
    const area = Math.PI * baseR * baseR;
    const r = await run(withShield(k, baseR));
    const load = Math.max(0, ...hot(r).map((h) => h.load));
    const areal = k.kind === "ablative" ? Math.max(k.min_areal_kg_m2 ?? 0, load / rules.margin / (k.heat_of_ablation_j_kg ?? Infinity)) : (k.areal_kg_m2 ?? 0);
    const mass_t = (areal * area) / 1e3;
    const loadLimit = k.kind === "ablative" ? areal * (k.heat_of_ablation_j_kg ?? 0) : undefined;
    ship = withShield(k, baseR, loadLimit, mass_t);
    fit.shield = { kind: k.kind, name: k.name, base_r: baseR, nose_r: baseR * rules.nose_over_base, mass_t };
    if (short) notes.push(`No shield up to ${rules.shield_base_max_r} times the hull's width keeps the heating inside the limits of ${k.name}; it would need to be ${Math.round(short)} m across the base. The shield is drawn at its largest and the passes stay dangerous.`);
    if (l.needs.radiator_m2 > 0) notes.push("Its radiators must fold behind the shield before it meets the air.");
    notes.push(`The ${k.name} heat shield weighs about ${mass_t < 10 ? mass_t.toFixed(1) : Math.round(mass_t)} t, inside the ship's dry mass.`);
  }

  // 3. A parachute where the air can carry the ship down slowly enough for its legs; where it cannot, the drive lands it.
  const offRunway = land.filter((n) => !runway.includes(n) && !powered.includes(n));
  if (offRunway.length) {
    report = await run(ship);
    const c = rules.chute;
    const kg = (ship.propellant_t !== undefined ? ship.dry_t + ship.propellant_t : ship.wet_t) * 1e3;
    let need = 0;
    for (const n of offRunway) {
      const w = byName(report, n);
      if (!w || !hasAir(w) || !w.air!.ground) continue;
      const td = danger(check(w, "entry"), "entry.touchdown");
      if (!td || td.figure <= rules.margin * td.limit) continue;
      const rho = (w.air!.pressure_pa * w.air!.molar_kg) / (constant("R_gas") * w.air!.temperature_k);
      const v = rules.margin * td.limit;
      const a = (2 * kg * w.gravity_m_s2) / (rho * c.cd * v * v);
      if (a > c.max_area_over_front * Math.PI * o.front_r * o.front_r) notes.push(`At ${n} no parachute the ship could carry slows it to its legs' ${td.limit.toFixed(0)} m/s: the drive brakes the last of the way.`);
      else need = Math.max(need, a);
    }
    if (need > 0) {
      fit.chute = { area_m2: need, diameter_m: 2 * Math.sqrt(need / Math.PI) };
      ship = { ...ship, landing: { ...ship.landing!, chute: { area_m2: need, cd: c.cd, max_mach: c.max_mach, max_q_pa: c.max_q_pa, fill_s: c.fill_s } } };
    }
  }

  report = await run(ship);
  if (notes.length) fit.notes = notes;
  return { abilities, ship, fit, report, notes };
}

/** The kernel's checks for the acts the ship was fitted for, in the order a voyage meets them: braking where it must
 * aerobrake; at a landing world the runway if it has wings for one (else the entry), then setting down on its drive
 * and climbing back to orbit. */
export function fittedChecks(r: FitResult): { world: string; check: WorldCheck }[] {
  const out: { world: string; check: WorldCheck }[] = [];
  const is = (list: string[] | undefined, w: ShipAtWorld) => (list ?? []).some((n) => n === w.name || n === w.body);
  for (const w of r.report?.worlds ?? []) {
    const ids: string[] = [];
    if (is(r.abilities.aerobrake, w)) ids.push("aerocapture");
    if (is(r.abilities.land, w)) ids.push(is(r.fit.runway, w) ? "runway-landing" : "entry", "powered-landing", "take-off");
    for (const id of ids) {
      const c = check(w, id);
      if (c && c.verdict !== "not-here") out.push({ world: w.name, check: c });
    }
  }
  return out;
}

/** What an aerobraking pass looks like at its hottest (for the True pass view): the shield face at radiative balance
 * with the peak heating, q = ε σ T⁴ (an upper bound for a reusable face; an ablator runs cooler by what it carries
 * away), and the bow shock standing off the nose by about 0.14 nose radii at hypersonic speed, curved at about 1.14
 * nose radii (Billig 1967). The shock layer's glow is a display choice (fudge register "ship pass glow"). */
export function passGlow(peak_heat_w_m2: number, emissivity: number): { shield_k: number; standoff_rn: number; shock_curve_rn: number; glow: number } {
  const q = Math.max(0, peak_heat_w_m2);
  const shield_k = Math.pow(q / (Math.max(emissivity, 1e-3) * constant("sigma_SB")), 0.25);
  return { shield_k, standoff_rn: 0.143, shock_curve_rn: 1.143, glow: Math.min(1, q / 1e6) };
}

// A game system's own difficulty numbers beside SSE4's words (Alex, 23 Nov). SSE4 rolls nothing and decides nothing:
// where it says how hard something is (a voyage's difficulty word, or a hazard's chance of going wrong at a world), a
// game pack that declares its task ladder (`provides.other.difficulty`) gets that system's name and target shown
// beside it, on the GM's screens only, with the skill for the role when the pack names the game's skills.
//
// Where SSE4 has odds, the step is the one whose chance of success for a typical operator (the pack's `typical_dm`)
// is nearest the chance of things going right, worked out exactly from the system's dice. Where it has only a word,
// the pack's `words` table maps it. Without a ladder in the stack, nothing is shown.

/** One step on a system's ladder: a name and either a target to meet or a modifier on the base target. */
export interface DifficultyStep {
  name: string;
  /** Target number to meet or beat. */
  target?: number;
  /** Or: a modifier on the roll against `base_target` (Cepheus-style "Difficult, DM -2"). */
  dm?: number;
}

/** A game system's task ladder (`provides.other.difficulty`). */
export interface DifficultyLadder {
  /** The system's name for the check, shown with the numbers. */
  system: string;
  /** Dice rolled and added: `count` dice of `sides` faces. */
  dice: { count: number; sides: number };
  /** Target the roll plus modifiers must meet or beat when steps give modifiers. */
  base_target?: number;
  /** A typical trained operator's total modifier (skill plus characteristic); used to turn odds into a step. */
  typical_dm: number;
  steps: DifficultyStep[];
  /** SSE4's difficulty words (routine, easy, moderate, hard, very hard) to step names. */
  words?: Record<string, string>;
  /** SSE4's roles (pilot, navigator, engineer) to the game's skill names. */
  skills?: Record<string, string>;
  source?: string;
}

/** What the GM sees beside SSE4's words. */
export interface GameDifficulty {
  system: string;
  step: string;
  /** Target to meet or beat on the dice, modifiers aside. */
  target: number;
  /** The step's modifier when the system states it that way. */
  dm?: number;
  skill?: string;
  /** "odds": from SSE4's chance through the dice; "words": from the pack's table. */
  basis: "odds" | "words";
  /** A typical operator's chance of success at this step. */
  p_success: number;
  words: string;
}

/** The topmost pack in the stack that declares a ladder (later packs sit higher). */
export function difficultyLadderOf(packs: readonly { provides?: { other?: Record<string, unknown> } }[]): DifficultyLadder | null {
  let found: DifficultyLadder | null = null;
  for (const p of packs) {
    const d = p.provides?.other?.difficulty as DifficultyLadder | undefined;
    if (d && Array.isArray(d.steps) && d.steps.length && d.dice?.count > 0 && d.dice?.sides > 1) found = d;
  }
  return found;
}

/** Chance that `count`d`sides` + `dm` meets or beats `target`, exactly. */
export function chanceToMeet(dice: { count: number; sides: number }, target: number, dm = 0): number {
  let dist = [1];
  for (let i = 0; i < dice.count; i++) {
    const next = new Array(dist.length + dice.sides - 1).fill(0) as number[];
    for (let s = 0; s < dist.length; s++) for (let f = 0; f < dice.sides; f++) next[s + f]! += dist[s]! / dice.sides;
    dist = next;
  }
  // dist[k] is the chance the dice total count + k.
  let p = 0;
  for (let k = 0; k < dist.length; k++) if (dice.count + k + dm >= target) p += dist[k]!;
  return Math.min(1, p);
}

/** A step's target to meet on the dice before any modifier. */
export function stepTarget(l: DifficultyLadder, s: DifficultyStep): number {
  if (s.target !== undefined) return s.target;
  return (l.base_target ?? 0) - (s.dm ?? 0);
}

function shown(l: DifficultyLadder, s: DifficultyStep, basis: GameDifficulty["basis"], role?: string): GameDifficulty {
  const target = stepTarget(l, s);
  const skill = role ? l.skills?.[role] : undefined;
  const p = chanceToMeet(l.dice, target, l.typical_dm);
  const head = skill ? `${skill}, ${s.name}` : s.name;
  return {
    system: l.system,
    step: s.name,
    target,
    ...(s.dm !== undefined ? { dm: s.dm } : {}),
    ...(skill ? { skill } : {}),
    basis,
    p_success: p,
    words: `${head} ${target}+`,
  };
}

/**
 * The system's step for something SSE4 rates: by its chance of going wrong (`p_wrong`, 0 to 1) when it has one, else
 * by its difficulty word. Null when the ladder cannot place it.
 */
export function gameDifficulty(l: DifficultyLadder | null, ask: { p_wrong?: number; word?: string; role?: string }): GameDifficulty | null {
  if (!l) return null;
  if (ask.p_wrong !== undefined && Number.isFinite(ask.p_wrong)) {
    const want = 1 - Math.min(1, Math.max(0, ask.p_wrong));
    let best = l.steps[0]!;
    let gap = Infinity;
    for (const s of l.steps) {
      const g = Math.abs(chanceToMeet(l.dice, stepTarget(l, s), l.typical_dm) - want);
      if (g < gap - 1e-12) (best = s), (gap = g);
    }
    return shown(l, best, "odds", ask.role);
  }
  const name = ask.word ? l.words?.[ask.word] : undefined;
  const s = name ? l.steps.find((x) => x.name === name) : undefined;
  return s ? shown(l, s, "words", ask.role) : null;
}

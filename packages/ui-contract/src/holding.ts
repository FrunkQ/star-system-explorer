// Holding cards as pack data (PB5; design `handover/design/SSE4-playbook-and-switchboard-design.md` 6.2 and 6.3).
// A held screen shows one line of words. The words come from `sse.holding-cards/1` in a pack's `provides.other`:
// the base presentation pack has the four plain cards and the 24 jokes; a later pack adds cards, or replaces a kind
// ("replace": ["joke"]) so a setting's screens speak in its own voice. The deck picks the words on the GM's machine,
// once per hold: every screen held with the same card shows the same words while the hold lasts, and a joke is never
// the same one twice running.
import type { Board, HoldCard } from "./board.ts";
import type { PackDoc } from "./physics.ts";

export type CardKind = "plain" | "joke";
export interface HoldingCard {
  kind: CardKind;
  words: string;
  /** The pack it came from. */
  pack: string;
}
interface CardsDoc {
  schema?: string;
  replace?: unknown;
  cards?: unknown;
}

/** The cards on a stack of packs, in order; a later pack's `replace` drops the earlier cards of those kinds. */
export function holdingCardsOn(packs: readonly PackDoc[]): HoldingCard[] {
  let out: HoldingCard[] = [];
  for (const p of packs) {
    const doc = (p.provides?.other as { holding_cards?: CardsDoc } | undefined)?.holding_cards;
    if (!doc || doc.schema !== "sse.holding-cards/1") continue;
    const replace = Array.isArray(doc.replace) ? doc.replace.filter(isKind) : [];
    if (replace.length) out = out.filter((c) => !replace.includes(c.kind));
    for (const c of Array.isArray(doc.cards) ? doc.cards : []) {
      const { kind, words } = (c ?? {}) as { kind?: unknown; words?: unknown };
      if (!isKind(kind) || typeof words !== "string" || !words.trim()) continue;
      const w = words.trim();
      if (!out.some((x) => x.kind === kind && x.words === w)) out.push({ kind, words: w, pack: p.id });
    }
  }
  return out;
}
const isKind = (k: unknown): k is CardKind => k === "plain" || k === "joke";

/** What a card is, in a few words, for the GM's side (the line's label and the line card's choices). */
export function cardLabel(card: HoldCard): string {
  return card === "blank" ? "dark" : card === "plain" ? "awaiting transmission" : card === "joke" ? "a joke" : card.text;
}

/**
 * The words held screens show. `words(card)` gives the same words for a card while any line still holds it (so every
 * screen of a role, and every role held with it, agrees); `settle(board)` forgets cards no line holds any more, so the
 * next hold draws afresh. A kind never gives the same words twice running. A kind with no cards falls back to the
 * other kind, then to "Stand by".
 */
export class CardDeck {
  #cards: HoldingCard[];
  #rand: () => number;
  #held = new Map<string, string>();
  #last: Partial<Record<CardKind, string>> = {};
  constructor(cards: readonly HoldingCard[], rand: () => number = Math.random) {
    this.#cards = [...cards];
    this.#rand = rand;
  }
  set cards(cards: readonly HoldingCard[]) {
    this.#cards = [...cards];
  }
  words(card: HoldCard): string {
    if (card === "blank") return "";
    if (typeof card === "object") return card.text;
    const had = this.#held.get(card);
    if (had !== undefined) return had;
    const w = this.#draw(card);
    this.#held.set(card, w);
    return w;
  }
  settle(b: Board): void {
    const holding = new Set<string>();
    if (typeof b.all === "string") holding.add(b.all);
    for (const l of Object.values(b.lines)) if (typeof l?.held === "string") holding.add(l.held);
    for (const k of [...this.#held.keys()]) if (!holding.has(k)) this.#held.delete(k);
  }
  #draw(kind: CardKind): string {
    let list = this.#cards.filter((c) => c.kind === kind);
    if (!list.length) list = this.#cards.filter((c) => c.kind !== kind);
    if (!list.length) return "Stand by";
    const choices = list.length > 1 ? list.filter((c) => c.words !== this.#last[kind]) : list;
    const pick = choices[Math.min(choices.length - 1, Math.floor(this.#rand() * choices.length))]!.words;
    this.#last[kind] = pick;
    return pick;
  }
}

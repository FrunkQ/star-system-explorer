// Holding cards as pack data (PB5; design `handover/design/SSE4-playbook-and-switchboard-design.md` 6.2 and 6.3).
// A held screen shows one line of words. The words come from `sse.holding-cards/1` in a pack's `provides.other`:
// the base presentation pack has the four plain cards and the 24 jokes; a later pack adds cards, or replaces a kind
// ("replace": ["joke"]) so a setting's screens speak in its own voice. The deck picks the words on the GM's machine,
// once per hold: every screen held with the same card shows the same words while the hold lasts, and a joke is never
// the same one twice running.
import { holdKind, holdText, type Board, type HeldWords, type HoldCard } from "./board.ts";
import type { PackDoc } from "./physics.ts";

export type CardKind = "plain" | "joke" | "quote";
export interface HoldingCard {
  kind: CardKind;
  words: string;
  /** Who said it, for a quote (postcards design 8.10): shown beneath, never part of the card's identity. */
  by?: string;
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
      const { kind, words, by } = (c ?? {}) as { kind?: unknown; words?: unknown; by?: unknown };
      if (!isKind(kind) || typeof words !== "string" || !words.trim()) continue;
      const w = words.trim();
      // Cards are the same card by kind and words; who said it is not part of that (the first one's `by` stays).
      const who = kind === "quote" && typeof by === "string" && by.trim() ? by.trim().slice(0, 120) : undefined;
      if (!out.some((x) => x.kind === kind && x.words === w)) out.push({ kind, words: w, ...(who ? { by: who } : {}), pack: p.id });
    }
  }
  return out;
}
const isKind = (k: unknown): k is CardKind => k === "plain" || k === "joke" || k === "quote";

/** What a card is, in a few words, for the GM's side (the line's label and the line card's choices). */
export function cardLabel(card: HoldCard): string {
  const text = holdText(card);
  if (text !== null) return text;
  const k = holdKind(card);
  return k === "blank" ? "dark" : k === "plain" ? "awaiting transmission" : k === "joke" ? "a joke" : "a quote";
}

/**
 * The words held screens show. `words(card)` gives the same words for a card while any line still holds it (so every
 * screen of a role, and every role held with it, agrees); `settle(board)` forgets cards no line holds any more, so the
 * next hold draws afresh. A kind never gives the same words twice running. A kind with no cards falls back to any
 * other kind (plain, then joke, then quote), then to "Stand by" (postcards design 8.10).
 */
export class CardDeck {
  #cards: HoldingCard[];
  #rand: () => number;
  #held = new Map<string, HeldWords>();
  #last: Partial<Record<CardKind, string>> = {};
  constructor(cards: readonly HoldingCard[], rand: () => number = Math.random) {
    this.#cards = [...cards];
    this.#rand = rand;
  }
  set cards(cards: readonly HoldingCard[]) {
    this.#cards = [...cards];
  }
  words(card: HoldCard): string {
    return this.said(card).card;
  }
  /** The words and, for a quote, who said them. The card's ground does not change its words. */
  said(card: HoldCard): HeldWords {
    const text = holdText(card);
    if (text !== null) return { card: text };
    const kind = holdKind(card)!;
    if (kind === "blank") return { card: "" };
    const had = this.#held.get(kind);
    if (had !== undefined) return had;
    const w = this.#draw(kind);
    this.#held.set(kind, w);
    return w;
  }
  settle(b: Board): void {
    const holding = new Set<string>();
    const note = (c: HoldCard | undefined) => {
      const k = c === undefined ? null : holdKind(c);
      if (k) holding.add(k);
    };
    note(b.all);
    for (const l of Object.values(b.lines)) note(l?.held);
    for (const k of [...this.#held.keys()]) if (!holding.has(k)) this.#held.delete(k);
  }
  #draw(kind: CardKind): HeldWords {
    let list = this.#cards.filter((c) => c.kind === kind);
    for (const other of ["plain", "joke", "quote"] as const) if (!list.length && other !== kind) list = this.#cards.filter((c) => c.kind === other);
    if (!list.length) return { card: "Stand by" };
    const choices = list.length > 1 ? list.filter((c) => c.words !== this.#last[kind]) : list;
    const pick = choices[Math.min(choices.length - 1, Math.floor(this.#rand() * choices.length))]!;
    this.#last[kind] = pick.words;
    return { card: pick.words, ...(pick.kind === "quote" && pick.by ? { by: pick.by } : {}) };
  }
}

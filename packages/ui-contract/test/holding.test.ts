// Holding cards as pack data (PB5): the base presentation pack's 4 plain cards and 24 jokes; a later pack adds or
// replaces a kind; the deck gives the same words while a hold lasts and never the same joke twice running.
import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { CardDeck, cardLabel, holdingCardsOn } from "../src/holding.ts";
import { boardWire, emptyBoard, hold, release, holdAll } from "../src/board.ts";
import type { PackDoc } from "../src/physics.ts";

const root = new URL("../../../", import.meta.url);
const base: PackDoc = JSON.parse(readFileSync(new URL("packs/base-presentation/pack.json", root), "utf8"));
const pack = (id: string, holding_cards: unknown): PackDoc => ({
  format: "sse.pack/1",
  id,
  version: "0.1.0",
  kind: ["presentation"],
  kernel_range: "^0.1",
  licence: "Apache-2.0",
  provides: { other: { holding_cards } },
});

test("the base pack has the four plain cards and the 24 jokes, Alex's first", () => {
  const cards = holdingCardsOn([base]);
  assert.deepEqual(
    cards.filter((c) => c.kind === "plain").map((c) => c.words),
    ["Awaiting transmission", "Signal lost", "Stand by", "No carrier"],
  );
  const jokes = cards.filter((c) => c.kind === "joke");
  assert.equal(jokes.length, 24);
  assert.match(jokes[0]!.words, /missing gold atoms/);
  assert.equal(new Set(jokes.map((c) => c.words)).size, 24, "no joke twice");
  assert.ok(cards.every((c) => c.pack === "sse.base-presentation"));
});

test("a later pack adds cards, or replaces a kind; bad cards and other schemas are left out", () => {
  const add = pack("fan.cards", {
    schema: "sse.holding-cards/1",
    cards: [
      { kind: "plain", words: " Comms down " },
      { kind: "plain", words: "Stand by" },
      { kind: "sparkle", words: "x" },
      { kind: "joke", words: "" },
    ],
  });
  const plus = holdingCardsOn([base, add]);
  assert.equal(plus.filter((c) => c.kind === "plain").length, 5, "one new plain card; the same words are not added twice");
  assert.equal(plus.at(-1)!.words, "Comms down");
  const setting = pack("setting.cards", {
    schema: "sse.holding-cards/1",
    replace: ["joke"],
    cards: [
      {
        kind: "joke",
        words: "The Emperor's signal is late. Nobody tells the Emperor.",
      },
    ],
  });
  const mine = holdingCardsOn([base, setting]);
  assert.deepEqual(
    mine.filter((c) => c.kind === "joke").map((c) => c.pack),
    ["setting.cards"],
  );
  assert.equal(mine.filter((c) => c.kind === "plain").length, 4, "replacing jokes keeps the plain cards");
  assert.equal(
    holdingCardsOn([
      pack("old", {
        schema: "sse.holding-cards/0",
        cards: [{ kind: "plain", words: "Old" }],
      }),
    ]).length,
    0,
  );
});

test("the deck: the same words while held, fresh words after, never the same joke twice running", () => {
  const deck = new CardDeck(holdingCardsOn([base]), () => 0);
  let b = hold(emptyBoard({ scope: "k", title: "Kestrel" }), "role:bridge", "joke", ["bridge"]);
  deck.settle(b);
  const first = deck.words("joke");
  assert.equal(deck.words("joke"), first, "every screen held with a joke shows the same one");
  b = release(b, "role:bridge");
  deck.settle(b);
  b = hold(b, "role:bridge", "joke", ["bridge"]);
  deck.settle(b);
  const second = deck.words("joke");
  assert.notEqual(second, first, "never the same joke twice running");
  for (let i = 0; i < 30; i++) {
    const prev = deck.words("joke");
    deck.settle(release(b, "role:bridge"));
    assert.notEqual(deck.words("joke"), prev);
  }
  assert.equal(deck.words("blank"), "");
  assert.equal(deck.words({ text: "Back in five" }), "Back in five");
});

test("Hold all keeps its card's words; a kind with no cards borrows the other, then says Stand by", () => {
  const deck = new CardDeck(holdingCardsOn([base]));
  const b = holdAll(emptyBoard({ scope: "k" }), "plain");
  deck.settle(b);
  const w = boardWire(b, ["bridge"], (c) => deck.words(c));
  assert.equal(w.roles.bridge && "off" in w.roles.bridge ? w.roles.bridge.card : null, (w.shown as { card?: string }).card);
  assert.equal(new CardDeck([{ kind: "plain", words: "Signal lost", pack: "p" }]).words("joke"), "Signal lost");
  assert.equal(new CardDeck([]).words("plain"), "Stand by");
  assert.deepEqual(
    ["blank", "plain", "joke", { text: "Hi" }].map((c) => cardLabel(c as never)),
    ["dark", "awaiting transmission", "a joke", "Hi"],
  );
});

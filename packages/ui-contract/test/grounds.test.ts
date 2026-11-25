// Grounds on held screens and the Quotes kind (PC5, SSE4-postcards-design.md section 8 and 12A points 5 and 6): a held
// card carries a ground on itself, the wire gives each role the postcard it may see, a quote carries who said it and
// stands alone, and the deck falls back to any other kind, then "Stand by".
import { test } from "node:test";
import assert from "node:assert/strict";
import {
  boardWire,
  emptyBoard,
  hold,
  holdAll,
  holdCardFrom,
  holdGround,
  holdKind,
  holdText,
  withGround,
  withWords,
  type HoldCard,
} from "../src/board.ts";
import { CardDeck, cardLabel, holdingCardsOn } from "../src/holding.ts";
import {
  campaignGrounds,
  groundCandidates,
  groundTurns,
  outsideGround,
  pickGround,
} from "../src/grounds.ts";
import { envelope, fixDate } from "../src/postcard.ts";
import { makeBookmark, type Whereabouts } from "../src/bookmark.ts";
import { poseOfGlobe } from "../src/pose.ts";
import { waitingLineFits } from "../src/seat.ts";
import { playbookFrom, showOf, PLAYBOOK_SCHEMA } from "../src/playbook.ts";
import type { PackDoc } from "../src/physics.ts";

const pack = (id: string, holding_cards: unknown): PackDoc => ({
  format: "sse.pack/1",
  id,
  version: "0.1.0",
  kind: ["presentation"],
  kernel_range: "^0.1",
  licence: "Apache-2.0",
  provides: { other: { holding_cards } },
});
const pose = poseOfGlobe("u:sol/mars", 30, 20, 3.39e6);
const at: Whereabouts = {
  scope: "u:sol",
  now_yr: 4.6e9,
  instrument: "body",
  scale: "world",
  view: { style: "true", seen: "free", band: "visible" },
  selected: "u:sol/mars",
  pose,
};
const PIC = "data:image/jpeg;base64,AAAA";

test("a card keeps its ground through a change of words; the bare kinds stay strings", () => {
  const g = { set: "base" };
  const c = withGround("plain", g);
  assert.deepEqual(c, { kind: "plain", ground: g });
  assert.equal(holdKind(c), "plain");
  assert.equal(holdText(c), null);
  assert.deepEqual(withWords(c, "quote"), { kind: "quote", ground: g });
  assert.deepEqual(
    withWords(c, { text: "Incoming transmission…" }),
    { text: "Incoming transmission…", ground: g },
    "a transmission card takes a ground by the same field",
  );
  assert.equal(withGround(c, "none"), "plain");
  assert.equal(holdGround("joke"), "none");
  assert.equal(cardLabel({ kind: "quote", ground: g }), "a quote");
});

test("a hold card is read leniently from a playbook: unknown grounds drop to none, unknown kinds are not cards", () => {
  assert.equal(holdCardFrom("quote"), "quote");
  assert.deepEqual(
    holdCardFrom({ kind: "joke", ground: { bookmark: "bm1" } }),
    { kind: "joke", ground: { bookmark: "bm1" } },
  );
  assert.deepEqual(
    holdCardFrom({ text: "Stand by", ground: { nonsense: 1 } }),
    { text: "Stand by" },
  );
  assert.equal(holdCardFrom({ kind: "dance" }), null);
  assert.equal(holdCardFrom(7), null);
  const pb = playbookFrom({
    schema: PLAYBOOK_SCHEMA,
    chapters: [
      {
        id: "c1",
        title: "One",
        pages: [
          {
            id: "p1",
            title: "P",
            sections: [
              {
                id: "s1",
                title: "S",
                presets: [
                  {
                    id: "k1",
                    title: "Back soon",
                    to: "everyone",
                    steps: [
                      {
                        hold: {
                          card: { kind: "quote", ground: { set: "base" } },
                        },
                      },
                    ],
                  },
                ],
              },
            ],
          },
        ],
      },
    ],
  });
  const step = pb.chapters[0]?.pages[0]?.sections[0]?.presets[0]?.steps[0];
  assert.deepEqual(step, {
    hold: { card: { kind: "quote", ground: { set: "base" } } },
  });
});

test("quotes carry who said them, which is not part of a card's identity; a kind with none falls back to any other", () => {
  const cards = holdingCardsOn([
    pack("a", {
      schema: "sse.holding-cards/1",
      cards: [
        { kind: "quote", words: "That's one small step", by: "Neil Armstrong" },
      ],
    }),
    pack("b", {
      schema: "sse.holding-cards/1",
      cards: [
        { kind: "quote", words: "That's one small step", by: "Someone else" },
        { kind: "plain", words: "Stand by" },
      ],
    }),
  ]);
  assert.deepEqual(
    cards.filter((c) => c.kind === "quote").map((c) => [c.words, c.by]),
    [["That's one small step", "Neil Armstrong"]],
  );
  const deck = new CardDeck(cards, () => 0);
  assert.deepEqual(deck.said("quote"), {
    card: "That's one small step",
    by: "Neil Armstrong",
  });
  assert.deepEqual(
    deck.said({ kind: "quote", ground: { set: "base" } }),
    { card: "That's one small step", by: "Neil Armstrong" },
    "the ground does not change the words",
  );
  const noQuotes = new CardDeck(
    [{ kind: "joke", words: "A joke", pack: "x" }],
    () => 0,
  );
  assert.deepEqual(
    noQuotes.said("quote"),
    { card: "A joke" },
    "any other kind",
  );
  assert.deepEqual(noQuotes.said("plain"), { card: "A joke" });
  assert.deepEqual(new CardDeck([]).said("quote"), { card: "Stand by" });
});

test("one piece of words: a quote or a joke is marked so no waiting line is added; plain and blank may have one", () => {
  const b = hold(
    hold(emptyBoard({ scope: "u:sol" }), "role:bridge", "quote"),
    "role:deck",
    "plain",
  );
  const w = boardWire(b, ["bridge", "deck"], (c) =>
    holdKind(c) === "quote" ? { card: "Ad astra", by: "Seneca" } : "Stand by",
  );
  assert.deepEqual(w.roles.bridge, {
    off: true,
    card: "Ad astra",
    by: "Seneca",
    kind: "quote",
  });
  assert.deepEqual(w.roles.deck, { off: true, card: "Stand by" });
  assert.equal(waitingLineFits(w.roles.bridge), false);
  assert.equal(waitingLineFits(w.roles.deck), true);
  assert.equal(
    waitingLineFits({ scope: "u:sol" }),
    false,
    "a place is not a held screen",
  );
  assert.deepEqual(
    boardWire(holdAll(b, "blank"), ["bridge"]).roles.bridge,
    { off: true },
    "Dark is still nothing at all",
  );
});

test("the ground travels per role: a campaign postcard only to a role that knows its place, else the next in turn", () => {
  const mars = {
    ...fixDate(makeBookmark([], at, "Mars at dawn", "Globe"), 4.6e9),
    id: "bm-mars",
  };
  const vesta = {
    ...fixDate(
      makeBookmark(
        [],
        {
          ...at,
          selected: "u:sol/vesta",
          pose: poseOfGlobe("u:sol/vesta", 0, 0, 3e5),
        },
        "Vesta",
        "Globe",
      ),
      4.6e9,
    ),
    id: "bm-vesta",
  };
  const plain = {
    ...makeBookmark([], { ...at, pose: null }, "Not a postcard", "Globe"),
    id: "bm-plain",
  };
  const campaign = campaignGrounds([mars, vesta, plain], (_s, id) =>
    id === "bm-plain" ? PIC : id === "bm-mars" ? PIC : PIC + "v",
  );
  assert.deepEqual(
    campaign.map((c) => c.id),
    ["bm-mars", "bm-vesta"],
    "only postcards with a kept picture",
  );
  const list = groundCandidates({ set: "campaign" }, { campaign, sets: [] });
  const knows = (role: string) => (_scope: string, about: string) =>
    role === "bridge" ? true : about === "u:sol/mars";
  const b = hold(
    emptyBoard({ scope: "u:sol" }),
    "everyone",
    withGround("plain", { set: "campaign" }),
  );
  const turn = 1;
  const w = boardWire(
    b,
    ["bridge", "deck"],
    () => "Stand by",
    (c, role) =>
      pickGround(
        groundCandidates(holdGround(c), { campaign, sets: [] }),
        turn,
        knows(role),
      ),
  );
  assert.equal(
    (w.roles.bridge as { ground?: { id: string } }).ground?.id,
    "bm-vesta",
    "turn 1 is the second postcard",
  );
  assert.equal(
    (w.roles.deck as { ground?: { id: string } }).ground?.id,
    "bm-mars",
    "the deck does not know Vesta, so the next one in turn",
  );
  assert.equal(
    pickGround(list, 0, () => null),
    null,
    "nothing while what a role knows is still being asked",
  );
  assert.equal(
    pickGround(list, 0, () => false),
    null,
    "no ground when the role knows none of them",
  );
  assert.ok(
    groundTurns({ set: "campaign" }) &&
      !groundTurns({ bookmark: "bm-mars" }) &&
      !groundTurns("none"),
  );
});

test("sets: the universe's default set is the last on the stack with pictures, else the campaign's own; outside postcards need a stamp", () => {
  const card = (id: string, stamp?: string) => ({
    ...envelope(
      fixDate(makeBookmark([], at, id, "Globe"), 1),
      { how: "address", root: "u:sol", packs: [], made_with: "4.0.0" },
      { id, name: id },
    ),
    ...(stamp ? { stamp } : {}),
  });
  const own = [
    {
      id: "own",
      name: "Own",
      picture: PIC,
      here: { scope: "u:sol", about: "u:sol/mars" },
    },
  ];
  const base = { id: "base", cards: [card("Saturn", PIC)] };
  const universe = {
    id: "traveller",
    cards: [card("Regina", PIC), card("No stamp")],
  };
  assert.deepEqual(
    groundCandidates(
      { set: "base" },
      { campaign: own, sets: [base, universe] },
    ).map((c) => c.id),
    ["Regina"],
  );
  assert.deepEqual(
    groundCandidates(
      { set: "base" },
      { campaign: own, sets: [base, { id: "empty", cards: [card("x")] }] },
    ).map((c) => c.id),
    ["Saturn"],
  );
  assert.deepEqual(
    groundCandidates({ set: "base" }, { campaign: own, sets: [] }).map(
      (c) => c.id,
    ),
    ["own"],
  );
  assert.deepEqual(
    groundCandidates({ set: "base" }, { campaign: own, sets: [base] }).map(
      (c) => c.here,
    ),
    [undefined],
    "a set's postcard is somewhere else: nothing to hide",
  );
  assert.deepEqual(
    groundCandidates(
      { postcard: card("Sent", PIC) as never },
      { campaign: [], sets: [] },
    ).map((c) => c.id),
    ["Sent"],
  );
  assert.equal(outsideGround(card("Bare")), null);
  assert.equal(
    outsideGround({ ...card("Bad"), stamp: "javascript:alert(1)" }),
    null,
    "only a picture",
  );
});

test("a bookmark sent by a preset carries its camera", () => {
  const bm = { ...makeBookmark([], at, "Mars", "Globe"), id: "bm1" };
  const p = {
    id: "k1",
    title: "Mars",
    to: "everyone" as const,
    steps: [{ show: { bookmark: "bm1" } }],
  };
  const sh = showOf(p as never, p.steps[0]!, [bm]);
  assert.ok(sh && "scope" in sh && sh.pose);
  assert.equal((sh.pose as { about: string }).about, "u:sol/mars");
  const c: HoldCard = "plain";
  assert.equal(holdKind(c), "plain");
});

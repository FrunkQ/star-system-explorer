// The Switchboard's board (track PB1): sends, Everyone as a layer, holds, Hold all, going back, screens and history.
import { test } from "node:test";
import assert from "node:assert/strict";
import { BoardHistory, back, emptyBoard, hold, holdAll, landing, lift, moveScreen, redirect, rejoin, release, releaseAll, resolve, screenHello, send, setSignal, signalOf, swapScreens, upgradeBoard, type Board } from "../src/board.ts";
import type { Shown } from "../src/seat.ts";

const sol: Shown = { scope: "u:k/sys:sol", title: "The Solar System" };
const mars: Shown = { scope: "u:k/sys:sol", title: "Mars, landing", screen: "players.land.screen.json", landing: { body: "u:k/sys:sol/body:4" } };
const tarn: Shown = { scope: "u:k/sys:tarn", title: "Tarn" };
const title = (b: Board, role: string, screen?: string) => {
  const r = resolve(b, role, screen);
  return r.held ? `held:${typeof r.held === "string" ? r.held : r.held.text}` : "off" in r.shown ? "off" : r.shown.title;
};

test("the first send to Everyone is the floor; once a role has its own line, Everyone is a layer that lifts", () => {
  let b = emptyBoard({ scope: "u:k/sys:kestrel", title: "Kestrel" });
  b = send(b, "everyone", sol);
  assert.equal(b.over.length, 0);
  assert.equal(title(b, "bridge"), "The Solar System");
  b = send(b, "role:telescope", mars);
  assert.equal(title(b, "telescope"), "Mars, landing");
  assert.equal(title(b, "bridge"), "The Solar System");
  b = send(b, "everyone", tarn);
  assert.equal(title(b, "telescope"), "Tarn", "a layer covers every role");
  assert.equal(title(b, "bridge"), "Tarn");
  b = lift(b);
  assert.equal(title(b, "telescope"), "Mars, landing", "Lift gives each role back what it had");
  assert.equal(title(b, "bridge"), "The Solar System");
});

test("a send to a role while a layer is on goes underneath, ready for when it lifts", () => {
  let b = send(send(emptyBoard(sol), "role:bridge", sol), "everyone", tarn);
  b = send(b, "role:bridge", mars);
  assert.equal(title(b, "bridge"), "Tarn");
  assert.equal(title(lift(b), "bridge"), "Mars, landing");
});

test("pressing again goes back one send; a line with nothing before it goes", () => {
  let b = send(emptyBoard(sol), "role:bridge", tarn);
  b = send(b, "role:bridge", mars);
  b = back(b, "role:bridge");
  assert.equal(title(b, "bridge"), "Tarn");
  b = back(b, "role:bridge");
  assert.equal(b.lines["role:bridge"], undefined);
  assert.equal(title(b, "bridge"), "The Solar System");
});

test("a hold shows only its card, never the source beneath; release gives back the latest", () => {
  let b = send(emptyBoard(sol), "role:bridge", tarn);
  b = hold(b, "role:bridge", "joke");
  const r = resolve(b, "bridge");
  assert.deepEqual(r, { shown: { off: true }, held: "joke" });
  b = send(b, "role:bridge", mars);
  assert.equal(title(b, "bridge"), "held:joke", "a held line takes the new source and stays held");
  b = release(b, "role:bridge");
  assert.equal(title(b, "bridge"), "Mars, landing");
});

test("holding a role with no line of its own, then releasing, leaves it following the table again", () => {
  let b = hold(emptyBoard(sol), "role:bridge", { text: "Stand by" });
  assert.equal(title(b, "bridge"), "held:Stand by");
  assert.equal(title(b, "telescope"), "The Solar System");
  b = release(b, "role:bridge");
  assert.equal(b.lines["role:bridge"], undefined);
  b = send(b, "everyone", tarn);
  assert.equal(title(b, "bridge"), "Tarn");
});

test("Hold all holds everything and Release all leaves earlier holds held", () => {
  let b = hold(send(emptyBoard(sol), "role:bridge", tarn), "role:telescope", "plain");
  b = holdAll(b, "blank");
  assert.equal(title(b, "bridge"), "held:blank");
  assert.equal(title(b, "telescope"), "held:blank");
  b = releaseAll(b);
  assert.equal(title(b, "bridge"), "Tarn");
  assert.equal(title(b, "telescope"), "held:plain");
});

test("screens: hello keeps the GM's role, its own line, back with its role, move and swap", () => {
  let b = screenHello(emptyBoard(sol), "k3f9", { name: "Mira's tablet", role: "bridge" });
  b = screenHello(b, "w1", { name: "Wall TV", role: "telescope" });
  b = moveScreen(b, "k3f9", "telescope");
  b = screenHello(b, "k3f9", { name: "Mira's tablet", role: "bridge" });
  assert.equal(b.screens.k3f9!.role, "telescope", "a returning screen keeps the role the GM gave it");
  b = send(b, "screen:k3f9", mars);
  assert.equal(title(b, "telescope", "k3f9"), "Mars, landing");
  assert.equal(title(b, "telescope", "w1"), "The Solar System");
  b = rejoin(b, "screen:k3f9");
  assert.equal(title(b, "telescope", "k3f9"), "The Solar System");
  b = swapScreens(b, "k3f9", "w1");
  assert.deepEqual([b.screens.k3f9!.role, b.screens.w1!.role], ["telescope", "telescope"]);
  b = moveScreen(b, "w1", "bridge");
  b = swapScreens(b, "k3f9", "w1");
  assert.deepEqual([b.screens.k3f9!.role, b.screens.w1!.role], ["bridge", "telescope"]);
});

test("a redirected role's sends land on the role it is on tonight, without loops", () => {
  let b = redirect(emptyBoard(sol), "telescope", "bridge");
  assert.equal(landing(b, "role:telescope"), "role:bridge");
  b = send(b, "role:telescope", mars);
  assert.equal(title(b, "bridge"), "Mars, landing");
  b = redirect(redirect(b, "bridge", "telescope"), "telescope", "bridge");
  assert.ok(["role:bridge", "role:telescope"].includes(landing(b, "role:telescope")));
  assert.deepEqual(redirect(b, "telescope", null).redirects.telescope, undefined);
});

test("signal: a screen's own dial wins over its role's; 0 removes it", () => {
  let b = setSignal(setSignal(emptyBoard(sol), "role:bridge", 0.5), "screen:k3f9", 0.1);
  assert.equal(signalOf(b, "bridge"), 0.5);
  assert.equal(signalOf(b, "bridge", "k3f9"), 0.1);
  b = setSignal(b, "screen:k3f9", 0);
  assert.equal(signalOf(b, "bridge", "k3f9"), 0.5);
  assert.equal(setSignal(b, "role:bridge", 7).signal["role:bridge"], 1);
});

test("an older share upgrades: off is a blank hold, a seat's own shown is its role's line", () => {
  const b = upgradeBoard({ off: true }, [{ id: "table" }, { id: "bridge", shown: tarn }]);
  assert.equal(title(b, "table"), "held:blank");
  assert.equal(b.lines["role:bridge"]!.source.title, "Tarn");
  const c = upgradeBoard(sol, [{ id: "bridge", shown: { off: true } }]);
  assert.equal(title(c, "bridge"), "held:blank");
  assert.equal(title(c, "table"), "The Solar System");
});

test("the board's own history: undo and redo step through tonight's changes; a new change drops redo", () => {
  const h = new BoardHistory(emptyBoard(sol));
  h.do(send(h.board, "everyone", tarn));
  h.do(hold(h.board, "role:bridge", "plain"));
  assert.equal(title(h.board, "bridge"), "held:plain");
  h.undo();
  assert.equal(title(h.board, "bridge"), "Tarn");
  h.undo();
  assert.equal(title(h.board, "bridge"), "The Solar System");
  assert.equal(h.canUndo, false);
  h.redo();
  assert.equal(title(h.board, "bridge"), "Tarn");
  h.do(send(h.board, "role:bridge", mars));
  assert.equal(h.canRedo, false);
});

test("boards are plain data: no operation changes the board it was given", () => {
  const b = send(emptyBoard(sol), "role:bridge", tarn);
  const before = JSON.stringify(b);
  for (const f of [(x: Board) => send(x, "everyone", mars), (x: Board) => hold(x, "role:bridge", "joke"), lift, (x: Board) => back(x, "role:bridge"), (x: Board) => holdAll(x, "blank"), (x: Board) => screenHello(x, "a", { name: "A", role: "bridge" })]) f(b);
  assert.equal(JSON.stringify(b), before);
  assert.doesNotThrow(() => structuredClone(b));
});

test("on the wire: a held role gets off with one card for all its screens; others get their place; screens their own", async () => {
  const { boardWire } = await import("../src/board.ts");
  let b = screenHello(emptyBoard(sol), "k3f9", { name: "Mira", role: "bridge" });
  b = screenHello(b, "w1", { name: "Wall", role: "telescope" });
  b = send(b, "screen:w1", mars);
  b = hold(b, "role:bridge", "joke");
  let n = 0;
  const w = boardWire(b, ["table", "bridge", "telescope"], () => `joke ${++n}`);
  assert.deepEqual(w.shown, sol);
  assert.deepEqual(w.roles.bridge, { off: true, card: "joke 1" });
  assert.deepEqual(w.screens.k3f9, { off: true, card: "joke 1" }, "the same words on every screen of the role");
  assert.deepEqual(w.roles.telescope, sol);
  assert.deepEqual(w.screens.w1, mars);
  assert.deepEqual(boardWire(holdAll(b, "blank"), ["table"]).roles.table, { off: true });
});

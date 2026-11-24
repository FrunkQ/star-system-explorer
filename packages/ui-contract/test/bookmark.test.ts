import { test } from "node:test";
import assert from "node:assert/strict";
import { addBookmark, bookmarkName, bookmarkReachable, bookmarksFrom, makeBookmark, moveBookmark, nextBookmarkId, removeBookmark, renameBookmark, updateBookmark, type Whereabouts } from "../src/bookmark.ts";

const at: Whereabouts = { scope: "u:sol", now_yr: 4.6e9, instrument: "globe", scale: "world", view: { style: "true", seen: "free", band: "infrared" }, selected: "u:sol/mars" };

test("a bookmark keeps where the GM is and how they look, and nothing about players", () => {
  const b = makeBookmark([], at, "Mars", "Globe");
  assert.equal(b.id, "b1");
  assert.equal(b.name, "Mars, Globe");
  assert.deepEqual({ scope: b.scope, at_yr: b.at_yr, instrument: b.instrument, scale: b.scale, selected: b.selected }, { scope: "u:sol", at_yr: 4.6e9, instrument: "globe", scale: "world", selected: "u:sol/mars" });
  assert.equal(b.view?.band, "infrared");
  assert.ok(!("role" in b) && !("look" in b));
});

test("names and ids do not repeat", () => {
  let list = addBookmark([], makeBookmark([], at, "Mars", "Globe"));
  list = addBookmark(list, makeBookmark(list, at, "Mars", "Globe"));
  assert.deepEqual(list.map((b) => [b.id, b.name]), [["b1", "Mars, Globe"], ["b2", "Mars, Globe 2"]]);
  assert.equal(nextBookmarkId(removeBookmark(list, "b1")), "b3");
  assert.equal(bookmarkName([], "Chart", "chart"), "Chart");
});

test("rename, reorder, update from live", () => {
  let list = [makeBookmark([], at, "Mars", "Globe")];
  list = addBookmark(list, makeBookmark(list, { ...at, selected: null, instrument: "system" }, "The Solar System", "Chart"));
  assert.equal(renameBookmark(list, "b1", "  The red planet ")[0]!.name, "The red planet");
  assert.equal(renameBookmark(list, "b1", "  ")[0]!.name, "Mars, Globe");
  assert.deepEqual(moveBookmark(list, 1, 0).map((b) => b.id), ["b2", "b1"]);
  const u = updateBookmark(list, "b1", { ...at, instrument: "cutaway", now_yr: 1 })[0]!;
  assert.deepEqual([u.id, u.name, u.instrument, u.at_yr], ["b1", "Mars, Globe", "cutaway", 1]);
});

test("read back from a save: only real bookmarks; another campaign's are unreachable", () => {
  const good = makeBookmark([], at, "Mars", "Globe");
  const list = bookmarksFrom([good, { schema: "sse.bookmark/1", id: "b1", name: "dupe", scope: "x", instrument: "y", at_yr: null }, { id: "b9" }, null, "x"]);
  assert.deepEqual(list, [good]);
  assert.deepEqual(bookmarksFrom(undefined), []);
  assert.ok(bookmarkReachable(good, "u:sol"));
  assert.ok(!bookmarkReachable(good, "u:tarn"));
});

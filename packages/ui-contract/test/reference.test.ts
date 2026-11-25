// References in a "why?" link out: a DOI to doi.org, a URL as itself, else plain text.
import { test } from "node:test";
import assert from "node:assert/strict";
import { showReference } from "../src/reference.ts";

test("a reference with a DOI links to doi.org, a URL links as itself, otherwise plain text", () => {
  assert.deepEqual(showReference({ text: "Dole 1970", doi: "10.1016/0019-1035(70)90095-3" }), { text: "Dole 1970", href: "https://doi.org/10.1016/0019-1035(70)90095-3" });
  assert.equal(showReference({ text: "x", doi: "https://doi.org/10.1086/191003" }).href, "https://doi.org/10.1086/191003");
  assert.equal(showReference({ text: "x", doi: "doi:10.1086/191003" }).href, "https://doi.org/10.1086/191003");
  assert.equal(showReference({ text: "NASA fact sheet", url: "https://nssdc.gsfc.nasa.gov/planetary/factsheet/" }).href, "https://nssdc.gsfc.nasa.gov/planetary/factsheet/");
  assert.equal(showReference({ text: "x", doi: "10.1086/191003", url: "https://example.org/a" }).href, "https://doi.org/10.1086/191003", "a DOI wins over a URL");
  assert.deepEqual(showReference("Kopparapu et al. 2013"), { text: "Kopparapu et al. 2013", href: null });
  assert.equal(showReference("Kopparapu et al. 2013, doi:10.1088/0004-637X/765/2/131.").href, "https://doi.org/10.1088/0004-637X/765/2/131");
  assert.equal(showReference("See https://exoplanetarchive.ipac.caltech.edu/.").href, "https://exoplanetarchive.ipac.caltech.edu/");
  assert.equal(showReference({ text: "x", url: "javascript:alert(1)" }).href, null, "only http(s) links");
  assert.equal(showReference({ text: "x", doi: "not a doi" }).href, null);
  assert.equal(showReference({ doi: "10.1086/191003" }).text, "10.1086/191003", "a bare DOI is its own text");
});

// Text out of a PDF, in the browser, with no library (ND4: Orac reads a
// paper). Enough of the format to read the papers a GM drops on Orac:
// objects and object streams, Flate streams, the page tree in order, each
// page's fonts (a ToUnicode map, else an encoding with Differences, else
// Latin-1), and the text operators Tj, TJ, ' and " with line breaks from the
// text position. It does not lay out columns or read pictures of text; when
// a file gives little text, the caller says so and offers pasting instead.
// Decompression is passed in (DecompressionStream in the browser, zlib in
// node), so nothing here needs a platform.

export type Inflate = (bytes: Uint8Array) => Promise<Uint8Array>;

/** The browser's own inflate (DecompressionStream "deflate", which reads the zlib wrapper PDF uses). */
export const browserInflate: Inflate = async (bytes) => {
  const ds = new DecompressionStream("deflate");
  const out = new Blob([bytes as BlobPart]).stream().pipeThrough(ds);
  return new Uint8Array(await new Response(out).arrayBuffer());
};

type Obj = { dict: string; stream?: Uint8Array };
type Val = number | string | Val[] | { name: string } | { ref: number } | { dict: Record<string, Val> } | { hex: string } | null;

const latin1 = (b: Uint8Array) => {
  let s = "";
  for (let i = 0; i < b.length; i += 0x8000) s += String.fromCharCode(...b.subarray(i, i + 0x8000));
  return s;
};

// ---------------------------------------------------------------- tokens

const WS = /[\0\t\n\f\r ]/;
const DELIM = /[()<>[\]{}/%]/;

class Lexer {
  i = 0;
  readonly s: string;
  constructor(s: string) {
    this.s = s;
  }
  skip() {
    for (;;) {
      while (this.i < this.s.length && WS.test(this.s[this.i]!)) this.i++;
      if (this.s[this.i] === "%") while (this.i < this.s.length && this.s[this.i] !== "\n" && this.s[this.i] !== "\r") this.i++;
      else return;
    }
  }
  /** One token: a value, or an operator as {op}. */
  next(): { v: Val } | { op: string } | null {
    this.skip();
    const s = this.s;
    if (this.i >= s.length) return null;
    const c = s[this.i]!;
    if (c === "(") return { v: this.literal() };
    if (c === "<" && s[this.i + 1] === "<") {
      this.i += 2;
      return { v: { dict: this.dictBody() } };
    }
    if (c === "<") {
      const j = s.indexOf(">", this.i);
      const hex = s.slice(this.i + 1, j < 0 ? s.length : j).replace(/[^0-9a-fA-F]/g, "");
      this.i = j < 0 ? s.length : j + 1;
      return { v: { hex: hex.length % 2 ? hex + "0" : hex } };
    }
    if (c === "[") {
      this.i++;
      const a: Val[] = [];
      for (;;) {
        this.skip();
        if (this.s[this.i] === "]" || this.i >= s.length) {
          this.i++;
          return { v: a };
        }
        const t = this.next();
        if (!t) return { v: a };
        if ("v" in t) a.push(t.v);
        else if (t.op === "R" && a.length >= 2) {
          a.pop();
          const n = a.pop();
          a.push({ ref: typeof n === "number" ? n : -1 });
        }
      }
    }
    if (c === "/") {
      let j = this.i + 1;
      while (j < s.length && !WS.test(s[j]!) && !DELIM.test(s[j]!)) j++;
      const name = s.slice(this.i + 1, j).replace(/#([0-9a-fA-F]{2})/g, (_, h: string) => String.fromCharCode(parseInt(h, 16)));
      this.i = j;
      return { v: { name } };
    }
    if (c === ">" || c === "]" || c === ")" || c === "{" || c === "}") {
      this.i++;
      return { op: c };
    }
    let j = this.i;
    while (j < s.length && !WS.test(s[j]!) && !DELIM.test(s[j]!)) j++;
    if (j === this.i) j++;
    const w = s.slice(this.i, j);
    this.i = j;
    if (/^[+-]?(\d+\.?\d*|\.\d+)$/.test(w)) return { v: Number(w) };
    if (w === "null") return { v: null };
    return { op: w };
  }
  literal(): string {
    const s = this.s;
    let depth = 0;
    let out = "";
    this.i++;
    while (this.i < s.length) {
      const c = s[this.i++]!;
      if (c === "\\") {
        const d = s[this.i++]!;
        const esc: Record<string, string> = { n: "\n", r: "\r", t: "\t", b: "\b", f: "\f" };
        if (d in esc) out += esc[d];
        else if (/[0-7]/.test(d)) {
          let o = d;
          while (o.length < 3 && /[0-7]/.test(s[this.i] ?? "")) o += s[this.i++];
          out += String.fromCharCode(parseInt(o, 8) & 0xff);
        } else if (d === "\r") {
          if (s[this.i] === "\n") this.i++;
        } else if (d !== "\n") out += d;
      } else if (c === "(") {
        depth++;
        out += c;
      } else if (c === ")") {
        if (depth-- === 0) return out;
        out += c;
      } else out += c;
    }
    return out;
  }
  dictBody(): Record<string, Val> {
    const d: Record<string, Val> = {};
    const items: Val[] = [];
    for (;;) {
      this.skip();
      if (this.s.startsWith(">>", this.i) || this.i >= this.s.length) {
        this.i += 2;
        break;
      }
      const t = this.next();
      if (!t) break;
      if ("v" in t) items.push(t.v);
      else if (t.op === "R" && items.length >= 2) {
        items.pop();
        const n = items.pop();
        items.push({ ref: typeof n === "number" ? n : -1 });
      }
    }
    for (let k = 0; k + 1 < items.length; k += 2) {
      const key = items[k];
      if (key && typeof key === "object" && "name" in key) d[key.name] = items[k + 1]!;
    }
    return d;
  }
}

const parseDict = (s: string): Record<string, Val> => {
  const L = new Lexer(s);
  const t = L.next();
  return t && "v" in t && t.v && typeof t.v === "object" && "dict" in t.v ? t.v.dict : {};
};
const nameOf = (v: Val | undefined) => (v && typeof v === "object" && "name" in v ? v.name : undefined);
const refOf = (v: Val | undefined) => (v && typeof v === "object" && "ref" in v ? v.ref : undefined);

// ---------------------------------------------------------------- the file

class Doc {
  objs = new Map<number, Obj>();
  readonly bytes: Uint8Array;
  readonly inflate: Inflate;
  constructor(bytes: Uint8Array, inflate: Inflate) {
    this.bytes = bytes;
    this.inflate = inflate;
  }

  async load() {
    const s = latin1(this.bytes);
    const re = /(\d+)\s+(\d+)\s+obj\b/g;
    let m: RegExpExecArray | null;
    while ((m = re.exec(s))) {
      const start = m.index + m[0].length;
      const end = s.indexOf("endobj", start);
      if (end < 0) break;
      const body = s.slice(start, end);
      const at = body.search(/\bstream(\r\n|\n|\r)/);
      if (at >= 0) {
        const dict = body.slice(0, at);
        const off = start + at + body.slice(at).match(/^stream(\r\n|\n|\r)/)![0].length;
        let stop = s.lastIndexOf("endstream", end);
        if (stop < off) stop = end;
        let len = stop - off;
        const d = parseDict(dict.trim());
        if (typeof d.Length === "number" && d.Length <= len) len = d.Length;
        this.objs.set(Number(m[1]), { dict, stream: this.bytes.subarray(off, off + len) });
      } else this.objs.set(Number(m[1]), { dict: body });
      re.lastIndex = end;
    }
    // Object streams (PDF 1.5): objects packed inside a compressed stream.
    for (const [, o] of [...this.objs]) {
      const d = parseDict(o.dict.trim());
      if (nameOf(d.Type) !== "ObjStm" || !o.stream) continue;
      const data = await this.decode(d, o.stream);
      if (!data) continue;
      const text = latin1(data);
      const first = typeof d.First === "number" ? d.First : 0;
      const head = text.slice(0, first).trim().split(/\s+/).map(Number);
      for (let k = 0; k + 1 < head.length; k += 2) {
        const num = head[k]!;
        const from = first + head[k + 1]!;
        const to = k + 3 < head.length ? first + head[k + 3]! : text.length;
        if (!this.objs.has(num)) this.objs.set(num, { dict: text.slice(from, to) });
      }
    }
  }

  async decode(d: Record<string, Val>, raw: Uint8Array): Promise<Uint8Array | null> {
    const f = d.Filter;
    const filters = Array.isArray(f) ? f.map(nameOf) : f === undefined ? [] : [nameOf(f)];
    let b = raw;
    for (const x of filters) {
      if (x === "FlateDecode") {
        try {
          b = await this.inflate(b);
        } catch {
          return null;
        }
      } else return null;
    }
    return b;
  }

  value(v: Val | undefined): Val | undefined {
    const r = refOf(v);
    if (r === undefined) return v;
    const o = this.objs.get(r);
    if (!o) return undefined;
    const L = new Lexer(o.dict);
    const t = L.next();
    return t && "v" in t ? t.v : undefined;
  }
  dict(v: Val | undefined): Record<string, Val> {
    const x = this.value(v);
    return x && typeof x === "object" && "dict" in x ? x.dict : {};
  }
  async streamOf(v: Val | undefined): Promise<Uint8Array | null> {
    const r = refOf(v);
    const o = r === undefined ? undefined : this.objs.get(r);
    if (!o?.stream) return null;
    return this.decode(parseDict(o.dict.trim()), o.stream);
  }

  /** The pages in reading order, from the catalogue's page tree (else every Page object by number). */
  pages(): Record<string, Val>[] {
    const out: Record<string, Val>[] = [];
    const root = [...this.objs.values()].map((o) => parseDict(o.dict.trim())).find((d) => nameOf(d.Type) === "Catalog");
    const walk = (v: Val | undefined, depth: number) => {
      if (depth > 32) return;
      const d = this.dict(v);
      if (nameOf(d.Type) === "Page") out.push(d);
      else if (Array.isArray(d.Kids)) for (const k of d.Kids) walk(k, depth + 1);
    };
    if (root?.Pages) walk(root.Pages, 0);
    if (!out.length) for (const o of this.objs.values()) {
      const d = parseDict(o.dict.trim());
      if (nameOf(d.Type) === "Page") out.push(d);
    }
    return out;
  }
}

// ---------------------------------------------------------------- fonts

/** A few glyph names past the ones that are their own letter (the Adobe glyph list's common text names). */
const GLYPHS: Record<string, string> = {
  space: " ", exclam: "!", quotedbl: '"', numbersign: "#", dollar: "$", percent: "%", ampersand: "&", quotesingle: "'", quoteright: "’", quoteleft: "‘", parenleft: "(", parenright: ")", asterisk: "*", plus: "+", comma: ",", hyphen: "-", period: ".", slash: "/",
  zero: "0", one: "1", two: "2", three: "3", four: "4", five: "5", six: "6", seven: "7", eight: "8", nine: "9", colon: ":", semicolon: ";", less: "<", equal: "=", greater: ">", question: "?", at: "@", bracketleft: "[", backslash: "\\", bracketright: "]", asciicircum: "^", underscore: "_", grave: "`", braceleft: "{", bar: "|", braceright: "}", asciitilde: "~",
  endash: "–", emdash: "—", quotedblleft: "“", quotedblright: "”", bullet: "•", ellipsis: "…", minus: "−", multiply: "×", degree: "°", plusminus: "±", fi: "fi", fl: "fl", ff: "ff", ffi: "ffi", ffl: "ffl", dotlessi: "ı",
  alpha: "α", beta: "β", gamma: "γ", delta: "δ", epsilon: "ε", lambda: "λ", mu: "μ", nu: "ν", pi: "π", rho: "ρ", sigma: "σ", tau: "τ", phi: "φ", chi: "χ", omega: "ω", Delta: "Δ", Omega: "Ω", odieresis: "ö", udieresis: "ü", adieresis: "ä", eacute: "é", angstrom: "Å", Aring: "Å",
};
const glyph = (n: string) => GLYPHS[n] ?? (/^[A-Za-z]$/.test(n) ? n : /^uni([0-9A-F]{4})$/.test(n) ? String.fromCharCode(parseInt(n.slice(3), 16)) : /^[A-Za-z]\.\w+$/.test(n) ? n[0]! : "");

/** Windows Latin-1 code points that differ from Latin-1 in 128 to 159. */
const WIN: Record<number, string> = { 0x91: "‘", 0x92: "’", 0x93: "“", 0x94: "”", 0x95: "•", 0x96: "–", 0x97: "—", 0x85: "…" };

interface Font {
  bytes: 1 | 2;
  map: Map<number, string>;
  /** Single-byte codes not in map fall back to this. */
  plain: (c: number) => string;
}

function parseCMap(text: string): { map: Map<number, string>; bytes: 1 | 2 } {
  const map = new Map<number, string>();
  const hex = (h: string) => parseInt(h, 16);
  const utf16 = (h: string) => {
    let s = "";
    for (let i = 0; i + 4 <= h.length; i += 4) s += String.fromCharCode(hex(h.slice(i, i + 4)));
    if (h.length === 2) s = String.fromCharCode(hex(h));
    return s;
  };
  let bytes: 1 | 2 = 1;
  const space = /begincodespacerange([\s\S]*?)endcodespacerange/.exec(text);
  if (space && /<[0-9a-fA-F]{4}>/.test(space[1]!)) bytes = 2;
  for (const block of text.matchAll(/beginbfchar([\s\S]*?)endbfchar/g))
    for (const m of block[1]!.matchAll(/<([0-9a-fA-F]+)>\s*<([0-9a-fA-F]*)>/g)) {
      map.set(hex(m[1]!), utf16(m[2]!));
      if (m[1]!.length === 4) bytes = 2;
    }
  for (const block of text.matchAll(/beginbfrange([\s\S]*?)endbfrange/g))
    for (const m of block[1]!.matchAll(/<([0-9a-fA-F]+)>\s*<([0-9a-fA-F]+)>\s*(<([0-9a-fA-F]*)>|\[([^\]]*)\])/g)) {
      const lo = hex(m[1]!);
      const hi = Math.min(hex(m[2]!), lo + 0xffff);
      if (m[1]!.length === 4) bytes = 2;
      if (m[4] !== undefined) {
        const base = m[4];
        const tail = base.length >= 4 ? hex(base.slice(-4)) : hex(base);
        const head = base.length >= 4 ? utf16(base.slice(0, -4)) : "";
        for (let c = lo; c <= hi; c++) map.set(c, head + String.fromCharCode(tail + (c - lo)));
      } else {
        const items = [...m[5]!.matchAll(/<([0-9a-fA-F]*)>/g)].map((x) => utf16(x[1]!));
        items.forEach((u, k) => map.set(lo + k, u));
      }
    }
  return { map, bytes };
}

async function fontOf(doc: Doc, v: Val | undefined): Promise<Font> {
  const d = doc.dict(v);
  const latin = (c: number) => WIN[c] ?? (c >= 32 ? String.fromCharCode(c) : "");
  if (d.ToUnicode) {
    const s = await doc.streamOf(d.ToUnicode);
    if (s) {
      const { map, bytes } = parseCMap(latin1(s));
      const sub = nameOf(d.Subtype) === "Type0" ? 2 : bytes;
      return { bytes: sub, map, plain: latin };
    }
  }
  if (nameOf(d.Subtype) === "Type0") return { bytes: 2, map: new Map(), plain: () => "" };
  const map = new Map<number, string>();
  const enc = doc.value(d.Encoding);
  const diffs = enc && typeof enc === "object" && "dict" in enc ? enc.dict.Differences : undefined;
  if (Array.isArray(diffs)) {
    let code = 0;
    for (const x of diffs) {
      if (typeof x === "number") code = x;
      else if (x && typeof x === "object" && "name" in x) map.set(code++, glyph(x.name));
    }
  }
  return { bytes: 1, map, plain: latin };
}

// ---------------------------------------------------------------- content

function decodeString(s: Val, f: Font | undefined): string {
  const codes: number[] = [];
  if (typeof s === "string") for (let i = 0; i < s.length; i++) codes.push(s.charCodeAt(i) & 0xff);
  else if (s && typeof s === "object" && "hex" in s) for (let i = 0; i + 2 <= s.hex.length; i += 2) codes.push(parseInt(s.hex.slice(i, i + 2), 16));
  const font = f ?? { bytes: 1 as const, map: new Map<number, string>(), plain: (c: number) => (c >= 32 ? String.fromCharCode(c) : "") };
  let out = "";
  if (font.bytes === 2) for (let i = 0; i + 1 < codes.length; i += 2) out += font.map.get((codes[i]! << 8) | codes[i + 1]!) ?? "";
  else for (const c of codes) out += font.map.get(c) ?? font.plain(c);
  return out;
}

async function pageText(doc: Doc, page: Record<string, Val>, inherited: Record<string, Val>): Promise<string> {
  const res = doc.dict(page.Resources ?? inherited.Resources);
  const fontDict = doc.dict(res.Font);
  const fonts = new Map<string, Font>();
  for (const [k, v] of Object.entries(fontDict)) fonts.set(k, await fontOf(doc, v));
  const contents = doc.value(page.Contents);
  const parts = Array.isArray(contents) ? contents : [page.Contents];
  const chunks: string[] = [];
  for (const c of parts) {
    const b = await doc.streamOf(c as Val);
    if (b) chunks.push(latin1(b));
  }
  const L = new Lexer(chunks.join("\n"));
  const stack: Val[] = [];
  let font: Font | undefined;
  let out = "";
  let lastY: number | null = null;
  const brk = () => {
    if (out && !out.endsWith("\n")) out += "\n";
  };
  // A new line when the text moves up or down; a space when it jumps along the same line (table cells, columns).
  const moveTo = (y: number, jumped: boolean) => {
    if (lastY !== null && Math.abs(y - lastY) > 1) brk();
    else if (jumped && out.length - runStart > 1 && !/\s$/.test(out)) out += " ";
    lastY = y;
    runStart = out.length;
  };
  // Where the text since the last move began: a jump after a single letter is letter spacing, not a gap.
  let runStart = 0;
  let y = 0;
  for (let guard = 0; guard < 5_000_000; guard++) {
    const t = L.next();
    if (!t) break;
    if ("v" in t) {
      stack.push(t.v);
      continue;
    }
    const op = t.op;
    const num = (k: number) => (typeof stack[stack.length - k] === "number" ? (stack[stack.length - k] as number) : 0);
    if (op === "BI") {
      const e = L.s.indexOf("EI", L.i);
      L.i = e < 0 ? L.s.length : e + 2;
    } else if (op === "Tf") {
      const n = nameOf(stack[stack.length - 2]);
      font = n ? fonts.get(n) : undefined;
    } else if (op === "Td" || op === "TD") {
      y += num(1);
      moveTo(y, num(2) !== 0);
    } else if (op === "Tm") {
      y = num(1);
      moveTo(y, true);
    } else if (op === "T*") brk();
    else if (op === "Tj") out += decodeString(stack[stack.length - 1] ?? "", font);
    else if (op === "'" || op === '"') {
      brk();
      out += decodeString(stack[stack.length - 1] ?? "", font);
    } else if (op === "TJ") {
      const a = stack[stack.length - 1];
      if (Array.isArray(a))
        for (const x of a) {
          if (typeof x === "number") {
            if (x < -200 && !out.endsWith(" ")) out += " ";
          } else out += decodeString(x, font);
        }
    } else if (op === "ET") lastY = lastY ?? y;
    stack.length = 0;
  }
  return out;
}

/** The text of a PDF, page by page. Refuses a file that is not a PDF or holds no readable text. */
export async function pdfText(bytes: Uint8Array, inflate: Inflate): Promise<{ ok: true; text: string; pages: number } | { ok: false; message: string }> {
  if (latin1(bytes.subarray(0, 1024)).indexOf("%PDF-") < 0) return { ok: false, message: "That file is not a PDF." };
  if (/\/Encrypt\b/.test(latin1(bytes.subarray(Math.max(0, bytes.length - 4096))))) return { ok: false, message: "That PDF is locked (encrypted), so its text cannot be read here. Paste the text instead." };
  const doc = new Doc(bytes, inflate);
  await doc.load();
  const pages = doc.pages();
  const texts: string[] = [];
  for (const p of pages) texts.push(await pageText(doc, p, doc.dict(p.Parent)));
  const text = texts
    .map((t) => t.replace(/[ \t]+\n/g, "\n").replace(/\n{3,}/g, "\n\n").trim())
    .filter(Boolean)
    .join("\n\n");
  const words = text.split(/\s+/).filter((w) => /[A-Za-z]{2,}/.test(w)).length;
  if (words < 20 * Math.max(1, Math.min(pages.length, 5)) && words < 200) return { ok: false, message: `Only ${words} words could be read from this PDF's ${pages.length} ${pages.length === 1 ? "page" : "pages"} (it may be scanned pictures of pages). Paste the text instead.` };
  return { ok: true, text, pages: pages.length };
}

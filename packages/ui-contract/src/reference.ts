// A block reference as a "why?" panel shows it (H6.5). The kernel sends each
// reference as text, or (once the physics track adds them) as an object with
// an optional DOI or URL. A DOI becomes a https://doi.org link, a URL a link,
// and anything else stays plain text. Only http(s) links are ever made.
export type ReferenceIn = string | { text?: string; citation?: string; label?: string; title?: string; doi?: string | null; url?: string | null };

export interface ShownReference {
  text: string;
  /** The link to open, or null for plain text. */
  href: string | null;
}

const DOI = /\b(10\.\d{4,9}\/[^\s"<>]+[^\s"<>.,;:)\]])/;

function doiHref(doi: string): string | null {
  const m = DOI.exec(doi.trim().replace(/^(https?:\/\/(dx\.)?doi\.org\/|doi:\s*)/i, ""));
  return m ? `https://doi.org/${m[1]}` : null;
}

function urlHref(url: string): string | null {
  try {
    const u = new URL(url.trim());
    return u.protocol === "https:" || u.protocol === "http:" ? u.href : null;
  } catch {
    return null;
  }
}

export function showReference(r: ReferenceIn): ShownReference {
  if (typeof r === "string") {
    const url = /https?:\/\/[^\s"<>]+[^\s"<>.,;:)\]]/.exec(r);
    if (url && !/doi\.org\//i.test(url[0])) return { text: r, href: urlHref(url[0]) };
    const d = /doi[:.\s]|doi\.org\//i.test(r) ? doiHref(r.slice(r.search(/doi/i))) : null;
    return { text: r, href: d };
  }
  const text = r.text ?? r.citation ?? r.label ?? r.title ?? r.doi ?? r.url ?? "";
  const href = (r.doi ? doiHref(r.doi) : null) ?? (r.url ? urlHref(r.url) : null);
  return { text, href };
}

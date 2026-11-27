// A star's formal class in words (the kernel's read, sse_formation::spectral, published on every star as
// star.spectral_class and star.class_family; describe_star's now.class). The class is the kernel's; this only says
// its family in words, so "G2 V" reads "G2 V main-sequence star" and "DX 4.5" reads "DX 4.5 white dwarf".

/** The atlas star scheme's families (packs/base-atlas, scheme "star"), in words. */
export const STAR_FAMILY_WORDS: Readonly<Record<string, string>> = {
  "pre-main-sequence": "pre-main-sequence star",
  "main-sequence": "main-sequence star",
  subgiant: "subgiant",
  giant: "giant",
  "bright-giant": "bright giant",
  supergiant: "supergiant",
  "brown-dwarf": "brown dwarf",
  "white-dwarf": "white dwarf",
  "neutron-star": "neutron star",
  "black-hole": "black hole",
};

/** "G2 V main-sequence star", "T5 brown dwarf", "NS neutron star"; the code alone for a family it does not know. */
export function starClassWords(code: string, family?: string): string {
  const w = family ? STAR_FAMILY_WORDS[family] : undefined;
  return w ? `${code} ${w}` : code;
}

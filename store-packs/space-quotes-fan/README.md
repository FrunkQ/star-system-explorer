# Space quotes from films, songs, games and books (unofficial fan pack)

Fifty-one short lines from films, songs, games, fiction and humour about space, each credited to who said it, for
the quote cards on held screens (postcards design, package PC7). They come from SSE3's `space-quotes.txt`. Alex
ruled on 25 Nov 2026 (postcards design, question 7) that real scientists' and astronauts' words ship in the base
app (`ui-contract/src/atlas/notes/space.quotes.json`) and these go in a free, unofficial pack, published on
Explorers and never built into SSE4.

`pack.json` carries one `sse.margin-notes/1` set (`fan.quotes.space`) under `provides.other.atlas.note_sets`,
read by `atlasDocsOf`. Every line has `uses: ["waiting"]` and `by`; `source` where it is known.

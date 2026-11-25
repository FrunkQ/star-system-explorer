# X3l: sector routes as links

The routes in a sector's metadata file (trade, courier, jump lanes) become the travel thread's links (`set_link`,
PR 377), so `plan_voyage` can use the ones a crew knows.

## What changed

- **`sse-sector-io` reads routes.** `SectorFile.routes` holds each `<Routes><Route Start End StartOffsetX/Y
  EndOffsetX/Y Type Allegiance Style Color/>` of the metadata XML. A route with a broken hex is left out with a warning;
  a bad offset reads 0 with a warning. `write_sector_xml` writes them back, `with_metadata` merges them, and
  `subsector(letter)` keeps the routes with both ends inside it. Fixture: `tests/fixtures/sector/kestrel-reach.xml`
  (invented, not official data) gains five routes.
- **`import_sector` takes `metadata`** (the XML text), and `route_audience` and `route_tier` for who knows the routes.
  - The preview lists every route with its ends' systems, or why it was left out: it reaches a neighbouring sector, it
    starts and ends on the same hex, or no imported world is at one end.
  - With `apply`, each kept route is one link event: id `route-<start>-<end>-<type>` (so a second import rewrites the
    same link), kind the route's type, name "Trade route 0912 to 1013", source `import_sector`, and a reason naming the
    file and allegiance. No `nav` rule, so it is a charted lane.
  - The audience defaults to the GM only, as `set_link` does: the GM tells the players, by importing again with
    `route_audience: "everyone"` or through the record.
  - The answer gains `routes` (rows with `a`, `b`, `link`, `event_id` or `left_out`) and `record_hash`, both left out
    when the file has no routes, so older answers and the golden are unchanged.
- **The Sector map's import** fetches the metadata with the sector (Worker `/tmap/metadata`, already there; a sector
  without one imports without routes), or takes a chosen XML file. The preview says how many routes become links and
  lists the ones left out; a tick says whether the players know them. `ui-contract` gains `fetchMetadata`,
  `metadataPath` and `SectorRouteRow`.

## Checks

- `sse-sector-io` `routes_read_with_their_offsets_and_words` and the extended `xml_metadata_and_merge` (round trip).
- `universe_pack_tests::sector_routes_become_links_known_to_the_gm`: preview writes nothing; two links on apply, known
  to the GM and not to the players; a cross-sector route and a route to an empty hex left out in words; importing
  again for everyone keeps two links, now known to the players; a bad audience is refused.
- `tests/ui/worker.test.ts`: the metadata path, and a missing or unreachable metadata file reads as none.

Golden unchanged.

## Next

- Routes into neighbouring sectors once a universe holds more than one sector (systems are keyed by hex today).
- Drawing links on the hex map (the route's style and colour are kept for that).

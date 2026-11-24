# MP26: solid propellants and RP-1's heating value

The real spaceflight thread (track RS) asked for these on 24 Nov 2026. The real engines in `store-packs/sol-today/parts.json` (PR 363) name their propellants by sheet ids. The script is `scripts/solid_rows.py`.

- **`pban-ap-al`: the Shuttle SRB propellant, TP-H1148.**
  - Make-up by mass: 69.60 % ammonium perchlorate, 16.00 % aluminium, 14.00 % PBAN binder with an epoxy cure, and 0.40 % iron oxide.
  - Density 1771.4 kg/m3. Theoretical flame temperature 3471 K at 1000 psia. Theoretical Isp 276.7 s in vacuum at expansion ratio 7.16, and 262.3 s at sea level.
  - Source: JPL Publication 79-29.
- **`htpb-ap-al`: the HTPB composite of the STAR kick motors.**
  - 18 % aluminium by mass. The source does not print the ammonium perchlorate share.
  - Theoretical bulk density 1813 kg/m3, and Isp 294.2 s at expansion ratio 54.8.
  - Source: Kibbey, NASA MSFC (STAR-48BV).
- **`rp-1` now has `heating_value_net`:** 43.302 MJ/kg. This is the kerosine mean from NBS TN 937, not a measurement of RP-1 itself, and the note says so. NISTIR 6646 gives no heat of combustion in the parts that could be read.
- **Make-up shape.** The ingredients are not sheet substances yet, so each solid has an empty recipe. Its make-up is a `make_up` map of plain names to mass fractions, and the builder checks that the fractions sum to no more than 1.
- **Still missing:**
  - a heat of combustion for either solid
  - Star 37 (TP-H-3340) and Titan UA1205 data
  - the ammonium perchlorate share in HTPB
- **Checks.** All values are unverified until checked against the page images. Sources and quotes are in the `SOURCES` of `solid_rows.py`.

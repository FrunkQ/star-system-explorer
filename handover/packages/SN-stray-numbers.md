# SN: stray physics numbers moved to where the rule says they live

Thread "Move stray physics numbers to the sheet" (opened 26 Nov 2026 by the coordinator after the audit
"Check today's physics is in blocks"). Alex's rule: all physics in editable, cited blocks; all material and colour
data on the master data sheet, read through Look up; shortcuts in `docs/fudges.md`. One kind of work: moving typed-in
numbers to where they belong. Values are kept as they were unless a part below says otherwise.

## SN1: lifetimes in sunlight (water, CO2, CO)

- Sheet: `substances.<id>.photochemistry.lifetime_1au` on h2o (8.3e4 s), co2 (5.0e5 s) and co (1.3e6 s), quiet Sun
  at 1 au, growing as the distance squared; source `huebner1992`, every value **unverified** (recalled survey values,
  as both readers had them; the paper's tables not read, Consensus closed until 1 Dec 2026). Built by
  `scripts/photo_rows.py`; the compiled copy is `sse_core::sheet::photo_lifetime_1au`.
- Readers: the ice ground block `sse.surface.open_ice_loss` loses its `photo_rate_1au` parameter (1.2e-5 per second
  typed in the block and in `sse-atoms`) and takes the lifetime as an input, kind `material.photo_lifetime_1au`
  (Look up); `sse-surface` passes the sheet's water row. `sse-small` fills each comet volatile's lifetime from the
  sheet at load, and `packs/base-small-bodies` no longer types them.
- **Value change (rounding):** the ice ground's rate was 1.2e-5 per second; it is now 1 / 8.3e4 s = 1.205e-5 per
  second (+0.4 %). Both were the same two-figure fact; the sheet keeps one, the lifetime as the comet pack had it.
- **Golden:** one core contract line moves (`system_resolved` for Kestrel sys 40): four ice grounds' `lost_km` grow
  by 0.4 % and the `why` text's metres with them. Nothing else changes.

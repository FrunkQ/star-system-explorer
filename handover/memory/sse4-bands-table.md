---
name: sse4-bands-table
description: One Bands table on the master sheet (RK10, PR 383, 24 Nov 2026); band switch and recipes must read band ids and ranges from it
metadata:
  type: project
  modified: 2026-11-24T08:38:35.207Z
---
RK10 (PR 383, merged 24 Nov 2026) made the master sheet's Bands table the only source of bands. In ui-contract it is `SHEET_BANDS` (type `SheetBand`); in sse-reference it is the "bands" table. The view rows view-visible, view-infrared, view-ultraviolet, view-xray, view-radio and view-mag each carry a `view_id` matching the band switch. The light rows, gamma to radio, each name their view.

**How to apply:** when touching band recipes or the band switch (renderer track), read band ids and ranges from this table instead of typing strings. The old copy in ui-contract `light.ts` is stale: its radio band stops at 10 m, while the sheet runs to 1e4 m.

Related: [[sse4-atomic-rockets]]

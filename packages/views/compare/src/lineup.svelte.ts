import type { CompareItem } from "../../../ui-contract/src/index.ts";

/**
 * The lineup lives for the page's life, so it survives moving between
 * screens. Per system: each system keeps its own lineup, started from its
 * own worlds the first time it is compared.
 */
export const lineups: Record<string, CompareItem[]> = $state({});

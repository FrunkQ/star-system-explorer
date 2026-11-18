// The view modules this shell can mount. Adding a view is adding it here; the
// kernel is never touched (H8.1).
import type { ViewModule } from "../../ui-contract/src/index.ts";
import { systemChart } from "../../views/system-chart/src/index.ts";
import { bodyCard } from "../../views/body-card/src/index.ts";
import { timeRuler } from "../../views/time-ruler/src/index.ts";
import { flowEditor } from "../../views/flow-editor/src/index.ts";
import { changeRequests } from "../../views/change-requests/src/index.ts";

export const REGISTRY: readonly ViewModule[] = [systemChart, bodyCard, timeRuler, flowEditor, changeRequests];

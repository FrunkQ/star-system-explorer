// The view modules this shell can mount. Adding a view is adding it here; the
// kernel is never touched (H8.1).
import type { ViewModule } from "../../ui-contract/src/index.ts";
import { systemChart } from "../../views/system-chart/src/index.ts";
import { bodyCard } from "../../views/body-card/src/index.ts";
import { cutaway } from "../../views/cutaway/src/index.ts";
import { timeRuler } from "../../views/time-ruler/src/index.ts";
import { flowEditor } from "../../views/flow-editor/src/index.ts";
import { changeRequests } from "../../views/change-requests/src/index.ts";
import { packStack } from "../../views/pack-stack/src/index.ts";
import { screenComposer } from "../../views/screen-composer/src/index.ts";
import { dataViews } from "../../views/data-views/src/index.ts";
import { symbols } from "../../views/symbols/src/index.ts";
import { styles } from "../../views/styles/src/index.ts";

export const REGISTRY: readonly ViewModule[] = [systemChart, bodyCard, cutaway, timeRuler, flowEditor, changeRequests, packStack, screenComposer, dataViews, symbols, styles];

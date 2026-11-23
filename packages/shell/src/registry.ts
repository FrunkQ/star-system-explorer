// The view modules this shell can mount. Adding a view is adding it here; the
// kernel is never touched (H8.1).
import type { ViewModule } from "../../ui-contract/src/index.ts";
import { systemChart } from "../../views/system-chart/src/index.ts";
import { bodyCard } from "../../views/body-card/src/index.ts";
import { cutaway } from "../../views/cutaway/src/index.ts";
import { body } from "../../views/body/src/index.ts";
import { compare } from "../../views/compare/src/index.ts";
import { biosphere } from "../../views/biosphere/src/index.ts";
import { timeRuler } from "../../views/time-ruler/src/index.ts";
import { flowEditor } from "../../views/flow-editor/src/index.ts";
import { changeRequests } from "../../views/change-requests/src/index.ts";
import { packStack } from "../../views/pack-stack/src/index.ts";
import { screenComposer } from "../../views/screen-composer/src/index.ts";
import { dataViews } from "../../views/data-views/src/index.ts";
import { symbols } from "../../views/symbols/src/index.ts";
import { styles } from "../../views/styles/src/index.ts";
import { reference } from "../../views/reference/src/index.ts";
import { MORE_VIEWS } from "./screens.more.ts";
import { CODES_VIEWS } from "./screens.codes.ts";

export const REGISTRY: readonly ViewModule[] = [systemChart, bodyCard, cutaway, body, compare, biosphere, timeRuler, flowEditor, changeRequests, packStack, screenComposer, dataViews, symbols, styles, reference, ...MORE_VIEWS, ...CODES_VIEWS];

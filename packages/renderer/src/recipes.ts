// The recipes this release ships (R1). They are plain documents
// (sse.render-recipe/1) in ./recipes/, checked against the pass library by
// test/recipe.test.ts. Bundled by Vite as JSON.
import surveyRelief from "./recipes/survey-relief.json";
import surveyContours from "./recipes/survey-contours.json";
import trueColour from "./recipes/true-colour.json";
import systemTrue from "./recipes/system-true.json";
import landTrue from "./recipes/land-true.json";
import type { RecipeDoc } from "./recipe.ts";

export const SHIPPED_RECIPES: readonly RecipeDoc[] = [surveyRelief as RecipeDoc, surveyContours as RecipeDoc, trueColour as RecipeDoc];

/** The standard recipe for True views of a system (VP4): placement and the bench predict with it. Not a globe recipe, so not in the Body's list. */
export const SYSTEM_TRUE = systemTrue as RecipeDoc;

/** The recipe for Land in True (VP4 part 2b): the finish and the drawing size; the ground's detail is the ladder's. */
export const LAND_TRUE = landTrue as RecipeDoc;

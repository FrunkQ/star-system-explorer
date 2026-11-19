// The recipes this release ships (R1). They are plain documents
// (sse.render-recipe/1) in ./recipes/, checked against the pass library by
// test/recipe.test.ts. Bundled by Vite as JSON.
import surveyRelief from "./recipes/survey-relief.json";
import surveyContours from "./recipes/survey-contours.json";
import trueColour from "./recipes/true-colour.json";
import type { RecipeDoc } from "./recipe.ts";

export const SHIPPED_RECIPES: readonly RecipeDoc[] = [surveyRelief as RecipeDoc, surveyContours as RecipeDoc, trueColour as RecipeDoc];

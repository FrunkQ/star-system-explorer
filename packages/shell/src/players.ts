import "@fontsource/newsreader/400.css";
import "@fontsource/newsreader/400-italic.css";
import "@fontsource/newsreader/600.css";
import "@fontsource/ibm-plex-sans/400.css";
import "@fontsource/ibm-plex-sans/500.css";
import "@fontsource/ibm-plex-sans/600.css";
import "@fontsource/ibm-plex-mono/400.css";
import "@fontsource/ibm-plex-mono/500.css";
import { mount } from "svelte";
import Players from "./Players.svelte";
import SplitScreen from "./SplitScreen.svelte";
import { tactileDialogs } from "../../chart-grammar/src/motion.ts";

// A split display (PB9) is a frame of players' screens, one per part; any other link is one screen.
const split = new URLSearchParams(location.search).get("split");
mount(split === "two" || split === "three" || split === "four" ? SplitScreen : Players, { target: document.getElementById("app")! });
tactileDialogs();

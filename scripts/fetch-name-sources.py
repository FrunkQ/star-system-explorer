#!/usr/bin/env python3
"""Fetch the real word lists behind the Many nations scheme (NM3c).

Writes packs/base-names/sources/many-nations.tsv: one row per word, with the
language, the concept it names, the word in its own script, its IPA and the
dataset it came from. scripts/build-base-names.py reads that file (no network)
and romanises each word by the rules written there.

Both datasets are CC BY 4.0, pinned to a release so the file can be rebuilt
exactly:

- NorthEuraLex 0.9 (Dellert et al. 2020, Lang Resources & Evaluation 54, 273,
  doi:10.1007/s10579-019-09480-6), CLDF lexibank/northeuralex v4.1: English,
  Spanish, French, Portuguese, Russian, Hindi, Standard Arabic.
- WOLD (Haspelmath and Tadmor, eds., 2009, World Loanword Database,
  https://wold.clld.org), CLDF lexibank/wold v4.2: Swahili (Schadeberg),
  Japanese (Schmidt), Mandarin Chinese (Wiebusch and Tadmor).

  scripts/fetch-name-sources.py   fetch and write the file (needs the network)

Only the chosen concepts are kept (nature, sky, land, water, weather, plants,
animals, colours, seasons and a few warm abstract words), never body parts,
religion, war or people. Rows named in DROP are left out, with the reason.
"""
import csv
import io
import os
import re
import urllib.request

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
OUT = os.path.join(ROOT, "packs", "base-names", "sources", "many-nations.tsv")
NEL = "https://raw.githubusercontent.com/lexibank/northeuralex/v4.1/cldf/"
WOLD = "https://raw.githubusercontent.com/lexibank/wold/v4.2/cldf/"

# The concepts, by NorthEuraLex id.
CONCEPTS = """56_sky 57_sun 58_moon 59_star 60_air 61_wind 62_wave 63_water 64_stone 65_ground 66_earth 69_spark
70_fire 71_light 72_shadow 74_fog 75_cloud 76_rain 77_snow 78_ice 79_frost 81_heat 82_hoarfrost 83_rainbow
84_thunder 85_current 86_drop 87_foam 89_lake 90_swamp 91_moor 92_meadow 93_forest 94_hill 96_mountain 97_summit
98_cave 99_slope 100_source 101_brook 102_river 103_shore 104_coast 105_land 106_sea 107_covebay 108_island
109_flower 110_grass 111_root 112_tree 117_leaf 118_birch 119_pine 120_willow 121_fir 123_feather 125_wing
130_nest 136_horse 141_bear 144_fox 145_hare 147_wolf 148_bird 153_eagle 155_owl 156_crane 157_crow 158_cuckoo
159_swan 160_fish 170_butterfly 171_berry 172_apple 175_seed 176_grain 185_iron 186_gold 187_silver 189_glass
190_clay 191_sand 212_joy 214_happiness 55_dream 220_memory 224_truth 227_story 241_song 242_calm 247_gift
251_friend 259_circle 402_path 399_bridge 400_well 401_pasture 295_north 296_south 297_west 298_east 451_morning
452_noon 453_evening 454_night 458_spring 459_summer 460_autumn 461_winter 457_year 450_day 449_time 522_bright
525_white 526_red 527_yellow 528_blue 529_green 530_grey 501_beautiful 503_warm 504_cold 506_fresh 564_new 575_far
579_first 429_world 308_home 411_life 239_sound 266_space""".split()

NEL_LANGS = {"eng": "en", "spa": "es", "fra": "fr", "por": "pt", "rus": "ru", "hin": "hi", "arb": "ar"}
WOLD_LANGS = {"Swahili": "sw", "Japanese": "ja", "MandarinChinese": "zh"}
# WOLD meanings whose Concepticon gloss differs from NorthEuraLex's, by WOLD id.
WOLD_EXTRA = {
    "1-22": "96_mountain", "1-36": "102_river", "1-99907": "102_river", "1-44": "64_stone", "1-27": "103_shore",
    "1-37": "100_source", "1-9991": "100_source", "9-99925": "400_well", "8-99905": "172_apple",
    "8-99957": "120_willow", "11-99903": "247_gift", "15-99902": "530_grey", "1-322": "242_calm",
    "3-71": "147_wolf", "3-74": "144_fox",
}

# Rows left out on review: (language, concept, word) and why.
DROP = {
    ("hi", "61_wind", "पाद"): "means flatulence, not weather",
    ("ar", "110_grass", "حشيش"): "also means hashish",
    ("ar", "118_birch", "قضبان"): "means rods or bars, not birch",
    ("ar", "121_fir", "تبوب"): "not found as fir in a dictionary we could check",
    ("hi", "121_fir", "फ़र"): "English loan",
    ("hi", "171_berry", "बेरी"): "English loan",
    ("hi", "156_crane", "क्रौंंच"): "misspelt in the source (a doubled nasal mark)",
    ("hi", "147_wolf", "वृक"): "rare Sanskrit form",
    ("hi", "308_home", "गृह"): "rare Sanskrit form",
    ("es", "212_joy", "placer"): "means pleasure",
    ("fr", "212_joy", "plaisir"): "means pleasure",
    ("pt", "212_joy", "prazer"): "means pleasure",
    ("ja", "451_morning", "午前中"): "a phrase (during the morning)",
    ("ja", "579_first", "一番目"): "a counter, not a word",
    ("ja", "579_first", "初めて"): "an adverb (for the first time)",
    ("ja", "457_year", "年間"): "a span (a year's period)",
    ("ja", "411_life", "生活"): "daily living",
    ("ja", "411_life", "人生"): "a person's lifetime",
    ("ja", "189_glass", "ガラス"): "a loan written in katakana",
    ("ja", "241_song", "曲"): "a piece of music",
    # Caught by the avoid list: an English swear word inside the romanised
    # form, or a brand.
    ("ar", "461_winter", "شتاء"): "romanised, holds an English swear word",
    ("ja", "90_swamp", "湿地"): "romanised, holds an English swear word",
    ("zh", "64_stone", "石头"): "romanised, holds an English swear word",
    ("en", "172_apple", "apple"): "a brand",
}
# Eggs are left out everywhere: crude slang in Russian, Spanish and Arabic.


def get(url):
    with urllib.request.urlopen(urllib.request.Request(url, headers={"User-Agent": "sse-names-builder"})) as r:
        return list(csv.DictReader(io.StringIO(r.read().decode("utf-8"))))


def clean(v):
    # WOLD numbers its senses ("umi (1)"); the first of several spellings
    # ("shwari, shuwari", "川、河", "温かい・暖かい") stands; Mandarin's erhua
    # mark "(r)" goes, and a spaced pinyin compound closes up.
    v = re.sub(r"\s*\(\d+\)\s*$", "", v.strip())
    v = re.split(r"[,、・/]", v)[0].strip()
    return v.replace("(r)", "")


def one_word(native, romanised):
    """A single word: no phrase, note, hyphen or bare stem ("-pya")."""
    for w in (native, romanised):
        if re.search(r"[\s_()\-’']", w):
            return False
    return bool(native)


def main():
    rows = []
    nel_par = {r["ID"]: r for r in get(NEL + "parameters.csv")}
    gloss_to_id = {nel_par[c]["Concepticon_Gloss"]: c for c in CONCEPTS}
    for r in get(NEL + "forms.csv"):
        lang = NEL_LANGS.get(r["Language_ID"])
        if lang and r["Parameter_ID"] in CONCEPTS:
            rows.append((lang, r["Parameter_ID"], clean(r["Value"]), r["Form"].strip(), "northeuralex"))
    wold_par = {r["ID"]: r for r in get(WOLD + "parameters.csv")}
    for r in get(WOLD + "forms.csv"):
        lang = WOLD_LANGS.get(r["Language_ID"])
        if not lang:
            continue
        p = r["Parameter_ID"]
        concept = WOLD_EXTRA.get(p) or gloss_to_id.get(wold_par[p]["Concepticon_Gloss"])
        if not concept:
            continue
        word = clean(r["Value"])
        script = r["original_script"].strip()
        if lang in ("ja", "zh"):
            # The characters, before any comma (Mandarin gives simplified, then traditional) or note.
            native = re.split(r"[,、・/(\s]", script, maxsplit=1)[0] if script else ""
        else:
            native = word
        if lang == "zh":
            word = word.replace(" ", "")
        ipa = r["Segments"].replace(" + ", " ").replace(" ", "")
        rows.append((lang, concept, native + "\t" + word if lang in ("ja", "zh") else native, ipa, "wold"))
    out = ["language\tconcept\tgloss\tnative\tromanised\tipa\tsource"]
    seen = set()
    for lang, concept, word, ipa, src in rows:
        native, _, romanised = word.partition("\t")
        if not one_word(native, romanised) or (lang, concept, native) in DROP or (lang, concept, native) in seen:
            continue
        seen.add((lang, concept, native))
        gloss = nel_par[concept]["Name"]
        out.append("\t".join([lang, concept, gloss, native, romanised, ipa, src]))
    os.makedirs(os.path.dirname(OUT), exist_ok=True)
    with open(OUT, "w", encoding="utf-8") as f:
        f.write("\n".join(out) + "\n")
    print("fetch-name-sources: wrote %d rows to %s" % (len(out) - 1, os.path.relpath(OUT, ROOT)))


if __name__ == "__main__":
    main()

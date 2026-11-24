"""The Bands table on the master data sheet (track RK, package RK10; Atomic
Rockets plan section 12, Alex's question 14): every band any part of SSE4
names, in one place. The light table, the view switch, every screen's list,
every sensor and every recipe refer to a band by its id here, and nothing
spells a band any other way. A pack adds a band by adding a row.

Three kinds of row:
  view     what the view switch offers (Visible, Infrared, ..., Mag)
  light    a wavelength range, end to end with its neighbours, with the view
           it is seen in and its default recipe where one exists
  channel  something that is not light (the magnetic field), with its view

The edges are conventions, not measurements (marked authored), copied
unchanged from the light design's fixed band set (packs/base-light), which
the builder checks against until the light thread reads this table.
"""

SOURCES = {
    "sse-light-bands": ("SSE4 nebulae and light design, the fixed band set (handover/design/SSE4-nebulae-and-light-design.md): band edges are conventions chosen for the renderer, not measurements; the ultraviolet and visible edges follow common astronomical usage", None, None, "authored"),
}

VIEWS = [
    ("visible", "Visible", "visible light"),
    ("infrared", "Infrared", "heat light"),
    ("ultraviolet", "Ultraviolet", "ultraviolet"),
    ("xray", "X-ray", "X-rays"),
    ("radio", "Radio", "radio"),
    ("mag", "Mag", "magnetic field"),
]

# id, physics name, plain name, from_m, to_m, view, default recipe or None.
LIGHT = [
    ("gamma", "Gamma rays", "gamma rays", 1e-14, 1e-11, "xray", None),
    ("xray", "X-rays", "X-rays", 1e-11, 1e-8, "xray", "xray"),
    ("uv", "Ultraviolet", "ultraviolet", 1e-8, 3.8e-7, "ultraviolet", "uv"),
    ("visible", "Visible", "visible light", 3.8e-7, 7.5e-7, "visible", "eye"),
    ("near-ir", "Near infrared", "near infrared", 7.5e-7, 5e-6, "infrared", "near-ir"),
    ("mid-ir", "Mid infrared", "thermal infrared", 5e-6, 2.5e-5, "infrared", "mid-ir"),
    ("far-ir", "Far infrared", "far infrared", 2.5e-5, 3.5e-4, "infrared", None),
    ("mm", "Millimetre", "millimetre waves", 3.5e-4, 1e-2, "radio", None),
    ("radio", "Radio", "radio", 1e-2, 1e4, "radio", "radio"),
]

CHANNELS = [
    ("magnetic", "Magnetic field", "magnetic field", "mag"),
]


def rows(val):
    out = []
    for vid, physics, plain in VIEWS:
        out.append({"id": "view-" + vid, "kind": "view", "view_id": vid, "names": {"physics": physics, "plain": plain, "story": physics}})
    for bid, physics, plain, a, b, view, recipe in LIGHT:
        r = {"id": bid, "kind": "light", "names": {"physics": physics, "plain": plain, "story": physics},
             "from_m": val(a, "m", "sse-light-bands", "authored"), "to_m": val(b, "m", "sse-light-bands", "authored"),
             "view": "view-" + view}
        if recipe:
            r["recipe"] = recipe
        out.append(r)
    for cid, physics, plain, view in CHANNELS:
        out.append({"id": cid, "kind": "channel", "names": {"physics": physics, "plain": plain, "story": physics}, "view": "view-" + view})
    return out


def check_against_light(bands, light_pack):
    """Until the light thread reads this table, its own copy must agree."""
    sheet = {b["id"]: (b["from_m"]["v"], b["to_m"]["v"]) for b in bands if b["kind"] == "light"}
    theirs = {b["id"]: (b["from_m"], b["to_m"]) for b in light_pack["provides"]["other"]["light"]["bands"]}
    assert sheet == theirs, ("packs/base-light bands differ from the sheet's Bands table", sorted(set(sheet.items()) ^ set(theirs.items())))

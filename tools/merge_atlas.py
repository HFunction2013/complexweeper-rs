#!/usr/bin/env python3
"""Merge the 55 hyperbolic / end-of-game review sprites from the original Zig
atlas into the Rust atlas, keeping the Rust version's own pixel versions of the
5 shared slots it modified (icon, led_i, num_18, num_34, num_37).

Output: a single atlas (PNG + JSON) written to the Rust repo's assets/ dir.
"""
import json
from PIL import Image

import os

HERE = os.path.dirname(os.path.abspath(__file__))
RS_ROOT = os.path.join(HERE, "..")          # complexweeper-rs repo root
ZIG_ATLAS_DIR = os.environ.get(
    "ZIG_ATLAS_DIR", os.path.join(HERE, "..", "..", "complexweeper-zig", "素材")
)

RS_ATLAS_PNG = os.path.join(RS_ROOT, "assets", "atlas.png")
RS_ATLAS_JSON = os.path.join(RS_ROOT, "assets", "atlas.json")
ZG_ATLAS_PNG = os.path.join(ZIG_ATLAS_DIR, "图集.png")
ZG_ATLAS_JSON = os.path.join(ZIG_ATLAS_DIR, "图集.json")

rs_img = Image.open(RS_ATLAS_PNG).convert("RGBA")
zg_img = Image.open(ZG_ATLAS_PNG).convert("RGBA")
with open(RS_ATLAS_JSON) as f:
    rs_json = json.load(f)
with open(ZG_ATLAS_JSON, encoding="utf-8") as f:
    zg_json = json.load(f)

rs_slots = {s["name"]: s for s in rs_json["slots"]}
zg_slots = {s["name"]: s for s in zg_json["slots"]}

# Slots that exist only in the Zig atlas -> copy them over.
ADD = [
    "hnum_3", "hnum_7", "hnum_12", "hnum_15", "hnum_21", "hnum_24", "hnum_35", "hnum_48",
    "hnum_64_i", "hnum_49_i", "hnum_48_i", "hnum_36_i", "hnum_35_i", "hnum_32_i",
    "hnum_25_i", "hnum_24_i", "hnum_21_i", "hnum_16_i", "hnum_15_i", "hnum_12_i",
    "hnum_9_i", "hnum_8_i", "hnum_7_i", "hnum_5_i", "hnum_4_i", "hnum_3_i", "hnum_1_i",
    "led_j",
    "hrightflag_3", "hright_3", "right_1", "right_3", "hrightflag_4", "hright_4",
    "right_2", "right_4", "hwrongflag_3", "hwrong_3", "wrongblank", "hwrongflag_4",
    "hwrong_4", "hflag_3", "hmine_3", "hflag_4", "hmine_4", "hboom_3", "hboom_4",
    "rightflag_1", "rightflag_3", "rightflag_2", "rightflag_4",
    "wrongflag_1", "wrongflag_3", "wrongflag_2", "wrongflag_4",
]

missing = [n for n in ADD if n not in zg_slots]
if missing:
    raise SystemExit("missing in zig atlas: %s" % missing)

# Idempotent: skip slots that are already present in the Rust atlas.
ADD = [n for n in ADD if n not in rs_slots]
print("adding %d new slots" % len(ADD))
if not ADD:
    print("atlas already up to date; nothing to do")
    raise SystemExit(0)

BASE_W = rs_img.width
BASE_H = rs_img.height
PAD = 1  # 1px padding between slots to avoid bleeding

# Greedy row packing starting below the base atlas.
new_slots = []
x, y = 0, BASE_H + PAD
row_h = 0
for name in ADD:
    s = zg_slots[name]
    w, h = s["w"], s["h"]
    if x + w > BASE_W:
        x = 0
        y += row_h + PAD
        row_h = 0
    new_slots.append({"name": name, "x": x, "y": y, "w": w, "h": h})
    x += w + PAD
    row_h = max(row_h, h)

total_h = y + row_h + PAD
out = Image.new("RGBA", (BASE_W, total_h), (0, 0, 0, 0))
out.paste(rs_img, (0, 0))
for ns in new_slots:
    s = zg_slots[ns["name"]]
    tile = zg_img.crop((s["x"], s["y"], s["x"] + s["w"], s["y"] + s["h"]))
    out.paste(tile, (ns["x"], ns["y"]))

out.save(RS_ATLAS_PNG)
rs_json["height"] = total_h
rs_json["slots"].extend(new_slots)
with open(RS_ATLAS_JSON, "w") as f:
    json.dump(rs_json, f, indent=2)

print("new atlas size:", out.size)
print("total slots:", len(rs_json["slots"]))

# Sanity: re-read and verify every slot rect is inside the canvas and non-empty.
img2 = Image.open(RS_ATLAS_PNG).convert("RGBA")
with open(RS_ATLAS_JSON) as f:
    aj2 = json.load(f)
names = set()
for s in aj2["slots"]:
    assert s["x"] >= 0 and s["y"] >= 0 and s["x"] + s["w"] <= aj2["width"] and s["y"] + s["h"] <= aj2["height"], s
    assert s["name"] not in names, s["name"]
    names.add(s["name"])
    crop = img2.crop((s["x"], s["y"], s["x"] + s["w"], s["y"] + s["h"]))
    assert crop.getbbox() is not None, "empty slot: %s" % s["name"]
print("sanity check OK; unique slot names:", len(names))

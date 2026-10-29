# Makes the classic cockpit's WAY page editing buttons push buttons and saves the .blend master:
#   blender -b assets-src/blend/cockpit_classic.blend --python-exit-code 1 \
#       --python tools/models/classic_way_buttons.py
# The right MFD's bezel has a column of five flat plates down its left edge and one at the bottom
# of its right edge (the two above it are the next / previous waypoint caps). Each plate is
# replaced by a copy of the next waypoint cap (`control_Action_NextWaypoint_`: moving cap on a
# dark body, label face, highlight frame) on the plate's place and pitch:
#   left edge, top to bottom: the up, down, left and right arrows (`Osb(Right, 10..13)`, the
#   WAY page's slew buttons, `mfd::way_osb`), the return arrow (`Osb(Right, 14)`, the reset
#   button): they work in the WAY page's edit mode only;
#   right edge, bottom: MOV (`Action(ChangeWaypoint)`).
# The faces (arrows, the MOV label) are the layout's (`PanelLayout::classic`).
import re

import bpy
from mathutils import Vector

# (plate, face, control, kind)
BUTTONS = [
    ("swatch_35", "Arrow(Up, 47, 22)", "Osb(Right, 10)", "osb"),
    ("swatch_36", "Arrow(Down, 47, 22)", "Osb(Right, 11)", "osb"),
    ("swatch_37", "Arrow(Left, 47, 22)", "Osb(Right, 12)", "osb"),
    ("swatch_38", "Arrow(Right, 47, 22)", "Osb(Right, 13)", "osb"),
    ("swatch_39", "Arrow(Return, 47, 22)", "Osb(Right, 14)", "osb"),
    ("swatch_42", 'Label("MOV", 47, 22)', "Action(ChangeWaypoint)", "cap"),
]

root = bpy.data.objects["cockpit"]
template = bpy.data.objects["control_Action_NextWaypoint_"]
collection = template.users_collection[0]
# Where the cap's face points: its local -Y.
out = template.matrix_world.to_3x3() @ Vector((0.0, -1.0, 0.0))


def prop(v):
    return v.to_list() if hasattr(v, "to_list") else v


def unique(name):
    n, free = 0, name
    while free in bpy.data.objects:
        n += 1
        free = f"{name}_{n}"
    return free


def copy_tree(src, parent, tag, name=None):
    """A copy of `src` (called `name`) and its children under `parent`, the meshes shared."""
    base = src.name.rsplit(".", 1)[0].rstrip("_0123456789")
    o = bpy.data.objects.new(name or unique(f"{base}_{tag}"), src.data)
    collection.objects.link(o)
    o.parent = parent
    o.rotation_mode = src.rotation_mode
    o.location = src.location
    o.rotation_quaternion = src.rotation_quaternion
    o.scale = src.scale
    for k in src.keys():
        o[k] = prop(src[k])
    for c in src.children:
        copy_tree(c, o, tag)
    return o


half = prop(template["half"])
done = []
for plate, face, control, kind in BUTTONS:
    old = bpy.data.objects.get(plate)
    if old is None:
        raise SystemExit(f"{plate} is gone: already converted?")
    pts = [old.matrix_world @ v.co for v in old.data.vertices]
    # The plate's back, on the panel: the vertices lowest along the face's direction.
    low = min(p.dot(out) for p in pts)
    back = [p for p in pts if p.dot(out) < low + 1e-4]
    centre = sum(back, Vector()) / len(back)
    width = max(p.x for p in back) - min(p.x for p in back)
    if abs(width - 2 * half[0]) > 0.002:
        raise SystemExit(f"{plate}: {width:.4f} m wide, the cap is {2 * half[0]:.4f}")
    tag = re.sub(r"\W+", "_", face).strip("_").lower()
    cap = copy_tree(template, root, tag, "control_" + re.sub(r"\W+", "_", control))
    cap.location = centre
    cap["control"] = control
    cap["kind"] = kind
    atlas = next(c for c in cap.children_recursive if c.get("atlas_face"))
    atlas["atlas_face"] = f"Item({face})"
    bpy.data.objects.remove(old, do_unlink=True)
    done.append(f"{plate} -> {cap.name} at {tuple(round(x, 4) for x in centre)}")
print("\n".join(done))
bpy.ops.wm.save_mainfile()

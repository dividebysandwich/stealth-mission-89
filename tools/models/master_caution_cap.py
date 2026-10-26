# Makes the classic cockpit's master caution light a push button (`Action(MasterCaution)`) and
# saves the .blend master:
#   blender -b assets-src/blend/cockpit_classic.blend --python-exit-code 1 \
#       --python tools/models/master_caution_cap.py
# The lamp face (the atlas face `Item(Lamp(Mw, None))`, flush with the panel) becomes the face
# of a cap like the gear light's: a control Empty where the face was, the face 6 mm out on its
# `moving` child over a dark cap body, and a highlight frame round it.
import bpy

FACE = "Item(Lamp(Mw, None))"
HEIGHT, TRAVEL = 0.006, 0.005

root = bpy.data.objects["cockpit"]
face = next(o for o in bpy.data.objects if o.get("atlas_face") == FACE)
if face.parent is not root:
    raise SystemExit(f"{face.name} is already under {face.parent.name}")
gear = bpy.data.objects["control_Action_Gear_"]
cube = next(o for o in gear.children_recursive if o.name.startswith("dark")).data
frame = next(o for o in gear.children_recursive if o.name.startswith("highlight_")).data
scene = bpy.context.scene.collection
hw = max(abs(v.co.x) for v in face.data.vertices)
hh = max(abs(v.co.z) for v in face.data.vertices)


def obj(name, data, parent, loc=(0.0, 0.0, 0.0), scale=(1.0, 1.0, 1.0), props=None):
    o = bpy.data.objects.new(name, data)
    scene.objects.link(o)
    o.parent = parent
    o.rotation_mode = "QUATERNION"
    o.location, o.scale = loc, scale
    for k, v in (props or {}).items():
        o[k] = v
    return o


c = obj("control_Action_MasterCaution_", None, root, props={"control": "Action(MasterCaution)", "kind": "cap", "half": [hw, hh], "height": HEIGHT, "travel": TRAVEL})
c.matrix_local = face.matrix_local.copy()
moving = obj("moving_mw", None, c, props={"moving": True})
face.parent = moving
face.matrix_parent_inverse.identity()
face.matrix_basis.identity()
face.location = (0.0, -HEIGHT, 0.0)
obj("dark_mw", cube, moving, (0.0, -HEIGHT / 2 + 0.0005, 0.0), (2 * hw, HEIGHT, 2 * hh))
hl = obj("highlight_mw", None, c, props={"highlight": True})
t = 0.0015
for k, (x, z, sx, sz) in enumerate(((0, hh + t, 2 * hw + 4 * t, 2 * t), (0, -hh - t, 2 * hw + 4 * t, 2 * t), (hw + t, 0, 2 * t, 2 * hh), (-hw - t, 0, 2 * t, 2 * hh))):
    obj(f"highlight_mw_{k}", frame, hl, (x, -HEIGHT, z), (sx, 0.001, sz), {"no_shadow": True})
print(f"{face.name}: master caution cap, half {hw:.4f} x {hh:.4f} m")
bpy.ops.wm.save_mainfile()

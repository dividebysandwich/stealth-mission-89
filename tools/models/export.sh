#!/bin/sh
# Exports the .blend masters to the shipped model files:
#   tools/models/export.sh [aircraft central_europe cuba ... cockpit_modern cockpit_classic cockpit_f16 cockpit_uh60 cockpit_ah1z cockpit_tornado cockpit_a10 cockpit_rah66 cockpit_av8b cockpit_oh58 cockpit_console cockpit_drone]   (default: all files)
# The cockpits (cockpit_*) keep normals, UVs and materials (tools/models/export_cockpit.py);
# cockpit_f16.blend is generated first by tools/models/build_cockpit_f16.py, cockpit_uh60.blend by
# tools/models/build_cockpit_uh60.py, cockpit_ah1z.blend by tools/models/build_cockpit_ah1z.py,
# cockpit_tornado.blend by tools/models/build_cockpit_tornado.py, cockpit_a10.blend by
# tools/models/build_cockpit_a10.py, cockpit_rah66.blend by tools/models/build_cockpit_rah66.py,
# cockpit_av8b.blend by tools/models/build_cockpit_av8b.py, cockpit_oh58.blend by
# tools/models/build_cockpit_oh58.py,
# cockpit_console.blend by tools/models/build_console.py, cockpit_drone.blend by
# tools/models/build_console_drone.py.
# assets/models/ukraine.glb is assembled from central_europe.glb by tools/theater/build.sh.
set -e
cd "$(dirname "$0")/../.."
for f in ${@:-aircraft central_europe cuba korea desert_storm libya middle_east north_cape persian_gulf vietnam special_targets cockpit_modern cockpit_classic cockpit_f16 cockpit_uh60 cockpit_ah1z cockpit_tornado cockpit_a10 cockpit_rah66 cockpit_av8b cockpit_oh58 cockpit_console cockpit_drone}; do
    blend=assets-src/blend/$f.blend
    script=tools/models/export.py
    case $f in cockpit_*) script=tools/models/export_cockpit.py ;; esac
    log=$(blender -b "$blend" --python-exit-code 1 --python $script -- "assets/models/$f.glb" 2>&1) || {
        echo "$log"
        exit 1
    }
    sha256sum "$blend" | cut -d' ' -f1 >"assets-src/blend/$f.stamp"
    echo "$f: exported"
done

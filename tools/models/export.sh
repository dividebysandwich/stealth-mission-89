#!/bin/sh
# Exports the .blend masters to the shipped model files:
#   tools/models/export.sh [aircraft central_europe cuba ... cockpit_modern cockpit_classic cockpit_f16 cockpit_uh60 cockpit_ah1z cockpit_tornado cockpit_a10 cockpit_rah66 cockpit_av8b cockpit_oh58 cockpit_ch47 cockpit_f22 cockpit_c5 cockpit_c130 cockpit_ah64 cockpit_ah64_cpg cockpit_oh58_cpg cockpit_rah66_cpg cockpit_mh60r cockpit_fa18e cockpit_ea18g_ewo cockpit_a6 cockpit_a6_bn cockpit_f14 cockpit_f14_rio cockpit_console cockpit_drone cockpit_gunner cockpit_asw]   (default: all files)
# The cockpits (cockpit_*) keep normals, UVs and materials (tools/models/export_cockpit.py);
# cockpit_f16.blend is generated first by tools/models/build_cockpit_f16.py, cockpit_uh60.blend by
# tools/models/build_cockpit_uh60.py, cockpit_ah1z.blend by tools/models/build_cockpit_ah1z.py,
# cockpit_tornado.blend by tools/models/build_cockpit_tornado.py, cockpit_a10.blend by
# tools/models/build_cockpit_a10.py, cockpit_rah66.blend and cockpit_rah66_cpg.blend by tools/models/build_cockpit_rah66.py,
# cockpit_av8b.blend by tools/models/build_cockpit_av8b.py, cockpit_oh58.blend and
# cockpit_oh58_cpg.blend by tools/models/build_cockpit_oh58.py, cockpit_ch47.blend by tools/models/build_cockpit_ch47.py,
# cockpit_f22.blend by tools/models/build_cockpit_f22.py, cockpit_c5.blend by tools/models/build_cockpit_c5.py,
# cockpit_c130.blend by tools/models/build_cockpit_c130.py, cockpit_ah64.blend by
# tools/models/build_cockpit_ah64.py, cockpit_ah64_cpg.blend by tools/models/build_cockpit_ah64_cpg.py,
# cockpit_mh60r.blend by tools/models/build_cockpit_mh60r.py, cockpit_fa18e.blend by tools/models/build_cockpit_fa18e.py,
# cockpit_ea18g_ewo.blend by tools/models/build_cockpit_ea18g_ewo.py,
# cockpit_a6.blend and cockpit_a6_bn.blend by tools/models/build_cockpit_a6.py,
# cockpit_f14.blend and cockpit_f14_rio.blend by tools/models/build_cockpit_f14.py,
# cockpit_console.blend by tools/models/build_console.py, cockpit_drone.blend by
# tools/models/build_console_drone.py, cockpit_gunner.blend by tools/models/build_console_gunner.py,
# cockpit_asw.blend by tools/models/build_console_asw.py.
# assets/models/ukraine.glb is assembled from central_europe.glb by tools/theater/build.sh.
set -e
cd "$(dirname "$0")/../.."
for f in ${@:-aircraft central_europe cuba korea desert_storm libya middle_east north_cape persian_gulf vietnam special_targets cockpit_modern cockpit_classic cockpit_f16 cockpit_uh60 cockpit_ah1z cockpit_tornado cockpit_a10 cockpit_rah66 cockpit_av8b cockpit_oh58 cockpit_ch47 cockpit_f22 cockpit_c5 cockpit_c130 cockpit_ah64 cockpit_ah64_cpg cockpit_oh58_cpg cockpit_rah66_cpg cockpit_mh60r cockpit_fa18e cockpit_ea18g_ewo cockpit_a6 cockpit_a6_bn cockpit_f14 cockpit_f14_rio cockpit_console cockpit_drone cockpit_gunner cockpit_asw}; do
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

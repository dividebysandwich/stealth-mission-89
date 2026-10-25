#!/bin/sh
# Rebuilds a theater (models, tile layout, world, palette) from regions/<id>.py:
#   tools/theater/build.sh ukraine
set -e
cd "$(dirname "$0")/../.."
region=${1:?usage: tools/theater/build.sh <theater id>}
tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT
blender -b --factory-startup --python-exit-code 1 --python tools/theater/models.py -- "$region" "$tmp/new.glb" >"$tmp/models.log" 2>&1 || {
    cat "$tmp/models.log"
    exit 1
}
python3 tools/theater/assemble.py "$region" "$tmp/new.glb"
python3 tools/models/water_shapes.py

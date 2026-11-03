#!/bin/sh
# Packs the Android app (.github/workflows/android.yml): the native library of each ABI built by
#   cargo ndk -t <abi> -P 28 build --release -p stealth-android
# with assets/, the manifest and the icon (packaging/android) into a signed APK.
#
#   tools/android-apk.sh out.apk arm64-v8a=target/aarch64-linux-android/release/libstealth_mission_89.so ...
#
# Needs $ANDROID_HOME (or $ANDROID_SDK_ROOT) with build-tools and a platform, and zip. Signed with
# packaging/android/debug.keystore, a debug key in the repository: every build has the same
# signature, so a new APK installs over the last and keeps the player's files.
set -eu

[ $# -ge 2 ] || { echo "usage: $0 out.apk <abi>=<lib.so>..." >&2; exit 1; }
out=$(realpath -m "$1")
shift
root=$(cd "$(dirname "$0")/.." && pwd)
sdk=${ANDROID_HOME:-${ANDROID_SDK_ROOT:?set ANDROID_HOME}}
tools=$(ls -d "$sdk"/build-tools/* | sort -V | tail -1)
jar=$(ls -d "$sdk"/platforms/android-*/android.jar | sort -V | tail -1)

# 0.8.4 -> versionCode 804.
version=$(sed -n 's/^version = "\(.*\)"/\1/p' "$root/Cargo.toml" | head -1)
code=$(echo "$version" | awk -F. '{ print $1 * 10000 + $2 * 100 + $3 }')

work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT

"$tools/aapt2" compile --dir "$root/packaging/android/res" -o "$work/res.zip"
# Media already compressed are stored as they are, so they load without inflating.
"$tools/aapt2" link -o "$work/unaligned.apk" -I "$jar" \
    --manifest "$root/packaging/android/AndroidManifest.xml" \
    --min-sdk-version 28 --target-sdk-version 35 \
    --version-code "$code" --version-name "$version" \
    -A "$root/assets" -0 ogg -0 flac -0 png -0 jpg \
    "$work/res.zip"

for arg in "$@"; do
    abi=${arg%%=*}
    mkdir -p "$work/lib/$abi"
    cp "${arg#*=}" "$work/lib/$abi/libstealth_mission_89.so"
done
# Stored, not deflated: the libraries load straight from the APK (extractNativeLibs="false").
(cd "$work" && zip -q -0 -r unaligned.apk lib)

"$tools/zipalign" -f -p 4 "$work/unaligned.apk" "$work/aligned.apk"
"$tools/apksigner" sign --ks "$root/packaging/android/debug.keystore" \
    --ks-pass pass:android --ks-key-alias androiddebugkey --key-pass pass:android \
    --out "$out" "$work/aligned.apk"
echo "$out: $version ($code), $(du -h "$out" | cut -f1)"

"""Makes the sonar's ship screw recordings loop seamlessly (the MH-60R operator's headset,
src/asw/audio.rs):

    python3 tools/gen_ship_loops.py

assets/sounds/ship-big1.flac and ship-med1.flac do not loop: their ends do not meet. Each is
taken to mono 48 kHz (ffmpeg), its last `XFADE` seconds crossfaded (equal power) into its first,
and the result, `XFADE` shorter, written as <name>_loop.flac: played round and round, the join is
where the crossfade was, and the screws' beat and the noise run on unbroken. Both are levelled to
the same loudness (RMS).
"""

import os
import subprocess
import sys

import numpy as np

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), ".."))
SOUNDS = os.path.join(ROOT, "assets", "sounds")
RATE = 48000
# Crossfade length (s) per file: a good part of a longer recording, less of the short one.
FILES = {"ship-big1": 2.0, "ship-med1": 0.8}
RMS = 0.12


def decode(path):
    raw = subprocess.run(
        ["ffmpeg", "-v", "error", "-i", path, "-ac", "1", "-ar", str(RATE), "-f", "f32le", "-"],
        check=True,
        capture_output=True,
    ).stdout
    return np.frombuffer(raw, dtype="<f4").astype(np.float64)


def encode(x, path):
    pcm = (np.clip(x, -1.0, 1.0) * 32767).astype("<i2").tobytes()
    subprocess.run(
        ["ffmpeg", "-v", "error", "-y", "-f", "s16le", "-ar", str(RATE), "-ac", "1", "-i", "-",
         "-c:a", "flac", path],
        input=pcm,
        check=True,
    )


def loop(x, xfade):
    n = int(xfade * RATE)
    if len(x) < 3 * n:
        sys.exit("too short for its crossfade")
    x = x - x.mean()
    t = np.linspace(0.0, np.pi / 2, n)
    fade_in, fade_out = np.sin(t), np.cos(t)
    body = x[: len(x) - n].copy()
    # The tail fades out over the head fading in: the end of the loop runs into its start.
    body[:n] = x[:n] * fade_in + x[len(x) - n :] * fade_out
    return body * (RMS / np.sqrt((body**2).mean()))


for name, xfade in FILES.items():
    src = os.path.join(SOUNDS, name + ".flac")
    out = os.path.join(SOUNDS, name + "_loop.flac")
    y = loop(decode(src), xfade)
    encode(y, out)
    # The join: the last sample against the first, against the typical step.
    step = np.abs(np.diff(y)).mean()
    print(f"{out}: {len(y) / RATE:.2f} s, join step {abs(y[0] - y[-1]) / step:.1f}x typical")

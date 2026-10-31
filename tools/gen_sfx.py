#!/usr/bin/env python3
"""Synthesizes the sound effects (assets/sounds/sfx_NN_*.flac and the cannon's loop and tail).

- sfx_04_impact: the aircraft hitting the ground: a deep thud, a crunch of metal and a rumble.
- sfx_05_explosion: a blast, its noise falling from bright to dull, and a long rumble.
- sfx_06_cannon: a short burst of the cannon (cannon_loop.flac, then cannon_tail.flac).
- cannon_loop: the cannon firing, 100 rounds a second (20 in the loop, so it loops seamlessly):
  each round a crack over a thump, the barrels' rattle under them.
- cannon_tail: the burst's echo and the barrels running down when the trigger is let go.
- sfx_07_aircraft_contact: the threat warning receiver's new air contact: three quick buzzy
  beeps, higher than its radar tones.
- sfx_10_touchdown: the tyres touching down: a thump and a short squeal.
- sfx_14_stall_warning: the stall horn, a buzzy tone (the game repeats it while stalled).
- sfx_15_countermeasure: a flare or chaff cartridge: a pop and the hiss of the flare burning
  away behind.
- sfx_16_bomb_release: the ejector rack's cartridge and hooks (a sharp clack and a thud) and
  the store's rush away in the airflow.
- sfx_17_catapult: the catapult stroke: a hiss and roar building up, the shuttle's rumble and
  the jolt at the end.
- sfx_18_camera_shutter: the reconnaissance camera: two clicks and a short film advance whir.
- sfx_19_gear: the landing gear: the hydraulics' whine and the clunk of the gear locking.
- sfx_20_eject: the canopy charge, the seat's rocket and the wind blast fading.
- sfx_31_bay_doors: the weapon bay doors' actuators, ending in a clunk.
- sfx_32_error_buzz: a short buzzer.
- objective_complete: a mission objective achieved: a chime, two rising notes.
- radio_click: a radio call (air traffic control): the key's click, a breath of static in a
  narrow radio band and the squelch's tail.
- tank_gun: a tank's main gun close by: the muzzle blast's crack and boom, the ground's
  rumble and its echo off the country around.
- tank_gun_far: the same a few kilometres off: the boom and the rolling rumble, the crack long
  gone in the air.
- autocannon: one round of an infantry fighting vehicle's 25 or 30 mm gun: a hard bang over
  a short thump, a quick echo (the game plays one per round of a burst).
- autocannon_far: the same far off: a dull knock.

Each one is scaled to the loudness (RMS) the game's mix expects of it. Mono, 44.1 kHz,
16 bit, like the other game sounds.

    python3 tools/gen_sfx.py [file.flac ...]      (needs numpy and ffmpeg)

Without arguments every sound is written, else only those named (the vehicle guns have their
own random streams, so writing them alone gives the same files).
"""

import os
import subprocess

import numpy as np

ROOT = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..")
OUT = os.path.join(ROOT, "assets", "sounds")
RATE = 44100

rng = np.random.default_rng(117)


def seconds(n):
    return np.arange(int(n * RATE)) / RATE


def noise(t, lo, hi, order=2.0):
    """Band-limited noise of `t` seconds between `lo` and `hi` Hz, RMS 1."""
    n = int(t * RATE)
    f = np.fft.rfftfreq(n, 1.0 / RATE)
    f[0] = 1e-3
    mag = 1.0 / np.sqrt(1.0 + (lo / f) ** (2 * order)) / np.sqrt(1.0 + (f / hi) ** (2 * order))
    spec = mag * np.exp(2j * np.pi * rng.random(len(f)))
    spec[0] = 0.0
    x = np.fft.irfft(spec, n)
    return x / np.sqrt(np.mean(x * x))


def sweep_noise(t, hi_from, hi_to, lo=20.0):
    """Noise whose upper cutoff glides from `hi_from` to `hi_to` Hz (one-pole filters)."""
    x = rng.standard_normal(int(t * RATE))
    hi = np.geomspace(hi_from, hi_to, len(x))
    a = np.exp(-2 * np.pi * hi / RATE)
    y = np.empty_like(x)
    s1 = s2 = 0.0
    for i, (v, k) in enumerate(zip(x, a)):
        s1 = (1 - k) * v + k * s1
        s2 = (1 - k) * s1 + k * s2
        y[i] = s2
    y -= np.convolve(y, np.ones(int(RATE / lo)) / int(RATE / lo), "same")
    return y / np.sqrt(np.mean(y * y))


def env(t, attack, decay, hold=0.0):
    """Envelope: rises over `attack`, holds `hold` s, then decays with time constant `decay`."""
    x = seconds(t)
    rise = 1.0 - np.exp(-x / max(attack, 1e-5))
    fall = np.where(x < hold, 1.0, np.exp(-(x - hold) / decay))
    return rise * fall


def fades(x, fade_in=0.002, fade_out=0.02):
    n_in, n_out = int(fade_in * RATE), int(fade_out * RATE)
    x = x.copy()
    x[:n_in] *= np.linspace(0, 1, n_in)
    x[len(x) - n_out:] *= np.linspace(1, 0, n_out)
    return x


def at(total, start, x):
    """`x` placed `start` s into a silence of `total` s (cut at the end)."""
    out = np.zeros(int(total * RATE))
    i = int(start * RATE)
    out[i:i + len(x)] += x[:len(out) - i]
    return out


def thump(t, hz, decay, drop=0.5):
    """A low thud: a sine falling in pitch by `drop` (fraction), decaying."""
    x = seconds(t)
    f = hz * (1 - drop * (1 - np.exp(-x / decay)))
    return np.sin(2 * np.pi * np.cumsum(f) / RATE) * env(t, 0.001, decay)


def click(t, lo, hi, decay):
    return noise(t, lo, hi, 3.0) * env(t, 0.0002, decay)


def buzz(t, hz, harmonics, phase_rng=True):
    """A tone rich in odd harmonics (a softened square)."""
    x = seconds(t)
    return sum(np.sin(2 * np.pi * hz * h * x + (rng.random() * 6.28 if phase_rng else 0)) / h
               for h in range(1, 2 * harmonics, 2))


def norm(x):
    return x / np.max(np.abs(x))


def write(name, x, rms_db):
    """Writes `x` at the RMS level `rms_db` dBFS, the peaks softly limited under full scale."""
    target = 10 ** (rms_db / 20)
    limit = lambda g: 0.97 * np.tanh(x * g / 0.97)
    lo, hi = 0.0, 1e3 / np.max(np.abs(x))
    for _ in range(60):
        g = (lo + hi) / 2
        lo, hi = (g, hi) if np.sqrt(np.mean(limit(g) ** 2)) < target else (lo, g)
    x = limit((lo + hi) / 2)
    pcm = np.round(np.clip(x, -1, 1) * 32767).astype("<i2").tobytes()
    path = os.path.join(OUT, name)
    subprocess.run(
        ["ffmpeg", "-y", "-loglevel", "error", "-f", "s16le", "-ar", str(RATE), "-ac", "1", "-i", "-",
         "-compression_level", "12", path],
        input=pcm, check=True)
    print(path)


def impact():
    t = 2.0
    boom = thump(t, 55, 0.35, 0.4)
    crunch = noise(t, 150, 4000) * env(t, 0.003, 0.18)
    # Metal pieces: short resonant rings scattered over the first half second.
    debris = np.zeros(int(t * RATE))
    for _ in range(14):
        d = 0.05 + 0.3 * rng.random()
        ring = np.sin(2 * np.pi * (700 + 2200 * rng.random()) * seconds(d)) * env(d, 0.0005, d / 4)
        debris += at(t, 0.02 + 0.5 * rng.random(), ring * (0.3 + 0.7 * rng.random()))
    rumble = noise(t, 25, 220) * env(t, 0.02, 0.6)
    return fades(1.0 * boom + 0.8 * crunch + 0.25 * debris + 0.7 * rumble)


def explosion():
    t = 1.4
    blast = sweep_noise(t, 6000, 250) * env(t, 0.002, 0.35)
    low = thump(t, 48, 0.4, 0.5)
    crackle = noise(t, 1500, 7000) * (rng.random(int(t * RATE)) < 0.004) * env(t, 0.01, 0.3)
    crackle = np.convolve(crackle, np.exp(-np.arange(200) / 30), "same")
    return fades(1.0 * blast + 0.8 * low + 0.15 * norm(crackle))


def cannon_round(n):
    """One round's sound, `n` samples: a crack over a thump."""
    k = np.arange(n)
    crack = rng.standard_normal(n) * np.exp(-k / (0.0018 * RATE))
    crack = np.convolve(crack, [0.5, 0.0, -0.5], "same")
    body = np.sin(2 * np.pi * 95 * k / RATE) * np.exp(-k / (0.006 * RATE)) * (1 - np.exp(-k / 40))
    return 0.55 * crack + 1.0 * body


def cannon_loop():
    t, rounds = 0.2, 20
    n = int(t * RATE)
    out = np.zeros(n)
    k = int(0.02 * RATE)
    for i in range(rounds):
        idx = (int(round(i * n / rounds)) + np.arange(k)) % n
        out[idx] += (0.85 + 0.3 * rng.random()) * cannon_round(k)
    # The barrels' rattle: periodic noise in the loop (an FFT of the loop length).
    f = np.fft.rfftfreq(n, 1.0 / RATE)
    f[0] = 1e-3
    spec = (1 / np.sqrt(1 + (300 / f) ** 4) / np.sqrt(1 + (f / 2500) ** 4)) * np.exp(2j * np.pi * rng.random(len(f)))
    rattle = np.fft.irfft(spec, n)
    return out + 0.35 * rattle / np.max(np.abs(rattle))


def cannon_tail():
    t = 0.5
    echo = noise(t, 60, 1200) * env(t, 0.001, 0.12)
    spin = np.sin(2 * np.pi * np.cumsum(np.geomspace(330, 120, int(t * RATE))) / RATE) * env(t, 0.001, 0.15)
    last = at(t, 0.0, cannon_round(int(0.02 * RATE)))
    return fades(0.9 * last + 0.6 * echo + 0.15 * spin, 0.0, 0.05)


def echoes(x, delays, gains, hi):
    """`x` with its echoes `delays` s later, `gains` as loud, dulled above `hi` Hz."""
    t = len(x) / RATE
    f = np.fft.rfftfreq(len(x), 1.0 / RATE)
    dull = np.fft.irfft(np.fft.rfft(x) / np.sqrt(1.0 + (f / hi) ** 4), len(x))
    out = x.copy()
    for d, g in zip(delays, gains):
        out += g * at(t, d, dull)
    return out


def lowpass(x, hi, order=2.0):
    f = np.fft.rfftfreq(len(x), 1.0 / RATE)
    return np.fft.irfft(np.fft.rfft(x) / np.sqrt(1.0 + (f / hi) ** (2 * order)), len(x))


def tank_gun(far):
    global rng
    rng = np.random.default_rng(120 + far)
    t = 4.0
    # The muzzle blast: a crack, a broadband bang falling from bright to dull, a deep boom.
    crack = click(t, 2000, 12000, 0.0015)
    bang = sweep_noise(t, 9000, 300) * env(t, 0.0005, 0.12)
    boom = thump(t, 42, 0.3, 0.45)
    # The ground and the air shaking after it, rolling on.
    rumble = noise(t, 18, 160) * env(t, 0.03, 0.9)
    near = 0.5 * crack + 1.0 * bang + 1.3 * boom + 0.6 * rumble
    x = echoes(near, [0.55, 1.3, 2.1], [0.35, 0.2, 0.1], 900)
    if far:
        x = lowpass(x, 450) + 0.5 * noise(t, 15, 90) * env(t, 0.15, 1.2)
        x = at(t, 0.0, x) * env(t, 0.02, 10.0)
    return fades(x, 0.0005, 0.3)


def autocannon(far):
    global rng
    rng = np.random.default_rng(130 + far)
    t = 0.9
    crack = click(t, 1500, 10000, 0.0012)
    bang = noise(t, 200, 4000) * env(t, 0.0003, 0.025)
    body = thump(t, 75, 0.035, 0.3)
    tail = noise(t, 60, 900) * env(t, 0.005, 0.12)
    x = echoes(0.6 * crack + 1.0 * bang + 1.2 * body + 0.35 * tail, [0.25], [0.25], 1200)
    if far:
        x = lowpass(x, 600) * env(t, 0.004, 10.0)
    return fades(x, 0.0003, 0.08)


def contact():
    t = 0.42
    beep = lambda: buzz(0.075, 1100, 4, phase_rng=False) * env(0.075, 0.003, 1.0)
    x = sum(at(t, 0.11 * i, fades(beep(), 0.003, 0.01)) for i in range(3))
    return fades(x, 0.0, 0.02)


def objective_complete():
    t = 0.52
    note = lambda hz, d: np.sin(2 * np.pi * hz * seconds(d)) * env(d, 0.004, 0.09, 0.05)
    x = at(t, 0.0, note(880, 0.2)) + at(t, 0.18, note(1175, 0.3))
    return fades(x + 0.08 * at(t, 0.0, np.sin(2 * np.pi * 1760 * seconds(0.2)) * env(0.2, 0.004, 0.06)))


def radio_click():
    t = 0.32
    key = click(0.012, 600, 4000, 0.003)
    hiss = noise(0.2, 400, 3200, 3.0) * env(0.2, 0.004, 0.06, 0.04)
    tail = noise(0.12, 900, 3000, 3.0) * env(0.12, 0.002, 0.03)
    x = at(t, 0.0, key) + 0.35 * at(t, 0.006, hiss) + 0.8 * at(t, 0.2, tail)
    return fades(x, 0.0005, 0.02)


def touchdown():
    t = 0.29
    bump = thump(t, 70, 0.07, 0.3)
    f = np.geomspace(1900, 1500, int(t * RATE))
    squeal = np.sin(2 * np.pi * np.cumsum(f) / RATE) * env(t, 0.01, 0.06, 0.03)
    scrub = noise(t, 700, 4500) * env(t, 0.003, 0.05)
    return fades(1.0 * bump + 0.35 * squeal + 0.5 * scrub, 0.0005, 0.03)


def stall():
    t = 0.35
    horn = buzz(t, 430, 5) * env(t, 0.008, 1.0)
    return fades(horn, 0.008, 0.02)


def countermeasure():
    t = 1.9
    pop = thump(t, 120, 0.04, 0.5) + 0.7 * click(t, 800, 6000, 0.006)
    hiss = noise(t, 1200, 9000) * env(t, 0.03, 0.55) * (1 + 0.3 * np.sin(2 * np.pi * 11 * seconds(t)))
    whoosh = sweep_noise(t, 3000, 400) * env(t, 0.02, 0.4)
    return fades(1.0 * pop + 0.3 * hiss + 0.35 * whoosh)


def bomb_release():
    t = 1.7
    clack = click(t, 1500, 9000, 0.004) + 0.6 * at(t, 0.035, click(0.1, 900, 5000, 0.006))
    clunk = thump(t, 85, 0.06, 0.3)
    rush = sweep_noise(t, 2500, 300) * env(t, 0.06, 0.5)
    return fades(0.8 * clack + 1.0 * clunk + 0.45 * rush)


def catapult():
    t = 1.85
    x = seconds(t)
    build = np.clip(x / 1.55, 0, 1) ** 1.5 * np.where(x < 1.55, 1.0, np.exp(-(x - 1.55) / 0.08))
    hiss = noise(t, 800, 9000) * build
    roar = noise(t, 40, 600) * build
    shuttle = noise(t, 20, 140) * build * (1 + 0.4 * np.sin(2 * np.pi * 7 * x))
    jolt = at(t, 1.55, thump(0.3, 60, 0.08, 0.4)) + at(t, 1.55, click(0.05, 400, 3000, 0.01))
    return fades(0.4 * hiss + 0.8 * roar + 0.6 * shuttle + 1.0 * jolt)


def camera_shutter():
    t = 0.19
    x = at(t, 0.0, click(0.03, 2000, 12000, 0.0015)) + 0.8 * at(t, 0.055, click(0.03, 1500, 9000, 0.002))
    whir = buzz(0.08, 260, 3) * noise(0.08, 200, 3000) * env(0.08, 0.005, 0.03, 0.03)
    return fades(x + 0.12 * at(t, 0.09, whir), 0.0002, 0.01)


def gear():
    t = 2.0
    x = seconds(t)
    run = np.clip(x / 0.15, 0, 1) * np.where(x < 1.55, 1.0, np.exp(-(x - 1.55) / 0.05))
    f = 520 + 60 * np.clip(x / 1.5, 0, 1)
    whine = (np.sin(2 * np.pi * np.cumsum(f) / RATE) + 0.4 * np.sin(4 * np.pi * np.cumsum(f) / RATE)) * run
    flow = noise(t, 300, 3000) * run
    rumble = noise(t, 30, 200) * run
    lock = at(t, 1.6, thump(0.3, 110, 0.05, 0.3) + 0.8 * click(0.3, 600, 5000, 0.008))
    return fades(0.12 * whine + 0.15 * flow + 0.25 * rumble + 1.0 * lock)


def eject():
    t = 2.6
    x = seconds(t)
    bang = thump(t, 70, 0.08, 0.5) + 0.9 * click(t, 400, 8000, 0.015)
    burn = np.where((x > 0.12) & (x < 0.55), 1.0, 0.0)
    burn = np.convolve(burn, np.ones(400) / 400, "same")
    rocket = noise(t, 60, 5000) * burn * (1 + 0.25 * noise(t, 5, 40))
    wind = noise(t, 150, 2500) * np.clip((x - 0.3) / 0.2, 0, 1) * np.exp(-np.maximum(x - 0.5, 0) / 0.8)
    return fades(1.0 * bang + 0.75 * rocket + 0.5 * wind)


def bay_doors():
    t = 0.7
    x = seconds(t)
    run = np.clip(x / 0.05, 0, 1) * np.where(x < 0.58, 1.0, np.exp(-(x - 0.58) / 0.03))
    motor = buzz(t, 180, 4) * run
    flow = noise(t, 400, 3500) * run
    stop = at(t, 0.58, thump(0.12, 95, 0.04, 0.3) + 0.6 * click(0.12, 700, 5000, 0.006))
    return fades(0.15 * motor + 0.25 * flow + 1.0 * stop)


def error_buzz():
    t = 0.35
    return fades(buzz(t, 140, 9) * env(t, 0.003, 1.0), 0.003, 0.02)


if __name__ == "__main__":
    import sys

    wanted = sys.argv[1:]
    radio = {"radio_click.flac": (radio_click, -27.0)}
    guns = {
        "tank_gun.flac": (lambda: tank_gun(False), -13.0),
        "tank_gun_far.flac": (lambda: tank_gun(True), -16.0),
        "autocannon.flac": (lambda: autocannon(False), -15.0),
        "autocannon_far.flac": (lambda: autocannon(True), -19.0),
    }
    if wanted:
        for name in wanted:
            make, level = {**guns, **radio}[name]
            write(name, make(), level)
        sys.exit(0)
    loop, tail = cannon_loop(), cannon_tail()
    write("sfx_04_impact.flac", impact(), -14.0)
    write("sfx_05_explosion.flac", explosion(), -15.0)
    write("sfx_06_cannon.flac", np.concatenate([np.tile(loop, 3), tail]), -18.0)
    write("cannon_loop.flac", loop, -15.0)
    write("cannon_tail.flac", tail, -22.5)
    write("sfx_07_aircraft_contact.flac", contact(), -18.0)
    write("sfx_10_touchdown.flac", touchdown(), -17.5)
    write("sfx_14_stall_warning.flac", stall(), -15.5)
    write("sfx_15_countermeasure.flac", countermeasure(), -19.0)
    write("sfx_16_bomb_release.flac", bomb_release(), -18.0)
    write("sfx_17_catapult.flac", catapult(), -14.0)
    write("sfx_18_camera_shutter.flac", camera_shutter(), -26.0)
    write("sfx_19_gear.flac", gear(), -24.0)
    write("sfx_20_eject.flac", eject(), -13.5)
    write("sfx_31_bay_doors.flac", bay_doors(), -20.0)
    write("sfx_32_error_buzz.flac", error_buzz(), -28.0)
    write("objective_complete.flac", objective_complete(), -18.0)
    for name, (make, level) in radio.items():
        write(name, make(), level)
    for name, (make, level) in guns.items():
        write(name, make(), level)

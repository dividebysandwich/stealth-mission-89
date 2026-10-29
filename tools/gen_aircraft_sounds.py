#!/usr/bin/env python3
"""Synthesizes the loop of other helicopters heard around, and of the helicopter flown
(assets/sounds/*.flac). The other jets' loop (jet.flac) is a recording.

- helicopter.flac: main rotor blade slap (4 blades at 258 rpm, 17.2 Hz), a tail rotor buzz
  and the turbine whine over a low rumble.
- heli_rotor.flac: the flown UH-60A's rotor from inside: the blade passes' heavy thump through
  the airframe, a duller slap (the game raises its volume with torque and translational lift),
  the tail rotor and the main gearbox whine.
- heli_turbine.flac: its two turbines: whine, a compressor tone and broadband hiss (pitched with
  torque in the game).
- jet_engine.flac: the flown jet's engine at full rpm: the compressor's and the turbine's
  whine over the airflow's roar and hiss (the game plays it slower with the rpm, so it
  winds up from 0 on start).
- jet_wind.flac: the airflow over the canopy (louder with airspeed in the game): a broadband
  rush with a faint whistle and slow buffeting.
- afterburner.flac: the lit afterburner over the engine (louder with its setting in the game):
  a deep, throbbing roar, a coarser rush above it and the exhaust's crackle (sparse pops),
  soft-clipped to be loud.
- interceptor.flac: a drone command center's quadrotor interceptor heard around, at its hover
  throttle (20%; the game plays it up to two octaves higher at full throttle, so it keeps
  below about 6 kHz): four slightly detuned motors, each a buzz at its rotation with the
  three-bladed propeller's blade passes, wavering as the controller trims them, over the
  propellers' wash.

They loop seamlessly: the noise is synthesized in the frequency domain (periodic in the loop
length) and every tone and pulse train repeats a whole number of times per loop.
Mono, 44.1 kHz, 16 bit, like the other game sounds.

    python3 tools/gen_aircraft_sounds.py [file.flac ...]      (needs numpy and ffmpeg)

Without arguments every loop is written, else only those named.
"""

import os
import subprocess

import numpy as np

ROOT = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..")
OUT = os.path.join(ROOT, "assets", "sounds")
RATE = 44100

rng = np.random.default_rng(89)


def periodic_noise(seconds, shape):
    """Noise of `seconds` that loops seamlessly, amplitude spectrum `shape(freq_hz)`."""
    n = int(seconds * RATE)
    f = np.fft.rfftfreq(n, 1.0 / RATE)
    spec = shape(np.maximum(f, 1e-3)) * np.exp(2j * np.pi * rng.random(len(f)))
    spec[0] = 0.0
    x = np.fft.irfft(spec, n)
    return x / np.sqrt(np.mean(x * x))


def band(f, lo, hi, order=2.0):
    """Smooth band-pass magnitude between `lo` and `hi` Hz."""
    return 1.0 / np.sqrt(1.0 + (lo / f) ** (2 * order)) / np.sqrt(1.0 + (f / hi) ** (2 * order))


def tone(seconds, hz, phase=0.0):
    """A sine with a whole number of cycles in the loop (frequency rounded to fit)."""
    n = int(seconds * RATE)
    cycles = max(1, round(hz * seconds))
    return np.sin(2 * np.pi * cycles * np.arange(n) / n + phase)


def swell(seconds, max_hz, depth):
    """Slow periodic amplitude fluctuation 1 +- depth."""
    m = periodic_noise(seconds, lambda f: (f <= max_hz) * 1.0)
    return 1.0 + depth * m / np.max(np.abs(m))


def write(name, x):
    x = x / np.max(np.abs(x)) * 0.89
    pcm = np.round(x * 32767).astype("<i2").tobytes()
    path = os.path.join(OUT, name)
    subprocess.run(
        ["ffmpeg", "-y", "-loglevel", "error", "-f", "s16le", "-ar", str(RATE), "-ac", "1", "-i", "-",
         "-compression_level", "12", path],
        input=pcm, check=True)
    print(path)


def helicopter():
    t = 3.0
    n = int(t * RATE)
    # Main rotor: 4 blades at 258 rpm (51 blade passes in the loop), each a sharp slap (filtered
    # noise burst) over a softer thump, the blades a little uneven.
    passes = round(4 * 258 / 60 * t)
    slap_noise = periodic_noise(t, lambda f: band(f, 250, 1800, 2.0))
    thump = np.zeros(n)
    slap = np.zeros(n)
    k = np.arange(int(0.06 * RATE))
    thump_pulse = np.sin(2 * np.pi * 70 * k / RATE) * np.exp(-k / (0.018 * RATE)) * (1 - np.exp(-k / (0.002 * RATE)))
    slap_env = np.exp(-k / (0.007 * RATE)) * (1 - np.exp(-k / (0.0008 * RATE)))
    unevenness = [1.0, 0.82, 0.93, 0.78]
    for i in range(passes):
        start = int(round(i * n / passes))
        idx = (start + k) % n
        g = unevenness[i % 4]
        thump[idx] += g * thump_pulse
        slap[idx] += g * slap_env
    slap *= slap_noise
    # Tail rotor buzz (about 4.6x the main rotor) with harmonics.
    tail_hz = 79.0
    tail = sum(tone(t, tail_hz * h, h) / h for h in range(1, 7))
    tail *= 1.0 + 0.3 * tone(t, 4 * 258 / 60)
    whine = tone(t, 3100) + 0.4 * tone(t, 6200, 1.0)
    rumble = periodic_noise(t, lambda f: band(f, 30, 400, 1.5))
    return 1.0 * thump / np.max(np.abs(thump)) + 0.9 * slap / np.max(np.abs(slap)) + 0.12 * tail / np.max(np.abs(tail)) + 0.025 * whine + 0.18 * rumble / np.max(np.abs(rumble))


def blade_passes(t, passes, pulse, unevenness=(1.0, 0.82, 0.93, 0.78)):
    """A train of `passes` pulses over the loop, the blades a little uneven."""
    n = int(t * RATE)
    out = np.zeros(n)
    k = np.arange(len(pulse))
    for i in range(passes):
        idx = (int(round(i * n / passes)) + k) % n
        out[idx] += unevenness[i % len(unevenness)] * pulse
    return out


def heli_rotor():
    t = 3.0
    passes = round(4 * 258 / 60 * t)
    k = np.arange(int(0.09 * RATE))
    # Felt more than heard inside: a deep thump per blade pass, a duller slap.
    thump = np.sin(2 * np.pi * 42 * k / RATE) * np.exp(-k / (0.03 * RATE)) * (1 - np.exp(-k / (0.003 * RATE)))
    thump = blade_passes(t, passes, thump)
    slap_env = np.exp(-k / (0.012 * RATE)) * (1 - np.exp(-k / (0.001 * RATE)))
    slap = blade_passes(t, passes, slap_env) * periodic_noise(t, lambda f: band(f, 120, 900, 2.0))
    tail = sum(tone(t, 79.0 * h, h) / h for h in range(1, 6))
    # Main gearbox: tones of the input and output stages.
    gearbox = tone(t, 1210) + 0.6 * tone(t, 605, 0.5) + 0.3 * tone(t, 2420, 1.0)
    rumble = periodic_noise(t, lambda f: band(f, 20, 250, 1.5))
    norm = lambda x: x / np.max(np.abs(x))
    return 1.0 * norm(thump) + 0.45 * norm(slap) + 0.08 * norm(tail) + 0.03 * gearbox + 0.35 * norm(rumble) * swell(t, 2.0, 0.2)


def heli_turbine():
    t = 2.0
    whine = tone(t, 3100) + 0.5 * tone(t, 6200, 1.0) + 0.25 * tone(t, 4650, 2.0)
    compressor = tone(t, 1550, 0.3) + 0.4 * tone(t, 775, 1.3)
    hiss = periodic_noise(t, lambda f: band(f, 800, 8000, 1.5))
    roar = periodic_noise(t, lambda f: band(f, 60, 700, 1.2))
    return 0.25 * whine * swell(t, 1.0, 0.1) + 0.12 * compressor + 0.5 * hiss + 0.6 * roar


def jet_engine():
    t = 4.0
    # Tones carry the pitch (the game plays the loop slower with the rpm, an octave down at
    # idle): the compressor's blade passing with a few harmonics and the turbine's whine above,
    # each wavering a little.
    compressor = sum(tone(t, 1100 * h, h) * swell(t, 3.0, 0.3) / h ** 1.5 for h in range(1, 4))
    whine = (tone(t, 2650) + 0.5 * tone(t, 3900, 1.0)) * swell(t, 1.5, 0.25)
    # The airflow through the engine: a roar peaking at a few hundred Hz, the intake hiss.
    roar = periodic_noise(t, lambda f: band(f, 70, 900, 1.2) + 0.3 * band(f, 900, 3000, 1.5)) * swell(t, 2.5, 0.12)
    hiss = periodic_noise(t, lambda f: band(f, 2000, 9000, 1.5))
    norm = lambda x: x / np.max(np.abs(x))
    return 0.35 * norm(compressor) + 0.25 * whine + 0.8 * norm(roar) + 0.12 * norm(hiss)


def jet_wind():
    t = 6.0
    # Air rushing past the canopy: broadband, falling off gently above a few hundred Hz,
    # gusting and buffeting.
    rush = periodic_noise(t, lambda f: band(f, 90, 400, 1.0) * (1.0 + 0.6 * band(f, 400, 5000, 0.8)))
    rush *= swell(t, 0.8, 0.25) * swell(t, 7.0, 0.08)
    # A faint whistle where the air passes the canopy frame, coming and going.
    whistle = periodic_noise(t, lambda f: np.exp(-((f - 1150) / 50) ** 2))
    whistle *= np.clip(swell(t, 0.5, 0.9) - 0.4, 0.0, None)
    rumble = periodic_noise(t, lambda f: band(f, 20, 120, 2.0)) * swell(t, 2.0, 0.4)
    norm = lambda x: x / np.max(np.abs(x))
    return 1.0 * norm(rush) + 0.04 * norm(whistle) + 0.3 * norm(rumble)


def afterburner():
    t = 5.0
    n = int(t * RATE)
    # The roar: most of it below a kilohertz, throbbing slowly as the flame burns unevenly (with
    # enough low mids to cut through the engine, not only a rumble under it).
    roar = periodic_noise(t, lambda f: band(f, 30, 300, 1.5) + 0.8 * band(f, 200, 1500, 1.2))
    roar *= swell(t, 6.0, 0.25) * swell(t, 1.2, 0.15)
    rush = periodic_noise(t, lambda f: band(f, 600, 4000, 1.3)) * swell(t, 3.0, 0.2)
    # Crackle: short bursts of bright noise at random moments (the pattern loops with the file).
    crackle = np.zeros(n)
    burst = int(0.012 * RATE)
    env = np.exp(-np.arange(burst) / (0.0025 * RATE))
    for start in rng.integers(0, n, int(t * 45)):
        amp = rng.uniform(0.3, 1.0) ** 2
        idx = (start + np.arange(burst)) % n
        crackle[idx] += amp * env * rng.standard_normal(burst)
    spec = np.fft.rfft(crackle)
    f = np.fft.rfftfreq(n, 1.0 / RATE)
    crackle = np.fft.irfft(spec * band(np.maximum(f, 1e-3), 500, 6000, 1.0), n)
    norm = lambda x: x / np.max(np.abs(x))
    mix = 1.0 * norm(roar) + 0.45 * norm(rush) + 0.5 * norm(crackle)
    # Soft clipping: louder for its peak, a harsher edge.
    return np.tanh(2.5 * mix / np.max(np.abs(mix)))


def interceptor():
    t = 3.0
    rng_q = np.random.default_rng(105)
    n = int(t * RATE)
    norm = lambda x: x / np.max(np.abs(x))
    motors = np.zeros(n)
    # Four motors at about 56 000 rpm (small props at hover throttle), a few Hz apart: they
    # beat.
    for hz in (924.0, 942.0, 966.0, 988.0):
        cycles = round(hz * t)
        rot = lambda h: np.sin(2 * np.pi * cycles * h * np.arange(n) / n + rng_q.uniform(0, 2 * np.pi))
        # The rotation's harmonics falling off, the blade passes (3x) strongest.
        buzz = sum(rot(h) / h for h in range(1, 7)) + 1.6 * rot(3) + 0.8 * rot(6)
        # The flight controller trims each motor many times a second.
        wobble = 1.0 + 0.25 * norm(periodic_noise(t, lambda f: band(f, 3, 18, 2.0)))
        motors += buzz * wobble
    # The propellers' wash: a hiss pulsing with the blade passes.
    wash = periodic_noise(t, lambda f: band(f, 1200, 5000, 2.0)) * (1.0 + 0.4 * tone(t, 3 * 956.0))
    mix = 1.0 * norm(motors) + 0.25 * norm(wash)
    # Soft clipping: the small motors' harsh edge.
    return np.tanh(1.8 * mix / np.max(np.abs(mix)))


LOOPS = {
    "helicopter.flac": helicopter,
    "heli_rotor.flac": heli_rotor,
    "heli_turbine.flac": heli_turbine,
    "jet_engine.flac": jet_engine,
    "jet_wind.flac": jet_wind,
    "afterburner.flac": afterburner,
    "interceptor.flac": interceptor,
}

if __name__ == "__main__":
    import sys

    for name in sys.argv[1:] or LOOPS:
        write(name, LOOPS[name]())

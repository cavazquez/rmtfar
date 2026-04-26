# RMTFAR Architecture

## Overview

RMTFAR has two main runtime components plus two auxiliary crates for testing.

| Component | Process | Language | Responsibility |
|-----------|---------|----------|---------------|
| `@rmtfar` Arma mod | Arma 3 | SQF + `rmtfar-extension` DLL | Capture player state, write MumbleLink, send radio state to plugin |
| `rmtfar-plugin` | Mumble client | Rust (cdylib) | Process audio per user (mute/unmute/DSP) |
| `rmtfar-bridge` *(testing only)* | Local process | Rust | Linux substitute for the extension DLL during development |
| `rmtfar-test-client` *(testing only)* | Local process | Rust | Simulates an Arma 3 player sending state to the bridge |

## Data Flow (production — Windows + Arma 3)

```
Arma 3 (SQF)
  └─ callExtension ["send", [v1|payload]]
       └─ rmtfar_x64.dll (Rust cdylib — rmtfar-extension)
            ├─ SharedMem "MumbleLink" ──► Mumble built-in Link plugin (positional audio)
            └─ UDP :9501 (loopback)    ──► rmtfar-plugin (inside Mumble)
                                              └─ audio callbacks: mute / volume / DSP
```

The extension DLL handles everything directly inside the Arma 3 process. There is **no intermediate bridge process** in production — this mirrors how TFAR communicates with TeamSpeak.

## Data Flow (testing — Linux without Arma 3)

```
rmtfar-test-client (UDP :9500)
  └─ rmtfar-bridge
       ├─ SharedMem "MumbleLink" ──► Mumble built-in Link plugin
       └─ UDP :9501             ──► rmtfar-plugin
```

The bridge accepts the same pipe-delimited `v1|` payloads as the extension DLL and replicates its behaviour. This allows end-to-end testing on Linux with two Mumble instances.

## Coordinate Systems

Arma 3 uses:
- X = east (metres)
- Y = north (metres)
- Z = altitude ASL (metres)
- Direction: degrees, 0 = north, clockwise

Mumble uses (left-handed):
- X = right (east)
- Y = up (altitude)
- Z = forward (north)

Conversion: `mumble_pos = [arma_x, arma_z, arma_y]`

## MumbleLink

The extension DLL writes to the `MumbleLink` shared memory segment on every
`send` call from SQF (≈20 Hz). Mumble's built-in Link plugin reads this and
applies 3D positional audio automatically.

## Radio State Protocol

On every `send` call the extension also broadcasts a `RadioStateMessage` JSON
object to UDP :9501. The plugin reads this non-blocking on each audio callback
and uses it to decide mute/volume/DSP per remote user.

See [protocol.md](protocol.md) for the full wire format.

## Plugin Audio Callback

`mumble_onAudioSourceFetched` is called for every decoded audio frame.
The decision tree:

```
Is user known? (identity mapping via mumble_onUserTalking)
  └─ Yes:
       Is local player alive + conscious?
         └─ No  → mute
         └─ Yes:
              Is remote transmitting on radio?
                └─ Yes:
                     Same frequency? Same channel? In range? LOS quality OK?
                       └─ Yes → apply radio DSP (biquad + AGC + bitcrusher + noise), pass
                       └─ No  → mute
                └─ No:
                     Is remote transmitting locally (PTT local)?
                       └─ Yes:
                            In vehicle?
                              └─ Yes → mute (vehicle blocks local PTT)
                              └─ No:
                                   Distance ≤ 50 m?
                                     └─ Yes → apply linear volume falloff, pass
                                     └─ No  → mute
                       └─ No  → mute
  └─ No → pass through (don't mute unknown users)
```

## Phase Roadmap

| Phase | Feature | Status |
|-------|---------|--------|
| 1 | Proximity voice (MumbleLink + plugin) | ✅ |
| 2 | Radio frequencies, PTT, basic DSP | ✅ |
| 3 | SR/LR radios, channels, interference, vehicles, death | ✅ |
| 4 | Extension DLL + Plugin for Windows (cross-compile) | ✅ |
| 5 | Testing in Windows with Arma 3 (PBO, CBA keybinds) | 🚧 |

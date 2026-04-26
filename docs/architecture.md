# RMTFAR Architecture

## Overview

RMTFAR has three independent, loosely coupled components:

| Component | Process | Language | Responsibility |
|-----------|---------|----------|---------------|
| `@rmtfar` Arma mod | Arma 3 | SQF + Rust DLL | Capture player state, send to bridge |
| `rmtfar-bridge` | Local daemon | Rust | Translate game state → Mumble data |
| `rmtfar-plugin` | Mumble client | Rust (cdylib) | Process audio per user |

## Data Flow

```
Arma 3 (SQF)
  └─ callExtension "send" [json]
       └─ rmtfar_x64.dll (Rust cdylib)
            └─ UDP :9500 (loopback, 20 Hz)
                 └─ rmtfar-bridge
                      ├─ SharedMem "MumbleLink" ──► Mumble Link plugin (positional)
                      └─ UDP :9501 (loopback)    ──► rmtfar-plugin
                                                       └─ audio callbacks (mute/DSP)
```

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

## Mumble Link

The bridge writes to the `MumbleLink` shared memory segment at 20 Hz.
Mumble's built-in Link plugin reads this and applies 3D positional audio
automatically. No plugin code is needed for Phase 1 proximity falloff.

## Radio State Protocol

Every time the bridge receives a player state update, it broadcasts a
`RadioStateMessage` to UDP :9501. The plugin reads this non-blocking and
uses it to decide mute/volume/DSP per user on the next audio callback.

## Plugin Audio Callback

`mumble_onAudioSourceFetched` is called for every decoded audio frame.
The decision tree:

```
Is user known? (identity mapping)
  └─ Yes:
       Is local player alive + conscious?
         └─ No  → mute
         └─ Yes:
              Is remote transmitting on radio?
                └─ Yes:
                     Same frequency? In range?
                       └─ Yes → apply radio DSP, pass
                       └─ No  → mute
                └─ No:
                     Is remote transmitting locally?
                       └─ Yes:
                            Distance ≤ 50m?
                              └─ Yes → apply volume falloff, pass
                              └─ No  → mute
                       └─ No  → mute
  └─ No → pass through (don't mute unknown users)
```

## Phase Roadmap

| Phase | Feature |
|-------|---------|
| 1 | Proximity voice (MumbleLink + plugin skeleton) |
| 2 | Radio frequencies, PTT, basic DSP effect |
| 3 | SR/LR radios, channels, interference, vehicles, death |
| 4 | UI polish, faction presets, Zeus integration |

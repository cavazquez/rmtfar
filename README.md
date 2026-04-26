# RMTFAR — Radio Mumble Task Force Arma Radio

Open source TFAR-style radio mod for Arma 3 using Mumble/Murmur instead of TeamSpeak.

**Full Rust stack. GPLv3 licensed.**

---

## What is this?

RMTFAR replicates the core experience of [Task Force Arrowhead Radio (TFAR)](https://github.com/michail-nikolaev/task-force-arma-3-radio) — directional proximity voice, radio channels, frequencies, and transmission effects — but built entirely on [Mumble](https://www.mumble.info/) and [Murmur](https://www.mumble.info/documentation/mumble-server/) as the voice transport.

No TeamSpeak. No Discord. No unofficial APIs. Just Mumble.

---

## Architecture

The system is split into three independent components:

```
┌──────────────────────────────┐
│  Arma 3 Client               │
│  ┌──────────────────────┐    │
│  │ SQF Scripts (@rmtfar)│    │
│  │  getPos, getDir, PTT │    │
│  └──────────┬───────────┘    │
│             │ callExtension  │
│  ┌──────────▼───────────┐    │
│  │ Extension DLL (Rust) │    │
│  │  rmtfar_x64.dll      │    │
│  └──────────┬───────────┘    │
└─────────────┼────────────────┘
              │ UDP :9500 (localhost)
┌─────────────▼────────────────┐
│  RMTFAR Bridge (Rust)        │
│  - Receives player state     │
│  - Writes Mumble Link shm    │
│  - Broadcasts radio state    │
│    to plugin via UDP :9501   │
└──────┬───────────────────────┘
       │
       ├─── SharedMem "MumbleLink" ──────────────┐
       │                                          │
       └─── UDP :9501 ────────────────────────────┤
                                                  │
┌─────────────────────────────────────────────────▼──┐
│  Mumble Client                                      │
│  ┌──────────────────────────────────────────────┐  │
│  │ RMTFAR Plugin (Rust, via rust-mumble-sys)     │  │
│  │  - Reads MumbleLink (positional audio)        │  │
│  │  - Reads radio state from bridge              │  │
│  │  - Audio callbacks: mute/unmute per user      │  │
│  │  - DSP: bandpass filter + noise (radio FX)   │  │
│  └──────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────┘
```

| Component | Lives in | Language | Role |
|-----------|----------|----------|------|
| `@rmtfar` Arma mod | Arma 3 | SQF + Rust DLL | Capture and send player state |
| Bridge | Local machine | Rust | Translate game state → Mumble data |
| Mumble Plugin | Mumble client | Rust | Process audio per user |

---

## Phases

### Phase 1 — Proximity Voice (current)
- Arma 3 sends position and direction to the local bridge via UDP
- Bridge writes to Mumble Link shared memory
- Mumble applies positional (directional + distance) audio automatically
- No radios yet

### Phase 2 — Simple Radio
- Separate push-to-talk for radio
- Single frequency per player
- Players only hear each other if on the same frequency
- Basic radio DSP effect (bandpass filter + soft clip + noise)

### Phase 3 — TFAR-style Logic
- Short range (SR) and long range (LR) radios with different reach
- Multiple channels per frequency
- Signal interference by distance
- Vehicle radios
- Dead / unconscious state handling
- Faction frequency presets (BLUFOR, OPFOR, INDEP)

---

## Message Protocol

### Arma 3 → Bridge (UDP :9500)

```json
{
  "v": 1,
  "type": "player_state",
  "steam_id": "76561198000000000",
  "server_id": "192.168.1.100:2302",
  "tick": 123456,
  "pos": [1234.5, 567.8, 12.3],
  "dir": 145.0,
  "alive": true,
  "conscious": true,
  "vehicle": "",
  "ptt_local": false,
  "ptt_radio_sr": false,
  "ptt_radio_lr": false,
  "radio_sr": {
    "freq": "152.000",
    "channel": 1,
    "volume": 1.0,
    "enabled": true
  },
  "radio_lr": null
}
```

### Bridge → Plugin (UDP :9501)

```json
{
  "v": 1,
  "type": "radio_state",
  "server_id": "192.168.1.100:2302",
  "tick": 123456,
  "local_player": "76561198000000000",
  "players": [
    {
      "steam_id": "76561198000000000",
      "pos": [1234.5, 567.8, 12.3],
      "dir": 145.0,
      "alive": true,
      "transmitting_local": false,
      "transmitting_radio": true,
      "radio_freq": "152.000"
    }
  ]
}
```

---

## Repository Structure

```
rmtfar/
├── Cargo.toml                   # Rust workspace
├── LICENSE                      # GPLv3
├── README.md
├── docs/
│   ├── architecture.md
│   ├── protocol.md
│   ├── building.md
│   └── setup-guide.md
├── crates/
│   ├── rmtfar-protocol/         # Shared types (PlayerState, etc.)
│   ├── rmtfar-extension/        # Arma 3 extension DLL (cdylib)
│   ├── rmtfar-bridge/           # Local bridge process
│   ├── rmtfar-plugin/           # Mumble plugin (cdylib)
│   └── rmtfar-test-client/      # Simulator for testing without Arma
├── arma-mod/
│   └── @rmtfar/                 # Arma 3 mod files + prebuilt DLL
└── scripts/
    ├── build-all.sh
    ├── build-extension.sh       # Cross-compile to Windows
    └── package-release.sh
```

---

## Dependencies

### Rust
- `serde` + `serde_json` — serialization
- `anyhow` — error handling
- [`mumble-link`](https://crates.io/crates/mumble-link) — Mumble Link shared memory helper
- [`rust-mumble-sys`](https://github.com/Dessix/rust-mumble-sys) — Mumble plugin C API bindings
- `dasp` — DSP for radio audio effects
- `windows` — Windows shared memory (bridge)

### Arma 3
- [CBA_A3](https://github.com/CBATeam/CBA_A3) — recommended for keybinds and settings
- [ACE3](https://github.com/acemod/ACE3) — optional, unconscious state integration

### Voice Server
- [Mumble](https://www.mumble.info/) 1.4.0+
- [Murmur](https://www.mumble.info/documentation/mumble-server/) (any recent version)

---

## Similar Projects

| Project | Voice Backend | Language |
|---------|--------------|----------|
| [TFAR](https://github.com/michail-nikolaev/task-force-arma-3-radio) | TeamSpeak 3 | C++/SQF |
| [ACRE2](https://github.com/IDI-Systems/acre2) | TeamSpeak 3 | C++/SQF |
| [FGCom-mumble](https://github.com/hbeni/fgcom-mumble) | Mumble | C++/Lua |
| **RMTFAR** | **Mumble** | **Rust/SQF** |

---

## License

GPLv3 — see [LICENSE](LICENSE).

Compatible with Mumble (GPLv3) and ACRE2 (GPLv3).
Does not use Bohemia Interactive assets, so APL-SA does not apply.

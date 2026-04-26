# 📻 RMTFAR — Radio Mumble Task Force Arma Radio

> Open-source TFAR-style radio mod for Arma 3, powered by Mumble/Murmur instead of TeamSpeak.

<div align="center">

[![Rust](https://img.shields.io/badge/Rust-1.75+-f74c00?logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](LICENSE)
[![Mumble](https://img.shields.io/badge/Mumble-1.4.0+-darkgreen?logo=mumble&logoColor=white)](https://www.mumble.info/)
[![Arma 3](https://img.shields.io/badge/Arma_3-SQF-8B5C14)](https://store.steampowered.com/app/107410/Arma_3/)

</div>

---

## 🛠️ Tech Stack

| | Tecnología | Rol |
|---|---|---|
| 🦀 | **Rust** | Extension DLL, bridge y plugin de Mumble |
| 🎮 | **SQF** | Scripts dentro de Arma 3 |
| 🎙️ | **Mumble 1.4.0+** | Transporte de voz (cliente) |
| 🖥️ | **Murmur** | Servidor de voz |
| 📦 | **serde / serde_json** | Serialización del protocolo |
| 🔊 | **dasp** | DSP: filtro bandpass, soft-clip y ruido de radio |
| 🧵 | **UDP** | Comunicación local entre componentes |
| 🧠 | **MumbleLink** | Shared memory para audio posicional |
| ⚙️ | **C FFI** | Bindings al API de plugin de Mumble (1.4.0) |
| 🔒 | **GPLv3** | Licencia compatible con Mumble y ACRE2 |

---

## 🗺️ Arquitectura

```
┌──────────────────────────────┐
│  🎮 Arma 3 Client            │
│  ┌──────────────────────┐    │
│  │ SQF Scripts (@rmtfar)│    │
│  │  getPos, getDir, PTT │    │
│  └──────────┬───────────┘    │
│             │ callExtension  │
│  ┌──────────▼───────────┐    │
│  │ 🦀 Extension DLL     │    │
│  │   rmtfar_x64.dll     │    │
│  └──────────┬───────────┘    │
└─────────────┼────────────────┘
              │ UDP :9500 (localhost)
┌─────────────▼────────────────┐
│  🦀 RMTFAR Bridge            │
│  - Recibe estado del jugador │
│  - Escribe MumbleLink (shm)  │
│  - Broadcast radio → :9501   │
└──────┬───────────────────────┘
       │
       ├─── 🧠 SharedMem "MumbleLink" ──────────┐
       │                                         │
       └─── 📡 UDP :9501 ───────────────────────┤
                                                 │
┌────────────────────────────────────────────────▼──┐
│  🎙️ Mumble Client                                  │
│  ┌──────────────────────────────────────────────┐  │
│  │ 🦀 RMTFAR Plugin (Rust + C FFI)              │  │
│  │  - Lee MumbleLink (audio posicional)         │  │
│  │  - Recibe radio state del bridge             │  │
│  │  - Audio callbacks: mute/unmute por usuario  │  │
│  │  - 🔊 DSP: bandpass + soft-clip + ruido      │  │
│  └──────────────────────────────────────────────┘  │
└───────────────────────────────────────────────────┘
```

| Componente | Dónde vive | Lenguaje | Rol |
|---|---|---|---|
| `@rmtfar` (mod Arma) | Arma 3 | SQF + DLL Rust | Captura y envía estado del jugador |
| Bridge | Máquina local | 🦀 Rust | Traduce estado de juego → datos Mumble |
| Plugin Mumble | Cliente Mumble | 🦀 Rust + C FFI | Procesa audio por usuario |

---

## 🚀 Fases de desarrollo

### ✅ Fase 1 — Voz por proximidad
- Arma 3 envía posición y dirección al bridge vía UDP
- El bridge escribe en la shared memory MumbleLink
- Mumble aplica audio posicional (dirección + distancia) automáticamente

### ✅ Fase 2 — Radio simple
- Push-to-talk separado para radio
- Frecuencia única por jugador
- Solo escuchan entre sí los jugadores en la misma frecuencia
- Efecto DSP de radio (filtro bandpass + soft-clip + ruido)

### ✅ Fase 3 — Lógica tipo TFAR
- Radios de corto alcance (SR) y largo alcance (LR)
- Múltiples canales por frecuencia
- Interferencia de señal por distancia
- Radios de vehículo
- Estado muerto / inconsciente
- Presets de frecuencia por facción (BLUFOR, OPFOR, INDEP)

---

## 📨 Protocolo de mensajes

### 🎮 Arma 3 → Bridge (UDP :9500)

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
  "radio_sr": { "freq": "152.000", "channel": 1, "volume": 1.0, "enabled": true },
  "radio_lr": null
}
```

### 🦀 Bridge → Plugin (UDP :9501)

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

## 📁 Estructura del repositorio

```
rmtfar/
├── 📄 Cargo.toml                  # Workspace de Rust
├── 📄 LICENSE                     # GPLv3
├── 📄 README.md
├── 🔍 check.sh                    # Quality gate: fmt + clippy + tests + SQF lint
├── 📂 .github/
│   └── workflows/
│       └── dep-audit.yml          # Auditoría anual de dependencias (diciembre)
├── 📚 docs/
│   ├── architecture.md
│   ├── protocol.md
│   ├── building.md
│   └── setup-guide.md
├── 📦 crates/
│   ├── rmtfar-protocol/           # Tipos compartidos (PlayerState, RadioStateMessage…)
│   ├── rmtfar-extension/          # DLL para Arma 3 (cdylib, C ABI)
│   ├── rmtfar-bridge/             # Proceso bridge local
│   ├── rmtfar-plugin/             # Plugin de Mumble (cdylib, C FFI)
│   └── rmtfar-test-client/        # Simulador sin necesidad de Arma 3
├── 🪖 arma-mod/
│   └── @rmtfar/                   # Mod de Arma 3 + DLL precompilada
└── 🔧 scripts/
    ├── build-all.sh
    ├── build-extension.sh         # Cross-compile a Windows (x64)
    ├── build-plugin.sh
    └── package-release.sh
```

---

## 📦 Dependencias

### 🦀 Rust

| Crate | Uso |
|---|---|
| `serde` + `serde_json` | Serialización del protocolo UDP |
| `anyhow` | Manejo de errores ergonómico |
| `mumble-link` | Shared memory MumbleLink (audio posicional) |
| `rust-mumble-sys` | Bindings al API C de plugins de Mumble 1.4 |
| `dasp` | DSP: filtro bandpass, soft-clip, generación de ruido |
| `windows` | Shared memory en Windows (bridge) |
| `libc` | Shared memory en Linux/macOS (bridge) |
| `clap` | CLI del bridge y del test-client |
| `tracing` | Logging estructurado |

### 🎮 Arma 3

| Mod | Requerido | Uso |
|---|---|---|
| [CBA_A3](https://github.com/CBATeam/CBA_A3) | Recomendado | Keybinds y settings |
| [ACE3](https://github.com/acemod/ACE3) | Opcional | Estado inconsciente |

### 🎙️ Voz

| Software | Versión mínima |
|---|---|
| [Mumble](https://www.mumble.info/) | 1.4.0+ |
| [Murmur](https://www.mumble.info/documentation/mumble-server/) | Cualquier versión reciente |

---

## 🔍 Calidad de código

```bash
./check.sh        # fmt + clippy + tests + doc + SQF lint
cargo fmt --all   # Formateo automático
cargo test --workspace
```

El CI ejecuta una [auditoría de seguridad](`.github/workflows/dep-audit.yml`) automática cada **1 de diciembre**, o cuando se activa manualmente desde la pestaña Actions.

---

## 🔗 Proyectos similares

| Proyecto | Backend de voz | Lenguaje |
|---|---|---|
| [TFAR](https://github.com/michail-nikolaev/task-force-arma-3-radio) | TeamSpeak 3 | C++ / SQF |
| [ACRE2](https://github.com/IDI-Systems/acre2) | TeamSpeak 3 | C++ / SQF |
| [FGCom-mumble](https://github.com/hbeni/fgcom-mumble) | Mumble | C++ / Lua |
| **RMTFAR** | **Mumble** | **🦀 Rust / SQF** |

---

## 📜 Licencia

**GPLv3** — ver [LICENSE](LICENSE).

Compatible con Mumble (GPLv3) y ACRE2 (GPLv3).  
No utiliza assets de Bohemia Interactive, por lo que la APL-SA no aplica.

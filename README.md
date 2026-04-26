# 📻 RMTFAR — Radio Mumble Task Force Arma Radio

> Open-source TFAR-style radio mod for Arma 3, powered by Mumble/Murmur instead of TeamSpeak.

<div align="center">

[![CI](https://github.com/cavazquez/rmtfar/actions/workflows/ci.yml/badge.svg)](https://github.com/cavazquez/rmtfar/actions/workflows/ci.yml)
[![Rust](https://img.shields.io/badge/Rust-1.75+-f74c00?logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](LICENSE)
[![Mumble](https://img.shields.io/badge/Mumble-1.5.735-darkgreen?logo=mumble&logoColor=white)](https://www.mumble.info/)
[![Arma 3](https://img.shields.io/badge/Arma_3-SQF-8B5C14)](https://store.steampowered.com/app/107410/Arma_3/)

</div>

---

## 🛠️ Tech Stack

|| | Tecnología | Rol |
|---|---|---|
|| 🦀 | **Rust** | Extension DLL, bridge y plugin de Mumble |
|| 🎮 | **SQF** | Scripts dentro de Arma 3 |
|| 🎙️ | **Mumble 1.5.735** | Transporte de voz (cliente, verificado) |
|| 🖥️ | **Murmur** | Servidor de voz |
|| 📦 | **serde / serde_json** | Serialización del protocolo |
|| 🔊 | **dasp** | DSP: filtro bandpass, soft-clip y ruido de radio |
|| 🧵 | **UDP** | Comunicación local entre componentes |
|| 🧠 | **MumbleLink** | Shared memory para audio posicional |
|| ⚙️ | **C FFI** | Bindings al API de plugin de Mumble |
|| 🔒 | **GPLv3** | Licencia compatible con Mumble y ACRE2 |

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

## 🚀 Estado de desarrollo

### ✅ Milestones verificados en Linux (Mumble 1.5.735)

| Funcionalidad | Estado | Notas |
|---|---|---|
| Pipeline end-to-end sin Arma 3 | ✅ | bridge + plugin + test-client |
| Plugin carga en Mumble 1.5.735 | ✅ | API v1.0.x, `MUMBLE_FEATURE_AUDIO` |
| Mapeo de identidad session → username | ✅ | lazy registration en talking callback |
| **Voz de proximidad** — atenuación por distancia | ✅ | rango 50m, lineal |
| **Filtro de frecuencia SR** — mute si no coincide | ✅ | string match exacto |
| **Filtro de canal** — mute si distinto canal en misma freq | ✅ | u8 match |
| **Rango de radio** — mute si dist > radio_range_m | ✅ | override con `--radio-range` |
| **DSP de radio** — bandpass + soft-clip + ruido | ✅ | audible, varía con distancia |
| **Muerte** — `alive=false` bloquea todo PTT | ✅ | log: `dead — muted` |
| **Inconsciente** — `conscious=false` bloquea PTT | ✅ | log: `unconscious — muted` |
| Mute correcto (zerear buffer, return true) | ✅ | fix API Mumble |
| Extension DLL para Arma 3 | ⚠️ | Solo Windows (cross-compile pendiente) |

### 🗺️ Fases de desarrollo

| Fase | Descripción | Estado |
|---|---|---|
| **1** | Voz por proximidad (posición 3D, atenuación por distancia) | ✅ |
| **2** | Radio simple (frecuencia, canal, rango, PTT, efecto DSP, muerte) | ✅ |
| **3** | Lógica tipo TFAR (SR/LR, potencia, interferencia, vehículos) | 🔜 |

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
  "ptt_radio_sr": true,
  "ptt_radio_lr": false,
  "radio_sr": { "freq": "43.0", "channel": 1, "volume": 1.0, "enabled": true },
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
  "local_player": "Jugador2",
  "players": [
    {
      "steam_id": "cristian",
      "pos": [200.0, 0.0, 0.0],
      "dir": 0.0,
      "alive": true,
      "conscious": true,
      "transmitting_local": false,
      "transmitting_radio": true,
      "radio_type": "sr",
      "radio_freq": "43.0",
      "radio_channel": 1,
      "radio_range_m": 500.0,
      "tuned_sr_freq": "43.0",
      "tuned_sr_channel": 1,
      "tuned_lr_freq": "",
      "tuned_lr_channel": 1
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
├── 🔧 install-plugin.sh           # Compila e instala el plugin en todos los paths de Mumble
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
| `dasp` | DSP: filtro bandpass, soft-clip, generación de ruido |
| `windows` | Shared memory en Windows (bridge) |
| `libc` | Shared memory en Linux/macOS (bridge) |
| `clap` | CLI del bridge |
| `tracing` | Logging estructurado |

### 🎮 Arma 3

| Mod | Requerido | Uso |
|---|---|---|
| [CBA_A3](https://github.com/CBATeam/CBA_A3) | Recomendado | Keybinds y settings |
| [ACE3](https://github.com/acemod/ACE3) | Opcional | Estado inconsciente |

### 🎙️ Voz

| Software | Versión mínima | Versión verificada |
|---|---|---|
| [Mumble](https://www.mumble.info/) | 1.5.x | **1.5.735** ✅ |
| [Murmur](https://www.mumble.info/documentation/mumble-server/) | Cualquier reciente | — |

---

## 🐧 Cómo probar en Linux (sin Arma 3)

Guía paso a paso para verificar el sistema completo en Linux con dos instancias de Mumble.

### Requisitos

```bash
sudo apt install mumble murmur    # Mumble 1.5.735 en Ubuntu 24.04+
sudo systemctl start mumbled      # Arrancar el servidor local
```

### Paso 1 — Compilar e instalar el plugin

```bash
./install-plugin.sh
```

El script compila el plugin y lo copia a todos los paths que Mumble puede usar:
- `~/.local/share/Mumble/Mumble/Plugins/librmtfar_plugin.so` (path real, detectado desde `mumble_settings.json`)
- `~/.local/share/mumble/Plugins/rmtfar.mumble_plugin` (path estándar XDG)

> **Nota:** después de instalar, cerrá y reabrí Mumble para que tome el nuevo binario.

### Paso 2 — Abrir dos instancias de Mumble y conectarlas al servidor local

```bash
mumble &             # Instancia A — Jugador2 (tiene el plugin activo)
mumble --multiple &  # Instancia B — cristian
```

Conectá ambas a `localhost`. El nombre de usuario de cada instancia debe coincidir con el `--id` del test-client.

### Paso 3 — Arrancar el bridge

```bash
cargo run --release -p rmtfar-bridge -- --local-id "Jugador2"
```

`--local-id` fija qué jugador es el oyente local (el que tiene Mumble con el plugin).

### Paso 4 — Simular estado de jugadores

#### 🔊 Prueba de frecuencia de radio

```bash
# Jugador2 sintonizado en 43.0 (sin PTT — solo escucha)
cargo run --release -p rmtfar-test-client -- --id "Jugador2" --freq 43.0

# cristian transmite en la misma frecuencia → se escucha con efecto radio
cargo run --release -p rmtfar-test-client -- \
  --id "cristian" --freq 43.0 --ptt-radio --pos 200,0,0 --radio-range 500

# cristian en frecuencia diferente → silencio
cargo run --release -p rmtfar-test-client -- \
  --id "cristian" --freq 50.0 --ptt-radio --pos 200,0,0 --radio-range 500
```

#### 📡 Prueba de rango de radio

```bash
# Dentro del rango (200m < 500m) → DSP aplicado, audible
cargo run --release -p rmtfar-test-client -- \
  --id "cristian" --freq 43.0 --ptt-radio --pos 200,0,0 --radio-range 500

# Fuera del rango (800m > 500m) → silencio
cargo run --release -p rmtfar-test-client -- \
  --id "cristian" --freq 43.0 --ptt-radio --pos 800,0,0 --radio-range 500
```

#### 🔢 Prueba de canal

```bash
# Jugador2 en canal 1, cristian en canal 2 → silencio (misma freq, distinto canal)
cargo run --release -p rmtfar-test-client -- --id "Jugador2" --freq 43.0 --channel 1
cargo run --release -p rmtfar-test-client -- \
  --id "cristian" --freq 43.0 --channel 2 --ptt-radio --pos 200,0,0 --radio-range 500

# Ambos en canal 1 → audible
cargo run --release -p rmtfar-test-client -- \
  --id "cristian" --freq 43.0 --channel 1 --ptt-radio --pos 200,0,0 --radio-range 500
```

#### ☠️ Prueba de muerte / inconsciente

```bash
# cristian muerto → PTT bloqueado, log: "dead — muted"
cargo run --release -p rmtfar-test-client -- \
  --id "cristian" --freq 43.0 --ptt-radio --pos 200,0,0 --radio-range 500 --dead

# cristian inconsciente (ACE) → PTT bloqueado, log: "unconscious — muted"
cargo run --release -p rmtfar-test-client -- \
  --id "cristian" --freq 43.0 --ptt-radio --pos 200,0,0 --radio-range 500 --unconscious
```

#### 👂 Prueba de proximidad local

```bash
# cristian a 20m → se escucha fuerte
cargo run --release -p rmtfar-test-client -- \
  --id "cristian" --ptt-local --pos 20,0,0

# cristian a 45m → se escucha suave (volumen ~10%)
cargo run --release -p rmtfar-test-client -- \
  --id "cristian" --ptt-local --pos 45,0,0

# cristian a 60m → silencio (> 50m de rango local)
cargo run --release -p rmtfar-test-client -- \
  --id "cristian" --ptt-local --pos 60,0,0
```

### Paso 5 — Verificar logs del plugin

Los logs aparecen en el terminal donde corrés Mumble:

```
INFO  RMTFAR: registering identity user_id=37 name=cristian
DEBUG rmtfar_plugin: radio — applying DSP uid=cristian dist=200.0
DEBUG rmtfar_plugin: out of radio range — muted uid=cristian dist=800.0
DEBUG rmtfar_plugin: radio freq mismatch — muted uid=cristian sender_freq=50.0 local_freq=43.0
DEBUG rmtfar_plugin: radio channel mismatch — muted uid=cristian sender_ch=2 local_ch=1
DEBUG rmtfar_plugin: dead — muted uid=cristian
DEBUG rmtfar_plugin: unconscious — muted uid=cristian
DEBUG rmtfar_plugin: local voice uid=cristian dist=20.0 volume=0.60
DEBUG rmtfar_plugin: out of local range — muted uid=cristian dist=60.0
```

---

## 🔍 Calidad de código

```bash
./check.sh        # fmt + clippy + tests + doc + SQF lint
cargo fmt --all   # Formateo automático
cargo test --workspace
```

El CI corre en cada push: formato, clippy, tests y build del plugin/bridge para Linux.  
Una [auditoría de dependencias](.github/workflows/dep-audit.yml) se ejecuta automáticamente cada **1 de diciembre**.

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

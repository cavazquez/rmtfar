# Building RMTFAR

## Prerequisites

- Rust stable (`rustup update stable`)
- For Windows DLLs: `sudo apt install mingw-w64` + `rustup target add x86_64-pc-windows-gnu`
- For PBO packaging: `cargo install armake2`

## Quick Start — Linux (plugin + bridge + test-client)

```bash
./scripts/build-linux.sh           # release + instala plugin en Mumble
./scripts/build-linux.sh --debug   # debug (más logs de tracing)
```

Compila el plugin, el bridge y el test-client. Instala automáticamente
`librmtfar_plugin.so` en los paths que Mumble busca y genera el
`rmtfar_plugin.mumble_plugin` instalable desde la UI.

## Quick Start — Windows (cross-compile desde Linux)

```bash
RELEASE=1 ./scripts/build-windows.sh
```

Produce los tres binarios en `arma-mod/@rmtfar/`:
- `rmtfar_x64.dll` — extension para Arma 3
- `rmtfar_plugin.dll` — plugin para Mumble (Windows)
- `rmtfar-bridge.exe` — bridge para testing en Windows

## Packing the SQF mod (PBO)

```bash
./scripts/pack-pbo.sh
# Output: arma-mod/@rmtfar/addons/rmtfar.pbo
```

Requires `armake2` (`cargo install armake2`).

## Running Tests

```bash
cargo test --workspace
# Or with the quality gate (fmt + clippy + tests + doc + SQF + audit):
./check.sh
```

## Manual Integration Test (Linux)

Open three terminals:

**Terminal 1 — bridge:**
```bash
cargo run --release -p rmtfar-bridge -- --local-id Jugador2
```

**Terminal 2 — Jugador2 (escucha, sin PTT):**
```bash
cargo run --release -p rmtfar-test-client -- --id Jugador2 --freq 43.0
```

**Terminal 3 — Jugador1 (transmite):**
```bash
cargo run --release -p rmtfar-test-client -- \
  --id Jugador1 --freq 43.0 --ptt-radio --pos 200,0,0 --radio-range 500
```

See the full Linux testing guide in the [README](../README.md#-cómo-probar-en-linux-sin-arma-3).

## Release Packaging (local)

```bash
./scripts/package-release.sh
# Output: dist/rmtfar-v<version>.zip
```

CI publishes releases automatically when a `v*` tag is pushed.
See [`.github/workflows/release.yml`](../.github/workflows/release.yml).

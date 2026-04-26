# Building RMTFAR

## Prerequisites

- Rust stable (`rustup update stable`)
- For Windows DLLs: `sudo apt install mingw-w64` + `rustup target add x86_64-pc-windows-gnu`
- For PBO packaging: `cargo install armake2`

## Quick Start — Linux (bridge + plugin)

```bash
cargo build --release -p rmtfar-bridge -p rmtfar-plugin -p rmtfar-test-client
# Or use the helper script (compiles + runs tests):
RELEASE=1 ./scripts/build-all.sh
```

Binaries land in `target/release/`.

## Building the Arma 3 Extension DLL (Windows)

```bash
RELEASE=1 ./scripts/build-extension.sh
# Output: arma-mod/@rmtfar/rmtfar_x64.dll
```

## Building the Mumble Plugin DLL (Windows)

```bash
TARGET=windows RELEASE=1 ./scripts/build-plugin.sh
# Output: arma-mod/@rmtfar/rmtfar_plugin.dll
# Install to: %APPDATA%\Mumble\Plugins\
```

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

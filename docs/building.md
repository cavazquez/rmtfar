# Building RMTFAR

## Prerequisites

- Rust 1.75+ (`rustup update stable`)
- For Windows targets: `cargo install cross` + Docker

## Quick Start (Linux bridge + test client)

```bash
cargo build --release -p rmtfar-bridge
cargo build --release -p rmtfar-test-client
```

## Building the Arma 3 Extension DLL (Windows target)

```bash
# Install cross-compilation toolchain
rustup target add x86_64-pc-windows-gnu
# Or use cross (recommended, handles all deps automatically):
cargo install cross

scripts/build-extension.sh
# Output: arma-mod/@rmtfar/rmtfar_x64.dll
```

## Building the Mumble Plugin DLL (Windows target)

```bash
scripts/build-plugin.sh
# Output: target/x86_64-pc-windows-gnu/release/rmtfar_plugin.dll
# Install to: %APPDATA%\Mumble\Plugins\
```

## Running Tests

```bash
cargo test --workspace
```

## Manual Integration Test

Open three terminals:

**Terminal 1 — bridge:**
```bash
./target/release/rmtfar-bridge
```

**Terminal 2 — player A (proximity):**
```bash
./target/release/rmtfar-test-client --steam-id 111 --pos 0,0,10 --ptt-local
```

**Terminal 3 — player B (nearby):**
```bash
./target/release/rmtfar-test-client --steam-id 222 --pos 30,0,10
```

Player A and B should hear each other through Mumble positional audio.

**Radio test (3 clients):**
```bash
# Player A on freq 152.000, transmitting
./target/release/rmtfar-test-client --steam-id 111 --freq 152.000 --ptt-radio

# Player B on same freq — should hear A
./target/release/rmtfar-test-client --steam-id 222 --freq 152.000

# Player C on different freq — should NOT hear A
./target/release/rmtfar-test-client --steam-id 333 --freq 155.000
```

## Release Packaging

```bash
scripts/build-all.sh
scripts/build-extension.sh
scripts/build-plugin.sh
scripts/package-release.sh
# Output: dist/rmtfar-0.1.0.zip
```

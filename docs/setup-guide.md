# RMTFAR Setup Guide

## Requirements

- Arma 3
- [CBA_A3](https://github.com/CBATeam/CBA_A3)
- [Mumble](https://www.mumble.info/) 1.5.0+
- A Murmur (Mumble) server

## Installation

### 1. Install the Mumble Plugin

Copy `rmtfar_plugin.dll` to your Mumble plugins directory:

- **Windows:** `%APPDATA%\Mumble\Plugins\`
- **Linux:** `~/.local/share/Mumble/Mumble/Plugins/`

Restart Mumble. Go to **Configure → Plugins** and enable **RMTFAR**.

### 2. Install the Arma 3 Mod

Copy the `@rmtfar` folder to your Arma 3 directory (or use the launcher).

The `rmtfar_x64.dll` must be in the `@rmtfar` root folder.

> **BattlEye note:** The extension is not yet whitelisted. You must either
> disable BattlEye or whitelist the DLL manually. See the project README
> for the whitelist request status.

### 3. Launch

1. Open Mumble and connect to the Murmur server
2. Launch Arma 3 with `@rmtfar` and `@CBA_A3` mods enabled
3. Join a mission

> **No bridge needed.** The extension DLL communicates directly with the
> Mumble plugin via UDP on localhost (like TFAR does with TeamSpeak).

### 4. Configure Keys

In Arma 3, open **Settings → Controls → Configure Addons → RMTFAR**:

| Action | Default Key |
|--------|-------------|
| Local PTT (proximity) | Caps Lock |
| SR Radio PTT | T |

### 5. Change Frequency (in mission)

Open the debug console or use a radio action:

```sqf
["155.500", 1] call RMTFAR_fnc_setFrequency;
```

## Troubleshooting

**No audio effect / positional audio not working:**
- Make sure Mumble's positional audio is enabled: *Configure → Settings → Audio Output → Positional Audio*
- Verify both players are on the same Arma 3 server and the same Murmur server

**"Extension not loaded" message:**
- Check `rmtfar_x64.dll` is in `@rmtfar/`
- On Linux/Proton: the DLL must be accessible from the Wine prefix

**Players can't hear each other on radio:**
- Confirm both players are on the same frequency and channel
- Check the Arma 3 RPT logs for RMTFAR errors

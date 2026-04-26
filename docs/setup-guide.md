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

### 3. Configure your Mumble nickname

> **This step is critical.** The plugin matches Mumble users to Arma 3 players
> by comparing the **Mumble nickname** with the **Arma 3 profile name** (the name
> shown in the game lobby and above your character). They must be identical.

1. In Mumble, open **Configure → Settings → Personal**
2. Set **Username** to exactly your **Arma 3 profile name**
   (the name you set in the Arma 3 launcher, e.g. `Cristian`)

If they don't match the plugin will treat you as an unknown user and audio
will pass through without any radio filtering.

### 4. Launch

1. Open Mumble and connect to the Murmur server
2. Launch Arma 3 with `@rmtfar` and `@CBA_A3` mods enabled
3. Join a mission

> **No bridge needed.** The extension DLL communicates directly with the
> Mumble plugin via UDP on localhost (like TFAR does with TeamSpeak).

### 5. Configure Keys

In Arma 3, open **Settings → Controls → Configure Addons → RMTFAR**:

| Action | Default Key |
|--------|-------------|
| Local PTT (proximity) | Caps Lock |
| SR Radio PTT | T |

### 6. Change Frequency (in mission)

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

**Radio filters not working (everyone hears everyone, no radio effect):**
- Verify your Mumble username matches your Arma 3 profile name exactly (case-sensitive)
- Check that the RMTFAR plugin is enabled in Mumble: *Configure → Plugins → RMTFAR*

**Players can't hear each other on radio:**
- Confirm both players are on the same frequency and channel
- Check the Arma 3 RPT logs for RMTFAR errors

**Port 9501 already in use:**
- Only one Mumble instance can run at a time with RMTFAR
- Check that no other application is using UDP port 9501: `ss -ulnp | grep 9501`

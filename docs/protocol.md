# RMTFAR Wire Protocol

## Arma 3 → Extension DLL (`callExtension`)

SQF sends player state via `callExtension` using a pipe-delimited payload.
JSON is intentionally avoided here for performance and because SQF string
escaping is simpler with this format.

### Format

```text
v1|steam_id|server_id|tick|x|y|z|dir|alive|conscious|vehicle|ptt_local|ptt_sr|ptt_lr|sr_freq|sr_ch|lr_freq|lr_ch[|radio_los[|sr_range_m|lr_range_m]]
```

| Field count | Meaning |
|-------------|---------|
| 18 | No LOS quality (defaults to 1.0), no range override |
| 19 | Includes `radio_los` factor |
| 21 | Includes `radio_los` + `sr_range_m` + `lr_range_m` |

### Fields

| # | Field | Type | Description |
|---|-------|------|-------------|
| 0 | `v1` | literal | Protocol version tag |
| 1 | `steam_id` | string | SteamID64 of the player sending this state |
| 2 | `server_id` | string | Arma `serverName` |
| 3 | `tick` | uint64 | Mission time in ms |
| 4 | `x` | f32 | Easting (metres) |
| 5 | `y` | f32 | Northing (metres) |
| 6 | `z` | f32 | Altitude ASL (metres) |
| 7 | `dir` | f32 | Heading 0–360°, 0 = north |
| 8 | `alive` | 0/1 | Player is alive |
| 9 | `conscious` | 0/1 | Player is not ACE-unconscious |
| 10 | `vehicle` | string | `typeOf` current vehicle, `""` = on foot |
| 11 | `ptt_local` | 0/1 | Local voice PTT pressed |
| 12 | `ptt_sr` | 0/1 | SR radio PTT pressed |
| 13 | `ptt_lr` | 0/1 | LR radio PTT pressed |
| 14 | `sr_freq` | string | SR frequency, e.g. `"43.0"` |
| 15 | `sr_ch` | uint8 | SR channel 1–16 |
| 16 | `lr_freq` | string | LR frequency, `""` = no LR radio |
| 17 | `lr_ch` | uint8 | LR channel 1–16 |
| 18 | `radio_los` | f32 (opt) | LOS factor 0.0–1.0 (1 = clear line of sight) |
| 19 | `sr_range_m` | f32 (opt) | SR range override in metres (`0` = use protocol default) |
| 20 | `lr_range_m` | f32 (opt) | LR range override in metres (`0` = use protocol default) |

Pipe characters inside string fields are escaped as `\|`.

### Example (21 fields)

```text
v1|76561198000000000|Servidor Test|123456|1234.5|567.8|12.3|145.0|1|1||0|1|0|43.0|1||1|1|5000|0
```

### SQF call

```sqf
private _result = "rmtfar" callExtension ["send", [_payloadV1]];
```

---

## Extension → Plugin (UDP :9501)

After processing each `send` call the extension broadcasts a
`RadioStateMessage` JSON object over UDP loopback to port 9501.

### `RadioStateMessage`

```json
{
  "v": 1,
  "type": "radio_state",
  "server_id": "192.168.1.100:2302",
  "tick": 123456,
  "local_player": "76561198000000000",
  "players": [
    {
      "steam_id": "76561198000000001",
      "pos": [1334.5, 567.8, 12.3],
      "dir": 270.0,
      "alive": true,
      "conscious": true,
      "in_vehicle": false,
      "transmitting_local": false,
      "transmitting_radio": true,
      "radio_type": "sr",
      "radio_freq": "43.0",
      "radio_channel": 1,
      "radio_range_m": 5000.0,
      "tuned_sr_freq": "43.0",
      "tuned_sr_channel": 1,
      "tuned_lr_freq": "",
      "tuned_lr_channel": 1,
      "radio_los_quality": 1.0
    }
  ]
}
```

#### Top-level fields

| Field | Type | Description |
|-------|------|-------------|
| `v` | uint8 | Protocol version (1) |
| `type` | string | Always `"radio_state"` |
| `server_id` | string | Arma server identifier |
| `tick` | uint64 | Mission time in ms |
| `local_player` | string | SteamID64 of the local player |
| `players` | PlayerSummary[] | All known players including local |

#### `PlayerSummary` fields

| Field | Type | Description |
|-------|------|-------------|
| `steam_id` | string | SteamID64 |
| `pos` | [f32;3] | Position [x, y, z] metres ASL |
| `dir` | f32 | Heading 0–360° |
| `alive` | bool | Player is alive |
| `conscious` | bool | Player is not ACE-unconscious |
| `in_vehicle` | bool | Player is inside a vehicle |
| `transmitting_local` | bool | Sending local voice |
| `transmitting_radio` | bool | Sending radio voice |
| `radio_type` | string | `"sr"`, `"lr"`, or `""` |
| `radio_freq` | string | Active transmit frequency |
| `radio_channel` | uint8 | Active transmit channel |
| `radio_range_m` | f32 | Max range in metres |
| `tuned_sr_freq` | string | Local player's tuned SR frequency |
| `tuned_sr_channel` | uint8 | Local player's tuned SR channel |
| `tuned_lr_freq` | string | Local player's tuned LR frequency |
| `tuned_lr_channel` | uint8 | Local player's tuned LR channel |
| `radio_los_quality` | f32 | LOS factor 0.0–1.0 |

---

## Testing Protocol (Linux without Arma 3)

When testing on Linux, `rmtfar-test-client` sends the same `v1|` payloads to
`rmtfar-bridge` over UDP :9500. The bridge translates them and drives MumbleLink
and the plugin exactly as the extension DLL would in production.

```
rmtfar-test-client  →  UDP :9500  →  rmtfar-bridge  →  MumbleLink shm
                                                      →  UDP :9501  →  rmtfar-plugin
```

This path is **not used in production**. See the [setup guide](setup-guide.md)
for instructions on how to run the full Linux testing pipeline.

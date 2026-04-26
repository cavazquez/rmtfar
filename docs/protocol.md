# RMTFAR Wire Protocol

All messages are JSON over UDP on localhost. Each UDP datagram contains
exactly one complete JSON object.

## Arma 3 → Bridge (UDP :9500)

### `player_state`

Sent at 20 Hz by each Arma 3 client.

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

| Field | Type | Description |
|-------|------|-------------|
| `v` | uint8 | Protocol version (must be 1) |
| `steam_id` | string | SteamID64 |
| `server_id` | string | Arma `serverName` |
| `tick` | uint64 | Mission time in ms |
| `pos` | [f32;3] | [x, y, z] metres ASL |
| `dir` | f32 | Heading 0–360°, 0 = north |
| `alive` | bool | Player is alive |
| `conscious` | bool | Player is not ACE-unconscious |
| `vehicle` | string | typeOf current vehicle, "" = on foot |
| `ptt_local` | bool | Local voice PTT pressed |
| `ptt_radio_sr` | bool | SR radio PTT pressed |
| `ptt_radio_lr` | bool | LR radio PTT pressed |
| `radio_sr` | RadioConfig\|null | Short-range radio |
| `radio_lr` | RadioConfig\|null | Long-range radio |

### `RadioConfig`

| Field | Type | Description |
|-------|------|-------------|
| `freq` | string | Frequency e.g. `"152.000"` |
| `channel` | uint8 | Channel 1–16 |
| `volume` | f32 | 0.0–1.0 |
| `enabled` | bool | Radio turned on |

---

## Bridge → Plugin (UDP :9501)

### `radio_state`

Sent after each `player_state` update received by the bridge.

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
      "transmitting_local": false,
      "transmitting_radio": true,
      "radio_type": "sr",
      "radio_freq": "152.000",
      "radio_channel": 1,
      "radio_range_m": 5000.0
    }
  ]
}
```

| Field | Type | Description |
|-------|------|-------------|
| `local_player` | string | SteamID64 of the bridge owner |
| `players` | PlayerSummary[] | All known players |

### `PlayerSummary`

| Field | Type | Description |
|-------|------|-------------|
| `steam_id` | string | SteamID64 |
| `pos` | [f32;3] | Position metres ASL |
| `dir` | f32 | Heading |
| `transmitting_local` | bool | Sending local voice |
| `transmitting_radio` | bool | Sending radio voice |
| `radio_type` | string | `"sr"`, `"lr"`, or `""` |
| `radio_freq` | string | Active frequency |
| `radio_channel` | uint8 | Active channel |
| `radio_range_m` | f32 | Max range in metres |

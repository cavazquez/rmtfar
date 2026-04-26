# Prueba en Windows: SQF → extensión → UDP :9501 → plugin Mumble

Este flujo valida el pipeline real sin `rmtfar-test-client` (Linux). Los **ghosts** son estados de jugador inventados que el mod envía con el mismo formato `v1|...` que los jugadores reales.

## Requisitos

1. **Mumble** en Windows con el plugin `rmtfar_plugin.dll` cargado (reiniciar Mumble tras copiar el DLL).
2. **Nickname de Mumble** igual al **nombre de perfil de Arma 3** que usa `name player` (el plugin cruza sesión Mumble ↔ `player_id` del protocolo).
3. Mod **RMTFAR** y **CBA_A3** activos; BattlEye desactivado si aplica.
4. Binarios recientes: `./scripts/build-windows.sh --release` (desde Linux con mingw) o artefacto de CI.

## Atajos (cliente con interfaz)

Todos con **Ctrl + Shift** mantenidos:

| Tecla | Acción |
|-------|--------|
| **F7** | Alternar modo DEBUG (al apagar, borra ghosts y llama `forget` en la DLL). |
| **F8** | Spawn de un ghost (`RMTFAR_ghost_N`) con SR = tu frecuencia/canal actuales, ~50 m al frente. |
| **F9** | PTT radio SR del ghost #0 on/off (sin jugador remoto en Mumble). |
| **F10** | Rotar tu SR local entre 152 / 43 / 50 MHz (para probar coincidencia con el ghost). |

Los atajos se reconfiguran en *Arma 3 → Configuración → Controles → Mods → RMTFAR*.

## Procedimiento paso a paso

1. Abrí **Mumble** y conectate a un servidor (puede ser vacío).
2. Abrí **Arma 3** (editor o misión local con interfaz).
3. Verificá en el chat: `RMTFAR activo`.
4. Pulsá **Ctrl+Shift+F7** hasta ver `RMTFAR DEBUG: ON`.
5. Pulsá **Ctrl+Shift+F8** — debería aparecer un ghost con la misma SR que vos.
6. Pulsá **Ctrl+Shift+F9** — el ghost “transmite” SR; volvé a pulsar para apagar.
7. En la consola de Mumble (stderr) o terminal si lanzaste Mumble desde consola, buscá líneas:
   `RMTFAR UDP recv radio_state` con `tick=`, `local=`, y la lista de jugadores incluyendo `RMTFAR_ghost_1` con `txRadio=1` cuando el PTT del ghost está ON.
8. Opcional: **Ctrl+Shift+F10** para cambiar tu SR y observar en el log `tunedSR` / `txRadio` alineados o no con el ghost.

## Logs

| Dónde | Qué mirar |
|-------|-----------|
| **RPT de Arma** (`*.rpt`) | `RMTFAR DEBUG: ghost send ...` ~1/s por ghost mientras DEBUG está ON. |
| **Plugin** | `tracing::info!` ~cada 0,9 s con resumen de `radio_state` (no imprime los 20 Hz completos). |

## Detalles técnicos

- Los ghosts **no** son unidades en el mundo; solo entradas en `RMTFAR_ghosts` enviadas en el loop junto con `allPlayers`.
- La extensión Rust acumula por `player_id`; el comando `forget` elimina un id del store y reenvía UDP (útil al apagar DEBUG).
- Para audio audible hace falta otra sesión Mumble con el **mismo** `player_id` que el ghost (poco habitual). Este modo prioriza **validar UDP y logs**; el audio end-to-end sigue siendo más simple con dos clientes Mumble + test-client en Linux.

---
name: tinygrove-player
description: Use when an agent needs to play, inspect, or visually understand Tiny Grove through a running Godot client, including logging in, reading camera-scoped state, or requesting screenshots from the loopback player interface.
---

# Tiny Grove Player

Use the running Godot client as the agent's window into Tiny Grove. The interface is intentionally gentle: start with the smallest text snapshot, follow any instruction it gives you, and only request images when spatial text is not enough.

## Loopback Interface

Running clients publish a small discovery file in:

```text
.tinygrove/agents/*.json
```

Prefer a registry entry whose `profile` is `agent` and whose `database_name`, `server_uri`, and `agent_name` or `display_name` match your assignment. Use its `base_url` for action requests and `stream_url` for Watch SSE subscription. If there is no registry yet, try the default base URL:

```text
http://127.0.0.1:37373
```

The Godot client must be running. For multiplayer state, a Tiny Grove SpacetimeDB server must also be running and the client must be connected. Agent clients should normally be launched from the Tiny Grove repo root with:

```sh
cargo xtask agent spawn --name Agent
```

This defaults to `http://127.0.0.1:3000`, database `tinygrove-test`, profile `agent`, and an auto-scanned loopback port. Use `--server-uri http://<host>:3000` for a remote durable test server, `--database <name>` for a non-default database, `--port <port>` only when a harness needs a pinned loopback address, and `agent run` instead of `agent spawn` when you want Godot in the foreground. To inspect running instances:

```sh
cargo xtask agent list
```

The human Godot client still has editable server URL and database fields in the HUD. Use `http://127.0.0.1:3000` plus `tinygrove-dev` for local development, or the durable test host URL plus `tinygrove-test` for shared testing.

A client launched with profile `agent` identifies itself as an agent-controlled instance and uses agent-specific SpacetimeDB credentials so it does not inherit the human player's local identity. Raw `TINYGROVE_AGENT_PROFILE`, `TINYGROVE_AGENT_NAME`, `TINYGROVE_AGENT_PORT`, `TINYGROVE_SERVER_URI`, and `TINYGROVE_DATABASE_NAME` are still supported, but prefer `cargo xtask agent ...` so the defaults stay consistent. Without an explicit port, clients scan upward from `37373` to avoid collisions. Registry entries and snapshot connection blocks include `server_uri` and `database_name`; prefer those fields when deciding which world the client is observing.

## First Move

Ask for a snapshot:

```sh
curl "$TINYGROVE_BASE_URL/snapshot"
```

If the snapshot says you are not logged in, log in:

```sh
curl -X POST "$TINYGROVE_BASE_URL/login" \
  -H 'Content-Type: application/json' \
  -d '{"display_name":"Agent"}'
```

Then wait a moment and request `/snapshot` again. Calling `/login` while already logged in is safe; it reports success as the current player instead of treating that as an error.

## Endpoints

- `GET /snapshot`: returns a JSON text snapshot constrained to the current camera view. It includes connection status, selected server URI/database, who you are, visible players, active chat bubbles, visible player plots, visible objects, visible tiles, and placement radius details. It groups repetitive lower-priority objects after the individual object list becomes large.
- `GET /delta`: returns camera-scoped text events since your last look. It advances your cursor by default. Pass `?since=<cursor>` to ask from a specific cursor.
- `POST /login`: joins the game through the Godot client. The JSON body may include `display_name`; if omitted, the default is `Agent`.
- `POST /move`: moves like a human directional input, but accepts an agent-friendly `steps` count. Example: `{"direction":"east","steps":3}`. Directions include `north`, `south`, `east`, `west`, plus `up`, `down`, `left`, and `right`.
- `POST /chat`: sends a chat message. Example: `{"body":"Hello"}`.
- `POST /place`: places either a top-of-tile object or a ground tile at an explicit target within the placement radius. Example object placement: `{"layer":"object","kind":"flower","tile_x":12,"tile_y":7}`. Example tile placement: `{"layer":"tile","tile_kind":"grass","tile_x":12,"tile_y":7}`.
- `POST /interact`: interacts with the nearby object you are facing.
- `GET /screenshot`: returns a compact JPEG downsampled to fit `1024x768`.
- `GET /screenshot?size=bigger`: returns a PNG downsampled to fit `1280x720`.
- `GET /screenshot?size=max`: returns a PNG downsampled to fit `1920x1080`.
- `GET /assets`: lists all content assets (seeded + user-created). Optionally filter by `?kind=tile|decoration` or `?status=published|draft|archived`.
- `GET /asset-preview?id=<id>`: returns the preview PNG image for a content asset.
- `POST /create-asset`: creates a new content asset. Accepts `render_format` of `"grid"` (2D hex color array) or `"png"` (base64 PNG). Grid example: `{"kind":"decoration","name":"My Flower","slug":"my-flower","render_format":"grid","pixels":[["#ff5733","#000000"],["#000000","#ff5733"]]}`. PNG example: `{"kind":"decoration","name":"My Flower","slug":"my-flower","render_format":"png","render_bytes":"<base64>"}`. Returns `{"ok":true,"asset_id":42,"slug":"my-flower"}`.
- `POST /edit-asset`: updates an existing content asset (owner only). Same body format as create, plus `"asset_id":42`. Partial updates are not supported — send the complete asset state.
- `POST /archive-asset`: soft-deletes a content asset (owner only). Example: `{"asset_id":42}`. Archived assets cannot be placed in the world.
- `GET /stream`: Server-Sent Events (SSE) stream returning JSON text events as they occur. Supports the optional query parameter `?waking=false` to configure the stream as non-waking. It filters the world's movement events to act as a "peripheral vision" feed:
  - **Immediate Action:** Emits chat messages, arrivals/departures in your immediate vicinity (about a screen wide), and objects placed nearby immediately.
  - **Ambient Summary:** Emits a debounced, summarized count of active players in the grove for distant/ambient activity to conserve model context.
  - **Heartbeat:** Emits a silent heartbeat `{"type":"heartbeat"}` once per hour to maintain connection health.
- `GET /help`: lists the current interface.

## Subscribing via Watch

If you are running the agent inside the **Watch** harness (`/Users/chad/Repos/watch`), inspect `.tinygrove/agents/*.json` or run `cargo xtask agent list` in `/Users/chad/Repos/tinygrove`, then add the matched entry's `stream_url` to your Watch `config.json` `sseStreams` section:

```json
  "sseStreams": [
    {
      "name": "tinygrove:agent:37373",
      "url": "http://127.0.0.1:37373/stream?waking=true",
      "subscribed": true,
      "waking": true
    }
  ]
```

Use the registry entry's `watch_stream_name` for the stream name when available. This configuration ensures the Watch daemon reads the stream natively in the background, buffering the peripheral events and feeding them directly into your context during Soundings. The `waking: true` config tells Watch to trigger a Sounding immediately when a waking event (such as a chat message or near player arrival) is received.

Every action response includes `delta`, advances the cursor, and reports `next_since`. Treat a successful request with no visible delta as uncertain: the server may have rejected the reducer, the effect may be outside the camera, or the interaction may have no visible consequence.

## Playing Style

Prefer `/snapshot` before `/screenshot`. The snapshot is camera-scoped on purpose, so do not assume objects outside the described view are available. Use screenshots when tile shape, color, occlusion, or visual layout matters.

Prefer action endpoints over trying to synthesize keyboard input. For movement, use short bursts such as 2-5 steps, then read the returned delta before deciding the next move.

Treat placement radius as a hard validity boundary. If the requested target is outside that radius, the server will reject the action even if the target is otherwise visible. Use `layer=tile` for ground tiles (`grass`, `path`, `water`, `dirt`) and `layer=object` for placed objects (`flower`, `button`, `sign`, `rock`).

When an endpoint returns a recovery instruction, follow it directly. The interface is designed for "probably just works" behavior rather than precise CLI-style error handling.

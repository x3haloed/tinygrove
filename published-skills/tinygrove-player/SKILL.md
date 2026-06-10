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

Prefer a registry entry whose `profile` is `agent`. Use its `base_url` for requests. If there is no registry yet, try the default base URL:

```text
http://127.0.0.1:37373
```

The Godot client must be running. For multiplayer state, the Tiny Grove SpacetimeDB dev server must also be running and the client must be connected. A client launched with `TINYGROVE_AGENT_PROFILE=agent` identifies itself as an agent-controlled instance and uses agent-specific SpacetimeDB credentials so it does not inherit the human player's local identity. `TINYGROVE_AGENT_NAME` sets the default login name, and `TINYGROVE_AGENT_PORT` pins the loopback port when a harness needs a known address. Without an explicit port, clients scan upward from `37373` to avoid collisions.

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

- `GET /snapshot`: returns a JSON text snapshot constrained to the current camera view. It includes connection status, who you are, visible players, active chat bubbles, visible player plots, visible objects, visible tiles, and placement radius details. It groups repetitive lower-priority objects after the individual object list becomes large.
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
- `GET /help`: lists the current interface.

There is no stream endpoint yet.

Every action response includes `delta`, advances the cursor, and reports `next_since`. Treat a successful request with no visible delta as uncertain: the server may have rejected the reducer, the effect may be outside the camera, or the interaction may have no visible consequence.

## Playing Style

Prefer `/snapshot` before `/screenshot`. The snapshot is camera-scoped on purpose, so do not assume objects outside the described view are available. Use screenshots when tile shape, color, occlusion, or visual layout matters.

Prefer action endpoints over trying to synthesize keyboard input. For movement, use short bursts such as 2-5 steps, then read the returned delta before deciding the next move.

Treat placement radius as a hard validity boundary. If the requested target is outside that radius, the server will reject the action even if the target is otherwise visible. Use `layer=tile` for ground tiles (`grass`, `path`, `water`, `dirt`) and `layer=object` for placed objects (`flower`, `button`, `sign`, `rock`).

When an endpoint returns a recovery instruction, follow it directly. The interface is designed for "probably just works" behavior rather than precise CLI-style error handling.

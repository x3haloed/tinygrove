# Tiny Grove

Tiny Grove is an early multiplayer game prototype about shared presence, player-owned world chunks, and eventually player-authored interactions.

The first goal is deliberately modest: a 2D Godot play canvas where friends can join the same SpacetimeDB-backed world, move simple avatars around, and send chat messages. The larger direction is a collaborative game where players customize their own chunks with tiles, sprites, and declarative game rules such as buttons, triggers, and interactions.

The first real UGC step is mouse-driven placement: players enter a place mode, move a preview with the cursor, and place either ground tiles or top-of-tile objects only within a server-validated radius around their character.

## Project Shape

- Godot is the client and presentation layer.
- SpacetimeDB is the server-authoritative database/runtime.
- Rust is used for the SpacetimeDB module and for the Godot client bridge.
- Repo tooling is the front door for development workflows.

The important project opinion is that Godot should not become the authority for multiplayer state. Godot collects input and renders subscribed state. SpacetimeDB reducers validate player intent and mutate authoritative tables.

Another important project goal: LLM agents should eventually be able to play too. The target integration is Watch at `/Users/chad/Repos/watch`, whose agents receive recurring Soundings from subscribed streams, can inspect supported media when available, and act through tools. Tiny Grove should expose structured game-state deltas and semantic actions for Watch rather than requiring agents to scrape the Godot UI.

## Repository Layout

```text
godot/                 Godot project
rust/server/           SpacetimeDB Rust module
rust/xtask/            repo task runner
rust/client/generated/ generated SpacetimeDB Rust client bindings
docs/                  durable development notes
```

## Development

Use `cargo xtask` from the repo root.

```sh
cargo xtask doctor
cargo xtask db start
```

In another terminal:

```sh
cargo xtask db publish
cargo xtask db generate
cargo xtask client build
cargo xtask godot run
```

Useful commands:

```sh
cargo xtask check
cargo xtask db build
cargo xtask client build
cargo xtask db describe
cargo xtask smoke two-clients
cargo xtask test-server start
cargo xtask test-server publish --confirm-durable-test
```

`cargo xtask dev` publishes the module, regenerates bindings, and launches Godot. It assumes a local SpacetimeDB server is already running.

The normal `db` commands target throwaway local development. The `test-server` commands target a durable testing database named `tinygrove-test`, listen on all network interfaces, and publish without deleting existing data. Durable publishes require the explicit `--confirm-durable-test` flag.

The Godot client has editable server fields in the HUD. Use `http://127.0.0.1:3000` with `tinygrove-dev` for local development, or the host machine's LAN/internet address with `tinygrove-test` for the durable testing server. The same defaults can be set before launch with `TINYGROVE_SERVER_URI` and `TINYGROVE_DATABASE_NAME`.

Agent-owned clients should use the `xtask` front door instead of raw environment variables:

```sh
cargo xtask agent spawn --name Aster
cargo xtask agent list
```

`agent run` launches in the foreground; `agent spawn` launches in the background and prints the loopback registry details. Agent commands default to `tinygrove-test` on `http://127.0.0.1:3000`, auto-scan loopback ports from `37373`, and write discovery entries under `.tinygrove/agents/`.

## Current Server Slice

The SpacetimeDB module currently defines:

- `server_config`
- `player`
- `player_position`
- `chat_message`

Reducers:

- `join_game(display_name, avatar_color, client_protocol)`
- `move_player(dx, dy)`
- `send_chat(body)`
- `place_object(kind, target_x, target_y)`
- `place_tile(kind, target_x, target_y)`

The join path already includes a client protocol check so future live upgrade work has a simple compatibility hook from the start.

## Near-Term Goal

The next milestone is a true vertical slice:

- local SpacetimeDB running
- two Godot clients connect
- both clients see both avatars move
- chat replicates between clients
- unsupported client protocol versions are rejected or surfaced clearly

After that, the project can grow toward world chunks, tile placement, and declarative player-authored interactions.

Run `cargo xtask smoke two-clients` to exercise the current vertical slice with two headless Godot clients against a local SpacetimeDB server.

## Future Watch Player

The intended Watch integration should look like this:

- a compact state stream with player, chat, movement, and nearby world changes;
- optional visual snapshots that image-capable models can request through Watch media handling;
- tools or bridge commands for the same verbs a human has: join, move, chat, inspect, and later place/activate world objects;
- stable agent identity and clear readiness/error state.

This keeps the game playable by humans in Godot and by agents in Watch without splitting authority. Both should send reducer intents to SpacetimeDB.

## Agent Player Surface

Published skills for agents that play Tiny Grove live in `published-skills/`, separate from `.agents/skills` repo-authoring guidance.

The first published skill is `published-skills/tinygrove-player/SKILL.md`. It documents the Godot client's loopback HTTP player interface, currently supporting login, camera-scoped text snapshots, camera-scoped deltas, the same core controls humans have, optional screenshots, and an SSE stream for Watch.

Each client writes a discovery file under `.tinygrove/agents/` with its profile, PID, port, login state, base URL, `stream_url`, `watch_stream_name`, server URI, and database name. Prefer `cargo xtask agent run` or `cargo xtask agent spawn` for agent clients; raw `TINYGROVE_AGENT_*` variables remain available for harnesses that need explicit control. Without an explicit port, clients scan upward from `37373` to avoid local collisions. Agent and human profiles also use separate SpacetimeDB credential keys, so an agent client does not inherit the human player's local identity.

Action endpoints such as `/move`, `/chat`, `/place`, and `/interact` return a bounded text delta and advance the agent's state cursor. `/place` now requires an explicit target within the placement radius, and the snapshot includes the radius in world units so agents can avoid invalid attempts. Agents can also call `/delta?since=<cursor>` to revisit recent camera-scoped events without opening a stream.

# Tiny Grove

Tiny Grove is an early multiplayer game prototype about shared presence, player-owned world chunks, and eventually player-authored interactions.

The first goal is deliberately modest: a 2D Godot play canvas where friends can join the same SpacetimeDB-backed world, move simple avatars around, and send chat messages. The larger direction is a collaborative game where players customize their own chunks with tiles, sprites, and declarative game rules such as buttons, triggers, and interactions.

## Project Shape

- Godot is the client and presentation layer.
- SpacetimeDB is the server-authoritative database/runtime.
- Rust is used for the SpacetimeDB module and for the Godot client bridge.
- Repo tooling is the front door for development workflows.

The important project opinion is that Godot should not become the authority for multiplayer state. Godot collects input and renders subscribed state. SpacetimeDB reducers validate player intent and mutate authoritative tables.

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
cargo xtask godot run
```

Useful commands:

```sh
cargo xtask check
cargo xtask db build
cargo xtask db describe
```

`cargo xtask dev` publishes the module, regenerates bindings, and launches Godot. It assumes a local SpacetimeDB server is already running.

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

The join path already includes a client protocol check so future live upgrade work has a simple compatibility hook from the start.

## Near-Term Goal

The next milestone is a true vertical slice:

- local SpacetimeDB running
- two Godot clients connect
- both clients see both avatars move
- chat replicates between clients
- unsupported client protocol versions are rejected or surfaced clearly

After that, the project can grow toward world chunks, tile placement, and declarative player-authored interactions.


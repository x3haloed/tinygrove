# Development

Use the repo task runner instead of invoking external tools directly.

```sh
cargo xtask doctor
cargo xtask db start
cargo xtask db publish
cargo xtask db generate
cargo xtask client build
cargo xtask godot run
```

Common commands:

```sh
cargo xtask check
cargo xtask dev
cargo xtask client build
cargo xtask db describe
cargo xtask smoke two-clients
```

`cargo xtask dev` assumes a local SpacetimeDB server is already running. Start
one in a separate terminal with `cargo xtask db start`.

The first multiplayer slice is intentionally small:

- SpacetimeDB owns player identity, position, chat, and client protocol checks.
- Godot owns presentation and input.
- Clients send reducer intents; they do not mutate game state directly.
- Generated Rust bindings belong in `rust/client/generated`.
- The Godot Rust extension is built and staged with `cargo xtask client build`.
- The repeatable local replication smoke test is `cargo xtask smoke two-clients`.

Future Watch integration:

- Watch lives at `/Users/chad/Repos/watch`.
- Watch agents perceive through subscribed Sounding streams and optional media attachments.
- Tiny Grove should eventually expose a structured game-state stream plus semantic action tools/commands for join, move, chat, inspect, and later world editing.
- Visual snapshots should be optional media references for image-capable models, not the only source of state.

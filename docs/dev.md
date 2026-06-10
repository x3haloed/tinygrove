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
cargo xtask test-server start
cargo xtask test-server publish --confirm-durable-test
```

`cargo xtask dev` assumes a local SpacetimeDB server is already running. Start
one in a separate terminal with `cargo xtask db start`.

## Durable Test Server

Use the durable test server when multiple machines should connect to the same long-lived testing world.

```sh
cargo xtask test-server start
```

This starts SpacetimeDB on `0.0.0.0:3000` with data in `.spacetime-test-data/` and database name `tinygrove-test`. Clients on the LAN can connect to `http://<host-lan-ip>:3000`; wider internet access depends on router/firewall port forwarding.

Publish durable updates with:

```sh
cargo xtask test-server publish --confirm-durable-test
```

Durable publishes run `cargo xtask check` first and do not pass the destructive local-dev delete-data option. Use additive schema changes and explicit migrations for this environment. The regular `cargo xtask db publish` command remains the fast destructive path for `tinygrove-dev`.

The Godot HUD includes server URL and database fields plus a Connect button. The same values can be provided before launch:

```sh
TINYGROVE_SERVER_URI=http://192.168.1.42:3000 TINYGROVE_DATABASE_NAME=tinygrove-test cargo xtask godot run
```

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

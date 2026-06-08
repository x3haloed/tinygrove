# Development

Use the repo task runner instead of invoking external tools directly.

```sh
cargo xtask doctor
cargo xtask db start
cargo xtask db publish
cargo xtask db generate
cargo xtask godot run
```

Common commands:

```sh
cargo xtask check
cargo xtask dev
cargo xtask db describe
```

`cargo xtask dev` assumes a local SpacetimeDB server is already running. Start
one in a separate terminal with `cargo xtask db start`.

The first multiplayer slice is intentionally small:

- SpacetimeDB owns player identity, position, chat, and client protocol checks.
- Godot owns presentation and input.
- Clients send reducer intents; they do not mutate game state directly.
- Generated Rust bindings belong in `rust/client/generated`.

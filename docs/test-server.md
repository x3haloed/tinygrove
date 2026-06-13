# Durable Test Server

Tiny Grove has two SpacetimeDB workflows:

- `cargo xtask db ...` targets throwaway local development database `tinygrove-dev`.
- `cargo xtask test-server ...` targets durable testing database `tinygrove-test`.

Start the durable test server from the repo root:

```sh
cargo xtask test-server start
```

The durable server listens on `0.0.0.0:3000` so other machines can connect to the host's LAN address. For internet play, configure router and firewall forwarding separately.

Publish server updates with:

```sh
cargo xtask test-server publish --confirm-durable-test
```

This command preserves existing data. It does not use `--delete-data=always`, and it runs `cargo xtask check` before publishing. Keep schema changes additive for this environment unless a dedicated migration or reset has been chosen intentionally.

Useful inspection commands:

```sh
cargo xtask test-server status
cargo xtask test-server describe
```

In the Godot HUD, set:

```text
Server URL: http://<host-lan-ip>:3000
Database:   tinygrove-test
```

The same values can be supplied before launch:

```sh
TINYGROVE_SERVER_URI=http://192.168.1.42:3000 TINYGROVE_DATABASE_NAME=tinygrove-test cargo xtask godot run
```

Use `http://127.0.0.1:3000` and `tinygrove-dev` for local development.

## Agent Clients

Agent-owned Godot clients should use the repo task runner instead of hand-written environment variables:

```sh
cargo xtask agent spawn --name Aster --server-uri http://127.0.0.1:3000
cargo xtask agent list
```

`agent spawn` defaults to `tinygrove-test`, profile `agent`, and an auto-scanned loopback port. Each running client writes `.tinygrove/agents/<port>.json`; use the entry's `base_url` for action endpoints and `stream_url` for Watch SSE subscription.

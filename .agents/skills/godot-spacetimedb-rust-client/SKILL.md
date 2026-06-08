---
name: godot-spacetimedb-rust-client
description: Use when wiring a Godot client in Rust to SpacetimeDB, especially binding generation, subscriptions, connection lifecycle, cache syncing, node updates, or the surprising runtime rules that make the client appear stuck or desynced.
---

# Godot + SpacetimeDB Rust Client

Use this skill when the client is Godot, Rust is still the main language, and the work is about connecting the engine to replicated database state.

## What this skill covers

- Generating and importing Rust module bindings
- Connecting the Godot client to a SpacetimeDB database
- Advancing the connection so callbacks and subscriptions actually run
- Subscribing to authoritative rows and rendering from the local cache
- Translating database changes into scene updates, spawning, despawning, and UI refreshes
- Avoiding the runtime traps that make a multiplayer client look "connected" but inert

## Default routing

Use this skill for:

- "My Godot Rust client connects, but nothing updates"
- "How do I subscribe to game state and drive nodes from it?"
- "Where should I generate bindings and how should I import them?"
- "Why do callbacks never fire?"
- "How do I keep the client rendering only replicated state?"

If the question is about schema design, reducer behavior, auth, scheduling, or server-side game rules, switch to `godot-spacetimedb-rust-core`.

## Working rules

1. Treat the replicated cache as the read model for the client.
2. Subscribe before reading or reacting to rows that may not yet be present.
3. Make connection advancement explicit wherever the runtime does not do it for you.
4. Keep scene logic and UI updates separate from authoritative game rules.
5. Prefer one-way data flow: SpacetimeDB state drives Godot nodes, not the other way around.

## High-value gotchas

- Generated bindings are module-specific; regenerate after schema changes.
- If the connection is not advanced, subscriptions and callbacks will appear dead.
- Do not read from the cache before the initial subscription has applied.
- Connection callbacks are where you persist tokens, log failures, and detect disconnects.
- The client should send intent to reducers, not directly mutate server-owned state.
- Mismatched binding paths or stale generated files often look like "compiler weirdness" but are really codegen drift.
- Godot project setup issues often masquerade as SpacetimeDB problems; confirm the client project builds before debugging networking.

## Implementation pattern

When wiring a new feature, work in this order:

1. Regenerate bindings from the current module.
2. Connect the client and handle success/error/disconnect paths.
3. Subscribe to the minimal rows needed for the scene.
4. Wait for the subscription to apply.
5. Drive nodes/UI from the local cache.
6. Send player intent back through reducers only.

## Review checklist

- Are bindings generated from the same module version the client expects?
- Is the connection advanced in the current runtime model?
- Are reads delayed until subscriptions are applied?
- Are node updates derived from replicated rows rather than ad hoc local state?
- Is token persistence and reconnect behavior handled?

## References

Read the reference files when you need exact reminders about connection behavior and binding generation:

- `references/connection.md`
- `references/codegen.md`


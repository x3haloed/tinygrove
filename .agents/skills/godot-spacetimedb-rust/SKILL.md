---
name: godot-spacetimedb-rust
description: Use when working on a Godot game whose authoritative logic and database integration are both in Rust, especially when deciding whether the task is server/game-state design or client/runtime wiring.
---

# Godot + SpacetimeDB Rust

This is the router skill for the Rust-first Godot + SpacetimeDB setup.

## Choose the right subskill

- Use `godot-spacetimedb-rust-core` for schema, reducers, schedules, auth, initialization, and game-state authority.
- Use `godot-spacetimedb-rust-client` for bindings, connections, subscriptions, cache-driven rendering, and client runtime gotchas.

## Routing rule

If the task could change server authority, data model, or reducer behavior, start with the core skill.
If the task is about "why the Godot client is not reflecting state" or "how do I get the scene to follow replicated rows," start with the client skill.


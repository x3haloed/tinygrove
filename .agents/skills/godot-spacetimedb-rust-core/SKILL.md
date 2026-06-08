---
name: godot-spacetimedb-rust-core
description: Use when building or refactoring the authoritative Rust game module for a Godot + SpacetimeDB project, especially tables, reducers, schedules, auth, persistence, or data-flow bugs where Rust should own game logic and database writes.
---

# Godot + SpacetimeDB Rust Core

Use this skill when Rust is the source of truth for game state and database mutations.

## What this skill covers

- Designing the SpacetimeDB module schema for a Godot game
- Writing reducers, scheduled reducers, and lifecycle reducers
- Choosing which game state belongs in tables vs derived client state
- Avoiding the common mistakes that break multiplayer authority, sync, or initialization
- Keeping game logic in Rust rather than leaking authority into the client

## Default routing

Use this skill for:

- "How should I model this game state in SpacetimeDB?"
- "Where should this logic live, the client or the module?"
- "Why is my reducer not safe / not deterministic / not syncing?"
- "How do I initialize or respawn state correctly?"
- "How do I avoid duplicate rows, wrong primary keys, or broken schedule tables?"

If the user is mainly asking about Godot node wiring, asset loading, or runtime integration on the client side, switch to `godot-spacetimedb-rust-client`.

## Working rules

1. Treat the Rust module as authoritative. The client should request actions, not invent state.
2. Keep persistent or queryable game facts in SpacetimeDB tables.
3. Keep derived presentation state on the client.
4. Put validation, anti-cheat, and ownership checks in reducers.
5. Prefer explicit, boring schema over clever schema. Multiplayer debugging is harder than it looks.

## High-value gotchas

- A reducer is transactional. If it fails, none of its writes should be treated as committed.
- `ctx.sender()` / sender identity is server-established, not client-supplied.
- Public tables are readable by clients, but writes still go through reducers.
- Primary keys define row identity. Changing one is delete+insert behavior, not a plain update.
- Scheduled reducers need a schedule table with the right shape and scheduled fields.
- `init` runs on first publish and on destructive republish flows; do not assume prior state exists.
- Prefer `try_insert` when you want constraint-safe setup and recoverable failure.
- Avoid storing values in the module just because the client might want to display them later. If it can be reconstructed, keep it derived.

## Implementation pattern

When shaping a new feature, work in this order:

1. Decide the authoritative entity or event.
2. Decide the table row(s) needed to persist it.
3. Decide which reducer mutates it.
4. Decide whether the state needs a scheduled reducer or lifecycle reducer.
5. Decide what the client should subscribe to and what it should only render.

## Review checklist

- Is every mutation funneled through a reducer?
- Are all rows keyed so updates do not accidentally duplicate entities?
- Is ownership enforced in the reducer, not in the client?
- Does the initialization path work for first publish and delete-data republish?
- Is the schema minimal enough to generate stable bindings?

## References

Read the reference files when you need detailed reminders about SpacetimeDB tables, reducers, binding generation, or the Godot tutorial flow:

- `references/spacetime-core.md`
- `references/godot-tutorial-notes.md`


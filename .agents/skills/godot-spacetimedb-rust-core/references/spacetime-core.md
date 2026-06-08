# SpacetimeDB core notes for Godot + Rust

Use these reminders when designing the authoritative game module.

## Module design

- Tables store persistent game state.
- Reducers are the only place writes should happen.
- Keep business rules and validation in reducers.
- Use public tables for data the client must observe; visibility does not imply client write access.

## Transaction and identity reminders

- Reducers run in a transaction.
- A failing reducer should leave no partial state behind.
- The sender identity is established by SpacetimeDB, not passed in by the client.

## Initialization and scheduling

- `init`-style reducers are for bootstrap state and can run again on destructive republish paths.
- Scheduled reducers need a schedule table with the expected fields.
- Use scheduling for simulation ticks, respawns, spawn waves, or housekeeping that should live in the authoritative module.

## Schema gotchas

- Primary keys determine row identity.
- Updating a primary key is effectively a delete plus insert.
- `try_insert` is safer when a duplicate row would be a bug, not a panic-worthy event.
- Prefer explicit row types and small tables over overloading one table with too many meanings.


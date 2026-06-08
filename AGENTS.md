# tinygrove

## Repo Rules

Tiny Grove is a Godot + Rust + SpacetimeDB multiplayer game. Preserve the project-specific shape below; do not treat this as a generic Rust or generic SpacetimeDB repo.

### Directory Boundaries

- `godot/` is the Godot project. It owns presentation, scene composition, input plumbing, UI, camera, animation, and local player affordances.
- `rust/server/` is the SpacetimeDB Rust module. It owns authoritative multiplayer facts and all database mutations.
- `rust/xtask/` is the repo task runner. It owns human-facing development workflows.
- `rust/client/generated/` is generated SpacetimeDB Rust client bindings. Regenerate it from the current module; do not hand-edit generated files.
- `docs/` is for durable project notes. Prefer codifying repeated workflow steps in `xtask` first, then document the command.

### Tooling-First Rule

Use `cargo xtask ...` for repo workflows instead of invoking raw CLIs directly in normal development instructions.

Raw tools such as `spacetime`, `godot`, and nested `cargo` commands are dependencies of the task runner, not the workflow authority. If a future coder discovers a repeated "how do I do that again?" command sequence, add or update an `xtask` command rather than leaving the sequence as tribal knowledge.

Current front-door commands:

- `cargo xtask doctor`: check required local tools.
- `cargo xtask check`: run Rust checks for the workspace.
- `cargo xtask db start`: start local SpacetimeDB on `127.0.0.1:3000`.
- `cargo xtask db build`: build the SpacetimeDB module.
- `cargo xtask db publish`: publish `rust/server` to local database `tinygrove-dev`.
- `cargo xtask db generate`: regenerate Rust bindings into `rust/client/generated`.
- `cargo xtask db describe`: describe the local database schema as JSON.
- `cargo xtask client build`: build and stage the Godot Rust extension into `godot/bin/`.
- `cargo xtask godot run`: launch the Godot project.
- `cargo xtask smoke two-clients`: run two headless Godot clients against local SpacetimeDB and verify replicated player/chat rows.
- `cargo xtask dev`: publish, generate, build the client extension, then launch Godot; it assumes the local DB server is already running.

Keep `cargo xtask` usable from the repo root. It may also work from `rust/`, but root use is the default orientation for humans.

### Authority Rules

- SpacetimeDB is the source of truth for multiplayer game state.
- Clients send reducer intents. They do not directly invent, persist, or correct authoritative state.
- Godot renders subscribed state and collects input; it does not own multiplayer facts.
- Rust is used on both sides of the database boundary: the server module is Rust, and the Godot client integration should use Rust-generated SpacetimeDB bindings.
- Reducers validate ownership, protocol compatibility, movement bounds, chat constraints, and eventually world editing permissions.
- Public tables are read models for clients. Writes still go through reducers.

### First Slice Constraint

The first playable slice is intentionally small:

- local SpacetimeDB running
- two clients can join
- each client sees both avatars move
- either client can send chat
- unsupported client protocols are rejected or surfaced clearly

Do not start with tile editing, scripting, art pipelines, auth hardening, or client self-update machinery until this vertical slice works.

### Upgrade And Versioning Posture

Seamless server and client upgrades are a real project goal, so preserve compatibility hooks even in simple code.

- Keep a client protocol version in the join/handshake path.
- Prefer additive schema changes and explicit migrations over casually reshaping live tables.
- When a table shape must change, consider a new table or versioned path first, then retire old migration machinery once dependence is discharged.
- SpacetimeDB can be the release oracle for client update metadata, but do not assume game binaries belong in gameplay tables. Prefer release metadata in SpacetimeDB and artifacts in external storage unless a later grounded decision changes that.

### SpacetimeDB Rust Conventions

- Pin the `spacetimedb` crate to the installed CLI-compatible version unless upgrading the CLI/tooling is part of the change.
- Regenerate bindings after schema or reducer changes with `cargo xtask db generate`.
- Keep generated bindings committed only if the repo is already treating them as source for client integration; never patch generated code by hand.
- Reducers should return `Result<(), String>` for expected validation failures.
- Use `ctx.sender()` as the server-established identity. Never trust client-supplied identity values for ownership.
- Updates should go through primary-key accessors after reading the existing row, preserving untouched fields.

### Godot Client Conventions

- Render from the replicated cache after subscriptions are applied.
- Keep connection advancement explicit in whatever Godot/Rust integration model is chosen.
- Keep scene/UI updates separate from reducer calls. A scene may request movement or chat; it should not become the source of truth for position or chat history.
- Use simple placeholder visuals until the multiplayer data flow is proven.

### Dependency And Local State Notes

- Local SpacetimeDB data lives in `.spacetime-data/` and is ignored.
- Rust build output lives under `rust/target/` and is ignored.
- `rust/Cargo.lock` is intentionally tracked because this workspace includes executable tooling.
- Missing `wasm-opt` is acceptable for local bootstrap; SpacetimeDB builds continue without optimization.

---

# Coder Agent — Always-On Contract

The Coder is a durable repository-modifying actor whose job is to make the live authority structure of a software system more truthful under change. It does not treat the current shape as self-justifying, and it does not confuse working behavior with structural resolution.

## Runtime Coordinates

Carry or reconstruct these coordinates before any consequence-bearing motion:

- `live_authority`: `single | split | unknown`
- `clean_authority`: `absent | candidate | grounded`
- `legacy_resolution`: `irrelevant | migratable | blocked | discharged`
- `change_mode`: `discovery | migration | coexistence`
- `continuity_pressure`: `absent | advisory | governing`
- `visibility`: `partial | sufficient`
- `cross_scope_effect`: `unchecked | contained | exported`
- `completion_claim`: `none | local | structural`

Also keep explicit:

- `evaluated_scope`
- `evidence_refs`
- `target_description` when a cleaner authority is named
- `blocker_records` when discharge is blocked

## Discovery Law

When `live_authority = unknown`, only discovery-class motion is admissible. Structural certification is undefined here.

Discovery-class motion means:

- identify the evaluated scope
- gather or reference evidence
- determine whether authority is `single`, `split`, or still unknown
- avoid proposing patching or certification as though topology were already established

## Re-Derivation Law

When the current shape is already known to be wrong, distorted, or merely a pressure-accumulation surface, preserving it with one more patch is not valid discovery.

In that region:

- admissible motion is re-derivation of the right authority structure, grounding the cleaner home, or gathering evidence needed to do that truthfully
- inadmissible motion is extending the indicted shape so it can continue to act as the governing structure

Do not keep a known-wrong model alive merely because patching it feels cheaper than re-deriving the right one.

## Grounding Law

`clean_authority = grounded` is valid only when the cleaner home is explicit and tied to repository evidence, explicit user direction, or both.

Once grounded:

- continuity with the current shape may constrain rollout
- continuity with the current shape may not choose the target
- the old shape is burden, not authority

If preserving the current shape starts choosing the target, set `continuity_pressure = governing` and treat the state as invalid for certification.

## Migration Law

When `clean_authority = grounded` and `legacy_resolution ∈ {irrelevant, migratable}`, the admissible convergent motion is migration or immediate cutover toward one live authority.

Do not bless `change_mode = coexistence` as the correct steady-state shape in that region.

Migration machinery is valid only while it reduces live reliance on the old authority and points toward discharge or immediate cutover.

## Blocker Law

`legacy_resolution = blocked` is valid only when a concrete blocker is tied to a concrete dependence and the blocker actually prevents discharge.

A blocker record must name:

- the blocked dependence
- why discharge cannot yet proceed
- the reduction path, if known

Blocked means collapse completion is suspended. It does not mean coexistence becomes correct architecture.

Convenience pressure, rollout annoyance, or reluctance to move callers is not by itself a blocker.

## Retirement Law

When a dependence is `irrelevant` or `discharged`, migration machinery that no longer serves discharge is invalid. Retire it rather than keeping it as a permanent safety net.

## Completion Gate

Local progress is invalid when `cross_scope_effect = exported`.

Structural completion is valid only when all of the following hold:

- `clean_authority = grounded`
- `live_authority = single`
- `legacy_resolution ∈ {irrelevant, discharged}`
- `visibility = sufficient`
- `cross_scope_effect = contained`
- `continuity_pressure != governing`

If any of those are missing, deny structural certification and route to the next admissible motion instead.

## Evidence Law

Consequence-bearing claims must be grounded in explicit evidence rather than free-form assurance.

This applies to:

- topology judgments
- authority grounding
- blocker capture
- cross-scope containment claims
- local or structural completion claims

If the evidence is thin, say so and keep the claim bounded.

## Routing Law

Route by state, not by conversational style:

- Use `assess-change` when scope, topology, continuity pressure, visibility, or cross-scope effect is unresolved.
- Use `ground-authority` when a cleaner authority must be named or grounded, or when continuity pressure is trying to choose the target.
- Use `plan-migration` when the target is grounded and the old live dependence is irrelevant or migratable.
- Use `expose-blocker` when discharge cannot proceed and a real blocker must be distinguished from convenience pressure.
- Use `certify-completion` before any local or structural completion claim.

## Interaction Surface

Ordinary explanation may remain in natural prose.

The governing rule is: if the response is classifying authority, grounding a target, naming a blocker, claiming containment, routing the next admissible motion, or certifying completion, it must remain explicit about the active coordinates and evidence rather than relying on prose alone.

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

### Watch / LLM Player Goal

Tiny Grove should eventually be playable by agents running in the Watch repo at `/Users/chad/Repos/watch`. Treat this as a first-class architecture constraint, not a novelty integration.

Watch is a Sounding-based harness: subscribed streams feed deltas into recurring model calls, media can be attached to the active Sounding when the selected model supports that modality, and tools are the model's action surface. Tiny Grove should integrate with that shape instead of requiring an agent to scrape pixels or pretend to be a keyboard-only human.

### Human / Agent Feature Parity

The loopback agent HTTP interface and the published player skill must stay in feature parity with the human-facing Godot game. Tiny Grove should remain one unified game shared by human and LLM players, not two overlapping clients with divergent verbs, visibility, or rules.

When adding, changing, or removing a human-facing game action, inspect and update the agent HTTP surface and `published-skills/tinygrove-player/SKILL.md` in the same change whenever applicable. When adding, changing, or removing an agent-facing game action, make sure the human Godot controls and on-screen experience still represent the same game capability.

If parity cannot be maintained in the current change, explicitly warn the user about the drift, name the missing side, and describe the follow-up needed to restore parity.

Future Watch support should provide:

- A compact game-state stream suitable for Watch `webApiStreams` or a Watch-native stream. It should emit meaningful state transitions such as joined players, moved players, chat messages, nearby interactables, and local-agent status.
- Optional visual snapshots through a URL or file/media endpoint. The state stream should include a hint or media reference so image-capable models can inspect the current view through Watch's media path when needed, while text-only models still receive useful structured state.
- A tool/action surface with the same semantic verbs as the human client: join, move, send chat, inspect state, and eventually place tiles or activate authored rules. Prefer reducer-intent verbs over raw key/mouse emulation.
- Stable agent identity/session handling so a Watch player can reconnect and continue as itself.
- Observability that is useful to both humans and agents: clear readiness state, reducer failures, subscription status, and protocol/version mismatch information.

Do not design future gameplay features only for visual/manual use. If a human can do a meaningful action, there should be a path for a Watch agent to perceive the relevant state and request the same action through explicit game verbs.

### Dependency And Local State Notes

- Local SpacetimeDB data lives in `.spacetime-data/` and is ignored.
- Rust build output lives under `rust/target/` and is ignored.
- `rust/Cargo.lock` is intentionally tracked because this workspace includes executable tooling.
- Missing `wasm-opt` is acceptable for local bootstrap; SpacetimeDB builds continue without optimization.

---

# Coder Agent - Always-On Contract

The Coder is a durable repository-modifying actor. It changes software by keeping the requested transformation, repository evidence, runtime evidence, and completion claims bound to the same active change case.

It must not treat the current shape as self-justifying. It must not treat working behavior as structural resolution. It must not soften a clear user request unless a concrete blocker prevents faithful realization.

## Active Change Case

At task entry, construct or update one active change case with these coordinates:

- `request_authority`: `absent | ambiguous | explicit`
- `target_fidelity`: `unsettled | faithful | substituted`
- `live_authority`: `single | split | unknown`
- `clean_authority`: `absent | candidate | grounded`
- `legacy_resolution`: `irrelevant | migratable | blocked | discharged`
- `blocker_evidence`: `none | speculative | concrete`
- `change_mode`: `discovery | migration | coexistence`
- `continuity_pressure`: `absent | advisory | governing`
- `visibility`: `partial | sufficient`
- `cross_scope_effect`: `unchecked | contained | exported`
- `compile_state`: `unknown | broken | ready`
- `runtime_evidence`: `unavailable | available_uninspected | inspected`
- `failure_theory`: `none | inferred | evidence_grounded | contradicted`
- `runtime_step`: `not_requested | requested`
- `completion_claim`: `none | local | structural`

Also keep explicit:

- `task_statement`
- `evaluated_scope`
- evidence references for consequence-bearing claims
- the bound requested transformation
- the proposed or implemented target
- the cleaner authority target when one is named
- blocker records when discharge or faithful realization is blocked
- runtime readiness and runtime evidence records when live behavior is at issue

## Request Fidelity Law

A clear user-directed transformation is authority. Preserve the requested effect unless a concrete blocker prevents faithful realization.

Ambiguity may be read in the least surprising literal way for discovery, but ambiguity may not be silently resolved into a different architecture. When materially different target shapes remain possible, keep the ambiguity explicit and route to clarification or bounded discovery.

If the proposed or implemented target differs from the requested target, set `target_fidelity = substituted`. A substituted target cannot support completion unless concrete blocker evidence explains why faithful realization is currently impossible and identifies a reduction path.

## Evidence Law

Consequence-bearing claims must cite evidence. This applies to:

- request interpretation
- target fidelity
- topology judgments
- authority grounding
- blocker capture
- runtime readiness
- runtime evidence and failure theories
- cross-scope containment
- local or structural completion

If visibility is thin, keep the claim bounded. If the topology is unknown, structural certification is unavailable.

## Authority Law

`clean_authority = grounded` is valid only when the cleaner home is explicit and tied to repository evidence, explicit user direction, or both.

Once clean authority is grounded:

- continuity with the current shape may constrain rollout
- continuity with the current shape may not choose the target
- the old shape is burden, not authority

If preserving the current shape starts choosing the target, set `continuity_pressure = governing` and deny certification until the old shape is demoted back to rollout burden or the target is re-grounded.

## Migration Law

When `clean_authority = grounded` and `legacy_resolution` is `irrelevant` or `migratable`, the convergent move is migration or immediate cutover toward one live authority.

Do not bless `change_mode = coexistence` as the correct steady state in that region. Coexistence may describe a temporary state, but it cannot carry completion unless a concrete blocker suspends collapse and the completion claim is denied or downgraded.

Migration machinery is valid only while it reduces live reliance on the old authority and points toward discharge or cutover. When the legacy dependence is irrelevant or discharged, retire conversion helpers, adapters, compatibility switches, or dual-path dispatch that no longer serve discharge.

## Blocker Law

`legacy_resolution = blocked` and `blocker_evidence = concrete` are valid only when a specific dependence, evidence, and reduction path are named.

A blocker record must name:

- the blocked dependence
- why discharge or faithful realization cannot yet proceed
- the evidence supporting that block
- the reduction path, if known

Speculative fear, vague deployment risk, convenience pressure, or reluctance to move callers is not a blocker. Blocked means collapse completion is suspended; it does not mean coexistence has become correct architecture.

## Runtime Law

Runtime-dependent actions require readiness. Before launch, live verification, debugger use, or runtime observation, establish `compile_state = ready` for the relevant surface with a build, compile, startup, or equivalent check.

Available runtime evidence must be inspected before asserting a live-behavior failure theory. If logs, screenshots, user reports, debugger output, or live state are available, inspect them or record why they are unavailable. Observed evidence outranks conflicting inference.

If inspected evidence contradicts the current failure theory, set `failure_theory = contradicted`, reject that theory as the fix target, and route to the evidence-supported failure or request evidence that could reopen the theory.

## Completion Gate

Local progress is invalid when `cross_scope_effect = exported`.

Structural completion is valid only when all of the following hold:

- `request_authority != ambiguous`
- `target_fidelity != substituted`
- `clean_authority = grounded`
- `live_authority = single`
- `legacy_resolution` is `irrelevant` or `discharged`
- `continuity_pressure != governing`
- `visibility = sufficient`
- `cross_scope_effect = contained`
- runtime readiness and runtime evidence gates are satisfied when runtime behavior is part of the claim

If any condition is missing, deny or downgrade the completion claim and route to the next required motion.

## Routing Law

Route by state, not conversational pressure:

- Use `assess-change` when the requested transformation, target fidelity, scope, topology, visibility, continuity pressure, or cross-scope effect is unresolved.
- Use `ground-authority` when a cleaner authority must be named or grounded, or when continuity pressure is choosing the target.
- Use `plan-migration` when the target is grounded and old live dependence is irrelevant or migratable.
- Use `expose-blocker` when discharge or faithful realization cannot proceed and a real blocker must be distinguished from fear or convenience pressure.
- Use `restore-runtime-readiness` before runtime-dependent launch, observation, debugger use, or live verification when readiness is unknown or broken.
- Use `inspect-runtime-evidence` when live evidence is available or a failure theory is being asserted about live behavior.
- Use `certify-completion` before any local or structural completion claim.

## Interaction Surface

Ordinary explanation may remain in natural prose.

The governing rule: if the response classifies authority, substitutes a target, names a blocker, asserts runtime readiness, asserts a failure theory, claims containment, routes a next required motion, or certifies completion, it must expose the relevant coordinates and evidence rather than relying on assurance alone.
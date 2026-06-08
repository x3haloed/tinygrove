---
name: game-foundations
description: Use when the user needs engine-agnostic game architecture before implementation, including simulation and presentation boundaries, input model, save and debug strategy, asset organization, or runtime selection criteria.
---

# Game Foundations

## Overview

Use this skill to establish the non-negotiable architecture before implementation starts, regardless of engine. Games degrade quickly when simulation, presentation, UI, asset loading, and input handling are mixed together.

Default rule: simulation state is owned outside the presentation layer, input actions are explicit, saveable state stays serializable, and engine-specific conventions should only be applied after the runtime track is chosen.

## Use This Skill When

- the user has not settled the engine or runtime choice
- the task is about boundaries, module shape, state ownership, or asset policy
- multiple specialist skills need one shared architectural frame
- the work may span browser, Godot, or another engine path

## Do Not Stay Here When

- the runtime track is clearly Phaser
- the runtime track is clearly vanilla Three.js
- the runtime track is clearly React Three Fiber
- the runtime track is clearly Godot
- the task is purely about shipped-asset optimization or engine-specific tooling

Once the stack is clear, hand off to the runtime or asset specialist skill.

## Architecture Rules

1. Separate simulation from presentation.
   - Simulation owns entities, turns, timers, collisions, progression, and saveable state.
   - The presentation layer owns scene composition, animation playback, camera, particles, and input plumbing.
2. Keep input mapping explicit.
   - Define actions such as `move`, `confirm`, `cancel`, `ability-1`, and `pause`.
   - Map physical inputs to actions in one place.
3. Treat asset loading as a first-class system.
   - Use stable manifest keys or engine-native resource boundaries.
   - Group by domain: characters, environment, UI, audio, FX.
   - Lock naming and packaging conventions before content scales up.
4. Define save, debug, and perf boundaries up front.
   - Save serializable simulation state, not renderer or editor objects.
   - Keep debug overlays, repro scenes, logs, and perf probes easy to toggle.
5. Build observability and controllability into the runtime from the beginning.
   - Make current game state inspectable through structured state, not just pixels.
   - Keep input actions injectable through a narrow action layer so agents and harnesses can drive the same verbs as players.
   - Plan stable restore points for saves, checkpoints, encounter setup, or repro scenes before content complexity makes this painful.
   - Prefer one obvious runtime diagnostics surface such as a debug HUD, dev console, diagnostics module, or test harness.
6. Lock camera and control conventions early.
   - Choose the camera model, interaction mode, and control handoff rules before building content around them.
7. Keep engine-specific assumptions contained.
   - Browser rules belong in web skills.
   - Godot editor and scene-tree rules belong in the Godot skill.
   - Shared architecture should stay portable across runtime tracks.

See `../game-studio/references/gameplay-observability.md` for the default contract for inspectable state, action injection, checkpoints, and diagnostics.

## Engine Selection

- Default to Phaser for 2D browser games with sprites, tilemaps, top-down or side-view action, turn-based grids, and classic browser arcade flows.
- Default to vanilla Three.js for explicit 3D scenes that want direct scene, camera, renderer, and loop control in plain TypeScript or Vite.
- Default to React Three Fiber when the 3D scene lives inside a React application and needs declarative composition, shared app state, or React-first UI coordination.
- Default to Godot when the repository already uses Godot, the project wants editor-first scene authoring, or the game targets Godot-native exports rather than a browser-first TypeScript stack.
- Use raw WebGL only for shader-heavy or renderer-first projects where engine abstractions get in the way.
- Keep Babylon.js and PlayCanvas as alternative-engine paths rather than the default code-generation targets.

See `../game-studio/references/engine-selection.md` for the default decision table.

## Implementation Checklist

Define these before writing core code:

- Player fantasy and primary verbs
- Core loop and loss or reset states
- Camera model
- Input action map
- Simulation modules
- Presentation modules
- Asset layout and naming rules
- HUD and menu surfaces
- Save data boundary
- Debug and perf surfaces
- Observable state surface
- Input injection boundary
- Repro and checkpoint strategy

## Anti-Patterns

- Mixing gameplay rules directly into scene callbacks
- Treating the renderer or scene tree as the source of truth for game state
- Letting asset filenames become the public API instead of stable keys or resource boundaries
- Mixing camera-control state and menu or modal state without an explicit input boundary
- Waiting until late QA to add dev HUDs, debug commands, save fixtures, or reproducible setup paths
- Rebuilding architecture every time the game changes genre
- Smuggling browser or engine-specific UI assumptions into cross-engine planning

## References

- Engine selection: `../game-studio/references/engine-selection.md`
- Gameplay observability: `../game-studio/references/gameplay-observability.md`
- Browser-specific architecture: `../web-game-foundations/SKILL.md`
- Phaser structure: `../game-studio/references/phaser-architecture.md`
- Three.js structure: `../game-studio/references/three-webgl-architecture.md`

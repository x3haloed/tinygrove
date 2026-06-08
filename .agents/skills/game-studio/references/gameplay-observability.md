# Gameplay Observability

Use this reference when a game needs to be playable by an agent later, not just debuggable by a human once. The goal is a runtime that exposes enough truth and control for `../gameplay/SKILL.md` and `../game-playtest/SKILL.md` to work efficiently.

## Minimum contract

Every serious game path should try to provide:

- one structured state surface
- one explicit action surface
- one reproducible restore surface
- one visible diagnostics surface

If any of these are missing, gameplay automation becomes slower, more brittle, and more dependent on guessing from pixels alone.

## Structured state surface

Expose the facts the agent would otherwise have to infer indirectly:

- current objective or mission step
- player position or room or scene identity
- HP, energy, cooldowns, status effects, inventory summary
- encounter state such as combat, dialog, menu, paused, or cutscene
- important world flags such as gate unlocked, puzzle solved, checkpoint reached

Good shapes:

- a diagnostics module the runtime can query
- a debug store exposed to the app shell
- a dev-only endpoint or console hook
- inspector-visible or autoload-backed state in Godot

Bad shapes:

- only visible on screen with no structured source
- scattered globals with no stable owner
- renderer-owned state that drifts from simulation truth

## Action surface

Route runtime control through the same verbs players conceptually use:

- `move`
- `aim`
- `confirm`
- `cancel`
- `pause`
- `interact`
- `ability-1`

Prefer one narrow action dispatcher over ad hoc event spoofing. Browser automation can still press keys, but the project should have one obvious place where those inputs become gameplay actions.

This makes it much easier to:

- drive the game deterministically
- swap keyboard, gamepad, touch, and automation without changing game rules
- record and replay small repro sequences

## Restore surface

Give the runtime a fast path back to useful states:

- save slots or save fixtures
- checkpoints
- repro scenes
- encounter loaders
- debug commands that start in a specific room or mission step

Use restore surfaces for states that are expensive to reach manually, such as:

- boss fights
- late tutorial stages
- specific puzzle configurations
- scene transitions that fail intermittently
- low-health or status-effect edge cases

## Visible diagnostics surface

The live runtime should expose enough information to make screenshots and recordings useful:

- current objective
- current mode such as combat, dialog, paused, or free roam
- important prompts and input hints
- optional dev-only state panel for player and encounter data

This does not mean covering the screen with debug UI. Keep the main play view readable and let deeper panels toggle on when needed.

## Good defaults by runtime

### Browser games

- DOM debug panel or collapsible dev HUD
- diagnostics module hanging off the app shell
- action dispatcher between physical inputs and simulation
- queryable save or checkpoint commands

### Phaser

- debug scene or DOM overlay
- `sceneBridge` exposing simulation reads and action dispatch
- save fixture or encounter loader in dev mode

### Three.js

- app-shell diagnostics module
- explicit movement and camera action boundaries
- toggleable HUD or debug drawer for player and encounter state

### React Three Fiber

- dev-facing store or diagnostics slice outside scene-local refs
- DOM debug panel tied to simulation state
- explicit action dispatch shared by UI, inputs, and automation

### Godot

- debug scene, autoload singleton, or inspector-visible diagnostics node
- reproducible entry scenes
- debug commands for room, encounter, or mission-step setup

## Anti-patterns

- adding debug info only to logs when visual state is what matters
- using screenshots as the only source of truth when structured state is available
- hiding the useful state in scene-local refs, node callbacks, or renderer objects
- creating save files manually with no supported restore workflow
- relying on 20-step manual traversal to reproduce a known bug

## Sanity check

Before calling a runtime gameplay-ready, ask:

1. Can I tell what state the game is in without guessing?
2. Can I issue one small action cleanly?
3. Can I get back to this scenario quickly?
4. Will a screenshot or short video actually show the important truth?

If any answer is no, strengthen the diagnostics or control surface before content grows further.

---
name: godot-game
description: Use when the project uses Godot and needs scene authoring, editor or runtime debugging, headless project operations, node or resource changes, or Godot-specific project maintenance.
---

# Godot Game

## Overview

Use this skill for the Godot engine path in the game-studio suite. Treat Godot as a real runtime and authoring environment, not just a pile of `.tscn` files.

Prefer the bundled headless operations script for scene creation, node edits, sprite wiring, mesh-library export, and UID maintenance. Use direct file edits only when the change is clearly safer as plain text.

## Use This Skill When

- the repository contains `project.godot`
- the user explicitly asks for Godot, GDScript, Godot C#, scenes, nodes, autoloads, or editor-driven workflows
- the task involves creating or modifying scenes, resources, nodes, or inspector-backed properties
- the task needs editor launch, debug runs, or runtime log inspection
- the task needs Godot 4.4+ UID-safe resource maintenance

## Do Not Use This Skill When

- the runtime is clearly a browser-first Phaser project
- the runtime is clearly a plain Three.js project
- the runtime is clearly React Three Fiber inside an existing React app
- the task is only about generic game planning and the engine has not been chosen yet

If the engine is still undecided, route through `../game-studio/SKILL.md` first.

## Core Rules

1. Keep engine-facing content in Godot-native structures.
   - Scenes, nodes, resources, autoloads, animations, and project settings should remain legible to Godot tools instead of being forced into foreign conventions.
2. Treat scenes as structured data, not loose text blobs.
   - Prefer headless engine operations when ownership, packed scenes, resource identity, or inspector-backed properties matter.
3. Keep gameplay architecture clean even when the editor makes mutation easy.
   - Separate game rules from presentation and scene wiring when the project grows beyond a toy prototype.
4. Respect project boundaries.
   - Use `project.godot` as the root, normalize to `res://` paths, and do not invent filesystem assumptions outside the project.
5. Verify runtime behavior through the engine when needed.
   - Launch, run, inspect logs, and capture screenshots instead of guessing from serialized scene files.
6. Keep gameplay state inspectable and reproducible.
   - Prefer debug scenes, autoload diagnostics, or inspector-visible state surfaces that make current objective, player state, and encounter state easy to inspect.
   - Add reproducible entry scenes, save fixtures, or debug commands when future gameplay testing will need to reach specific states quickly.

## Best Fit Scenarios

- extending an existing Godot game or prototype
- creating scenes or node hierarchies with the engine rather than manual text surgery
- debugging runtime errors, rendering issues, or scene transitions in a Godot project
- automating repetitive scene or resource edits headlessly
- refreshing UIDs and resaving resources safely after structural changes

## Recommended Structure

Use a repo shape that makes the engine boundaries obvious:

- `scenes/`: packed scenes and reusable scene fragments
- `scripts/`: gameplay, UI, editor, and utility scripts
- `assets/`: textures, audio, models, and imported source assets
- `autoload/` or project-level singletons: global state and services that truly need global lifetime
- `addons/`: editor plugins and third-party Godot extensions
- `tests/` or debug scenes: isolated repro scenes, harnesses, and verification hooks when the project supports them

Keep scene wiring and runtime diagnostics easy to inspect. If the game has significant systems complexity, keep gameplay state boundaries explicit instead of burying everything in node callbacks.

## Workflow

1. Locate the project root by `project.godot`.
2. Normalize in-project paths to `res://...` before handing them to Godot.
3. Use the bundled headless script for structural edits.
4. Use the editor launch/run/debug flow for runtime issues.
5. Resave resources after UID-sensitive changes on Godot 4.4+.
6. Use `../gameplay/SKILL.md` when the task requires actively progressing the runtime.
7. Validate that the playable loop behaves correctly through `../game-playtest/SKILL.md`, not just that the project launches.

## Shared Routing Notes

- For engine selection and cross-engine planning, start with `../game-studio/SKILL.md`.
- For shared architecture questions before coding, use `../game-foundations/SKILL.md`.
- Use `../web-game-foundations/SKILL.md` only when the project is truly browser-first.
- For UI, asset, and QA concerns, keep the Godot runtime conventions intact rather than importing DOM- or browser-specific assumptions automatically.
- For active runtime traversal or reproducible progression work, use `../gameplay/SKILL.md` alongside the Godot launch and diagnostics flow.

## Operation Selection

- `launch_editor`: open the editor for a project.
- `run_project`: start a debug run and capture output.
- `get_debug_output`: inspect stdout and stderr from the active run.
- `stop_project`: stop the current run and keep the final logs.
- `get_godot_version`: confirm the installed version before UID-sensitive work.
- `list_projects`: discover projects under a directory.
- `get_project_info`: gather version and project structure.
- `create_scene`: create a new scene with a root node type.
- `add_node`: add nodes and optional properties to an existing scene.
- `load_sprite`: assign a texture to a sprite-compatible node.
- `export_mesh_library`: create a `.res` MeshLibrary from a 3D scene.
- `save_scene`: save a scene or create a variant.
- `get_uid`: read a UID for a project resource on Godot 4.4+.
- `update_project_uids`: resave resources to refresh UID references.

## Path Rules

Follow the path conventions in [godot-paths.md](references/godot-paths.md). In short:

- Treat `project.godot` as the project boundary.
- Convert project-relative paths to `res://...` at the Godot boundary.
- Use `root/...` style scene node paths consistently.
- Avoid inventing paths outside the project root.

## References

Read the bundled references when needed:

- [godot-workflows.md](references/godot-workflows.md) for choosing the right operation.
- [godot-paths.md](references/godot-paths.md) for `res://`, `root/`, and project-root handling.
- [godot-uid-notes.md](references/godot-uid-notes.md) for UID behavior in Godot 4.4+.
- [godot-visual-verification.md](references/godot-visual-verification.md) for screenshot-based runtime checks.
- `../game-studio/references/gameplay-observability.md` for default diagnostics, state, action, and repro surfaces that pair well with `gameplay`.

## Bundled Script

Use [scripts/godot_operations.gd](scripts/godot_operations.gd) for deterministic project edits that need Godot to instantiate scenes, modify nodes, or save resources headlessly. Prefer it over ad hoc edits when the scene graph, packed scene, or UID state matters.

## Editing Rule

If the task is a simple text-only `.tscn` change and the structure is obvious, edit the scene file directly. If the task touches node ownership, packed scenes, texture assignment, mesh export, or resource identity, use the bundled script instead.

## Visual Verification

When debugging runtime behavior, verify the app state visually after the relevant change. A screenshot is the fastest way to confirm whether the scene, UI, or animation state matches expectations.

- Prefer a screenshot of the live Godot window when you need to confirm what actually rendered.
- Escalate to `../video-capture/SKILL.md` when the important question is about timing, motion, or sequence rather than a stable frame.
- For Codex inspection, capture to a temp file and inspect that image before making another guess.
- If capture is blocked on macOS, the issue is usually Screen Recording permission rather than the project itself.

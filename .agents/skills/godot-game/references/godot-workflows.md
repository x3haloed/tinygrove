# Godot Workflows

## Choose The Operation

Use the smallest operation that preserves Godot semantics:

- `list_projects` when you need to find valid projects.
- `get_project_info` when you need version and structure before changing anything.
- `launch_editor` when the user needs to inspect or interact in the editor.
- `run_project` when you need runtime behavior, logs, or crash reproduction.
- `create_scene`, `add_node`, `load_sprite`, `save_scene`, or `export_mesh_library` when the scene graph or packed resource must be updated by Godot.
- `get_uid` or `update_project_uids` when renamed or moved resources need UID reconciliation.

## Preferred Sequence

1. Confirm the project root.
2. Confirm the Godot version if UIDs or version-specific behavior matter.
3. Normalize all project-relative inputs.
4. Run the smallest structural edit needed.
5. Re-open, re-run, or resave only if the operation changed runtime or UID state.

## Failure Handling

- If the project path is invalid, stop and re-check `project.godot`.
- If a scene cannot be loaded, confirm the scene path and the root node path.
- If a sprite texture or resource path fails, confirm the target exists under `res://`.
- If UID-related work fails on older Godot versions, fall back to resource paths.

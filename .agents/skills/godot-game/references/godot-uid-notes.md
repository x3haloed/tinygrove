# Godot UID Notes

## When UIDs Matter

Use UID-aware workflows only on Godot 4.4 or later.

## What To Do

- Use `get_uid` to inspect a specific resource UID state.
- Use `update_project_uids` after moving or renaming resources that need identity refresh.
- Resave affected scenes or resources when references need to be regenerated.

## Expected Behavior

- A `.uid` file next to a resource indicates UID metadata exists for that resource.
- If the UID file is missing, resaving the resource usually regenerates it.
- If the project runs on an earlier Godot version, prefer resource-path references instead of UID-based workflows.

# Godot Path Rules

## Core Conventions

- Treat the directory containing `project.godot` as the project root.
- Convert project-relative inputs to `res://...` when calling Godot.
- Use absolute filesystem paths only at the outer boundary, when locating the project or invoking Godot from the host shell.
- Keep scene node paths in the form `root/...` when addressing nodes inside packed scenes.

## Common Path Transforms

- `scenes/main.tscn` -> `res://scenes/main.tscn`
- `assets/player.png` -> `res://assets/player.png`
- `root/Player/Sprite2D` stays as a scene node path.

## Practical Rules

- Do not pass `..` segments into Godot operations.
- Check that every referenced scene, texture, or resource exists before the edit.
- Create missing output directories before saving new scene variants or exports.

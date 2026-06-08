# Godot tutorial notes

Source reminders from the Godot tutorial flow.

## Project setup

- The Godot tutorial expects the .NET-capable Godot build.
- The client project needs the SpacetimeDB Godot SDK package.
- The tutorial flow uses a `Main` 2D scene and a `GameManager` script to bootstrap the client.

## Structure

- The module lives in a sibling `spacetimedb/` directory under the game project.
- Client bindings are generated into a `module_bindings/` directory inside the Godot project.
- The Godot side subscribes to the authoritative data and renders from the replicated cache.


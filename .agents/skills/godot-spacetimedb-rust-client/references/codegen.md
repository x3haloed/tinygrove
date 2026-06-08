# Binding generation reminders

## Generate bindings

Run `spacetime generate` for the Rust client bindings after the module changes.

Typical shape:

```bash
mkdir -p src/module_bindings
spacetime generate --lang rust --out-dir src/module_bindings --module-path PATH-TO-MODULE-DIRECTORY
```

## What codegen gives you

- Module-specific types that mirror tables and reducers
- Typed access to the replicated cache
- Typed reducer invocations
- Registration points for row and reducer callbacks

## Common failure mode

- If the generated bindings are stale, the client may compile against the wrong schema or appear to behave inconsistently.


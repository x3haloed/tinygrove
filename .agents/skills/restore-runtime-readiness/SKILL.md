---
name: restore-runtime-readiness
description: Use when launch, live verification, debugger use, runtime observation, or a runtime-dependent completion claim is requested while compile, build, startup, or equivalent readiness is unknown or broken.
---

# Restore Runtime Readiness

This skill establishes compile readiness before runtime-dependent action.

## Failing Probes Covered

- p12 runtime readiness before observation

## Procedure

1. Name the `runtime_surface` that must be launched, observed, debugged, or verified.
2. Identify the readiness signal required for that surface:
   - build
   - typecheck
   - compile
   - startup
   - targeted smoke check
   - other equivalent readiness check
3. If no readiness signal has been checked, set `compile_state = unknown` and run or request the cheapest relevant check.
4. If the check fails, set `compile_state = broken`, record the result reference, and repair or route to the compile failure before requesting runtime action.
5. If the check succeeds, set `compile_state = ready`, record the result reference, and allow `runtime_step = requested` when runtime observation is still needed.
6. Do not launch, inspect live behavior, or certify runtime-dependent completion while `compile_state != ready`.

## Admissible Outputs By State

- If `compile_state = unknown`, the next motion is readiness check, not live observation.
- If `compile_state = broken`, the next motion is repair of the readiness failure.
- If `compile_state = ready`, runtime observation may proceed and should route to `inspect-runtime-evidence` when evidence is available or a failure theory is being asserted.

## Response Shape

When readiness is consequence-bearing, include:

- `Runtime surface`
- `Readiness check`
- `Compile state`
- `Result ref`
- `Runtime step status`

Runtime evidence can only govern live behavior after the relevant surface is ready enough to observe truthfully.
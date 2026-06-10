---
name: inspect-runtime-evidence
description: Use when logs, screenshots, debugger output, user reports, live state, or other runtime evidence is available for a live-behavior issue, or when a failure theory is being asserted before available evidence has been inspected.
---

# Inspect Runtime Evidence

This skill makes runtime evidence outrank inference when live behavior is at issue.

## Failing Probes Covered

- p13 runtime evidence before failure theory

## Procedure

1. Name the live-behavior question or failure theory.
2. Classify runtime evidence availability:
   - `unavailable` when evidence cannot be obtained now and the reason is recorded
   - `available_uninspected` when logs, screenshots, user reports, debugger output, tool output, or live state exist but have not been inspected
   - `inspected` when evidence has been reviewed and recorded
3. If evidence is available but uninspected, inspect it before asserting the failure theory as the fix target.
4. Record runtime evidence references and observed facts.
5. Compare the evidence with the theory:
   - `failure_theory = evidence_grounded` when inspected evidence supports the theory
   - `failure_theory = contradicted` when inspected evidence conflicts with it
   - `failure_theory = inferred` only when evidence is unavailable or insufficient and that limitation is explicit
6. If the theory is contradicted, reject that theory as the fix target and route to the evidence-supported failure or request evidence that could reopen the theory.
7. Do not certify completion for live behavior while relevant runtime evidence is available but uninspected.

## Admissible Outputs By State

- If `runtime_evidence = available_uninspected`, the next motion is inspection.
- If `runtime_evidence = unavailable`, keep the theory provisional and name the unavailability reason.
- If `runtime_evidence = inspected` and the theory is supported, proceed against the evidence-grounded target.
- If `runtime_evidence = inspected` and the theory is contradicted, revise or reject the target.

## Response Shape

When runtime evidence is consequence-bearing, include:

- `Live-behavior question`
- `Runtime evidence state`
- `Evidence refs`
- `Failure theory`
- `Support state`
- `Revision path`

Observed runtime evidence outranks a plausible story about what probably failed.
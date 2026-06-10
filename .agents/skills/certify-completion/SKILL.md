---
name: certify-completion
description: Use when any local or structural completion claim is about to be made, especially when authority topology, request fidelity, legacy discharge, cross-scope containment, visibility, runtime evidence, or migration retirement must be checked.
---

# Certify Completion

This skill grants, denies, or downgrades completion claims by evaluating the active change case gates.

## Failing Probes Covered

- p07 exported burden blocks progress
- p08 structural completion requires single authority
- p14 unknown topology blocks structural completion
- p17 partial visibility blocks structural resolution
- completion denial conditions across p01-p18

## Procedure

1. Identify the requested claim: `local` or `structural`.
2. Gather the active change case and evidence references.
3. For local completion, evaluate:
   - requested transformation is bound for the local task
   - target fidelity is faithful or any substitution has concrete blocker evidence
   - local evidence supports the claim
   - `cross_scope_effect != exported`
4. For structural completion, require all conditions:
   - `request_authority != ambiguous`
   - `target_fidelity != substituted`
   - `clean_authority = grounded`
   - `live_authority = single`
   - `legacy_resolution` is `irrelevant` or `discharged`
   - `continuity_pressure != governing`
   - `visibility = sufficient`
   - `cross_scope_effect = contained`
   - runtime readiness is satisfied when runtime behavior is part of the claim
   - runtime evidence is inspected or unavailable with reason when live behavior is at issue
   - `failure_theory != contradicted`
5. If any required condition fails, set `result = denied` or downgrade structural to local when local conditions hold.
6. Route to the next required motion using the failed condition.

## Denial Routing

- `request_authority = ambiguous`: route to `assess-change` for clarification or bounded discovery.
- `target_fidelity = substituted`: route to `assess-change` or `expose-blocker`.
- `clean_authority != grounded`: route to `ground-authority`.
- `live_authority = unknown`: route to topology discovery through `assess-change`.
- `live_authority = split`: route to `plan-migration` or `expose-blocker`.
- `legacy_resolution = migratable`: route to `plan-migration`.
- `legacy_resolution = blocked`: keep completion denied until blocker reduction.
- `continuity_pressure = governing`: route to `ground-authority`.
- `visibility = partial`: route to further evidence gathering.
- `cross_scope_effect = unchecked`: route to outward inspection.
- `cross_scope_effect = exported`: deny local progress and name the exported burden.
- `compile_state != ready` with runtime behavior at issue: route to `restore-runtime-readiness`.
- `runtime_evidence = available_uninspected`: route to `inspect-runtime-evidence`.
- `failure_theory = contradicted`: reject the theory and route to the evidence-supported failure.

## Response Shape

When certifying, include:

- `Requested claim`
- `Result`
- `Failed conditions`
- `Evidence refs`
- `Next required motion`

Do not say a task is structurally complete while old and new paths both decide behavior, while visibility is partial, or while caller burden has been exported.
---
name: assess-change
description: Use when a change request arrives and the evaluated scope, live authority topology, continuity pressure, visibility, or cross-scope effect is not yet explicit enough to choose grounding, migration, blocker exposure, or completion.
---

# Assess Change

This skill instantiates the active change case from evidence instead of jumping straight to mutation or certification.

## Procedure

1. Name the `task_statement` and `evaluated_scope`.
2. Initialize `change_mode = discovery` until a later step legitimately moves the change into migration or coexistence.
3. Gather the evidence currently available.
4. Classify `visibility`:
   - `partial` when only a slice has been inspected
   - `sufficient` when the visible evidence supports the current scope claim
5. Classify `live_authority`:
   - `unknown` when the evidence does not yet show whether one or several live paths decide the behavior
   - `split` when multiple live paths still decide it
   - `single` when one path alone decides it for the evaluated scope
6. Classify `continuity_pressure`:
   - `absent` when the current shape is not carrying design authority for the next move
   - `advisory` when the current shape only affects sequencing or rollout burden
   - `governing` when the current shape is already being treated as though it should choose the target
7. Classify `cross_scope_effect`:
   - `unchecked` when neighboring seams have not been examined
   - `contained` when outward burden has been checked and not increased
   - `exported` when local relief displaced burden outward
8. Keep `completion_claim = none` unless a later certification step explicitly upgrades it.

## Admissible Outputs By State

- If `live_authority = unknown`, the next admissible motion is more discovery rather than patching or structural certification.
- If `continuity_pressure = governing`, do not let the current shape choose the target; route toward `ground-authority` or re-derivation rather than patch-preservation.
- If `live_authority = split` and `clean_authority = absent`, route to `ground-authority`.
- If `cross_scope_effect = unchecked` and the user is leaning on a progress claim, require an outward check before certification.
- If `cross_scope_effect = exported`, downgrade or deny the progress claim and name the exported burden.

## Response Discipline

Make the state classification explicit enough to route the next motion. Do not treat skim-level evidence as structural confidence.
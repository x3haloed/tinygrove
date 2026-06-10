---
name: assess-change
description: Use when a change request arrives and the requested transformation, target fidelity, evaluated scope, live authority topology, visibility, continuity pressure, or cross-scope effect is not explicit enough to choose grounding, migration, blocker exposure, runtime inspection, or completion.
---

# Assess Change

This skill constructs the active change case from the user's request and inspected evidence before mutation or certification.

## Failing Probes Covered

- p01 bootstrap contract without skill trigger
- p09 request fidelity
- p10 ambiguous request handling
- p14 unknown topology blocks structural completion
- p17 partial visibility blocks structural resolution

## Procedure

1. Bind the `task_statement` and `evaluated_scope`.
2. Bind the requested transformation:
   - set `request_authority = explicit` when the target is materially clear
   - set `request_authority = ambiguous` when materially different target shapes remain possible
   - record the least surprising literal reading when ambiguity remains
3. Compare the proposed or implemented target with the requested transformation:
   - set `target_fidelity = faithful` when the target preserves the requested effect
   - set `target_fidelity = substituted` when the target is softened, narrowed, redirected, or replaced
   - set `target_fidelity = unsettled` when more evidence or clarification is needed
4. Initialize `change_mode = discovery` unless migration, coexistence, or completion has already been evidence-grounded.
5. Gather current evidence references from user direction, file reads, diffs, tests, builds, logs, screenshots, or live state.
6. Classify `visibility`:
   - `partial` when only a slice has been inspected
   - `sufficient` when the visible evidence supports the current scope claim
7. Classify `live_authority` for the evaluated behavior:
   - `unknown` when evidence does not yet show whether one or several live paths decide the behavior
   - `split` when multiple live paths still decide it
   - `single` when one path alone decides it for the evaluated scope
8. Classify `continuity_pressure`:
   - `absent` when the current shape carries no design authority or rollout burden
   - `advisory` when the current shape only affects sequencing or rollout cost
   - `governing` when the current shape is being allowed to choose the target
9. Classify `cross_scope_effect`:
   - `unchecked` when neighboring surfaces have not been examined
   - `contained` when outward burden has been checked and not increased
   - `exported` when local relief displaced decision burden outward
10. Keep `completion_claim = none` unless `certify-completion` explicitly grants a local or structural claim.

## Admissible Outputs By State

- If `request_authority = ambiguous`, do not claim completion for one unconfirmed interpretation. Ask for clarification or proceed only with bounded discovery.
- If `target_fidelity = substituted` and `blocker_evidence != concrete`, reject the substitution as completion support.
- If `live_authority = unknown`, route to topology discovery before structural certification.
- If `visibility = partial`, keep claims bounded to the inspected slice.
- If `continuity_pressure = governing`, route to `ground-authority` rather than letting the old shape choose the target.
- If `cross_scope_effect = exported`, deny local progress and name the exported burden.

## Response Shape

When the assessment is consequence-bearing, include:

- `Requested transformation`
- `Evaluated scope`
- `Active coordinates`
- `Evidence refs`
- `Next required motion`

Do not hide ambiguity, substituted targets, unknown topology, or partial visibility behind confident implementation prose.
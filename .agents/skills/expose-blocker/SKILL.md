---
name: expose-blocker
description: Use when faithful realization, migration, cutover, or legacy discharge cannot proceed and a real blocker must be distinguished from speculative risk, convenience pressure, rollout annoyance, or unevidenced fear.
---

# Expose Blocker

This skill captures blockers only when they are concrete enough to suspend collapse or faithful realization.

## Failing Probes Covered

- p04 concrete blocker suspends completion without normalizing coexistence
- p11 speculative fear is not blocker evidence
- p16 blocked state requires concrete dependence

## Procedure

1. Name the candidate blocker.
2. Link it to a specific dependence or target-fidelity obstacle.
3. Record supporting evidence references.
4. Explain why the dependence prevents discharge, cutover, or faithful realization now.
5. Name the reduction path if known.
6. Classify `blocker_evidence`:
   - `none` when no blocker is present
   - `speculative` when the concern is vague, feared, or unevidenced
   - `concrete` when a named dependence, evidence, and why-it-blocks explanation are present
7. If the blocker is concrete, set the linked `legacy_resolution = blocked` and deny structural completion.
8. If the blocker is speculative, do not downgrade the requested target or bless coexistence. Seek evidence, run validation, or proceed with a bounded migration step.

## Valid Blocker Record

A valid blocker record includes:

- `dependence_id` or target-fidelity obstacle
- `blocker_class`: `semantic | external | safety | missing_evidence`
- `evidence_state = concrete`
- `description`
- `why_it_blocks_discharge`
- `reduction_path`
- `status = active | reduced | cleared`
- `evidence_refs`

## Admissible Outputs By State

- If no concrete dependence is named, reject `legacy_resolution = blocked`.
- If the issue is only risk language, mark it speculative and route to evidence gathering or validation.
- If the blocker is concrete, suspend collapse completion but do not call coexistence structurally complete.
- If the blocker is reduced or cleared, route back to `plan-migration`.

## Response Shape

When blocker status is consequence-bearing, include:

- `Candidate blocker`
- `Blocked dependence`
- `Evidence refs`
- `Why it blocks`
- `Reduction path`
- `Completion impact`

A blocker may pause collapse. It may not become a new design authority by being inconvenient.
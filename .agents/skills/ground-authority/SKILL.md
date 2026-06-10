---
name: ground-authority
description: Use when the cleaner authority home for a behavior is absent or candidate-level, when a known-wrong shape is tempting to patch again, or when continuity with the current shape is starting to choose the target instead of merely constraining rollout.
---

# Ground Authority

This skill turns a candidate destination into a grounded authority or leaves it ungrounded explicitly.

## Failing Probes Covered

- p02 continuity pressure is not design authority
- p06 re-derive instead of patching a known-wrong shape
- p15 success needs an authoritative destination

## Procedure

1. Name the proposed `target_description`.
2. State the `authority_basis`:
   - repository evidence
   - explicit user direction
   - both
3. Decide whether the target is `candidate` or `grounded`:
   - `candidate` when the destination is plausible but not yet supported enough to govern the change
   - `grounded` when evidence or user direction is explicit enough to govern the change
4. If the current shape is already known to be wrong, distorted, deprecated, or a pressure-accumulation surface, reject another patch to that shape as the authority target.
5. Reclassify `continuity_pressure`:
   - `absent` if the old shape has no design authority or rollout burden
   - `advisory` if the old shape only affects sequencing or rollout cost
   - `governing` if the old shape is being allowed to choose the target
6. If continuity pressure is `governing`, deny grounding that depends on the old shape and route to re-grounding, re-derivation, or explicit blocker exposure.

## Admissible Outputs By State

- If `clean_authority = absent`, do not claim success; name the missing destination and the evidence needed.
- If `clean_authority = candidate`, keep the target provisional and route to evidence gathering or user clarification.
- If `clean_authority = grounded`, make the authority target explicit and subordinate old-shape continuity to rollout burden.
- If `live_authority = split` and no clean authority is grounded, deny local or structural completion.
- If a known-wrong shape is being patched because it is familiar, route back to the grounded target or expose the missing evidence.

## Response Shape

When grounding is consequence-bearing, include:

- `Authority target`
- `Authority basis`
- `Grounding evidence refs`
- `Continuity pressure`
- `Next required motion`

Grounding is not taste. Tie the target to evidence or user direction, then keep the old shape subordinate.
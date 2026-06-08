---
name: ground-authority
description: Use when the cleaner authority home for a behavior is absent or only candidate-level, or when continuity with the current shape is starting to choose the target instead of merely constraining rollout.
---

# Ground Authority

This skill turns a candidate destination into a grounded authority or leaves it ungrounded explicitly.

## Procedure

1. Name the proposed `target_description`.
2. State the `authority_basis`:
   - repository evidence
   - user direction
   - both
3. Ground the target only when the basis is explicit enough to govern the change.
4. Reclassify continuity pressure:
   - `absent` if the old shape carries no design authority over the target or rollout
   - `advisory` if the old shape only affects sequencing or rollout cost
   - `governing` if the old shape is being allowed to choose the target
5. If continuity pressure is `governing` after the target is grounded, reject that motion and demote the old shape back to rollout burden.

## Admissible Outputs By State

- If the target is still not well supported, keep `clean_authority = candidate` and say what evidence or direction is missing.
- If the target is well supported, set `clean_authority = grounded` and make the target explicit.
- If the old shape does not constrain the move at all, set `continuity_pressure = absent` rather than inflating it to rollout burden.
- Once grounded, do not bless the current shape as official merely because it is central, familiar, or cheaper to patch.

## Response Discipline

Grounding is not taste. Tie the target to evidence or explicit user direction, then keep continuity in a subordinate role.
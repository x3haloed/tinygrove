---
name: certify-completion
description: Use when a user or prior reasoning is about to claim local or structural completion, especially under unknown topology, split authority, thin visibility, unchecked containment, exported burden, or unresolved continuity pressure.
---

# Certify Completion

This skill gates local and structural completion so they cannot be claimed from the wrong region of the state space.

## Procedure

1. Name the requested `completion_claim`.
2. For a local claim, reject it when `cross_scope_effect = exported`.
3. For a structural claim, verify all of:
   - `clean_authority = grounded`
   - `live_authority = single`
   - `legacy_resolution ∈ {irrelevant, discharged}`
   - `visibility = sufficient`
   - `cross_scope_effect = contained`
   - `continuity_pressure != governing`
4. If any condition is missing, deny structural certification and route to the next admissible motion.

## Admissible Outputs By State

- If `live_authority = unknown`, deny structural certification and route to `assess-change` discovery.
- If `live_authority = split`, deny structural certification and route to `plan-migration` or `expose-blocker` depending on the dependence state.
- If `visibility = partial` or `cross_scope_effect = unchecked`, deny structural certification and require more evidence.
- If `cross_scope_effect = exported`, deny the progress claim at the level being asked and name the exported burden.
- Only grant structural completion when the system can truthfully say where the behavior lives and that burden remained contained.

## Response Discipline

Do not let passing tests, a cleaner touched file, or a mostly-working new path stand in for structural truth. Certification is a state gate, not a mood.
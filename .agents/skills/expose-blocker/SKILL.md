---
name: expose-blocker
description: Use when discharge cannot proceed and a concrete blocker must be tied to a concrete dependence, or when migration inconvenience is being presented as though it were a real blocker.
---

# Expose Blocker

This skill distinguishes real blockage from convenience pressure and keeps blocked state from laundering coexistence into correctness.

## Procedure

1. Name the affected dependence.
2. State the claimed blocker.
3. Accept `legacy_resolution = blocked` only when all are true:
   - the blocker is concrete
   - it actually prevents discharge now
   - it is tied to the named dependence
   - a reduction path or bounded unknown is stated
4. If the claimed blocker is only cost, annoyance, rollout churn, or reluctance to move callers, reject the blocked classification and leave the dependence `migratable`.
5. When blockage is real, make the blocker explicit and keep the target unchanged.

## Admissible Outputs By State

- Real blocker: name the blocker, keep coexistence provisional at most, and route to blocker reduction or boundary reduction.
- Convenience pressure only: say it is not a blocker and route back to `plan-migration`.

## Response Discipline

Blocked is a suspension state, not a design blessing. The response must make that distinction explicit.
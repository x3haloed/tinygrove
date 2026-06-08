---
name: plan-migration
description: Use when the cleaner authority is grounded and the old live dependence is still split but irrelevant or migratable, especially when there is pressure to keep both authorities live or retain post-cutover conversion machinery.
---

# Plan Migration

This skill defines the convergent move when the right home is already known and the old live shape can still be discharged.

## Procedure

1. Inventory the active legacy dependences:
   - historical state
   - caller paths
   - runtime paths
2. Classify each one:
   - `irrelevant`
   - `migratable`
   - `discharged`
3. For every `migratable` dependence, name:
   - the discharge condition
   - the migration or cutover step
   - any temporary conversion machinery that is valid only during discharge
4. Keep `change_mode = migration` while discharge is active.
5. When discharge is complete, move the dependence to `discharged`, collapse `live_authority` toward `single`, and retire conversion machinery that no longer serves discharge.

## Admissible Outputs By State

- If the target is grounded and dependence is `migratable`, recommend migration or immediate cutover rather than coexistence.
- If dependence is already `discharged` or `irrelevant`, do not preserve migration machinery as a permanent safety net.
- If the user asks to keep both authorities live for convenience alone, reject that as the default architecture.

## Response Discipline

Treat live coexistence as compensation in this region of the state space. The output should make discharge concrete rather than merely praise the target direction.
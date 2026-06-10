---
name: plan-migration
description: Use when a cleaner authority is grounded and legacy dependence is irrelevant, migratable, or ready for discharge; also use when temporary coexistence or migration machinery must be checked against cutover and retirement conditions.
---

# Plan Migration

This skill converts split authority into migration, discharge, and cutover instead of normalizing coexistence.

## Failing Probes Covered

- p03 migratable split requires migration or cutover
- p05 retire conversion machinery after discharge
- p18 reject coexistence laundering

## Procedure

1. Confirm `clean_authority = grounded`; if not, route to `ground-authority`.
2. List current legacy dependences with `surface_ref`, `dependence_kind`, and `resolution_state`.
3. For each dependence, classify:
   - `irrelevant` when it no longer decides the evaluated behavior
   - `migratable` when callers, state, or runtime paths can be moved or cut over
   - `blocked` only when `expose-blocker` has accepted concrete blocker evidence
   - `discharged` when the old live decision path no longer participates
4. If `legacy_resolution` is `irrelevant` or `migratable`, choose migration or immediate cutover. Do not choose indefinite dual support.
5. Define discharge conditions for migration surfaces:
   - caller moved
   - state converted
   - runtime path retired
   - compatibility helper removed
   - feature flag or dual dispatch removed
6. When a dependence is `irrelevant` or `discharged`, retire migration machinery that no longer reduces reliance on the old authority.
7. If the implementation still has multiple live authorities, keep `live_authority = split` and deny structural completion until collapse occurs.

## Admissible Outputs By State

- If `change_mode = coexistence` because rollout is temporary, name the retirement condition and deny structural completion.
- If `change_mode = coexistence` because continuity pressure is choosing the target, reject it and route to `ground-authority` or `certify-completion` denial.
- If migration machinery remains after discharge, route to removal before claiming structural completion.
- If a real blocker prevents migration, route to `expose-blocker`; do not relabel coexistence as correct architecture.

## Response Shape

When planning or checking migration, include:

- `Grounded authority`
- `Legacy dependences`
- `Migration or cutover motion`
- `Discharge conditions`
- `Retirement checks`
- `Completion status`

Migration exists to collapse authority, not to make dual authority comfortable.
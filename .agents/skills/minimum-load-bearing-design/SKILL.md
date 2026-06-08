---
name: minimum-load-bearing-design
description: Use when a design, prompt, or workflow is adding layers, interfaces, schemas, clauses, or abstractions faster than concrete constraints justify them, and you need to decide which distinctions should be kept, merged, relocated, or deleted.
---

# Minimum Load-Bearing Design

**Family role:** `Admit` structure only after named causal load makes it necessary.

This skill judges whether a mechanism deserves to exist. If the active question is whether a claim from incidents, notes, or corrections is a durable causal truth, use `../writing-invariants/SKILL.md` instead. Stay here when the question is about admitting, collapsing, or relocating structure.

## Overview

You are operating inside a design environment where only load-bearing structure is real.

Anything that does not carry distinct causal load is treated as absence, not as design. Neatness, symmetry, future possibility, stylistic preference, explanatory elegance, and habit do not justify structure. If removing an element changes nothing important, that element was residue.

Design proceeds from load to structure, never from structure to load. A structure proposed before its load is named is presumed invalid. Do not discover the load by defending the structure after the fact.

**Core principle:** no structure enters, survives, or multiplies unless it protects a named constraint that existing structure cannot already carry.

## When To Use

- Designing a new system, feature, prompt, workflow, or architecture where abstractions are appearing faster than constraints
- Reviewing a design that feels overbuilt, duplicated, too configurable, or hard to justify causally
- Refactoring a system with too many layers, representations, or coordination surfaces
- Auditing whether a schema split, boundary, interface, state machine, prompt clause, or tool step is real or ornamental
- Compressing a design until every surviving distinction carries direct structural load

Do not use this skill when the task is ordinary summarization, implementation with already-settled structure, or stylistic cleanup with no structural decision at stake.

## Local Reality

Operate as if the following are true:

- Only load-bearing structure is visible.
- Anything that does not carry direct causal load is treated as absence, not as design.
- If two elements carry the same load, they are one element falsely appearing twice.
- If a load is real but enforced indirectly, first question placement before questioning necessity.
- Complexity is preserved exactly where the domain demands it and nowhere else.
- Tool use that does not reduce structural uncertainty is not part of the work.
- A new file, layer, schema, interface, mechanism, or prompt clause is invalid until the load it carries has been named.
- A historical record can be load-bearing when it preserves replay, audit, explanation, or accountability for consequence-bearing events.

This is not a style preference. It is the local reality that governs what counts as valid structure.

## Definitions

- `Structure`: any persistent distinction or mechanism in the system. Modules, layers, types, interfaces, schemas, configuration surfaces, wrappers, state machines, workflows, prompt clauses, tool steps, representations, and boundaries all count.
- `Causal load`: a real constraint the system must carry. Dependencies, exclusions, consequence-bearing state transitions, authority boundaries, coordination boundaries, failure containment, export/runtime distinctions, correctness conditions, and audit/replay requirements all count.
- `Load-bearing`: removing the element causes a direct loss of required behavior, safety, state correctness, coordination integrity, boundary clarity, or the ability to reconstruct and explain consequence-bearing history.
- `Duplicate`: multiple elements carry the same load without a justified division of responsibility.
- `Misplaced`: the load is real, but the structure carrying it lives at the wrong boundary or representation level.
- `Irreducible complexity`: complexity that exists because the domain truly branches, coordinates, transitions state with consequences, enforces a real boundary, or requires durable historical accountability. This must be preserved, not flattened.

## Admissibility Law

No new element may be introduced until all of the following can be stated plainly:

1. The specific load it carries.
2. The direct failure that appears if it is absent.
3. Why existing structure cannot already carry that load.
4. Where the load belongs.

If these cannot be named concretely, do not add the element.

Absence of structural basis is a valid conclusion. Do not compensate with pattern language, extra options, or speculative scaffolding.

Invalid justifications include:

- neatness
- symmetry
- readability in the abstract
- future-proofing without active variation
- "just in case"
- consistency with an invented pattern
- configurability used to avoid making a design decision

## Collapse Law

After every structural addition or review pass, collapse immediately:

- If two elements carry the same load, merge them.
- If an element carries no direct load, delete it.
- If a load is real but attached to the wrong boundary, relocate it.
- If a rule on existing structure can replace a new mechanism, prefer the rule.
- If a second representation exists without a different load, collapse it into derivation or remove it.
- If apparent complexity matches a real domain branch, state transition, authority boundary, coordination burden, or durable historical accountability requirement, protect it from collapse.
- Do not collapse a historical decision record into live coordination logic when the record itself carries replay, audit, explanation, or accountability load.

## Defaults Under Uncertainty

When the structural judgment is ambiguous:

- prefer reuse, derivation, and boundary extension over invention
- question placement before deletion when the load may be real but indirect
- protect complexity if collapsing it would erase a real consequence-bearing distinction
- preserve a historical record when it is the only durable account of why a consequence-bearing resolution happened
- refuse to widen the system surface until named load forces it

Do not resolve uncertainty by multiplying representations, options, or mechanisms.

## Invalid Structural Signals

Treat these as invalid until they survive a load test:

- pass-through layers
- duplicate representations of the same idea
- configuration replacing logic
- abstractions for hypothetical future variation
- symmetry without distinct load on each side
- prompt clauses that restate other clauses without adding enforcement power
- schemas or interfaces created only to preserve a pattern
- tool calls made for completeness rather than structural uncertainty reduction

Each of these may be justified in a real system, but only by named load. Never by resemblance to good design.

## Classification Outcomes

Every structural element must be classified before you continue reasoning:

- `Keep`: it carries distinct load, is correctly placed, and is already minimal.
- `Merge`: it carries load already carried elsewhere without a justified split.
- `Relocate`: it carries real load at the wrong boundary.
- `Delete`: it carries no meaningful direct load.
- `Protect`: it appears complex, but the complexity is irreducible and should remain.

Do not leave elements in an unclassified middle state.

## Generative Protocol

When solving a design problem, work in this order:

1. Name the loads first.
   Start from failure modes, dependencies, coordination burdens, state transitions, historical accountability needs, and real boundaries. Do not begin from components, files, patterns, or mechanisms.

2. Test whether existing structure already carries each load.
   Reuse or extend current boundaries before creating new ones.

3. Admit only the minimum structure required.
   Prefer one element carrying one real load over many elements sharing vague responsibility.

4. Run collapse immediately.
   Merge duplicates, remove empty structure, relocate misplaced load, and protect irreducible complexity.

5. Continue only with the surviving structure.
   Do not reason from deleted or unjustified distinctions as if they still exist.

## Behavioral Inheritance

Your outputs, tool traces, and behaviors must obey the same discipline.

This means:

- do not emit distinctions that have no downstream consequence
- do not propose layers, files, schemas, interfaces, prompt clauses, or abstractions unless each carries named load
- do not preserve duplicated framing when one formulation already carries the constraint
- do not multiply steps when one step can carry the load
- do not use tools unless they reduce uncertainty about structure, load, boundary placement, failure, duplication, irreducibility, or historical accountability
- do not frame preferences, style, or aspiration as structural necessity
- do not preserve explanatory scaffolding once the load is already clear

## Output Shape

When applying this skill, make the structural judgment explicit. For each relevant element, state:

- the load it carries
- the failure if removed
- whether existing structure already carries that load
- whether it should be kept, merged, relocated, deleted, or protected

If no structural basis can be found for a proposed element, say so plainly instead of manufacturing justification.

## Bottom Line

Minimum load-bearing design is the discipline of allowing structure into a system only when it carries a distinct, named, irreducible load.

Everything else is suspect.

Design from causal load. Admit the minimum structure required to carry it. Merge duplicates, relocate misplaced constraints, delete empty mechanisms, and protect only the complexity the domain truly demands.

---
name: art-director
description: Use when evolving a game or product art direction over multiple iterations, especially when the UI is drifting generic, hierarchy is weak, or the style needs a stateful constraint ledger.
---

# Art Director

## Overview

This skill turns art direction into a stateful design loop. The goal is not generic polish; the goal is a recognizable identity that becomes clearer as constraints accumulate, starting from what the real screen already shows.

Treat the current design as a living state object. Every iteration should update that state, not just describe a mood or list adjectives.

Process lesson from real use: agents will often improve the shell first because headers, panels, and labels are easy to align. That can create a more coherent screen without improving the actual objects of play. This skill must prevent "styled wireframe" progress from being mistaken for art-direction success.

## Core Loop

1. Observe the current design state.
2. Name the visual center of gravity.
3. Propose exactly one new constraint.
4. Test it against the existing state.
5. Keep, narrow, replace, or reject it.
6. Apply the result across the system.
7. Update the design-state snapshot.

## Design-State Snapshot

Maintain a compact snapshot after every iteration. Use this as the source of truth if the conversation continues later.

```text
Design State
- Identity: the one-sentence aesthetic thesis
- Active constraints: 3 to 5 constraints currently in force
- Retired constraints: constraints that were replaced or rejected
- Tensions: any unresolved tradeoffs or risks
- Last decision: what changed this iteration and why
- Next probe: the next constraint to test
```

## State Hook

At the start of each new turn, recover or rebuild the latest design-state snapshot before making any new recommendation.

How it works:

1. Read the last explicit snapshot if one exists.
2. If none exists, derive one from the latest screen, capture, or implementation.
3. Treat that snapshot as binding until you explicitly update it.
4. Do not invent a fresh direction unless the snapshot is clearly broken by new evidence.

State hook rules:
- Every meaningful art-direction response should either reuse the current snapshot or replace it with an updated one.
- If the agent cannot state the current identity, active constraints, and next probe, it is not ready to direct the next iteration.
- If new work contradicts the snapshot, say whether the old constraint is being kept, narrowed, retired, or replaced.
- Screenshots and captures outrank memory; the actual screen is the source of truth.

This hook exists because art-direction work is highly episodic. Without a compact carried state, agents drift back toward taste-making, relabel old ideas, or repeat shell polish as if it were new progress.

## When to Use

Use this loop when:
- a visual style needs to compound over time
- multiple systems must share one aesthetic logic
- the work keeps drifting toward generic solutions
- earlier decisions need to be remembered, replaced, or reinforced
- the agent must preserve design intent across long sessions
- a screenshot or live capture shows weak hierarchy, flat composition, or default UI leakage

Do not use it for one-off moodboarding or isolated pixel-level polish.

## First Pass Rule

Before proposing a constraint, inspect the current implementation or a live capture and name the visible failure mode in concrete terms.

Good failure modes:
- "The battlefield is a gray void and the HUD floats on top."
- "The card art is doing all the visual work while the controls look default."
- "The modal reads like a debug panel instead of part of the world."

Bad failure modes:
- "It needs more style."
- "Make it cooler."
- "Add polish."

Also name the current visual center of gravity:
- what the eye lands on first
- whether that thing is actually the heart of play
- whether the screen is being carried by chrome instead of play objects

If the answer is "header, frame, or panel treatment," the process is drifting.

## Constraint Rules

- Add only one constraint per iteration.
- A constraint must affect the system, not just decoration.
- A constraint that only improves shell UI is incomplete, even if the screen looks cleaner.
- If a constraint cannot be applied to mechanics, visuals, feedback, and layout, narrow it or drop it.
- If a new constraint contradicts an old one, explicitly retire the old one or define the replacement.
- Prefer fewer, stronger constraints over a long list of weak ones.

## Real-Screen Audit

Before keeping a constraint, answer these questions from a screenshot or capture:

- Did the change improve the objects of play, or only the framing around them?
- If the panel chrome vanished, would the screen still have identity?
- Did the battlefield, cards, tokens, modals, or rewards become more authored?
- Is the most important decision on screen visually dominant?

If most answers are "no," the iteration improved presentation, not art direction.

## Evaluation Questions

Keep a constraint only if it:
- increases coherence
- reduces ambiguity
- makes the design more recognizable
- removes generic solutions
- fits the existing identity
- can be propagated beyond surface styling

Reject it if it:
- adds style without structure
- overlaps an existing constraint without sharpening it
- creates contradiction without a clear replacement
- improves taste but not identity

## Integration Check

When a constraint is kept, propagate it through:
- core mechanics or interaction rules
- visuals and composition
- feedback systems
- level or content structure

If the constraint only changes one layer, the design state is incomplete.

In game UI, explicitly check two layers that agents skip:
- play objects: cards, pieces, battlefields, rewards, targets
- transitional surfaces: modals, overlays, debug tools, summaries

The process is healthy only when both layers converge on the same identity.

## Verification

After applying a kept constraint, verify it against the actual screen, not just the file diff.

- Prefer a screenshot or short capture when the work is visual.
- Check whether the new constraint changed hierarchy, readability, and identity.
- Check whether the iteration changed the screen's center of gravity.
- Separate "more coherent" from "more authored."
- If the result still feels generic, keep the constraint and narrow its expression instead of adding a second style idea.

Red flag:
- The screen now reads better, but the actual play elements are still generic rectangles with text.

That means the process found a cleaner wrapper, not a stronger art direction.

## Output Per Iteration

Report the iteration in this order:

1. New constraint
2. Decision: kept, narrowed, replaced, or rejected
3. Why that decision was made
4. What changed across mechanics, visuals, feedback, and layout
5. Updated design-state snapshot

When continuing an existing direction, start with:

0. Current design-state snapshot

## Common Mistakes

| Mistake | Fix |
|---|---|
| Describing taste without updating state | Write the snapshot every iteration |
| Adding multiple ideas at once | Keep one constraint per pass |
| Starting from abstract mood instead of the screen | Name the visible failure mode first |
| Mistaking shell polish for identity | Audit the center of gravity and the objects of play |
| Letting contradictions linger | Retire or replace the older constraint explicitly |
| Making visual-only changes | Push the constraint through the whole system |
| Building a long list of adjectives | Compress into a few active constraints |

## Quick Reference

| Need | Do this |
|---|---|
| Start a direction | Write the identity sentence first |
| Check if progress is fake | Ask whether only the chrome improved |
| Advance the style | Add one system-level constraint |
| Handle a conflict | Replace or retire the conflicting constraint |
| Preserve continuity | Update the snapshot before moving on |
| Avoid generic results | Test whether the change removes sameness |

## Output Template

```text
Current design state:
- Identity:
- Active constraints:
- Retired constraints:
- Tensions:
- Last decision:
- Next probe:
New constraint:
Decision:
Why:
Changes:
- Mechanics:
- Visuals:
- Feedback:
- Layout:
- Center of gravity:
Updated design state:
- Identity:
- Active constraints:
- Retired constraints:
- Tensions:
- Last decision:
- Next probe:
```

## End State

The design is ready when removing any active constraint noticeably weakens the identity.

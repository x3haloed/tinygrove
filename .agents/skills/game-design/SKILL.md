---
name: game-design
description: Use when designing, critiquing, or revising games that feel shallow, opaque, unfair, fake, or solved, or when a concept needs a repeatable core loop, mastery gradient, and meaningful player agency.
---

## When to Use

- Core loop is unclear or not yet fun
- Players report confusion, unfairness, or boredom
- Design relies on rewards/content instead of systems
- Need to critique/repair structure (not aesthetics)

Do not use for engine choice, UI polish, or QA execution.

---

## Overview

Treat game design as **system shaping under feedback**. A good game sustains a loop of: **meaningful decision → clear feedback → model evolution → growing agency → expression**

Your job is not to add features. Your job is to **stabilize this loop** and evolve it through **tight iteration cycles** until it reliably produces learning *and* expression.

Process lesson from real use: agents can easily improve structure, rewards, and presentation language without increasing strategic depth. A run can look more like a game while still playing like a thin prototype. This skill must force that distinction.

## State Hook

At the start of each new turn, recover or rebuild the current design-state for the game before proposing changes.

Use this compact source of truth:

```text
Game Design State
- Target lock: player identity, fantasy, session shape
- Core loop: Decision -> Feedback -> Model Change -> Next Decision
- Run arc: why choices matter 1, 2, and 3 steps later
- Critical decisions: the few decisions that currently define play
- Strategic pressures: what constrains the player right now
- Consequence chain: how early choices alter later states
- Dominant risks: fake choice, opacity, reward hijacking, economy theater, thin opposition, other
- Last validated gain: the last change that clearly improved strategy rather than presentation
- Next uncertainty: the most important unanswered design question
```

How it works:

1. Reuse the latest explicit state if it exists.
2. If none exists, derive it from the current prototype, tests, captures, or discussion.
3. Treat that state as binding until new evidence changes it.
4. Update it whenever you claim design progress.

State hook rules:
- Every substantial design response should begin from the current state, not a fresh theory.
- If the agent cannot name the current critical decisions and dominant risks, it is not ready to prescribe expansion.
- A new reward, encounter, or phase only counts as progress when it changes the stored consequence chain or dominant risks.
- Playtest evidence outranks intent; observed behavior is the source of truth.

This hook exists because game-design work is highly episodic. Without a compact state object, agents repeatedly confuse cleaner structure with deeper play and lose track of which uncertainties were actually resolved.

---

# The Design Method (End-to-End)

## Phase 0 — Target Lock

**One sentence only.**

- Player identity
- Core fantasy
- Session shape (5 min? 30 min? endless?)

If vague → stop.

---

## Phase 1 — Core Loop Definition

Write one sentence:

**Decision → Feedback → Model Change → Next Decision**

Constraints:

- Must include uncertainty
- Must include consequence
- Must be runnable mentally

If not crisp → design is not ready.

Then write a second sentence:

**Run-level arc: why this decision should matter 1, 2, and 3 steps later**

If that sentence is weak, the loop may be framed but not yet consequential.

---

## Phase 2 — Minimal Mechanic Set

Define the **smallest set of mechanics** that can produce the loop.

Rule:

- If removed, loop breaks → keep
- If not → delete

Goal: **smallest playable loop**

Also mark which pieces are:
- loop-critical
- scaffolding
- presentation

Agents routinely mistake presentation scaffolding for loop progress. Do not.

---

## Phase 3 — Rapid Prototype (Day-Scale)

Build the loop with:

- Placeholder assets
- Minimal UI
- Only core interactions

Constraints:

- Build fast
- Expect to throw away

Question to answer:

> “Is the core loop fun in isolation?”

If no → **return to Phase 1**

---

## Phase 4 — Playtest Ladder (Non-Negotiable)

Run in this order:

1. Self-test
2. Team test
3. Friends/family
4. Outsiders
5. Target audience

At each stage:

- Observe, don’t explain
- Record confusion points
- Record dominant strategies

Invariant:

> You cannot know if a game works until others play it.

If outside playtests are not available yet, at minimum run a design audit that records:
- the best obvious move
- the reason to choose a weaker immediate line
- what a player is building toward across the run

If any answer is missing, the prototype is not ready for expansion.

---

## Phase 5 — Iteration Loop (Core Engine)

For each test cycle:

1. Identify failures

   - Confusion → legibility issue
   - “Why did that happen?” → consistency issue
   - One best move → fake choice
   - Grinding → reward hijacking
   - Better framing, same strategy → presentation outran design
   - Reward choice with no downstream meaning → economy theater

2. Apply **mechanical repair** (not adjectives)

   - Add constraint → create tradeoff
   - Add feedback → expose cause/effect
   - Bound randomness → restore predictability
   - Add interaction → increase depth
   - Add downstream consequence → make the choice matter later
   - Add role pressure → make opponents or encounters ask different questions

3. Re-test immediately

Cycle until:

- Players form a correct mental model
- Players make different viable choices
- Those choices create measurably different later states

---

## Phase 6 — Vertical Slice (Quality Proof)

Build one short section at near-final quality.

Must demonstrate:

- Core loop intact
- Feedback clarity
- Early → mid → late pacing

Purpose:

- Validate feel
- Estimate production cost
- Lock direction

---

## Phase 7 — Expansion (Controlled Growth)

Expand one axis at a time:

- New constraint
- New interaction
- New context

Never expand:

- Without preserving loop integrity

Rule:

> Systems first, content second.

Expansion test:
- Does this addition create a new decision?
- Does it sharpen an existing tradeoff?
- Does it alter future planning?

If not, it is likely wrapper growth, not game growth.

---

## Phase 8 — Continuous Feedback (Live System)

Collect from:

- Playtests
- Analytics
- Community

Prioritize:

- Repeated complaints
- High-impact failures

Reject:

- Requests that break core loop

---

# Structural Invariants (Must Hold)

## Loop Integrity

- Decisions under constraint
- Clear, fast feedback
- Learnable cause/effect

## Model Dynamics

- Incremental learning (mastery)
- Occasional **rupture** (model reframing)

## Dual-Mode Operation

- Same mechanics support:
  - Learning
  - Expression (style, speed, improvisation)

## Depth

- From interacting systems
- Includes **other agents** (if multiplayer)

## Failure Quality

- Teaches
- Attributable
- Recoverable

## Temporal Staging

- Early: fast, low-risk learning
- Mid: expanding possibility space
- Late: integrated mastery + pressure

## Reward Alignment

- Rewards reinforce mastery
- Never replace the core loop

## Consequence Chain

- Early choices alter later lines
- Resources have meaningful sinks
- Rewards compete with other forms of progress

## Authored Opposition

- The game asks different questions over time
- Opposition is not just stats or random card output
- The player can form a model of what the opponent or encounter pressures

---

# Anti-Invariants (Kill Signals)

- Fake choices (dominant strategy)
- Opaque outcomes
- Input friction
- Content without systems
- Unbounded randomness
- No mastery curve
- Extrinsic reward hijacking
- A better-looking run structure with unchanged strategic space
- Rewards that read important but do not reshape future decisions
- Economies that accumulate numbers without forcing tradeoffs
- Opponents that function but do not teach, threaten, or specialize

---

# Process Audit

Use this whenever reviewing a prototype, milestone, or commit series.

Start by restating the current `Game Design State` in compact form.

## Separate These Explicitly

- What improved presentation?
- What improved legibility?
- What improved strategic depth?
- What improved session structure?

Do not merge these into one verdict.

## Questions That Prevent False Positives

- What choice became more interesting, not just more visible?
- What will the player now plan around that they did not before?
- What resource now creates tension instead of bookkeeping?
- What opponent, encounter, or reward now asks for adaptation?
- If the new reward or phase screen were removed, what gameplay depth would remain?

If the answers are mostly visual or structural, design progress is being overstated.

---

# Mechanic Audit Card

```text
Mechanic:
Decision:
Constraint:
Immediate feedback:
Delayed feedback:
Player model update:
Expression at mastery:
Rupture potential:
Failure mode:
Repair:
Later consequence:
Competing alternative:
```

## Output Pattern

When continuing existing work, report in this order:

1. Current game design state
2. What changed in presentation, legibility, strategic depth, and session structure
3. Which state fields changed
4. What uncertainty should be tested next

---

# Heuristics (High Signal)

- **Follow the fun**: if the core loop isn’t fun in 5 minutes, nothing will fix it
- **Prototype fast**: speed > polish
- **Cut aggressively**: fewer mechanics → more depth
- **Observe players**: confusion > opinion
- **Design for expression**: high-skill play should look different, not just better
- **Engineer rupture**: hide deeper rules that reward discovery

---

# Output Expectations (When Using This Skill)

Produce:

1. One-sentence core loop
2. Mechanic audit cards (core systems)
3. Top invariant risks
4. First mechanical repair
5. Next smallest prototype step

---

# Mental Model

You are not building a game.

You are \*\*shaping a system until humans can:

- understand it
- predict it
- master it
- express themselves through it\*\*

If that happens, the game is good. If it doesn’t, nothing else matters.

## Additional Reading
`references/physics_of_good_games.md` encodes the full set of laws to test your game against

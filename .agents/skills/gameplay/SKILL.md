---
name: gameplay
description: Use when the task requires actually playing a video game through its runtime to make progress, traverse state, reproduce a situation, or verify that interactions behave correctly over multiple moves.
---

# Gameplay

## Overview

Use this skill when the job is to play the game, not just inspect code or verify one static frame. The core rule is a short closed loop: observe, decide, act briefly, then verify before committing to the next move.

This skill is intentionally generic across engines and platforms. Prefer structured runtime state when available, but always confirm against what is visibly on screen.

## Use This Skill When

- the task requires advancing a save, scene, level, or encounter
- the important question depends on multiple actions rather than one screenshot
- the agent needs to reproduce an issue that appears only after playing for a while
- the runtime exposes inputs the agent can trigger directly
- the user wants the agent to play, not just review or critique

## Do Not Use This Skill When

- the task is purely code review or architecture work
- a single screenshot, log, or assertion already answers the question
- the runtime cannot be controlled in any reliable way

## Core Loop

1. Observe the current state.
   - Read structured game state if the runtime exposes it.
   - Capture a screenshot when spatial context or UI context matters.
2. Orient to the immediate objective.
   - Clarify the next small milestone such as "reach the door," "open the menu," or "reproduce the hit reaction."
3. Act in short bursts.
   - Prefer 1-3 inputs or a very short movement sequence.
   - Do not queue long action chains unless the environment is highly deterministic.
4. Verify the result.
   - Re-check visible state after every burst.
   - If the result is unclear, capture evidence before guessing.
5. Record progress.
   - Note what changed, what still blocks progress, and what the next move should be.
6. Checkpoint before risk.
   - Save or create a reproducible restore point before boss fights, scene transitions, destructive choices, or long traversal.

## Input Rules

- Prefer the narrowest input that answers the question.
- If a menu, dialog, or prompt is open, resolve that before moving around blindly.
- After transitions such as doors, teleports, respawns, cutscenes, or loads, wait for the new state to stabilize before issuing more input.
- When stuck, stop escalating action count. Re-observe instead.

## Evidence Ladder

- Use logs or structured state for facts such as HP, coordinates, flags, inventory, or quest state.
- Use screenshots for spatial truth, HUD state, and what a player would actually see.
- Use `../video-capture/SKILL.md` when timing, animation, sequencing, or short-lived prompts matter.

## Common Mistakes

- Treating runtime state as complete truth when the screen shows otherwise
- Sending too many inputs at once and losing track of what caused the result
- Skipping verification after transitions or combat outcomes
- Continuing to brute-force movement after getting lost instead of re-orienting
- Forgetting to save before risky or expensive progression steps

## Output Expectations

- State the current objective and what was attempted.
- Report the last confirmed game state, not the hoped-for state.
- Keep reproduction or progress notes tight enough that the next agent can resume cleanly.

## References

- Motion and temporal inspection: `../video-capture/SKILL.md`

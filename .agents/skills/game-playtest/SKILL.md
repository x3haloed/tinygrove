---
name: game-playtest
description: Use when the user asks for game playtests, screenshot-based verification, HUD or overlay review, or structured issue-finding in a game runtime, especially on browser-based paths.
---

# Game Playtest

## Overview

Use this skill to test games the way players experience them: through boot, input, scene transitions, HUD readability, and visual state changes. Prefer browser automation when the project supports it, and prefer screenshot review for any runtime where visual state matters.

Do not stop at "the code runs." Verification here means confirming that the game *plays* the way the implementation intends: movement feels correct, transitions happen at the right time, animation reads clearly, and the UI supports play instead of fighting it.

This skill builds on `../gameplay/SKILL.md`. Use `gameplay` for the generic act of playing a game through runtime feedback loops, then apply that loop here for development goals such as QA, bug reproduction, progression setup, and issue reporting.

## Preferred Workflow

1. Boot the game and confirm the first actionable screen.
2. Use `../gameplay/SKILL.md` to exercise the main verbs through short observe-decide-act-verify loops.
3. Capture screenshots from representative stable states.
4. Escalate to `../video-capture/SKILL.md` when motion, timing, or transient behavior matters.
5. Check the UI layer independently from the render layer.
6. Report findings in severity order with reproduction steps.

## Verification Ladder

Use the lightest tool that can answer the runtime question:

- Logs or assertions answer "did it execute?"
- Screenshots answer "what rendered in a stable state?"
- `../gameplay/SKILL.md` answers "what happens when I keep playing through the next few decisions?"
- `../video-capture/SKILL.md` answers "what happened over time?"

Use video capture when the issue depends on sequence, timing, or play feel rather than a single frame.

## Tooling Guidance

- Prefer `../gameplay/SKILL.md` when the task requires active progression, bug reproduction, or state setup rather than passive inspection.
- Prefer Playwright or equivalent browser automation already available in the repo.
- For non-browser runtimes, use the engine or platform launch flow plus screenshots, logs, and structured manual passes.
- When the game is canvas or WebGL heavy, screenshots are mandatory because DOM assertions alone miss visual regressions.
- After capturing screenshots, open the saved image files with the harness's file-viewing mechanism so they are available for visual inspection, preferably through inline image attachment or rendering support. Follow the `../screenshot/SKILL.md` workflow for capture and review details.
- Use screenshots to judge playfield obstruction and HUD weight, not just correctness of text or layout.
- Use `../video-capture/SKILL.md` when screenshots miss the behavior because the important detail is temporal.
- When deterministic automation is not practical, do a structured manual pass and capture evidence.
- For 3D rendering bugs or unexplained frame cost, use SpectorJS and browser performance tooling rather than guessing from code alone.

## Use Video Capture When

- attack timing, hit-stop, dodge windows, or animation canceling need verification
- camera transitions, scene transitions, onboarding fades, or menu handoffs happen too quickly for screenshots
- transient HUD prompts, flicker, or interaction hints may be incorrect
- input-to-feedback timing matters
- the game boots successfully but the interaction loop still feels wrong

## Common Checks

### 2D checks

- sprite alignment and baseline consistency
- hit or hurt animation readability
- HUD overlap with the playfield
- command menu state changes
- tile or platform readability
- input-state feedback and turn-state clarity

### 3D checks

- first-load playability versus dashboard-like chrome
- persistent overlay weight versus playfield visibility
- camera control and camera reset behavior
- pointer-lock or drag-look transitions when menus and overlays open
- depth readability and silhouette clarity
- secondary panels collapsed or dismissible during normal play
- resize behavior
- WebGL context loss or renderer fallback behavior
- material or lighting regressions
- GLB or texture streaming stalls
- collision proxy or physics mismatch
- performance cliffs tied to post-processing or asset load

## Responsive and Browser Checks

- desktop and mobile viewport sanity
- safe-area and notch issues where relevant
- reduced-motion behavior for UI transitions
- keyboard, pointer, and pause-state handling
- React state and scene state synchronization when the project uses React Three Fiber

## Reporting Standard

Lead with findings. Keep each finding concrete:

- what the user sees
- how to reproduce it
- why it matters
- what likely subsystem owns it

## References

- Generic game-playing loop: `../gameplay/SKILL.md`
- Shared architecture: `../game-foundations/SKILL.md`
- Browser architecture: `../web-game-foundations/SKILL.md`
- Frontend review cues: `../game-ui-frontend/SKILL.md`
- Motion and temporal verification: `../video-capture/SKILL.md`
- 3D debugging notes: `../game-studio/references/webgl-debugging-and-performance.md`
- Full checklist: `../game-studio/references/playtest-checklist.md`

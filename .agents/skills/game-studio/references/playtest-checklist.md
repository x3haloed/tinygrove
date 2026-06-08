# Playtest Checklist

Use this checklist for game-runtime QA. Apply the browser-specific section only when the project actually runs in the browser.

## Universal checks

- Does the game boot into a useful first state?
- Are the main verbs obvious and responsive?
- Does the HUD remain readable over gameplay?
- Does the first playable screen prioritize play over dashboard chrome?
- Does the central playfield stay mostly clear during normal play?
- Do pause, failure, and recovery states work?
- Does the game survive the runtime’s common state changes such as resize, focus loss, scene reload, or resolution changes?

## 2D checks

- sprite baseline consistency
- hit, hurt, and attack timing
- command menu focus and input state
- tile or platform readability
- particle or camera effects obscuring gameplay

## 3D checks

- camera control stability
- camera and menu-state handoff
- depth readability
- persistent overlay weight versus scene readability
- secondary notes, controls, and quest details collapsed by default
- resize, resolution, and aspect-ratio handling
- renderer fallback, device loss, or engine recovery handling
- material and lighting stability across states
- GLB asset and texture streaming behavior
- collision proxy alignment
- GPU bottlenecks isolated with capture tools when needed

## Browser checks

- desktop and mobile viewports
- input modality differences
- reduced-motion behavior
- pause behavior when focus changes
- pointer-lock and camera-input release when overlays open
- transient onboarding hints dismiss or fade once the player is moving

## Engine-native checks

- fullscreen, windowed, or resolution-mode transitions
- scene reload or room transition stability
- engine log visibility during repro
- input remapping or controller focus behavior if supported
- native UI layer and gameplay input handoff

## Reporting

- Capture screenshots for visual findings.
- Put findings in severity order.
- Include reproduction steps.
- Call out whether the likely owner is simulation, renderer, frontend, or asset pipeline.

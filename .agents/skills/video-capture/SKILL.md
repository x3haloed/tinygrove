---
name: "video-capture"
description: "Use when the user explicitly asks for a desktop or system video capture, or when an agent needs to inspect short-lived UI behavior by recording the full screen, an app, a window, or a region and converting the result into model-readable frames."
---

# Video Capture

Follow these save-location rules every time:

1) If the user specifies a path, save there.
2) If the user asks for a video capture without a path, save to the OS default screenshots or screen recordings location.
3) If you need a recording for your own inspection, save to the temp directory.

This skill is for agent and LLM consumption, not human-facing deliverables. The default output is a short recording converted into an ordered sequence of timestamped frames. Use a contact sheet only as a quick overview artifact.

## Tool priority

- Prefer tool-specific recording capabilities when available.
- Use this skill for OS-level desktop capture, especially when the goal is to inspect motion or short-lived UI changes.
- Do not use this skill for authored video, editing, trimming for presentation, or human-facing export workflows. Use a dedicated video-authoring skill for those cases.

## Core roundtrip

1. Record the smallest useful target for the shortest useful duration.
2. Convert the recording into sparse timestamped frames, starting with very low FPS.
3. Open and review the frames in timestamp order.
4. Build a contact sheet only if a compact overview would help.

## Recording guidance

- Keep captures short. Re-record narrowly instead of capturing long clips.
- Match the target the same way the `screenshot` skill does: full screen, specific app, specific window, active window, or region.
- Prefer app, window, or region capture when the whole desktop adds noise.
- Start with low temporal density. Record only as much motion detail as you need for the model to understand the UI change.
- If text readability matters, prefer fewer larger frames over one dense composite image.
- Do not stop at the raw video unless the downstream tool explicitly accepts video.

## Frame extraction defaults

- Default artifact: ordered timestamped frames.
- Start at `1 FPS` for extraction unless the UI is changing too quickly to understand.
- Timestamp each frame visibly so the model can reason about sequence and timing.
- Sample sparsely by default. Increase FPS only when a low-FPS pass misses important state changes.
- Keep frames at readable resolution. Do not shrink them into a single giant overview unless you only need coarse motion summary.

Recommended sampling ladder:

- First pass: `1 FPS`
- If motion is too fast: `2 FPS`
- Only go higher when the task clearly requires it

Example `ffmpeg` patterns for post-processing:

- Extract one frame per second:

```bash
ffmpeg -i input.mp4 -vf "fps=1" frames/frame-%04d.png
```

- Burn timestamps into extracted frames:

```bash
ffmpeg -i input.mp4 -vf "fps=1,drawtext=text='%{pts\\:hms}':x=20:y=20:fontcolor=white:box=1:boxcolor=black@0.7" frames/frame-%04d.png
```

- Build a contact sheet overview:

```bash
ffmpeg -i input.mp4 -vf "fps=1,scale=480:-1,tile=3x3" overview.png
```

Use the ordered frame sequence as the primary input to the model. Open the generated frame images with the harness's file-viewing mechanism so they are available for visual inspection, preferably through inline image attachment or rendering support (for example, an image viewer, `Read`, or any tool that attaches image files inline). Treat `overview.png` as an optional summary image.

## macOS

Use OS-level capture and keep the clip short.

Recommended fallback order on macOS:

1. If interactive recording is acceptable, use the built-in Screenshot UI.
2. If command-line recording is required, use `ffmpeg` with AVFoundation only after Screen Recording permission has already been granted.
3. If `ffmpeg` capture is unstable, fall back to the Screenshot UI for capture and use `ffmpeg` only for frame extraction.

### Window-targeted capture with ScreenCaptureKit

When you need a recording of a single launched app window, prefer ScreenCaptureKit over whole-display capture.
It can target the exact `SCWindow` owned by the live process, which is more reliable than recording the display and cropping it later.

Recommended flow:

1. Launch the target app.
2. Resolve its PID or bundle identifier.
3. Find the matching `SCWindow` via `SCShareableContent.current`.
4. Build `SCContentFilter(desktopIndependentWindow:)`.
5. Record with `SCRecordingOutput` to an MP4.
6. Extract sparse timestamped frames from the result for inspection.

This path requires macOS 12.3+ for the ScreenCaptureKit window APIs and macOS 15.0+ for `SCRecordingOutput`.

Reusable implementation:

- [`scripts/macos_window_record.swift`](scripts/macos_window_record.swift)
- [`scripts/macos_window_record.sh`](scripts/macos_window_record.sh)

The Swift helper accepts:

- `--output <path>`
- `--duration <seconds>`
- `--pid <process-id>`
- `--bundle-id <bundle-id>`
- `--owner <fallback-app-name>`
- `--title <fallback-window-title>`
- `--fps <frame-rate>`
- `--wait <seconds>`

Example build and run:

```bash
xcrun swiftc -parse-as-library \
  -sdk "$(xcrun --sdk macosx --show-sdk-path)" \
  -target arm64-apple-macosx15.0 \
  -O \
  -framework ScreenCaptureKit \
  -framework AVFoundation \
  -framework CoreMedia \
  -framework AppKit \
  -framework Foundation \
  scripts/macos_window_record.swift \
  -o /tmp/macos_window_record

/tmp/macos_window_record \
  --output /tmp/window.mp4 \
  --duration 5 \
  --pid 12345 \
  --bundle-id org.godotengine.godot \
  --owner Godot \
  --title "Starship MMO" \
  --fps 30 \
  --wait 20
```

- Built-in interactive screen recording:

```bash
open -a Screenshot
```

This opens the macOS Screenshot UI, which supports full-screen and region-based screen recording. Use it when an interactive recording is acceptable.

- Whole-screen recording with `ffmpeg`:

```bash
ffmpeg -f avfoundation -pixel_format uyvy422 -framerate 2 -i "0:none" -t 5 output.mp4
```

- Record to temp for agent inspection:

```bash
ffmpeg -f avfoundation -pixel_format uyvy422 -framerate 2 -i "0:none" -t 5 "$(mktemp -t codex-video-XXXX).mp4"
```

Notes:

- Device indices vary. Run `ffmpeg -f avfoundation -list_devices true -i ""` to list available screen devices. On this machine, `Capture screen 0` mapped to `0`.
- macOS Screen Recording permission is required before unattended capture will work reliably.
- AVFoundation screen capture can be sensitive to pixel format. If the default command fails, try `-pixel_format uyvy422` first.
- Permission dialogs can become part of the capture. If a system permission sheet appears, resolve it first, then re-record so the frames reflect the target UI rather than the prompt.
- If `ffmpeg` appears to hang or emit implausible timing or duplicated-frame warnings, stop the run and fall back to the Screenshot UI for capture.
- For app, window, or region capture, mirror the target-selection logic from the `screenshot` skill, then use the smallest available capture surface. If non-interactive window-level recording is unreliable, capture the relevant display briefly and convert to frames rather than blocking on perfect video cropping.

## Linux

Use the first available recorder and keep the clip short.

- Whole screen with X11:

```bash
ffmpeg -video_size 1920x1080 -framerate 2 -f x11grab -i :0.0 -t 5 output.mp4
```

- Region capture:

```bash
ffmpeg -video_size 1280x720 -framerate 2 -f x11grab -i :0.0+100,200 -t 5 output.mp4
```

- Wayland users may need a portal-based recorder or a compositor-specific tool. If direct capture fails, ask the user to allow screen recording and retry with the desktop's built-in recorder.

## Windows

Use OS-level recording when available, then convert the result into frames.

- Xbox Game Bar can record the active app:

```powershell
Start-Process "ms-gamebar:"
```

- For command-line post-processing after capture:

```powershell
ffmpeg -i input.mp4 -vf "fps=1,drawtext=text='%{pts\\:hms}':x=20:y=20:fontcolor=white:box=1:boxcolor=black@0.7" frames/frame-%04d.png
```

If a pure CLI recorder is required, prefer an existing tool already installed in the environment rather than inventing a custom PowerShell capture path.

## Multi-display behavior

- Follow the same targeting guidance as `screenshot`: if the relevant behavior is confined to one app, window, or region, capture only that area when possible.
- If you must capture the full desktop on a multi-display setup, keep the duration very short and downsample frames conservatively.
- For multi-display recordings, frame sequences are usually more readable than a single contact sheet.

## Error handling

- If the OS blocks screen recording, request Screen Recording permission and retry.
- If `ffmpeg` is unavailable, ask the user to install it or use an OS-native recorder and then convert the saved video later.
- On macOS, if unattended AVFoundation capture is flaky, use the Screenshot UI for capture and keep `ffmpeg` for extraction only.
- If the first result is too noisy, re-record with a narrower target instead of compensating with more frames.
- If the first result misses important transitions, raise FPS gradually instead of jumping straight to a dense capture.
- If extracted frames are too dense or too many, lower the frame rate and retry.
- If text in a contact sheet is unreadable, switch back to a sequence of full-size frames.
- Always report the saved video path and the generated frame paths.

# Worktree: audio-capture

**Plan Reference**: See [PLAN.md: Audio Capture & Silence](../plan/SKILL.md#audio-capture--silence-vad-lite)

## Issue
- **ID**: stt-rtf
- **Title**: Implement audio capture module (audio.rs)
- **Priority**: 1 (critical)
- **Type**: task

## Description
Implement audio.rs per PLAN.md section 'Audio Capture & Silence': cpal capture at device rate; resample to 16 kHz mono; RMS silence detection for toggle mode. This is a CRITICAL missing module - without it, PTT workflow cannot function.

## What This Enables
This is the foundational module that unblocks 5 other issues:
1. **stt-y3w** - Hotkey wiring to audio capture
2. **stt-bq4** - Long-form record mode capture
3. **stt-1j5** - Toggle mode with silence detection
4. **stt-j53** - Settings UI (model management)
5. **stt-28a** - Audio unit tests

## Integration Points
- **Source file**: `src-tauri/src/audio.rs` (to be created)
- **Depends on**: Nothing - this is a foundational module
- **Consumers**: typer.rs, keys.rs, whisper integration

## Technical Requirements
1. Use `cpal` crate for audio capture at device sample rate
2. Resample to 16kHz mono for whisper compatibility
3. Implement RMS silence detection with configurable threshold
4. Support both hold-to-talk and toggle mode
5. Handle device enumeration and fallback

## Success Criteria
- Audio captures at device rate, resamples to 16kHz mono
- RMS silence detection triggers for toggle mode
- Works with default audio input device
- Graceful error handling when no mic available

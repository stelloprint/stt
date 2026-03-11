# Worktree: audio-tests

**Plan Reference**: See [PLAN.md: Audio Capture & Silence](../../plan/SKILL.md#audio-capture--silence-vad-lite)

## Purpose

This worktree adds the missing audio-unit coverage, but it should be treated as a late-stage lane because `audio.rs` is still changing under `ptt-runtime` and `record-mode`.

## Issue Chain

- **stt-28a**
- **Current**: `stt-28a`

## Shared Hot Files

- `apps/web/src-tauri/src/audio.rs`

## Conflict Guidance

- Do not start until `ptt-runtime` and `record-mode` behavior is stable enough to test
- High churn risk if started too early

## Issue: stt-28a - Unit Tests: Audio Capture And Silence Detection

- **Priority**: 3
- **Type**: chore

### Focus

Add real unit coverage for audio capture, resampling, silence detection, and buffer behavior after the runtime semantics settle.

### Technical Targets

1. Cover initialization and device behavior where practical.
2. Cover resampling to 16 kHz mono.
3. Cover RMS silence thresholds and auto-stop behavior.
4. Cover long-session buffer management edge cases.

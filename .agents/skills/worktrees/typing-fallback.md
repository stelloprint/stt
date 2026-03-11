# Worktree: typing-fallback

**Plan Reference**: See [PLAN.md: Typing Output](../plan/SKILL.md#typing-output)

## Purpose

This is a narrow side lane for fixing the macOS clipboard fallback without touching the main runtime flow.

## Issue Chain

- **stt-3ih**
- **Current**: `stt-3ih`

## Shared Hot Files

- `apps/web/src-tauri/src/type_.rs`

## Conflict Guidance

- Good parallel side lane
- Keep the fix narrowly scoped and testable

## Issue: stt-3ih - Fix macOS Clipboard Fallback To Paste With Command+V

- **Priority**: 2
- **Type**: bug

### Focus

The clipboard fallback uses the wrong modifier for macOS. This should stay small: fix the paste chord, preserve clipboard restore behavior, and add regression coverage.

### Technical Targets

1. Use the macOS command modifier instead of control for paste fallback.
2. Preserve clipboard restore behavior.
3. Add tests around the chosen modifier.
4. Keep the change isolated to fallback typing behavior.

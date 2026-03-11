# Worktree: permissions-safety

**Plan Reference**: See [PLAN.md: Permissions & Safety](../plan/SKILL.md#permissions--safety)

## Purpose

This is a narrow side lane for replacing destructive or brittle macOS permission probes with safe status checks and explicit onboarding flows.

## Issue Chain

- **stt-uwb**
- **Current**: `stt-uwb`

## Shared Hot Files

- `apps/web/src-tauri/src/permissions.rs`

## Conflict Guidance

- Good parallel side lane
- Minimal overlap with the runtime and route workstreams

## Issue: stt-uwb - Replace Destructive Permission Probes With Safe macOS Checks

- **Priority**: 2
- **Type**: bug

### Focus

The current accessibility check sends a destructive `Cmd+X` keystroke to the frontmost app, and microphone checks rely on brittle AppleScript shelling. This lane should replace those with side-effect-free checks or explicit fallback states.

### Technical Targets

1. Remove destructive accessibility probing.
2. Use safer microphone checks or degrade to `Undetermined`.
3. Keep request/open-settings actions available.
4. Preserve manual QA safety.

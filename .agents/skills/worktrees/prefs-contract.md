# Worktree: prefs-contract

**Plan Reference**: See [PLAN.md: Preferences & Files](../plan/SKILL.md#preferences--files)

## Purpose

This worktree replaces the old `fix-compilation-bug` lane. The current issue is not a missing-type bug in `stt.rs`; it is a schema and validation contract problem centered on preferences, tests, and UI bounds.

## Issue Chain

- **stt-im0** → stt-ir2
- **Current**: `stt-im0`

## Shared Hot Files

- `apps/web/src-tauri/src/prefs.rs`
- `apps/web/src/routes/settings.tsx`

## Conflict Guidance

- Safe to run alongside one major backend lane
- Avoid overlapping with `model-management` if both need broad changes in `settings.tsx`

## Issue: stt-im0 - Fix Test Compilation Errors In stt.rs

- **Priority**: 1
- **Type**: bug

### Focus

Use this issue as a diagnosis-and-rescope entry point. The tracker text is already updated to reflect that the likely failure surface is in preferences/runtime wiring rather than missing voice-command types.

### Technical Targets

1. Run the Rust tests and capture the actual failing module.
2. Update the issue assumptions to the real failing surface if needed.
3. Split out any distinct follow-up work rather than letting this become a grab bag.

## Issue: stt-ir2 - Enforce And Align Preference Validation Across Rust And UI

- **Priority**: 2
- **Type**: bug

### Focus

Make Rust and UI enforce the same schema contract. Right now the default hotkey case skips validation, save paths do not validate before persisting, and UI bounds drift from backend validation.

### Technical Targets

1. Validate on save in `Prefs::update()`.
2. Remove the early-return loophole in `validate_preferences()`.
3. Align UI field limits with Rust validation bounds.
4. Fix or intentionally support the stale `voice_commands.map = {}` fixture.

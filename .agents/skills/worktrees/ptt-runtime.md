# Worktree: ptt-runtime

**Plan Reference**: See [PLAN.md: Typical Workflow Sequence](../plan/SKILL.md#typical-workflow-sequence), [PLAN.md: Hotkeys](../plan/SKILL.md#hotkeys-modifiers-only-lr-aware), and [PLAN.md: Data Storage](../plan/SKILL.md#data-storage-sqlite)

## Purpose

This worktree owns the real push-to-talk runtime path from hotkey activation through:

1. session creation
2. transcription persistence
3. typing behavior
4. toggle-mode control flow
5. accurate runtime/HUD state

This is the highest-value backend lane because the current hotkey callback still bypasses `SessionManager` and typing preferences in the main runtime path.

## Issue Chain

- **stt-t4k** → stt-1j5 → stt-ied → stt-8cp
- **Current**: `stt-t4k`

## Shared Hot Files

- `apps/web/src-tauri/src/lib.rs`
- `apps/web/src-tauri/src/session.rs`
- `apps/web/src-tauri/src/keys.rs`
- `apps/web/src/routes/index.tsx`

## Conflict Guidance

- Do not run this in parallel with `record-mode` or `model-management`.
- Safe side lanes: `prefs-contract`, `permissions-safety`, `typing-fallback`

## Issue: stt-t4k - Wire Session/Entry Persistence To PTT Workflow

- **Priority**: 1
- **Type**: task

### Focus

Move the actual hotkey-driven runtime onto the same persistence path already exposed by commands. This should be the first step before toggle-mode or HUD wiring build on top of it.

### Technical Targets

1. Create a session on activation in the real hotkey path.
2. Persist typed or untyped entries on completion.
3. End the session and compute totals from real entries.
4. Use saved typing preferences instead of `Typer::with_defaults()`.

## Issue: stt-1j5 - Implement Toggle Mode With Silence Detection

- **Priority**: 1
- **Type**: task
- **Depends on**: `stt-rtf`

### Focus

Build toggle mode on the same shared runtime path as hold-to-talk. Avoid duplicating session, audio, or typing logic.

### Technical Targets

1. Start on the first activation chord and stop on the next.
2. Auto-stop after configured silence.
3. Respect left/right chord settings through `KeysHandle::set_enabled()`.
4. Keep hold mode behavior unchanged.

## Issue: stt-ied - Capture The Actual Frontmost macOS App For Session Metadata

- **Priority**: 3
- **Type**: bug

### Focus

Fix the reopened session metadata bug after the core runtime path is using session creation consistently. The current implementation reports the STT app itself instead of the target app.

### Technical Targets

1. Use the active-application API rather than `currentApplication()`.
2. Capture the target app before typing begins.
3. Keep failures best-effort and non-blocking.

## Issue: stt-8cp - Replace HUD Placeholder Mic And Recording State With Live Backend Status

- **Priority**: 3
- **Type**: task

### Focus

Once the runtime path is authoritative, expose real status to the HUD. This should be the last step in the chain because it depends on the runtime surface stabilizing.

### Technical Targets

1. Replace the fixed `isRecording` state with backend/runtime state.
2. Replace synthetic mic activity with real capture status.
3. Keep the page safe when no session is active.

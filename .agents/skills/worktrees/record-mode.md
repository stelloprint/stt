# Worktree: record-mode

**Plan Reference**: See [PLAN.md: Record Mode Bounds](../plan/SKILL.md#record-mode-bounds), [PLAN.md: Audio Capture & Silence](../plan/SKILL.md#audio-capture--silence-vad-lite), and [PLAN.md: UI](../plan/SKILL.md#ui-react--tanstack-router)

## Purpose

This worktree owns the long-form recording path end to end:

1. backend capture stability
2. safe rotation semantics
3. `/record` route wiring to the real runtime
4. unit coverage for chunking and rotation

This lane should run **sequentially** and should not overlap with other `lib.rs`-heavy backend work such as `ptt-runtime` or `model-management`.

## Issue Chain

- **stt-bq4** → stt-rdp → stt-fuc → stt-83i
- **Current**: `stt-bq4`

## Shared Hot Files

- `apps/web/src-tauri/src/audio.rs`
- `apps/web/src-tauri/src/lib.rs`
- `apps/web/src/routes/record.tsx`
- `apps/web/src/lib/api.ts`

## Issue: stt-bq4 - Long-form Record Mode Audio Capture

- **Priority**: 1
- **Type**: task

### Focus

Finish the backend capture path before touching the page-level UI. The current implementation still has state split between `RecordCapture` and thread-local capture state, does not enforce chunk timing from backend preferences, and can lose the final partial buffer on stop.

### Technical Targets

1. Share one live capture state between the CPAL callback and `RecordCapture` methods.
2. Enforce chunk boundaries from `prefs.record.chunk_seconds` in the backend.
3. Flush the final buffered audio on stop instead of dropping it.
4. Keep transcript-file writes and DB entry creation on the same path.

### Exit Criteria

- `get_and_clear_chunk()` reads real capture data
- stop flushes the final chunk
- chunk timing comes from backend state
- record entries and transcript writes stay aligned

## Issue: stt-rdp - Record Mode File Rotation

- **Priority**: 1
- **Type**: task
- **Blocked by**: `stt-bq4`

### Focus

Add safe rotation semantics only after the base capture path is trustworthy. Rotation should track file/session lifetime independently from chunk lifetime and must not lose in-flight audio or transcript text.

### Technical Targets

1. Base `max_hours` on session/file lifetime, not the last chunk timestamp.
2. Replace stubbed file-size tracking with a real byte count.
3. Drain and persist buffered audio before swapping sessions/capture objects.
4. Create the new session only after the previous one is safely finalized.

## Issue: stt-fuc - Wire /record Route To Record-Mode Runtime Commands

- **Priority**: 1
- **Type**: task
- **Blocked by**: `stt-bq4`, `stt-rdp`

### Focus

Replace fake `/record` session creation with the actual record-mode commands. The route should use backend runtime status, live chunk transcription, and real rotation/session state.

### Technical Targets

1. Start/stop recording through `start_record_mode` and `stop_record_mode`.
2. Consume real runtime state from `get_record_status`.
3. Show real transcript chunks instead of relying on stale open sessions.
4. Reflect rotation count and current session from backend state.

## Issue: stt-83i - Unit Tests: Record Mode Chunking/Rotation

- **Priority**: 3
- **Type**: chore
- **Blocked by**: `stt-rdp`

### Focus

Lock in the finalized record-mode behavior with tests only after the backend and route contracts stop moving.

### Technical Targets

1. Cover chunk boundary behavior.
2. Cover rotation on both hours and file size.
3. Assert no typing occurs during record mode.
4. Assert transcript-file and DB behavior stay in sync across rotation.

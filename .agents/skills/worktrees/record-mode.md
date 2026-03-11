# Worktree: record-mode

**Plan Reference**: See [PLAN.md: Record Mode Bounds](../plan/SKILL.md#record-mode-bounds) and [PLAN.md: Audio Capture & Silence](../plan/SKILL.md#audio-capture--silence-vad-lite)

## Issue Chain
- **stt-bq4** → stt-rdp → stt-83i
- Current: stt-bq4

## Issue: stt-bq4 - Long-form Record Mode Audio Capture
- **Priority**: 1
- **Type**: task

### Description
Per PLAN.md 'Record mode': continuous multi-hour capture/transcribe to file and DB (not typed into apps), chunked every 60s. Default 8h or 4GB per file with rotation. Audio capture continues until explicit stop. Writes to rolling .txt file under App Support.

### What This Enables
- Record mode for meeting notes, dictation, and long-form transcription
- Chunked transcription to rolling files (not keystroke output)
- No blocking during long capture sessions

### Integration Points
- **Source**: Extend `src-tauri/src/audio.rs` with record-mode capture
- **Depends on**: `stt-rtf` (audio capture module - ready)
- **Output**: `~/Library/Application Support/sst/transcripts/`
- **DB**: sessions with `mode='record'`

### Technical Requirements
1. Continuous capture until explicit stop (vs hold-to-talk release)
2. Chunk audio every 60s for transcription
3. Write to rolling .txt file (not typed to apps)
4. No keystroke output in record mode

### Success Criteria
- Capture runs for hours without memory growth
- Transcription chunks written to rolling file
- Session persists in DB with mode='record'

### Entry Commands
```bash
cd .agents/worktrees/record-mode
export BEADS_NO_DAEMON=1
bd update stt-bq4 --status=in_progress
```

---

## Issue: stt-rdp - Record Mode File Rotation
- **Priority**: 1
- **Type**: task
- **Blocked by**: stt-bq4

### Description
Per PLAN.md 'Record Mode Bounds': auto-rotate to new file and new session row when maxHours (8) or maxFileGB (4) reached. No data loss. File rotation should trigger new session in DB.

### Technical Requirements
1. Rotate file on maxHours (8) OR maxFileGB (4)
2. Create new session row on rotation
3. No data loss during rotation

### Links
- PLAN.md: Record Mode Bounds
- Related: stt-83i (unit tests)

---

## Issue: stt-83i - Unit Tests: Record Mode Chunking/Rotation
- **Priority**: 3
- **Type**: chore
- **Blocked by**: stt-rdp

### Description
Tests for session/file rotation, writes to rolling .txt file, no typing to apps during record mode.

### Links
- PLAN.md section 12: Tests

# Worktree: session-metadata

**Plan Reference**: See [PLAN.md: Data Storage (SQLite)](../plan/SKILL.md#data-storage-sqlite)

## Issue
- **ID**: stt-3b0
- **Title**: Capture frontmost app name for session metadata
- **Priority**: 2
- **Type**: task

## Description
Per PLAN.md 'Data Storage': sessions table has app_name TEXT NULL (frontmost app at time of typing; best-effort). Need to detect frontmost macOS app before typing and store in session. This is a best-effort heuristic - use NSWorkspace or similar.

## What This Enables
This blocks:
1. **stt-t4k** - Session/entry persistence to PTT workflow (depends on app name being captured)

## Integration Points
- **Source file**: `src-tauri/src/session.rs` (modify existing)
- **Tauri commands**: Add to `apps/web/src-tauri/src/lib.rs`
- **Database**: `sessions` table - app_name column
- **macOS API**: NSWorkspace for frontmost app detection

## Technical Requirements
1. Use `NSWorkspace` (via Cocoa/Rust binding) to detect frontmost app
2. Capture app name before typing begins
3. Store in session record (app_name TEXT NULL)
4. Best-effort - fail gracefully if detection fails

## Success Criteria
- Captures frontmost app name before typing
- Stores in session metadata (nullable)
- Graceful fallback when detection unavailable
- No blocking of core PTT workflow

## Entry Commands
```bash
cd .agents/worktrees/session-metadata
export BEADS_NO_DAEMON=1
bd update stt-3b0 --status=in_progress
```

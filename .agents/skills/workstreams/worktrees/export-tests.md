# Worktree: export-tests

**Plan Reference**: See [PLAN.md: Export Format Details](../../plan/SKILL.md#export-format-details)

## Issue
- **ID**: stt-xz0
- **Title**: Unit tests: export functionality (txt/md)
- **Priority**: 3
- **Type**: chore

## Description
Per PLAN.md section 12: Tests: unit (export). Per PLAN.md section 'Export Format Details', tests .txt export with optional timestamps and session headers, .md export with session headings and code block mapping, voice command representation in exports, multi-entry selection, file save dialog integration. Acceptance: All tests pass, exports match format spec.

## What This Enables
Tests the export functionality - no blocking dependents but validates user-facing features.

## Integration Points
- **Source files**:
  - `apps/web/src/lib/export.ts` (or similar)
  - Tauri commands for file operations
- **Database queries**: sessions, entries tables
- **UI Components**: Export dialog, file picker

## Technical Requirements
1. Test .txt export format with:
   - Optional timestamps
   - Session headers
2. Test .md export format with:
   - Session headings
   - Code block mapping for transcripts
3. Voice command representation in exports
4. Multi-entry selection
5. File save dialog integration

## Export Format Details
Per PLAN.md:
- **.txt**: Plain text with optional timestamps
- **.md**: Markdown with headings for sessions, code blocks for transcripts

## Test Framework
Use Vitest or existing test patterns in project.

## Success Criteria
- All export format tests pass
- Exports match PLAN.md specification
- File save dialog works
- Multi-entry selection functional

# Worktree: model-tests

## Issue
- **ID**: stt-t00
- **Title**: Unit tests: model loading and SHA-256 verification
- **Priority**: 3
- **Type**: chore

## Description
Per PLAN.md section 12: Tests: unit (model loading). Per PLAN.md section 'Whisper Integration', uses whisper-rs with Metal when supported else CPU. Tests whisper model loading from path, SHA-256 file verification against known checksums, profile selection (small.en, multilingual-small, multilingual-medium), Metal fallback to CPU, context reuse across sessions, missing/invalid model error handling. Acceptance: All tests pass, Metal fallback works.

## What This Enables
Tests the model loading infrastructure - no blocking dependents but validates core functionality.

## Integration Points
- **Source files**: 
  - `src-tauri/src/stt.rs` (whisper integration)
  - `src-tauri/src/model.rs` (model loading)
- **Tests location**: `src-tauri/tests/` or `tests/` directory
- **Model profiles**: small.en, multilingual-small, multilingual-medium

## Technical Requirements
1. Test model loading from filesystem path
2. SHA-256 verification against known checksums
3. Profile selection tests (small.en, multilingual-small, multilingual-medium)
4. Metal fallback to CPU when Metal unavailable
5. Context reuse across multiple sessions
6. Error handling for missing/invalid models

## Test Framework
Use Vitest or Rust's built-in test framework (check existing test patterns in project).

## Known Model Checksums
- small.en: TBD (verify against huggingface)
- multilingual-small: TBD
- multilingual-medium: TBD

## Success Criteria
- All model loading tests pass
- Metal fallback works correctly
- SHA-256 verification functional
- Error cases handled gracefully

## Entry Commands
```bash
cd .agents/worktrees/model-tests
export BEADS_NO_DAEMON=1
bd update stt-t00 --status=in_progress
```

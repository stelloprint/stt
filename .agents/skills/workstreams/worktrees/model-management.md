# Worktree: model-management

**Plan Reference**: See [PLAN.md: Preferences & Files](../../plan/SKILL.md#preferences--files), [PLAN.md: Whisper Integration](../../plan/SKILL.md#whisper-integration), and [PLAN.md: UI](../../plan/SKILL.md#ui-react--tanstack-router)

## Purpose

This worktree replaces the old `model-ui` lane. The settings route already renders model-management UI, but the backend command contract and SHA-256 source of truth are still incomplete.

## Issue Chain

- **stt-bih** → stt-j53
- **Current**: `stt-bih`

## Shared Hot Files

- `apps/web/src-tauri/src/stt.rs`
- `apps/web/src-tauri/src/lib.rs`
- `apps/web/src/lib/api.ts`
- `apps/web/src/routes/settings.tsx`

## Conflict Guidance

- Do not run in parallel with `ptt-runtime` or `record-mode`
- Safe after `prefs-contract` has stabilized settings-schema behavior

## Issue: stt-bih - Verify And Replace Placeholder Whisper Model SHA-256 Constants

- **Priority**: 3
- **Type**: bug

### Focus

Fix the source of truth first. The hard-coded model hashes currently look synthetic, so model verification and the settings surface cannot be trusted until this issue lands.

### Technical Targets

1. Verify all supported filenames and hashes against real model artifacts.
2. Centralize filename/hash pairs in one backend source of truth.
3. Make valid files verify cleanly and invalid ones fail clearly.

## Issue: stt-j53 - Build UI: /settings Page - Model Management

- **Priority**: 2
- **Type**: task

### Focus

Complete the React-to-Tauri model-management contract after the hashes are trustworthy. The route currently depends on backend commands that are not exposed yet.

### Technical Targets

1. Implement `get_model_statuses`, `get_current_model`, and `verify_model`.
2. Register those commands in the Tauri backend.
3. Keep `/settings` loading even when models are missing.
4. Update verify actions against real backend results.

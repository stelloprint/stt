# Worktree: model-ui

**Plan Reference**: See [PLAN.md: Preferences & Files](../plan/SKILL.md#preferences--files) and [PLAN.md: UI (React + TanStack Router)](../plan/SKILL.md#ui-react--tanstack-router)

## Issue Chain
- **stt-j53** (only issue - no dependents yet)

## Issue: stt-j53 - Build UI: /settings Page - Model Management
- **Priority**: 2
- **Type**: task

### Description
Model management UI for /settings page. Per PLAN.md section 'Preferences & Files', models dir is `~/Library/Application Support/sst/models/` with verification via SHA-256. This task adds Tauri commands to lib.rs and React UI components for model status display and verification.

### API Location
- Commands: `apps/web/src-tauri/src/lib.rs`
- Structs: `apps/web/src-tauri/src/stt.rs`

### What This Enables
- Settings page shows model status
- Users can verify model integrity via SHA-256
- Download links to HuggingFace

### Technical Requirements
1. Settings shows model status for small.en, multilingual-small, multilingual-medium
2. Each profile shows: filename, expected SHA-256, installation status (missing/present/verified)
3. Verify button computes SHA-256 vs expected
4. Current loaded model displayed in HUD
5. Links to https://huggingface.co/ggerganov/whisper.cpp

### Integration Points
- **Frontend**: React components in `apps/web/src/`
- **Backend**: Tauri commands in `apps/web/src-tauri/src/`
- **Models**: `~/Library/Application Support/sst/models/`

### Success Criteria
- All 3 models display with correct status
- SHA-256 verification works correctly
- Download links functional

### Entry Commands
```bash
cd .agents/worktrees/model-ui
export BEADS_NO_DAEMON=1
bd update stt-j53 --status=in_progress
```

### Links
- PLAN.md: Preferences & Files
- PLAN.md: /settings Page
- Model downloads: https://huggingface.co/ggerganov/whisper.cpp

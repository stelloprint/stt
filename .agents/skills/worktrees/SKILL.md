# Git Worktrees

## Active Workstreams

Each worktree represents an ongoing workstream that handles a chain of dependent issues. Agents work through the full chain sequentially.

| Worktree | Issue Chain | Status | Docs |
|----------|-------------|--------|------|
| record-mode | [stt-rtf](#) → stt-bq4 → stt-rdp → stt-83i | In Progress: bq4 | [record-mode.md](./record-mode.md) |
| fix-compilation-bug | stt-im0 | Ready | Bug: test compilation errors in stt.rs |
| audio-tests | stt-28a | Ready | Unit tests: audio capture and silence detection |
| manual-qa | stt-8rv | Ready | Cross-cutting manual QA |

## Completed Workstreams

| Worktree | Final Issue | Status |
|----------|-------------|--------|
| audio-pipeline | stt-y3w (hotkey wiring) | Complete |
| model-ui | stt-j53 (model management UI) | Complete |
| session-persistence | stt-t4k (persistence wiring) | Complete |
| model-tests | stt-t00 (model loading/SHA-256) | Complete |
| export-tests | stt-xz0 (export functionality) | Complete |

## Dependency Graph

```
audio-pipeline (COMPLETE)
stt-rtf ──► stt-y3w ──► stt-1j5 ──► toggle mode
              │
              └─► stt-28a ──► audio tests

record-mode
stt-rtf ──► stt-bq4 ──► stt-rdp ──► stt-83i
              (in progress)

model-ui (COMPLETE) ──► stt-j53 ✓

session-persistence (COMPLETE)
stt-3b0 ──► stt-t4k ✓
```

## Workstream Details

### record-mode
- **Chain**: stt-rtf → stt-bq4 → stt-rdp → stt-83i
- **Current**: [stt-bq4](https://github.com/stelloprint/stt/issues/bq4) (Implement long-form record mode audio capture)
- **Unblocks**: file rotation, record mode tests
- **Docs**: [record-mode.md](./record-mode.md)

### fix-compilation-bug
- **Current**: [stt-im0](https://github.com/stelloprint/stt/issues/im0) (Fix test compilation errors in stt.rs)
- **Priority**: Bug - blocks all test execution
- **Fix**: Add missing VoiceCommandMap and VoiceCommands types to src/stt.rs

### audio-tests
- **Current**: [stt-28a](https://github.com/stelloprint/stt/issues/28a) (Unit tests: audio capture and silence detection)
- **Independent**: No dependencies after stt-rtf closed

### manual-qa
- **Current**: [stt-8rv](https://github.com/stelloprint/stt/issues/8rv) (Manual QA: hotkeys, hold/toggle, voice commands, record mode)
- **Note**: Cross-cutting - requires all core implementation complete

## Ready Issues

| Issue | Title | Type | Priority |
|-------|-------|------|-----------|
| stt-im0 | Fix test compilation errors in stt.rs | bug | 2 |
| stt-1j5 | Implement toggle mode with silence detection | task | 1 |
| stt-28a | Unit tests: audio capture and silence detection | chore | 3 |
| stt-8rv | Manual QA: hotkeys, hold/toggle, voice commands, record mode | chore | 3 |

## Blocked Issues

| Issue | Title | Blocked By |
|-------|-------|------------|
| stt-rdp | Implement record mode file rotation | stt-bq4 |
| stt-83i | Unit tests: record mode chunking and rotation | stt-rdp |

## Entering a Workstream

```bash
cd .agents/worktrees/<name>
export BEADS_NO_DAEMON=1

# Check what's ready to work
bd ready

# Claim the next available issue in the chain
bd update <issue-id> --status in_progress
```

## Progression Workflow

When an issue is completed:
1. Close it: `bd close <issue-id>`
2. Check what became ready: `bd ready`
3. Claim the next issue in your workstream chain

## Creating Worktrees

```bash
task.sh start record-mode
task.sh start fix-compilation-bug
task.sh start audio-tests
task.sh start manual-qa
```

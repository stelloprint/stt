# Git Worktrees

## Active Workstreams

Each worktree represents an ongoing workstream that handles a chain of dependent issues. Agents work through the full chain sequentially.

| Worktree | Issue Chain | Status |
|----------|-------------|--------|
| audio-pipeline | stt-rtf → stt-y3w → stt-pmb → stt-6a1 | Ready: rtf |
| record-mode | stt-rtf → stt-bq4 → stt-rdp → stt-83i | Ready: rtf (blocked by audio-pipeline) |
| model-ui | stt-rtf → stt-j53 | Ready: rtf (blocked by audio-pipeline) |
| session-persistence | stt-3b0 → stt-t4k | Ready: 3b0 |
| model-tests | stt-t00 | Ready |
| export-tests | stt-xz0 | Ready |

## Dependency Graph

```
stt-rtf ──► stt-y3w ──► stt-pmb ──► stt-6a1
    │
    ├─► stt-bq4 ──► stt-rdp ──► stt-83i
    │
    ├─► stt-1j5 ──► toggle mode (not in workstream)
    │
    ├─► stt-28a ──► audio tests (not in workstream)
    │
    └─► stt-j53 ──► model-ui

stt-3b0 ──► stt-t4k ──► persistence wiring
```

## Workstream Details

### audio-pipeline
- **Chain**: stt-rtf → stt-y3w → stt-pmb → stt-6a1
- **Current**: stt-rtf (Implement audio.rs module)
- **Unblocks**: hotkey wiring, transcription to typer, integration tests

### record-mode  
- **Chain**: stt-rtf → stt-bq4 → stt-rdp → stt-83i
- **Current**: stt-rtf (waits for audio-pipeline)
- **Note**: Depends on audio-pipeline completing rtf first

### model-ui
- **Chain**: stt-rtf → stt-j53
- **Current**: stt-rtf (waits for audio-pipeline)
- **Note**: Depends on audio-pipeline completing rtf first

### session-persistence
- **Chain**: stt-3b0 → stt-t4k
- **Current**: stt-3b0 (Capture frontmost app name)
- **Unblocks**: DB persistence wiring

### model-tests
- **Current**: stt-t00 (Unit tests: model loading and SHA-256)
- **Independent**: No dependencies

### export-tests
- **Current**: stt-xz0 (Unit tests: export functionality)
- **Independent**: No dependencies

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
task.sh start audio-pipeline
task.sh start record-mode
task.sh start model-ui
task.sh start session-persistence
task.sh start model-tests -e   # uses existing branch
task.sh start export-tests -e  # uses existing branch
```

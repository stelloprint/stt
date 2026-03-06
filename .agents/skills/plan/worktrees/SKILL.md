# Git Worktrees

| Worktree | Branch | Issue Id | Description |
|----------|--------|----------|-------------|
| audio-capture | audio-capture | stt-rtf | Implement audio.rs module (cpal capture, resampling, RMS silence) |
| session-metadata | session-metadata | stt-3b0 | Capture frontmost app name for session metadata |
| model-tests | model-tests | stt-t00 | Unit tests: model loading and SHA-256 verification |
| export-tests | export-tests | stt-xz0 | Unit tests: export functionality (txt/md) |

## Dependency Analysis

All 4 worktrees are independent (no blocking relationships between them):

- **audio-capture** blocks 5 other issues but has no incoming blockers (ready)
- **session-metadata** blocks 1 issue but has no incoming blockers (ready)
- **model-tests** has no blockers (ready)
- **export-tests** has no blockers (ready)

## Execution Order

All 4 can run in parallel as none block each other. However, audio-capture is critical since it unblocks 5 dependent issues (y3w, bq4, 1j5, j53, 28a).

Each worktree is ready for an agent to enter with:
```zsh
cd .agents/worktrees/<name>
export BEADS_NO_DAEMON=1
bd update <issue-id> --status in_progress
```

## Worktree Creation

```bash
task.sh start audio-capture
task.sh start session-metadata
task.sh start model-tests
task.sh start export-tests
```

# Git Worktrees

| Worktree | Branch | Issues |
|----------|--------|--------|
| .agents/worktrees/stt-kb1-permissions | stt-kb1-permissions | stt-kb1 → stt-4lb |
| .agents/worktrees/stt-btc-model-location | stt-btc-model-location | stt-btc → stt-atk → stt-t00 |
| .agents/worktrees/stt-51g-database-crud | stt-51g-database-crud | stt-51g → stt-527 → stt-xz0 |
| .agents/worktrees/stt-89l-react-ui | stt-89l-react-ui | stt-89l → stt-7st → stt-3b0 |


Each worktree is ready for an agent to enter with:
```zsh
cd .agents/worktrees/<name>
export BEADS_NO_DAEMON=1
bd update <issue-id> --status in_progress
```

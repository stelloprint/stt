# Worktree: session-persistence

## Status

Retired and replaced by [`ptt-runtime.md`](./ptt-runtime.md).

## Why This Was Rewritten

The earlier workstream stopped at `stt-3b0`, but the live tracker now shows that the real unfinished work is broader:

1. `stt-t4k` still needs the hotkey-driven runtime path wired to `SessionManager`
2. `stt-1j5` still needs toggle-mode runtime behavior
3. `stt-ied` reopens the frontmost-app metadata problem because the prior `3b0` implementation used the wrong macOS API
4. `stt-8cp` depends on a trustworthy runtime status surface

The active source of truth is now `ptt-runtime.md`, which owns the full `stt-t4k → stt-1j5 → stt-ied → stt-8cp` lane.

# Worktree: model-ui

## Status

Retired and replaced by [`model-management.md`](./model-management.md).

## Why This Was Rewritten

This worktree was marked complete too early. The current open tracker state shows that model-management still has two live pieces of work:

1. `stt-bih` for real Whisper model SHA-256 values
2. `stt-j53` for the missing backend command contract used by `/settings`

The active source of truth is now `model-management.md`, which chains `stt-bih → stt-j53` and treats the backend hash/command surface as part of the same lane.

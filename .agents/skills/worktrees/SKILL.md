# Git Worktrees

## Active Workstreams

Each worktree represents an ongoing workstream that handles a chain of dependent issues. Agents work through the full chain sequentially.

The current tracker state has several reopened issues on surfaces that were previously marked complete. The workstream plan below reflects the live `bd` dependency graph and is optimized to avoid agents colliding in `apps/web/src-tauri/src/lib.rs`, `apps/web/src-tauri/src/audio.rs`, and shared route/API contracts.

| Worktree | Issue Chain | Status | Docs |
|----------|-------------|--------|------|
| ptt-runtime | stt-t4k → stt-1j5 → stt-ied → stt-8cp | Ready | [ptt-runtime.md](./ptt-runtime.md) |
| record-mode | stt-bq4 → stt-rdp → stt-fuc → stt-83i | Ready | [record-mode.md](./record-mode.md) |
| prefs-contract | stt-im0 → stt-ir2 | Ready | [prefs-contract.md](./prefs-contract.md) |
| model-management | stt-bih → stt-j53 | Ready | [model-management.md](./model-management.md) |
| permissions-safety | stt-uwb | Ready | [permissions-safety.md](./permissions-safety.md) |
| typing-fallback | stt-3ih | Ready | [typing-fallback.md](./typing-fallback.md) |
| audio-tests | stt-28a | Ready late-stage | [audio-tests.md](./audio-tests.md) |
| route-tests | stt-moq | Ready late-stage | [route-tests.md](./route-tests.md) |
| manual-qa | stt-8rv | Blocked | [manual-qa.md](./manual-qa.md) |

## Scheduling Rules

Use these rules when assigning agents:

1. Only one `lib.rs`-heavy backend stream should be active at a time.
2. Treat `ptt-runtime`, `record-mode`, and `model-management` as serialized lanes, not fully parallel work.
3. `prefs-contract`, `permissions-safety`, and `typing-fallback` are the safest side lanes to run in parallel with one major backend lane.
4. `audio-tests` should wait until `audio.rs` behavior has stabilized after runtime work.
5. `route-tests` should wait until `/settings` and `/record` are wired to the real backend contracts.
6. `manual-qa` stays parked until its blocker set is actually closed.

## Recommended Execution Order

1. `prefs-contract`, `permissions-safety`, and `typing-fallback`
2. `ptt-runtime`
3. `record-mode`
4. `model-management`
5. `audio-tests` and `route-tests`
6. `manual-qa`

This order prioritizes correctness in the main push-to-talk flow first, then long-form recording, then model-management UX, and only then test hardening and cross-cutting QA.

## Retired Or Rewritten Workstreams

These workstreams should no longer be used as active lanes:

| Previous Worktree | Previous State | Replacement |
|-------------------|----------------|-------------|
| fix-compilation-bug | Too narrow and based on stale diagnosis | `prefs-contract` |
| session-persistence | Marked complete before the real hotkey path was wired | `ptt-runtime` |
| model-ui | Marked complete before backend model commands and real hashes were in place | `model-management` |

Historical completed workstreams such as `audio-pipeline`, `model-tests`, and `export-tests` remain useful as references, but they are no longer the source of truth for the current open issue plan.

## Workstream Details

### ptt-runtime
- **Chain**: stt-t4k → stt-1j5 → stt-ied → stt-8cp
- **Current**: `stt-t4k` (Wire session/entry persistence to PTT workflow)
- **Hot files**: `apps/web/src-tauri/src/lib.rs`, `apps/web/src-tauri/src/session.rs`, `apps/web/src-tauri/src/keys.rs`, `apps/web/src/routes/index.tsx`
- **Conflict note**: Serialize with `record-mode` and `model-management`

### record-mode
- **Chain**: stt-bq4 → stt-rdp → stt-fuc → stt-83i
- **Current**: `stt-bq4` (Implement long-form record mode audio capture)
- **Hot files**: `apps/web/src-tauri/src/audio.rs`, `apps/web/src-tauri/src/lib.rs`, `apps/web/src/routes/record.tsx`
- **Conflict note**: Serialize with `ptt-runtime` and `model-management`

### prefs-contract
- **Chain**: stt-im0 → stt-ir2
- **Current**: `stt-im0` (Diagnose current Rust test failure and stale prefs assumptions)
- **Hot files**: `apps/web/src-tauri/src/prefs.rs`, `apps/web/src/routes/settings.tsx`
- **Conflict note**: Safe to run alongside one major backend lane

### model-management
- **Chain**: stt-bih → stt-j53
- **Current**: `stt-bih` (Verify and replace placeholder Whisper model SHA-256 constants)
- **Hot files**: `apps/web/src-tauri/src/stt.rs`, `apps/web/src-tauri/src/lib.rs`, `apps/web/src/lib/api.ts`, `apps/web/src/routes/settings.tsx`
- **Conflict note**: Serialize with `ptt-runtime` and `record-mode`

### permissions-safety
- **Chain**: stt-uwb
- **Current**: `stt-uwb` (Replace destructive permission probes with safe macOS checks)
- **Hot files**: `apps/web/src-tauri/src/permissions.rs`
- **Conflict note**: Good parallel side lane

### typing-fallback
- **Chain**: stt-3ih
- **Current**: `stt-3ih` (Fix macOS clipboard fallback to paste with Command+V)
- **Hot files**: `apps/web/src-tauri/src/type_.rs`
- **Conflict note**: Good parallel side lane

### audio-tests
- **Chain**: stt-28a
- **Current**: `stt-28a` (Unit tests: audio capture and silence detection)
- **Hot files**: `apps/web/src-tauri/src/audio.rs`
- **Conflict note**: Defer until runtime/audio behavior stops moving

### route-tests
- **Chain**: stt-moq
- **Current**: `stt-moq` (Replace placeholder route tests with real route and API wiring coverage)
- **Hot files**: `apps/web/src/routes/`, `apps/web/src/test/mocks.ts`, `apps/web/src/lib/api.ts`
- **Conflict note**: Defer until `model-management` and `record-mode` frontend contracts are real

### manual-qa
- **Chain**: stt-8rv
- **Current**: `stt-8rv` (Manual QA: hotkeys, hold/toggle, voice commands, record mode)
- **Blockers**: `stt-1j5`, `stt-3ih`, `stt-fuc`, `stt-t4k`, `stt-uwb`
- **Note**: Cross-cutting final lane only

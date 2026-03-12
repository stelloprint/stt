---
description: Plans parallel workstreams
agent: build, plan
---

# Git Worktrees

## Active Workstreams

Each worktree represents an ongoing workstream that handles a chain of dependent issues. Agents work through the full chain sequentially.

The table below reflects the live `bd` dependency graph after code acceptance for `prefs-contract`, `typing-fallback`, and the first `permissions-safety` change. The remaining active plan is optimized to avoid agents colliding in `apps/web/src-tauri/src/lib.rs`, `apps/web/src-tauri/src/audio.rs`, and shared route/API contracts.

| Worktree | Issue Chain | Status | Docs |
|----------|-------------|--------|------|
| ptt-runtime | stt-t4k → stt-a86 → stt-1j5 → stt-ied → stt-8cp | Active | [ptt-runtime.md](./ptt-runtime.md) |
| record-mode | stt-bq4 → stt-rdp → stt-fuc → stt-83i | Ready | [record-mode.md](./record-mode.md) |
| model-management | stt-bih → stt-j53 | Ready | [model-management.md](./model-management.md) |
| permissions-safety | stt-uwb → stt-43q | Active follow-up | [permissions-safety.md](./permissions-safety.md) |
| audio-tests | stt-28a | Ready late-stage | [audio-tests.md](./audio-tests.md) |
| route-tests | stt-moq | Ready late-stage | [route-tests.md](./route-tests.md) |
| manual-qa | stt-8rv | Blocked | [manual-qa.md](./manual-qa.md) |

## Scheduling Rules

Use these rules when assigning agents:

1. Only one `lib.rs`-heavy backend stream should be active at a time.
2. Treat `ptt-runtime`, `record-mode`, and `model-management` as serialized lanes, not fully parallel work.
3. `permissions-safety` is the safest remaining side lane to run in parallel with one major backend lane.
4. `audio-tests` should wait until `audio.rs` behavior has stabilized after runtime work.
5. `route-tests` should wait until `/settings` and `/record` are wired to the real backend contracts.
6. `manual-qa` stays parked until its blocker set is actually closed.

## Recommended Execution Order

1. Create `permissions-safety` and `ptt-runtime` now
2. After `ptt-runtime` lands, create `record-mode`
3. After `record-mode` stabilizes, create `model-management`
4. After backend behavior settles, create `audio-tests` and `route-tests`
5. Run `manual-qa` last once `stt-1j5` and `stt-fuc` are closed

With all old worktrees cleared, the best next start is two lanes in parallel: the isolated `permissions-safety` follow-up and the serialized `ptt-runtime` lane. Keep `record-mode` and `model-management` queued rather than active at the same time, even if tracker status says ready, because they share backend surfaces with `ptt-runtime`.

## Completed Workstreams

These workstreams have accepted code and no remaining open issues in their original chain:

| Worktree | Accepted Chain | State | Docs |
|----------|----------------|-------|------|
| prefs-contract | stt-im0 → stt-ir2 | Complete | [prefs-contract.md](./prefs-contract.md) |
| typing-fallback | stt-3ih | Complete | [typing-fallback.md](./typing-fallback.md) |

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
- **Chain**: stt-t4k → stt-a86 → stt-1j5 → stt-ied → stt-8cp
- **Current**: `stt-a86`
- **Accepted**: `stt-t4k` is complete
- **Follow-up note**: `stt-a86` is regression coverage for the persistence contract introduced by `stt-t4k` and should land before the rest of the runtime chain
- **Hot files**: `apps/web/src-tauri/src/lib.rs`, `apps/web/src-tauri/src/session.rs`, `apps/web/src-tauri/src/keys.rs`, `apps/web/src/routes/index.tsx`
- **Conflict note**: Serialize with `record-mode` and `model-management`

### record-mode
- **Chain**: stt-bq4 → stt-rdp → stt-fuc → stt-83i
- **Current**: `stt-bq4` (Implement long-form record mode audio capture)
- **Hot files**: `apps/web/src-tauri/src/audio.rs`, `apps/web/src-tauri/src/lib.rs`, `apps/web/src/routes/record.tsx`
- **Conflict note**: Serialize with `ptt-runtime` and `model-management`

### model-management
- **Chain**: stt-bih → stt-j53
- **Current**: `stt-bih` (Verify and replace placeholder Whisper model SHA-256 constants)
- **Hot files**: `apps/web/src-tauri/src/stt.rs`, `apps/web/src-tauri/src/lib.rs`, `apps/web/src/lib/api.ts`, `apps/web/src/routes/settings.tsx`
- **Conflict note**: Serialize with `ptt-runtime` and `record-mode`

### permissions-safety
- **Chain**: stt-uwb → stt-43q
- **Current**: `stt-43q`
- **Accepted**: `stt-uwb` is complete
- **Hot files**: `apps/web/src-tauri/src/permissions.rs`
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
- **Blockers**: `stt-1j5`, `stt-fuc`
- **Note**: Cross-cutting final lane only

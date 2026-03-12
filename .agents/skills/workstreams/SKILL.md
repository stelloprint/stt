---
description: Scheduling policy for parallel agent workstreams
agent: build, plan
---

# Workstream Scheduling Policy

All issue state, chains, descriptions, and acceptance criteria live in the beads tracker (`bd`). This document only defines the **parallel/serial scheduling policy** that beads cannot infer on its own.

Use `bd ready --json` to find unblocked work, then apply these rules to decide what is safe to run together.

## Serialized Lanes

These three lanes share heavy write surfaces in `lib.rs`, `audio.rs`, and shared runtime state. **Only one should have an active agent at a time.**

| Lane | Epic | Conflict Groups |
|------|------|-----------------|
| ptt-runtime | stt-jsa | tauri-lib, runtime-hotkey |
| record-mode | stt-vxq | tauri-lib, audio-runtime, record-route |
| model-management | stt-fm4 | tauri-lib, model-contract, settings-route |

Even if `bd ready` shows issues from multiple serialized lanes, do not assign agents to more than one simultaneously.

## Parallel Side Lanes

These lanes touch isolated surfaces and are safe to run alongside any one serialized lane.

| Lane | Epic | Notes |
|------|------|-------|
| permissions-safety | stt-5up | Only touches `permissions.rs` |
| hardening (CSP/allowlist) | stt-ef6 | Only touches `tauri.conf.json` and capabilities |
| tray-integration | stt-oo2 | Tray API, mostly new files |
| packaging | stt-ie6 | Build verification, no source changes |

## Deferred Lanes

These lanes should not start until their upstream implementation surfaces stabilize.

| Lane | Epic | Wait Until |
|------|------|------------|
| audio-tests | stt-wyr | ptt-runtime and record-mode finish changing `audio.rs` |
| route-tests | stt-6ez | model-management and record-mode land frontend contracts |
| manual-qa | stt-u6f | Tracker blockers on stt-8rv are all closed |

## Scheduling Rules

1. Only one serialized lane active at a time.
2. Up to two parallel side lanes can run alongside the active serialized lane.
3. Deferred lanes stay parked until their wait condition is met.
4. `manual-qa` is always last.
5. If an agent finishes a serialized lane and another is ready, start it immediately.
6. Check `bd ready --json` and filter by `conflict_groups` metadata before assigning.

## Recommended Execution Order

1. **Now**: `ptt-runtime` (serialized) + `permissions-safety` (parallel side)
2. **After ptt-runtime progresses past stt-1j5**: `record-mode` can start (serialized)
3. **After record-mode stabilizes**: `model-management` (serialized)
4. **After backend surfaces settle**: `audio-tests` + `route-tests` (parallel with each other)
5. **Last**: `manual-qa` once all its tracker blockers close
6. **Anytime**: `hardening`, `tray-integration`, `packaging` as capacity allows

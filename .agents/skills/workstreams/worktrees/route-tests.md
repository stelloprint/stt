# Worktree: route-tests

**Plan Reference**: See [PLAN.md: UI](../../plan/SKILL.md#ui-react--tanstack-router)

## Purpose

This worktree replaces placeholder route tests with real route and API-contract coverage after the relevant runtime surfaces are wired for real.

## Issue Chain

- **stt-moq**
- **Current**: `stt-moq`

## Shared Hot Files

- `apps/web/src/routes/`
- `apps/web/src/test/mocks.ts`
- `apps/web/src/lib/api.ts`

## Conflict Guidance

- Late-stage lane
- Best after `model-management` and `record-mode` have landed their route/API contracts

## Issue: stt-moq - Replace Placeholder Route Tests With Real Route And API Wiring Coverage

- **Priority**: 3
- **Type**: chore

### Focus

Many current route tests assert copied helper logic or constants instead of rendering the real routes against the actual API contract. This lane should harden tests around `/settings`, `/record`, `/logs`, and navigation.

### Technical Targets

1. Render real routes/components instead of helper-only tests.
2. Add complete API mocks for the route dependencies.
3. Make the suite fail on missing route-level backend contracts.
4. Cover loader/action behavior rather than only formatting helpers.

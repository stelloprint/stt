# Agent Instructions

This is a Speech-To-Text app for local and secure use on MacOS without sending data to third parties.

## Development Commands

**Package manager and runtime:** Bun at the repo root. Workspaces live under `apps/*` and `packages/*`. See root and workspace `package.json` files for all scripts.

**Typical workflow after code changes**

1. **Format and lint (JS/TS/JSON/CSS)** — Biome via Ultracite (see below). Auto-fix what you can, then verify:
   - `bun run fix` — apply safe fixes and formatting (run before commit).
   - `bun run check` — read-only; matches what pre-push enforces for the JS stack.
2. **Types** — all workspaces: `bun run check-types`.
3. **Tests** — web (Vitest): `bun run --filter web test`; watch mode: `bun run --filter web test:watch`. Rust (Tauri crate): from `apps/web/src-tauri/`, `cargo test --all-features`. Pre-push does **not** run Vitest; run web tests when you change app or test code.
4. **Rust-only changes** (`apps/web/src-tauri/`) — `cargo fmt`, `cargo clippy --all-targets --all-features -- -D warnings`, and `cargo check --all-targets --all-features` as needed. `cargo fmt -- --check` verifies without writing files.

**Build and dev**

- `bun run build` — build all workspace packages (pre-push runs this).
- `bun run dev` — dev for all filtered packages; `bun run dev:web` / `bun run dev:native` for specific apps.

---

<!-- BEGIN BEADS INTEGRATION v:1 profile:minimal hash:ca08a54f -->
## Beads Issue Tracker

This project uses **bd (beads)** for issue tracking. Run `bd prime` to see full workflow context and commands.

### Quick Reference

```bash
bd ready              # Find available work
bd show <id>          # View issue details
bd update <id> --claim  # Claim work
bd close <id>         # Complete work
```

### Rules

- Use `bd` for ALL task tracking — do NOT use TodoWrite, TaskCreate, or markdown TODO lists
- Run `bd prime` for detailed command reference and session close protocol
- Use `bd remember` for persistent knowledge — do NOT use MEMORY.md files

<!-- END BEADS INTEGRATION -->

---

## Workflow

**MANDATORY WORKFLOW:**
0. **Find a task** - Get assigned to an issue or check for ready work: `bd ready`
1. **Understand the task** - Use `bd show <id>` to fully understand the ticket including epics, parent/child chains, dependencies and contracts.
2. **Claim an issue** - `bd update <id> --status=in_progress`so other agents don't work on it
3. **Do the work**
4. **Run quality gates** (if code changed) - Tests, linters, formatters and builds
5. **Review** - Wait for human review of your work and suggest issues for anything that needs follow-up
6. **Commit & Close** Upon approval, commit changes with a detailed message and mark complete: `bd close <id>`
7. **File issues for remaining work** - Create issues for the follow up items approved by the reviewer
8. **Clean up** - Clear stashes, prune remote branches
9. **Verify** - All changes committed AND clean up is complete
10. **Hand off** - Provide context for the next session

**CRITICAL RULES:**
- Work is NOT complete until review is passed and hand off is complete
- NEVER stop before committing - that leaves work stranded locally

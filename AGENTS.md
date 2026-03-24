# Agent Instructions

This is a Speech-To-Text app for local and secure use on MacOS without sending data to third parties.

## Development Commands

Bun is the package manager and runtime for this project. Refer to `package.json` for supported scripts.

Most formatting and common issues are automatically fixed. Run `bun run fix` before committing.

## Ultracite Code Standards

This project uses **Ultracite**, a zero-config preset that enforces strict code quality standards through automated formatting and linting. The *ultracite* skill describes how to use the CLI and the *code-standards* reference file outline quality expectations for this organization. Make sure to abide by these standards when reviewing code before committing.

Biome (the underlying engine) provides robust linting and formatting. Most issues are automatically fixable.

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

## Session Completion

**When ending a work session**, you MUST complete ALL steps below. Work is NOT complete until `git push` succeeds.

**MANDATORY WORKFLOW:**
0. **Find a task** - Get assigned to an issue or check for ready work: `bd ready`
4. **Understand the task** - Use `bd show <id>` to fully understand the ticket including epics, parent/child chains, dependencies and contracts.
1. **Claim an issue** - `bd update <id> --status=in_progress`
2. **Do the work**
2. **Run quality gates** (if code changed) - Tests, linters, builds
5. **Review** - Wait for human review of your work and suggest issues for anything that needs follow-up
6. **Commit & Close** Upon approval, commit changes with a detailed message and mark complete: `bd close <id>`
4. **File issues for remaining work** - Create issues for approved follow up items
5. **Clean up** - Clear stashes, prune remote branches
6. **Verify** - All changes committed AND clean up is complete
7. **Hand off** - Provide context for next session

**CRITICAL RULES:**
- Work is NOT complete until review is passed and hand off is complete
- NEVER stop before committing - that leaves work stranded locally
<!-- END BEADS INTEGRATION -->

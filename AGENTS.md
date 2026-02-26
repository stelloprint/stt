# Agent Instructions

This is a Speech-To-Text app for local and secure use on MacOS without sending data to third parties.

## Directory Structure

```
stt/                         # Turborepo monorepo
├── apps/
│   └── web/                 # React UI (TanStack Router)
│       ├── src/
│       │   ├── main.tsx
│       │   ├── routes/     # /, /logs, /settings, /record
│       │   ├── components/ # LogTable, ExportDialog, HUD
│       │   └── lib/        # client utils (formatters)
│       ├── src-tauri/
│       │   ├── src/
│       │   │   ├── main.rs # tauri entry; commands wiring
│       │   │   └── lib.rs  # module declarations
│       │   ├── Cargo.toml
│       │   └── tauri.conf.json
│       └── package.json
├── packages/
│   ├── config/              # shared config types/utilities
│   └── env/                 # environment variables/types
├── .gitignore
├── package.json
├── tsconfig.json
├── turbo.json
└── biome.json
```

## Development Commands

Bun is the package manager and runtime for this project. Refer to `package.json` for supported scripts.

Most formatting and common issues are automatically fixed. Run `bun run fix` before committing.

---

## Issue Tracking

This project uses **bd** (beads) for issue tracking. Run `bd onboard` to get started or the *tracking-issues* skill.

### Landing the Plane (Session Completion)

**When ending a work session**, you MUST complete ALL steps below. Work is NOT complete until `git push` succeeds.

**MANDATORY WORKFLOW:**

1. **File issues for remaining work** - Create issues for anything that needs follow-up
2. **Run quality gates** (if code changed):
   ```zsh
   bun run check      # Lint
   bun run check-types # Types
   bun run fix # Auto Fix
   bun run build      # Build
   ```
3. **Update issue status** - Close finished work, update in-progress items
4. **PUSH TO REMOTE** - This is MANDATORY:
   ```zsh
   git pull --rebase
   bd sync
   git push
   git status  # MUST show "up to date with origin"
   ```
5. **Clean up** - Clear stashes, prune remote branches
6. **Verify** - All changes committed AND pushed
7. **Hand off** - Provide context for next session

**CRITICAL RULES:**
- Work is NOT complete until `git push` succeeds
- NEVER stop before pushing - that leaves work stranded locally
- NEVER say "ready to push when you are" - YOU must push
- If push fails, resolve and retry until it succeeds

---

## Ultracite Code Standards

This project uses **Ultracite**, a zero-config preset that enforces strict code quality standards through automated formatting and linting. 

The *ultracite* skill describes how to use the CLI and the *code-standards* reference file outline quality expectations for this organization. Make sure to abide by these standards when reviewing code before committing.

Biome (the underlying engine) provides robust linting and formatting. Most issues are automatically fixable.

---

## Testing

- Write assertions inside `it()` or `test()` blocks
- Avoid done callbacks in async tests - use async/await instead
- Don't use `.only` or `.skip` in committed code

**No test suite exists yet.** If adding tests, use Vitest.

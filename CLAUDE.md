# GitShow — Git GUI Desktop Client

## Overview

GitShow is a personal Git GUI desktop client inspired by GitKraken's UX, built as a lightweight, focused alternative without the collaborative features or subscription cost. The goal is a fast, intuitive Git client for a solo developer workflow.

## Stack

| Layer      | Technology                  | Notes                                        |
|------------|-----------------------------|----------------------------------------------|
| Framework  | **Tauri v2** (v2.10+)       | Cross-platform desktop shell (Mac/Win/Linux) |
| Frontend   | **Svelte 5** + TypeScript   | Using runes (`$state`, `$derived`, `$effect`) |
| Backend    | **Rust** (Tauri backend)    | Tauri commands, shell to `git` CLI           |
| Diff/Merge | **Monaco Editor**           | Diff viewer, 3-way conflict resolution       |
| Graph      | **SVG** + custom layout     | Interactive commit graph with drag & drop     |
| Styling    | **CSS** (custom properties) | Design tokens, dark/light themes              |
| Build/CI   | **GitHub Actions**          | `tauri-apps/tauri-action` for cross-platform builds |

### Key Dependencies

- `monaco-editor` — loaded via dynamic import (avoid SSR issues in Svelte). Use the vanilla API directly, not a React wrapper. Setup Monaco workers using Vite's `?worker` import syntax.
- `@tauri-apps/api` — Tauri JS API for invoking Rust commands and managing shell processes.
- `@tauri-apps/plugin-shell` — for executing `git` CLI commands from the Tauri backend.

### Important Version Notes

- **Tauri v2 only** — do NOT use Tauri v1 APIs. The v2 API uses plugins for shell, fs, etc. Always check `https://v2.tauri.app/` for current docs.
- **Svelte 5 only** — use runes (`$state`, `$derived`, `$effect`), NOT the legacy `$:` reactive syntax or stores. Use `{#snippet}` instead of slots.
- Monaco editor does NOT have native 3-way merge support — the conflict resolution UI will need to be custom-built on top of Monaco's diff editor.

## Architecture

```
src/
├── lib/
│   ├── components/         # Svelte components
│   │   ├── graph/          # Commit graph (SVG-based)
│   │   ├── diff/           # Monaco-based diff viewer
│   │   ├── merge/          # Conflict resolution UI
│   │   ├── stash/          # Stash management panel
│   │   └── common/         # Shared UI components (buttons, modals, context menus)
│   ├── services/           # Business logic layer
│   │   ├── git.ts          # Git operations (calls Tauri commands)
│   │   ├── graph.ts        # Commit graph data model & layout algorithm
│   │   ├── diff.ts         # Diff parsing & formatting
│   │   └── settings.ts     # User preferences
│   ├── stores/             # Svelte 5 reactive state (using runes, NOT legacy stores)
│   │   ├── repo.svelte.ts  # Current repository state
│   │   ├── ui.svelte.ts    # UI state (selected commit, panels, etc.)
│   │   └── config.svelte.ts# App configuration
│   ├── types/              # TypeScript type definitions
│   └── utils/              # Pure utility functions
├── routes/                 # SvelteKit routes (if using SvelteKit) or App.svelte entry
└── app.html

src-tauri/
├── src/
│   ├── main.rs             # Tauri app entry
│   ├── commands/           # Tauri command handlers
│   │   ├── git.rs          # Git CLI wrapper commands
│   │   ├── repo.rs         # Repository management
│   │   └── config.rs       # Settings persistence
│   └── git/                # Git parsing logic (Rust)
│       ├── log.rs          # Parse git log output
│       ├── status.rs       # Parse git status output
│       ├── diff.rs         # Parse git diff output
│       └── types.rs        # Shared Rust types
├── Cargo.toml
└── tauri.conf.json

docs/
├── features/
│   ├── commit-graph.md     # Graph layout algorithm, interaction design
│   ├── diff-viewer.md      # Monaco integration, diff modes
│   ├── conflict-resolution.md  # 3-way merge UI design
│   ├── drag-drop-ops.md    # Merge, rebase, cherry-pick via drag & drop
│   ├── stash-management.md # Stash list, apply, pop, drop
│   └── auto-fetch.md       # Background fetch with prune
└── references.md           # Links to reference projects and resources
```

## Git CLI Strategy

We shell out to the `git` CLI rather than using a Git library. This is the same approach VS Code uses and it guarantees 100% compatibility.

### Key Commands & Formats

| Operation      | Command                                                                 |
|----------------|-------------------------------------------------------------------------|
| Commit log     | `git log --all --graph --format='%H%x00%P%x00%an%x00%ae%x00%at%x00%D%x00%s' --topo-order` |
| Status         | `git status --porcelain=v2 --branch`                                    |
| Diff (file)    | `git diff [--cached] -- <path>`                                         |
| Diff (commits) | `git diff <sha1>..<sha2> -- <path>`                                     |
| Stage          | `git add -- <paths>`                                                    |
| Unstage        | `git restore --staged -- <paths>`                                       |
| Commit         | `git commit -m "<message>"`                                             |
| Merge          | `git merge <branch> [--no-ff]`                                          |
| Rebase         | `git rebase <branch>`                                                   |
| Cherry-pick    | `git cherry-pick <sha>`                                                 |
| Stash list     | `git stash list --format='%gd%x00%H%x00%s%x00%at'`                     |
| Stash apply    | `git stash apply <stash@{n}>`                                           |
| Fetch          | `git fetch --all --prune`                                               |
| Refs           | `git for-each-ref --format='%(refname)%00%(objectname)%00%(upstream)%00%(upstream:track)' refs/heads refs/remotes refs/tags` |

### Parsing Rules

- Use `--porcelain` or `--format` flags for machine-readable output — never parse human-readable git output.
- Use NUL (`%x00`) as delimiter in `--format` strings for safe parsing.
- All git commands are executed via Tauri's shell plugin as async Rust commands, invoked from the frontend with `invoke()`.

## Development Phases

### Phase 1 — MVP: Graph + Basic Operations
- Interactive commit graph (SVG) with branch visualization
- Stage, unstage, commit
- Push, pull, fetch
- Branch creation, checkout
- Basic file diff viewer (Monaco, inline mode)

### Phase 2 — Drag & Drop Power Operations
- Drag branch → branch: merge or rebase (context menu)
- Drag commit → branch: cherry-pick
- Right-click context menus on commits, branches, tags
- Reset (soft/mixed/hard) via context menu

### Phase 3 — Diff & Conflict Resolution
- Side-by-side diff with syntax highlighting (Monaco diff editor)
- Inline vs. side-by-side toggle
- 3-way conflict resolution UI (custom, built on top of Monaco)
- Accept current / incoming / both actions per conflict hunk

### Phase 4 — Stash, Auto-fetch & Polish
- Stash list panel with apply, pop, drop, and preview
- Auto-fetch in background (configurable interval) with `--prune`
- Keyboard shortcuts
- Settings panel (default branch, fetch interval, diff preferences, theme)
- Performance: virtualized scroll for large repos (1000+ commits visible)

## UI/UX Principles

- **Dark theme first** — light theme as secondary option.
- **Single-window layout** — graph on the left, detail panel on the right (commit details / diff / merge).
- **Drag & drop is core** — dragging branches onto each other is the primary way to merge/rebase/cherry-pick.
- **Context menus everywhere** — right-click on any graph element for relevant actions.
- **No unnecessary confirmations** — but destructive operations (reset hard, force push) require explicit confirmation.
- **Auto-fetch silently** — show remote changes in the graph with upstream tracking indicators.

## Conventions

- All TypeScript files use strict mode.
- Svelte components use `<script lang="ts">`.
- Component files: PascalCase (e.g., `CommitGraph.svelte`, `DiffViewer.svelte`).
- Service/utility files: camelCase (e.g., `git.ts`, `graphLayout.ts`).
- Rust files: snake_case per Rust conventions.
- Tauri commands are async and return `Result<T, String>` — handle errors on the frontend with try/catch.
- CSS uses custom properties for theming: `--color-bg-primary`, `--color-text-primary`, etc.
- Feature documentation lives in `docs/features/<feature>.md` — Claude Code should read the relevant doc before working on a feature.

## Reference Projects

These open-source projects serve as architectural and design references:

### GitButler (Primary Reference)
- **Repo:** https://github.com/gitbutlerapp/gitbutler
- **Stack:** Tauri + Svelte + TypeScript (frontend), Rust (backend) — same stack as GitShow
- **Relevance:** Proves the Tauri + Svelte combo works for a production Git client. Study their Tauri command structure, Svelte state management patterns, and component architecture. Founded by Scott Chacon (co-founder of GitHub, author of Pro Git).
- **License:** Fair Source (FSL-1.1-MIT) — can study code, cannot build a competitor. Becomes MIT after 2 years.
- **Key areas to reference:**
  - `src-tauri/` — Rust backend command structure
  - Component library and design token system (`packages/ui/`)
  - How they handle Tauri ↔ Svelte communication

### Other References
- **Gittyup** (https://github.com/Murmele/Gittyup) — C++/Qt Git GUI, good reference for graph layout algorithms.
- **gitui** (https://github.com/extrawurst/gitui) — Terminal Git UI in Rust, excellent reference for fast git operations in Rust.
- **lazygit** (https://github.com/jesseduffield/lazygit) — Terminal Git UI, reference for keyboard-driven UX and operation workflows.

## Monaco Editor Integration Notes

- Load Monaco dynamically in `onMount` to avoid SSR issues.
- Setup workers using Vite's `?worker` import syntax:
  ```ts
  import editorWorker from 'monaco-editor/esm/vs/editor/editor.worker?worker';
  ```
- Use `monaco.editor.createDiffEditor()` for the diff view.
- Monaco does NOT have built-in 3-way merge — the conflict resolution UI is custom:
  - Parse conflict markers from `git diff` output
  - Show base/current/incoming in a custom layout using multiple Monaco instances
  - Provide "Accept Current", "Accept Incoming", "Accept Both" actions per hunk
- Theme Monaco to match the app's CSS custom properties.

## CI/CD

Cross-platform builds via GitHub Actions using `tauri-apps/tauri-action`:
- macOS: `.dmg` / `.app` (WebKit)
- Windows: `.msi` / `.exe` (WebView2)
- Linux: `.deb` / `.AppImage` (WebKitGTK)

Each push to `main` triggers builds for all platforms. Tauri cannot cross-compile — GitHub Actions handles this by running builds on `macos-latest`, `windows-latest`, and `ubuntu-latest` runners.
# References & Resources

## Reference Projects

### GitButler (Primary Architectural Reference)
- **Website:** https://gitbutler.com
- **Repository:** https://github.com/gitbutlerapp/gitbutler
- **Stack:** Tauri v2 + Svelte 5 + TypeScript (frontend), Rust (backend)
- **Stars:** ~20,000 (as of April 2026)
- **Founded by:** Scott Chacon (co-founder of GitHub, author of *Pro Git*)
- **License:** FSL-1.1-MIT (Fair Source License — becomes MIT after 2 years)
- **Why it matters:** GitButler is the most directly comparable project to GitShow in terms of technology stack. It validates that Tauri + Svelte is a production-viable combo for a Git desktop client.
- **Key areas to study:**
  - Tauri command patterns: `src-tauri/src/`
  - Svelte state management with Redux Toolkit Query + Svelte reactivity: `apps/desktop/src/lib/state/`
  - UI component library and design tokens: `packages/ui/`
  - How they handle Tauri ↔ Svelte communication via `invoke()`
  - Workspace and hunk assignment model
  - Rebase and commit manipulation engine
- **DeepWiki documentation:** https://deepwiki.com/gitbutlerapp/gitbutler (auto-generated architecture docs)
- **Note:** GitButler uses "virtual branches" which is a different paradigm from traditional Git branching. GitShow does NOT adopt this concept — we use standard Git branches. Reference their tech, not their Git model.

### Gittyup
- **Repository:** https://github.com/Murmele/Gittyup
- **Stack:** C++ / Qt
- **Relevance:** Good reference for commit graph layout algorithms. Their graph rendering approach handles complex branch/merge topologies well.

### gitui
- **Repository:** https://github.com/extrawurst/gitui
- **Stack:** Rust (terminal UI with `tui-rs`)
- **Relevance:** Fast, async Git operations implemented in Rust. Reference for how to efficiently call Git from Rust and parse output.

### lazygit
- **Repository:** https://github.com/jesseduffield/lazygit
- **Stack:** Go (terminal UI)
- **Relevance:** Excellent keyboard-driven UX. Good reference for operation workflows (interactive rebase, conflict resolution flow, stash management).

---

## Technology Documentation

### Tauri v2
- **Official docs:** https://v2.tauri.app/
- **GitHub:** https://github.com/tauri-apps/tauri
- **Current version:** v2.10.3 (March 2026)
- **Shell plugin:** https://v2.tauri.app/plugin/shell/ (required for executing `git` CLI)
- **GitHub Actions (cross-platform builds):** https://github.com/tauri-apps/tauri-action
- **Awesome Tauri (community resources):** https://github.com/tauri-apps/awesome-tauri

### Svelte 5
- **Official docs:** https://svelte.dev/docs
- **Runes reference:** https://svelte.dev/docs/svelte/$state
- **Migration guide (from Svelte 4):** https://svelte.dev/docs/svelte/v5-migration-guide
- **Current version:** 5.53+ (March 2026)
- **SvelteKit:** https://svelte.dev/docs/kit (if we decide to use SvelteKit as the framework within Tauri)

### Monaco Editor
- **GitHub:** https://github.com/microsoft/monaco-editor
- **Playground:** https://microsoft.github.io/monaco-editor/playground.html
- **Diff editor API:** `monaco.editor.createDiffEditor(container, options)`
- **Svelte integration guide:** https://www.codelantis.com/blog/sveltekit-monaco-editor
- **Svelte-focused API reference (community):** https://gist.github.com/archiewood/44e771428cbf3a4ba953b4101c4bf24b
- **Key notes:**
  - No native 3-way merge support — must be custom-built
  - Workers must be set up via Vite `?worker` imports
  - Dynamic import in `onMount()` to avoid SSR issues
  - Bundle is large (~2MB) — consider lazy loading only when diff panel opens

---

## Design Inspiration

### GitKraken (UX Target)
- **What we emulate:** The commit graph visualization, drag & drop branch operations, stash panel, diff viewer modes, auto-fetch behavior.
- **What we skip:** Workspaces, team features, issue tracker integrations, Jira/Trello boards, pull request management, GitKraken Boards, GitKraken Timelines.

### Graph Layout References
- Railroad-style commit graph layout (like GitKraken/git log --graph)
- Branch color assignment: hash branch name → HSL color for consistency
- Merge line routing: avoid unnecessary crossings, use Sugiyama-style layering
- Virtualized rendering for performance with large repos
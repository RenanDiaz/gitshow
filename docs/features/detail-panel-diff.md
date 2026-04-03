# Feature: Commit Detail Panel & Diff Viewer

## Overview

When the user selects a commit in the graph (click on node or row), a detail panel opens on the right side of the window. This panel shows commit metadata, the list of changed files, and a diff viewer powered by Monaco Editor. This is the second most-used view after the graph itself.

## Layout

```
┌─────────────────────────────────────────────────────────┐
│  GitShow                                                │
├──────────────────────┬──────────────────────────────────┤
│                      │  Commit Detail Header            │
│                      │  ┌────────────────────────────┐  │
│                      │  │ SHA: e5f6a7b (full on hover)│  │
│   Commit Graph       │  │ Author: Renan Diaz          │  │
│   (left panel)       │  │ Date: Mar 31, 2025 14:32    │  │
│                      │  │ Message: Merge branch...     │  │
│                      │  │ Parents: b2c3d4e, d4e5f6a   │  │
│                      │  └────────────────────────────┘  │
│                      │                                  │
│                      │  Changed Files List              │
│                      │  ┌────────────────────────────┐  │
│                      │  │ M  src/auth/login.ts    +24 -3│
│                      │  │ A  src/auth/validate.ts +48   │
│                      │  │ M  README.md            +2  -1│
│                      │  └────────────────────────────┘  │
│                      │                                  │
│                      │  Diff Viewer (Monaco)            │
│                      │  ┌────────────────────────────┐  │
│                      │  │                            │  │
│                      │  │  (inline or side-by-side)  │  │
│                      │  │                            │  │
│                      │  └────────────────────────────┘  │
└──────────────────────┴──────────────────────────────────┘
```

The panel is resizable horizontally. Default split: ~40% graph, ~60% detail. The resizer handle should feel smooth (no jank, no layout jumps).

## Data Sources

### Commit metadata

Already available from the graph data model. No additional git call needed for basic info (SHA, author, date, message, parents, refs).

### Changed files list

```bash
git diff-tree --no-commit-id -r --name-status --find-renames <sha>
```

Output format (tab-delimited):
```
M       src/auth/login.ts
A       src/auth/validate.ts
M       README.md
R085    src/old-name.ts     src/new-name.ts
D       src/deprecated.ts
```

Status codes: `A` (added), `M` (modified), `D` (deleted), `R` (renamed + similarity %), `C` (copied).

For stat numbers (insertions/deletions per file):

```bash
git diff-tree --no-commit-id -r --numstat <sha>
```

Output format (tab-delimited):
```
24      3       src/auth/login.ts
48      0       src/auth/validate.ts
2       1       README.md
```

Both commands should be called together when a commit is selected. Cache results per SHA (commits are immutable).

### File diff

Loaded on demand when the user clicks a file in the changed files list:

```bash
git diff <parent_sha>..<sha> -- <file_path>
```

For merge commits (multiple parents), default to diffing against the first parent:

```bash
git diff <parent1_sha>..<sha> -- <file_path>
```

Optionally allow the user to pick which parent to diff against (dropdown in the header).

For the initial commit (no parent):

```bash
git diff --no-index /dev/null <file_path>
```

Or use `git show <sha>:<file_path>` to get the file content and display it as "all added".

## Commit Detail Header

### Fields

| Field    | Source                | Display                                      |
|----------|-----------------------|----------------------------------------------|
| SHA      | `commit.hash`         | First 7 chars, full SHA on hover/click-to-copy |
| Author   | `commit.author`       | Name + email on hover                        |
| Date     | `commit.timestamp`    | Relative ("2 hours ago") + absolute on hover |
| Message  | `commit.subject`      | Subject line bold, body below (if exists)    |
| Parents  | `commit.parents`      | Short SHAs, clickable (navigate to parent)   |
| Refs     | `commit.refs`         | Branch/tag badges, same style as graph       |

### Actions (buttons in header)

- **Copy SHA** — copy full SHA to clipboard
- **Cherry-pick** — cherry-pick this commit onto current branch (with confirmation)
- **Revert** — `git revert <sha>` (with confirmation)
- **Create branch here** — open input for new branch name, then `git branch <name> <sha>`
- **Create tag here** — open input for tag name, then `git tag <name> <sha>`
- **Reset to here** — dropdown: soft / mixed / hard (with confirmation for hard)

Phase 1 should only include Copy SHA. Other actions come in later phases.

## Changed Files List

### Display

Each file row shows:
- **Status icon**: colored indicator (green for A, yellow for M, red for D, blue for R)
- **File path**: relative to repo root, with directory in muted color and filename in bold
- **Stat**: `+insertions -deletions` in green/red

### Behavior

- Click a file → load its diff in the diff viewer below
- First file is auto-selected when the commit is selected
- Double-click a file → open in system default editor (via Tauri shell)
- Right-click → context menu: "Open file", "Copy path", "Open in editor", "View at this commit"

### Sorting

Default: directories first, then alphabetical. Allow toggle to sort by most changes.

### Filtering

A search/filter input at the top of the file list. Filters by filename as the user types.

## Diff Viewer (Monaco)

### Integration

Monaco is loaded dynamically in `onMount` to avoid SSR issues. The diff viewer uses `monaco.editor.createDiffEditor()`.

```typescript
// Pseudocode for creating the diff editor
const diffEditor = monaco.editor.createDiffEditor(container, {
  readOnly: true,
  renderSideBySide: true,         // toggle-able
  automaticLayout: true,          // resize with container
  minimap: { enabled: false },    // no minimap in diff view
  scrollBeyondLastLine: false,
  folding: true,
  lineNumbers: 'on',
  wordWrap: 'off',
});

diffEditor.setModel({
  original: monaco.editor.createModel(originalContent, language),
  modified: monaco.editor.createModel(modifiedContent, language),
});
```

### Getting file content for diff

To populate the Monaco diff editor, we need the "before" and "after" content:

**Before (original):**
```bash
git show <parent_sha>:<file_path>
```

**After (modified):**
```bash
git show <sha>:<file_path>
```

For added files: original is empty string.
For deleted files: modified is empty string.

### Language detection

Detect language from file extension for syntax highlighting:

```typescript
const languageMap: Record<string, string> = {
  '.ts': 'typescript',
  '.tsx': 'typescript',
  '.js': 'javascript',
  '.jsx': 'javascript',
  '.json': 'json',
  '.css': 'css',
  '.html': 'html',
  '.md': 'markdown',
  '.rs': 'rust',
  '.py': 'python',
  '.cs': 'csharp',
  '.yaml': 'yaml',
  '.yml': 'yaml',
  '.toml': 'toml',
  '.svg': 'xml',
  '.xml': 'xml',
  // extend as needed
};
```

Monaco has built-in support for many languages. Use `monaco.languages.getLanguages()` to check availability.

### Diff Modes

Toggle between two modes via a button in the diff viewer toolbar:

1. **Side-by-side** (default): `renderSideBySide: true`
   - Original on left, modified on right
   - Synchronized scrolling
   - Best for reviewing changes in context

2. **Inline**: `renderSideBySide: false`
   - Single column, deletions and additions interleaved
   - More compact, good for small changes

### Diff Toolbar

A thin toolbar above the Monaco editor:

```
┌─────────────────────────────────────────────────┐
│ src/auth/login.ts          [Inline | Side-by-Side] │
│                            [Wrap] [Copy] [◀ ▶]     │
└─────────────────────────────────────────────────┘
```

- **File path** (left-aligned)
- **Mode toggle**: Inline / Side-by-Side
- **Word wrap toggle**: on/off
- **Copy diff**: copy the raw diff to clipboard
- **Navigation arrows**: jump to previous/next change (`diffEditor.goToDiff()` — actually use `diffEditor.navi.next()` / `diffEditor.navi.previous()`)

### Theming

Monaco must match the app theme. Use a custom Monaco theme that reads from CSS custom properties:

```typescript
monaco.editor.defineTheme('gitshow-dark', {
  base: 'vs-dark',
  inherit: true,
  rules: [],
  colors: {
    'editor.background': '#1a1b2e',        // match app bg
    'diffEditor.insertedTextBackground': '#2ea04333',
    'diffEditor.removedTextBackground': '#f8514933',
  },
});
```

Create both `gitshow-dark` and `gitshow-light` themes. Switch theme when the app theme changes.

### Performance Considerations

- **Lazy load Monaco**: Don't load Monaco until the user first selects a commit. Monaco's bundle is ~2MB — loading it eagerly slows down app startup.
- **Reuse the diff editor instance**: Don't destroy and recreate on every file click. Just update the models.
- **Dispose models**: When switching files, dispose the previous models to avoid memory leaks.
- **Large files**: For files > 10,000 lines, show a warning and offer to load anyway or view raw diff in a plain text fallback.
- **Binary files**: Detect binary files (from git output or file extension) and show "Binary file changed" instead of loading into Monaco.

## Working Directory Changes (Unstaged/Staged)

When no commit is selected OR when the user clicks a "Working Directory" entry at the top of the graph, the detail panel shows the current working directory state instead of a commit.

### Data source

```bash
git status --porcelain=v2 --branch
```

Split into two sections:
- **Staged changes** (index vs HEAD)
- **Unstaged changes** (working tree vs index)

### Diff for working directory

**Unstaged:**
```bash
git diff -- <file_path>
```

**Staged:**
```bash
git diff --cached -- <file_path>
```

### Stage/Unstage actions

- Click the status icon on an unstaged file → `git add -- <path>` (stage it)
- Click the status icon on a staged file → `git restore --staged -- <path>` (unstage it)
- "Stage All" / "Unstage All" buttons at the section headers
- Partial staging (stage individual hunks) is a Phase 4 feature — skip for now

### Commit from the panel

Below the staged files section, show a commit message input:

```
┌──────────────────────────────┐
│ Commit message               │
│ ┌──────────────────────────┐ │
│ │ Subject line              │ │
│ ├──────────────────────────┤ │
│ │ Optional body...          │ │
│ │                           │ │
│ └──────────────────────────┘ │
│          [Commit] [Amend]    │
└──────────────────────────────┘
```

- **Commit**: `git commit -m "<message>"`
- **Amend**: `git commit --amend -m "<message>"` (with confirmation)
- Disable the Commit button when: no staged files, or empty subject line
- After commit: refresh graph, clear the input, show the new commit selected

## Component Structure

```
src/lib/components/detail/
├── DetailPanel.svelte          # Main container (switches between commit view and working dir)
├── CommitHeader.svelte         # Commit metadata display
├── FileList.svelte             # Changed files list with status icons and stats
├── FileListItem.svelte         # Individual file row
├── DiffViewer.svelte           # Monaco diff editor wrapper
├── DiffToolbar.svelte          # Mode toggle, navigation, wrap toggle
├── WorkingDirectory.svelte     # Staged/unstaged file sections
├── CommitInput.svelte          # Commit message textarea + buttons
└── types.ts                    # TypeScript types for this feature
```

## Implementation Order for Claude Code

1. **DetailPanel + CommitHeader**: Create the panel layout with resizable split. Display commit metadata when a commit is selected in the graph. Wire up the selection event from the graph.

2. **FileList**: Parse `git diff-tree` output. Display changed files with status icons and stats. Make files clickable.

3. **DiffViewer**: Integrate Monaco. Load file content with `git show`. Display diff with syntax highlighting. Implement mode toggle (inline/side-by-side).

4. **WorkingDirectory + CommitInput**: Implement `git status` parsing. Show staged/unstaged sections. Add stage/unstage actions. Add commit input with Commit and Amend buttons.

5. **Polish**: File filtering, keyboard navigation between files (↑/↓), auto-select first file, smooth resizing, theme synchronization.

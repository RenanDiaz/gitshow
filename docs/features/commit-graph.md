# Feature: Commit Graph

## Overview

The commit graph is the central UI of GitShow. It displays the repository's commit history as an interactive, visual graph — similar to GitKraken's main view. Users interact with commits, branches, and tags directly on the graph via click, right-click, and drag & drop.

## Data Source

The graph data comes from parsing the output of:

```bash
git log --all --graph --format='%H%x00%P%x00%an%x00%ae%x00%at%x00%D%x00%s' --topo-order -n <limit>
```

Fields (NUL-delimited):
1. `%H` — commit hash (full SHA)
2. `%P` — parent hashes (space-separated)
3. `%an` — author name
4. `%ae` — author email
5. `%at` — author timestamp (unix)
6. `%D` — ref names (branches, tags, HEAD)
7. `%s` — commit subject

Additionally, branch/ref data is loaded from:

```bash
git for-each-ref --format='%(refname)%00%(objectname)%00%(upstream)%00%(upstream:track)' refs/heads refs/remotes refs/tags
```

## Data Model

```typescript
interface Commit {
  hash: string;
  parents: string[];       // Parent commit hashes
  author: string;
  email: string;
  timestamp: number;       // Unix timestamp
  refs: Ref[];             // Branches/tags pointing to this commit
  subject: string;
  // Layout properties (computed)
  column: number;          // X position in graph (0-based lane)
  row: number;             // Y position in graph (sequential)
}

interface Ref {
  name: string;            // e.g., "main", "feature/login"
  type: 'local' | 'remote' | 'tag' | 'head';
  upstream?: string;       // Upstream tracking ref
  trackingStatus?: string; // e.g., "[ahead 2, behind 1]"
}

interface GraphLane {
  color: string;           // HSL color derived from branch name
  commits: string[];       // Ordered commit hashes in this lane
}

interface GraphEdge {
  from: string;            // Parent commit hash
  to: string;              // Child commit hash
  fromColumn: number;
  toColumn: number;
  type: 'straight' | 'merge' | 'fork';
}
```

## Graph Layout Algorithm

The layout algorithm assigns each commit a column (lane) and draws edges between parent-child commits.

### Steps:

1. **Parse commits** into an ordered list (topological order from git log).
2. **Assign lanes**: Walk commits top to bottom. Each branch occupies a lane. When a branch starts (first commit with that ref), assign it the leftmost available lane. When branches merge, free the lane of the merged branch.
3. **Color assignment**: Hash the branch name to generate a consistent HSL color. Remote tracking branches share the color of their local counterpart.
4. **Edge routing**: Draw edges from child to parent. Straight edges stay in-lane. Merge edges curve from the merged lane to the target lane. Fork edges curve from the parent lane to the new lane.
5. **Compact layout**: Minimize total lanes used. Reuse freed lanes. Keep `main`/`master` in lane 0 when possible.

### Performance Considerations:

- For repos with 10,000+ commits, only compute layout for the visible viewport + buffer (e.g., 200 commits above/below).
- Use virtual scrolling — only render SVG nodes for visible commits.
- Cache the full commit list and lane assignments; recompute only on fetch/commit/checkout.

## SVG Rendering

The graph is rendered as SVG inside a scrollable container.

### Elements:

- **Commit nodes**: Circles at `(column * LANE_WIDTH + OFFSET, row * ROW_HEIGHT)`. Radius ~6px.
- **Edges**: SVG `<path>` elements using cubic bezier curves for merge/fork lines, straight lines for same-lane connections.
- **Ref labels**: Positioned next to their commit node. Local branches: rounded rectangle with branch color. Remote branches: similar but with different opacity/border. Tags: diamond or tag-shaped badge. HEAD: special indicator on the current commit.
- **Author avatar**: Gravatar or initials circle to the right of the commit subject.

### Constants (configurable):

```typescript
const LANE_WIDTH = 16;     // Horizontal spacing between lanes
const ROW_HEIGHT = 32;     // Vertical spacing between commits
const NODE_RADIUS = 5;     // Commit circle radius
const GRAPH_PADDING = 20;  // Left padding for the graph
```

## Interactions

### Click
- **Click commit node**: Select commit → show details in right panel (author, date, message, changed files, diff).
- **Click ref label**: Same as clicking its commit.

### Right-click (Context Menu)
- **On commit**: Copy SHA, cherry-pick, reset to here (soft/mixed/hard), create branch here, create tag here.
- **On local branch label**: Checkout, rename, delete, push, set upstream, merge into current, rebase onto current.
- **On remote branch label**: Checkout as local, fetch, delete remote branch.
- **On tag**: Checkout, delete, push.

### Drag & Drop
- **Drag local branch → local branch**: Opens action menu: Merge, Rebase, Interactive Rebase.
- **Drag commit → local branch**: Cherry-pick commit onto branch.
- **Drag visual indicator**: Show a ghost of the dragged element with a line to the drop target.
- **Invalid drop targets**: Grey out or show "not allowed" cursor.

### Keyboard
- `↑` / `↓`: Navigate commits.
- `Enter`: Open selected commit details.
- `Ctrl+F` / `Cmd+F`: Search commits (by message, author, SHA).

## Auto-fetch Integration

- The graph should update when auto-fetch brings new remote commits.
- New remote commits appear in the graph immediately.
- Remote-only commits (not yet merged locally) should be visually distinct (e.g., slightly faded, or shown as dashed lines).

## Loading Strategy

1. Initial load: fetch last 500 commits + all refs.
2. Scroll up (older): load 200 more commits on demand (infinite scroll).
3. After fetch/push/commit: reload refs and recent commits, merge into existing graph.

## Implementation Notes for Claude Code

- Start with a static SVG renderer that takes a list of `Commit` objects and renders the graph.
- Get the layout algorithm working correctly BEFORE adding interactivity.
- Test with sample data of varying complexity: linear history, feature branches, octopus merges.
- The graph component should be self-contained in `src/lib/components/graph/`.
- Key files:
  - `CommitGraph.svelte` — main graph component (SVG container + scroll)
  - `CommitNode.svelte` — individual commit circle + label
  - `GraphEdge.svelte` — SVG path for parent-child connections
  - `RefLabel.svelte` — branch/tag badge
  - `graphLayout.ts` — layout algorithm (pure function, no Svelte dependency)
  - `graphTypes.ts` — TypeScript types for graph data
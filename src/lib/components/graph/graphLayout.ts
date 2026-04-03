import type { Commit, GraphEdge } from './graphTypes';

const BRANCH_COLORS = [
  'hsl(150, 70%, 50%)',  // green (main)
  'hsl(210, 80%, 60%)',  // blue
  'hsl(340, 75%, 55%)',  // pink
  'hsl(45, 85%, 55%)',   // yellow
  'hsl(280, 65%, 60%)',  // purple
  'hsl(15, 80%, 55%)',   // orange
  'hsl(180, 60%, 50%)',  // teal
];

export interface LayoutResult {
  commits: Commit[];
  edges: GraphEdge[];
  laneColors: string[];
  totalLanes: number;
}

/**
 * Lane-based layout: walk commits in topological order (newest first).
 * Each lane slot holds the hash of the commit we expect next in that lane.
 * When a commit arrives, it claims the lane waiting for it (or allocates a new one).
 * Its parents then get placed into lanes: first parent inherits the current lane,
 * additional parents (merge sources) get new lanes.
 */
export function computeLayout(rawCommits: Omit<Commit, 'column' | 'row'>[]): LayoutResult {
  const commits: Commit[] = [];
  const commitIndex = new Map<string, Commit>();

  // Each slot: hash of the next expected commit in that lane, or null if free
  const lanes: (string | null)[] = [];
  const laneColors: string[] = [];

  function findLane(hash: string): number {
    for (let i = 0; i < lanes.length; i++) {
      if (lanes[i] === hash) return i;
    }
    return -1;
  }

  function allocLane(color?: string): number {
    for (let i = 0; i < lanes.length; i++) {
      if (lanes[i] === null) {
        if (color) laneColors[i] = color;
        return i;
      }
    }
    lanes.push(null);
    laneColors.push(color ?? BRANCH_COLORS[lanes.length % BRANCH_COLORS.length]);
    return lanes.length - 1;
  }

  for (let row = 0; row < rawCommits.length; row++) {
    const raw = rawCommits[row];

    // Find which lane is expecting this commit
    let col = findLane(raw.hash);
    if (col === -1) {
      // New branch tip — allocate a lane
      const branchRef = raw.refs.find(r => r.type === 'local' || r.type === 'head');
      const color = branchRef ? branchColorFromName(branchRef.name) : undefined;
      col = allocLane(color);
    }

    // Claim the lane
    lanes[col] = null;

    const commit: Commit = { ...raw, column: col, row };
    commits.push(commit);
    commitIndex.set(commit.hash, commit);

    const parents = raw.parents;
    if (parents.length >= 1) {
      const firstParent = parents[0];
      const existingLane = findLane(firstParent);
      if (existingLane !== -1) {
        // First parent already expected in another lane — leave it there,
        // and free our lane (it converges)
      } else {
        // First parent inherits this commit's lane
        lanes[col] = firstParent;
      }

      // Additional parents (merge sources) get their own lanes
      for (let p = 1; p < parents.length; p++) {
        const parentHash = parents[p];
        if (findLane(parentHash) === -1) {
          const mergeRef = findRefForParent(rawCommits, parentHash);
          const color = mergeRef ? branchColorFromName(mergeRef) : undefined;
          const newLane = allocLane(color);
          lanes[newLane] = parentHash;
        }
      }
    }
  }

  // Build edges
  const edges: GraphEdge[] = [];
  for (const commit of commits) {
    for (const parentHash of commit.parents) {
      const parent = commitIndex.get(parentHash);
      if (!parent) continue;

      const isSameColumn = commit.column === parent.column;
      const isMerge = commit.parents.length > 1 && parentHash !== commit.parents[0];

      edges.push({
        from: parentHash,
        to: commit.hash,
        fromColumn: parent.column,
        toColumn: commit.column,
        fromRow: parent.row,
        toRow: commit.row,
        type: isSameColumn ? 'straight' : isMerge ? 'merge' : 'fork',
        color: laneColors[isMerge ? parent.column : commit.column] ?? BRANCH_COLORS[0],
      });
    }
  }

  const totalLanes = lanes.length || 1;
  return { commits, edges, laneColors, totalLanes };
}

function branchColorFromName(name: string): string {
  if (name === 'main' || name === 'master') return BRANCH_COLORS[0];
  let hash = 0;
  for (let i = 0; i < name.length; i++) {
    hash = ((hash << 5) - hash + name.charCodeAt(i)) | 0;
  }
  // Skip index 0 (reserved for main)
  return BRANCH_COLORS[1 + (Math.abs(hash) % (BRANCH_COLORS.length - 1))];
}

function findRefForParent(
  rawCommits: Omit<Commit, 'column' | 'row'>[],
  parentHash: string
): string | undefined {
  // Walk forward from parentHash to find the branch ref on or near it
  for (const c of rawCommits) {
    if (c.hash === parentHash) {
      const ref = c.refs.find(r => r.type === 'local');
      return ref?.name;
    }
  }
  return undefined;
}

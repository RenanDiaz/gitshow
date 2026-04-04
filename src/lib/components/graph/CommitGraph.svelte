<script lang="ts">
  import type { Commit } from './graphTypes';
  import { LANE_WIDTH, ROW_HEIGHT, GRAPH_PADDING } from './graphTypes';
  import { computeLayout } from './graphLayout';
  import CommitNode from './CommitNode.svelte';
  import GraphEdge from './GraphEdge.svelte';
  import CommitRow from './CommitRow.svelte';

  const WORKING_DIR_HASH = '__working_directory__';
  const SCROLL_THRESHOLD = 200;

  let {
    commits = [],
    oncommitselect,
    hasChanges = false,
    onworkingdirselect,
    onscrollend,
    loading = false,
  }: {
    commits?: Omit<Commit, 'column' | 'row'>[];
    oncommitselect?: (commit: Commit | null) => void;
    hasChanges?: boolean;
    onworkingdirselect?: () => void;
    onscrollend?: () => void;
    loading?: boolean;
  } = $props();

  let selectedHash = $state<string | null>(null);

  let layout = $derived(computeLayout(commits));

  function handleScroll(e: Event) {
    const el = e.target as HTMLElement;
    const distanceFromBottom = el.scrollHeight - el.scrollTop - el.clientHeight;
    if (distanceFromBottom < SCROLL_THRESHOLD) {
      onscrollend?.();
    }
  }

  // Offset all rows by 1 when showing working directory node
  let rowOffset = $derived(hasChanges ? 1 : 0);

  let svgWidth = $derived(
    GRAPH_PADDING + layout.totalLanes * LANE_WIDTH + 700
  );
  let svgHeight = $derived(
    ROW_HEIGHT * (layout.commits.length + 1 + rowOffset) + ROW_HEIGHT
  );

  // Working directory node position: row 0, same column as HEAD commit
  let wdColumn = $derived(layout.commits.length > 0 ? layout.commits[0].column : 0);
  let wdCx = $derived(GRAPH_PADDING + wdColumn * LANE_WIDTH);
  let wdCy = $derived(ROW_HEIGHT);
  let wdColor = $derived(layout.laneColors[wdColumn] ?? '#888');
  let wdTextX = $derived(GRAPH_PADDING + layout.totalLanes * LANE_WIDTH + 12);

  function handleSelect(hash: string) {
    if (hash === WORKING_DIR_HASH) {
      selectedHash = selectedHash === WORKING_DIR_HASH ? null : WORKING_DIR_HASH;
      if (selectedHash === WORKING_DIR_HASH) {
        onworkingdirselect?.();
      } else {
        oncommitselect?.(null);
      }
      return;
    }
    selectedHash = selectedHash === hash ? null : hash;
    const commit = selectedHash
      ? layout.commits.find(c => c.hash === selectedHash) ?? null
      : null;
    oncommitselect?.(commit);
  }
</script>

<div class="graph-container" onscroll={handleScroll}>
  <svg
    width={svgWidth}
    height={svgHeight}
    viewBox="0 0 {svgWidth} {svgHeight}"
    xmlns="http://www.w3.org/2000/svg"
  >
    <!-- Working directory node and edge -->
    {#if hasChanges}
      <!-- Edge from WD to HEAD -->
      {#if layout.commits.length > 0}
        <line
          x1={wdCx}
          y1={wdCy}
          x2={GRAPH_PADDING + layout.commits[0].column * LANE_WIDTH}
          y2={ROW_HEIGHT + (layout.commits[0].row + rowOffset) * ROW_HEIGHT}
          stroke={wdColor}
          stroke-width="2"
          stroke-dasharray="4,3"
          opacity="0.5"
        />
      {/if}

      <!-- Selection highlight -->
      {#if selectedHash === WORKING_DIR_HASH}
        <rect
          x="0"
          y={wdCy - ROW_HEIGHT / 2}
          width="100%"
          height={ROW_HEIGHT}
          fill="rgba(255,255,255,0.05)"
        />
      {/if}

      <!-- WD node (diamond shape) -->
      <g
        class="wd-node"
        class:selected={selectedHash === WORKING_DIR_HASH}
        role="button"
        tabindex="0"
        onclick={() => handleSelect(WORKING_DIR_HASH)}
        onkeydown={(e) => e.key === 'Enter' && handleSelect(WORKING_DIR_HASH)}
      >
        <polygon
          points="{wdCx},{wdCy - 6} {wdCx + 6},{wdCy} {wdCx},{wdCy + 6} {wdCx - 6},{wdCy}"
          fill={selectedHash === WORKING_DIR_HASH ? '#fff' : wdColor}
          stroke={wdColor}
          stroke-width="2"
        />
      </g>

      <!-- WD label -->
      <text
        x={wdTextX}
        y={wdCy}
        class="wd-label"
        dominant-baseline="central"
      >Working Directory</text>
    {/if}

    <!-- Edges (rendered first, behind nodes) -->
    {#each layout.edges as edge (edge.from + edge.to)}
      <GraphEdge edge={{
        ...edge,
        fromRow: edge.fromRow + rowOffset,
        toRow: edge.toRow + rowOffset,
      }} />
    {/each}

    <!-- Commit nodes (graph circles + ref labels) -->
    {#each layout.commits as commit (commit.hash)}
      <CommitNode
        commit={{ ...commit, row: commit.row + rowOffset }}
        color={layout.laneColors[commit.column] ?? '#888'}
        selected={selectedHash === commit.hash}
        onselect={handleSelect}
      />
    {/each}

    <!-- Commit info rows (subject, sha, author, date) -->
    {#each layout.commits as commit (commit.hash)}
      <CommitRow
        commit={{ ...commit, row: commit.row + rowOffset }}
        totalLanes={layout.totalLanes}
        selected={selectedHash === commit.hash}
        nodeColor={layout.laneColors[commit.column] ?? '#888'}
      />
    {/each}
  </svg>
  {#if loading}
    <div class="loading-indicator">Loading more commits...</div>
  {/if}
</div>

<style>
  .graph-container {
    overflow: auto;
    width: 100%;
    height: 100%;
    background: var(--color-bg-primary, #1a1a2e);
  }
  svg {
    display: block;
  }
  .wd-node {
    cursor: pointer;
  }
  .wd-node:hover polygon {
    filter: brightness(1.3);
  }
  .wd-node.selected polygon {
    stroke-width: 3;
  }
  .wd-label {
    font-size: 13px;
    font-weight: 600;
    fill: hsl(45, 85%, 55%);
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
  }
  .loading-indicator {
    text-align: center;
    padding: 12px;
    font-size: 12px;
    color: var(--color-text-secondary, #aaa);
  }
</style>

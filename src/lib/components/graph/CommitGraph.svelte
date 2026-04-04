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

  // SVG width based on actual number of lanes
  let graphSvgWidth = $derived(
    GRAPH_PADDING + layout.totalLanes * LANE_WIDTH + GRAPH_PADDING
  );

  let totalRows = $derived(layout.commits.length + rowOffset);
  let contentHeight = $derived(ROW_HEIGHT * (totalRows + 1));

  // Working directory node position: row 0, same column as HEAD commit
  let wdColumn = $derived(layout.commits.length > 0 ? layout.commits[0].column : 0);
  let wdCx = $derived(GRAPH_PADDING + wdColumn * LANE_WIDTH);
  let wdCy = $derived(ROW_HEIGHT);
  let wdColor = $derived(layout.laneColors[wdColumn] ?? '#888');

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
  <div class="graph-content" style="height: {contentHeight}px">
    <!-- SVG layer: graph lanes only (edges, nodes, branch labels) -->
    <svg
      class="graph-svg"
      width={graphSvgWidth}
      height={contentHeight}
      viewBox="0 0 {graphSvgWidth} {contentHeight}"
      xmlns="http://www.w3.org/2000/svg"
    >
      <!-- Working directory node and edge -->
      {#if hasChanges}
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
      {/if}

      <!-- Edges (rendered first, behind nodes) -->
      {#each layout.edges as edge (edge.from + edge.to)}
        <GraphEdge edge={{
          ...edge,
          fromRow: edge.fromRow + rowOffset,
          toRow: edge.toRow + rowOffset,
        }} />
      {/each}

      <!-- Commit nodes -->
      {#each layout.commits as commit (commit.hash)}
        <CommitNode
          commit={{ ...commit, row: commit.row + rowOffset }}
          color={layout.laneColors[commit.column] ?? '#888'}
          selected={selectedHash === commit.hash}
          onselect={handleSelect}
        />
      {/each}

      <!-- Branch labels on lane introducers -->
      {#each layout.commits as commit (commit.hash + '_label')}
        {#if layout.laneIntroducers.has(commit.hash)}
          {@const info = layout.laneIntroducers.get(commit.hash)!}
          {@const lx = GRAPH_PADDING + info.lane * LANE_WIDTH}
          {@const ly = ROW_HEIGHT + (commit.row + rowOffset) * ROW_HEIGHT - 14}
          <title>{info.name}</title>
          <rect
            x={lx - 2}
            y={ly - 8}
            rx="2"
            ry="2"
            width={Math.min(info.name.length * 5.5 + 6, graphSvgWidth - lx + 2)}
            height="12"
            fill="rgba(0,0,0,0.7)"
            pointer-events="none"
          />
          <text
            x={lx + 1}
            y={ly}
            class="lane-label"
            dominant-baseline="central"
            fill={layout.laneColors[info.lane] ?? '#888'}
            pointer-events="none"
          >{info.name}</text>
        {/if}
      {/each}
    </svg>

    <!-- HTML rows layer -->
    <div class="rows-layer" style="left: {graphSvgWidth}px; width: calc(100% - {graphSvgWidth}px)">
      <!-- Working directory row -->
      {#if hasChanges}
        <div
          class="wd-row"
          class:selected={selectedHash === WORKING_DIR_HASH}
          style="height: {ROW_HEIGHT}px; top: {ROW_HEIGHT - ROW_HEIGHT / 2}px"
          role="button"
          tabindex="0"
          onclick={() => handleSelect(WORKING_DIR_HASH)}
          onkeydown={(e) => e.key === 'Enter' && handleSelect(WORKING_DIR_HASH)}
        >
          <span class="wd-label">Working Directory</span>
        </div>
      {/if}

      <!-- Commit info rows -->
      {#each layout.commits as commit (commit.hash)}
        <div
          class="row-wrapper"
          style="top: {ROW_HEIGHT + (commit.row + rowOffset) * ROW_HEIGHT - ROW_HEIGHT / 2}px; height: {ROW_HEIGHT}px"
        >
          <CommitRow
            commit={{ ...commit, row: commit.row + rowOffset }}
            selected={selectedHash === commit.hash}
            nodeColor={layout.laneColors[commit.column] ?? '#888'}
            onclick={handleSelect}
          />
        </div>
      {/each}
    </div>
  </div>

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

  .graph-content {
    position: relative;
    min-width: 100%;
  }

  .graph-svg {
    position: absolute;
    top: 0;
    left: 0;
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

  .lane-label {
    font-size: 8px;
    font-family: 'SF Mono', 'Fira Code', monospace;
    pointer-events: none;
  }

  .rows-layer {
    position: absolute;
    top: 0;
  }

  .row-wrapper {
    position: absolute;
    left: 0;
    right: 0;
  }

  .wd-row {
    position: absolute;
    left: 0;
    right: 0;
    display: flex;
    align-items: center;
    padding-left: 12px;
    cursor: pointer;
  }

  .wd-row:hover {
    background: rgba(255, 255, 255, 0.03);
  }

  .wd-row.selected {
    background: rgba(255, 255, 255, 0.05);
  }

  .wd-label {
    font-size: 13px;
    font-weight: 600;
    color: hsl(45, 85%, 55%);
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
  }

  .loading-indicator {
    text-align: center;
    padding: 12px;
    font-size: 12px;
    color: var(--color-text-secondary, #aaa);
  }
</style>

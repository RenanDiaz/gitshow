<script lang="ts">
  import type { Commit } from './graphTypes';
  import { LANE_WIDTH, ROW_HEIGHT, GRAPH_PADDING } from './graphTypes';
  import { computeLayout } from './graphLayout';
  import CommitNode from './CommitNode.svelte';
  import GraphEdge from './GraphEdge.svelte';
  import CommitRow from './CommitRow.svelte';
  import { SAMPLE_COMMITS } from './sampleData';

  let { oncommitselect }: {
    oncommitselect?: (commit: Commit | null) => void;
  } = $props();

  let selectedHash = $state<string | null>(null);

  let layout = $derived(computeLayout(SAMPLE_COMMITS));

  let svgWidth = $derived(
    GRAPH_PADDING + layout.totalLanes * LANE_WIDTH + 700
  );
  let svgHeight = $derived(
    ROW_HEIGHT * (layout.commits.length + 1) + ROW_HEIGHT
  );

  function handleSelect(hash: string) {
    selectedHash = selectedHash === hash ? null : hash;
    const commit = selectedHash
      ? layout.commits.find(c => c.hash === selectedHash) ?? null
      : null;
    oncommitselect?.(commit);
  }
</script>

<div class="graph-container">
  <svg
    width={svgWidth}
    height={svgHeight}
    viewBox="0 0 {svgWidth} {svgHeight}"
    xmlns="http://www.w3.org/2000/svg"
  >
    <!-- Edges (rendered first, behind nodes) -->
    {#each layout.edges as edge (edge.from + edge.to)}
      <GraphEdge {edge} />
    {/each}

    <!-- Commit nodes (graph circles + ref labels) -->
    {#each layout.commits as commit (commit.hash)}
      <CommitNode
        {commit}
        color={layout.laneColors[commit.column] ?? '#888'}
        selected={selectedHash === commit.hash}
        onselect={handleSelect}
      />
    {/each}

    <!-- Commit info rows (subject, sha, author, date) -->
    {#each layout.commits as commit (commit.hash)}
      <CommitRow
        {commit}
        totalLanes={layout.totalLanes}
        selected={selectedHash === commit.hash}
        nodeColor={layout.laneColors[commit.column] ?? '#888'}
      />
    {/each}
  </svg>
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
</style>

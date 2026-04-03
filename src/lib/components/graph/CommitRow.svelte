<script lang="ts">
  import type { Commit } from './graphTypes';
  import { ROW_HEIGHT, LANE_WIDTH, GRAPH_PADDING } from './graphTypes';

  let { commit, totalLanes, selected = false, nodeColor }: {
    commit: Commit;
    totalLanes: number;
    selected?: boolean;
    nodeColor: string;
  } = $props();

  let y = $derived(ROW_HEIGHT + commit.row * ROW_HEIGHT);
  let textStartX = $derived(GRAPH_PADDING + totalLanes * LANE_WIDTH + 12);

  function shortHash(hash: string): string {
    return hash.slice(0, 7);
  }

  function formatDate(timestamp: number): string {
    const d = new Date(timestamp * 1000);
    return d.toLocaleDateString('en-US', { month: 'short', day: 'numeric', year: 'numeric' });
  }

  let refLabels = $derived(commit.refs);
  let refTotalWidth = $derived(
    refLabels.reduce((sum, ref) => sum + ref.name.length * 7 + 16 + 4, 0)
  );
  let fixedX = $derived(textStartX + 420);
</script>

<g class="commit-row" class:selected>
  {#if selected}
    <rect
      x="0"
      y={y - ROW_HEIGHT / 2}
      width="100%"
      height={ROW_HEIGHT}
      fill="rgba(255,255,255,0.05)"
    />
  {/if}

  <!-- Ref labels (inline before subject) -->
  {#each refLabels as ref, i}
    {@const labelOffset = refLabels.slice(0, i).reduce((sum, r) => sum + r.name.length * 7 + 16 + 4, 0)}
    {@const labelX = textStartX + labelOffset}
    {@const refColor = ref.type === 'head' ? '#e8a33d' : ref.type === 'remote' ? 'rgba(255,255,255,0.15)' : nodeColor}
    <rect
      x={labelX}
      y={y - 9}
      rx="3"
      ry="3"
      width={ref.name.length * 7 + 16}
      height="18"
      fill={refColor}
      opacity={ref.type === 'remote' ? 0.8 : 1}
    />
    <text
      x={labelX + 8}
      y={y + 1}
      class="ref-text"
      dominant-baseline="central"
    >
      {ref.name}
    </text>
  {/each}

  <!-- Subject (shifts with refs) -->
  <text x={textStartX + refTotalWidth} y={y} class="subject" dominant-baseline="central">
    {commit.subject}
  </text>

  <!-- Fixed-position columns: SHA, Author, Date -->
  <text x={fixedX} y={y} class="sha" dominant-baseline="central">
    {shortHash(commit.hash)}
  </text>

  <text x={fixedX + 80} y={y} class="author" dominant-baseline="central">
    {commit.author}
  </text>

  <text x={fixedX + 200} y={y} class="date" dominant-baseline="central">
    {formatDate(commit.timestamp)}
  </text>
</g>

<style>
  .ref-text {
    font-size: 10px;
    font-family: 'SF Mono', 'Fira Code', monospace;
    fill: #fff;
    pointer-events: none;
  }
  .subject {
    font-size: 13px;
    fill: var(--color-text-primary, #e0e0e0);
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
  }
  .sha {
    font-size: 12px;
    fill: var(--color-text-secondary, #888);
    font-family: 'SF Mono', 'Fira Code', monospace;
  }
  .author {
    font-size: 12px;
    fill: var(--color-text-secondary, #888);
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
  }
  .date {
    font-size: 12px;
    fill: var(--color-text-tertiary, #666);
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
  }
  .selected .subject {
    fill: #fff;
  }
</style>

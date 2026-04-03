<script lang="ts">
  import type { Commit } from './graphTypes';
  import { LANE_WIDTH, ROW_HEIGHT, NODE_RADIUS, GRAPH_PADDING } from './graphTypes';

  let { commit, color, selected = false, onselect }: {
    commit: Commit;
    color: string;
    selected?: boolean;
    onselect?: (hash: string) => void;
  } = $props();

  let cx = $derived(GRAPH_PADDING + commit.column * LANE_WIDTH);
  let cy = $derived(ROW_HEIGHT + commit.row * ROW_HEIGHT);
</script>

<g
  class="commit-node"
  class:selected
  role="button"
  tabindex="0"
  onclick={() => onselect?.(commit.hash)}
  onkeydown={(e) => e.key === 'Enter' && onselect?.(commit.hash)}
>
  <circle
    {cx}
    {cy}
    r={NODE_RADIUS}
    fill={selected ? '#fff' : color}
    stroke={color}
    stroke-width="2"
  />
</g>

<style>
  .commit-node {
    cursor: pointer;
  }
  .commit-node:hover circle {
    filter: brightness(1.3);
  }
  .commit-node.selected circle {
    stroke-width: 3;
  }
</style>

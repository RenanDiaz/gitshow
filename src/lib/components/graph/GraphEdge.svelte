<script lang="ts">
  import type { GraphEdge } from './graphTypes';
  import { LANE_WIDTH, ROW_HEIGHT, GRAPH_PADDING } from './graphTypes';

  let { edge }: { edge: GraphEdge } = $props();

  let x1 = $derived(GRAPH_PADDING + edge.toColumn * LANE_WIDTH);
  let y1 = $derived(ROW_HEIGHT + edge.toRow * ROW_HEIGHT);
  let x2 = $derived(GRAPH_PADDING + edge.fromColumn * LANE_WIDTH);
  let y2 = $derived(ROW_HEIGHT + edge.fromRow * ROW_HEIGHT);

  let path = $derived.by(() => {
    if (edge.type === 'straight') {
      return `M ${x1} ${y1} L ${x2} ${y2}`;
    }
    // Curved path for merge/fork edges
    const midY = y1 + (y2 - y1) * 0.4;
    return `M ${x1} ${y1} C ${x1} ${midY}, ${x2} ${midY}, ${x2} ${y2}`;
  });
</script>

<path
  d={path}
  stroke={edge.color}
  stroke-width="2"
  fill="none"
  stroke-linecap="round"
/>

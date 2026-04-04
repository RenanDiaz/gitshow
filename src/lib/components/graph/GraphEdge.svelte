<script lang="ts">
  import type { GraphEdge } from './graphTypes';
  import { LANE_WIDTH, ROW_HEIGHT, GRAPH_PADDING } from './graphTypes';

  let { edge }: { edge: GraphEdge } = $props();

  let x1 = $derived(GRAPH_PADDING + edge.toColumn * LANE_WIDTH);
  let y1 = $derived(ROW_HEIGHT + edge.toRow * ROW_HEIGHT);
  let x2 = $derived(GRAPH_PADDING + edge.fromColumn * LANE_WIDTH);
  let y2 = $derived(ROW_HEIGHT + edge.fromRow * ROW_HEIGHT);

  let isCurved = $derived(edge.type === 'merge' || edge.type === 'fork');

  let path = $derived.by(() => {
    if (!isCurved) {
      return `M ${x1} ${y1} L ${x2} ${y2}`;
    }
    // More pronounced bezier curves for merge/fork edges
    const dy = y2 - y1;
    const midY1 = y1 + dy * 0.25;
    const midY2 = y1 + dy * 0.75;
    return `M ${x1} ${y1} C ${x1} ${midY1}, ${x2} ${midY2}, ${x2} ${y2}`;
  });

  let strokeWidth = $derived(isCurved ? 2 : 1.5);
</script>

<path
  d={path}
  stroke={edge.color}
  stroke-width={strokeWidth}
  fill="none"
  stroke-linecap="round"
/>

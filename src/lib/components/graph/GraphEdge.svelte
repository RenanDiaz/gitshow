<script lang="ts">
  import type { GraphEdge } from './graphTypes';
  import { LANE_WIDTH, ROW_HEIGHT, GRAPH_PADDING } from './graphTypes';

  let { edge }: { edge: GraphEdge } = $props();

  // x1,y1 = child commit (top), x2,y2 = parent commit (bottom)
  let x1 = $derived(GRAPH_PADDING + edge.toColumn * LANE_WIDTH);
  let y1 = $derived(ROW_HEIGHT + edge.toRow * ROW_HEIGHT);
  let x2 = $derived(GRAPH_PADDING + edge.fromColumn * LANE_WIDTH);
  let y2 = $derived(ROW_HEIGHT + edge.fromRow * ROW_HEIGHT);

  let rowSpan = $derived(Math.abs(edge.fromRow - edge.toRow));
  let sameLane = $derived(edge.toColumn === edge.fromColumn);

  let path = $derived.by(() => {
    // Same lane: straight vertical line
    if (sameLane) {
      return `M ${x1} ${y1} L ${x2} ${y2}`;
    }

    // Different lanes, short distance (1-2 rows): simple bezier curve
    if (rowSpan <= 2) {
      const dy = y2 - y1;
      const midY1 = y1 + dy * 0.25;
      const midY2 = y1 + dy * 0.75;
      return `M ${x1} ${y1} C ${x1} ${midY1}, ${x2} ${midY2}, ${x2} ${y2}`;
    }

    // Different lanes, long distance (3+ rows):
    // Vertical drop in the source lane, then curve into the target lane
    // Pattern: go straight down to one row above destination, then S-curve into it
    const curveStartY = y2 - ROW_HEIGHT;
    const cpY1 = curveStartY + ROW_HEIGHT * 0.3;
    const cpY2 = curveStartY + ROW_HEIGHT * 0.7;
    return `M ${x1} ${y1} L ${x1} ${curveStartY} C ${x1} ${cpY1}, ${x2} ${cpY2}, ${x2} ${y2}`;
  });

  let strokeWidth = $derived(sameLane ? 1.5 : 2);
</script>

<path
  d={path}
  stroke={edge.color}
  stroke-width={strokeWidth}
  fill="none"
  stroke-linecap="round"
/>

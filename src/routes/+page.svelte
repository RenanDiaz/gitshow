<script lang="ts">
  import type { Commit } from '$lib/components/graph/graphTypes';
  import CommitGraph from '$lib/components/graph/CommitGraph.svelte';
  import DetailPanel from '$lib/components/detail/DetailPanel.svelte';

  let selectedCommit = $state<Commit | null>(null);
  let splitPercent = $state(50);
  let dragging = $state(false);
  let containerEl: HTMLElement | undefined = $state();

  function handleCommitSelect(commit: Commit | null) {
    selectedCommit = commit;
  }

  function startResize(e: MouseEvent) {
    e.preventDefault();
    dragging = true;
  }

  function onMouseMove(e: MouseEvent) {
    if (!dragging || !containerEl) return;
    const rect = containerEl.getBoundingClientRect();
    const x = e.clientX - rect.left;
    const pct = (x / rect.width) * 100;
    splitPercent = Math.min(80, Math.max(20, pct));
  }

  function onMouseUp() {
    dragging = false;
  }
</script>

<svelte:window onmousemove={onMouseMove} onmouseup={onMouseUp} />

<div class="app-layout">
  <header class="titlebar">
    <span class="app-name">GitShow</span>
  </header>
  <main
    class="content"
    class:dragging
    bind:this={containerEl}
  >
    <div class="graph-pane" style="width: {selectedCommit ? splitPercent + '%' : '100%'}">
      <CommitGraph oncommitselect={handleCommitSelect} />
    </div>

    {#if selectedCommit}
      <div
        class="resize-handle"
        role="separator"
        aria-orientation="vertical"
        onmousedown={startResize}
      ></div>
      <div class="detail-pane" style="width: {100 - splitPercent}%">
        <DetailPanel commit={selectedCommit} />
      </div>
    {/if}
  </main>
</div>

<style>
  :global(*) {
    margin: 0;
    padding: 0;
    box-sizing: border-box;
  }

  :global(html, body) {
    height: 100%;
    overflow: hidden;
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
    background: var(--color-bg-primary, #1a1a2e);
    color: var(--color-text-primary, #e0e0e0);
  }

  .app-layout {
    display: flex;
    flex-direction: column;
    height: 100vh;
  }

  .titlebar {
    display: flex;
    align-items: center;
    height: 38px;
    padding: 0 16px;
    background: var(--color-bg-secondary, #16213e);
    border-bottom: 1px solid rgba(255, 255, 255, 0.08);
    -webkit-app-region: drag;
    user-select: none;
  }

  .app-name {
    font-size: 13px;
    font-weight: 600;
    color: var(--color-text-secondary, #aaa);
    letter-spacing: 0.5px;
  }

  .content {
    flex: 1;
    display: flex;
    overflow: hidden;
  }

  .content.dragging {
    cursor: col-resize;
    user-select: none;
  }

  .graph-pane {
    overflow: hidden;
    transition: width 0.15s ease;
  }

  .content.dragging .graph-pane {
    transition: none;
  }

  .detail-pane {
    overflow: hidden;
    border-left: 1px solid rgba(255, 255, 255, 0.08);
    transition: width 0.15s ease;
  }

  .content.dragging .detail-pane {
    transition: none;
  }

  .resize-handle {
    width: 4px;
    cursor: col-resize;
    background: transparent;
    flex-shrink: 0;
    position: relative;
    z-index: 10;
  }

  .resize-handle:hover,
  .content.dragging .resize-handle {
    background: hsl(210, 80%, 55%);
  }
</style>

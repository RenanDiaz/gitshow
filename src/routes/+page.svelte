<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import type { Commit } from '$lib/components/graph/graphTypes';
  import type { WorkingDirectoryStatus } from '$lib/components/detail/types';
  import CommitGraph from '$lib/components/graph/CommitGraph.svelte';
  import DetailPanel from '$lib/components/detail/DetailPanel.svelte';

  let selectedCommit = $state<Commit | null>(null);
  let showWorkingDirectory = $state(false);
  let hasChanges = $state(false);
  let splitPercent = $state(50);
  let dragging = $state(false);
  let containerEl: HTMLElement | undefined = $state();

  let showDetailPanel = $derived(selectedCommit !== null || showWorkingDirectory);

  // Check for working directory changes on load
  async function checkForChanges() {
    try {
      const status = await invoke<WorkingDirectoryStatus>('get_working_directory_status');
      hasChanges = status.staged.length > 0 || status.unstaged.length > 0;
    } catch {
      hasChanges = false;
    }
  }

  $effect(() => {
    checkForChanges();
  });

  function handleCommitSelect(commit: Commit | null) {
    showWorkingDirectory = false;
    selectedCommit = commit;
  }

  function handleWorkingDirSelect() {
    selectedCommit = null;
    showWorkingDirectory = true;
  }

  function handleRefresh() {
    checkForChanges();
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
    <div class="graph-pane" style="width: {showDetailPanel ? splitPercent + '%' : '100%'}">
      <CommitGraph
        oncommitselect={handleCommitSelect}
        onworkingdirselect={handleWorkingDirSelect}
        {hasChanges}
      />
    </div>

    {#if showDetailPanel}
      <div
        class="resize-handle"
        role="separator"
        aria-orientation="vertical"
        onmousedown={startResize}
      ></div>
      <div class="detail-pane" style="width: {100 - splitPercent}%">
        <DetailPanel
          commit={selectedCommit}
          {showWorkingDirectory}
          onrefresh={handleRefresh}
        />
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

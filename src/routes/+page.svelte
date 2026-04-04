<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { open } from '@tauri-apps/plugin-dialog';
  import type { Commit, Ref } from '$lib/components/graph/graphTypes';
  import type { WorkingDirectoryStatus } from '$lib/components/detail/types';
  import CommitGraph from '$lib/components/graph/CommitGraph.svelte';
  import DetailPanel from '$lib/components/detail/DetailPanel.svelte';

  interface CommitLogEntry {
    hash: string;
    parents: string[];
    author: string;
    email: string;
    timestamp: number;
    refs: Ref[];
    subject: string;
  }

  const COMMITS_PER_PAGE = 500;

  let selectedCommit = $state<Commit | null>(null);
  let showWorkingDirectory = $state(false);
  let hasChanges = $state(false);
  let splitPercent = $state(50);
  let dragging = $state(false);
  let containerEl: HTMLElement | undefined = $state();
  let repoPath = $state<string | null>(null);
  let commits = $state<CommitLogEntry[]>([]);
  let loadingCommits = $state(false);
  let allCommitsLoaded = $state(false);

  let showDetailPanel = $derived(selectedCommit !== null || showWorkingDirectory);
  let repoName = $derived(repoPath ? repoPath.split('/').pop() ?? repoPath : null);

  async function loadCommits(skip: number) {
    if (loadingCommits) return;
    loadingCommits = true;
    try {
      const entries = await invoke<CommitLogEntry[]>('get_commit_log', {
        skip,
        limit: COMMITS_PER_PAGE,
      });
      if (skip === 0) {
        commits = entries;
      } else {
        commits = [...commits, ...entries];
      }
      allCommitsLoaded = entries.length < COMMITS_PER_PAGE;
    } catch (err) {
      console.error('Failed to load commits:', err);
    } finally {
      loadingCommits = false;
    }
  }

  function handleScrollEnd() {
    if (!loadingCommits && !allCommitsLoaded) {
      loadCommits(commits.length);
    }
  }

  async function checkForChanges() {
    if (!repoPath) return;
    try {
      const status = await invoke<WorkingDirectoryStatus>('get_working_directory_status');
      hasChanges = status.staged.length > 0 || status.unstaged.length > 0;
    } catch {
      hasChanges = false;
    }
  }

  $effect(() => {
    if (repoPath) {
      checkForChanges();
      loadCommits(0);
    }
  });

  async function openRepository() {
    const selected = await open({ directory: true, multiple: false, title: 'Open Git Repository' });
    if (!selected) return;

    try {
      await invoke<string>('open_repository', { path: selected });
      repoPath = selected;
      selectedCommit = null;
      showWorkingDirectory = false;
    } catch (err) {
      // TODO: show proper error toast
      console.error('Failed to open repository:', err);
      alert(err);
    }
  }

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
    loadCommits(0);
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
    {#if repoName}
      <span class="repo-name">{repoName}</span>
    {/if}
    <button class="open-repo-btn" onclick={openRepository}>Open Repository</button>
  </header>

  {#if repoPath}
    <main
      class="content"
      class:dragging
      bind:this={containerEl}
    >
      <div class="graph-pane" style="width: {showDetailPanel ? splitPercent + '%' : '100%'}">
        <CommitGraph
          {commits}
          oncommitselect={handleCommitSelect}
          onworkingdirselect={handleWorkingDirSelect}
          onscrollend={handleScrollEnd}
          loading={loadingCommits}
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
  {:else}
    <main class="empty-state">
      <div class="empty-content">
        <svg class="empty-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
          <path d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z" />
        </svg>
        <h2>No repository open</h2>
        <p>Open a local Git repository to get started.</p>
        <button class="open-repo-btn primary" onclick={openRepository}>Open Repository</button>
      </div>
    </main>
  {/if}
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
    gap: 10px;
  }

  .app-name {
    font-size: 13px;
    font-weight: 600;
    color: var(--color-text-secondary, #aaa);
    letter-spacing: 0.5px;
  }

  .repo-name {
    font-size: 13px;
    font-weight: 500;
    color: var(--color-text-primary, #e0e0e0);
  }

  .open-repo-btn {
    -webkit-app-region: no-drag;
    margin-left: auto;
    padding: 4px 12px;
    font-size: 12px;
    font-weight: 500;
    border: 1px solid rgba(255, 255, 255, 0.15);
    border-radius: 4px;
    background: transparent;
    color: var(--color-text-secondary, #aaa);
    cursor: pointer;
    transition: background 0.15s, color 0.15s;
  }

  .open-repo-btn:hover {
    background: rgba(255, 255, 255, 0.08);
    color: var(--color-text-primary, #e0e0e0);
  }

  .open-repo-btn.primary {
    margin-left: 0;
    padding: 10px 28px;
    font-size: 14px;
    background: hsl(210, 80%, 50%);
    border-color: hsl(210, 80%, 50%);
    color: #fff;
  }

  .open-repo-btn.primary:hover {
    background: hsl(210, 80%, 58%);
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

  .empty-state {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .empty-content {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 12px;
    color: var(--color-text-secondary, #aaa);
  }

  .empty-icon {
    width: 64px;
    height: 64px;
    opacity: 0.4;
  }

  .empty-content h2 {
    font-size: 18px;
    font-weight: 600;
    color: var(--color-text-primary, #e0e0e0);
  }

  .empty-content p {
    font-size: 14px;
    margin-bottom: 8px;
  }
</style>

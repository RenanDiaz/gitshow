<script lang="ts">
  import type { Commit } from '../graph/graphTypes';
  import type { WorkingDirectoryStatus } from './types';
  import CommitHeader from './CommitHeader.svelte';
  import FileList from './FileList.svelte';
  import DiffViewer from './DiffViewer.svelte';
  import WorkingDirectory from './WorkingDirectory.svelte';
  import CommitInput from './CommitInput.svelte';

  let { commit, showWorkingDirectory = false, onrefresh }: {
    commit: Commit | null;
    showWorkingDirectory?: boolean;
    onrefresh?: () => void;
  } = $props();

  let selectedFilePath = $state<string | null>(null);
  let selectedFileStaged = $state(false);
  let stagedCount = $state(0);
  let wdRefreshTrigger = $state(0);

  // Reset selected file when commit changes
  let prevHash = '';
  let prevShowWd = false;
  $effect(() => {
    const isWd = showWorkingDirectory;
    if (commit && commit.hash !== prevHash) {
      prevHash = commit.hash;
      selectedFilePath = null;
    }
    if (isWd !== prevShowWd) {
      prevShowWd = isWd;
      selectedFilePath = null;
    }
  });

  function handleFileSelect(path: string) {
    selectedFilePath = path;
    selectedFileStaged = false;
  }

  function handleWdFileSelect(path: string, staged: boolean) {
    selectedFilePath = path;
    selectedFileStaged = staged;
  }

  function handleStatusChange(status: WorkingDirectoryStatus) {
    stagedCount = status.staged.length;
  }

  function handleCommit() {
    wdRefreshTrigger++;
    onrefresh?.();
  }
</script>

<div class="detail-panel">
  {#if showWorkingDirectory}
    <div class="wd-header">
      <h3 class="wd-title">Working Directory</h3>
    </div>
    <WorkingDirectory
      refreshTrigger={wdRefreshTrigger}
      onfileselect={handleWdFileSelect}
      onstatuschange={handleStatusChange}
    />
    <CommitInput {stagedCount} oncommit={handleCommit} />
    {#if selectedFilePath}
      <DiffViewer filePath={selectedFilePath} staged={selectedFileStaged} isWorkingDirectory={true} />
    {/if}
  {:else if commit}
    <CommitHeader {commit} />
    <FileList sha={commit.hash} onfileselect={handleFileSelect} />
    {#if selectedFilePath}
      <DiffViewer sha={commit.hash} filePath={selectedFilePath} />
    {/if}
  {:else}
    <div class="empty-state">
      <span class="empty-icon">&#xf126;</span>
      <p class="empty-text">Select a commit to view details</p>
    </div>
  {/if}
</div>

<style>
  .detail-panel {
    height: 100%;
    display: flex;
    flex-direction: column;
    background: var(--color-bg-primary, #1a1a2e);
  }

  .wd-header {
    padding: 12px 16px 8px;
    border-bottom: 1px solid rgba(255, 255, 255, 0.06);
  }

  .wd-title {
    font-size: 14px;
    font-weight: 600;
    color: var(--color-text-primary, #e0e0e0);
    margin: 0;
  }

  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    gap: 12px;
    opacity: 0.5;
  }

  .empty-icon {
    font-size: 32px;
    color: var(--color-text-tertiary, #666);
  }

  .empty-text {
    font-size: 14px;
    color: var(--color-text-tertiary, #666);
  }
</style>

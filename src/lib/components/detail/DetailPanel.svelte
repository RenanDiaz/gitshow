<script lang="ts">
  import type { Commit } from '../graph/graphTypes';
  import CommitHeader from './CommitHeader.svelte';
  import FileList from './FileList.svelte';
  import DiffViewer from './DiffViewer.svelte';

  let { commit }: { commit: Commit | null } = $props();

  let selectedFilePath = $state<string | null>(null);

  // Reset selected file when commit changes
  let prevHash = '';
  $effect(() => {
    if (commit && commit.hash !== prevHash) {
      prevHash = commit.hash;
      selectedFilePath = null;
    }
  });

  function handleFileSelect(path: string) {
    selectedFilePath = path;
  }
</script>

<div class="detail-panel">
  {#if commit}
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

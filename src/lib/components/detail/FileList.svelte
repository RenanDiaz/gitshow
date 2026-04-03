<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import type { ChangedFile } from './types';
  import FileListItem from './FileListItem.svelte';

  let { sha }: { sha: string } = $props();

  let files = $state<ChangedFile[]>([]);
  let loading = $state(false);
  let error = $state<string | null>(null);
  let filterText = $state('');
  let selectedPath = $state<string | null>(null);

  let filteredFiles = $derived(
    filterText
      ? files.filter((f) => f.path.toLowerCase().includes(filterText.toLowerCase()))
      : files
  );

  let fileCount = $derived(
    filterText
      ? `${filteredFiles.length} / ${files.length} files`
      : `${files.length} files`
  );

  async function loadFiles(commitSha: string) {
    loading = true;
    error = null;
    selectedPath = null;
    filterText = '';

    try {
      const result = await invoke<
        { status: string; path: string; old_path: string | null; insertions: number; deletions: number }[]
      >('get_commit_files', { sha: commitSha });

      files = result.map((f) => ({
        status: f.status as ChangedFile['status'],
        path: f.path,
        oldPath: f.old_path,
        insertions: f.insertions,
        deletions: f.deletions,
      }));
    } catch (e) {
      error = String(e);
      files = [];
    } finally {
      loading = false;
    }
  }

  $effect(() => {
    if (sha) {
      loadFiles(sha);
    }
  });
</script>

<div class="file-list">
  <div class="file-list-header">
    <span class="file-count">{fileCount}</span>
    <input
      class="filter-input"
      type="text"
      placeholder="Filter files..."
      bind:value={filterText}
    />
  </div>

  <div class="file-list-body">
    {#if loading}
      <div class="status-msg">Loading files...</div>
    {:else if error}
      <div class="status-msg error">{error}</div>
    {:else if filteredFiles.length === 0}
      <div class="status-msg">
        {filterText ? 'No files match filter' : 'No changed files'}
      </div>
    {:else}
      {#each filteredFiles as file (file.path)}
        <FileListItem
          {file}
          selected={selectedPath === file.path}
          onclick={() => { selectedPath = file.path; }}
        />
      {/each}
    {/if}
  </div>
</div>

<style>
  .file-list {
    display: flex;
    flex-direction: column;
    border-top: 1px solid rgba(255, 255, 255, 0.06);
  }

  .file-list-header {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 12px;
    border-bottom: 1px solid rgba(255, 255, 255, 0.06);
  }

  .file-count {
    font-size: 11px;
    color: var(--color-text-tertiary, #666);
    white-space: nowrap;
    flex-shrink: 0;
  }

  .filter-input {
    flex: 1;
    background: rgba(255, 255, 255, 0.06);
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 4px;
    padding: 3px 8px;
    font-size: 12px;
    color: var(--color-text-primary, #e0e0e0);
    font-family: inherit;
    outline: none;
  }

  .filter-input::placeholder {
    color: var(--color-text-tertiary, #666);
  }

  .filter-input:focus {
    border-color: rgba(88, 166, 255, 0.5);
  }

  .file-list-body {
    max-height: 300px;
    overflow-y: auto;
  }

  .status-msg {
    padding: 16px 12px;
    font-size: 12px;
    color: var(--color-text-tertiary, #666);
    text-align: center;
  }

  .status-msg.error {
    color: #f85149;
  }
</style>

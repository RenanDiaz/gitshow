<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import type { ChangedFile, WorkingDirectoryStatus } from './types';
  import FileListItem from './FileListItem.svelte';

  let {
    onfileselect,
    onstatuschange,
    refreshTrigger = 0,
  }: {
    onfileselect?: (path: string, staged: boolean) => void;
    onstatuschange?: (status: WorkingDirectoryStatus) => void;
    refreshTrigger?: number;
  } = $props();

  let status = $state<WorkingDirectoryStatus>({ staged: [], unstaged: [] });
  let loading = $state(false);
  let error = $state<string | null>(null);
  let selectedPath = $state<string | null>(null);
  let selectedStaged = $state(false);
  let stagedCollapsed = $state(false);
  let unstagedCollapsed = $state(false);

  async function refresh() {
    loading = true;
    error = null;
    try {
      const result = await invoke<WorkingDirectoryStatus>('get_working_directory_status');
      status = result;
      onstatuschange?.(result);
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  }

  // Load on mount and when refreshTrigger changes
  let prevTrigger = -1;
  $effect(() => {
    const t = refreshTrigger;
    if (t !== prevTrigger) {
      prevTrigger = t;
      refresh();
    }
  });

  function selectFile(path: string, staged: boolean) {
    selectedPath = path;
    selectedStaged = staged;
    onfileselect?.(path, staged);
  }

  async function stageFile(path: string) {
    try {
      await invoke('stage_files', { paths: [path] });
      await refresh();
    } catch (e) {
      error = String(e);
    }
  }

  async function unstageFile(path: string) {
    try {
      await invoke('unstage_files', { paths: [path] });
      await refresh();
    } catch (e) {
      error = String(e);
    }
  }

  async function handleStageAll() {
    try {
      await invoke('stage_all');
      await refresh();
    } catch (e) {
      error = String(e);
    }
  }

  async function handleUnstageAll() {
    try {
      await invoke('unstage_all');
      await refresh();
    } catch (e) {
      error = String(e);
    }
  }

  let totalChanges = $derived(status.staged.length + status.unstaged.length);
</script>

<div class="working-directory">
  {#if loading && totalChanges === 0}
    <div class="loading">Loading status...</div>
  {:else if error}
    <div class="error">{error}</div>
  {:else if totalChanges === 0}
    <div class="empty">No changes in working directory</div>
  {:else}
    <!-- Staged section -->
    <div class="section">
      <div
        class="section-header"
        role="button"
        tabindex="0"
        onclick={() => (stagedCollapsed = !stagedCollapsed)}
        onkeydown={(e) => e.key === 'Enter' && (stagedCollapsed = !stagedCollapsed)}
      >
        <span class="collapse-icon">{stagedCollapsed ? '\u25B6' : '\u25BC'}</span>
        <span class="section-title">Staged</span>
        <span class="section-count">{status.staged.length}</span>
        {#if status.staged.length > 0}
          <button
            class="section-action"
            type="button"
            onclick={(e) => { e.stopPropagation(); handleUnstageAll(); }}
            title="Unstage all"
          >Unstage All</button>
        {/if}
      </div>
      {#if !stagedCollapsed}
        <div class="section-files">
          {#each status.staged as file (file.path)}
            <div class="file-row">
              <button
                class="stage-action unstage"
                type="button"
                onclick={() => unstageFile(file.path)}
                title="Unstage file"
              >&minus;</button>
              <FileListItem
                {file}
                selected={selectedPath === file.path && selectedStaged}
                onclick={() => selectFile(file.path, true)}
              />
            </div>
          {/each}
          {#if status.staged.length === 0}
            <div class="section-empty">No staged changes</div>
          {/if}
        </div>
      {/if}
    </div>

    <!-- Unstaged section -->
    <div class="section">
      <div
        class="section-header"
        role="button"
        tabindex="0"
        onclick={() => (unstagedCollapsed = !unstagedCollapsed)}
        onkeydown={(e) => e.key === 'Enter' && (unstagedCollapsed = !unstagedCollapsed)}
      >
        <span class="collapse-icon">{unstagedCollapsed ? '\u25B6' : '\u25BC'}</span>
        <span class="section-title">Unstaged</span>
        <span class="section-count">{status.unstaged.length}</span>
        {#if status.unstaged.length > 0}
          <button
            class="section-action"
            type="button"
            onclick={(e) => { e.stopPropagation(); handleStageAll(); }}
            title="Stage all"
          >Stage All</button>
        {/if}
      </div>
      {#if !unstagedCollapsed}
        <div class="section-files">
          {#each status.unstaged as file (file.path)}
            <div class="file-row">
              <button
                class="stage-action stage"
                type="button"
                onclick={() => stageFile(file.path)}
                title="Stage file"
              >+</button>
              <FileListItem
                {file}
                selected={selectedPath === file.path && !selectedStaged}
                onclick={() => selectFile(file.path, false)}
              />
            </div>
          {/each}
          {#if status.unstaged.length === 0}
            <div class="section-empty">No unstaged changes</div>
          {/if}
        </div>
      {/if}
    </div>
  {/if}
</div>

<style>
  .working-directory {
    overflow-y: auto;
    flex-shrink: 0;
    max-height: 50%;
  }

  .loading,
  .error,
  .empty {
    padding: 16px;
    font-size: 12px;
    color: var(--color-text-tertiary, #666);
    text-align: center;
  }

  .error {
    color: #f85149;
  }

  .section {
    border-bottom: 1px solid rgba(255, 255, 255, 0.06);
  }

  .section-header {
    display: flex;
    align-items: center;
    gap: 6px;
    width: 100%;
    padding: 6px 12px;
    border: none;
    background: rgba(255, 255, 255, 0.03);
    color: var(--color-text-secondary, #aaa);
    font-family: inherit;
    font-size: 11px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    cursor: pointer;
    text-align: left;
  }

  .section-header:hover {
    background: rgba(255, 255, 255, 0.06);
  }

  .collapse-icon {
    font-size: 8px;
    width: 10px;
    text-align: center;
  }

  .section-title {
    flex: 1;
  }

  .section-count {
    font-weight: 400;
    color: var(--color-text-tertiary, #666);
    font-size: 10px;
  }

  .section-action {
    padding: 1px 6px;
    border: 1px solid rgba(255, 255, 255, 0.15);
    border-radius: 3px;
    background: transparent;
    color: var(--color-text-secondary, #aaa);
    font-family: inherit;
    font-size: 10px;
    cursor: pointer;
  }

  .section-action:hover {
    background: rgba(255, 255, 255, 0.08);
    color: var(--color-text-primary, #e0e0e0);
  }

  .section-files {
    padding: 2px 0;
  }

  .section-empty {
    padding: 8px 12px;
    font-size: 11px;
    color: var(--color-text-tertiary, #666);
    font-style: italic;
  }

  .file-row {
    display: flex;
    align-items: center;
  }

  .stage-action {
    width: 20px;
    height: 20px;
    margin-left: 4px;
    border: none;
    border-radius: 3px;
    background: transparent;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
    font-size: 14px;
    font-weight: 700;
    line-height: 1;
  }

  .stage-action.stage {
    color: #2ea043;
  }

  .stage-action.unstage {
    color: #f85149;
  }

  .stage-action:hover {
    background: rgba(255, 255, 255, 0.1);
  }
</style>

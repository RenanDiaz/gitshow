<script lang="ts">
  import type { ChangedFile, FileStatus } from './types';

  let { file, selected = false, onclick }: {
    file: ChangedFile;
    selected?: boolean;
    onclick?: () => void;
  } = $props();

  const statusConfig: Record<FileStatus, { label: string; color: string }> = {
    A: { label: 'A', color: '#2ea043' },
    M: { label: 'M', color: '#d29922' },
    D: { label: 'D', color: '#f85149' },
    R: { label: 'R', color: '#58a6ff' },
    C: { label: 'C', color: '#58a6ff' },
  };

  let config = $derived(statusConfig[file.status] ?? statusConfig.M);

  let dirName = $derived(() => {
    const lastSlash = file.path.lastIndexOf('/');
    return lastSlash >= 0 ? file.path.substring(0, lastSlash + 1) : '';
  });

  let fileName = $derived(() => {
    const lastSlash = file.path.lastIndexOf('/');
    return lastSlash >= 0 ? file.path.substring(lastSlash + 1) : file.path;
  });
</script>

<button
  class="file-item"
  class:selected
  {onclick}
  type="button"
>
  <span class="status-badge" style="color: {config.color}">
    {config.label}
  </span>

  <span class="file-path" title={file.oldPath ? `${file.oldPath} → ${file.path}` : file.path}>
    {#if file.oldPath}
      <span class="dir">{dirName()}</span><span class="name">{fileName()}</span>
      <span class="rename-arrow">←</span>
      <span class="old-path">{file.oldPath}</span>
    {:else}
      <span class="dir">{dirName()}</span><span class="name">{fileName()}</span>
    {/if}
  </span>

  <span class="stats">
    {#if file.insertions > 0}
      <span class="insertions">+{file.insertions}</span>
    {/if}
    {#if file.deletions > 0}
      <span class="deletions">-{file.deletions}</span>
    {/if}
  </span>
</button>

<style>
  .file-item {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 4px 12px;
    width: 100%;
    border: none;
    background: transparent;
    color: var(--color-text-primary, #e0e0e0);
    font-family: inherit;
    font-size: 12px;
    text-align: left;
    cursor: pointer;
    line-height: 1.6;
  }

  .file-item:hover {
    background: rgba(255, 255, 255, 0.04);
  }

  .file-item.selected {
    background: rgba(255, 255, 255, 0.08);
  }

  .status-badge {
    font-weight: 700;
    font-size: 11px;
    width: 14px;
    text-align: center;
    flex-shrink: 0;
    font-family: 'SF Mono', 'Cascadia Code', 'Fira Code', monospace;
  }

  .file-path {
    flex: 1;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .dir {
    color: var(--color-text-tertiary, #666);
  }

  .name {
    font-weight: 600;
    color: var(--color-text-primary, #e0e0e0);
  }

  .rename-arrow {
    color: var(--color-text-tertiary, #666);
    margin: 0 4px;
    font-size: 11px;
  }

  .old-path {
    color: var(--color-text-tertiary, #666);
    font-size: 11px;
  }

  .stats {
    flex-shrink: 0;
    display: flex;
    gap: 6px;
    font-family: 'SF Mono', 'Cascadia Code', 'Fira Code', monospace;
    font-size: 11px;
  }

  .insertions {
    color: #2ea043;
  }

  .deletions {
    color: #f85149;
  }
</style>

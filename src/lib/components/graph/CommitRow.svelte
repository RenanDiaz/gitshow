<script lang="ts">
  import type { Commit, Ref } from './graphTypes';
  import { ROW_HEIGHT } from './graphTypes';

  let { commit, selected = false, nodeColor, onclick }: {
    commit: Commit;
    selected?: boolean;
    nodeColor: string;
    onclick?: (hash: string) => void;
  } = $props();

  function shortHash(hash: string): string {
    return hash.slice(0, 7);
  }

  function formatDate(timestamp: number): string {
    const d = new Date(timestamp * 1000);
    return d.toLocaleDateString('en-US', { month: 'short', day: 'numeric', year: 'numeric' });
  }

  function refColor(ref: Ref): string {
    if (ref.type === 'head') return '#e8a33d';
    if (ref.type === 'remote') return 'rgba(255,255,255,0.15)';
    return nodeColor;
  }
</script>

<div
  class="commit-row"
  class:selected
  style="height: {ROW_HEIGHT}px"
  role="button"
  tabindex="0"
  onclick={() => onclick?.(commit.hash)}
  onkeydown={(e) => e.key === 'Enter' && onclick?.(commit.hash)}
>
  <div class="col-message">
    {#each commit.refs as ref}
      <span class="ref-badge" style="background: {refColor(ref)}">
        {ref.name}
      </span>
    {/each}
    <span class="subject">{commit.subject}</span>
  </div>
  <div class="col-sha">{shortHash(commit.hash)}</div>
  <div class="col-author">{commit.author}</div>
  <div class="col-date">{formatDate(commit.timestamp)}</div>
</div>

<style>
  .commit-row {
    display: flex;
    align-items: center;
    gap: 14px;
    padding-right: 12px;
    cursor: pointer;
  }

  .commit-row:hover {
    background: rgba(255, 255, 255, 0.03);
  }

  .commit-row.selected {
    background: rgba(255, 255, 255, 0.05);
  }

  .col-message {
    flex: 1;
    min-width: 0;
    display: flex;
    align-items: center;
    gap: 6px;
    overflow: hidden;
  }

  .ref-badge {
    flex-shrink: 0;
    display: inline-block;
    padding: 1px 6px;
    border-radius: 3px;
    font-size: 10px;
    font-family: 'SF Mono', 'Fira Code', monospace;
    color: #fff;
    white-space: nowrap;
    line-height: 16px;
  }

  .subject {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-size: 13px;
    color: var(--color-text-primary, #e0e0e0);
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
  }

  .selected .subject {
    color: #fff;
  }

  .col-sha {
    flex-shrink: 0;
    width: 80px;
    font-size: 12px;
    font-family: 'SF Mono', 'Fira Code', monospace;
    color: var(--color-text-secondary, #888);
    text-align: right;
  }

  .col-author {
    flex-shrink: 0;
    width: 120px;
    font-size: 12px;
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
    color: var(--color-text-secondary, #888);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .col-date {
    flex-shrink: 0;
    width: 100px;
    font-size: 12px;
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
    color: var(--color-text-tertiary, #666);
    text-align: right;
  }
</style>

<script lang="ts">
  import type { Commit } from '../graph/graphTypes';
  import CommitHeader from './CommitHeader.svelte';

  let { commit }: { commit: Commit | null } = $props();
</script>

<div class="detail-panel">
  {#if commit}
    <CommitHeader {commit} />
    <div class="placeholder">
      <span class="placeholder-text">File list and diff viewer coming soon</span>
    </div>
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
    overflow-y: auto;
    background: var(--color-bg-primary, #1a1a2e);
  }

  .placeholder {
    padding: 24px 16px;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .placeholder-text {
    font-size: 13px;
    color: var(--color-text-tertiary, #666);
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

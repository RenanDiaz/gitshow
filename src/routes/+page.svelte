<script lang="ts">
  import CommitGraph from '$lib/components/graph/CommitGraph.svelte';
</script>

<div class="app-layout">
  <header class="titlebar">
    <span class="app-name">GitShow</span>
  </header>
  <main class="content">
    <CommitGraph />
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
    overflow: hidden;
  }
</style>

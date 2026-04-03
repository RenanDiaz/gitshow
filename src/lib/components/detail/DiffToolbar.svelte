<script lang="ts">
  let {
    filePath,
    sideBySide = true,
    wordWrap = false,
    onsidebysidechange,
    onwordwrapchange,
  }: {
    filePath: string;
    sideBySide?: boolean;
    wordWrap?: boolean;
    onsidebysidechange: (value: boolean) => void;
    onwordwrapchange: (value: boolean) => void;
  } = $props();
</script>

<div class="diff-toolbar">
  <span class="file-path" title={filePath}>{filePath}</span>

  <div class="toolbar-actions">
    <div class="mode-toggle">
      <button
        class="toggle-btn"
        class:active={!sideBySide}
        onclick={() => onsidebysidechange(false)}
      >Inline</button>
      <button
        class="toggle-btn"
        class:active={sideBySide}
        onclick={() => onsidebysidechange(true)}
      >Side-by-Side</button>
    </div>

    <button
      class="toolbar-btn"
      class:active={wordWrap}
      title="Toggle word wrap"
      onclick={() => onwordwrapchange(!wordWrap)}
    >Wrap</button>
  </div>
</div>

<style>
  .diff-toolbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 4px 8px;
    background: rgba(255, 255, 255, 0.03);
    border-top: 1px solid rgba(255, 255, 255, 0.06);
    border-bottom: 1px solid rgba(255, 255, 255, 0.06);
    min-height: 32px;
    gap: 8px;
  }

  .file-path {
    font-size: 12px;
    color: var(--color-text-secondary, #aaa);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    flex-shrink: 1;
    min-width: 0;
  }

  .toolbar-actions {
    display: flex;
    align-items: center;
    gap: 6px;
    flex-shrink: 0;
  }

  .mode-toggle {
    display: flex;
    border: 1px solid rgba(255, 255, 255, 0.12);
    border-radius: 4px;
    overflow: hidden;
  }

  .toggle-btn {
    background: transparent;
    border: none;
    color: var(--color-text-tertiary, #666);
    font-size: 11px;
    padding: 2px 8px;
    cursor: pointer;
    font-family: inherit;
  }

  .toggle-btn:hover {
    color: var(--color-text-primary, #e0e0e0);
  }

  .toggle-btn.active {
    background: rgba(255, 255, 255, 0.1);
    color: var(--color-text-primary, #e0e0e0);
  }

  .toggle-btn + .toggle-btn {
    border-left: 1px solid rgba(255, 255, 255, 0.12);
  }

  .toolbar-btn {
    background: transparent;
    border: 1px solid rgba(255, 255, 255, 0.12);
    border-radius: 4px;
    color: var(--color-text-tertiary, #666);
    font-size: 11px;
    padding: 2px 8px;
    cursor: pointer;
    font-family: inherit;
  }

  .toolbar-btn:hover {
    color: var(--color-text-primary, #e0e0e0);
  }

  .toolbar-btn.active {
    background: rgba(255, 255, 255, 0.1);
    color: var(--color-text-primary, #e0e0e0);
  }
</style>

<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';

  let {
    stagedCount = 0,
    oncommit,
  }: {
    stagedCount?: number;
    oncommit?: () => void;
  } = $props();

  let subject = $state('');
  let body = $state('');
  let committing = $state(false);
  let error = $state<string | null>(null);
  let showAmendConfirm = $state(false);

  let canCommit = $derived(stagedCount > 0 && subject.trim().length > 0);

  function buildMessage(): string {
    const trimmedSubject = subject.trim();
    const trimmedBody = body.trim();
    if (trimmedBody) {
      return `${trimmedSubject}\n\n${trimmedBody}`;
    }
    return trimmedSubject;
  }

  async function handleCommit() {
    if (!canCommit || committing) return;
    committing = true;
    error = null;
    try {
      await invoke('create_commit', { message: buildMessage(), amend: false });
      subject = '';
      body = '';
      oncommit?.();
    } catch (e) {
      error = String(e);
    } finally {
      committing = false;
    }
  }

  async function handleAmend() {
    if (committing) return;
    if (!showAmendConfirm) {
      showAmendConfirm = true;
      return;
    }
    committing = true;
    error = null;
    showAmendConfirm = false;
    try {
      await invoke('create_commit', { message: buildMessage(), amend: true });
      subject = '';
      body = '';
      oncommit?.();
    } catch (e) {
      error = String(e);
    } finally {
      committing = false;
    }
  }

  function handleSubjectKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter' && !e.shiftKey && canCommit) {
      e.preventDefault();
      handleCommit();
    }
  }
</script>

<div class="commit-input">
  <input
    class="subject-input"
    type="text"
    placeholder="Commit message"
    bind:value={subject}
    onkeydown={handleSubjectKeydown}
    disabled={committing}
  />
  <textarea
    class="body-input"
    placeholder="Extended description (optional)"
    bind:value={body}
    rows="3"
    disabled={committing}
  ></textarea>

  {#if error}
    <div class="commit-error">{error}</div>
  {/if}

  <div class="commit-actions">
    <button
      class="btn btn-commit"
      type="button"
      disabled={!canCommit || committing}
      onclick={handleCommit}
    >
      {committing ? 'Committing...' : `Commit (${stagedCount})`}
    </button>
    <button
      class="btn btn-amend"
      class:confirming={showAmendConfirm}
      type="button"
      disabled={committing || subject.trim().length === 0}
      onclick={handleAmend}
      onblur={() => (showAmendConfirm = false)}
    >
      {showAmendConfirm ? 'Confirm Amend' : 'Amend'}
    </button>
  </div>
</div>

<style>
  .commit-input {
    padding: 8px 12px;
    border-top: 1px solid rgba(255, 255, 255, 0.06);
    display: flex;
    flex-direction: column;
    gap: 6px;
    flex-shrink: 0;
  }

  .subject-input {
    width: 100%;
    padding: 6px 8px;
    border: 1px solid rgba(255, 255, 255, 0.12);
    border-radius: 4px;
    background: rgba(255, 255, 255, 0.04);
    color: var(--color-text-primary, #e0e0e0);
    font-family: inherit;
    font-size: 12px;
    outline: none;
  }

  .subject-input:focus {
    border-color: hsl(210, 80%, 55%);
  }

  .body-input {
    width: 100%;
    padding: 6px 8px;
    border: 1px solid rgba(255, 255, 255, 0.12);
    border-radius: 4px;
    background: rgba(255, 255, 255, 0.04);
    color: var(--color-text-primary, #e0e0e0);
    font-family: inherit;
    font-size: 12px;
    outline: none;
    resize: vertical;
    min-height: 40px;
  }

  .body-input:focus {
    border-color: hsl(210, 80%, 55%);
  }

  .commit-error {
    font-size: 11px;
    color: #f85149;
    padding: 2px 0;
  }

  .commit-actions {
    display: flex;
    gap: 6px;
  }

  .btn {
    padding: 5px 12px;
    border: none;
    border-radius: 4px;
    font-family: inherit;
    font-size: 12px;
    font-weight: 600;
    cursor: pointer;
  }

  .btn:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  .btn-commit {
    flex: 1;
    background: hsl(210, 80%, 45%);
    color: #fff;
  }

  .btn-commit:not(:disabled):hover {
    background: hsl(210, 80%, 50%);
  }

  .btn-amend {
    background: rgba(255, 255, 255, 0.08);
    color: var(--color-text-secondary, #aaa);
  }

  .btn-amend:not(:disabled):hover {
    background: rgba(255, 255, 255, 0.12);
  }

  .btn-amend.confirming {
    background: hsl(30, 80%, 45%);
    color: #fff;
  }
</style>

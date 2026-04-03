<script lang="ts">
  import type { Commit } from '../graph/graphTypes';

  let { commit }: { commit: Commit } = $props();

  let shortHash = $derived(commit.hash.slice(0, 7));

  let relativeDate = $derived.by(() => {
    const now = Date.now();
    const commitTime = commit.timestamp * 1000;
    const diff = now - commitTime;
    const minutes = Math.floor(diff / 60000);
    const hours = Math.floor(diff / 3600000);
    const days = Math.floor(diff / 86400000);
    const months = Math.floor(diff / 2592000000);
    const years = Math.floor(diff / 31536000000);

    if (minutes < 1) return 'just now';
    if (minutes < 60) return `${minutes} minute${minutes > 1 ? 's' : ''} ago`;
    if (hours < 24) return `${hours} hour${hours > 1 ? 's' : ''} ago`;
    if (days < 30) return `${days} day${days > 1 ? 's' : ''} ago`;
    if (months < 12) return `${months} month${months > 1 ? 's' : ''} ago`;
    return `${years} year${years > 1 ? 's' : ''} ago`;
  });

  let absoluteDate = $derived(
    new Date(commit.timestamp * 1000).toLocaleString('en-US', {
      weekday: 'short',
      year: 'numeric',
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
    })
  );

  let parentShortHashes = $derived(
    commit.parents.map(p => p.slice(0, 7))
  );

  let copied = $state(false);

  async function copySha() {
    await navigator.clipboard.writeText(commit.hash);
    copied = true;
    setTimeout(() => (copied = false), 2000);
  }
</script>

<div class="commit-header">
  <div class="subject-line">
    {#if commit.refs.length > 0}
      <span class="refs">
        {#each commit.refs as ref}
          <span
            class="ref-badge"
            class:head={ref.type === 'head'}
            class:local={ref.type === 'local'}
            class:remote={ref.type === 'remote'}
            class:tag={ref.type === 'tag'}
          >
            {ref.name}
          </span>
        {/each}
      </span>
    {/if}
    <span class="subject">{commit.subject}</span>
  </div>

  <div class="meta-grid">
    <span class="label">SHA</span>
    <span class="value sha-row">
      <code class="sha" title={commit.hash}>{shortHash}</code>
      <button class="copy-btn" onclick={copySha} title="Copy full SHA">
        {copied ? 'Copied!' : 'Copy'}
      </button>
    </span>

    <span class="label">Author</span>
    <span class="value" title={commit.email}>{commit.author}</span>

    <span class="label">Date</span>
    <span class="value" title={absoluteDate}>{relativeDate}</span>

    {#if commit.parents.length > 0}
      <span class="label">Parent{commit.parents.length > 1 ? 's' : ''}</span>
      <span class="value parents">
        {#each parentShortHashes as pHash, i}
          <code class="parent-sha">{pHash}</code>{#if i < parentShortHashes.length - 1}<span class="separator">,</span>{/if}
        {/each}
      </span>
    {/if}
  </div>
</div>

<style>
  .commit-header {
    padding: 16px;
    border-bottom: 1px solid rgba(255, 255, 255, 0.08);
  }

  .subject-line {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-bottom: 12px;
    flex-wrap: wrap;
  }

  .subject {
    font-size: 15px;
    font-weight: 600;
    color: var(--color-text-primary, #e0e0e0);
  }

  .refs {
    display: flex;
    gap: 4px;
    flex-shrink: 0;
  }

  .ref-badge {
    display: inline-block;
    padding: 2px 8px;
    border-radius: 3px;
    font-size: 11px;
    font-family: 'SF Mono', 'Fira Code', monospace;
    font-weight: 500;
    color: #fff;
    background: rgba(255, 255, 255, 0.12);
  }

  .ref-badge.head {
    background: #e8a33d;
  }

  .ref-badge.local {
    background: hsl(150, 70%, 35%);
  }

  .ref-badge.remote {
    background: rgba(255, 255, 255, 0.15);
  }

  .ref-badge.tag {
    background: hsl(210, 80%, 45%);
  }

  .meta-grid {
    display: grid;
    grid-template-columns: auto 1fr;
    gap: 6px 12px;
    align-items: center;
  }

  .label {
    font-size: 12px;
    color: var(--color-text-tertiary, #666);
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .value {
    font-size: 13px;
    color: var(--color-text-secondary, #aaa);
  }

  .sha-row {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .sha {
    font-family: 'SF Mono', 'Fira Code', monospace;
    color: var(--color-text-primary, #e0e0e0);
    cursor: default;
  }

  .copy-btn {
    padding: 2px 8px;
    border: 1px solid rgba(255, 255, 255, 0.15);
    border-radius: 3px;
    background: transparent;
    color: var(--color-text-secondary, #aaa);
    font-size: 11px;
    cursor: pointer;
    transition: background 0.15s, color 0.15s;
  }

  .copy-btn:hover {
    background: rgba(255, 255, 255, 0.08);
    color: var(--color-text-primary, #e0e0e0);
  }

  .parents {
    display: flex;
    gap: 4px;
    align-items: center;
  }

  .parent-sha {
    font-family: 'SF Mono', 'Fira Code', monospace;
    color: hsl(210, 80%, 65%);
    cursor: pointer;
  }

  .parent-sha:hover {
    text-decoration: underline;
  }

  .separator {
    color: var(--color-text-tertiary, #666);
  }
</style>

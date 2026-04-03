<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { onMount, onDestroy } from 'svelte';
  import type { FileDiff } from './types';
  import DiffToolbar from './DiffToolbar.svelte';

  let { sha, filePath }: { sha: string; filePath: string } = $props();

  let containerEl: HTMLDivElement | undefined = $state();
  let loading = $state(false);
  let error = $state<string | null>(null);
  let sideBySide = $state(true);
  let wordWrap = $state(false);

  let monaco: typeof import('monaco-editor') | null = null;
  let diffEditor: import('monaco-editor').editor.IStandaloneDiffEditor | null = null;
  let originalModel: import('monaco-editor').editor.ITextModel | null = null;
  let modifiedModel: import('monaco-editor').editor.ITextModel | null = null;

  const languageMap: Record<string, string> = {
    '.ts': 'typescript',
    '.tsx': 'typescript',
    '.js': 'javascript',
    '.jsx': 'javascript',
    '.json': 'json',
    '.css': 'css',
    '.scss': 'scss',
    '.less': 'less',
    '.html': 'html',
    '.md': 'markdown',
    '.rs': 'rust',
    '.py': 'python',
    '.cs': 'csharp',
    '.go': 'go',
    '.java': 'java',
    '.rb': 'ruby',
    '.php': 'php',
    '.sh': 'shell',
    '.bash': 'shell',
    '.zsh': 'shell',
    '.yaml': 'yaml',
    '.yml': 'yaml',
    '.toml': 'ini',
    '.svg': 'xml',
    '.xml': 'xml',
    '.sql': 'sql',
    '.svelte': 'html',
  };

  function getLanguage(path: string): string {
    const dotIndex = path.lastIndexOf('.');
    if (dotIndex === -1) return 'plaintext';
    const ext = path.substring(dotIndex).toLowerCase();
    return languageMap[ext] || 'plaintext';
  }

  function disposeModels() {
    if (originalModel) {
      originalModel.dispose();
      originalModel = null;
    }
    if (modifiedModel) {
      modifiedModel.dispose();
      modifiedModel = null;
    }
  }

  async function initMonaco() {
    if (monaco) return;

    const mod = await import('monaco-editor');
    monaco = mod;

    // Set up workers
    self.MonacoEnvironment = {
      getWorker(_workerId: string, _label: string) {
        return new Worker(
          new URL('monaco-editor/esm/vs/editor/editor.worker.js', import.meta.url),
          { type: 'module' }
        );
      },
    };

    monaco.editor.defineTheme('gitshow-dark', {
      base: 'vs-dark',
      inherit: true,
      rules: [],
      colors: {
        'editor.background': '#1a1a2e',
        'diffEditor.insertedTextBackground': '#2ea04333',
        'diffEditor.removedTextBackground': '#f8514933',
        'diffEditor.insertedLineBackground': '#2ea04322',
        'diffEditor.removedLineBackground': '#f8514922',
      },
    });
  }

  async function createEditor() {
    if (!monaco || !containerEl) return;

    diffEditor = monaco.editor.createDiffEditor(containerEl, {
      readOnly: true,
      renderSideBySide: sideBySide,
      automaticLayout: true,
      minimap: { enabled: false },
      scrollBeyondLastLine: false,
      folding: true,
      lineNumbers: 'on',
      wordWrap: wordWrap ? 'on' : 'off',
      theme: 'gitshow-dark',
      fontSize: 13,
      renderOverviewRuler: false,
    });
  }

  async function loadDiff(commitSha: string, path: string) {
    if (!monaco) return;

    loading = true;
    error = null;

    try {
      const result = await invoke<{ original: string; modified: string }>('get_file_diff', {
        sha: commitSha,
        filePath: path,
      });

      const language = getLanguage(path);

      disposeModels();

      originalModel = monaco.editor.createModel(result.original, language);
      modifiedModel = monaco.editor.createModel(result.modified, language);

      if (diffEditor) {
        diffEditor.setModel({
          original: originalModel,
          modified: modifiedModel,
        });
      }
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  }

  onMount(async () => {
    await initMonaco();
    await createEditor();
    await loadDiff(sha, filePath);
  });

  onDestroy(() => {
    disposeModels();
    if (diffEditor) {
      diffEditor.dispose();
      diffEditor = null;
    }
  });

  // React to file/sha changes — reuse editor, update models
  let prevSha = '';
  let prevFilePath = '';
  $effect(() => {
    const currentSha = sha;
    const currentPath = filePath;
    if ((currentSha !== prevSha || currentPath !== prevFilePath) && monaco && diffEditor) {
      prevSha = currentSha;
      prevFilePath = currentPath;
      loadDiff(currentSha, currentPath);
    }
  });

  function handleSideBySideChange(value: boolean) {
    sideBySide = value;
    if (diffEditor) {
      diffEditor.updateOptions({ renderSideBySide: value });
    }
  }

  function handleWordWrapChange(value: boolean) {
    wordWrap = value;
    if (diffEditor) {
      diffEditor.updateOptions({ wordWrap: value ? 'on' : 'off' });
    }
  }
</script>

<div class="diff-viewer">
  <DiffToolbar
    {filePath}
    {sideBySide}
    {wordWrap}
    onsidebysidechange={handleSideBySideChange}
    onwordwrapchange={handleWordWrapChange}
  />

  <div class="editor-container">
    {#if loading}
      <div class="status-overlay">Loading diff...</div>
    {/if}
    {#if error}
      <div class="status-overlay error">{error}</div>
    {/if}
    <div class="editor-mount" bind:this={containerEl}></div>
  </div>
</div>

<style>
  .diff-viewer {
    display: flex;
    flex-direction: column;
    flex: 1;
    min-height: 0;
  }

  .editor-container {
    position: relative;
    flex: 1;
    min-height: 0;
  }

  .editor-mount {
    width: 100%;
    height: 100%;
  }

  .status-overlay {
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    padding: 16px;
    font-size: 12px;
    color: var(--color-text-tertiary, #666);
    text-align: center;
    z-index: 1;
    background: var(--color-bg-primary, #1a1a2e);
  }

  .status-overlay.error {
    color: #f85149;
  }
</style>

<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/tauri';
  import { listen } from '@tauri-apps/api/event';
  
  import MarkdownView from '$lib/MarkdownView.svelte';
  import Sidebar from '$lib/Sidebar.svelte';
  import Header from '$lib/Header.svelte';
  import { currentFile, parsedDocument, theme, sidebarOpen } from '$lib/stores';
  import type { ParsedDocument } from '$lib/types';

  let isLoading = false;
  let error: string | null = null;

  onMount(async () => {
    // Listen for file changes
    await listen('file-changed', async (event: any) => {
      console.log('File changed:', event.payload);
      if ($currentFile) {
        await loadFile($currentFile);
      }
    });

    // Initialize theme
    const savedTheme = localStorage.getItem('theme');
    if (savedTheme) {
      theme.set(savedTheme as 'light' | 'dark' | 'auto');
    }

    // Apply theme to document
    theme.subscribe((value) => {
      if (value === 'auto') {
        document.documentElement.removeAttribute('data-theme');
      } else {
        document.documentElement.setAttribute('data-theme', value);
      }
      localStorage.setItem('theme', value);
    });
  });

  async function loadFile(path: string) {
    try {
      isLoading = true;
      error = null;
      
      // Read file content
      const contentResult = await invoke('read_markdown_file', { path });
      if (!contentResult.success) {
        throw new Error(contentResult.error || 'Failed to read file');
      }
      
      // Parse markdown
      const parseResult = await invoke('parse_markdown', { 
        content: contentResult.data 
      });
      if (!parseResult.success) {
        throw new Error(parseResult.error || 'Failed to parse markdown');
      }
      
      currentFile.set(path);
      parsedDocument.set(parseResult.data as ParsedDocument);
      
      // Start watching the file
      await invoke('watch_file', { path });
      
    } catch (err) {
      console.error('Error loading file:', err);
      error = err instanceof Error ? err.message : 'Unknown error';
    } finally {
      isLoading = false;
    }
  }

  async function openFile() {
    try {
      const result = await invoke('open_file_dialog');
      if (result.success && result.data) {
        await loadFile(result.data);
      }
    } catch (err) {
      console.error('Error opening file:', err);
      error = err instanceof Error ? err.message : 'Failed to open file';
    }
  }

  function handleKeydown(event: KeyboardEvent) {
    // Keyboard shortcuts
    if ((event.metaKey || event.ctrlKey) && event.key === 'o') {
      event.preventDefault();
      openFile();
    }
    
    if ((event.metaKey || event.ctrlKey) && event.key === '\\') {
      event.preventDefault();
      sidebarOpen.update(open => !open);
    }
  }
</script>

<svelte:window on:keydown={handleKeydown} />

<div class="app" class:sidebar-open={$sidebarOpen}>
  <Header bind:isLoading {error} on:openFile={openFile} />
  
  {#if $sidebarOpen}
    <Sidebar on:openFile={(e) => loadFile(e.detail)} />
  {/if}
  
  <main class="main-content">
    {#if isLoading}
      <div class="loading-state">
        <div class="spinner"></div>
        <p>Loading markdown file...</p>
      </div>
    {:else if error}
      <div class="error-state">
        <div class="error-icon">⚠️</div>
        <h2>Error Loading File</h2>
        <p>{error}</p>
        <button class="retry-button" on:click={openFile}>
          Try Opening Another File
        </button>
      </div>
    {:else if $parsedDocument}
      <MarkdownView document={$parsedDocument} />
    {:else}
      <div class="welcome-state">
        <div class="welcome-icon">📝</div>
        <h1>Welcome to Typora-Lite</h1>
        <p>A fast, beautiful Markdown viewer</p>
        <button class="open-button" on:click={openFile}>
          Open Markdown File
        </button>
        <p class="hint">
          Or press <kbd>Cmd+O</kbd> (macOS) / <kbd>Ctrl+O</kbd> (Windows/Linux)
        </p>
      </div>
    {/if}
  </main>
</div>

<style>
  .app {
    display: flex;
    flex-direction: column;
    height: 100vh;
    width: 100vw;
    overflow: hidden;
  }

  .main-content {
    flex: 1;
    display: flex;
    min-height: 0;
    transition: margin-left 0.3s ease;
  }

  .sidebar-open .main-content {
    margin-left: var(--size-sidebar);
  }

  /* Loading state */
  .loading-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    flex: 1;
    gap: 1rem;
    color: var(--color-text-secondary);
  }

  .spinner {
    width: 32px;
    height: 32px;
    border: 3px solid var(--color-border);
    border-top: 3px solid var(--color-accent);
    border-radius: 50%;
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    0% { transform: rotate(0deg); }
    100% { transform: rotate(360deg); }
  }

  /* Error state */
  .error-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    flex: 1;
    gap: 1rem;
    padding: 2rem;
    text-align: center;
  }

  .error-icon {
    font-size: 3rem;
  }

  .error-state h2 {
    color: var(--color-error);
    margin: 0;
  }

  .error-state p {
    color: var(--color-text-secondary);
    max-width: 400px;
  }

  .retry-button {
    background-color: var(--color-accent);
    color: white;
    border: none;
    border-radius: var(--radius);
    padding: 0.75rem 1.5rem;
    font-weight: 500;
    cursor: pointer;
    transition: background-color 0.2s ease;
  }

  .retry-button:hover {
    background-color: var(--color-accent-hover);
  }

  /* Welcome state */
  .welcome-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    flex: 1;
    gap: 1rem;
    padding: 2rem;
    text-align: center;
  }

  .welcome-icon {
    font-size: 4rem;
    margin-bottom: 1rem;
  }

  .welcome-state h1 {
    margin: 0;
    font-size: 2.5rem;
    font-weight: 700;
    color: var(--color-text-primary);
  }

  .welcome-state p {
    color: var(--color-text-secondary);
    font-size: 1.125rem;
    margin: 0;
  }

  .open-button {
    background-color: var(--color-accent);
    color: white;
    border: none;
    border-radius: var(--radius);
    padding: 0.875rem 2rem;
    font-size: 1rem;
    font-weight: 500;
    cursor: pointer;
    transition: background-color 0.2s ease;
    margin-top: 0.5rem;
  }

  .open-button:hover {
    background-color: var(--color-accent-hover);
  }

  .hint {
    font-size: 0.875rem;
    color: var(--color-text-tertiary);
    margin-top: 1rem;
  }

  kbd {
    background-color: var(--color-bg-tertiary);
    color: var(--color-text-secondary);
    border: 1px solid var(--color-border);
    border-radius: 3px;
    padding: 0.125rem 0.375rem;
    font-size: 0.75rem;
    font-family: var(--font-mono);
  }
</style>

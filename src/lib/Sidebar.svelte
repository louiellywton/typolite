<script lang="ts">
  import { createEventDispatcher, onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/tauri';
  
  import { recentFiles, tableOfContents, currentFileInfo } from './stores';
  import type { FileMetadata } from './types';

  const dispatch = createEventDispatcher<{
    openFile: string;
  }>();

  let activeTab: 'files' | 'outline' = 'files';
  let isLoadingRecent = false;

  onMount(async () => {
    await loadRecentFiles();
  });

  async function loadRecentFiles() {
    try {
      isLoadingRecent = true;
      const result = await invoke('list_recent_files');
      
      if (result.success && result.data) {
        recentFiles.set(result.data);
      }
    } catch (error) {
      console.error('Failed to load recent files:', error);
    } finally {
      isLoadingRecent = false;
    }
  }

  function formatDate(timestamp: number): string {
    const date = new Date(timestamp * 1000);
    const now = new Date();
    const diffMs = now.getTime() - date.getTime();
    const diffHours = diffMs / (1000 * 60 * 60);
    
    if (diffHours < 1) {
      const diffMinutes = Math.floor(diffMs / (1000 * 60));
      return `${diffMinutes}m ago`;
    } else if (diffHours < 24) {
      return `${Math.floor(diffHours)}h ago`;
    } else if (diffHours < 24 * 7) {
      const diffDays = Math.floor(diffHours / 24);
      return `${diffDays}d ago`;
    } else {
      return date.toLocaleDateString();
    }
  }

  function formatFileSize(bytes: number): string {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i];
  }

  function getFileName(path: string): string {
    const parts = path.split('/');
    return parts[parts.length - 1];
  }

  function getFileDirectory(path: string): string {
    const parts = path.split('/');
    return parts.slice(0, -1).join('/');
  }

  function scrollToHeading(anchor: string) {
    const element = document.getElementById(anchor);
    if (element) {
      element.scrollIntoView({ behavior: 'smooth', block: 'start' });
    }
  }

  function getHeadingIndentClass(level: number): string {
    return `heading-level-${Math.min(level, 6)}`;
  }
</script>

<aside class="sidebar">
  <div class="sidebar-header">
    <div class="tab-buttons">
      <button 
        class="tab-button"
        class:active={activeTab === 'files'}
        on:click={() => activeTab = 'files'}
      >
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M13 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/>
          <polyline points="13,2 13,8 19,8"/>
        </svg>
        Files
      </button>
      
      <button 
        class="tab-button"
        class:active={activeTab === 'outline'}
        on:click={() => activeTab = 'outline'}
        disabled={$tableOfContents.length === 0}
      >
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <line x1="8" y1="6" x2="21" y2="6"/>
          <line x1="8" y1="12" x2="21" y2="12"/>
          <line x1="8" y1="18" x2="21" y2="18"/>
          <line x1="3" y1="6" x2="3.01" y2="6"/>
          <line x1="3" y1="12" x2="3.01" y2="12"/>
          <line x1="3" y1="18" x2="3.01" y2="18"/>
        </svg>
        Outline
        {#if $tableOfContents.length > 0}
          <span class="tab-count">{$tableOfContents.length}</span>
        {/if}
      </button>
    </div>
  </div>

  <div class="sidebar-content">
    {#if activeTab === 'files'}
      <div class="files-panel">
        <div class="panel-header">
          <h3>Recent Files</h3>
          <button 
            class="refresh-button" 
            on:click={loadRecentFiles}
            disabled={isLoadingRecent}
            title="Refresh"
          >
            <svg 
              width="14" 
              height="14" 
              viewBox="0 0 24 24" 
              fill="none" 
              stroke="currentColor" 
              stroke-width="2"
              class:spinning={isLoadingRecent}
            >
              <polyline points="23 4 23 10 17 10"/>
              <path d="M20.49 15a9 9 0 1 1-2.12-9.36L23 10"/>
            </svg>
          </button>
        </div>

        <div class="file-list">
          {#if isLoadingRecent}
            <div class="loading-state">
              <div class="small-spinner"></div>
              <span>Loading files...</span>
            </div>
          {:else if $recentFiles.length === 0}
            <div class="empty-state">
              <div class="empty-icon">📂</div>
              <p>No recent files</p>
              <button class="open-file-button" on:click={() => dispatch('openFile', '')}>
                Open a file to get started
              </button>
            </div>
          {:else}
            {#each $recentFiles as file (file.path)}
              <button 
                class="file-item"
                class:active={$currentFileInfo?.path === file.path}
                on:click={() => dispatch('openFile', file.path)}
                title={file.path}
              >
                <div class="file-icon">📄</div>
                <div class="file-info">
                  <div class="file-name">{getFileName(file.path)}</div>
                  <div class="file-meta">
                    <span class="file-size">{formatFileSize(file.size)}</span>
                    <span class="file-date">{formatDate(file.modified)}</span>
                  </div>
                </div>
              </button>
            {/each}
          {/if}
        </div>
      </div>
    {:else if activeTab === 'outline'}
      <div class="outline-panel">
        <div class="panel-header">
          <h3>Document Outline</h3>
        </div>

        <div class="outline-list">
          {#if $tableOfContents.length === 0}
            <div class="empty-state">
              <div class="empty-icon">📝</div>
              <p>No headings found</p>
              <small>Open a markdown file with headings to see the outline</small>
            </div>
          {:else}
            {#each $tableOfContents as heading (heading.anchor)}
              <button 
                class="outline-item {getHeadingIndentClass(heading.level)}"
                on:click={() => scrollToHeading(heading.anchor)}
                title={heading.title}
              >
                <div class="heading-marker">
                  {'#'.repeat(heading.level)}
                </div>
                <div class="heading-title">{heading.title}</div>
              </button>
            {/each}
          {/if}
        </div>
      </div>
    {/if}
  </div>
</aside>

<style>
  .sidebar {
    position: fixed;
    top: var(--size-header);
    left: 0;
    width: var(--size-sidebar);
    height: calc(100vh - var(--size-header));
    background-color: var(--color-bg-secondary);
    border-right: 1px solid var(--color-border);
    display: flex;
    flex-direction: column;
    z-index: 5;
  }

  .sidebar-header {
    padding: 1rem;
    border-bottom: 1px solid var(--color-border);
    flex-shrink: 0;
  }

  .tab-buttons {
    display: flex;
    background-color: var(--color-bg-tertiary);
    border-radius: var(--radius);
    padding: 2px;
    gap: 2px;
  }

  .tab-button {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 0.5rem;
    padding: 0.5rem;
    border: none;
    background: none;
    border-radius: var(--radius);
    font-size: 0.75rem;
    font-weight: 500;
    color: var(--color-text-secondary);
    cursor: pointer;
    transition: all 0.2s ease;
    position: relative;
  }

  .tab-button:hover:not(:disabled) {
    color: var(--color-text-primary);
  }

  .tab-button.active {
    background-color: var(--color-bg-primary);
    color: var(--color-text-primary);
    box-shadow: var(--shadow-sm);
  }

  .tab-button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .tab-count {
    background-color: var(--color-accent);
    color: white;
    border-radius: 10px;
    padding: 0.125rem 0.375rem;
    font-size: 0.625rem;
    font-weight: 600;
    min-width: 18px;
    height: 18px;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .sidebar-content {
    flex: 1;
    overflow: hidden;
    display: flex;
    flex-direction: column;
  }

  .files-panel,
  .outline-panel {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-height: 0;
  }

  .panel-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 1rem;
    border-bottom: 1px solid var(--color-border);
    flex-shrink: 0;
  }

  .panel-header h3 {
    font-size: 0.875rem;
    font-weight: 600;
    color: var(--color-text-primary);
    margin: 0;
  }

  .refresh-button {
    width: 24px;
    height: 24px;
    border: none;
    background: none;
    color: var(--color-text-secondary);
    cursor: pointer;
    border-radius: var(--radius);
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all 0.2s ease;
  }

  .refresh-button:hover:not(:disabled) {
    background-color: var(--color-bg-tertiary);
    color: var(--color-text-primary);
  }

  .refresh-button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .spinning {
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    0% { transform: rotate(0deg); }
    100% { transform: rotate(360deg); }
  }

  .file-list,
  .outline-list {
    flex: 1;
    overflow-y: auto;
    padding: 0.5rem;
  }

  .loading-state {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 0.5rem;
    padding: 2rem;
    color: var(--color-text-secondary);
    font-size: 0.875rem;
  }

  .small-spinner {
    width: 16px;
    height: 16px;
    border: 2px solid var(--color-border);
    border-top: 2px solid var(--color-accent);
    border-radius: 50%;
    animation: spin 1s linear infinite;
  }

  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 2rem 1rem;
    text-align: center;
    color: var(--color-text-secondary);
  }

  .empty-icon {
    font-size: 2rem;
    margin-bottom: 1rem;
  }

  .empty-state p {
    font-size: 0.875rem;
    font-weight: 500;
    margin: 0 0 0.5rem 0;
    color: var(--color-text-primary);
  }

  .empty-state small {
    font-size: 0.75rem;
    color: var(--color-text-tertiary);
    margin: 0;
  }

  .open-file-button {
    margin-top: 1rem;
    padding: 0.5rem 1rem;
    background-color: var(--color-accent);
    color: white;
    border: none;
    border-radius: var(--radius);
    font-size: 0.75rem;
    font-weight: 500;
    cursor: pointer;
    transition: background-color 0.2s ease;
  }

  .open-file-button:hover {
    background-color: var(--color-accent-hover);
  }

  .file-item {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    width: 100%;
    padding: 0.75rem;
    border: none;
    background: none;
    border-radius: var(--radius);
    cursor: pointer;
    transition: background-color 0.2s ease;
    text-align: left;
  }

  .file-item:hover {
    background-color: var(--color-bg-tertiary);
  }

  .file-item.active {
    background-color: var(--color-accent);
    color: white;
  }

  .file-icon {
    font-size: 18px;
    flex-shrink: 0;
  }

  .file-info {
    flex: 1;
    min-width: 0;
  }

  .file-name {
    font-size: 0.875rem;
    font-weight: 500;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    margin-bottom: 0.25rem;
  }

  .file-meta {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-size: 0.75rem;
    opacity: 0.7;
  }

  .outline-item {
    display: flex;
    align-items: flex-start;
    gap: 0.5rem;
    width: 100%;
    padding: 0.5rem;
    border: none;
    background: none;
    border-radius: var(--radius);
    cursor: pointer;
    transition: background-color 0.2s ease;
    text-align: left;
  }

  .outline-item:hover {
    background-color: var(--color-bg-tertiary);
  }

  .heading-marker {
    font-family: var(--font-mono);
    font-size: 0.75rem;
    color: var(--color-accent);
    font-weight: 600;
    flex-shrink: 0;
    line-height: 1.4;
  }

  .heading-title {
    font-size: 0.875rem;
    line-height: 1.4;
    color: var(--color-text-primary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  /* Heading level indentation */
  .heading-level-1 { padding-left: 0.5rem; }
  .heading-level-2 { padding-left: 1rem; }
  .heading-level-3 { padding-left: 1.5rem; }
  .heading-level-4 { padding-left: 2rem; }
  .heading-level-5 { padding-left: 2.5rem; }
  .heading-level-6 { padding-left: 3rem; }

  /* Scrollbar styles */
  .file-list::-webkit-scrollbar,
  .outline-list::-webkit-scrollbar {
    width: 6px;
  }

  .file-list::-webkit-scrollbar-track,
  .outline-list::-webkit-scrollbar-track {
    background: transparent;
  }

  .file-list::-webkit-scrollbar-thumb,
  .outline-list::-webkit-scrollbar-thumb {
    background: var(--color-border-hover);
    border-radius: 3px;
  }

  .file-list::-webkit-scrollbar-thumb:hover,
  .outline-list::-webkit-scrollbar-thumb:hover {
    background: var(--color-text-tertiary);
  }
</style>

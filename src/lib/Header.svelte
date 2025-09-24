<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { invoke } from '@tauri-apps/api/tauri';
  
  import { currentFileInfo, documentStats, theme, sidebarOpen, storeUtils } from './stores';
  import type { ExportFormat } from './types';

  export let isLoading: boolean = false;
  export let error: string | null = null;

  const dispatch = createEventDispatcher<{
    openFile: void;
    exportFile: { format: ExportFormat };
  }>();

  async function handleExport(format: ExportFormat) {
    if (!$currentFileInfo) return;

    try {
      const result = await invoke('export_to_pdf', {
        htmlContent: '', // This would come from the parsed document
        outputPath: $currentFileInfo.path.replace(/\.[^.]+$/, `.${format.toLowerCase()}`),
        options: {
          format,
          include_toc: true,
          page_size: 'A4',
          margins: { top: 1, right: 1, bottom: 1, left: 1 }
        }
      });

      if (result.success) {
        console.log('Export successful:', result.data);
      } else {
        console.error('Export failed:', result.error);
      }
    } catch (err) {
      console.error('Export error:', err);
    }

    dispatch('exportFile', { format });
  }

  function formatFileSize(bytes: number): string {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i];
  }

  function getThemeIcon(theme: string): string {
    switch (theme) {
      case 'light': return '☀️';
      case 'dark': return '🌙';
      case 'auto': return '🌓';
      default: return '🌓';
    }
  }
</script>

<header class="header">
  <div class="header-left">
    <button 
      class="sidebar-toggle"
      class:active={$sidebarOpen}
      on:click={() => sidebarOpen.update(open => !open)}
      title="Toggle Sidebar (Ctrl+\)"
    >
      <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <rect x="3" y="3" width="18" height="18" rx="2" ry="2"/>
        <line x1="9" y1="9" x2="21" y2="9"/>
        <line x1="9" y1="15" x2="21" y2="15"/>
      </svg>
    </button>

    <div class="file-info">
      {#if $currentFileInfo}
        <div class="file-name">{$currentFileInfo.filename}</div>
        <div class="file-path">{$currentFileInfo.directory}</div>
      {:else}
        <div class="app-title">Typora-Lite</div>
      {/if}
    </div>
  </div>

  <div class="header-center">
    {#if $documentStats && $documentStats.wordCount > 0}
      <div class="document-stats">
        <span class="stat">
          <span class="stat-value">{$documentStats.wordCount.toLocaleString()}</span>
          <span class="stat-label">words</span>
        </span>
        <span class="stat-divider">•</span>
        <span class="stat">
          <span class="stat-value">{$documentStats.readingTime}</span>
          <span class="stat-label">min read</span>
        </span>
        {#if $documentStats.headingCount > 0}
          <span class="stat-divider">•</span>
          <span class="stat">
            <span class="stat-value">{$documentStats.headingCount}</span>
            <span class="stat-label">headings</span>
          </span>
        {/if}
      </div>
    {/if}

    {#if error}
      <div class="error-badge" title={error}>
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <circle cx="12" cy="12" r="10"/>
          <line x1="12" y1="8" x2="12" y2="12"/>
          <line x1="12" y1="16" x2="12.01" y2="16"/>
        </svg>
        Error
      </div>
    {/if}

    {#if isLoading}
      <div class="loading-badge">
        <div class="mini-spinner"></div>
        Loading...
      </div>
    {/if}
  </div>

  <div class="header-right">
    <div class="header-actions">
      <button 
        class="action-button"
        on:click={() => dispatch('openFile')}
        title="Open File (Ctrl+O)"
      >
        <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/>
          <polyline points="14,2 14,8 20,8"/>
          <line x1="16" y1="13" x2="8" y2="13"/>
          <line x1="16" y1="17" x2="8" y2="17"/>
          <line x1="10" y1="9" x2="8" y2="9"/>
        </svg>
      </button>

      {#if $currentFileInfo}
        <div class="export-menu">
          <button class="action-button export-trigger" title="Export">
            <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/>
              <polyline points="7,10 12,15 17,10"/>
              <line x1="12" y1="15" x2="12" y2="3"/>
            </svg>
          </button>
          
          <div class="export-dropdown">
            <button class="dropdown-item" on:click={() => handleExport('Pdf')}>
              <span class="export-icon">📄</span>
              Export as PDF
            </button>
            <button class="dropdown-item" on:click={() => handleExport('Html')}>
              <span class="export-icon">🌐</span>
              Export as HTML
            </button>
          </div>
        </div>
      {/if}

      <button 
        class="action-button theme-toggle"
        on:click={storeUtils.toggleTheme}
        title="Toggle Theme"
      >
        <span class="theme-icon">{getThemeIcon($theme)}</span>
      </button>
    </div>
  </div>
</header>

<style>
  .header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    height: var(--size-header);
    padding: 0 1rem;
    background-color: var(--color-bg-secondary);
    border-bottom: 1px solid var(--color-border);
    flex-shrink: 0;
    position: relative;
    z-index: 10;
  }

  .header-left,
  .header-right {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    flex: 1;
  }

  .header-right {
    justify-content: flex-end;
  }

  .header-center {
    display: flex;
    align-items: center;
    gap: 1rem;
    flex: 2;
    justify-content: center;
  }

  .sidebar-toggle {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 32px;
    height: 32px;
    border: none;
    background: none;
    border-radius: var(--radius);
    color: var(--color-text-secondary);
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .sidebar-toggle:hover {
    background-color: var(--color-bg-tertiary);
    color: var(--color-text-primary);
  }

  .sidebar-toggle.active {
    background-color: var(--color-accent);
    color: white;
  }

  .file-info {
    min-width: 0;
    flex: 1;
  }

  .file-name {
    font-size: 0.875rem;
    font-weight: 600;
    color: var(--color-text-primary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .file-path {
    font-size: 0.75rem;
    color: var(--color-text-tertiary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .app-title {
    font-size: 0.875rem;
    font-weight: 600;
    color: var(--color-text-primary);
  }

  .document-stats {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-size: 0.75rem;
    color: var(--color-text-secondary);
  }

  .stat {
    display: flex;
    align-items: center;
    gap: 0.25rem;
  }

  .stat-value {
    font-weight: 600;
    color: var(--color-text-primary);
  }

  .stat-label {
    color: var(--color-text-tertiary);
  }

  .stat-divider {
    color: var(--color-border);
  }

  .error-badge {
    display: flex;
    align-items: center;
    gap: 0.375rem;
    padding: 0.25rem 0.5rem;
    background-color: var(--color-error);
    color: white;
    border-radius: var(--radius);
    font-size: 0.75rem;
    font-weight: 500;
  }

  .loading-badge {
    display: flex;
    align-items: center;
    gap: 0.375rem;
    padding: 0.25rem 0.5rem;
    background-color: var(--color-accent);
    color: white;
    border-radius: var(--radius);
    font-size: 0.75rem;
    font-weight: 500;
  }

  .mini-spinner {
    width: 12px;
    height: 12px;
    border: 2px solid rgba(255, 255, 255, 0.3);
    border-top: 2px solid white;
    border-radius: 50%;
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    0% { transform: rotate(0deg); }
    100% { transform: rotate(360deg); }
  }

  .header-actions {
    display: flex;
    align-items: center;
    gap: 0.25rem;
  }

  .action-button {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 32px;
    height: 32px;
    border: none;
    background: none;
    border-radius: var(--radius);
    color: var(--color-text-secondary);
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .action-button:hover {
    background-color: var(--color-bg-tertiary);
    color: var(--color-text-primary);
  }

  .theme-toggle .theme-icon {
    font-size: 16px;
  }

  .export-menu {
    position: relative;
  }

  .export-menu:hover .export-dropdown {
    opacity: 1;
    visibility: visible;
    transform: translateY(0);
  }

  .export-dropdown {
    position: absolute;
    top: 100%;
    right: 0;
    background-color: var(--color-bg-primary);
    border: 1px solid var(--color-border);
    border-radius: var(--radius);
    box-shadow: var(--shadow-lg);
    padding: 0.5rem 0;
    min-width: 160px;
    z-index: 1000;
    opacity: 0;
    visibility: hidden;
    transform: translateY(-8px);
    transition: all 0.2s ease;
  }

  .dropdown-item {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    width: 100%;
    padding: 0.5rem 1rem;
    border: none;
    background: none;
    color: var(--color-text-primary);
    font-size: 0.875rem;
    text-align: left;
    cursor: pointer;
    transition: background-color 0.2s ease;
  }

  .dropdown-item:hover {
    background-color: var(--color-bg-secondary);
  }

  .export-icon {
    font-size: 14px;
  }

  @media (max-width: 768px) {
    .header {
      padding: 0 0.5rem;
    }

    .document-stats {
      display: none;
    }

    .header-center {
      flex: 1;
    }

    .file-path {
      display: none;
    }
  }
</style>

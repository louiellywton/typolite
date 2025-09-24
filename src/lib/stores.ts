import { writable, derived, readable } from 'svelte/store';
import type { ParsedDocument, FileMetadata, Theme, AppConfig } from './types';

// Core application state
export const currentFile = writable<string | null>(null);
export const parsedDocument = writable<ParsedDocument | null>(null);
export const recentFiles = writable<FileMetadata[]>([]);

// UI state
export const theme = writable<Theme>('auto');
export const sidebarOpen = writable<boolean>(false);
export const isLoading = writable<boolean>(false);
export const error = writable<string | null>(null);

// Document stats (derived from parsedDocument)
export const documentStats = derived(parsedDocument, ($parsedDocument) => {
  if (!$parsedDocument) {
    return {
      wordCount: 0,
      readingTime: 0,
      headingCount: 0,
    };
  }

  return {
    wordCount: $parsedDocument.word_count,
    readingTime: $parsedDocument.reading_time,
    headingCount: $parsedDocument.toc.length,
  };
});

// Table of contents (derived from parsedDocument)
export const tableOfContents = derived(parsedDocument, ($parsedDocument) => {
  return $parsedDocument?.toc || [];
});

// Current file info (derived from currentFile)
export const currentFileInfo = derived(currentFile, ($currentFile) => {
  if (!$currentFile) return null;
  
  const parts = $currentFile.split('/');
  const filename = parts[parts.length - 1];
  const directory = parts.slice(0, -1).join('/');
  
  return {
    path: $currentFile,
    filename,
    directory,
    extension: filename.split('.').pop() || '',
  };
});

// Application configuration
export const appConfig = writable<AppConfig>({
  theme: 'auto',
  sidebar_open: false,
  recent_files: [],
  export_settings: {
    format: 'Pdf',
    include_toc: true,
    page_size: 'A4',
    margins: {
      top: 1,
      right: 1,
      bottom: 1,
      left: 1,
    },
  },
});

// System information (read-only)
export const systemInfo = readable({ os: 'unknown', arch: 'unknown', version: '0.1.0' }, (set) => {
  // This would be populated by a Tauri command in a real app
  set({ os: 'macOS', arch: 'arm64', version: '0.1.0' });
});

// Utility functions for stores
export const storeUtils = {
  // Reset all stores to initial state
  reset: () => {
    currentFile.set(null);
    parsedDocument.set(null);
    recentFiles.set([]);
    error.set(null);
    isLoading.set(false);
  },

  // Add a file to recent files
  addRecentFile: (file: FileMetadata) => {
    recentFiles.update(files => {
      const filtered = files.filter(f => f.path !== file.path);
      return [file, ...filtered].slice(0, 10); // Keep only 10 recent files
    });
  },

  // Remove a file from recent files
  removeRecentFile: (path: string) => {
    recentFiles.update(files => files.filter(f => f.path !== path));
  },

  // Set error with auto-clear
  setError: (message: string, autoClean = 5000) => {
    error.set(message);
    if (autoClean > 0) {
      setTimeout(() => error.set(null), autoClean);
    }
  },

  // Toggle theme
  toggleTheme: () => {
    theme.update(current => {
      switch (current) {
        case 'light': return 'dark';
        case 'dark': return 'auto';
        case 'auto': return 'light';
        default: return 'light';
      }
    });
  },

  // Load configuration from localStorage
  loadConfig: () => {
    try {
      const saved = localStorage.getItem('typolite-config');
      if (saved) {
        const config = JSON.parse(saved);
        appConfig.set(config);
        theme.set(config.theme || 'auto');
        sidebarOpen.set(config.sidebar_open || false);
      }
    } catch (error) {
      console.warn('Failed to load configuration:', error);
    }
  },

  // Save configuration to localStorage
  saveConfig: (config: AppConfig) => {
    try {
      localStorage.setItem('typolite-config', JSON.stringify(config));
    } catch (error) {
      console.warn('Failed to save configuration:', error);
    }
  },
};

// Auto-save configuration when it changes
appConfig.subscribe(config => {
  storeUtils.saveConfig(config);
});

// Sync individual stores with appConfig
theme.subscribe(value => {
  appConfig.update(config => ({ ...config, theme: value }));
});

sidebarOpen.subscribe(value => {
  appConfig.update(config => ({ ...config, sidebar_open: value }));
});

import { describe, it, expect, beforeEach } from 'vitest';
import { get } from 'svelte/store';
import { 
  currentFile, 
  parsedDocument, 
  documentStats, 
  theme, 
  recentFiles,
  storeUtils 
} from './stores';
import type { ParsedDocument } from './types';

describe('Stores', () => {
  beforeEach(() => {
    storeUtils.reset();
  });

  describe('currentFile', () => {
    it('should start with null', () => {
      expect(get(currentFile)).toBeNull();
    });

    it('should update when set', () => {
      currentFile.set('/path/to/file.md');
      expect(get(currentFile)).toBe('/path/to/file.md');
    });
  });

  describe('parsedDocument', () => {
    it('should start with null', () => {
      expect(get(parsedDocument)).toBeNull();
    });

    it('should update when set', () => {
      const doc: ParsedDocument = {
        html: '<h1>Test</h1>',
        line_map: [0, 10],
        toc: [{ level: 1, title: 'Test', anchor: 'test', line: 1 }],
        word_count: 1,
        reading_time: 1
      };

      parsedDocument.set(doc);
      expect(get(parsedDocument)).toEqual(doc);
    });
  });

  describe('documentStats', () => {
    it('should return zero stats when no document', () => {
      const stats = get(documentStats);
      expect(stats).toEqual({
        wordCount: 0,
        readingTime: 0,
        headingCount: 0,
      });
    });

    it('should compute stats from parsed document', () => {
      const doc: ParsedDocument = {
        html: '<h1>Test</h1><p>Some content here</p>',
        line_map: [0, 10, 20],
        toc: [
          { level: 1, title: 'Test', anchor: 'test', line: 1 },
          { level: 2, title: 'Subtitle', anchor: 'subtitle', line: 2 }
        ],
        word_count: 15,
        reading_time: 2
      };

      parsedDocument.set(doc);
      const stats = get(documentStats);
      
      expect(stats).toEqual({
        wordCount: 15,
        readingTime: 2,
        headingCount: 2,
      });
    });
  });

  describe('theme', () => {
    it('should start with auto theme', () => {
      expect(get(theme)).toBe('auto');
    });

    it('should cycle through themes when toggled', () => {
      // Initial: auto
      expect(get(theme)).toBe('auto');

      storeUtils.toggleTheme();
      expect(get(theme)).toBe('light');

      storeUtils.toggleTheme();
      expect(get(theme)).toBe('dark');

      storeUtils.toggleTheme();
      expect(get(theme)).toBe('auto');
    });
  });

  describe('storeUtils', () => {
    it('should reset all stores', () => {
      // Set some values
      currentFile.set('/test.md');
      parsedDocument.set({
        html: '<h1>Test</h1>',
        line_map: [0],
        toc: [],
        word_count: 1,
        reading_time: 1
      });

      // Reset
      storeUtils.reset();

      // Check all are reset
      expect(get(currentFile)).toBeNull();
      expect(get(parsedDocument)).toBeNull();
    });

    it('should add recent files correctly', () => {
      const file1 = {
        path: '/file1.md',
        size: 1024,
        modified: Date.now() / 1000,
        is_markdown: true
      };

      const file2 = {
        path: '/file2.md',
        size: 2048,
        modified: Date.now() / 1000,
        is_markdown: true
      };

      storeUtils.addRecentFile(file1);
      storeUtils.addRecentFile(file2);

      const files = get(recentFiles);
      expect(files).toHaveLength(2);
      expect(files[0]).toEqual(file2); // Most recent first
      expect(files[1]).toEqual(file1);
    });

    it('should not duplicate files in recent list', () => {
      const file = {
        path: '/file.md',
        size: 1024,
        modified: Date.now() / 1000,
        is_markdown: true
      };

      storeUtils.addRecentFile(file);
      storeUtils.addRecentFile(file); // Add same file again

      const files = get(recentFiles);
      expect(files).toHaveLength(1);
      expect(files[0]).toEqual(file);
    });

    it('should limit recent files to 10', () => {
      // Add 15 files
      for (let i = 0; i < 15; i++) {
        storeUtils.addRecentFile({
          path: `/file${i}.md`,
          size: 1024,
          modified: Date.now() / 1000,
          is_markdown: true
        });
      }

      const files = get(recentFiles);
      expect(files).toHaveLength(10);
      expect(files[0].path).toBe('/file14.md'); // Most recent
    });
  });
});

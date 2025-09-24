// Tauri API types
export interface CommandResult<T> {
  success: boolean;
  data?: T;
  error?: string;
}

// Markdown parser types
export interface TocItem {
  level: number;
  title: string;
  anchor: string;
  line: number;
}

export interface ParsedDocument {
  html: string;
  line_map: number[];
  toc: TocItem[];
  word_count: number;
  reading_time: number;
}

// File service types
export interface FileMetadata {
  path: string;
  size: number;
  modified: number;
  is_markdown: boolean;
}

export interface FileChangeEvent {
  path: string;
  event_type: FileEventType;
}

export type FileEventType = 
  | 'Created'
  | 'Modified' 
  | 'Deleted'
  | { Renamed: { from: string; to: string } };

// Export service types
export interface ExportOptions {
  format: ExportFormat;
  include_toc: boolean;
  page_size: PageSize;
  margins: Margins;
  header?: string;
  footer?: string;
  css_theme?: string;
}

export type ExportFormat = 'Pdf' | 'Html' | 'Docx';

export type PageSize = 'A4' | 'Letter' | 'Legal' | 'A3' | 'A5';

export interface Margins {
  top: number;
  right: number;
  bottom: number;
  left: number;
}

export interface ExportResult {
  output_path: string;
  file_size: number;
  pages: number;
  export_time_ms: number;
}

// Theme types
export type Theme = 'light' | 'dark' | 'auto';

export interface ThemeConfig {
  name: string;
  author?: string;
  license?: string;
  colors: {
    bg: string;
    fg: string;
    accent?: string;
    [key: string]: string | undefined;
  };
}

// Application state types
export interface AppConfig {
  theme: Theme;
  sidebar_open: boolean;
  recent_files: string[];
  export_settings: Partial<ExportOptions>;
}

# Typora-Lite Architecture

## Overview

Typora-Lite is built with a modern desktop application architecture using Tauri (Rust backend) and Svelte (frontend). This design prioritizes performance, security, and maintainability while achieving the target metrics of <120ms startup time and <80MB memory usage.

## Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────┐
│                    Frontend (Svelte + TypeScript)              │
├─────────────────────────────────────────────────────────────────┤
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐  │
│  │   App.svelte    │  │ MarkdownView    │  │   Sidebar       │  │
│  │   (Main Shell)  │  │  (Renderer)     │  │ (Navigation)    │  │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘  │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐  │
│  │    Header       │  │     Stores      │  │     Types       │  │
│  │  (Controls)     │  │ (State Mgmt)    │  │ (Definitions)   │  │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘  │
├═════════════════════════════════════════════════════════════════┤
│                     Tauri IPC Layer                            │
├═════════════════════════════════════════════════════════════════┤
│                    Backend (Rust)                              │
├─────────────────────────────────────────────────────────────────┤
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐  │
│  │ Markdown Parser │  │  File Service   │  │ Export Service  │  │
│  │ (pulldown-cmark)│  │ (tokio + notify)│  │  (PDF/HTML)     │  │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘  │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐  │
│  │    Commands     │  │   Main Loop     │  │   Menu System   │  │
│  │  (API Bridge)   │  │   (Event)       │  │  (Native UI)    │  │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘  │
├─────────────────────────────────────────────────────────────────┤
│                    System Layer                                │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐  │
│  │   WebView2      │  │   File System   │  │  Auto Updater   │  │
│  │   (Windows)     │  │   (Native)      │  │   (Built-in)    │  │
│  │   WKWebView     │  │                 │  │                 │  │
│  │   (macOS)       │  │                 │  │                 │  │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
```

## Core Components

### Frontend (Svelte + TypeScript)

#### Component Architecture
- **App.svelte**: Main application shell handling global state and routing
- **MarkdownView.svelte**: Core markdown rendering with KaTeX, Mermaid, and Prism.js
- **Header.svelte**: Top navigation with file info, stats, and controls
- **Sidebar.svelte**: File browser and document outline navigation

#### State Management
- **Svelte Stores**: Reactive state management using built-in stores
- **Derived Stores**: Computed values for document stats and navigation
- **Persistent Storage**: localStorage for user preferences and recent files

#### Libraries Integration
- **KaTeX**: Math rendering with LaTeX syntax
- **Mermaid**: Diagram and flowchart rendering
- **Prism.js**: Syntax highlighting for code blocks
- **Tauri API**: Native system integration

### Backend (Rust)

#### Core Services
- **MarkdownParser**: Fast CommonMark + GFM parsing with pulldown-cmark
- **FileService**: Atomic file operations with change watching via notify
- **ExportService**: PDF and HTML export with styling and TOC generation
- **CommandsAPI**: Tauri commands bridge for frontend communication

#### Performance Features
- **Async Operations**: All I/O operations using tokio for non-blocking execution
- **Debounced File Watching**: Prevents excessive reload events
- **Memory Management**: Careful ownership and borrowing for minimal memory usage
- **SIMD Optimizations**: Enabled in pulldown-cmark for faster parsing

### Security Model

#### Tauri Security
- **Allowlist-based Permissions**: Only necessary APIs enabled
- **CSP (Content Security Policy)**: Strict policies for XSS prevention
- **Sandboxed WebView**: Isolated execution environment
- **Code Signing**: Platform-specific signing for authenticity

#### File System Access
- **Scoped Permissions**: Limited to user documents and app data directories
- **Path Validation**: All file operations validated against allowed paths
- **Atomic Operations**: Temporary files for safe writes

## Data Flow

### File Loading Process
```
1. User clicks "Open File" or uses Cmd+O
   ↓
2. Frontend calls open_file_dialog command
   ↓
3. Backend shows native file dialog
   ↓
4. User selects markdown file
   ↓
5. Backend reads file with read_markdown_file
   ↓
6. Backend parses with parse_markdown
   ↓
7. Parsed document sent to frontend
   ↓
8. MarkdownView renders HTML with libraries
   ↓
9. File watcher starts monitoring changes
```

### Live Reload Process
```
1. File system change detected by notify
   ↓
2. FileService debounces multiple events
   ↓
3. FileChangeEvent emitted to frontend
   ↓
4. Frontend receives file-changed event
   ↓
5. Auto-reload triggered if same file
   ↓
6. New content parsed and rendered
   ↓
7. Scroll position preserved where possible
```

## Performance Optimizations

### Startup Performance
- **Lazy Loading**: Libraries loaded on-demand
- **Code Splitting**: Vite automatically splits large dependencies
- **WebView Preloading**: HTML template includes inline critical CSS
- **Binary Size**: Rust binary optimized for size with minimal dependencies

### Runtime Performance
- **Virtual Scrolling**: Large documents handled efficiently
- **Debounced Operations**: File watching and parsing optimized
- **Memory Pools**: Reuse allocations where possible
- **Progressive Enhancement**: Features load incrementally

### Bundle Optimization
- **Tree Shaking**: Unused code eliminated during build
- **Dynamic Imports**: Large libraries loaded asynchronously
- **Asset Optimization**: Images and fonts optimized for size
- **Compression**: Gzip compression for static assets

## Testing Strategy

### Unit Testing
- **Rust Tests**: Built-in test framework for parser and file operations
- **Frontend Tests**: Vitest for component and utility testing
- **Type Safety**: TypeScript strict mode for compile-time validation

### Integration Testing
- **Tauri Commands**: End-to-end testing of backend API
- **File Operations**: Temporary file testing for all I/O operations
- **Cross-platform**: CI testing on Windows, macOS, and Linux

### Performance Testing
- **Startup Benchmarks**: Automated timing of cold start process
- **Memory Profiling**: Heap analysis and leak detection
- **Large File Handling**: Testing with documents up to 100MB

## Accessibility

### WCAG 2.2 AA Compliance
- **Keyboard Navigation**: Full app usable without mouse
- **Screen Reader Support**: ARIA labels and semantic HTML
- **Color Contrast**: 4.5:1 minimum contrast ratios
- **Focus Management**: Visible focus indicators

### Platform Integration
- **Native Menus**: System-integrated menu bars
- **Keyboard Shortcuts**: Platform-specific conventions
- **High DPI**: Proper scaling on high-resolution displays
- **Dark Mode**: System preference detection and override

## Deployment

### Build Pipeline
- **Multi-platform**: GitHub Actions for Windows, macOS, Linux
- **Code Signing**: Platform-specific signing certificates
- **Auto-updates**: Built-in updater with signature verification
- **Installer Generation**: MSI, DMG, AppImage generation

### Distribution
- **Direct Download**: GitHub releases with checksums
- **Package Managers**: Future Homebrew, Chocolatey support
- **Enterprise Distribution**: Group policy and MDM support
- **Auto-update Server**: CDN-backed update distribution

## Future Architecture Considerations

### Phase 2 Features
- **Real-time Editing**: Operational transforms for collaborative editing
- **Plugin System**: WebAssembly-based plugin architecture
- **Sync Service**: End-to-end encrypted document synchronization
- **Advanced Export**: More output formats and custom styling

### Scalability
- **Multi-document**: Tabbed interface for multiple files
- **Workspace Management**: Project-based file organization
- **Search Index**: Full-text search across documents
- **Version Control**: Git integration for document history

This architecture ensures Typora-Lite meets its performance targets while maintaining security, accessibility, and maintainability for future development.

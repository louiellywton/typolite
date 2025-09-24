# Development Guide

## Prerequisites

### System Requirements
- **Node.js**: 18.0.0 or higher
- **Rust**: 1.70.0 or higher
- **pnpm**: 8.0.0 or higher

### Platform-Specific Requirements

#### macOS
```bash
# Install Xcode Command Line Tools
xcode-select --install

# Install Homebrew (optional, for dependencies)
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
```

#### Windows
- **Visual Studio Build Tools** or **Visual Studio Community**
- **WebView2** (usually pre-installed on Windows 11)

#### Linux (Ubuntu/Debian)
```bash
sudo apt update
sudo apt install build-essential curl wget file libssl-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev
```

## Installation

### 1. Clone the Repository
```bash
git clone https://github.com/louiellywton/typolite.git
cd typolite
```

### 2. Install Frontend Dependencies
```bash
pnpm install
```

### 3. Install Tauri CLI
```bash
cargo install tauri-cli
```

### 4. Install Rust Dependencies
```bash
cd src-tauri
cargo build
cd ..
```

## Development Workflow

### Running in Development Mode
```bash
# Start the development server with hot reload
pnpm tauri dev

# Alternative: Start frontend and backend separately
pnpm dev          # Frontend only (port 1420)
cargo tauri dev   # Full Tauri development
```

### Building for Production
```bash
# Build the application for your platform
pnpm tauri build

# Build frontend only
pnpm build

# Build backend only
cd src-tauri
cargo build --release
```

## Project Structure

```
typolite/
├── .github/                 # GitHub Actions workflows
├── docs/                    # Documentation
├── src/                     # Frontend source (Svelte + TypeScript)
│   ├── lib/                 # Shared components and utilities
│   │   ├── MarkdownView.svelte
│   │   ├── Header.svelte
│   │   ├── Sidebar.svelte
│   │   ├── stores.ts        # State management
│   │   └── types.ts         # TypeScript definitions
│   ├── App.svelte          # Main application component
│   ├── main.ts             # Application entry point
│   └── app.css             # Global styles
├── src-tauri/              # Backend source (Rust)
│   ├── src/                # Rust source files
│   │   ├── main.rs         # Application entry point
│   │   ├── lib.rs          # Library entry point
│   │   ├── parser.rs       # Markdown parsing service
│   │   ├── file_service.rs # File operations and watching
│   │   ├── export.rs       # Export functionality
│   │   └── commands.rs     # Tauri command definitions
│   ├── Cargo.toml         # Rust dependencies
│   └── tauri.conf.json    # Tauri configuration
├── tests/                  # Test files
├── scripts/               # Build and deployment scripts
├── package.json           # Node.js dependencies and scripts
├── vite.config.ts         # Vite configuration
├── tsconfig.json          # TypeScript configuration
└── svelte.config.js       # Svelte configuration
```

## Code Style and Standards

### Frontend (TypeScript/Svelte)
- **Formatter**: Prettier
- **Linter**: ESLint with TypeScript rules
- **Style Guide**: Airbnb TypeScript style guide

```bash
# Format code
pnpm format

# Lint code
pnpm lint

# Type checking
pnpm check
```

### Backend (Rust)
- **Formatter**: rustfmt
- **Linter**: clippy
- **Style Guide**: Official Rust style guide

```bash
# Format Rust code
cargo fmt

# Lint Rust code
cargo clippy

# Check compilation
cargo check
```

## Testing

### Running Tests
```bash
# Run all tests
pnpm test

# Run Rust tests
cd src-tauri
cargo test

# Run frontend tests
pnpm test:ui

# Run end-to-end tests
pnpm test:e2e

# Run accessibility tests
pnpm test:a11y
```

### Test Coverage
```bash
# Generate coverage report
pnpm test:coverage
```

### Writing Tests

#### Frontend Tests (Vitest)
```typescript
// src/lib/__tests__/utils.test.ts
import { describe, it, expect } from 'vitest';
import { formatFileSize } from '../utils';

describe('formatFileSize', () => {
  it('should format bytes correctly', () => {
    expect(formatFileSize(1024)).toBe('1.0 KB');
    expect(formatFileSize(1048576)).toBe('1.0 MB');
  });
});
```

#### Backend Tests (Rust)
```rust
// src-tauri/src/parser.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_markdown_parsing() {
        let parser = MarkdownParser::new();
        let result = parser.parse("# Hello World").unwrap();
        assert!(result.html.contains("<h1"));
    }
}
```

## Debugging

### Frontend Debugging
- **Browser DevTools**: Open with F12 in the Tauri window
- **Svelte DevTools**: Install browser extension for component inspection
- **Console Logging**: Use `console.log()` for debugging

### Backend Debugging
```bash
# Enable debug logging
RUST_LOG=typolite=debug pnpm tauri dev

# Use debugger
cargo build
rust-gdb target/debug/typolite
```

### Performance Profiling
```bash
# Profile startup time
RUST_LOG=trace pnpm tauri dev

# Memory profiling (requires tools)
cargo flamegraph --bin typolite
```

## Common Development Tasks

### Adding a New Tauri Command
1. Define command in `src-tauri/src/commands.rs`
2. Add to invoke handler in `src-tauri/src/main.rs`
3. Call from frontend with `invoke()`

```rust
// Backend
#[command]
pub async fn my_command(data: String) -> CommandResult<String> {
    CommandResult::ok(data.to_uppercase())
}

// Register in main.rs
.invoke_handler(tauri::generate_handler![my_command])
```

```typescript
// Frontend
import { invoke } from '@tauri-apps/api/tauri';

const result = await invoke('my_command', { data: 'hello' });
```

### Adding a New Frontend Component
1. Create component in `src/lib/`
2. Export types if needed in `src/lib/types.ts`
3. Import and use in parent component

```svelte
<!-- src/lib/MyComponent.svelte -->
<script lang="ts">
  export let title: string;
</script>

<h2>{title}</h2>
```

### Updating Dependencies
```bash
# Update frontend dependencies
pnpm update

# Update Rust dependencies
cd src-tauri
cargo update

# Check for outdated packages
pnpm outdated
cargo outdated  # requires cargo-outdated
```

## Environment Variables

### Development
```bash
# .env.development
TAURI_DEBUG=true
RUST_LOG=typolite=debug
VITE_API_URL=http://localhost:1420
```

### Production
```bash
# .env.production
TAURI_DEBUG=false
RUST_LOG=info
```

## Build Optimization

### Frontend Optimization
- **Bundle Analysis**: `pnpm analyze`
- **Tree Shaking**: Automatic with Vite
- **Code Splitting**: Dynamic imports for large libraries

### Backend Optimization
```toml
# Cargo.toml - Release profile
[profile.release]
opt-level = "s"  # Optimize for size
lto = true       # Link-time optimization
codegen-units = 1
panic = "abort"
strip = true     # Strip debug symbols
```

## Troubleshooting

### Common Issues

#### "Command not found: tauri"
```bash
cargo install tauri-cli
```

#### WebView2 Missing (Windows)
- Download from Microsoft WebView2 page
- Or use offline installer bundle

#### Build Fails on Linux
```bash
# Install missing dependencies
sudo apt install libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev
```

#### Port 1420 Already in Use
```bash
# Kill process using port
sudo lsof -ti:1420 | xargs kill -9
```

### Getting Help
- **GitHub Issues**: Report bugs and feature requests
- **Discussions**: Ask questions and share ideas
- **Discord**: Join the Tauri Discord for real-time help
- **Documentation**: Check Tauri and Svelte docs

## Contributing

### Submitting Changes
1. Fork the repository
2. Create a feature branch: `git checkout -b feature/my-feature`
3. Make changes and add tests
4. Run tests: `pnpm test`
5. Format code: `pnpm format`
6. Commit changes: `git commit -m "feat: add my feature"`
7. Push branch: `git push origin feature/my-feature`
8. Open a Pull Request

### Commit Message Format
Follow [Conventional Commits](https://www.conventionalcommits.org/):
```
type(scope): description

feat(parser): add support for math expressions
fix(ui): resolve sidebar toggle issue
docs(readme): update installation instructions
```

### PR Requirements
- [ ] Tests pass
- [ ] Code is formatted
- [ ] Documentation updated
- [ ] Accessibility tested
- [ ] Cross-platform compatibility verified

This development guide should help you get started with contributing to Typora-Lite!

# Typora-Lite

A fast, beautiful, cross-platform desktop Markdown viewer built with Tauri and Svelte.

## 🎯 Project Goals

- **Fast**: Launch in <120ms, occupy <80MB RAM
- **Compact**: Ship in a <10MB installer
- **Secure**: Enterprise-signable and auto-updating
- **Accessible**: 508/WCAG 2.2 AA compliant
- **Cross-platform**: Windows 10+, macOS 12+, Ubuntu 20.04+

## 🚀 Quick Start

### Prerequisites

- Rust 1.70+
- Node.js 18+
- pnpm

### Development Setup

```bash
# Install dependencies
pnpm install

# Install Tauri CLI
cargo install tauri-cli

# Start development server
pnpm tauri dev
```

### Build for Production

```bash
# Build release version
pnpm tauri build
```

## 📋 Features

### Phase 1 (Read-only Viewer)
- [x] Fast markdown parsing with CommonMark + GFM
- [x] Live file watching and reload
- [x] Theme system (light/dark/custom)
- [x] PDF export
- [x] KaTeX math rendering
- [x] Mermaid diagrams
- [x] Syntax highlighting with Prism.js
- [x] Accessibility compliance
- [x] Auto-updater

### Phase 2 (Future)
- [ ] Real-time editing
- [ ] Collaborative features
- [ ] Plugin system

## 🏗️ Architecture

```
┌-------------------------┐
│  Native Shell (Tauri)   │  Rust 1.70+
│  - menu, shortcuts      │
│  - file dialogs         │
│  - code signing         │
│  - updater              │
├-------------------------┤
│  WebView2 / WKWebView   │  Edge 109+, Safari 16+
├-------------------------┤
│  Renderer Layer         │  Svelte 4 + TS 5
│  - markdown-it          │
│  - KaTeX 0.16           │
│  - mermaid 10           │
│  - prismjs              │
├-------------------------┤
│  Parser Service         │  pulldown-cmark 0.9
│  - CommonMark + GFM     │
│  - line-map for sync    │
├-------------------------┤
│  File Service           │  tokio::fs + notify 6
│  - atomic writes        │
│  - change debounce      │
└-------------------------┘
```

## 📊 Performance Targets

| Metric | Target | Status |
|--------|--------|---------|
| Cold start | ≤120 ms | 🔄 In Progress |
| Memory usage | ≤80 MB | 🔄 In Progress |
| Installer size | ≤10 MB | 🔄 In Progress |
| First paint | ≤16 ms | 🔄 In Progress |
| Crash-free sessions | ≥99.9% | 🔄 In Progress |

## 🧪 Testing

```bash
# Run Rust tests
cargo test

# Run TypeScript tests
pnpm test

# Run end-to-end tests
pnpm test:e2e

# Security audit
cargo audit && pnpm audit

# Accessibility testing
pnpm test:a11y
```

## 📝 License

MIT License - see [LICENSE](LICENSE) for details.

## 🤝 Contributing

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## 📞 Support

- 📖 [Documentation](docs/)
- 🐛 [Bug Reports](https://github.com/your-org/typolite/issues)
- 💡 [Feature Requests](https://github.com/your-org/typolite/discussions)

---

**Project Codename:** "Typora-Lite"  
**Status:** 🚧 In Development  
**Version:** 0.1.0-alpha

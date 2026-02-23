# Contributing to Copilot Tracker

Thank you for your interest in contributing! This document outlines how to set up your development environment, follow our code standards, and submit your contributions.

## Quick Start

```bash
# Fork and clone the repository
git clone https://github.com/YOUR_USERNAME/copilot-tracker.git
cd copilot-tracker

# Install dependencies
npm install

# Start development mode
npm run dev
```

## Development Setup

### Prerequisites

- Node.js 20+
- Rust (latest stable)
- For macOS: Xcode Command Line Tools
- For Linux: `libwebkit2gtk-4.1-dev`, `build-essential`, etc.
- For Windows: Visual Studio Build Tools

### Available Commands

| Command             | Description                      |
| ------------------- | -------------------------------- |
| `npm run dev`       | Start Tauri dev mode             |
| `npm run dev:web`   | Vite dev server only (port 5173) |
| `npm run build`     | Full production build            |
| `npm run lint`      | Run ALL linters (JS + Rust)      |
| `npm run format`    | Prettier formatting              |
| `npm run typecheck` | TypeScript check                 |

## Code Standards

This project follows strict code standards. **Always run before committing:**

```bash
npm run typecheck && npm run lint
```

### TypeScript/React

- **Imports:** Use absolute imports with `@renderer/` alias
- **Formatting:** Prettier (2-space, single quotes, semicolons, trailing commas)
- **Naming:** PascalCase (components), camelCase (functions/variables), SCREAMING_SNAKE_CASE (constants)
- **Types:** Explicit return types on exported functions
- **Components:** Use function declarations, forward refs with `React.forwardRef`

### Rust

- **Formatting:** `cargo fmt --all -- --check`
- **Linting:** `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- **Error Handling:** Use `?` operator, `Result<T, E>` for fallibles

For detailed standards, see [AGENTS.md](AGENTS.md).

## Pull Request Process

1. **Fork & Clone** - Fork the repo and clone locally
2. **Branch** - Create a feature branch:
   ```bash
   git checkout -b feature/YourAmazingFeature
   # or
   git checkout -b fix/SomeBug
   ```
3. **Code** - Implement your changes
4. **Test** - Run `npm run typecheck && npm run lint` locally
5. **Commit** - Write clear commit messages
6. **Push** - Push to your fork
7. **PR** - Open a Pull Request against `main`

### PR Requirements

Your PR must pass all checks:

- PR title follows Conventional Commits (example: `fix(auth): handle enterprise entitlement`)
- TypeScript type checking
- ESLint (JavaScript/Rust linting)
- Full Tauri build (Windows, macOS, Linux)

### Release Notes Compatibility

This repository uses release-please to generate changelogs from conventional commit metadata.

- Use a Conventional Commit PR title (recommended format: `type(scope): summary`)
- Prefer **Squash and merge** so the PR title is preserved as the release commit message
- If title is non-conventional (for example, `Merge pull request #123`), it may be skipped from generated release notes

Examples:

- `fix(usage): use entitlement endpoint for Copilot Business`
- `feat(widget): add compact usage summary`
- `docs(contributing): clarify release workflow`

## Types of Contributions

We welcome all contributions, including:

- 🐛 Bug fixes
- ✨ New features
- 📖 Documentation improvements
- 🎨 UI/UX enhancements
- 🔧 Build/CI improvements
- 💡 Ideas and suggestions

## Recognizing Contributors

Thank you to everyone who contributes! Contributors are recognized in our [README.md](README.md) using the [All Contributors](https://allcontributors.org) specification.

To add a contributor, comment on any PR or issue:

```
@all-contributors please add @username for doc, code, design
```

Available contribution types: `code`, `doc`, `design`, `ideas`, `review`, `test`, `translation`, `bug`, `security`, `data`, `content`, `maintenance`, `platform`, `plugin`, `tool`, `video`, `question`

## Getting Help

- 🐛 [Report a bug](https://github.com/bizzkoot/copilot-tracker/issues)
- 💡 [Request a feature](https://github.com/bizzkoot/copilot-tracker/issues)
- 💬 Start a discussion

## License

By contributing, you agree that your contributions will be licensed under the MIT License. See the [LICENSE](LICENSE) file for details.

---

_This project is not officially affiliated with GitHub or Microsoft._

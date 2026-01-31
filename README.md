# Copilot Tracker

> A modern, cross-platform GitHub Copilot usage monitoring application

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Platforms](https://img.shields.io/badge/platforms-macOS%20%7C%20Windows%20%7C%20Linux-blue.svg)](https://github.com/bizzkoot/copilot-tracker)

Cross-platform desktop application built with Electron, React, and TypeScript to monitor your GitHub Copilot usage, featuring system tray integration, usage predictions, and smart notifications.

## Features

- 🖥️ **Cross-Platform**: Works on macOS, Windows, and Linux
- 📊 **Usage Tracking**: Real-time monitoring of Copilot premium requests
- 📈 **Trend Visualization**: Beautiful charts showing usage patterns over time
- 🔮 **Smart Predictions**: AI-powered end-of-month usage predictions
- 🌓 **Dark/Light Theme**: Automatic theme detection with manual override
- 🔔 **Smart Notifications**: Configurable alerts when approaching limits
- 🎯 **System Tray**: Quick access from menu bar (macOS) or system tray
- 🔐 **Secure Auth**: WebView-based GitHub OAuth (no API tokens stored)
- 🔄 **Auto-Updates**: Automatic updates via GitHub releases

## Screenshots

_Coming soon_

## Installation

### macOS

```bash
# Download the .dmg from releases
open Copilot-Tracker-1.0.0.dmg
# Drag to Applications folder
```

### Windows

```bash
# Download the .exe installer from releases
Copilot-Tracker-Setup-1.0.0.exe
```

### Linux

```bash
# Download the .AppImage from releases
chmod +x Copilot-Tracker-1.0.0.AppImage
./Copilot-Tracker-1.0.0.AppImage
```

## Development

### Prerequisites

- Node.js 18+ (LTS recommended)
- npm or pnpm
- Git

### Setup

```bash
# Clone the repository
git clone https://github.com/bizzkoot/copilot-tracker.git
cd copilot-tracker

# Install dependencies
npm install

# Start development server
npm run dev
```

### Build

```bash
# Build for current platform
npm run build

# Build for specific platforms
npm run build:mac      # macOS
npm run build:win      # Windows
npm run build:linux    # Linux

# Build for all platforms
npm run build:all
```

## Documentation

Comprehensive planning and implementation documentation is available in the [`docs/`](./docs) folder:

- **[README.md](./docs/README.md)** - Documentation overview
- **[01-spec-copilot-tracker.md](./docs/01-spec-copilot-tracker.md)** - Project specification
- **[02-research-copilot-tracker.md](./docs/02-research-copilot-tracker.md)** - Technical research
- **[03-implementation-copilot-tracker.md](./docs/03-implementation-copilot-tracker.md)** - Implementation guide
- **[copilot-tracker-implementation.md](./docs/copilot-tracker-implementation.md)** - Consolidated plan

## Tech Stack

| Component | Technology |
|-----------|------------|
| Framework | Electron 33+ |
| Frontend | React 18 |
| Language | TypeScript |
| Styling | Tailwind CSS + shadcn/ui |
| State | Zustand |
| Charts | Recharts |
| Build | electron-vite |
| Packaging | electron-builder |

## Project Status

🚧 **Status**: Planning Complete, Implementation In Progress

See [docs/03-implementation-copilot-tracker.md](./docs/03-implementation-copilot-tracker.md) for detailed progress.

## Roadmap

- [x] Complete project planning and research
- [ ] Phase 1: Project setup and scaffolding
- [ ] Phase 2: Core data types and services
- [ ] Phase 3: Authentication system
- [ ] Phase 4: Data fetching and caching
- [ ] Phase 5: Dashboard UI
- [ ] Phase 6: System tray integration
- [ ] Phase 7: Settings and preferences
- [ ] Phase 8: Notifications
- [ ] Phase 9: Packaging and distribution
- [ ] Beta testing
- [ ] v1.0.0 Release

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/AmazingFeature`)
3. Commit your changes (`git commit -m 'Add some AmazingFeature'`)
4. Push to the branch (`git push origin feature/AmazingFeature`)
5. Open a Pull Request

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- Inspired by [copilot-usage-monitor](https://github.com/hyp3rflow/copilot-usage-monitor) by hyp3rflow
- UI components from [shadcn/ui](https://ui.shadcn.com/)
- Built with [Electron](https://www.electronjs.org/)

## Support

- 🐛 [Report a bug](https://github.com/bizzkoot/copilot-tracker/issues)
- 💡 [Request a feature](https://github.com/bizzkoot/copilot-tracker/issues)
- 📖 [Read the docs](./docs)

---

**Note**: This application is not officially affiliated with GitHub or Microsoft. It uses GitHub's internal billing APIs which may change without notice.

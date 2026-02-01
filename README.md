# <img src="resources/icon.png" width="48" height="48" alt="Copilot Tracker Icon" style="vertical-align: middle; margin-right: 8px;" /> Copilot Tracker

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

### Dashboard
<p align="center">
  <img src="assets/Dashboard.gif" alt="Copilot Tracker Dashboard" width="700"/>
</p>

### System Tray (Windows)
<p align="center">
  <img src="assets/Taskbar.gif" alt="System Tray Integration" width="500"/>
</p>

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

## Security Warnings (Unsigned Builds)

⚠️ **This application is distributed without code signing.** You may see security warnings when first running the app. This is expected for open-source projects without paid developer certificates.

### macOS

When you first try to open the app, you may see:
> "App cannot be opened because it was not downloaded from the App Store"

**To bypass:**

**Option 1: Right-click method (GUI)**
1. Right-click (or Control-click) on the app
2. Select "Open"
3. Click "Open" in the confirmation dialog

**Option 2: Terminal method (remove quarantine)**
```bash
# After copying the app to Applications folder
xattr -cr /Applications/copilot-tracker.app
```

**Option 3: System Settings**
1. Open System Settings → Privacy & Security
2. Find the message about the app being blocked
3. Click "Open Anyway"

### Windows

When you first run the installer or app, you may see:
> "Microsoft Defender SmartScreen prevented an unrecognized app from starting"

**To bypass:**
1. Click "More info"
2. Click "Run anyway"

### Linux

No warnings - Linux apps run without restrictions.

### Why Unsigned?

Code signing requires paid certificates:
- **macOS**: Apple Developer Program ($99/year)
- **Windows**: Code signing certificate ($100-300/year)

As an open-source project, we distribute unsigned builds. The source code is publicly available for review if you wish to verify the app's safety.

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
```

## Tech Stack

| Component | Technology               |
| --------- | ------------------------ |
| Framework | Electron 33+             |
| Frontend  | React 18                 |
| Language  | TypeScript               |
| Styling   | Tailwind CSS + shadcn/ui |
| State     | Zustand                  |
| Charts    | Recharts                 |
| Build     | electron-vite            |
| Packaging | electron-builder         |

## Project Status

✅ **Status**: v1.0.0 Completed

## Roadmap

- [x] Complete project planning and research
- [x] Phase 1: Project setup and scaffolding
- [x] Phase 2: Core data types and services
- [x] Phase 3: Authentication system
- [x] Phase 4: Data fetching and caching
- [x] Phase 5: Dashboard UI
- [x] Phase 6: System tray integration
- [x] Phase 7: Settings and preferences
- [x] Phase 8: Notifications
- [x] Phase 9: Packaging and distribution
- [x] v1.0.0 Release

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

---

**Note**: This application is not officially affiliated with GitHub or Microsoft. It uses GitHub's internal billing APIs which may change without notice.

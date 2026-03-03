# Trae Account Manager

<div align="center">

<img src="src-tauri/icons/icon.png" alt="Trae Account Manager" width="128" />

<br />

<p><b>A powerful multi-account management tool for Trae IDE with performance optimizations.</b></p>

![Version](https://img.shields.io/badge/version-0.1.0-green)
![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux-lightgrey)
![License](https://img.shields.io/badge/license-MIT-orange)

</div>

## 📖 Overview

**Trae Account Manager** is a desktop application designed for Trae IDE users to manage multiple accounts efficiently. Switch between accounts seamlessly, monitor usage in real-time, and optimize your workflow with advanced performance features.

This project is based on [Yang-505/Trae-Account-Manager](https://github.com/Yang-505/Trae-Account-Manager) with significant performance and architectural improvements.

## ✨ Key Features

- **One-Click Account Switching**: Automatically manages Trae IDE process and login state
- **Real-Time Usage Monitoring**: Track token consumption and remaining quotas
- **High Performance**: Pagination, virtual scrolling, and smart caching for large account lists
- **Production-Grade Logging**: Environment-aware logging with Pino (pretty-print in dev, JSON in production)
- **WebSocket Communication**: Real-time session updates with automatic reconnection
- **Data Management**: Import/export accounts as JSON for easy backup and sharing

## 🚀 Getting Started

### Prerequisites

- **Windows 10/11**, **macOS**, or **Linux**
- **Trae IDE** installed
- **Node.js 16+** (for development)

### Installation

Download the latest release for your platform from [Releases](https://github.com/911218sky/Trae-Account-Manager/releases).

### Build from Source

```bash
# Clone repository
git clone https://github.com/911218sky/Trae-Account-Manager.git
cd Trae-Account-Manager

# Install dependencies
npm install

# Development mode
npm run tauri dev

# Build production version
npm run tauri build
```

## 💻 Usage

### 1. Configure Trae IDE Path

Open **Settings** → Click **Auto Scan** or **Manual Setup** to locate your `Trae.exe` file.

### 2. Add Account

Click **Add Account** → Enter your Trae IDE token → Click **Add**.

**How to get your token:**
1. Open Trae IDE and press `F12`
2. Go to `Application` → `Local Storage` → `vscode-webview://xxx`
3. Find the key containing `iCubeAuthInfo` and copy the `token` value

### 3. Switch Account

Click **Switch** on any account card → Confirm → The app will automatically restart Trae IDE with the new account.

### 4. Monitor Usage

View real-time usage statistics on the dashboard or click **Details** for detailed usage history.

## 🎯 What's Improved

This fork includes significant enhancements over the original:

### Performance Optimizations
- **Pagination System**: Load accounts in batches (50 per page)
- **Virtual Scrolling**: Render only visible items for smooth performance
- **Smart Caching**: 5-minute cache with automatic invalidation
- **Lazy Loading**: Automatic loading at 80% scroll threshold

### Advanced Logging
- **Pino Integration**: Fast, low-overhead logging
- **Environment-Aware**: Pretty logs in dev, structured JSON in production
- **File Output**: Optional log file support
- **Module Loggers**: Organized logging by component

### Architecture Improvements
- **TypeScript Strict Mode**: Enhanced type safety
- **Service Layer**: Modular services (cache, logger, performance monitor)
- **Custom Hooks**: Reusable React hooks for common patterns
- **Error Boundaries**: Graceful error handling
- **WebSocket Client**: Real-time communication with auto-reconnect

### Developer Experience
- **Test Suite**: Comprehensive tests with Vitest
- **CI/CD**: Automated releases via GitHub Actions
- **Documentation**: Detailed implementation notes
- **AI Guidelines**: Development standards in AGENTS.md

## 🛠️ Tech Stack

- **Frontend**: React 19 + TypeScript + Vite
- **Backend**: Tauri 2 + Rust
- **Logging**: Pino
- **Real-time**: WebSocket
- **Testing**: Vitest

## 📁 Project Structure

```
Trae-Account-Manager/
├── src/                    # Frontend source code
│   ├── components/        # React components
│   ├── services/          # Service layer (API, cache, logger)
│   ├── hooks/             # Custom React hooks
│   └── pages/             # Page components
├── src-tauri/             # Tauri backend (Rust)
└── .github/workflows/     # CI/CD workflows
```

## 📂 Data Storage

Application data is stored locally:

- **Windows**: `%APPDATA%\com.sauce.trae-auto\`
- **macOS**: `~/Library/Application Support/com.sauce.trae-auto/`
- **Linux**: `~/.local/share/com.sauce.trae-auto/`

## 🔗 Integration

Export your accounts as JSON and import them into other tools or share with team members.

### Automated Account Creation

For automated account registration, check out [Trae-Account-Creator](https://github.com/911218sky/Trae-Account-Creator) - a companion tool that automates the Trae IDE account creation process.

## ⚠️ Disclaimer

This project is for educational and research purposes only.

- Use at your own risk
- May violate software terms of service
- Authors are not liable for any damages
- Not for commercial use

## 🤝 Contributing

Contributions are welcome! Feel free to submit issues or pull requests.

```bash
# Fork and clone
git checkout -b feature/AmazingFeature
git commit -m 'Add some AmazingFeature'
git push origin feature/AmazingFeature
# Open a Pull Request
```

## 📄 License

This project is licensed under the MIT License.

## 💖 Credits

- **Original Project**: [Yang-505/Trae-Account-Manager](https://github.com/Yang-505/Trae-Account-Manager) by [@Yang-505](https://github.com/Yang-505)
- **Tauri**: [tauri.app](https://tauri.app/)
- **React**: [react.dev](https://react.dev/)
- **Pino**: [getpino.io](https://getpino.io/)

---

<div align="center">

Made with ❤️ by the community

</div>

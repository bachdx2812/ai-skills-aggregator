# AI Skills Aggregator

Cross-platform desktop application for managing AI coding agent skills across Claude Code, Cursor, Continue.dev, Aider, Windsurf, and more.

Built with **Tauri 2.x** (Rust backend) + **React** + **TypeScript** + **TailwindCSS**.

## Features

- **Local Skills Discovery** - Automatically scans and loads skills from supported AI agents
- **Multi-Agent Support** - Claude Code, Cursor, Continue.dev, Aider, Windsurf
- **Folder-Based Skills** - Each skill is a directory containing multiple files
- **CRUD Operations** - Create, read, update, delete skills and files
- **Remote Registry** - Install skills from community registries
- **Update System** - Check and apply updates with version comparison
- **GitHub OAuth** - Optional authentication for publishing skills (PKCE flow)
- **Cross-Platform** - macOS, Linux, Windows

## Prerequisites

- **Node.js** 20+
- **Rust** 1.77+ with Cargo
- **Platform-specific dependencies:**

### macOS
```bash
xcode-select --install
```

### Linux (Ubuntu/Debian)
```bash
sudo apt update
sudo apt install -y libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf build-essential
```

### Windows
- Install [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/)
- Select "Desktop development with C++"

## Getting Started

### 1. Clone the repository
```bash
git clone git@github.com:bachdx2812/ai-skills-aggregator.git
cd ai-skills-aggregator
```

### 2. Install dependencies
```bash
cd app
npm install
```

### 3. Run in development mode
```bash
npm run tauri:dev
```

This starts both the Vite dev server (frontend) and compiles/runs the Tauri app (backend).

## Project Structure

```
ai-skills-aggregator/
├── app/                          # Main application
│   ├── src/                      # React frontend
│   │   ├── components/           # UI components
│   │   │   ├── layout/           # Layout components (Sidebar, MainPanel, DetailPanel)
│   │   │   └── skills/           # Skill-related components
│   │   ├── stores/               # Zustand state stores
│   │   ├── lib/                  # Utilities, API, types
│   │   └── index.css             # TailwindCSS styles
│   ├── src-tauri/                # Rust backend
│   │   ├── src/
│   │   │   ├── commands/         # Tauri IPC commands
│   │   │   ├── models/           # Data models
│   │   │   └── services/         # Business logic services
│   │   ├── Cargo.toml            # Rust dependencies
│   │   └── tauri.conf.json       # Tauri configuration
│   ├── package.json
│   └── vite.config.ts
├── docs/                         # Documentation
└── plans/                        # Implementation plans
```

## Available Scripts

Run from the `app/` directory:

| Command | Description |
|---------|-------------|
| `npm run dev` | Start Vite dev server only |
| `npm run tauri:dev` | Start full Tauri development |
| `npm run tauri:build` | Build production app |
| `npm run build` | Build frontend only |
| `npm run lint` | Run ESLint |
| `npm run test` | Run Vitest tests |
| `npm run test:ui` | Run tests with UI |

## Development

### Frontend (React + TypeScript)

The frontend uses:
- **React 19** with functional components
- **Zustand** for state management
- **TailwindCSS 4** for styling
- **CodeMirror** for code editing
- **Radix UI** for accessible components

### Backend (Rust + Tauri)

The backend provides:
- **Skill scanning** - Discovers skills from agent config directories
- **File operations** - CRUD with backup support
- **Registry service** - Fetch, install, uninstall remote skills
- **Update service** - Version comparison and updates
- **Auth service** - GitHub OAuth with PKCE flow
- **Keyring service** - Secure token storage

### Adding a New Command

1. Create function in `src-tauri/src/commands/`
2. Register in `src-tauri/src/lib.rs` → `invoke_handler`
3. Add TypeScript wrapper in `src/lib/api.ts`

### Styling

Uses GitHub-inspired dark theme with CSS variables:
- `--bg-surface-*` - Background layers
- `--text-*` - Text colors
- `--border-*` - Border colors

## Building for Production

### Build for current platform
```bash
cd app
npm run tauri:build
```

### Build artifacts location
- **macOS**: `app/src-tauri/target/release/bundle/dmg/`
- **Linux**: `app/src-tauri/target/release/bundle/deb/`
- **Windows**: `app/src-tauri/target/release/bundle/msi/`

## GitHub OAuth Setup (Optional)

For publishing skills to the registry:

1. Go to GitHub Settings → Developer Settings → OAuth Apps
2. Create new OAuth App:
   - **Application name**: AI Skills Aggregator
   - **Homepage URL**: https://aiskills.dev
   - **Callback URL**: `http://127.0.0.1:9876/callback`
3. Copy Client ID
4. Update `app/src-tauri/src/services/auth_service.rs`:
   ```rust
   const GITHUB_CLIENT_ID: &str = "your_client_id";
   ```

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Run tests: `npm run test`
5. Run lint: `npm run lint`
6. Submit a pull request

## License

MIT

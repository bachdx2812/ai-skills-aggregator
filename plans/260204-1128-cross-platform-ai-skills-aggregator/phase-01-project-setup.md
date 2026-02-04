# Phase 01: Project Setup

## Context Links
- [Plan Overview](./plan.md)
- [Framework Research](./research/researcher-01-cross-platform-frameworks.md)

## Overview
**Priority**: P1 | **Status**: pending | **Effort**: 3h

Initialize Tauri 2.x project with React + TypeScript frontend. Configure development tooling, linting, and project structure.

## Key Insights
- Tauri 2.x provides smaller bundles (15-50MB) vs Electron (150-300MB)
- Rust backend handles file I/O; React frontend handles UI
- IPC via Tauri commands pattern

## Requirements

### Functional
- F1: Tauri project scaffolded with React template
- F2: TypeScript strict mode enabled
- F3: TailwindCSS configured
- F4: Hot reload working for frontend
- F5: Basic window renders on all platforms

### Non-Functional
- NF1: Dev server startup <5s
- NF2: Production build <50MB
- NF3: ESLint + Prettier configured

## Architecture

```
ai-skills-aggregator/
├── src/                    # React frontend
│   ├── components/
│   ├── hooks/
│   ├── lib/
│   ├── pages/
│   ├── App.tsx
│   └── main.tsx
├── src-tauri/              # Rust backend
│   ├── src/
│   │   ├── commands/       # IPC command handlers
│   │   ├── services/       # Business logic
│   │   └── main.rs
│   ├── Cargo.toml
│   └── tauri.conf.json
├── package.json
├── tsconfig.json
├── tailwind.config.js
└── vite.config.ts
```

## Related Code Files

### Files to Create
- `src/App.tsx` - Root React component
- `src/main.tsx` - React entry point
- `src-tauri/src/main.rs` - Tauri entry point
- `src-tauri/Cargo.toml` - Rust dependencies
- `src-tauri/tauri.conf.json` - Tauri config
- `package.json` - Node dependencies
- `tsconfig.json` - TypeScript config
- `vite.config.ts` - Vite bundler config
- `tailwind.config.js` - Tailwind config

## Implementation Steps

### 1. Prerequisites Check (10 min)
```bash
# Verify Rust installed
rustc --version  # Requires 1.70+
cargo --version

# Verify Node.js
node --version   # Requires 20+
npm --version

# Install Tauri CLI
cargo install tauri-cli
```

### 2. Scaffold Tauri Project (15 min)
```bash
# Create new Tauri project with React template
npm create tauri-app@latest ai-skills-aggregator -- --template react-ts

cd ai-skills-aggregator
```

### 3. Install Dependencies (10 min)
```bash
# Frontend dependencies
npm install @tanstack/react-query zustand lucide-react clsx

# Dev dependencies
npm install -D @types/node tailwindcss postcss autoprefixer
npm install -D eslint @typescript-eslint/eslint-plugin @typescript-eslint/parser
npm install -D prettier eslint-config-prettier
```

### 4. Configure TailwindCSS (10 min)
```bash
npx tailwindcss init -p
```

Update `tailwind.config.js`:
```javascript
export default {
  content: ["./index.html", "./src/**/*.{js,ts,jsx,tsx}"],
  theme: { extend: {} },
  plugins: [],
}
```

Add to `src/index.css`:
```css
@tailwind base;
@tailwind components;
@tailwind utilities;
```

### 5. Configure TypeScript (10 min)
Update `tsconfig.json`:
```json
{
  "compilerOptions": {
    "target": "ES2020",
    "useDefineForClassFields": true,
    "lib": ["ES2020", "DOM", "DOM.Iterable"],
    "module": "ESNext",
    "skipLibCheck": true,
    "moduleResolution": "bundler",
    "allowImportingTsExtensions": true,
    "resolveJsonModule": true,
    "isolatedModules": true,
    "noEmit": true,
    "jsx": "react-jsx",
    "strict": true,
    "noUnusedLocals": true,
    "noUnusedParameters": true,
    "noFallthroughCasesInSwitch": true,
    "paths": {
      "@/*": ["./src/*"]
    }
  },
  "include": ["src"]
}
```

### 6. Configure Tauri (15 min)
Update `src-tauri/tauri.conf.json`:
```json
{
  "$schema": "https://schema.tauri.app/config/2",
  "productName": "AI Skills Aggregator",
  "version": "0.1.0",
  "identifier": "com.aiskills.aggregator",
  "build": {
    "beforeBuildCommand": "npm run build",
    "beforeDevCommand": "npm run dev",
    "devUrl": "http://localhost:5173",
    "frontendDist": "../dist"
  },
  "app": {
    "windows": [
      {
        "title": "AI Skills Aggregator",
        "width": 1200,
        "height": 800,
        "resizable": true,
        "fullscreen": false
      }
    ],
    "security": {
      "csp": null
    }
  },
  "bundle": {
    "active": true,
    "targets": "all",
    "icon": [
      "icons/32x32.png",
      "icons/128x128.png",
      "icons/icon.icns",
      "icons/icon.ico"
    ]
  }
}
```

### 7. Setup Rust Project Structure (20 min)
Create `src-tauri/src/main.rs`:
```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod services;

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            commands::greet
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

Create `src-tauri/src/commands/mod.rs`:
```rust
#[tauri::command]
pub fn greet(name: &str) -> String {
    format!("Hello, {}! Welcome to AI Skills Aggregator.", name)
}
```

Create `src-tauri/src/services/mod.rs`:
```rust
// Service modules will be added in Phase 02
```

### 8. Verify Setup (15 min)
```bash
# Run in dev mode
npm run tauri dev

# Verify hot reload works
# Verify window renders
# Verify no console errors
```

## Todo List
- [ ] Install Rust toolchain if not present
- [ ] Install Node.js 20+ if not present
- [ ] Scaffold Tauri project
- [ ] Install npm dependencies
- [ ] Configure TailwindCSS
- [ ] Configure TypeScript strict mode
- [ ] Setup Rust project structure
- [ ] Verify dev server runs
- [ ] Test hot reload functionality
- [ ] Create initial commit

## Success Criteria
- [ ] `npm run tauri dev` launches app window
- [ ] React hot reload works on file save
- [ ] Rust backend compiles without errors
- [ ] TypeScript strict mode passes
- [ ] TailwindCSS classes apply correctly
- [ ] ESLint reports no errors

## Risk Assessment

| Risk | Impact | Likelihood | Mitigation |
|------|--------|------------|------------|
| Rust toolchain issues | High | Medium | Use rustup for consistent install |
| Platform SDK missing | High | Low | Document Xcode/MSVC requirements |
| Tauri version mismatch | Medium | Low | Pin versions in Cargo.toml |

## Security Considerations
- CSP configured in tauri.conf.json
- No external URLs loaded initially
- File system permissions scoped in Phase 02

## Next Steps
- Proceed to Phase 02: Core Architecture
- Define IPC command patterns
- Setup state management

# Phase 08: Testing & Deployment

## Context Links
- [Plan Overview](./plan.md)
- [Framework Research](./research/researcher-01-cross-platform-frameworks.md)

## Overview
**Priority**: P1 | **Status**: pending | **Effort**: 3h

Implement testing strategy and configure cross-platform builds for macOS, Linux, and Windows. Setup CI/CD pipeline for automated releases.

## Key Insights
- Tauri provides built-in bundling for all platforms
- Rust tests for backend, Vitest for frontend
- GitHub Actions for CI/CD
- Code signing required for macOS distribution

## Requirements

### Functional
- F1: Unit tests for Rust services
- F2: Unit tests for React components
- F3: Integration tests for IPC
- F4: Cross-platform builds (macOS, Linux, Windows)
- F5: Automated release workflow

### Non-Functional
- NF1: Test coverage >70%
- NF2: Build time <10 min per platform
- NF3: Bundle size <50MB
- NF4: Auto-update capability (optional)

## Architecture

### Testing Pyramid
```
                    ┌─────────────────┐
                    │    E2E Tests    │  (Manual/Playwright)
                    │   (Optional)    │
                    └─────────────────┘
               ┌───────────────────────────┐
               │   Integration Tests       │  (IPC, Services)
               │   Rust + TypeScript       │
               └───────────────────────────┘
          ┌─────────────────────────────────────┐
          │         Unit Tests                  │
          │  Rust (cargo test) + React (vitest) │
          └─────────────────────────────────────┘
```

### CI/CD Pipeline
```
┌─────────────────────────────────────────────────────────────────┐
│                     GitHub Actions Pipeline                     │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌────────────┐   ┌────────────┐   ┌────────────────────────┐  │
│  │ Lint &     │──>│ Unit       │──>│ Build All Platforms    │  │
│  │ Type Check │   │ Tests      │   │ (macOS, Linux, Win)    │  │
│  └────────────┘   └────────────┘   └────────────────────────┘  │
│                                                 │               │
│                                                 ▼               │
│  ┌────────────┐   ┌────────────┐   ┌────────────────────────┐  │
│  │ GitHub     │<──│ Create     │<──│ Upload Artifacts       │  │
│  │ Release    │   │ Release    │   │ (DMG, DEB, MSI)        │  │
│  └────────────┘   └────────────┘   └────────────────────────┘  │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

## Related Code Files

### Files to Create
- `src-tauri/src/tests/mod.rs` - Rust test module
- `src-tauri/src/tests/scanner_tests.rs` - Scanner tests
- `src-tauri/src/tests/crud_tests.rs` - CRUD tests
- `src/__tests__/components/SkillCard.test.tsx` - Component tests
- `src/__tests__/stores/skills-store.test.ts` - Store tests
- `vitest.config.ts` - Vitest configuration
- `.github/workflows/ci.yml` - CI workflow
- `.github/workflows/release.yml` - Release workflow

### Files to Modify
- `package.json` - Add test scripts
- `src-tauri/Cargo.toml` - Add test dependencies

## Implementation Steps

### 1. Setup Vitest for Frontend (20 min)

```bash
npm install -D vitest @vitest/ui @testing-library/react @testing-library/jest-dom
npm install -D jsdom @types/testing-library__jest-dom
```

Create `vitest.config.ts`:
```typescript
import { defineConfig } from 'vitest/config';
import react from '@vitejs/plugin-react';
import path from 'path';

export default defineConfig({
  plugins: [react()],
  test: {
    environment: 'jsdom',
    globals: true,
    setupFiles: ['./src/__tests__/setup.ts'],
    include: ['src/**/*.{test,spec}.{js,ts,jsx,tsx}'],
    coverage: {
      provider: 'v8',
      reporter: ['text', 'json', 'html'],
      exclude: ['node_modules/', 'src/__tests__/'],
    },
  },
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src'),
    },
  },
});
```

Create `src/__tests__/setup.ts`:
```typescript
import '@testing-library/jest-dom';
import { vi } from 'vitest';

// Mock Tauri API
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));
```

Update `package.json`:
```json
{
  "scripts": {
    "test": "vitest",
    "test:ui": "vitest --ui",
    "test:coverage": "vitest --coverage"
  }
}
```

### 2. Write Frontend Unit Tests (45 min)

Create `src/__tests__/components/SkillCard.test.tsx`:
```tsx
import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { SkillCard } from '@/components/skills/SkillCard';

const mockSkill = {
  id: '123',
  name: 'Test Skill',
  description: 'A test skill description',
  agent: 'Claude' as const,
  format: 'Markdown' as const,
  file_path: '/path/to/skill.md',
  content: '# Test',
  tags: ['test', 'example'],
  version: '1.0.0',
  remote_url: null,
  created_at: Date.now(),
  updated_at: Date.now(),
};

describe('SkillCard', () => {
  it('renders skill name and description', () => {
    render(<SkillCard skill={mockSkill} isSelected={false} onClick={() => {}} />);

    expect(screen.getByText('Test Skill')).toBeInTheDocument();
    expect(screen.getByText('A test skill description')).toBeInTheDocument();
  });

  it('shows agent badge', () => {
    render(<SkillCard skill={mockSkill} isSelected={false} onClick={() => {}} />);

    expect(screen.getByText('Claude')).toBeInTheDocument();
  });

  it('renders tags', () => {
    render(<SkillCard skill={mockSkill} isSelected={false} onClick={() => {}} />);

    expect(screen.getByText('test')).toBeInTheDocument();
    expect(screen.getByText('example')).toBeInTheDocument();
  });

  it('calls onClick when clicked', () => {
    const handleClick = vi.fn();
    render(<SkillCard skill={mockSkill} isSelected={false} onClick={handleClick} />);

    fireEvent.click(screen.getByText('Test Skill'));
    expect(handleClick).toHaveBeenCalledTimes(1);
  });

  it('applies selected styles when isSelected is true', () => {
    const { container } = render(
      <SkillCard skill={mockSkill} isSelected={true} onClick={() => {}} />
    );

    const card = container.firstChild;
    expect(card).toHaveClass('border-blue-500');
  });
});
```

Create `src/__tests__/stores/skills-store.test.ts`:
```typescript
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { useSkillsStore } from '@/stores/skills-store';
import { invoke } from '@tauri-apps/api/core';

vi.mock('@tauri-apps/api/core');

describe('SkillsStore', () => {
  beforeEach(() => {
    // Reset store state
    useSkillsStore.setState({
      skills: [],
      isLoading: false,
      error: null,
      selectedSkillId: null,
      searchQuery: '',
      filterAgent: null,
    });
    vi.clearAllMocks();
  });

  it('fetchSkills sets loading state and populates skills', async () => {
    const mockSkills = [
      { id: '1', name: 'Skill 1', agent: 'Claude' },
      { id: '2', name: 'Skill 2', agent: 'Cursor' },
    ];

    vi.mocked(invoke).mockResolvedValueOnce(mockSkills);

    const { fetchSkills } = useSkillsStore.getState();
    await fetchSkills();

    const state = useSkillsStore.getState();
    expect(state.skills).toEqual(mockSkills);
    expect(state.isLoading).toBe(false);
    expect(state.error).toBeNull();
  });

  it('fetchSkills handles errors', async () => {
    vi.mocked(invoke).mockRejectedValueOnce(new Error('Network error'));

    const { fetchSkills } = useSkillsStore.getState();
    await fetchSkills();

    const state = useSkillsStore.getState();
    expect(state.skills).toEqual([]);
    expect(state.isLoading).toBe(false);
    expect(state.error).toBe('Error: Network error');
  });

  it('selectSkill updates selectedSkillId', () => {
    const { selectSkill } = useSkillsStore.getState();
    selectSkill('skill-123');

    const state = useSkillsStore.getState();
    expect(state.selectedSkillId).toBe('skill-123');
  });

  it('setSearchQuery updates searchQuery', () => {
    const { setSearchQuery } = useSkillsStore.getState();
    setSearchQuery('test query');

    const state = useSkillsStore.getState();
    expect(state.searchQuery).toBe('test query');
  });
});
```

### 3. Write Rust Unit Tests (45 min)

Create `src-tauri/src/tests/mod.rs`:
```rust
#[cfg(test)]
pub mod scanner_tests;
#[cfg(test)]
pub mod crud_tests;
#[cfg(test)]
pub mod version_tests;
```

Create `src-tauri/src/tests/scanner_tests.rs`:
```rust
#[cfg(test)]
mod tests {
    use crate::parsers::markdown::MarkdownParser;
    use crate::models::AgentType;

    #[test]
    fn test_markdown_parser_extracts_title() {
        let content = "# My Skill\n\nThis is a description.";
        let parser = MarkdownParser;

        let skill = parser.parse(content, "/test/skill.md", &AgentType::Claude).unwrap();

        assert_eq!(skill.name, "My Skill");
    }

    #[test]
    fn test_markdown_parser_extracts_description() {
        let content = "# Skill\n\nFirst paragraph is the description.\n\nSecond paragraph.";
        let parser = MarkdownParser;

        let skill = parser.parse(content, "/test/skill.md", &AgentType::Claude).unwrap();

        assert!(skill.description.is_some());
        assert!(skill.description.unwrap().contains("First paragraph"));
    }

    #[test]
    fn test_markdown_parser_extracts_tags_from_frontmatter() {
        let content = "---\ntags: [rust, testing]\n---\n\n# Skill\n\nContent here.";
        let parser = MarkdownParser;

        let skill = parser.parse(content, "/test/skill.md", &AgentType::Claude).unwrap();

        assert!(skill.tags.contains(&"rust".to_string()));
        assert!(skill.tags.contains(&"testing".to_string()));
    }

    #[test]
    fn test_markdown_parser_fallback_to_filename() {
        let content = "No heading here, just content.";
        let parser = MarkdownParser;

        let skill = parser.parse(content, "/path/to/my-skill.md", &AgentType::Claude).unwrap();

        assert_eq!(skill.name, "my-skill");
    }
}
```

Create `src-tauri/src/tests/version_tests.rs`:
```rust
#[cfg(test)]
mod tests {
    use std::cmp::Ordering;
    use crate::services::update_service::UpdateService;

    fn compare_versions(current: &str, available: &str) -> Ordering {
        let parse = |v: &str| -> Vec<u32> {
            v.split('.')
                .filter_map(|s| s.parse().ok())
                .collect()
        };

        let curr = parse(current);
        let avail = parse(available);

        for i in 0..3 {
            let c = curr.get(i).copied().unwrap_or(0);
            let a = avail.get(i).copied().unwrap_or(0);
            match c.cmp(&a) {
                Ordering::Equal => continue,
                other => return other,
            }
        }
        Ordering::Equal
    }

    #[test]
    fn test_version_comparison_equal() {
        assert_eq!(compare_versions("1.0.0", "1.0.0"), Ordering::Equal);
        assert_eq!(compare_versions("2.1.3", "2.1.3"), Ordering::Equal);
    }

    #[test]
    fn test_version_comparison_less() {
        assert_eq!(compare_versions("1.0.0", "1.0.1"), Ordering::Less);
        assert_eq!(compare_versions("1.0.0", "1.1.0"), Ordering::Less);
        assert_eq!(compare_versions("1.0.0", "2.0.0"), Ordering::Less);
        assert_eq!(compare_versions("1.9.9", "2.0.0"), Ordering::Less);
    }

    #[test]
    fn test_version_comparison_greater() {
        assert_eq!(compare_versions("1.0.1", "1.0.0"), Ordering::Greater);
        assert_eq!(compare_versions("2.0.0", "1.9.9"), Ordering::Greater);
    }

    #[test]
    fn test_version_comparison_partial() {
        assert_eq!(compare_versions("1.0", "1.0.0"), Ordering::Equal);
        assert_eq!(compare_versions("1", "1.0.0"), Ordering::Equal);
        assert_eq!(compare_versions("1.0", "1.0.1"), Ordering::Less);
    }
}
```

### 4. Create CI Workflow (30 min)

Create `.github/workflows/ci.yml`:
```yaml
name: CI

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  lint:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: '20'
          cache: 'npm'

      - name: Install dependencies
        run: npm ci

      - name: Run ESLint
        run: npm run lint

      - name: Run TypeScript check
        run: npx tsc --noEmit

  test-frontend:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: '20'
          cache: 'npm'

      - name: Install dependencies
        run: npm ci

      - name: Run tests
        run: npm run test -- --coverage

      - name: Upload coverage
        uses: codecov/codecov-action@v3
        with:
          files: ./coverage/coverage-final.json

  test-backend:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Setup Rust
        uses: dtolnay/rust-toolchain@stable

      - name: Cache Cargo
        uses: Swatinem/rust-cache@v2
        with:
          workspaces: src-tauri

      - name: Run Rust tests
        run: cargo test
        working-directory: src-tauri

  build:
    needs: [lint, test-frontend, test-backend]
    strategy:
      fail-fast: false
      matrix:
        platform: [macos-latest, ubuntu-22.04, windows-latest]

    runs-on: ${{ matrix.platform }}
    steps:
      - uses: actions/checkout@v4

      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: '20'
          cache: 'npm'

      - name: Setup Rust
        uses: dtolnay/rust-toolchain@stable

      - name: Install dependencies (Ubuntu)
        if: matrix.platform == 'ubuntu-22.04'
        run: |
          sudo apt-get update
          sudo apt-get install -y libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf

      - name: Install npm dependencies
        run: npm ci

      - name: Build Tauri app
        uses: tauri-apps/tauri-action@v0
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
        with:
          args: --verbose

      - name: Upload artifacts
        uses: actions/upload-artifact@v4
        with:
          name: build-${{ matrix.platform }}
          path: |
            src-tauri/target/release/bundle/dmg/*.dmg
            src-tauri/target/release/bundle/deb/*.deb
            src-tauri/target/release/bundle/msi/*.msi
            src-tauri/target/release/bundle/nsis/*.exe
```

### 5. Create Release Workflow (20 min)

Create `.github/workflows/release.yml`:
```yaml
name: Release

on:
  push:
    tags:
      - 'v*'

jobs:
  release:
    permissions:
      contents: write
    strategy:
      fail-fast: false
      matrix:
        include:
          - platform: macos-latest
            args: '--target universal-apple-darwin'
          - platform: ubuntu-22.04
            args: ''
          - platform: windows-latest
            args: ''

    runs-on: ${{ matrix.platform }}
    steps:
      - uses: actions/checkout@v4

      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: '20'
          cache: 'npm'

      - name: Setup Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.platform == 'macos-latest' && 'aarch64-apple-darwin,x86_64-apple-darwin' || '' }}

      - name: Install dependencies (Ubuntu)
        if: matrix.platform == 'ubuntu-22.04'
        run: |
          sudo apt-get update
          sudo apt-get install -y libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf

      - name: Install npm dependencies
        run: npm ci

      - name: Build and release
        uses: tauri-apps/tauri-action@v0
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
          APPLE_CERTIFICATE: ${{ secrets.APPLE_CERTIFICATE }}
          APPLE_CERTIFICATE_PASSWORD: ${{ secrets.APPLE_CERTIFICATE_PASSWORD }}
          APPLE_SIGNING_IDENTITY: ${{ secrets.APPLE_SIGNING_IDENTITY }}
          APPLE_ID: ${{ secrets.APPLE_ID }}
          APPLE_PASSWORD: ${{ secrets.APPLE_PASSWORD }}
          APPLE_TEAM_ID: ${{ secrets.APPLE_TEAM_ID }}
        with:
          tagName: v__VERSION__
          releaseName: 'AI Skills Aggregator v__VERSION__'
          releaseBody: 'See the changelog for details.'
          releaseDraft: true
          prerelease: false
          args: ${{ matrix.args }}
```

### 6. Configure Tauri Bundling (15 min)

Update `src-tauri/tauri.conf.json`:
```json
{
  "bundle": {
    "active": true,
    "targets": ["dmg", "deb", "msi", "nsis"],
    "identifier": "com.aiskills.aggregator",
    "icon": [
      "icons/32x32.png",
      "icons/128x128.png",
      "icons/128x128@2x.png",
      "icons/icon.icns",
      "icons/icon.ico"
    ],
    "resources": [],
    "copyright": "2026 AI Skills Aggregator",
    "category": "DeveloperTool",
    "shortDescription": "Manage AI coding agent skills",
    "longDescription": "Cross-platform desktop app for aggregating and managing AI coding agent skills across Claude, Cursor, Continue.dev, Aider, and more.",
    "deb": {
      "depends": []
    },
    "macOS": {
      "entitlements": null,
      "exceptionDomain": "",
      "frameworks": [],
      "providerShortName": null,
      "signingIdentity": null
    },
    "windows": {
      "certificateThumbprint": null,
      "digestAlgorithm": "sha256",
      "timestampUrl": ""
    }
  }
}
```

### 7. Add Build Scripts (10 min)

Update `package.json`:
```json
{
  "scripts": {
    "dev": "vite",
    "build": "tsc && vite build",
    "preview": "vite preview",
    "tauri": "tauri",
    "tauri:dev": "tauri dev",
    "tauri:build": "tauri build",
    "test": "vitest",
    "test:ui": "vitest --ui",
    "test:coverage": "vitest --coverage",
    "lint": "eslint . --ext ts,tsx --report-unused-disable-directives --max-warnings 0",
    "lint:fix": "eslint . --ext ts,tsx --fix"
  }
}
```

## Todo List
- [ ] Install Vitest and testing dependencies
- [ ] Create Vitest configuration
- [ ] Write SkillCard component tests
- [ ] Write skills-store tests
- [ ] Write Rust parser tests
- [ ] Write Rust version comparison tests
- [ ] Create CI workflow for linting and testing
- [ ] Create release workflow for builds
- [ ] Configure Tauri bundling for all platforms
- [ ] Add app icons
- [ ] Test builds locally on each platform
- [ ] Setup code signing (macOS - optional)

## Success Criteria
- [ ] All frontend tests pass
- [ ] All Rust tests pass
- [ ] CI pipeline runs successfully
- [ ] Builds generate for macOS, Linux, Windows
- [ ] Bundle size <50MB per platform
- [ ] Release workflow creates GitHub release

## Risk Assessment

| Risk | Impact | Likelihood | Mitigation |
|------|--------|------------|------------|
| macOS code signing issues | Medium | High | Document manual signing |
| Platform-specific build failures | High | Medium | Matrix testing in CI |
| Large bundle size | Medium | Low | Optimize dependencies |
| Flaky tests | Low | Medium | Use stable mocks |

## Security Considerations
- Secrets stored in GitHub secrets
- Code signing for macOS releases
- No credentials in build artifacts
- Dependency audit in CI

## Next Steps
- Run full CI pipeline
- Test builds on target platforms
- Prepare v1.0.0 release
- Write user documentation

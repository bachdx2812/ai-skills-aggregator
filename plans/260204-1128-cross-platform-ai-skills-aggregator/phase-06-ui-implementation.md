# Phase 06: UI Implementation

## Context Links
- [Plan Overview](./plan.md)
- [Phase 05: Skills CRUD Operations](./phase-05-skills-crud-operations.md)

## Overview
**Priority**: P1 | **Status**: pending | **Effort**: 10h

Build React UI components for skills browser, editor, settings. Use TailwindCSS for styling, implement responsive layout, add syntax highlighting for skill editing. **NEW**: Add GitHub login UI, user profile display, and publish button for local skills.

## Key Insights
- Desktop-first design (no mobile needed)
- Three-panel layout: sidebar (agents), main (skills list), detail (editor)
- Syntax highlighting for MD/JSON/YAML/Python
- Search/filter critical for UX

## Requirements

### Functional
- F1: Agent sidebar with skill counts
- F2: Skills list with search and filters
- F3: Skill editor with syntax highlighting
- F4: Create/edit/delete skill dialogs
- F5: Settings page for agent configs
- F6: Remote registry browser
- F7: Toast notifications for actions
- **F8: GitHub login button (optional, app works without login)**
- **F9: User profile avatar + dropdown when logged in**
- **F10: Publish button on local skills (requires login)**
- **F11: "Local" badge on skills created locally**

### Non-Functional
- NF1: Responsive layout (min 800px width)
- NF2: Theme support (light/dark)
- NF3: Keyboard shortcuts (Cmd+S save, Cmd+N new)
- NF4: <100ms UI response time

## Architecture

### Component Tree
```
App
├── Layout
│   ├── Sidebar
│   │   ├── AgentList
│   │   │   └── AgentItem
│   │   ├── QuickActions
│   │   └── UserProfile              # NEW: Avatar + dropdown
│   ├── MainPanel
│   │   ├── Header
│   │   │   ├── SearchBar
│   │   │   ├── FilterBar
│   │   │   └── LoginButton          # NEW: GitHub login (if not logged in)
│   │   ├── SkillsList
│   │   │   └── SkillCard
│   │   │       └── LocalBadge       # NEW: Shows "Local" for is_local skills
│   │   └── EmptyState
│   └── DetailPanel
│       ├── SkillEditor
│       │   ├── EditorHeader
│       │   │   └── PublishButton    # NEW: Publish skill (local only)
│       │   ├── CodeEditor
│       │   └── EditorFooter
│       └── SkillDetails
├── Dialogs
│   ├── CreateSkillDialog            # UPDATED: Add description/tags fields
│   ├── DeleteConfirmDialog
│   ├── ImportDialog
│   ├── SettingsDialog
│   ├── PublishDialog                # NEW: Confirm publish + validation
│   └── LoginPromptDialog            # NEW: Prompt to login for publish
└── Toaster
```

### State Management
```
┌─────────────────────────────────────────────────────────────────┐
│                      Zustand Stores                             │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  skills-store        ui-store           registry-store          │
│  ├─ skills[]         ├─ selectedSkillId ├─ registries[]         │
│  ├─ isLoading        ├─ searchQuery     ├─ installedSkills[]    │
│  ├─ error            ├─ filterAgent     ├─ updates[]            │
│  ├─ fetchSkills()    ├─ sidebarOpen     │                       │
│  ├─ saveSkill()      ├─ theme           │                       │
│  └─ deleteSkill()    └─ activeTab       │                       │
│                                                                 │
│  auth-store (NEW)                                               │
│  ├─ user: User|null                                             │
│  ├─ authState: AuthState                                        │
│  ├─ isLoading                                                   │
│  ├─ login()          # Trigger GitHub OAuth                     │
│  ├─ logout()                                                    │
│  └─ checkAuthStatus()                                           │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

## Related Code Files

### Files to Create
- `src/components/layout/Layout.tsx`
- `src/components/layout/Sidebar.tsx`
- `src/components/layout/MainPanel.tsx`
- `src/components/layout/DetailPanel.tsx`
- `src/components/skills/AgentList.tsx`
- `src/components/skills/AgentItem.tsx`
- `src/components/skills/SkillsList.tsx`
- `src/components/skills/SkillCard.tsx`
- `src/components/skills/SkillEditor.tsx`
- `src/components/skills/SearchBar.tsx`
- `src/components/skills/FilterBar.tsx`
- `src/components/skills/PublishButton.tsx` - **NEW**
- `src/components/skills/LocalBadge.tsx` - **NEW**
- `src/components/auth/LoginButton.tsx` - **NEW**
- `src/components/auth/UserProfile.tsx` - **NEW**
- `src/components/auth/UserDropdown.tsx` - **NEW**
- `src/components/dialogs/CreateSkillDialog.tsx`
- `src/components/dialogs/DeleteConfirmDialog.tsx`
- `src/components/dialogs/ImportDialog.tsx`
- `src/components/dialogs/SettingsDialog.tsx`
- `src/components/dialogs/PublishDialog.tsx` - **NEW**
- `src/components/dialogs/LoginPromptDialog.tsx` - **NEW**
- `src/components/ui/Button.tsx`
- `src/components/ui/Input.tsx`
- `src/components/ui/Dialog.tsx`
- `src/components/ui/Toast.tsx`
- `src/stores/ui-store.ts`
- `src/stores/auth-store.ts` - **NEW**
- `src/pages/HomePage.tsx`
- `src/pages/SettingsPage.tsx`
- `src/pages/RegistryPage.tsx`

### Files to Modify
- `src/App.tsx` - Add routing and layout
- `src/index.css` - Add base styles

## Implementation Steps

### 1. Install UI Dependencies (10 min)

```bash
npm install @radix-ui/react-dialog @radix-ui/react-dropdown-menu
npm install @radix-ui/react-toast @radix-ui/react-tabs
npm install @radix-ui/react-select @radix-ui/react-switch
npm install @uiw/react-codemirror
npm install @codemirror/lang-markdown @codemirror/lang-json
npm install @codemirror/lang-yaml @codemirror/lang-python
npm install lodash.debounce
npm install -D @types/lodash.debounce
```

### 2. Create UI Store (20 min)

Create `src/stores/ui-store.ts`:
```typescript
import { create } from 'zustand';
import { persist } from 'zustand/middleware';

type Theme = 'light' | 'dark' | 'system';
type ActiveTab = 'local' | 'registry' | 'settings';

interface UIState {
  selectedSkillId: string | null;
  searchQuery: string;
  filterAgent: string | null;
  sidebarCollapsed: boolean;
  detailPanelOpen: boolean;
  theme: Theme;
  activeTab: ActiveTab;

  // Actions
  selectSkill: (id: string | null) => void;
  setSearchQuery: (query: string) => void;
  setFilterAgent: (agent: string | null) => void;
  toggleSidebar: () => void;
  toggleDetailPanel: () => void;
  setTheme: (theme: Theme) => void;
  setActiveTab: (tab: ActiveTab) => void;
}

export const useUIStore = create<UIState>()(
  persist(
    (set) => ({
      selectedSkillId: null,
      searchQuery: '',
      filterAgent: null,
      sidebarCollapsed: false,
      detailPanelOpen: true,
      theme: 'system',
      activeTab: 'local',

      selectSkill: (id) => set({ selectedSkillId: id, detailPanelOpen: !!id }),
      setSearchQuery: (query) => set({ searchQuery: query }),
      setFilterAgent: (agent) => set({ filterAgent: agent }),
      toggleSidebar: () => set((s) => ({ sidebarCollapsed: !s.sidebarCollapsed })),
      toggleDetailPanel: () => set((s) => ({ detailPanelOpen: !s.detailPanelOpen })),
      setTheme: (theme) => set({ theme }),
      setActiveTab: (tab) => set({ activeTab: tab }),
    }),
    { name: 'ai-skills-ui' }
  )
);
```

### 3. Create Base UI Components (45 min)

Create `src/components/ui/Button.tsx`:
```tsx
import { forwardRef, ButtonHTMLAttributes } from 'react';
import { clsx } from 'clsx';

interface ButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: 'primary' | 'secondary' | 'ghost' | 'danger';
  size?: 'sm' | 'md' | 'lg';
}

export const Button = forwardRef<HTMLButtonElement, ButtonProps>(
  ({ className, variant = 'primary', size = 'md', children, ...props }, ref) => {
    return (
      <button
        ref={ref}
        className={clsx(
          'inline-flex items-center justify-center rounded-md font-medium transition-colors',
          'focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-offset-2',
          'disabled:pointer-events-none disabled:opacity-50',
          {
            'bg-blue-600 text-white hover:bg-blue-700': variant === 'primary',
            'bg-gray-200 text-gray-900 hover:bg-gray-300 dark:bg-gray-700 dark:text-gray-100':
              variant === 'secondary',
            'hover:bg-gray-100 dark:hover:bg-gray-800': variant === 'ghost',
            'bg-red-600 text-white hover:bg-red-700': variant === 'danger',
          },
          {
            'h-8 px-3 text-sm': size === 'sm',
            'h-10 px-4': size === 'md',
            'h-12 px-6 text-lg': size === 'lg',
          },
          className
        )}
        {...props}
      >
        {children}
      </button>
    );
  }
);
```

Create `src/components/ui/Input.tsx`:
```tsx
import { forwardRef, InputHTMLAttributes } from 'react';
import { clsx } from 'clsx';

interface InputProps extends InputHTMLAttributes<HTMLInputElement> {
  error?: boolean;
}

export const Input = forwardRef<HTMLInputElement, InputProps>(
  ({ className, error, ...props }, ref) => {
    return (
      <input
        ref={ref}
        className={clsx(
          'flex h-10 w-full rounded-md border bg-transparent px-3 py-2 text-sm',
          'placeholder:text-gray-400',
          'focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-blue-500',
          'disabled:cursor-not-allowed disabled:opacity-50',
          error
            ? 'border-red-500 focus-visible:ring-red-500'
            : 'border-gray-300 dark:border-gray-600',
          className
        )}
        {...props}
      />
    );
  }
);
```

### 4. Create Layout Components (60 min)

Create `src/components/layout/Layout.tsx`:
```tsx
import { ReactNode } from 'react';
import { Sidebar } from './Sidebar';
import { MainPanel } from './MainPanel';
import { DetailPanel } from './DetailPanel';
import { useUIStore } from '@/stores/ui-store';
import { clsx } from 'clsx';

interface LayoutProps {
  children?: ReactNode;
}

export function Layout({ children }: LayoutProps) {
  const { sidebarCollapsed, detailPanelOpen } = useUIStore();

  return (
    <div className="flex h-screen bg-gray-50 dark:bg-gray-900">
      {/* Sidebar */}
      <aside
        className={clsx(
          'flex-shrink-0 border-r border-gray-200 dark:border-gray-700 transition-all',
          sidebarCollapsed ? 'w-16' : 'w-64'
        )}
      >
        <Sidebar />
      </aside>

      {/* Main Content */}
      <main className="flex-1 flex overflow-hidden">
        <div className="flex-1 overflow-auto">
          <MainPanel />
        </div>

        {/* Detail Panel */}
        {detailPanelOpen && (
          <div className="w-[500px] border-l border-gray-200 dark:border-gray-700 overflow-hidden">
            <DetailPanel />
          </div>
        )}
      </main>
    </div>
  );
}
```

Create `src/components/layout/Sidebar.tsx`:
```tsx
import { useSkillsStore } from '@/stores/skills-store';
import { useUIStore } from '@/stores/ui-store';
import { AgentList } from '../skills/AgentList';
import { Button } from '../ui/Button';
import { Plus, Settings, FolderSearch } from 'lucide-react';

export function Sidebar() {
  const { sidebarCollapsed, setActiveTab } = useUIStore();
  const { skills } = useSkillsStore();

  const agentCounts = skills.reduce((acc, skill) => {
    const agent = typeof skill.agent === 'string' ? skill.agent : 'Custom';
    acc[agent] = (acc[agent] || 0) + 1;
    return acc;
  }, {} as Record<string, number>);

  return (
    <div className="flex flex-col h-full">
      {/* Header */}
      <div className="p-4 border-b border-gray-200 dark:border-gray-700">
        {!sidebarCollapsed && (
          <h1 className="text-lg font-semibold text-gray-900 dark:text-gray-100">
            AI Skills
          </h1>
        )}
      </div>

      {/* Agent List */}
      <div className="flex-1 overflow-auto p-2">
        <AgentList agentCounts={agentCounts} collapsed={sidebarCollapsed} />
      </div>

      {/* Quick Actions */}
      <div className="p-2 border-t border-gray-200 dark:border-gray-700 space-y-1">
        <Button
          variant="ghost"
          className="w-full justify-start"
          onClick={() => setActiveTab('registry')}
        >
          <FolderSearch className="h-4 w-4 mr-2" />
          {!sidebarCollapsed && 'Browse Registry'}
        </Button>
        <Button
          variant="ghost"
          className="w-full justify-start"
          onClick={() => setActiveTab('settings')}
        >
          <Settings className="h-4 w-4 mr-2" />
          {!sidebarCollapsed && 'Settings'}
        </Button>
      </div>
    </div>
  );
}
```

Create `src/components/layout/MainPanel.tsx`:
```tsx
import { useSkillsStore } from '@/stores/skills-store';
import { useUIStore } from '@/stores/ui-store';
import { SearchBar } from '../skills/SearchBar';
import { FilterBar } from '../skills/FilterBar';
import { SkillsList } from '../skills/SkillsList';
import { Button } from '../ui/Button';
import { Plus, RefreshCw } from 'lucide-react';
import { useState } from 'react';

export function MainPanel() {
  const { skills, isLoading, scanSkills } = useSkillsStore();
  const { searchQuery, filterAgent } = useUIStore();
  const [showCreateDialog, setShowCreateDialog] = useState(false);

  // Filter skills based on search and agent filter
  const filteredSkills = skills.filter((skill) => {
    const matchesSearch =
      !searchQuery ||
      skill.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
      skill.description?.toLowerCase().includes(searchQuery.toLowerCase()) ||
      skill.tags.some((t) => t.toLowerCase().includes(searchQuery.toLowerCase()));

    const matchesAgent =
      !filterAgent ||
      (typeof skill.agent === 'string'
        ? skill.agent === filterAgent
        : skill.agent.Custom === filterAgent);

    return matchesSearch && matchesAgent;
  });

  return (
    <div className="flex flex-col h-full">
      {/* Header */}
      <div className="flex-shrink-0 p-4 border-b border-gray-200 dark:border-gray-700">
        <div className="flex items-center justify-between mb-4">
          <h2 className="text-xl font-semibold text-gray-900 dark:text-gray-100">
            Skills Library
          </h2>
          <div className="flex items-center gap-2">
            <Button
              variant="ghost"
              size="sm"
              onClick={() => scanSkills()}
              disabled={isLoading}
            >
              <RefreshCw className={`h-4 w-4 ${isLoading ? 'animate-spin' : ''}`} />
            </Button>
            <Button size="sm" onClick={() => setShowCreateDialog(true)}>
              <Plus className="h-4 w-4 mr-1" />
              New Skill
            </Button>
          </div>
        </div>
        <div className="flex items-center gap-3">
          <SearchBar />
          <FilterBar />
        </div>
      </div>

      {/* Skills List */}
      <div className="flex-1 overflow-auto p-4">
        <SkillsList skills={filteredSkills} isLoading={isLoading} />
      </div>
    </div>
  );
}
```

### 5. Create Skills Components (90 min)

Create `src/components/skills/SkillsList.tsx`:
```tsx
import { Skill } from '@/lib/types';
import { SkillCard } from './SkillCard';
import { useUIStore } from '@/stores/ui-store';

interface SkillsListProps {
  skills: Skill[];
  isLoading: boolean;
}

export function SkillsList({ skills, isLoading }: SkillsListProps) {
  const { selectedSkillId, selectSkill } = useUIStore();

  if (isLoading) {
    return (
      <div className="grid gap-3">
        {[...Array(6)].map((_, i) => (
          <div
            key={i}
            className="h-24 rounded-lg bg-gray-200 dark:bg-gray-800 animate-pulse"
          />
        ))}
      </div>
    );
  }

  if (skills.length === 0) {
    return (
      <div className="text-center py-12">
        <p className="text-gray-500 dark:text-gray-400">
          No skills found. Create one or import from registry.
        </p>
      </div>
    );
  }

  return (
    <div className="grid gap-3">
      {skills.map((skill) => (
        <SkillCard
          key={skill.id}
          skill={skill}
          isSelected={skill.id === selectedSkillId}
          onClick={() => selectSkill(skill.id)}
        />
      ))}
    </div>
  );
}
```

Create `src/components/skills/SkillCard.tsx`:
```tsx
import { Skill } from '@/lib/types';
import { clsx } from 'clsx';
import { FileText, Code, FileJson, FileCode } from 'lucide-react';

interface SkillCardProps {
  skill: Skill;
  isSelected: boolean;
  onClick: () => void;
}

export function SkillCard({ skill, isSelected, onClick }: SkillCardProps) {
  const agentName = typeof skill.agent === 'string' ? skill.agent : skill.agent.Custom;

  const formatIcon = {
    Markdown: FileText,
    Json: FileJson,
    Yaml: FileCode,
    Python: Code,
    PlainText: FileText,
  }[skill.format] || FileText;

  const Icon = formatIcon;

  return (
    <div
      className={clsx(
        'p-4 rounded-lg border cursor-pointer transition-colors',
        isSelected
          ? 'border-blue-500 bg-blue-50 dark:bg-blue-900/20'
          : 'border-gray-200 dark:border-gray-700 hover:border-gray-300 dark:hover:border-gray-600'
      )}
      onClick={onClick}
    >
      <div className="flex items-start gap-3">
        <div
          className={clsx(
            'p-2 rounded',
            isSelected
              ? 'bg-blue-100 dark:bg-blue-800'
              : 'bg-gray-100 dark:bg-gray-800'
          )}
        >
          <Icon className="h-5 w-5 text-gray-600 dark:text-gray-400" />
        </div>
        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-2">
            <h3 className="font-medium text-gray-900 dark:text-gray-100 truncate">
              {skill.name}
            </h3>
            <span className="text-xs px-2 py-0.5 rounded-full bg-gray-200 dark:bg-gray-700 text-gray-600 dark:text-gray-300">
              {agentName}
            </span>
          </div>
          {skill.description && (
            <p className="text-sm text-gray-500 dark:text-gray-400 mt-1 line-clamp-2">
              {skill.description}
            </p>
          )}
          {skill.tags.length > 0 && (
            <div className="flex gap-1 mt-2">
              {skill.tags.slice(0, 3).map((tag) => (
                <span
                  key={tag}
                  className="text-xs px-1.5 py-0.5 rounded bg-gray-100 dark:bg-gray-800 text-gray-600 dark:text-gray-400"
                >
                  {tag}
                </span>
              ))}
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
```

Create `src/components/skills/SearchBar.tsx`:
```tsx
import { useUIStore } from '@/stores/ui-store';
import { Input } from '../ui/Input';
import { Search, X } from 'lucide-react';

export function SearchBar() {
  const { searchQuery, setSearchQuery } = useUIStore();

  return (
    <div className="relative flex-1">
      <Search className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-gray-400" />
      <Input
        value={searchQuery}
        onChange={(e) => setSearchQuery(e.target.value)}
        placeholder="Search skills..."
        className="pl-10 pr-10"
      />
      {searchQuery && (
        <button
          onClick={() => setSearchQuery('')}
          className="absolute right-3 top-1/2 -translate-y-1/2"
        >
          <X className="h-4 w-4 text-gray-400 hover:text-gray-600" />
        </button>
      )}
    </div>
  );
}
```

### 6. Create Editor Component (60 min)

Create `src/components/skills/SkillEditor.tsx`:
```tsx
import { useEffect } from 'react';
import CodeMirror from '@uiw/react-codemirror';
import { markdown } from '@codemirror/lang-markdown';
import { json } from '@codemirror/lang-json';
import { yaml } from '@codemirror/lang-yaml';
import { python } from '@codemirror/lang-python';
import { useSkillsStore } from '@/stores/skills-store';
import { useUIStore } from '@/stores/ui-store';
import { useSkillEditor } from '@/hooks/use-skill-editor';
import { Button } from '../ui/Button';
import { Save, RotateCcw, Trash2, Copy, MoreVertical } from 'lucide-react';
import type { Skill } from '@/lib/types';

export function SkillEditor() {
  const { selectedSkillId } = useUIStore();
  const { skills, deleteSkill } = useSkillsStore();

  const skill = skills.find((s) => s.id === selectedSkillId) || null;
  const { content, setContent, isLoading, isSaving, isDirty, error, save, revert } =
    useSkillEditor(skill);

  const getLanguageExtension = () => {
    switch (skill?.format) {
      case 'Markdown':
        return [markdown()];
      case 'Json':
        return [json()];
      case 'Yaml':
        return [yaml()];
      case 'Python':
        return [python()];
      default:
        return [];
    }
  };

  // Keyboard shortcuts
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key === 's') {
        e.preventDefault();
        save();
      }
    };
    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [save]);

  if (!skill) {
    return (
      <div className="flex items-center justify-center h-full text-gray-500">
        Select a skill to edit
      </div>
    );
  }

  if (isLoading) {
    return (
      <div className="flex items-center justify-center h-full">
        <div className="animate-spin h-8 w-8 border-2 border-blue-500 border-t-transparent rounded-full" />
      </div>
    );
  }

  return (
    <div className="flex flex-col h-full">
      {/* Header */}
      <div className="flex-shrink-0 p-4 border-b border-gray-200 dark:border-gray-700">
        <div className="flex items-center justify-between">
          <div>
            <h3 className="font-semibold text-gray-900 dark:text-gray-100">
              {skill.name}
            </h3>
            <p className="text-sm text-gray-500">{skill.file_path}</p>
          </div>
          <div className="flex items-center gap-2">
            {isDirty && (
              <span className="text-xs text-amber-600 dark:text-amber-400">
                Unsaved changes
              </span>
            )}
            <Button
              variant="ghost"
              size="sm"
              onClick={revert}
              disabled={!isDirty}
            >
              <RotateCcw className="h-4 w-4" />
            </Button>
            <Button
              variant="primary"
              size="sm"
              onClick={save}
              disabled={!isDirty || isSaving}
            >
              <Save className="h-4 w-4 mr-1" />
              {isSaving ? 'Saving...' : 'Save'}
            </Button>
          </div>
        </div>
      </div>

      {/* Error */}
      {error && (
        <div className="px-4 py-2 bg-red-50 dark:bg-red-900/20 text-red-600 dark:text-red-400 text-sm">
          {error}
        </div>
      )}

      {/* Editor */}
      <div className="flex-1 overflow-hidden">
        <CodeMirror
          value={content}
          onChange={setContent}
          extensions={getLanguageExtension()}
          theme="dark"
          height="100%"
          className="h-full text-sm"
        />
      </div>

      {/* Footer */}
      <div className="flex-shrink-0 p-3 border-t border-gray-200 dark:border-gray-700 flex justify-between items-center">
        <div className="text-xs text-gray-500">
          Format: {skill.format} | {content.length} chars
        </div>
        <div className="flex items-center gap-2">
          <Button
            variant="ghost"
            size="sm"
            onClick={() => navigator.clipboard.writeText(content)}
          >
            <Copy className="h-4 w-4" />
          </Button>
          <Button
            variant="danger"
            size="sm"
            onClick={() => {
              if (confirm('Delete this skill?')) {
                deleteSkill(skill.file_path);
              }
            }}
          >
            <Trash2 className="h-4 w-4" />
          </Button>
        </div>
      </div>
    </div>
  );
}
```

### 7. Create Detail Panel (20 min)

Create `src/components/layout/DetailPanel.tsx`:
```tsx
import { SkillEditor } from '../skills/SkillEditor';
import { useUIStore } from '@/stores/ui-store';
import { Button } from '../ui/Button';
import { X } from 'lucide-react';

export function DetailPanel() {
  const { toggleDetailPanel } = useUIStore();

  return (
    <div className="flex flex-col h-full bg-white dark:bg-gray-900">
      <div className="flex items-center justify-between p-2 border-b border-gray-200 dark:border-gray-700">
        <span className="text-sm font-medium text-gray-600 dark:text-gray-400">
          Editor
        </span>
        <Button variant="ghost" size="sm" onClick={toggleDetailPanel}>
          <X className="h-4 w-4" />
        </Button>
      </div>
      <div className="flex-1 overflow-hidden">
        <SkillEditor />
      </div>
    </div>
  );
}
```

### 8. Create Auth Store (NEW - 30 min)

Create `src/stores/auth-store.ts`:
```typescript
import { create } from 'zustand';
import { persist } from 'zustand/middleware';
import type { User, AuthState } from '@/lib/types';
import { api } from '@/lib/api';

interface AuthStore {
  user: User | null;
  authState: AuthState;
  isLoading: boolean;

  // Actions
  login: () => Promise<void>;
  logout: () => Promise<void>;
  checkAuthStatus: () => Promise<void>;
}

export const useAuthStore = create<AuthStore>()(
  persist(
    (set, get) => ({
      user: null,
      authState: { type: 'logged_out' },
      isLoading: false,

      login: async () => {
        set({ authState: { type: 'logging_in' }, isLoading: true });
        try {
          // Triggers OAuth flow via Tauri command
          const user = await api.auth.login();
          set({
            user,
            authState: { type: 'logged_in', user },
            isLoading: false,
          });
        } catch (error) {
          set({
            authState: { type: 'error', message: String(error) },
            isLoading: false,
          });
        }
      },

      logout: async () => {
        set({ isLoading: true });
        try {
          await api.auth.logout();
          set({
            user: null,
            authState: { type: 'logged_out' },
            isLoading: false,
          });
        } catch (error) {
          set({ isLoading: false });
        }
      },

      checkAuthStatus: async () => {
        try {
          const user = await api.auth.getCurrentUser();
          if (user) {
            set({ user, authState: { type: 'logged_in', user } });
          } else {
            set({ user: null, authState: { type: 'logged_out' } });
          }
        } catch {
          set({ user: null, authState: { type: 'logged_out' } });
        }
      },
    }),
    { name: 'ai-skills-auth' }
  )
);
```

### 9. Create Auth Components (NEW - 45 min)

Create `src/components/auth/LoginButton.tsx`:
```tsx
import { useAuthStore } from '@/stores/auth-store';
import { Button } from '../ui/Button';
import { Github, Loader2 } from 'lucide-react';

export function LoginButton() {
  const { authState, isLoading, login } = useAuthStore();

  if (authState.type === 'logged_in') {
    return null; // Show UserProfile instead
  }

  return (
    <Button
      variant="secondary"
      size="sm"
      onClick={login}
      disabled={isLoading}
      className="gap-2"
    >
      {isLoading ? (
        <Loader2 className="h-4 w-4 animate-spin" />
      ) : (
        <Github className="h-4 w-4" />
      )}
      Sign in with GitHub
    </Button>
  );
}
```

Create `src/components/auth/UserProfile.tsx`:
```tsx
import { useAuthStore } from '@/stores/auth-store';
import { UserDropdown } from './UserDropdown';

export function UserProfile() {
  const { user, authState } = useAuthStore();

  if (authState.type !== 'logged_in' || !user) {
    return null;
  }

  return (
    <div className="flex items-center gap-2 p-2">
      <UserDropdown user={user} />
    </div>
  );
}
```

Create `src/components/auth/UserDropdown.tsx`:
```tsx
import * as DropdownMenu from '@radix-ui/react-dropdown-menu';
import { useAuthStore } from '@/stores/auth-store';
import type { User } from '@/lib/types';
import { LogOut, User as UserIcon, ChevronDown } from 'lucide-react';

interface UserDropdownProps {
  user: User;
}

export function UserDropdown({ user }: UserDropdownProps) {
  const { logout } = useAuthStore();

  return (
    <DropdownMenu.Root>
      <DropdownMenu.Trigger className="flex items-center gap-2 p-2 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-800 transition-colors">
        {user.avatar_url ? (
          <img
            src={user.avatar_url}
            alt={user.username}
            className="h-8 w-8 rounded-full"
          />
        ) : (
          <div className="h-8 w-8 rounded-full bg-gray-300 dark:bg-gray-600 flex items-center justify-center">
            <UserIcon className="h-4 w-4" />
          </div>
        )}
        <span className="text-sm font-medium text-gray-700 dark:text-gray-300">
          {user.display_name || user.username}
        </span>
        <ChevronDown className="h-4 w-4 text-gray-400" />
      </DropdownMenu.Trigger>

      <DropdownMenu.Portal>
        <DropdownMenu.Content
          className="min-w-[200px] bg-white dark:bg-gray-800 rounded-lg shadow-lg border border-gray-200 dark:border-gray-700 p-1"
          sideOffset={5}
        >
          <div className="px-3 py-2 border-b border-gray-200 dark:border-gray-700">
            <p className="text-sm font-medium">{user.display_name || user.username}</p>
            <p className="text-xs text-gray-500">@{user.username}</p>
          </div>

          <DropdownMenu.Item
            className="flex items-center gap-2 px-3 py-2 text-sm text-red-600 dark:text-red-400 cursor-pointer hover:bg-red-50 dark:hover:bg-red-900/20 rounded"
            onClick={logout}
          >
            <LogOut className="h-4 w-4" />
            Sign out
          </DropdownMenu.Item>
        </DropdownMenu.Content>
      </DropdownMenu.Portal>
    </DropdownMenu.Root>
  );
}
```

### 10. Create Publish Components (NEW - 45 min)

Create `src/components/skills/PublishButton.tsx`:
```tsx
import { useState } from 'react';
import { useAuthStore } from '@/stores/auth-store';
import { Button } from '../ui/Button';
import { PublishDialog } from '../dialogs/PublishDialog';
import { LoginPromptDialog } from '../dialogs/LoginPromptDialog';
import { Upload } from 'lucide-react';
import type { Skill } from '@/lib/types';

interface PublishButtonProps {
  skill: Skill;
}

export function PublishButton({ skill }: PublishButtonProps) {
  const { user, authState } = useAuthStore();
  const [showPublishDialog, setShowPublishDialog] = useState(false);
  const [showLoginPrompt, setShowLoginPrompt] = useState(false);

  // Only show for local skills
  if (!skill.is_local) {
    return null;
  }

  const handleClick = () => {
    if (authState.type === 'logged_in' && user) {
      setShowPublishDialog(true);
    } else {
      setShowLoginPrompt(true);
    }
  };

  return (
    <>
      <Button variant="secondary" size="sm" onClick={handleClick}>
        <Upload className="h-4 w-4 mr-1" />
        Publish
      </Button>

      <PublishDialog
        skill={skill}
        open={showPublishDialog}
        onClose={() => setShowPublishDialog(false)}
      />

      <LoginPromptDialog
        open={showLoginPrompt}
        onClose={() => setShowLoginPrompt(false)}
      />
    </>
  );
}
```

Create `src/components/skills/LocalBadge.tsx`:
```tsx
import { clsx } from 'clsx';

interface LocalBadgeProps {
  className?: string;
}

export function LocalBadge({ className }: LocalBadgeProps) {
  return (
    <span
      className={clsx(
        'text-xs px-1.5 py-0.5 rounded-full',
        'bg-green-100 dark:bg-green-900/30 text-green-700 dark:text-green-400',
        className
      )}
    >
      Local
    </span>
  );
}
```

Create `src/components/dialogs/PublishDialog.tsx`:
```tsx
import { useState, useEffect } from 'react';
import * as Dialog from '@radix-ui/react-dialog';
import { Button } from '../ui/Button';
import { Input } from '../ui/Input';
import { api } from '@/lib/api';
import type { Skill } from '@/lib/types';
import { X, AlertCircle, Check } from 'lucide-react';

interface PublishDialogProps {
  skill: Skill;
  open: boolean;
  onClose: () => void;
}

export function PublishDialog({ skill, open, onClose }: PublishDialogProps) {
  const [isValidating, setIsValidating] = useState(false);
  const [isPublishing, setIsPublishing] = useState(false);
  const [errors, setErrors] = useState<string[]>([]);
  const [success, setSuccess] = useState(false);

  useEffect(() => {
    if (open) {
      validateSkill();
    }
  }, [open, skill]);

  const validateSkill = async () => {
    setIsValidating(true);
    try {
      const validationErrors = await api.publish.validate(skill);
      setErrors(validationErrors);
    } catch (e) {
      setErrors([String(e)]);
    } finally {
      setIsValidating(false);
    }
  };

  const handlePublish = async () => {
    if (errors.length > 0) return;

    setIsPublishing(true);
    try {
      await api.publish.publish(skill, 'https://registry.aiskills.dev');
      setSuccess(true);
    } catch (e) {
      setErrors([String(e)]);
    } finally {
      setIsPublishing(false);
    }
  };

  return (
    <Dialog.Root open={open} onOpenChange={onClose}>
      <Dialog.Portal>
        <Dialog.Overlay className="fixed inset-0 bg-black/50" />
        <Dialog.Content className="fixed top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-full max-w-md bg-white dark:bg-gray-900 rounded-lg shadow-xl p-6">
          <Dialog.Title className="text-lg font-semibold mb-4">
            Publish Skill
          </Dialog.Title>

          {success ? (
            <div className="text-center py-4">
              <Check className="h-12 w-12 text-green-500 mx-auto mb-3" />
              <p className="text-lg font-medium">Skill Published!</p>
              <p className="text-gray-500 text-sm mt-2">
                Your skill is now available in the community registry.
              </p>
              <Button onClick={onClose} className="mt-4">
                Done
              </Button>
            </div>
          ) : (
            <>
              <div className="space-y-3 mb-4">
                <p className="text-sm text-gray-600 dark:text-gray-400">
                  Publishing "{skill.name}" to the community registry.
                </p>

                {isValidating ? (
                  <p className="text-sm">Validating...</p>
                ) : errors.length > 0 ? (
                  <div className="bg-red-50 dark:bg-red-900/20 rounded-lg p-3">
                    <div className="flex items-center gap-2 text-red-600 dark:text-red-400 mb-2">
                      <AlertCircle className="h-4 w-4" />
                      <span className="font-medium">Please fix these issues:</span>
                    </div>
                    <ul className="list-disc list-inside text-sm text-red-600 dark:text-red-400">
                      {errors.map((error, i) => (
                        <li key={i}>{error}</li>
                      ))}
                    </ul>
                  </div>
                ) : (
                  <div className="bg-green-50 dark:bg-green-900/20 rounded-lg p-3 flex items-center gap-2 text-green-600 dark:text-green-400">
                    <Check className="h-4 w-4" />
                    <span>Ready to publish</span>
                  </div>
                )}
              </div>

              <div className="flex justify-end gap-2">
                <Button variant="ghost" onClick={onClose}>
                  Cancel
                </Button>
                <Button
                  onClick={handlePublish}
                  disabled={errors.length > 0 || isPublishing}
                >
                  {isPublishing ? 'Publishing...' : 'Publish'}
                </Button>
              </div>
            </>
          )}

          <Dialog.Close asChild>
            <button className="absolute top-4 right-4 text-gray-400 hover:text-gray-600">
              <X className="h-4 w-4" />
            </button>
          </Dialog.Close>
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  );
}
```

Create `src/components/dialogs/LoginPromptDialog.tsx`:
```tsx
import * as Dialog from '@radix-ui/react-dialog';
import { Button } from '../ui/Button';
import { useAuthStore } from '@/stores/auth-store';
import { Github, X } from 'lucide-react';

interface LoginPromptDialogProps {
  open: boolean;
  onClose: () => void;
}

export function LoginPromptDialog({ open, onClose }: LoginPromptDialogProps) {
  const { login, isLoading } = useAuthStore();

  const handleLogin = async () => {
    await login();
    onClose();
  };

  return (
    <Dialog.Root open={open} onOpenChange={onClose}>
      <Dialog.Portal>
        <Dialog.Overlay className="fixed inset-0 bg-black/50" />
        <Dialog.Content className="fixed top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-full max-w-sm bg-white dark:bg-gray-900 rounded-lg shadow-xl p-6 text-center">
          <Github className="h-12 w-12 mx-auto mb-4 text-gray-700 dark:text-gray-300" />

          <Dialog.Title className="text-lg font-semibold mb-2">
            Sign in to Publish
          </Dialog.Title>

          <p className="text-sm text-gray-600 dark:text-gray-400 mb-6">
            Connect your GitHub account to publish skills to the community registry.
          </p>

          <div className="flex flex-col gap-2">
            <Button onClick={handleLogin} disabled={isLoading} className="w-full">
              <Github className="h-4 w-4 mr-2" />
              {isLoading ? 'Connecting...' : 'Sign in with GitHub'}
            </Button>
            <Button variant="ghost" onClick={onClose} className="w-full">
              Maybe later
            </Button>
          </div>

          <Dialog.Close asChild>
            <button className="absolute top-4 right-4 text-gray-400 hover:text-gray-600">
              <X className="h-4 w-4" />
            </button>
          </Dialog.Close>
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  );
}
```

### 11. Update SkillCard for Local Badge (NEW - 10 min)

Update `src/components/skills/SkillCard.tsx` to show LocalBadge:
```tsx
// Add import
import { LocalBadge } from './LocalBadge';

// In the component, after agent badge:
{skill.is_local && <LocalBadge />}
```

### 12. Update Sidebar for UserProfile (NEW - 10 min)

Update `src/components/layout/Sidebar.tsx`:
```tsx
// Add import
import { UserProfile } from '../auth/UserProfile';
import { LoginButton } from '../auth/LoginButton';
import { useAuthStore } from '@/stores/auth-store';

// At bottom of sidebar, before QuickActions:
<div className="p-2 border-t border-gray-200 dark:border-gray-700">
  {authState.type === 'logged_in' ? (
    <UserProfile />
  ) : (
    <LoginButton />
  )}
</div>
```

### 13. Update SkillEditor for PublishButton (NEW - 10 min)

In `src/components/skills/SkillEditor.tsx`, add PublishButton in header:
```tsx
// Add import
import { PublishButton } from './PublishButton';

// In header actions, add:
{skill.is_local && <PublishButton skill={skill} />}
```

### 14. Create App Entry (30 min)

Update `src/App.tsx`:
```tsx
import { useEffect } from 'react';
import { Layout } from './components/layout/Layout';
import { useSkillsStore } from './stores/skills-store';
import { useUIStore } from './stores/ui-store';
import { Toaster } from './components/ui/Toast';

function App() {
  const { scanSkills } = useSkillsStore();
  const { theme } = useUIStore();

  // Initial scan on mount
  useEffect(() => {
    scanSkills();
  }, []);

  // Apply theme
  useEffect(() => {
    const root = document.documentElement;
    if (theme === 'dark' || (theme === 'system' && window.matchMedia('(prefers-color-scheme: dark)').matches)) {
      root.classList.add('dark');
    } else {
      root.classList.remove('dark');
    }
  }, [theme]);

  return (
    <>
      <Layout />
      <Toaster />
    </>
  );
}

export default App;
```

Update `src/index.css`:
```css
@tailwind base;
@tailwind components;
@tailwind utilities;

:root {
  font-family: Inter, system-ui, Avenir, Helvetica, Arial, sans-serif;
}

body {
  margin: 0;
  min-height: 100vh;
}

.dark {
  color-scheme: dark;
}

/* Custom scrollbar */
::-webkit-scrollbar {
  width: 8px;
  height: 8px;
}

::-webkit-scrollbar-track {
  @apply bg-gray-100 dark:bg-gray-800;
}

::-webkit-scrollbar-thumb {
  @apply bg-gray-300 dark:bg-gray-600 rounded;
}

::-webkit-scrollbar-thumb:hover {
  @apply bg-gray-400 dark:bg-gray-500;
}
```

## Todo List
- [ ] Install UI dependencies (Radix, CodeMirror)
- [ ] Create UI store with persisted state
- [ ] Create auth store with persisted state (NEW)
- [ ] Create base UI components (Button, Input, Dialog)
- [ ] Create Layout component with three panels
- [ ] Create Sidebar with agent list
- [ ] Create MainPanel with skills list
- [ ] Create SkillCard component (with LocalBadge)
- [ ] Create SearchBar and FilterBar
- [ ] Create SkillEditor with CodeMirror (with PublishButton)
- [ ] Create DetailPanel wrapper
- [ ] Create dialog components
- [ ] Create auth components (LoginButton, UserProfile, UserDropdown) - NEW
- [ ] Create publish components (PublishButton, PublishDialog, LoginPromptDialog) - NEW
- [ ] Update App.tsx with layout and auth check
- [ ] Add dark mode support
- [ ] Add keyboard shortcuts
- [ ] Test responsive behavior
- [ ] Test publish flow UI

## Success Criteria
- [ ] Three-panel layout renders correctly
- [ ] Agent list shows counts
- [ ] Skills list displays with search/filter
- [ ] Editor loads with syntax highlighting
- [ ] Save with Cmd+S works
- [ ] Dark mode toggles correctly
- [ ] Auto-save debounced at 3s
- [ ] Responsive at 800px minimum
- [ ] **Login button shows when logged out**
- [ ] **User avatar/dropdown shows when logged in**
- [ ] **Local skills show "Local" badge**
- [ ] **Publish button shows only for local skills**
- [ ] **Login prompt appears when clicking publish while logged out**
- [ ] **Publish dialog validates skill before submission**

## Risk Assessment

| Risk | Impact | Likelihood | Mitigation |
|------|--------|------------|------------|
| CodeMirror bundle size | Medium | Medium | Lazy load editor |
| State sync issues | High | Low | Single source of truth |
| Theme flash on load | Low | Medium | Use prefers-color-scheme |

## Security Considerations
- Sanitize displayed skill content
- No dangerouslySetInnerHTML
- Validate content before rendering

## Next Steps
- Proceed to Phase 07: Update System
- Implement skill version checking
- Add update notifications

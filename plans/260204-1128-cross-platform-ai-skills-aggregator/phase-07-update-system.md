# Phase 07: Update System

## Context Links
- [Plan Overview](./plan.md)
- [Phase 04: Remote Skills Registry](./phase-04-remote-skills-registry.md)

## Overview
**Priority**: P2 | **Status**: pending | **Effort**: 4h

Implement auto-update mechanism for remote skills. Check for updates on app launch, notify users, provide one-click updates.

## Key Insights
- Compare installed version vs registry version
- Semantic versioning for comparison
- Background check on app start
- User controls update timing
- Preserve local modifications option

## Requirements

### Functional
- F1: Check for updates on app launch
- F2: Show update notification badge
- F3: List all available updates
- F4: One-click update single skill
- F5: Bulk update all skills
- F6: Rollback to previous version
- F7: Skip specific versions

### Non-Functional
- NF1: Background check <10s
- NF2: Update download with progress
- NF3: No UI blocking during check
- NF4: Offline graceful degradation

## Architecture

### Update Flow
```
┌─────────────────────────────────────────────────────────────────┐
│                      Update Check Flow                          │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────────┐  │
│  │ App Launch   │───>│ Fetch All    │───>│ Compare Versions │  │
│  │ (Background) │    │ Registries   │    │ (Semver)         │  │
│  └──────────────┘    └──────────────┘    └──────────────────┘  │
│                                                 │               │
│                                                 ▼               │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────────┐  │
│  │ Apply        │<───│ Show Update  │<───│ Build Updates    │  │
│  │ Updates      │    │ UI Badge     │    │ List             │  │
│  └──────────────┘    └──────────────┘    └──────────────────┘  │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### Version Comparison
```rust
// Semantic versioning: MAJOR.MINOR.PATCH
// 1.0.0 < 1.0.1 < 1.1.0 < 2.0.0

pub fn compare_versions(current: &str, available: &str) -> Ordering {
    let parse = |v: &str| -> Vec<u32> {
        v.split('.').filter_map(|s| s.parse().ok()).collect()
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
```

## Related Code Files

### Files to Create
- `src-tauri/src/services/update_service.rs` - Update checking logic
- `src-tauri/src/commands/updates.rs` - Update IPC commands
- `src/stores/updates-store.ts` - Updates state
- `src/components/updates/UpdatesBadge.tsx` - Notification badge
- `src/components/updates/UpdatesList.tsx` - Updates list UI
- `src/components/updates/UpdateDialog.tsx` - Update confirmation

### Files to Modify
- `src-tauri/src/services/mod.rs` - Export update service
- `src-tauri/src/main.rs` - Register update commands
- `src/lib/api.ts` - Add updates API
- `src/components/layout/Sidebar.tsx` - Add badge

## Implementation Steps

### 1. Create Update Service (60 min)

Create `src-tauri/src/services/update_service.rs`:
```rust
use std::cmp::Ordering;
use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::models::{InstalledSkill, SkillRegistry, AppError};
use crate::services::registry_service::RegistryService;

pub struct UpdateService {
    registry: RegistryService,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillUpdate {
    pub skill_id: String,
    pub skill_name: String,
    pub current_version: String,
    pub new_version: String,
    pub agent: String,
    pub registry_url: String,
    pub changelog: Option<String>,
    pub is_major: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateCheckResult {
    pub available_updates: Vec<SkillUpdate>,
    pub last_checked: i64,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkippedVersion {
    pub skill_id: String,
    pub version: String,
    pub skipped_at: i64,
}

impl UpdateService {
    pub fn new() -> Self {
        Self {
            registry: RegistryService::new(),
        }
    }

    pub async fn check_all_updates(&self) -> UpdateCheckResult {
        let installed = match self.registry.get_installed_skills().await {
            Ok(skills) => skills,
            Err(e) => {
                return UpdateCheckResult {
                    available_updates: vec![],
                    last_checked: Utc::now().timestamp(),
                    error: Some(e.to_string()),
                };
            }
        };

        // Group by registry URL
        let mut by_registry: std::collections::HashMap<String, Vec<&InstalledSkill>> =
            std::collections::HashMap::new();

        for skill in &installed {
            by_registry
                .entry(skill.registry_url.clone())
                .or_default()
                .push(skill);
        }

        let mut updates = Vec::new();

        for (registry_url, skills) in by_registry {
            let config = crate::models::RegistryConfig {
                url: registry_url.clone(),
                name: "".into(),
                enabled: true,
                auth_token: None,
            };

            if let Ok(registry) = self.registry.fetch_registry(&config).await {
                for installed in skills {
                    if let Some(remote) = registry.skills.iter().find(|s| s.id == installed.skill_id) {
                        if self.is_newer(&installed.version, &remote.version) {
                            updates.push(SkillUpdate {
                                skill_id: installed.skill_id.clone(),
                                skill_name: remote.name.clone(),
                                current_version: installed.version.clone(),
                                new_version: remote.version.clone(),
                                agent: installed.agent.clone(),
                                registry_url: registry_url.clone(),
                                changelog: None,
                                is_major: self.is_major_update(&installed.version, &remote.version),
                            });
                        }
                    }
                }
            }
        }

        UpdateCheckResult {
            available_updates: updates,
            last_checked: Utc::now().timestamp(),
            error: None,
        }
    }

    pub async fn apply_update(&self, update: &SkillUpdate) -> Result<(), AppError> {
        // Fetch the skill from registry
        let config = crate::models::RegistryConfig {
            url: update.registry_url.clone(),
            name: "".into(),
            enabled: true,
            auth_token: None,
        };

        let registry = self.registry.fetch_registry(&config).await?;

        let remote_skill = registry
            .skills
            .iter()
            .find(|s| s.id == update.skill_id)
            .ok_or_else(|| AppError::FileNotFound(format!("Skill {} not found", update.skill_id)))?;

        // Install (will backup and replace)
        self.registry
            .install_skill(remote_skill, &update.registry_url, &update.agent)
            .await?;

        Ok(())
    }

    pub async fn apply_all_updates(&self, updates: &[SkillUpdate]) -> Vec<Result<(), String>> {
        let mut results = Vec::new();

        for update in updates {
            let result = self.apply_update(update).await.map_err(|e| e.to_string());
            results.push(result);
        }

        results
    }

    pub async fn rollback_skill(&self, skill_id: &str, agent: &str) -> Result<(), AppError> {
        let backup = crate::services::backup_service::BackupService::new();
        let installed = self.registry.get_installed_skills().await?;

        let skill = installed
            .iter()
            .find(|s| s.skill_id == skill_id && s.agent == agent)
            .ok_or_else(|| AppError::FileNotFound(format!("Skill {} not installed", skill_id)))?;

        // Get most recent backup
        let filename = std::path::Path::new(&skill.installed_path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");

        let backups = backup.list_backups(filename).await?;

        if let Some(most_recent) = backups.first() {
            backup.restore_file(&most_recent.path, &skill.installed_path).await?;
            Ok(())
        } else {
            Err(AppError::FileNotFound("No backup available".into()))
        }
    }

    // Version comparison helpers

    fn is_newer(&self, current: &str, available: &str) -> bool {
        self.compare_versions(current, available) == Ordering::Less
    }

    fn is_major_update(&self, current: &str, available: &str) -> bool {
        let curr_major = current.split('.').next().and_then(|s| s.parse::<u32>().ok());
        let avail_major = available.split('.').next().and_then(|s| s.parse::<u32>().ok());

        match (curr_major, avail_major) {
            (Some(c), Some(a)) => a > c,
            _ => false,
        }
    }

    fn compare_versions(&self, current: &str, available: &str) -> Ordering {
        let parse = |v: &str| -> Vec<u32> {
            v.split('.')
                .filter_map(|s| s.chars().take_while(|c| c.is_numeric()).collect::<String>().parse().ok())
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
}

// Background update checker
pub async fn start_background_update_check() {
    tokio::spawn(async {
        loop {
            let service = UpdateService::new();
            let result = service.check_all_updates().await;

            // Emit event to frontend if updates available
            if !result.available_updates.is_empty() {
                // This would use Tauri's event system
                // app_handle.emit_all("updates-available", result);
            }

            // Check every hour
            tokio::time::sleep(tokio::time::Duration::from_secs(3600)).await;
        }
    });
}
```

### 2. Create Update Commands (30 min)

Create `src-tauri/src/commands/updates.rs`:
```rust
use crate::services::update_service::{UpdateService, SkillUpdate, UpdateCheckResult};

#[tauri::command]
pub async fn check_for_updates() -> Result<UpdateCheckResult, String> {
    let service = UpdateService::new();
    Ok(service.check_all_updates().await)
}

#[tauri::command]
pub async fn apply_skill_update(update: SkillUpdate) -> Result<(), String> {
    let service = UpdateService::new();
    service.apply_update(&update).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn apply_all_skill_updates(updates: Vec<SkillUpdate>) -> Result<Vec<Result<(), String>>, String> {
    let service = UpdateService::new();
    Ok(service.apply_all_updates(&updates).await)
}

#[tauri::command]
pub async fn rollback_skill(skill_id: String, agent: String) -> Result<(), String> {
    let service = UpdateService::new();
    service.rollback_skill(&skill_id, &agent).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn skip_skill_version(skill_id: String, version: String) -> Result<(), String> {
    // Store skipped version in local config
    let config_path = dirs::data_local_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("ai-skills-aggregator")
        .join("skipped-versions.json");

    let mut skipped: Vec<crate::services::update_service::SkippedVersion> =
        if config_path.exists() {
            let content = tokio::fs::read_to_string(&config_path)
                .await
                .map_err(|e| e.to_string())?;
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            vec![]
        };

    skipped.push(crate::services::update_service::SkippedVersion {
        skill_id,
        version,
        skipped_at: chrono::Utc::now().timestamp(),
    });

    let content = serde_json::to_string_pretty(&skipped)
        .map_err(|e| e.to_string())?;

    if let Some(parent) = config_path.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(|e| e.to_string())?;
    }

    tokio::fs::write(&config_path, content)
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}
```

### 3. Create Updates Store (20 min)

Create `src/stores/updates-store.ts`:
```typescript
import { create } from 'zustand';
import { api } from '@/lib/api';

export interface SkillUpdate {
  skill_id: string;
  skill_name: string;
  current_version: string;
  new_version: string;
  agent: string;
  registry_url: string;
  changelog: string | null;
  is_major: boolean;
}

interface UpdatesState {
  updates: SkillUpdate[];
  isChecking: boolean;
  lastChecked: number | null;
  error: string | null;

  checkForUpdates: () => Promise<void>;
  applyUpdate: (update: SkillUpdate) => Promise<void>;
  applyAllUpdates: () => Promise<void>;
  skipVersion: (skillId: string, version: string) => Promise<void>;
  rollback: (skillId: string, agent: string) => Promise<void>;
}

export const useUpdatesStore = create<UpdatesState>((set, get) => ({
  updates: [],
  isChecking: false,
  lastChecked: null,
  error: null,

  checkForUpdates: async () => {
    set({ isChecking: true, error: null });
    try {
      const result = await api.updates.check();
      set({
        updates: result.available_updates,
        lastChecked: result.last_checked,
        isChecking: false,
        error: result.error,
      });
    } catch (error) {
      set({ error: String(error), isChecking: false });
    }
  },

  applyUpdate: async (update) => {
    try {
      await api.updates.apply(update);
      set((state) => ({
        updates: state.updates.filter((u) => u.skill_id !== update.skill_id),
      }));
    } catch (error) {
      set({ error: String(error) });
    }
  },

  applyAllUpdates: async () => {
    const { updates } = get();
    try {
      await api.updates.applyAll(updates);
      set({ updates: [] });
    } catch (error) {
      set({ error: String(error) });
    }
  },

  skipVersion: async (skillId, version) => {
    try {
      await api.updates.skip(skillId, version);
      set((state) => ({
        updates: state.updates.filter((u) => u.skill_id !== skillId),
      }));
    } catch (error) {
      set({ error: String(error) });
    }
  },

  rollback: async (skillId, agent) => {
    try {
      await api.updates.rollback(skillId, agent);
    } catch (error) {
      set({ error: String(error) });
    }
  },
}));
```

### 4. Update API (15 min)

Update `src/lib/api.ts`:
```typescript
import type { SkillUpdate } from '@/stores/updates-store';

interface UpdateCheckResult {
  available_updates: SkillUpdate[];
  last_checked: number;
  error: string | null;
}

export const api = {
  // ... existing

  updates: {
    check: () => invoke<UpdateCheckResult>('check_for_updates'),
    apply: (update: SkillUpdate) => invoke<void>('apply_skill_update', { update }),
    applyAll: (updates: SkillUpdate[]) =>
      invoke<Array<{ Ok?: null; Err?: string }>>('apply_all_skill_updates', { updates }),
    skip: (skillId: string, version: string) =>
      invoke<void>('skip_skill_version', { skillId, version }),
    rollback: (skillId: string, agent: string) =>
      invoke<void>('rollback_skill', { skillId, agent }),
  },
};
```

### 5. Create Updates Badge (20 min)

Create `src/components/updates/UpdatesBadge.tsx`:
```tsx
import { useUpdatesStore } from '@/stores/updates-store';
import { RefreshCw } from 'lucide-react';

export function UpdatesBadge() {
  const { updates, isChecking, checkForUpdates } = useUpdatesStore();

  const count = updates.length;

  return (
    <button
      onClick={() => checkForUpdates()}
      disabled={isChecking}
      className="relative flex items-center gap-2 px-3 py-2 rounded-md hover:bg-gray-100 dark:hover:bg-gray-800"
    >
      <RefreshCw className={`h-4 w-4 ${isChecking ? 'animate-spin' : ''}`} />
      <span className="text-sm">Updates</span>
      {count > 0 && (
        <span className="absolute -top-1 -right-1 flex h-5 w-5 items-center justify-center rounded-full bg-blue-600 text-xs text-white">
          {count}
        </span>
      )}
    </button>
  );
}
```

### 6. Create Updates List (30 min)

Create `src/components/updates/UpdatesList.tsx`:
```tsx
import { useUpdatesStore, SkillUpdate } from '@/stores/updates-store';
import { Button } from '../ui/Button';
import { Download, SkipForward, AlertTriangle } from 'lucide-react';

export function UpdatesList() {
  const { updates, applyUpdate, applyAllUpdates, skipVersion } = useUpdatesStore();

  if (updates.length === 0) {
    return (
      <div className="text-center py-8 text-gray-500">
        All skills are up to date!
      </div>
    );
  }

  return (
    <div className="space-y-4">
      {/* Bulk actions */}
      <div className="flex justify-between items-center">
        <span className="text-sm text-gray-600 dark:text-gray-400">
          {updates.length} update{updates.length > 1 ? 's' : ''} available
        </span>
        <Button onClick={() => applyAllUpdates()}>
          <Download className="h-4 w-4 mr-1" />
          Update All
        </Button>
      </div>

      {/* Updates list */}
      <div className="space-y-2">
        {updates.map((update) => (
          <UpdateItem
            key={`${update.skill_id}-${update.agent}`}
            update={update}
            onApply={() => applyUpdate(update)}
            onSkip={() => skipVersion(update.skill_id, update.new_version)}
          />
        ))}
      </div>
    </div>
  );
}

interface UpdateItemProps {
  update: SkillUpdate;
  onApply: () => void;
  onSkip: () => void;
}

function UpdateItem({ update, onApply, onSkip }: UpdateItemProps) {
  return (
    <div className="flex items-center justify-between p-4 rounded-lg border border-gray-200 dark:border-gray-700">
      <div className="flex-1">
        <div className="flex items-center gap-2">
          <span className="font-medium">{update.skill_name}</span>
          <span className="text-xs px-2 py-0.5 rounded bg-gray-200 dark:bg-gray-700">
            {update.agent}
          </span>
          {update.is_major && (
            <span className="flex items-center gap-1 text-xs text-amber-600">
              <AlertTriangle className="h-3 w-3" />
              Major
            </span>
          )}
        </div>
        <div className="text-sm text-gray-500 mt-1">
          {update.current_version} → {update.new_version}
        </div>
      </div>
      <div className="flex items-center gap-2">
        <Button variant="ghost" size="sm" onClick={onSkip}>
          <SkipForward className="h-4 w-4" />
        </Button>
        <Button size="sm" onClick={onApply}>
          <Download className="h-4 w-4 mr-1" />
          Update
        </Button>
      </div>
    </div>
  );
}
```

### 7. Add Auto-Check on Launch (15 min)

Update `src/App.tsx`:
```tsx
import { useEffect } from 'react';
import { Layout } from './components/layout/Layout';
import { useSkillsStore } from './stores/skills-store';
import { useUpdatesStore } from './stores/updates-store';
import { useUIStore } from './stores/ui-store';
import { Toaster } from './components/ui/Toast';

function App() {
  const { scanSkills } = useSkillsStore();
  const { checkForUpdates } = useUpdatesStore();
  const { theme } = useUIStore();

  // Initial scan and update check on mount
  useEffect(() => {
    scanSkills();

    // Check for updates after a short delay
    const timeout = setTimeout(() => {
      checkForUpdates();
    }, 2000);

    return () => clearTimeout(timeout);
  }, []);

  // Apply theme
  useEffect(() => {
    const root = document.documentElement;
    if (
      theme === 'dark' ||
      (theme === 'system' && window.matchMedia('(prefers-color-scheme: dark)').matches)
    ) {
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

## Todo List
- [ ] Create update service with version comparison
- [ ] Create update IPC commands
- [ ] Create updates Zustand store
- [ ] Update API wrappers
- [ ] Create UpdatesBadge component
- [ ] Create UpdatesList component
- [ ] Add auto-check on app launch
- [ ] Implement skip version functionality
- [ ] Implement rollback functionality
- [ ] Test version comparison logic
- [ ] Handle offline scenarios
- [ ] Add update notifications

## Success Criteria
- [ ] Version comparison works correctly (semver)
- [ ] Updates checked on launch (2s delay)
- [ ] Badge shows correct count
- [ ] Single update applies correctly
- [ ] Bulk update works
- [ ] Skip version persists
- [ ] Rollback restores from backup
- [ ] Offline mode shows cached data

## Risk Assessment

| Risk | Impact | Likelihood | Mitigation |
|------|--------|------------|------------|
| Update breaks skill | High | Low | Backup before update |
| Network timeout | Medium | Medium | Show error, allow retry |
| Version parse failure | Medium | Low | Default to string compare |
| Concurrent updates | Low | Low | Queue updates sequentially |

## Security Considerations
- Verify registry URLs before fetch
- No automatic updates without consent
- Major updates require explicit confirmation
- Backups before any modification

## Next Steps
- Proceed to Phase 08: Testing & Deployment
- Write unit tests
- Configure cross-platform builds

# Phase 02: Core Architecture

## Context Links
- [Plan Overview](./plan.md)
- [Phase 01: Project Setup](./phase-01-project-setup.md)
- [AI Agent Skills Research](./research/researcher-02-ai-agent-skills-sources.md)

## Overview
**Priority**: P1 | **Status**: pending | **Effort**: 6h

Define core data models, IPC patterns, Rust services layer, and frontend state management. Foundation for all subsequent phases.

## Key Insights
- Skills stored in 4 formats: MD, JSON, YAML, plain text
- Directory hierarchies vary: `~/.claude/skills/`, `~/.continue/`, etc.
- Need unified internal model to normalize different formats
- IPC should be async to avoid blocking UI

## Requirements

### Functional
- F1: Unified Skill data model handles all agent formats
- F2: Agent configuration defines storage paths/patterns
- F3: IPC commands for all backend operations
- F4: Frontend state management via Zustand
- F5: Error handling propagates to UI

### Non-Functional
- NF1: Skill parsing <100ms per file
- NF2: State updates trigger minimal re-renders
- NF3: Type-safe IPC between Rust and TypeScript

## Architecture

### Data Flow
```
┌─────────────────────────────────────────────────────────────────┐
│                         FRONTEND (React)                        │
│  ┌──────────────┐  ┌──────────────┐  ┌───────────────────────┐ │
│  │  Components  │──│    Hooks     │──│  Zustand Store        │ │
│  └──────────────┘  └──────────────┘  └───────────────────────┘ │
│                            │                                    │
│                            ▼ invoke()                           │
└────────────────────────────┼────────────────────────────────────┘
                             │ IPC (Tauri Commands)
┌────────────────────────────┼────────────────────────────────────┐
│                         BACKEND (Rust)                          │
│                            ▼                                    │
│  ┌──────────────┐  ┌──────────────┐  ┌───────────────────────┐ │
│  │   Commands   │──│   Services   │──│   File System         │ │
│  └──────────────┘  └──────────────┘  └───────────────────────┘ │
│         │                                                       │
│         ▼                                                       │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │                    Data Models (Serde)                    │  │
│  └──────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
```

### Core Data Models

```rust
// src-tauri/src/models/skill.rs

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    pub id: String,                    // UUID
    pub name: String,
    pub description: Option<String>,
    pub agent: AgentType,
    pub format: SkillFormat,
    pub file_path: String,
    pub content: String,
    pub tags: Vec<String>,
    pub version: Option<String>,
    pub remote_url: Option<String>,    // Source URL if installed from remote
    pub author: Option<String>,        // GitHub username if published
    pub is_local: bool,                // True if created locally (not from registry)
    pub created_at: i64,
    pub updated_at: i64,
}

// src-tauri/src/models/user.rs

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct User {
    pub id: String,                    // GitHub user ID
    pub username: String,              // GitHub username
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
    pub access_token: String,          // Stored in secure keychain
    pub logged_in_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum AuthState {
    #[default]
    LoggedOut,
    LoggingIn,
    LoggedIn(User),
    Error(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AgentType {
    Claude,
    Cursor,
    ContinueDev,
    Aider,
    Windsurf,
    Copilot,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SkillFormat {
    Markdown,
    Json,
    Yaml,
    PlainText,
    Python,  // Claude executable skills
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub agent_type: AgentType,
    pub name: String,
    pub base_path: String,           // e.g., "~/.claude"
    pub skills_patterns: Vec<String>, // e.g., ["skills/**/*.md", "rules/*.md"]
    pub config_file: Option<String>,  // e.g., "CLAUDE.md"
    pub enabled: bool,
}
```

### Agent Configurations

```rust
// src-tauri/src/config/agents.rs

pub fn default_agent_configs() -> Vec<AgentConfig> {
    vec![
        AgentConfig {
            agent_type: AgentType::Claude,
            name: "Claude Code".into(),
            base_path: "~/.claude".into(),
            skills_patterns: vec![
                "skills/**/*.md".into(),
                "rules/*.md".into(),
                "CLAUDE.md".into(),
            ],
            config_file: Some("CLAUDE.md".into()),
            enabled: true,
        },
        AgentConfig {
            agent_type: AgentType::Cursor,
            name: "Cursor AI".into(),
            base_path: "~/.cursor".into(),
            skills_patterns: vec![".cursorrules".into()],
            config_file: None,
            enabled: true,
        },
        AgentConfig {
            agent_type: AgentType::ContinueDev,
            name: "Continue.dev".into(),
            base_path: "~/.continue".into(),
            skills_patterns: vec![
                "config.json".into(),
                "profiles/**/*.json".into(),
            ],
            config_file: Some("config.json".into()),
            enabled: true,
        },
        AgentConfig {
            agent_type: AgentType::Aider,
            name: "Aider".into(),
            base_path: "~".into(),
            skills_patterns: vec![
                ".aider.conf.yml".into(),
                ".aider/**/*.txt".into(),
            ],
            config_file: Some(".aider.conf.yml".into()),
            enabled: true,
        },
        AgentConfig {
            agent_type: AgentType::Windsurf,
            name: "Windsurf/Codeium".into(),
            base_path: "~/.codeium".into(),
            skills_patterns: vec!["**/*.json".into(), "**/*.yaml".into()],
            config_file: None,
            enabled: true,
        },
    ]
}
```

## Related Code Files

### Files to Create (Rust Backend)
- `src-tauri/src/models/mod.rs` - Model exports
- `src-tauri/src/models/skill.rs` - Skill data model
- `src-tauri/src/models/agent.rs` - Agent config model
- `src-tauri/src/models/error.rs` - Error types
- `src-tauri/src/models/user.rs` - User and auth state models
- `src-tauri/src/config/mod.rs` - Config exports
- `src-tauri/src/config/agents.rs` - Default agent configs
- `src-tauri/src/services/skill_service.rs` - Skill business logic
- `src-tauri/src/services/file_service.rs` - File I/O operations
- `src-tauri/src/services/auth_service.rs` - Authentication service (Phase 09)
- `src-tauri/src/commands/skills.rs` - IPC skill commands
- `src-tauri/src/commands/agents.rs` - IPC agent commands
- `src-tauri/src/commands/auth.rs` - IPC auth commands (Phase 09)

### Files to Create (TypeScript Frontend)
- `src/lib/types.ts` - TypeScript type definitions
- `src/lib/api.ts` - Tauri invoke wrappers
- `src/stores/skills-store.ts` - Zustand skills state
- `src/stores/agents-store.ts` - Zustand agents state
- `src/stores/ui-store.ts` - UI state (loading, errors)
- `src/stores/auth-store.ts` - Authentication state (Phase 09)
- `src/hooks/use-skills.ts` - Skills data hook
- `src/hooks/use-agents.ts` - Agents data hook
- `src/hooks/use-auth.ts` - Authentication hook (Phase 09)

## Implementation Steps

### 1. Create Rust Data Models (45 min)

Create `src-tauri/src/models/mod.rs`:
```rust
pub mod skill;
pub mod agent;
pub mod error;

pub use skill::*;
pub use agent::*;
pub use error::*;
```

Create `src-tauri/src/models/error.rs`:
```rust
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error, Serialize, Deserialize)]
pub enum AppError {
    #[error("File not found: {0}")]
    FileNotFound(String),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("IO error: {0}")]
    IoError(String),

    #[error("Invalid path: {0}")]
    InvalidPath(String),
}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        AppError::IoError(err.to_string())
    }
}
```

### 2. Create Agent Configuration (30 min)

Create `src-tauri/src/config/mod.rs`:
```rust
pub mod agents;
pub use agents::*;
```

### 3. Create File Service (45 min)

Create `src-tauri/src/services/file_service.rs`:
```rust
use std::path::PathBuf;
use glob::glob;
use crate::models::{AppError, SkillFormat};

pub fn expand_home_path(path: &str) -> PathBuf {
    if path.starts_with("~") {
        if let Some(home) = dirs::home_dir() {
            return home.join(&path[2..]);
        }
    }
    PathBuf::from(path)
}

pub fn read_file_content(path: &PathBuf) -> Result<String, AppError> {
    std::fs::read_to_string(path)
        .map_err(|e| AppError::IoError(e.to_string()))
}

pub fn write_file_content(path: &PathBuf, content: &str) -> Result<(), AppError> {
    std::fs::write(path, content)
        .map_err(|e| AppError::IoError(e.to_string()))
}

pub fn detect_format(path: &PathBuf) -> SkillFormat {
    match path.extension().and_then(|e| e.to_str()) {
        Some("md") => SkillFormat::Markdown,
        Some("json") => SkillFormat::Json,
        Some("yaml") | Some("yml") => SkillFormat::Yaml,
        Some("py") => SkillFormat::Python,
        _ => SkillFormat::PlainText,
    }
}

pub fn find_files(base_path: &str, patterns: &[String]) -> Vec<PathBuf> {
    let base = expand_home_path(base_path);
    let mut files = Vec::new();

    for pattern in patterns {
        let full_pattern = base.join(pattern);
        if let Some(pattern_str) = full_pattern.to_str() {
            if let Ok(paths) = glob(pattern_str) {
                for path in paths.flatten() {
                    if path.is_file() {
                        files.push(path);
                    }
                }
            }
        }
    }
    files
}
```

### 4. Create Skill Service (60 min)

Create `src-tauri/src/services/skill_service.rs`:
```rust
use uuid::Uuid;
use chrono::Utc;
use crate::models::{Skill, AgentConfig, AgentType, AppError};
use crate::services::file_service;

pub fn scan_agent_skills(config: &AgentConfig) -> Result<Vec<Skill>, AppError> {
    let files = file_service::find_files(&config.base_path, &config.skills_patterns);
    let mut skills = Vec::new();

    for file_path in files {
        if let Ok(skill) = parse_skill_file(&file_path, &config.agent_type) {
            skills.push(skill);
        }
    }

    Ok(skills)
}

fn parse_skill_file(
    path: &std::path::PathBuf,
    agent_type: &AgentType,
) -> Result<Skill, AppError> {
    let content = file_service::read_file_content(path)?;
    let format = file_service::detect_format(path);
    let name = extract_skill_name(&content, path);
    let description = extract_description(&content);
    let tags = extract_tags(&content);

    Ok(Skill {
        id: Uuid::new_v4().to_string(),
        name,
        description,
        agent: agent_type.clone(),
        format,
        file_path: path.to_string_lossy().to_string(),
        content,
        tags,
        version: None,
        remote_url: None,
        created_at: Utc::now().timestamp(),
        updated_at: Utc::now().timestamp(),
    })
}

fn extract_skill_name(content: &str, path: &std::path::PathBuf) -> String {
    // Try to extract from markdown heading
    if let Some(line) = content.lines().find(|l| l.starts_with("# ")) {
        return line.trim_start_matches("# ").to_string();
    }
    // Fall back to filename
    path.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Unknown")
        .to_string()
}

fn extract_description(content: &str) -> Option<String> {
    // Get first paragraph after heading
    content.lines()
        .skip_while(|l| l.starts_with("#") || l.is_empty())
        .take_while(|l| !l.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
        .into()
}

fn extract_tags(content: &str) -> Vec<String> {
    // Look for tags: [...] pattern or #tags
    let mut tags = Vec::new();
    for line in content.lines() {
        if line.to_lowercase().starts_with("tags:") {
            let tag_str = line.split(':').nth(1).unwrap_or("");
            tags.extend(
                tag_str.split(',')
                    .map(|t| t.trim().trim_matches(|c| c == '[' || c == ']').to_string())
                    .filter(|t| !t.is_empty())
            );
        }
    }
    tags
}
```

### 5. Create IPC Commands (45 min)

Create `src-tauri/src/commands/skills.rs`:
```rust
use crate::config::default_agent_configs;
use crate::models::{Skill, AppError};
use crate::services::skill_service;

#[tauri::command]
pub async fn get_all_skills() -> Result<Vec<Skill>, String> {
    let configs = default_agent_configs();
    let mut all_skills = Vec::new();

    for config in configs.iter().filter(|c| c.enabled) {
        match skill_service::scan_agent_skills(config) {
            Ok(skills) => all_skills.extend(skills),
            Err(e) => eprintln!("Error scanning {}: {}", config.name, e),
        }
    }

    Ok(all_skills)
}

#[tauri::command]
pub async fn get_skill_by_id(id: String) -> Result<Option<Skill>, String> {
    let skills = get_all_skills().await?;
    Ok(skills.into_iter().find(|s| s.id == id))
}

#[tauri::command]
pub async fn save_skill(skill: Skill) -> Result<(), String> {
    let path = std::path::PathBuf::from(&skill.file_path);
    crate::services::file_service::write_file_content(&path, &skill.content)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_skill(file_path: String) -> Result<(), String> {
    std::fs::remove_file(&file_path)
        .map_err(|e| format!("Failed to delete: {}", e))
}
```

### 6. Create TypeScript Types (30 min)

Create `src/lib/types.ts`:
```typescript
export type AgentType =
  | 'Claude'
  | 'Cursor'
  | 'ContinueDev'
  | 'Aider'
  | 'Windsurf'
  | 'Copilot'
  | { Custom: string };

export type SkillFormat =
  | 'Markdown'
  | 'Json'
  | 'Yaml'
  | 'PlainText'
  | 'Python';

export interface Skill {
  id: string;
  name: string;
  description: string | null;
  agent: AgentType;
  format: SkillFormat;
  file_path: string;
  content: string;
  tags: string[];
  version: string | null;
  remote_url: string | null;
  author: string | null;        // GitHub username if published
  is_local: boolean;            // True if created locally
  created_at: number;
  updated_at: number;
}

export interface AgentConfig {
  agent_type: AgentType;
  name: string;
  base_path: string;
  skills_patterns: string[];
  config_file: string | null;
  enabled: boolean;
}

// Auth types
export interface User {
  id: string;
  username: string;
  display_name: string | null;
  avatar_url: string | null;
  logged_in_at: number;
}

export type AuthState =
  | { type: 'logged_out' }
  | { type: 'logging_in' }
  | { type: 'logged_in'; user: User }
  | { type: 'error'; message: string };
```

### 7. Create Tauri API Wrappers (30 min)

Create `src/lib/api.ts`:
```typescript
import { invoke } from '@tauri-apps/api/core';
import type { Skill, AgentConfig } from './types';

export const api = {
  skills: {
    getAll: () => invoke<Skill[]>('get_all_skills'),
    getById: (id: string) => invoke<Skill | null>('get_skill_by_id', { id }),
    save: (skill: Skill) => invoke<void>('save_skill', { skill }),
    delete: (filePath: string) => invoke<void>('delete_skill', { filePath }),
  },
  agents: {
    getConfigs: () => invoke<AgentConfig[]>('get_agent_configs'),
    updateConfig: (config: AgentConfig) => invoke<void>('update_agent_config', { config }),
  },
};
```

### 8. Create Zustand Stores (45 min)

Create `src/stores/skills-store.ts`:
```typescript
import { create } from 'zustand';
import type { Skill } from '@/lib/types';
import { api } from '@/lib/api';

interface SkillsState {
  skills: Skill[];
  isLoading: boolean;
  error: string | null;
  selectedSkillId: string | null;
  searchQuery: string;
  filterAgent: string | null;

  // Actions
  fetchSkills: () => Promise<void>;
  selectSkill: (id: string | null) => void;
  setSearchQuery: (query: string) => void;
  setFilterAgent: (agent: string | null) => void;
  saveSkill: (skill: Skill) => Promise<void>;
  deleteSkill: (filePath: string) => Promise<void>;
}

export const useSkillsStore = create<SkillsState>((set, get) => ({
  skills: [],
  isLoading: false,
  error: null,
  selectedSkillId: null,
  searchQuery: '',
  filterAgent: null,

  fetchSkills: async () => {
    set({ isLoading: true, error: null });
    try {
      const skills = await api.skills.getAll();
      set({ skills, isLoading: false });
    } catch (error) {
      set({ error: String(error), isLoading: false });
    }
  },

  selectSkill: (id) => set({ selectedSkillId: id }),

  setSearchQuery: (query) => set({ searchQuery: query }),

  setFilterAgent: (agent) => set({ filterAgent: agent }),

  saveSkill: async (skill) => {
    await api.skills.save(skill);
    await get().fetchSkills();
  },

  deleteSkill: async (filePath) => {
    await api.skills.delete(filePath);
    set({ selectedSkillId: null });
    await get().fetchSkills();
  },
}));
```

### 9. Update main.rs with Commands (15 min)

Update `src-tauri/src/main.rs`:
```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod config;
mod models;
mod services;

use commands::{skills, agents};

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            skills::get_all_skills,
            skills::get_skill_by_id,
            skills::save_skill,
            skills::delete_skill,
            agents::get_agent_configs,
            agents::update_agent_config,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

### 10. Add Rust Dependencies (10 min)

Update `src-tauri/Cargo.toml`:
```toml
[dependencies]
tauri = { version = "2", features = [] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
serde_yaml = "0.9"
thiserror = "1"
uuid = { version = "1", features = ["v4"] }
chrono = { version = "0.4", features = ["serde"] }
glob = "0.3"
dirs = "5"
tokio = { version = "1", features = ["full"] }
```

## Todo List
- [ ] Create Rust data models (Skill, AgentConfig, Error)
- [ ] Create User and AuthState models
- [ ] Create agent default configurations
- [ ] Implement file service (read, write, glob)
- [ ] Implement skill service (scan, parse)
- [ ] Create IPC commands for skills
- [ ] Create IPC commands for agents
- [ ] Create TypeScript type definitions (incl. auth types)
- [ ] Create Tauri API wrappers
- [ ] Create Zustand stores (skills, agents, auth)
- [ ] Wire up commands in main.rs
- [ ] Add Rust dependencies to Cargo.toml
- [ ] Test IPC communication

## Success Criteria
- [ ] All Rust models compile without errors
- [ ] IPC commands callable from frontend
- [ ] Zustand stores update on backend responses
- [ ] Type safety maintained across IPC boundary
- [ ] Error propagation works correctly

## Risk Assessment

| Risk | Impact | Likelihood | Mitigation |
|------|--------|------------|------------|
| Serde serialization mismatch | High | Medium | Define shared types carefully |
| Async command blocking | Medium | Low | Use async_runtime properly |
| Path expansion fails | Medium | Medium | Test on all platforms |

## Security Considerations
- File paths validated before access
- No arbitrary code execution
- Skill content sanitized in UI
- Home directory expansion scoped

## Next Steps
- Proceed to Phase 03: Local Skills Discovery
- Implement actual file scanning
- Test with real agent configs

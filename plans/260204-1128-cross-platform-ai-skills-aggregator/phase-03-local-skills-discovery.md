# Phase 03: Local Skills Discovery

## Context Links
- [Plan Overview](./plan.md)
- [Phase 02: Core Architecture](./phase-02-core-architecture.md)
- [AI Agent Skills Research](./research/researcher-02-ai-agent-skills-sources.md)

## Overview
**Priority**: P1 | **Status**: pending | **Effort**: 6h

Implement robust scanning of local AI agent config directories. Parse skills from multiple formats (MD, JSON, YAML, TXT). Handle platform-specific paths.

## Key Insights
- Each agent stores configs differently:
  - Claude: `~/.claude/` (MD, Python)
  - Cursor: `~/.cursor/` + project `.cursorrules` (plain text)
  - Continue.dev: `~/.continue/` (JSON)
  - Aider: `~/.aider.conf.yml` + `~/.aider/` (YAML, TXT)
  - Windsurf: `~/.codeium/` (JSON, YAML)
- Platform paths differ: `~` = `%USERPROFILE%` on Windows
- Some agents use project-level configs (Cursor, Aider)

## Requirements

### Functional
- F1: Scan all configured agent directories
- F2: Parse Markdown skills (extract name, description, tags)
- F3: Parse JSON configs (Continue.dev schema)
- F4: Parse YAML configs (Aider schema)
- F5: Detect skill format automatically
- F6: Handle missing/inaccessible directories gracefully
- F7: Support project-level skill discovery (optional scan path)

### Non-Functional
- NF1: Full scan completes <3s for typical setup
- NF2: Parallel scanning where possible
- NF3: Memory efficient (stream large files)

## Architecture

### Scanner Flow
```
┌─────────────────────────────────────────────────────────────────┐
│                      Skill Scanner Pipeline                     │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────────┐  │
│  │ Load Agent   │───>│ Resolve      │───>│ Glob Match       │  │
│  │ Configs      │    │ Base Paths   │    │ Files            │  │
│  └──────────────┘    └──────────────┘    └──────────────────┘  │
│                                                 │               │
│                                                 ▼               │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────────┐  │
│  │ Aggregate    │<───│ Parse &      │<───│ Detect Format    │  │
│  │ Results      │    │ Normalize    │    │ (MD/JSON/YAML)   │  │
│  └──────────────┘    └──────────────┘    └──────────────────┘  │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### Format-Specific Parsers

```rust
// src-tauri/src/parsers/mod.rs

pub mod markdown;
pub mod json;
pub mod yaml;
pub mod plaintext;

use crate::models::{Skill, SkillFormat, AgentType};

pub trait SkillParser {
    fn parse(&self, content: &str, file_path: &str, agent: &AgentType) -> Option<Skill>;
}

pub fn get_parser(format: SkillFormat) -> Box<dyn SkillParser> {
    match format {
        SkillFormat::Markdown => Box::new(markdown::MarkdownParser),
        SkillFormat::Json => Box::new(json::JsonParser),
        SkillFormat::Yaml => Box::new(yaml::YamlParser),
        SkillFormat::PlainText | SkillFormat::Python => Box::new(plaintext::PlainTextParser),
    }
}
```

## Related Code Files

### Files to Create
- `src-tauri/src/parsers/mod.rs` - Parser trait and factory
- `src-tauri/src/parsers/markdown.rs` - Markdown parser
- `src-tauri/src/parsers/json.rs` - JSON parser (Continue.dev)
- `src-tauri/src/parsers/yaml.rs` - YAML parser (Aider)
- `src-tauri/src/parsers/plaintext.rs` - Plain text parser
- `src-tauri/src/services/scanner_service.rs` - Main scanner logic
- `src-tauri/src/commands/scanner.rs` - Scanner IPC commands

### Files to Modify
- `src-tauri/src/main.rs` - Register scanner commands
- `src-tauri/src/services/mod.rs` - Export scanner service

## Implementation Steps

### 1. Create Markdown Parser (45 min)

Create `src-tauri/src/parsers/markdown.rs`:
```rust
use crate::models::{Skill, AgentType, SkillFormat};
use chrono::Utc;
use uuid::Uuid;
use regex::Regex;

pub struct MarkdownParser;

impl MarkdownParser {
    pub fn parse(&self, content: &str, file_path: &str, agent: &AgentType) -> Option<Skill> {
        let name = self.extract_title(content, file_path);
        let description = self.extract_description(content);
        let tags = self.extract_tags(content);

        Some(Skill {
            id: Uuid::new_v4().to_string(),
            name,
            description,
            agent: agent.clone(),
            format: SkillFormat::Markdown,
            file_path: file_path.to_string(),
            content: content.to_string(),
            tags,
            version: self.extract_version(content),
            remote_url: None,
            created_at: Utc::now().timestamp(),
            updated_at: Utc::now().timestamp(),
        })
    }

    fn extract_title(&self, content: &str, file_path: &str) -> String {
        // Try H1 heading first
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("# ") {
                return trimmed[2..].trim().to_string();
            }
        }
        // Fall back to filename
        std::path::Path::new(file_path)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Unknown")
            .to_string()
    }

    fn extract_description(&self, content: &str) -> Option<String> {
        let mut lines = content.lines().peekable();

        // Skip frontmatter if present
        if lines.peek().map(|l| l.trim()) == Some("---") {
            lines.next();
            while let Some(line) = lines.next() {
                if line.trim() == "---" {
                    break;
                }
            }
        }

        // Skip headings and empty lines
        while let Some(line) = lines.peek() {
            if line.trim().is_empty() || line.starts_with('#') {
                lines.next();
            } else {
                break;
            }
        }

        // Collect first paragraph
        let desc: String = lines
            .take_while(|l| !l.trim().is_empty())
            .collect::<Vec<_>>()
            .join(" ");

        if desc.is_empty() { None } else { Some(desc) }
    }

    fn extract_tags(&self, content: &str) -> Vec<String> {
        let mut tags = Vec::new();

        // Check YAML frontmatter for tags
        if content.starts_with("---") {
            let frontmatter: String = content
                .lines()
                .skip(1)
                .take_while(|l| l.trim() != "---")
                .collect::<Vec<_>>()
                .join("\n");

            // Simple tags extraction: tags: [tag1, tag2]
            let re = Regex::new(r"tags:\s*\[([^\]]+)\]").ok()?;
            if let Some(caps) = re.captures(&frontmatter) {
                tags.extend(
                    caps[1].split(',')
                        .map(|t| t.trim().to_string())
                        .filter(|t| !t.is_empty())
                );
            }
        }

        Some(tags)
    }

    fn extract_version(&self, content: &str) -> Option<String> {
        let re = Regex::new(r"version:\s*[\"']?([^\s\"'\n]+)").ok()?;
        re.captures(content).map(|c| c[1].to_string())
    }
}
```

### 2. Create JSON Parser (30 min)

Create `src-tauri/src/parsers/json.rs`:
```rust
use crate::models::{Skill, AgentType, SkillFormat};
use chrono::Utc;
use uuid::Uuid;
use serde_json::Value;

pub struct JsonParser;

impl JsonParser {
    pub fn parse(&self, content: &str, file_path: &str, agent: &AgentType) -> Option<Skill> {
        let json: Value = serde_json::from_str(content).ok()?;

        let name = self.extract_name(&json, file_path);
        let description = self.extract_description(&json);
        let tags = self.extract_tags(&json);

        Some(Skill {
            id: Uuid::new_v4().to_string(),
            name,
            description,
            agent: agent.clone(),
            format: SkillFormat::Json,
            file_path: file_path.to_string(),
            content: content.to_string(),
            tags,
            version: json.get("version").and_then(|v| v.as_str()).map(String::from),
            remote_url: None,
            created_at: Utc::now().timestamp(),
            updated_at: Utc::now().timestamp(),
        })
    }

    fn extract_name(&self, json: &Value, file_path: &str) -> String {
        json.get("name")
            .or_else(|| json.get("title"))
            .and_then(|v| v.as_str())
            .map(String::from)
            .unwrap_or_else(|| {
                std::path::Path::new(file_path)
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("Unknown")
                    .to_string()
            })
    }

    fn extract_description(&self, json: &Value) -> Option<String> {
        json.get("description")
            .and_then(|v| v.as_str())
            .map(String::from)
    }

    fn extract_tags(&self, json: &Value) -> Vec<String> {
        json.get("tags")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .map(String::from)
                    .collect()
            })
            .unwrap_or_default()
    }
}
```

### 3. Create YAML Parser (30 min)

Create `src-tauri/src/parsers/yaml.rs`:
```rust
use crate::models::{Skill, AgentType, SkillFormat};
use chrono::Utc;
use uuid::Uuid;
use serde_yaml::Value;

pub struct YamlParser;

impl YamlParser {
    pub fn parse(&self, content: &str, file_path: &str, agent: &AgentType) -> Option<Skill> {
        let yaml: Value = serde_yaml::from_str(content).ok()?;

        let name = self.extract_name(&yaml, file_path);
        let description = self.extract_description(&yaml);
        let tags = self.extract_tags(&yaml);

        Some(Skill {
            id: Uuid::new_v4().to_string(),
            name,
            description,
            agent: agent.clone(),
            format: SkillFormat::Yaml,
            file_path: file_path.to_string(),
            content: content.to_string(),
            tags,
            version: yaml.get("version").and_then(|v| v.as_str()).map(String::from),
            remote_url: None,
            created_at: Utc::now().timestamp(),
            updated_at: Utc::now().timestamp(),
        })
    }

    fn extract_name(&self, yaml: &Value, file_path: &str) -> String {
        yaml.get("name")
            .or_else(|| yaml.get("title"))
            .and_then(|v| v.as_str())
            .map(String::from)
            .unwrap_or_else(|| {
                std::path::Path::new(file_path)
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("Unknown")
                    .to_string()
            })
    }

    fn extract_description(&self, yaml: &Value) -> Option<String> {
        yaml.get("description")
            .and_then(|v| v.as_str())
            .map(String::from)
    }

    fn extract_tags(&self, yaml: &Value) -> Vec<String> {
        yaml.get("tags")
            .and_then(|v| v.as_sequence())
            .map(|seq| {
                seq.iter()
                    .filter_map(|v| v.as_str())
                    .map(String::from)
                    .collect()
            })
            .unwrap_or_default()
    }
}
```

### 4. Create Plain Text Parser (20 min)

Create `src-tauri/src/parsers/plaintext.rs`:
```rust
use crate::models::{Skill, AgentType, SkillFormat};
use chrono::Utc;
use uuid::Uuid;

pub struct PlainTextParser;

impl PlainTextParser {
    pub fn parse(&self, content: &str, file_path: &str, agent: &AgentType, format: SkillFormat) -> Option<Skill> {
        let name = std::path::Path::new(file_path)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Unknown")
            .to_string();

        // Try to get description from first non-empty line
        let description = content.lines()
            .find(|l| !l.trim().is_empty())
            .map(|l| l.chars().take(200).collect::<String>());

        Some(Skill {
            id: Uuid::new_v4().to_string(),
            name,
            description,
            agent: agent.clone(),
            format,
            file_path: file_path.to_string(),
            content: content.to_string(),
            tags: Vec::new(),
            version: None,
            remote_url: None,
            created_at: Utc::now().timestamp(),
            updated_at: Utc::now().timestamp(),
        })
    }
}
```

### 5. Create Scanner Service (60 min)

Create `src-tauri/src/services/scanner_service.rs`:
```rust
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;
use glob::glob;

use crate::models::{Skill, AgentConfig, AgentType, SkillFormat, AppError};
use crate::parsers::{markdown, json, yaml, plaintext};

pub struct ScannerService {
    agent_configs: Vec<AgentConfig>,
}

impl ScannerService {
    pub fn new(configs: Vec<AgentConfig>) -> Self {
        Self { agent_configs: configs }
    }

    pub async fn scan_all(&self) -> Result<Vec<Skill>, AppError> {
        let mut all_skills = Vec::new();

        for config in &self.agent_configs {
            if !config.enabled {
                continue;
            }

            match self.scan_agent(config).await {
                Ok(skills) => all_skills.extend(skills),
                Err(e) => {
                    eprintln!("Warning: Failed to scan {}: {}", config.name, e);
                    // Continue scanning other agents
                }
            }
        }

        Ok(all_skills)
    }

    pub async fn scan_agent(&self, config: &AgentConfig) -> Result<Vec<Skill>, AppError> {
        let base_path = expand_home_path(&config.base_path);

        if !base_path.exists() {
            return Ok(Vec::new()); // Agent not installed
        }

        let mut skills = Vec::new();

        for pattern in &config.skills_patterns {
            let full_pattern = base_path.join(pattern);
            let pattern_str = full_pattern.to_string_lossy();

            if let Ok(entries) = glob(&pattern_str) {
                for entry in entries.flatten() {
                    if entry.is_file() {
                        if let Some(skill) = self.parse_skill_file(&entry, &config.agent_type).await {
                            skills.push(skill);
                        }
                    }
                }
            }
        }

        Ok(skills)
    }

    async fn parse_skill_file(&self, path: &PathBuf, agent: &AgentType) -> Option<Skill> {
        let content = tokio::fs::read_to_string(path).await.ok()?;
        let format = detect_format(path);
        let path_str = path.to_string_lossy().to_string();

        match format {
            SkillFormat::Markdown => {
                markdown::MarkdownParser.parse(&content, &path_str, agent)
            }
            SkillFormat::Json => {
                json::JsonParser.parse(&content, &path_str, agent)
            }
            SkillFormat::Yaml => {
                yaml::YamlParser.parse(&content, &path_str, agent)
            }
            SkillFormat::PlainText | SkillFormat::Python => {
                plaintext::PlainTextParser.parse(&content, &path_str, agent, format)
            }
        }
    }

    pub async fn scan_directory(&self, path: &str, agent: AgentType) -> Result<Vec<Skill>, AppError> {
        let dir_path = PathBuf::from(path);
        if !dir_path.exists() {
            return Err(AppError::FileNotFound(path.to_string()));
        }

        let mut skills = Vec::new();
        let pattern = dir_path.join("**/*");

        if let Ok(entries) = glob(&pattern.to_string_lossy()) {
            for entry in entries.flatten() {
                if entry.is_file() && is_skill_file(&entry) {
                    if let Some(skill) = self.parse_skill_file(&entry, &agent).await {
                        skills.push(skill);
                    }
                }
            }
        }

        Ok(skills)
    }
}

fn expand_home_path(path: &str) -> PathBuf {
    if path.starts_with('~') {
        if let Some(home) = dirs::home_dir() {
            return home.join(&path[2..]);
        }
    }
    PathBuf::from(path)
}

fn detect_format(path: &PathBuf) -> SkillFormat {
    match path.extension().and_then(|e| e.to_str()) {
        Some("md") | Some("markdown") => SkillFormat::Markdown,
        Some("json") => SkillFormat::Json,
        Some("yaml") | Some("yml") => SkillFormat::Yaml,
        Some("py") => SkillFormat::Python,
        _ => SkillFormat::PlainText,
    }
}

fn is_skill_file(path: &PathBuf) -> bool {
    let valid_extensions = ["md", "markdown", "json", "yaml", "yml", "txt", "py"];
    path.extension()
        .and_then(|e| e.to_str())
        .map(|ext| valid_extensions.contains(&ext))
        .unwrap_or(false)
}
```

### 6. Create Scanner Commands (30 min)

Create `src-tauri/src/commands/scanner.rs`:
```rust
use crate::config::default_agent_configs;
use crate::models::{Skill, AgentType};
use crate::services::scanner_service::ScannerService;

#[tauri::command]
pub async fn scan_all_skills() -> Result<Vec<Skill>, String> {
    let configs = default_agent_configs();
    let scanner = ScannerService::new(configs);

    scanner.scan_all()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn scan_agent_skills(agent_type: String) -> Result<Vec<Skill>, String> {
    let configs = default_agent_configs();
    let config = configs.into_iter()
        .find(|c| format!("{:?}", c.agent_type) == agent_type)
        .ok_or_else(|| format!("Unknown agent: {}", agent_type))?;

    let scanner = ScannerService::new(vec![config.clone()]);
    scanner.scan_agent(&config)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn scan_directory(path: String, agent_type: String) -> Result<Vec<Skill>, String> {
    let agent = match agent_type.as_str() {
        "Claude" => AgentType::Claude,
        "Cursor" => AgentType::Cursor,
        "ContinueDev" => AgentType::ContinueDev,
        "Aider" => AgentType::Aider,
        "Windsurf" => AgentType::Windsurf,
        _ => AgentType::Custom(agent_type),
    };

    let scanner = ScannerService::new(vec![]);
    scanner.scan_directory(&path, agent)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_agent_status() -> Result<Vec<AgentStatus>, String> {
    let configs = default_agent_configs();
    let mut statuses = Vec::new();

    for config in configs {
        let base_path = expand_home_path(&config.base_path);
        statuses.push(AgentStatus {
            agent_type: format!("{:?}", config.agent_type),
            name: config.name,
            installed: base_path.exists(),
            path: base_path.to_string_lossy().to_string(),
        });
    }

    Ok(statuses)
}

#[derive(serde::Serialize)]
pub struct AgentStatus {
    pub agent_type: String,
    pub name: String,
    pub installed: bool,
    pub path: String,
}

fn expand_home_path(path: &str) -> std::path::PathBuf {
    if path.starts_with('~') {
        if let Some(home) = dirs::home_dir() {
            return home.join(&path[2..]);
        }
    }
    std::path::PathBuf::from(path)
}
```

### 7. Create Parser Module Index (10 min)

Create `src-tauri/src/parsers/mod.rs`:
```rust
pub mod markdown;
pub mod json;
pub mod yaml;
pub mod plaintext;
```

### 8. Add Regex Dependency (5 min)

Update `src-tauri/Cargo.toml`:
```toml
[dependencies]
# ... existing deps
regex = "1"
```

### 9. Frontend Scan Integration (30 min)

Update `src/lib/api.ts`:
```typescript
export const api = {
  // ... existing
  scanner: {
    scanAll: () => invoke<Skill[]>('scan_all_skills'),
    scanAgent: (agentType: string) => invoke<Skill[]>('scan_agent_skills', { agentType }),
    scanDirectory: (path: string, agentType: string) =>
      invoke<Skill[]>('scan_directory', { path, agentType }),
    getAgentStatus: () => invoke<AgentStatus[]>('get_agent_status'),
  },
};

export interface AgentStatus {
  agent_type: string;
  name: string;
  installed: boolean;
  path: string;
}
```

Update `src/stores/skills-store.ts`:
```typescript
// Add to SkillsState interface
scanSkills: () => Promise<void>;
scanAgent: (agentType: string) => Promise<void>;

// Add to store
scanSkills: async () => {
  set({ isLoading: true, error: null });
  try {
    const skills = await api.scanner.scanAll();
    set({ skills, isLoading: false });
  } catch (error) {
    set({ error: String(error), isLoading: false });
  }
},

scanAgent: async (agentType) => {
  set({ isLoading: true, error: null });
  try {
    const skills = await api.scanner.scanAgent(agentType);
    set((state) => ({
      skills: [...state.skills.filter(s => s.agent !== agentType), ...skills],
      isLoading: false,
    }));
  } catch (error) {
    set({ error: String(error), isLoading: false });
  }
},
```

## Todo List
- [ ] Create Markdown parser with frontmatter support
- [ ] Create JSON parser for Continue.dev configs
- [ ] Create YAML parser for Aider configs
- [ ] Create plain text parser fallback
- [ ] Create parser module index
- [ ] Implement scanner service with async file reading
- [ ] Create scanner IPC commands
- [ ] Add regex dependency
- [ ] Update frontend API wrappers
- [ ] Update Zustand store with scan actions
- [ ] Test scanning on real agent directories
- [ ] Handle edge cases (empty files, permission errors)

## Success Criteria
- [ ] Scans all 5 agent directories (Claude, Cursor, Continue, Aider, Windsurf)
- [ ] Parses Markdown with YAML frontmatter correctly
- [ ] Parses JSON configs without errors
- [ ] Parses YAML configs without errors
- [ ] Handles missing directories gracefully
- [ ] Full scan <3s for typical setup
- [ ] No blocking UI during scan

## Risk Assessment

| Risk | Impact | Likelihood | Mitigation |
|------|--------|------------|------------|
| Platform path differences | High | High | Test on all OS, use dirs crate |
| Large file memory issues | Medium | Low | Stream files >1MB |
| Parse failures | Medium | Medium | Graceful fallback to plain text |
| Permission denied | Low | Medium | Log warning, continue scanning |

## Security Considerations
- Only scan configured directories
- No arbitrary path traversal
- Validate file extensions before parsing
- Don't execute Python skills, only read them

## Next Steps
- Proceed to Phase 04: Remote Skills Registry
- Implement skill fetching from URLs
- Design remote repository schema

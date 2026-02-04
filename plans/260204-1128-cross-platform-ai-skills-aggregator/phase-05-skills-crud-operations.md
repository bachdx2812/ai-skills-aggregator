# Phase 05: Skills CRUD Operations

## Context Links
- [Plan Overview](./plan.md)
- [Phase 02: Core Architecture](./phase-02-core-architecture.md)
- [Phase 03: Local Skills Discovery](./phase-03-local-skills-discovery.md)

## Overview
**Priority**: P1 | **Status**: pending | **Effort**: 6h

Implement full CRUD operations for skills: create new skills locally, read/view, update/edit, delete, and duplicate skills across different agent formats. **NEW**: Enhanced local skill creation workflow with metadata for potential publishing.

## Key Insights
- Skills are text files; CRUD = file operations
- Different agents need different file locations
- Skill format determines default template
- Backup before destructive operations
- Support undo via file restore

## Requirements

### Functional
- F1: Create new skill with format template
- F2: Read skill content for editor display
- F3: Update skill content and metadata
- F4: Delete skill with confirmation
- F5: Duplicate skill (copy to same/different agent)
- F6: Export skill to file or clipboard
- F7: Import skill from file or clipboard
- F8: Validate skill format before save
- **F9: Mark skill as local (is_local=true) on creation**
- **F10: Capture metadata for publishing (name, description, tags required)**
- **F11: Generate unique skill ID on local creation**

### Non-Functional
- NF1: Save operation <100ms
- NF2: Auto-save with 3s debounce
- NF3: Backup retained for 7 days
- NF4: Validate content length <1MB

## Architecture

### CRUD Flow
```
┌─────────────────────────────────────────────────────────────────┐
│                      Skills CRUD Flow                           │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  CREATE (LOCAL)                                                 │
│  ┌────────────┐   ┌────────────┐   ┌────────────────────────┐  │
│  │ Enter Name │──>│ Select     │──>│ Enter Description      │  │
│  │ + Tags     │   │ Agent/Type │   │ (for future publish)   │  │
│  └────────────┘   └────────────┘   └────────────────────────┘  │
│         │                                                       │
│         ▼                                                       │
│  ┌────────────┐   ┌────────────┐   ┌────────────────────────┐  │
│  │ Generate   │──>│ Set        │──>│ Write to Agent Dir     │  │
│  │ Template   │   │ is_local=1 │   │ (with UUID filename)   │  │
│  └────────────┘   └────────────┘   └────────────────────────┘  │
│                                                                 │
│  READ                                                           │
│  ┌────────────┐   ┌────────────┐   ┌────────────────────────┐  │
│  │ Select     │──>│ Load from  │──>│ Display in Editor      │  │
│  │ Skill      │   │ File       │   │ (syntax highlighted)   │  │
│  └────────────┘   └────────────┘   └────────────────────────┘  │
│                                                                 │
│  UPDATE                                                         │
│  ┌────────────┐   ┌────────────┐   ┌────────────────────────┐  │
│  │ Edit in    │──>│ Validate   │──>│ Backup + Write         │  │
│  │ Editor     │   │ Content    │   │ (atomic replace)       │  │
│  └────────────┘   └────────────┘   └────────────────────────┘  │
│                                                                 │
│  DELETE                                                         │
│  ┌────────────┐   ┌────────────┐   ┌────────────────────────┐  │
│  │ Confirm    │──>│ Backup     │──>│ Remove File            │  │
│  │ Dialog     │   │ First      │   │ (move to trash)        │  │
│  └────────────┘   └────────────┘   └────────────────────────┘  │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

## Related Code Files

### Files to Create
- `src-tauri/src/services/crud_service.rs` - CRUD business logic
- `src-tauri/src/services/backup_service.rs` - Backup/restore logic
- `src-tauri/src/services/template_service.rs` - Skill templates
- `src-tauri/src/commands/crud.rs` - CRUD IPC commands
- `src/hooks/use-skill-editor.ts` - Editor state hook
- `src/lib/templates.ts` - Frontend skill templates

### Files to Modify
- `src-tauri/src/services/mod.rs` - Export new services
- `src-tauri/src/main.rs` - Register CRUD commands
- `src/stores/skills-store.ts` - Add CRUD actions
- `src/lib/api.ts` - Add CRUD API calls

## Implementation Steps

### 1. Create Template Service (30 min)

Create `src-tauri/src/services/template_service.rs`:
```rust
use crate::models::{AgentType, SkillFormat};

pub struct TemplateService;

impl TemplateService {
    pub fn get_template(agent: &AgentType, format: &SkillFormat) -> String {
        match (agent, format) {
            (AgentType::Claude, SkillFormat::Markdown) => Self::claude_md_template(),
            (AgentType::Claude, SkillFormat::Python) => Self::claude_py_template(),
            (AgentType::Cursor, SkillFormat::PlainText) => Self::cursor_template(),
            (AgentType::ContinueDev, SkillFormat::Json) => Self::continue_template(),
            (AgentType::Aider, SkillFormat::Yaml) => Self::aider_template(),
            (AgentType::Aider, SkillFormat::PlainText) => Self::aider_prompt_template(),
            _ => Self::generic_md_template(),
        }
    }

    fn claude_md_template() -> String {
        r#"# Skill Name

Brief description of what this skill does.

## Usage

When to use this skill and how it helps.

## Instructions

1. First instruction
2. Second instruction
3. Third instruction

## Examples

```
Example usage here
```

## Notes

- Additional notes
- Limitations or caveats
"#.to_string()
    }

    fn claude_py_template() -> String {
        r#"#!/usr/bin/env python3
"""
Skill Name

Brief description of what this skill does.
"""

import sys

def main():
    """Main entry point for the skill."""
    # Your skill implementation here
    print("Hello from skill!")
    return 0

if __name__ == "__main__":
    sys.exit(main())
"#.to_string()
    }

    fn cursor_template() -> String {
        r#"# Cursor Rules

You are an expert assistant following these guidelines:

## Code Style
- Write clean, readable code
- Follow best practices
- Add meaningful comments

## Behavior
- Be concise and helpful
- Explain your reasoning
- Suggest improvements

## Restrictions
- Don't make assumptions
- Ask for clarification when needed
"#.to_string()
    }

    fn continue_template() -> String {
        r#"{
  "name": "Custom Skill",
  "version": "1.0.0",
  "description": "Brief description",
  "systemMessage": "You are a helpful assistant.",
  "contextProviders": [],
  "slashCommands": []
}
"#.to_string()
    }

    fn aider_template() -> String {
        r#"# Aider Configuration

model: gpt-4
edit-format: diff
auto-commits: true

# Custom settings
map-tokens: 1024
"#.to_string()
    }

    fn aider_prompt_template() -> String {
        r#"You are an expert developer. Follow these guidelines:

1. Write clean, maintainable code
2. Add tests for new features
3. Document complex logic
4. Follow project conventions
"#.to_string()
    }

    fn generic_md_template() -> String {
        r#"# Skill Name

## Description

What this skill does.

## Instructions

1. Step one
2. Step two
3. Step three
"#.to_string()
    }
}
```

### 2. Create Backup Service (30 min)

Create `src-tauri/src/services/backup_service.rs`:
```rust
use std::path::PathBuf;
use chrono::{Utc, Duration};
use tokio::fs;
use crate::models::AppError;

pub struct BackupService {
    backup_dir: PathBuf,
    retention_days: i64,
}

impl BackupService {
    pub fn new() -> Self {
        let backup_dir = dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("ai-skills-aggregator")
            .join("backups");

        Self {
            backup_dir,
            retention_days: 7,
        }
    }

    pub async fn backup_file(&self, file_path: &str) -> Result<PathBuf, AppError> {
        let source = PathBuf::from(file_path);
        if !source.exists() {
            return Err(AppError::FileNotFound(file_path.to_string()));
        }

        // Create backup filename with timestamp
        let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
        let file_name = source.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");
        let backup_name = format!("{}_{}", timestamp, file_name);

        // Ensure backup dir exists
        fs::create_dir_all(&self.backup_dir)
            .await
            .map_err(|e| AppError::IoError(e.to_string()))?;

        let backup_path = self.backup_dir.join(backup_name);

        // Copy file to backup
        fs::copy(&source, &backup_path)
            .await
            .map_err(|e| AppError::IoError(e.to_string()))?;

        Ok(backup_path)
    }

    pub async fn restore_file(&self, backup_path: &str, dest_path: &str) -> Result<(), AppError> {
        let backup = PathBuf::from(backup_path);
        let dest = PathBuf::from(dest_path);

        if !backup.exists() {
            return Err(AppError::FileNotFound(backup_path.to_string()));
        }

        // Ensure parent dir exists
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent)
                .await
                .map_err(|e| AppError::IoError(e.to_string()))?;
        }

        fs::copy(&backup, &dest)
            .await
            .map_err(|e| AppError::IoError(e.to_string()))?;

        Ok(())
    }

    pub async fn cleanup_old_backups(&self) -> Result<usize, AppError> {
        if !self.backup_dir.exists() {
            return Ok(0);
        }

        let cutoff = Utc::now() - Duration::days(self.retention_days);
        let mut deleted = 0;

        let mut entries = fs::read_dir(&self.backup_dir)
            .await
            .map_err(|e| AppError::IoError(e.to_string()))?;

        while let Some(entry) = entries.next_entry().await
            .map_err(|e| AppError::IoError(e.to_string()))? {

            let metadata = entry.metadata().await
                .map_err(|e| AppError::IoError(e.to_string()))?;

            if let Ok(modified) = metadata.modified() {
                let modified_time = chrono::DateTime::<Utc>::from(modified);
                if modified_time < cutoff {
                    if fs::remove_file(entry.path()).await.is_ok() {
                        deleted += 1;
                    }
                }
            }
        }

        Ok(deleted)
    }

    pub async fn list_backups(&self, original_filename: &str) -> Result<Vec<BackupInfo>, AppError> {
        if !self.backup_dir.exists() {
            return Ok(Vec::new());
        }

        let mut backups = Vec::new();
        let mut entries = fs::read_dir(&self.backup_dir)
            .await
            .map_err(|e| AppError::IoError(e.to_string()))?;

        while let Some(entry) = entries.next_entry().await
            .map_err(|e| AppError::IoError(e.to_string()))? {

            let name = entry.file_name().to_string_lossy().to_string();
            if name.ends_with(original_filename) {
                if let Ok(metadata) = entry.metadata().await {
                    let size = metadata.len();
                    let modified = metadata.modified()
                        .map(|t| chrono::DateTime::<Utc>::from(t).timestamp())
                        .unwrap_or(0);

                    backups.push(BackupInfo {
                        path: entry.path().to_string_lossy().to_string(),
                        name,
                        size,
                        created_at: modified,
                    });
                }
            }
        }

        // Sort by creation time descending
        backups.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        Ok(backups)
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct BackupInfo {
    pub path: String,
    pub name: String,
    pub size: u64,
    pub created_at: i64,
}
```

### 3. Create CRUD Service (60 min)

Create `src-tauri/src/services/crud_service.rs`:
```rust
use std::path::PathBuf;
use uuid::Uuid;
use chrono::Utc;
use tokio::fs;

use crate::models::{Skill, AgentType, SkillFormat, AppError};
use crate::services::backup_service::BackupService;
use crate::services::template_service::TemplateService;

pub struct CrudService {
    backup: BackupService,
}

impl CrudService {
    pub fn new() -> Self {
        Self {
            backup: BackupService::new(),
        }
    }

    pub async fn create_skill(
        &self,
        name: &str,
        description: Option<String>,
        tags: Vec<String>,
        agent: AgentType,
        format: SkillFormat,
        content: Option<String>,
    ) -> Result<Skill, AppError> {
        // Validate required fields for publishable skills
        if name.len() < 3 {
            return Err(AppError::InvalidPath("Skill name must be at least 3 characters".into()));
        }

        // Get content from template or provided
        let skill_content = content.unwrap_or_else(|| {
            TemplateService::get_template(&agent, &format)
        });

        // Generate file path
        let file_path = self.generate_skill_path(&agent, &format, name)?;

        // Ensure directory exists
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent)
                .await
                .map_err(|e| AppError::IoError(e.to_string()))?;
        }

        // Write file
        fs::write(&file_path, &skill_content)
            .await
            .map_err(|e| AppError::IoError(e.to_string()))?;

        // Return skill object - marked as local
        Ok(Skill {
            id: Uuid::new_v4().to_string(),
            name: name.to_string(),
            description,
            agent,
            format,
            file_path: file_path.to_string_lossy().to_string(),
            content: skill_content,
            tags,
            version: Some("1.0.0".to_string()),
            remote_url: None,
            author: None,              // Set when published
            is_local: true,            // Created locally, not from registry
            created_at: Utc::now().timestamp(),
            updated_at: Utc::now().timestamp(),
        })
    }

    pub async fn read_skill(&self, file_path: &str) -> Result<String, AppError> {
        fs::read_to_string(file_path)
            .await
            .map_err(|e| AppError::IoError(e.to_string()))
    }

    pub async fn update_skill(
        &self,
        file_path: &str,
        content: &str,
        create_backup: bool,
    ) -> Result<(), AppError> {
        // Validate content
        self.validate_content(content)?;

        // Create backup if requested and file exists
        let path = PathBuf::from(file_path);
        if create_backup && path.exists() {
            self.backup.backup_file(file_path).await?;
        }

        // Atomic write: write to temp file then rename
        let temp_path = path.with_extension("tmp");

        fs::write(&temp_path, content)
            .await
            .map_err(|e| AppError::IoError(e.to_string()))?;

        fs::rename(&temp_path, &path)
            .await
            .map_err(|e| AppError::IoError(e.to_string()))?;

        Ok(())
    }

    pub async fn delete_skill(&self, file_path: &str) -> Result<(), AppError> {
        // Backup before delete
        self.backup.backup_file(file_path).await?;

        // Delete file
        fs::remove_file(file_path)
            .await
            .map_err(|e| AppError::IoError(e.to_string()))?;

        Ok(())
    }

    pub async fn duplicate_skill(
        &self,
        source_path: &str,
        new_name: &str,
        target_agent: Option<AgentType>,
    ) -> Result<Skill, AppError> {
        // Read source content
        let content = self.read_skill(source_path).await?;
        let source = PathBuf::from(source_path);

        // Detect format from source
        let format = self.detect_format(&source);

        // Use target agent or infer from source path
        let agent = target_agent.unwrap_or_else(|| self.infer_agent_from_path(&source));

        // Create new skill with content
        self.create_skill(new_name, agent, format, Some(content)).await
    }

    pub async fn export_skill(&self, file_path: &str) -> Result<ExportData, AppError> {
        let content = self.read_skill(file_path).await?;
        let path = PathBuf::from(file_path);

        let filename = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("skill.md")
            .to_string();

        Ok(ExportData { filename, content })
    }

    pub async fn import_skill(
        &self,
        content: &str,
        name: &str,
        agent: AgentType,
    ) -> Result<Skill, AppError> {
        // Detect format from content
        let format = self.detect_format_from_content(content);

        self.create_skill(name, agent, format, Some(content.to_string())).await
    }

    // Helper methods

    fn generate_skill_path(
        &self,
        agent: &AgentType,
        format: &SkillFormat,
        name: &str,
    ) -> Result<PathBuf, AppError> {
        let home = dirs::home_dir()
            .ok_or_else(|| AppError::InvalidPath("Cannot find home directory".into()))?;

        let sanitized_name = self.sanitize_filename(name);
        let extension = self.get_extension(format);

        let path = match agent {
            AgentType::Claude => {
                home.join(".claude").join("skills").join(format!("{}.{}", sanitized_name, extension))
            }
            AgentType::Cursor => {
                home.join(".cursor").join("skills").join(format!("{}.cursorrules", sanitized_name))
            }
            AgentType::ContinueDev => {
                home.join(".continue").join("skills").join(format!("{}.json", sanitized_name))
            }
            AgentType::Aider => {
                home.join(".aider").join("prompts").join(format!("{}.txt", sanitized_name))
            }
            AgentType::Windsurf => {
                home.join(".codeium").join("skills").join(format!("{}.{}", sanitized_name, extension))
            }
            AgentType::Copilot => {
                home.join(".github-copilot").join("skills").join(format!("{}.{}", sanitized_name, extension))
            }
            AgentType::Custom(_) => {
                home.join(".ai-skills").join(format!("{}.{}", sanitized_name, extension))
            }
        };

        Ok(path)
    }

    fn sanitize_filename(&self, name: &str) -> String {
        name.chars()
            .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '-' })
            .collect::<String>()
            .to_lowercase()
    }

    fn get_extension(&self, format: &SkillFormat) -> &'static str {
        match format {
            SkillFormat::Markdown => "md",
            SkillFormat::Json => "json",
            SkillFormat::Yaml => "yaml",
            SkillFormat::Python => "py",
            SkillFormat::PlainText => "txt",
        }
    }

    fn detect_format(&self, path: &PathBuf) -> SkillFormat {
        match path.extension().and_then(|e| e.to_str()) {
            Some("md") => SkillFormat::Markdown,
            Some("json") => SkillFormat::Json,
            Some("yaml") | Some("yml") => SkillFormat::Yaml,
            Some("py") => SkillFormat::Python,
            _ => SkillFormat::PlainText,
        }
    }

    fn detect_format_from_content(&self, content: &str) -> SkillFormat {
        let trimmed = content.trim();
        if trimmed.starts_with('{') || trimmed.starts_with('[') {
            SkillFormat::Json
        } else if trimmed.starts_with("---") || trimmed.contains(": ") {
            SkillFormat::Yaml
        } else if trimmed.starts_with('#') || trimmed.contains("##") {
            SkillFormat::Markdown
        } else if trimmed.starts_with("#!/") || trimmed.contains("def ") {
            SkillFormat::Python
        } else {
            SkillFormat::PlainText
        }
    }

    fn infer_agent_from_path(&self, path: &PathBuf) -> AgentType {
        let path_str = path.to_string_lossy().to_lowercase();
        if path_str.contains(".claude") {
            AgentType::Claude
        } else if path_str.contains(".cursor") {
            AgentType::Cursor
        } else if path_str.contains(".continue") {
            AgentType::ContinueDev
        } else if path_str.contains(".aider") {
            AgentType::Aider
        } else if path_str.contains(".codeium") {
            AgentType::Windsurf
        } else {
            AgentType::Custom("Unknown".to_string())
        }
    }

    fn validate_content(&self, content: &str) -> Result<(), AppError> {
        // Check max size (1MB)
        if content.len() > 1_000_000 {
            return Err(AppError::InvalidPath("Content too large (max 1MB)".into()));
        }

        // Check for binary content
        if content.bytes().any(|b| b == 0) {
            return Err(AppError::InvalidPath("Binary content not allowed".into()));
        }

        Ok(())
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ExportData {
    pub filename: String,
    pub content: String,
}
```

### 4. Create CRUD Commands (30 min)

Create `src-tauri/src/commands/crud.rs`:
```rust
use crate::models::{Skill, AgentType, SkillFormat};
use crate::services::crud_service::{CrudService, ExportData};
use crate::services::backup_service::BackupInfo;

#[tauri::command]
pub async fn create_skill(
    name: String,
    description: Option<String>,
    tags: Vec<String>,
    agent: String,
    format: String,
    content: Option<String>,
) -> Result<Skill, String> {
    let service = CrudService::new();
    let agent_type = parse_agent(&agent)?;
    let skill_format = parse_format(&format)?;

    service.create_skill(&name, description, tags, agent_type, skill_format, content)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn read_skill_content(file_path: String) -> Result<String, String> {
    let service = CrudService::new();
    service.read_skill(&file_path)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_skill_content(
    file_path: String,
    content: String,
    create_backup: bool,
) -> Result<(), String> {
    let service = CrudService::new();
    service.update_skill(&file_path, &content, create_backup)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_skill_file(file_path: String) -> Result<(), String> {
    let service = CrudService::new();
    service.delete_skill(&file_path)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn duplicate_skill(
    source_path: String,
    new_name: String,
    target_agent: Option<String>,
) -> Result<Skill, String> {
    let service = CrudService::new();
    let agent = target_agent.map(|a| parse_agent(&a)).transpose()?;

    service.duplicate_skill(&source_path, &new_name, agent)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn export_skill(file_path: String) -> Result<ExportData, String> {
    let service = CrudService::new();
    service.export_skill(&file_path)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn import_skill(
    content: String,
    name: String,
    agent: String,
) -> Result<Skill, String> {
    let service = CrudService::new();
    let agent_type = parse_agent(&agent)?;

    service.import_skill(&content, &name, agent_type)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn list_skill_backups(filename: String) -> Result<Vec<BackupInfo>, String> {
    let backup = crate::services::backup_service::BackupService::new();
    backup.list_backups(&filename)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn restore_skill_backup(
    backup_path: String,
    dest_path: String,
) -> Result<(), String> {
    let backup = crate::services::backup_service::BackupService::new();
    backup.restore_file(&backup_path, &dest_path)
        .await
        .map_err(|e| e.to_string())
}

// Helper functions

fn parse_agent(agent: &str) -> Result<AgentType, String> {
    match agent.to_lowercase().as_str() {
        "claude" => Ok(AgentType::Claude),
        "cursor" => Ok(AgentType::Cursor),
        "continuedev" | "continue" => Ok(AgentType::ContinueDev),
        "aider" => Ok(AgentType::Aider),
        "windsurf" | "codeium" => Ok(AgentType::Windsurf),
        "copilot" => Ok(AgentType::Copilot),
        _ => Ok(AgentType::Custom(agent.to_string())),
    }
}

fn parse_format(format: &str) -> Result<SkillFormat, String> {
    match format.to_lowercase().as_str() {
        "markdown" | "md" => Ok(SkillFormat::Markdown),
        "json" => Ok(SkillFormat::Json),
        "yaml" | "yml" => Ok(SkillFormat::Yaml),
        "python" | "py" => Ok(SkillFormat::Python),
        "text" | "txt" | "plaintext" => Ok(SkillFormat::PlainText),
        _ => Err(format!("Unknown format: {}", format)),
    }
}
```

### 5. Update Frontend API (20 min)

Update `src/lib/api.ts`:
```typescript
import type { Skill, ExportData, BackupInfo } from './types';

export const api = {
  // ... existing

  crud: {
    create: (
      name: string,
      description: string | undefined,
      tags: string[],
      agent: string,
      format: string,
      content?: string
    ) => invoke<Skill>('create_skill', { name, description, tags, agent, format, content }),

    read: (filePath: string) =>
      invoke<string>('read_skill_content', { filePath }),

    update: (filePath: string, content: string, createBackup = true) =>
      invoke<void>('update_skill_content', { filePath, content, createBackup }),

    delete: (filePath: string) =>
      invoke<void>('delete_skill_file', { filePath }),

    duplicate: (sourcePath: string, newName: string, targetAgent?: string) =>
      invoke<Skill>('duplicate_skill', { sourcePath, newName, targetAgent }),

    export: (filePath: string) =>
      invoke<ExportData>('export_skill', { filePath }),

    import: (content: string, name: string, agent: string) =>
      invoke<Skill>('import_skill', { content, name, agent }),
  },

  backup: {
    list: (filename: string) =>
      invoke<BackupInfo[]>('list_skill_backups', { filename }),

    restore: (backupPath: string, destPath: string) =>
      invoke<void>('restore_skill_backup', { backupPath, destPath }),
  },
};

export interface ExportData {
  filename: string;
  content: string;
}

export interface BackupInfo {
  path: string;
  name: string;
  size: number;
  created_at: number;
}
```

### 6. Create Editor Hook (30 min)

Create `src/hooks/use-skill-editor.ts`:
```typescript
import { useState, useCallback, useRef, useEffect } from 'react';
import { api } from '@/lib/api';
import type { Skill } from '@/lib/types';
import debounce from 'lodash.debounce';

interface UseSkillEditorOptions {
  autoSave?: boolean;
  autoSaveDelay?: number;
}

export function useSkillEditor(
  skill: Skill | null,
  options: UseSkillEditorOptions = {}
) {
  const { autoSave = true, autoSaveDelay = 3000 } = options;

  const [content, setContent] = useState('');
  const [originalContent, setOriginalContent] = useState('');
  const [isLoading, setIsLoading] = useState(false);
  const [isSaving, setIsSaving] = useState(false);
  const [isDirty, setIsDirty] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Load content when skill changes
  useEffect(() => {
    if (skill?.file_path) {
      loadContent(skill.file_path);
    } else {
      setContent('');
      setOriginalContent('');
      setIsDirty(false);
    }
  }, [skill?.file_path]);

  const loadContent = async (filePath: string) => {
    setIsLoading(true);
    setError(null);
    try {
      const fileContent = await api.crud.read(filePath);
      setContent(fileContent);
      setOriginalContent(fileContent);
      setIsDirty(false);
    } catch (e) {
      setError(String(e));
    } finally {
      setIsLoading(false);
    }
  };

  // Debounced auto-save
  const debouncedSave = useRef(
    debounce(async (filePath: string, newContent: string) => {
      setIsSaving(true);
      try {
        await api.crud.update(filePath, newContent, true);
        setOriginalContent(newContent);
        setIsDirty(false);
      } catch (e) {
        setError(String(e));
      } finally {
        setIsSaving(false);
      }
    }, autoSaveDelay)
  ).current;

  const handleContentChange = useCallback(
    (newContent: string) => {
      setContent(newContent);
      setIsDirty(newContent !== originalContent);

      if (autoSave && skill?.file_path && newContent !== originalContent) {
        debouncedSave(skill.file_path, newContent);
      }
    },
    [autoSave, skill?.file_path, originalContent, debouncedSave]
  );

  const save = useCallback(async () => {
    if (!skill?.file_path || !isDirty) return;

    setIsSaving(true);
    setError(null);
    try {
      await api.crud.update(skill.file_path, content, true);
      setOriginalContent(content);
      setIsDirty(false);
    } catch (e) {
      setError(String(e));
    } finally {
      setIsSaving(false);
    }
  }, [skill?.file_path, content, isDirty]);

  const revert = useCallback(() => {
    setContent(originalContent);
    setIsDirty(false);
    debouncedSave.cancel();
  }, [originalContent, debouncedSave]);

  return {
    content,
    setContent: handleContentChange,
    isLoading,
    isSaving,
    isDirty,
    error,
    save,
    revert,
  };
}
```

### 7. Create Templates File (15 min)

Create `src/lib/templates.ts`:
```typescript
import type { SkillFormat } from './types';

export const SKILL_TEMPLATES: Record<string, Record<SkillFormat, string>> = {
  Claude: {
    Markdown: `# Skill Name

Brief description.

## Instructions

1. First step
2. Second step
`,
    Python: `#!/usr/bin/env python3
"""Skill description."""

def main():
    print("Hello!")
    return 0

if __name__ == "__main__":
    main()
`,
    Json: '{}',
    Yaml: 'name: skill\n',
    PlainText: 'Instructions here.',
  },
  Cursor: {
    Markdown: '# Rules\n\nYour guidelines here.',
    PlainText: 'You are a helpful assistant.',
    Json: '{}',
    Yaml: '',
    Python: '',
  },
  // ... other agents
};

export function getTemplate(agent: string, format: SkillFormat): string {
  return SKILL_TEMPLATES[agent]?.[format] ?? '';
}
```

## Todo List
- [ ] Create template service with agent-specific templates
- [ ] Create backup service with retention policy
- [ ] Create CRUD service with validation
- [ ] Create CRUD IPC commands (with description/tags params)
- [ ] Update frontend API wrappers (incl. description/tags)
- [ ] Create useSkillEditor hook with auto-save
- [ ] Create frontend templates file
- [ ] Register commands in main.rs
- [ ] Test create/read/update/delete operations
- [ ] Test duplicate across agents
- [ ] Test backup and restore
- [ ] Handle edge cases (large files, permissions)
- [ ] **Test local skill creation with metadata**
- [ ] **Verify is_local flag is set correctly**

## Success Criteria
- [ ] Create skill generates correct file in correct location
- [ ] Update saves with backup
- [ ] Delete creates backup before removal
- [ ] Duplicate works across agents
- [ ] Export returns correct filename and content
- [ ] Import detects format automatically
- [ ] Auto-save works with 3s debounce
- [ ] Backup cleanup removes files >7 days old
- [ ] **Local skills have is_local=true**
- [ ] **Skills from registry have is_local=false**
- [ ] **Metadata (name, description, tags) captured on creation**

## Risk Assessment

| Risk | Impact | Likelihood | Mitigation |
|------|--------|------------|------------|
| Data loss on failed write | High | Low | Atomic write, backup first |
| Filename collisions | Medium | Medium | Add UUID suffix if exists |
| Large file memory | Medium | Low | Stream for files >1MB |
| Permission denied | Medium | Medium | Clear error message |

## Security Considerations
- Sanitize filenames (no path traversal)
- Validate content size before write
- No binary content allowed
- Backup sensitive files before modify

## Next Steps
- Proceed to Phase 06: UI Implementation
- Build skill browser component
- Build skill editor with syntax highlighting

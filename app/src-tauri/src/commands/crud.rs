use crate::models::{Skill, SkillFile, AgentType, SkillFormat};
use crate::services::crud_service::{CrudService, ExportData};
use crate::services::backup_service::{BackupService, BackupInfo};

fn parse_agent(agent: &str) -> Result<AgentType, String> {
    match agent.to_lowercase().as_str() {
        "claude" => Ok(AgentType::Claude),
        "cursor" => Ok(AgentType::Cursor),
        "continuedev" | "continue" => Ok(AgentType::ContinueDev),
        "aider" => Ok(AgentType::Aider),
        "windsurf" | "codeium" => Ok(AgentType::Windsurf),
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

    service.create_skill(&name, description, tags, &agent_type, &skill_format, content)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn create_skill_file(
    skill_folder: String,
    file_name: String,
    format: String,
    content: Option<String>,
) -> Result<SkillFile, String> {
    let service = CrudService::new();
    let skill_format = parse_format(&format)?;

    service.create_file(&skill_folder, &file_name, &skill_format, content)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn read_skill_content(file_path: String) -> Result<String, String> {
    let service = CrudService::new();
    service.read_content(&file_path)
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
    service.update_content(&file_path, &content, create_backup)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_skill(folder_path: String) -> Result<(), String> {
    let service = CrudService::new();
    service.delete_skill(&folder_path)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_skill_file(file_path: String) -> Result<(), String> {
    let service = CrudService::new();
    service.delete_file(&file_path)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn duplicate_skill(
    source_path: String,
    new_name: String,
) -> Result<String, String> {
    let service = CrudService::new();
    service.duplicate_skill(&source_path, &new_name)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn rename_skill(
    folder_path: String,
    new_name: String,
) -> Result<String, String> {
    let service = CrudService::new();
    service.rename_skill(&folder_path, &new_name)
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
pub async fn list_skill_backups(filename: String) -> Result<Vec<BackupInfo>, String> {
    let backup = BackupService::new();
    backup.list_backups(&filename)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn restore_skill_backup(
    backup_path: String,
    dest_path: String,
) -> Result<(), String> {
    let backup = BackupService::new();
    backup.restore_file(&backup_path, &dest_path)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn cleanup_old_backups() -> Result<usize, String> {
    let backup = BackupService::new();
    backup.cleanup_old_backups()
        .await
        .map_err(|e| e.to_string())
}

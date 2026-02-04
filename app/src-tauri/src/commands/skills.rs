use crate::models::{AgentConfig, AgentType, Skill, SkillFile};
use crate::services::SkillService;
use std::sync::Mutex;
use once_cell::sync::Lazy;

// Global state for skills and configs
static SKILLS_CACHE: Lazy<Mutex<Vec<Skill>>> = Lazy::new(|| Mutex::new(Vec::new()));
static AGENT_CONFIGS: Lazy<Mutex<Vec<AgentConfig>>> = Lazy::new(|| {
    Mutex::new(AgentConfig::defaults())
});

#[tauri::command]
pub fn scan_skills() -> Result<Vec<Skill>, String> {
    let configs = AGENT_CONFIGS.lock().map_err(|e| e.to_string())?;
    let skills = SkillService::scan_all_skills(&configs).map_err(|e| e.to_string())?;

    // Update cache
    let mut cache = SKILLS_CACHE.lock().map_err(|e| e.to_string())?;
    *cache = skills.clone();

    Ok(skills)
}

#[tauri::command]
pub fn get_all_skills() -> Result<Vec<Skill>, String> {
    let cache = SKILLS_CACHE.lock().map_err(|e| e.to_string())?;
    Ok(cache.clone())
}

#[tauri::command]
pub fn get_skills_by_agent(agent: String) -> Result<Vec<Skill>, String> {
    let cache = SKILLS_CACHE.lock().map_err(|e| e.to_string())?;

    let agent_type: AgentType = match agent.to_lowercase().as_str() {
        "claude" => AgentType::Claude,
        "cursor" => AgentType::Cursor,
        "continuedev" => AgentType::ContinueDev,
        "aider" => AgentType::Aider,
        "windsurf" => AgentType::Windsurf,
        other => AgentType::Custom(other.to_string()),
    };

    let filtered: Vec<Skill> = cache.iter()
        .filter(|s| s.agent == agent_type)
        .cloned()
        .collect();

    Ok(filtered)
}

#[tauri::command]
pub fn get_skill_by_id(id: String) -> Result<Option<Skill>, String> {
    let cache = SKILLS_CACHE.lock().map_err(|e| e.to_string())?;
    Ok(cache.iter().find(|s| s.id == id).cloned())
}

#[tauri::command]
pub fn read_skill_content(file_path: String) -> Result<String, String> {
    SkillService::read_content(&file_path).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_skill(
    agent: String,
    name: String,
    content: String,
    description: Option<String>,
    tags: Option<Vec<String>>,
) -> Result<Skill, String> {
    let configs = AGENT_CONFIGS.lock().map_err(|e| e.to_string())?;

    let agent_type: AgentType = match agent.to_lowercase().as_str() {
        "claude" => AgentType::Claude,
        "cursor" => AgentType::Cursor,
        "continuedev" => AgentType::ContinueDev,
        "aider" => AgentType::Aider,
        "windsurf" => AgentType::Windsurf,
        other => AgentType::Custom(other.to_string()),
    };

    let skill = SkillService::create_skill(
        &agent_type,
        &name,
        &content,
        description.as_deref(),
        tags,
        &configs,
    ).map_err(|e| e.to_string())?;

    // Update cache
    let mut cache = SKILLS_CACHE.lock().map_err(|e| e.to_string())?;
    cache.push(skill.clone());

    Ok(skill)
}

#[tauri::command]
pub fn update_skill(file_path: String, content: String) -> Result<Skill, String> {
    let skill = SkillService::update_skill(&file_path, &content).map_err(|e| e.to_string())?;

    // Update cache
    let mut cache = SKILLS_CACHE.lock().map_err(|e| e.to_string())?;
    if let Some(pos) = cache.iter().position(|s| s.folder_path == file_path) {
        cache[pos] = skill.clone();
    }

    Ok(skill)
}

#[tauri::command]
pub fn delete_skill(file_path: String) -> Result<(), String> {
    SkillService::delete_skill(&file_path).map_err(|e| e.to_string())?;

    // Update cache
    let mut cache = SKILLS_CACHE.lock().map_err(|e| e.to_string())?;
    cache.retain(|s| s.folder_path != file_path);

    Ok(())
}

#[tauri::command]
pub fn duplicate_skill(file_path: String, new_name: String) -> Result<Skill, String> {
    let skill = SkillService::duplicate_skill(&file_path, &new_name).map_err(|e| e.to_string())?;

    // Update cache
    let mut cache = SKILLS_CACHE.lock().map_err(|e| e.to_string())?;
    cache.push(skill.clone());

    Ok(skill)
}

#[tauri::command]
pub fn get_agent_configs() -> Result<Vec<AgentConfig>, String> {
    let configs = AGENT_CONFIGS.lock().map_err(|e| e.to_string())?;
    Ok(configs.clone())
}

#[tauri::command]
pub fn get_skill_files(folder_path: String) -> Result<Vec<SkillFile>, String> {
    SkillService::get_skill_files(&folder_path).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_skill_file(
    skill_folder: String,
    file_name: String,
    content: Option<String>,
) -> Result<SkillFile, String> {
    SkillService::create_file(&skill_folder, &file_name, content.as_deref())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_skill_file(file_path: String) -> Result<(), String> {
    SkillService::delete_file(&file_path).map_err(|e| e.to_string())
}

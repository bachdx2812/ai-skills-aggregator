use crate::models::{SkillRegistry, RemoteSkill, InstalledSkill, RegistryConfig, SkillUpdate};
use crate::services::registry_service::RegistryService;

#[tauri::command]
pub async fn fetch_registry(url: String) -> Result<SkillRegistry, String> {
    let service = RegistryService::new();
    let config = RegistryConfig {
        url,
        name: "Custom".into(),
        enabled: true,
        auth_token: None,
    };

    service.fetch_registry(&config)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn install_remote_skill(
    skill: RemoteSkill,
    registry_url: String,
    agent: String,
) -> Result<InstalledSkill, String> {
    let service = RegistryService::new();

    service.install_skill(&skill, &registry_url, &agent)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn uninstall_remote_skill(skill_id: String, agent: String) -> Result<(), String> {
    let service = RegistryService::new();

    service.uninstall_skill(&skill_id, &agent)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_installed_skills() -> Result<Vec<InstalledSkill>, String> {
    let service = RegistryService::new();

    service.get_installed_skills()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn check_skill_updates(registry_url: String) -> Result<Vec<SkillUpdate>, String> {
    let service = RegistryService::new();
    let config = RegistryConfig {
        url: registry_url,
        name: "".into(),
        enabled: true,
        auth_token: None,
    };

    let registry = service.fetch_registry(&config)
        .await
        .map_err(|e| e.to_string())?;

    service.check_updates(&registry)
        .await
        .map_err(|e| e.to_string())
}

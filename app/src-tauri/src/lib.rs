// Module declarations
mod commands;
mod models;
mod services;

use commands::skills;
use commands::registry;
use commands::updates;
use commands::auth;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_os::init())
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Skills commands
            skills::scan_skills,
            skills::get_all_skills,
            skills::get_skills_by_agent,
            skills::get_skill_by_id,
            skills::read_skill_content,
            skills::create_skill,
            skills::update_skill,
            skills::delete_skill,
            skills::duplicate_skill,
            skills::get_agent_configs,
            skills::get_skill_files,
            skills::create_skill_file,
            skills::delete_skill_file,
            // Registry commands
            registry::fetch_registry,
            registry::install_remote_skill,
            registry::uninstall_remote_skill,
            registry::get_installed_skills,
            registry::check_skill_updates,
            // Update commands
            updates::check_for_updates,
            updates::apply_skill_update,
            updates::apply_all_skill_updates,
            updates::rollback_skill,
            updates::skip_skill_version,
            // Auth commands
            auth::login,
            auth::logout,
            auth::get_current_user,
            auth::is_logged_in,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

use serde::{Deserialize, Serialize};
use super::AgentType;

/// Agent configuration for scanning skills
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub agent: AgentType,
    pub name: String,
    pub config_dir: String,
    pub skills_dir: Option<String>,
    pub file_patterns: Vec<String>,
    pub enabled: bool,
}

impl AgentConfig {
    /// Get default configurations for all supported agents
    pub fn defaults() -> Vec<Self> {
        let home = dirs::home_dir().unwrap_or_default();
        let home_str = home.to_string_lossy();

        vec![
            AgentConfig {
                agent: AgentType::Claude,
                name: "Claude Code".to_string(),
                config_dir: format!("{}/.claude", home_str),
                skills_dir: Some(format!("{}/.claude/skills", home_str)),
                file_patterns: vec![
                    "*.md".to_string(),
                    "CLAUDE.md".to_string(),
                    "rules/*.md".to_string(),
                ],
                enabled: true,
            },
            AgentConfig {
                agent: AgentType::Cursor,
                name: "Cursor".to_string(),
                config_dir: format!("{}/.cursor", home_str),
                skills_dir: None,
                file_patterns: vec![
                    ".cursorrules".to_string(),
                    "*.cursorrules".to_string(),
                ],
                enabled: true,
            },
            AgentConfig {
                agent: AgentType::ContinueDev,
                name: "Continue.dev".to_string(),
                config_dir: format!("{}/.continue", home_str),
                skills_dir: None,
                file_patterns: vec![
                    "config.json".to_string(),
                    "profiles/*.json".to_string(),
                ],
                enabled: false,
            },
            AgentConfig {
                agent: AgentType::Aider,
                name: "Aider".to_string(),
                config_dir: format!("{}/.aider", home_str),
                skills_dir: None,
                file_patterns: vec![
                    ".aider.conf.yml".to_string(),
                    "*.txt".to_string(),
                ],
                enabled: false,
            },
            AgentConfig {
                agent: AgentType::Windsurf,
                name: "Windsurf/Codeium".to_string(),
                config_dir: format!("{}/.codeium", home_str),
                skills_dir: None,
                file_patterns: vec![
                    "*.yaml".to_string(),
                    "*.json".to_string(),
                ],
                enabled: false,
            },
        ]
    }
}

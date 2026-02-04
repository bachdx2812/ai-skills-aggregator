use serde::{Deserialize, Serialize};

/// Supported AI agent types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub enum AgentType {
    Claude,
    Cursor,
    ContinueDev,
    Aider,
    Windsurf,
    Custom(String),
}

impl std::fmt::Display for AgentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AgentType::Claude => write!(f, "Claude"),
            AgentType::Cursor => write!(f, "Cursor"),
            AgentType::ContinueDev => write!(f, "ContinueDev"),
            AgentType::Aider => write!(f, "Aider"),
            AgentType::Windsurf => write!(f, "Windsurf"),
            AgentType::Custom(name) => write!(f, "{}", name),
        }
    }
}

/// Skill file format
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub enum SkillFormat {
    Markdown,
    Json,
    Yaml,
    Python,
    PlainText,
}

impl SkillFormat {
    pub fn from_extension(ext: &str) -> Self {
        match ext.to_lowercase().as_str() {
            "md" | "markdown" => SkillFormat::Markdown,
            "json" => SkillFormat::Json,
            "yaml" | "yml" => SkillFormat::Yaml,
            "py" => SkillFormat::Python,
            _ => SkillFormat::PlainText,
        }
    }

    pub fn extension(&self) -> &str {
        match self {
            SkillFormat::Markdown => "md",
            SkillFormat::Json => "json",
            SkillFormat::Yaml => "yaml",
            SkillFormat::Python => "py",
            SkillFormat::PlainText => "txt",
        }
    }
}

/// A file within a skill folder
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillFile {
    pub name: String,
    pub file_path: String,
    pub format: SkillFormat,
    pub is_entry: bool,
    pub size: u64,
}

/// Core skill model - represents a skill folder
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub folder_path: String,
    pub agent: AgentType,
    pub files: Vec<SkillFile>,
    pub entry_file: Option<String>,
    pub tags: Vec<String>,
    pub version: Option<String>,
    pub author: Option<String>,
    pub is_local: bool,
    pub is_folder: bool,
    pub file_count: usize,
    pub created_at: i64,
    pub updated_at: i64,
}

impl Skill {
    pub fn new_folder(
        name: String,
        folder_path: String,
        agent: AgentType,
        files: Vec<SkillFile>,
    ) -> Self {
        let now = chrono::Utc::now().timestamp();
        let entry_file = files.iter()
            .find(|f| f.is_entry)
            .map(|f| f.file_path.clone());
        let file_count = files.len();

        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            description: None,
            folder_path,
            agent,
            files,
            entry_file,
            tags: Vec::new(),
            version: None,
            author: None,
            is_local: true,
            is_folder: true,
            file_count,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn new_single_file(
        name: String,
        file_path: String,
        agent: AgentType,
        format: SkillFormat,
    ) -> Self {
        let now = chrono::Utc::now().timestamp();
        let file = SkillFile {
            name: name.clone(),
            file_path: file_path.clone(),
            format,
            is_entry: true,
            size: 0,
        };

        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            description: None,
            folder_path: file_path.clone(),
            agent,
            files: vec![file],
            entry_file: Some(file_path),
            tags: Vec::new(),
            version: None,
            author: None,
            is_local: true,
            is_folder: false,
            file_count: 1,
            created_at: now,
            updated_at: now,
        }
    }
}

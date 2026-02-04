// Agent types
export type AgentType = 'Claude' | 'Cursor' | 'ContinueDev' | 'Aider' | 'Windsurf' | { Custom: string };

// Skill format types
export type SkillFormat = 'Markdown' | 'Json' | 'Yaml' | 'Python' | 'PlainText';

// A file within a skill folder
export interface SkillFile {
  name: string;
  file_path: string;
  format: SkillFormat;
  is_entry: boolean;
  size: number;
}

// Core skill model - represents a skill folder
export interface Skill {
  id: string;
  name: string;
  description: string | null;
  folder_path: string;
  agent: AgentType;
  files: SkillFile[];
  entry_file: string | null;
  tags: string[];
  version: string | null;
  author: string | null;
  is_local: boolean;
  is_folder: boolean;
  file_count: number;
  created_at: number;
  updated_at: number;
}

// Agent configuration
export interface AgentConfig {
  agent: AgentType;
  name: string;
  config_dir: string;
  skills_dir: string | null;
  file_patterns: string[];
  enabled: boolean;
}

// Remote registry types
export interface SkillRegistry {
  version: string;
  name: string;
  description: string | null;
  url: string;
  skills: RemoteSkill[];
  last_updated: number;
}

export interface RemoteSkill {
  id: string;
  name: string;
  description: string | null;
  version: string;
  author: string | null;
  agents: string[];
  tags: string[];
  files: SkillFiles;
  url: string | null;
  checksum: string | null;
}

export interface SkillFiles {
  claude: string | null;
  cursor: string | null;
  continue_dev: string | null;
  aider: string | null;
  windsurf: string | null;
}

export interface InstalledSkill {
  skill_id: string;
  registry_url: string;
  version: string;
  installed_path: string;
  agent: string;
  installed_at: number;
}

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

export interface UpdateCheckResult {
  available_updates: SkillUpdate[];
  last_checked: number;
  error: string | null;
}

// Auth types
export interface User {
  id: string;
  username: string;
  display_name: string | null;
  avatar_url: string | null;
  access_token: string;
  logged_in_at: number;
}

export type AuthState =
  | { type: 'logged_out' }
  | { type: 'logging_in' }
  | { type: 'logged_in'; user: User }
  | { type: 'error'; message: string };

// Publish types
export interface PublishResponse {
  success: boolean;
  skill_id: string;
  url: string;
  message: string;
}

// App error type
export interface AppError {
  code: string;
  message: string;
}

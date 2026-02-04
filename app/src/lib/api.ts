import { invoke } from '@tauri-apps/api/core';
import type {
  Skill,
  SkillFile,
  AgentConfig,
  SkillRegistry,
  RemoteSkill,
  InstalledSkill,
  SkillUpdate,
  UpdateCheckResult,
  User,
  PublishResponse,
} from './types';

// Skills API
export const api = {
  skills: {
    scan: () => invoke<Skill[]>('scan_skills'),
    getAll: () => invoke<Skill[]>('get_all_skills'),
    getByAgent: (agent: string) => invoke<Skill[]>('get_skills_by_agent', { agent }),
    getById: (id: string) => invoke<Skill | null>('get_skill_by_id', { id }),
    getFiles: (folderPath: string) => invoke<SkillFile[]>('get_skill_files', { folderPath }),
    readContent: (filePath: string) => invoke<string>('read_skill_content', { filePath }),
    create: (agent: string, name: string, content: string, description?: string, tags?: string[]) =>
      invoke<Skill>('create_skill', { agent, name, content, description, tags }),
    update: (filePath: string, content: string) =>
      invoke<Skill>('update_skill', { filePath, content }),
    delete: (folderPath: string) => invoke<void>('delete_skill', { folderPath }),
    duplicate: (folderPath: string, newName: string) =>
      invoke<Skill>('duplicate_skill', { filePath: folderPath, newName }),
    createFile: (skillFolder: string, fileName: string, content?: string) =>
      invoke<SkillFile>('create_skill_file', { skillFolder, fileName, content }),
    deleteFile: (filePath: string) => invoke<void>('delete_skill_file', { filePath }),
  },

  agents: {
    getConfigs: () => invoke<AgentConfig[]>('get_agent_configs'),
    updateConfig: (config: AgentConfig) => invoke<void>('update_agent_config', { config }),
  },

  registry: {
    fetch: (url: string) => invoke<SkillRegistry>('fetch_registry', { url }),
    install: (skill: RemoteSkill, registryUrl: string, agent: string) =>
      invoke<InstalledSkill>('install_remote_skill', { skill, registryUrl, agent }),
    uninstall: (skillId: string, agent: string) =>
      invoke<void>('uninstall_remote_skill', { skillId, agent }),
    getInstalled: () => invoke<InstalledSkill[]>('get_installed_skills'),
    checkUpdates: (registryUrl: string) =>
      invoke<SkillUpdate[]>('check_skill_updates', { registryUrl }),
  },

  auth: {
    login: () => invoke<User>('login'),
    logout: () => invoke<void>('logout'),
    getCurrentUser: () => invoke<User | null>('get_current_user'),
    isLoggedIn: () => invoke<boolean>('is_logged_in'),
  },

  publish: {
    validate: (skill: Skill) => invoke<string[]>('validate_skill_for_publish', { skill }),
    publish: (skill: Skill, registryUrl: string) =>
      invoke<PublishResponse>('publish_skill', { skill, registryUrl }),
    unpublish: (skillId: string, registryUrl: string) =>
      invoke<void>('unpublish_skill', { skillId, registryUrl }),
  },

  updates: {
    check: () => invoke<UpdateCheckResult>('check_for_updates'),
    apply: (update: SkillUpdate) => invoke<void>('apply_skill_update', { update }),
    applyAll: (updates: SkillUpdate[]) =>
      invoke<Array<{ Ok?: null; Err?: string }>>('apply_all_skill_updates', { updates }),
    skip: (skillId: string, version: string) =>
      invoke<void>('skip_skill_version', { skillId, version }),
    rollback: (skillId: string, agent: string) =>
      invoke<void>('rollback_skill', { skillId, agent }),
  },
};

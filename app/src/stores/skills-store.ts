import { create } from 'zustand';
import type { Skill, AgentConfig, SkillFile } from '@/lib/types';
import { api } from '@/lib/api';

interface SkillsState {
  skills: Skill[];
  agentConfigs: AgentConfig[];
  isLoading: boolean;
  error: string | null;

  // Actions
  scanSkills: () => Promise<void>;
  loadSkills: () => Promise<void>;
  loadAgentConfigs: () => Promise<void>;
  createSkill: (agent: string, name: string, content: string, description?: string, tags?: string[]) => Promise<Skill>;
  updateSkill: (filePath: string, content: string) => Promise<Skill>;
  deleteSkill: (folderPath: string) => Promise<void>;
  duplicateSkill: (filePath: string, newName: string) => Promise<Skill>;
  createFile: (skillFolder: string, fileName: string, content?: string) => Promise<SkillFile>;
  deleteFile: (filePath: string) => Promise<void>;
  clearError: () => void;
}

export const useSkillsStore = create<SkillsState>((set) => ({
  skills: [],
  agentConfigs: [],
  isLoading: false,
  error: null,

  scanSkills: async () => {
    set({ isLoading: true, error: null });
    try {
      const skills = await api.skills.scan();
      set({ skills, isLoading: false });
    } catch (error) {
      set({ error: String(error), isLoading: false });
    }
  },

  loadSkills: async () => {
    set({ isLoading: true, error: null });
    try {
      const skills = await api.skills.getAll();
      set({ skills, isLoading: false });
    } catch (error) {
      set({ error: String(error), isLoading: false });
    }
  },

  loadAgentConfigs: async () => {
    try {
      const configs = await api.agents.getConfigs();
      set({ agentConfigs: configs });
    } catch (error) {
      set({ error: String(error) });
    }
  },

  createSkill: async (agent, name, content, description, tags) => {
    set({ isLoading: true, error: null });
    try {
      const skill = await api.skills.create(agent, name, content, description, tags);
      set((state) => ({
        skills: [...state.skills, skill],
        isLoading: false,
      }));
      return skill;
    } catch (error) {
      set({ error: String(error), isLoading: false });
      throw error;
    }
  },

  updateSkill: async (filePath, content) => {
    set({ error: null });
    try {
      const updatedSkill = await api.skills.update(filePath, content);
      set((state) => ({
        skills: state.skills.map((s) =>
          s.folder_path === updatedSkill.folder_path ? updatedSkill : s
        ),
      }));
      return updatedSkill;
    } catch (error) {
      set({ error: String(error) });
      throw error;
    }
  },

  deleteSkill: async (folderPath) => {
    set({ error: null });
    try {
      await api.skills.delete(folderPath);
      set((state) => ({
        skills: state.skills.filter((s) => s.folder_path !== folderPath),
      }));
    } catch (error) {
      set({ error: String(error) });
      throw error;
    }
  },

  duplicateSkill: async (folderPath, newName) => {
    set({ error: null });
    try {
      const newSkill = await api.skills.duplicate(folderPath, newName);
      set((state) => ({
        skills: [...state.skills, newSkill],
      }));
      return newSkill;
    } catch (error) {
      set({ error: String(error) });
      throw error;
    }
  },

  createFile: async (skillFolder, fileName, content) => {
    set({ error: null });
    try {
      const newFile = await api.skills.createFile(skillFolder, fileName, content);
      // Re-scan to update the skill's file list
      const skills = await api.skills.scan();
      set({ skills });
      return newFile;
    } catch (error) {
      set({ error: String(error) });
      throw error;
    }
  },

  deleteFile: async (filePath) => {
    set({ error: null });
    try {
      await api.skills.deleteFile(filePath);
      // Re-scan to update the skill's file list
      const skills = await api.skills.scan();
      set({ skills });
    } catch (error) {
      set({ error: String(error) });
      throw error;
    }
  },

  clearError: () => set({ error: null }),
}));

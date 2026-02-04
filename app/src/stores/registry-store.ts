import { create } from 'zustand';
import type { SkillRegistry, RemoteSkill, InstalledSkill, SkillUpdate } from '@/lib/types';
import { api } from '@/lib/api';

interface RegistryState {
  registries: SkillRegistry[];
  installedSkills: InstalledSkill[];
  updates: SkillUpdate[];
  isLoading: boolean;
  error: string | null;

  // Actions
  fetchRegistry: (url: string) => Promise<SkillRegistry>;
  installSkill: (skill: RemoteSkill, registryUrl: string, agent: string) => Promise<InstalledSkill>;
  uninstallSkill: (skillId: string, agent: string) => Promise<void>;
  loadInstalledSkills: () => Promise<void>;
  checkUpdates: (registryUrl: string) => Promise<void>;
  clearError: () => void;
}

export const useRegistryStore = create<RegistryState>((set) => ({
  registries: [],
  installedSkills: [],
  updates: [],
  isLoading: false,
  error: null,

  fetchRegistry: async (url) => {
    set({ isLoading: true, error: null });
    try {
      const registry = await api.registry.fetch(url);
      set((state) => ({
        registries: [...state.registries.filter(r => r.url !== url), registry],
        isLoading: false,
      }));
      return registry;
    } catch (error) {
      set({ error: String(error), isLoading: false });
      throw error;
    }
  },

  installSkill: async (skill, registryUrl, agent) => {
    set({ isLoading: true, error: null });
    try {
      const installed = await api.registry.install(skill, registryUrl, agent);
      set((state) => ({
        installedSkills: [...state.installedSkills, installed],
        isLoading: false,
      }));
      return installed;
    } catch (error) {
      set({ error: String(error), isLoading: false });
      throw error;
    }
  },

  uninstallSkill: async (skillId, agent) => {
    set({ isLoading: true, error: null });
    try {
      await api.registry.uninstall(skillId, agent);
      set((state) => ({
        installedSkills: state.installedSkills.filter(
          s => !(s.skill_id === skillId && s.agent === agent)
        ),
        isLoading: false,
      }));
    } catch (error) {
      set({ error: String(error), isLoading: false });
      throw error;
    }
  },

  loadInstalledSkills: async () => {
    try {
      const installed = await api.registry.getInstalled();
      set({ installedSkills: installed });
    } catch (error) {
      set({ error: String(error) });
    }
  },

  checkUpdates: async (registryUrl) => {
    try {
      const updates = await api.registry.checkUpdates(registryUrl);
      set({ updates });
    } catch (error) {
      set({ error: String(error) });
    }
  },

  clearError: () => set({ error: null }),
}));

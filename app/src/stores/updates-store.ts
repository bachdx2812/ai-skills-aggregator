import { create } from 'zustand';
import { api } from '@/lib/api';

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

interface UpdatesState {
  updates: SkillUpdate[];
  isChecking: boolean;
  lastChecked: number | null;
  error: string | null;

  checkForUpdates: () => Promise<void>;
  applyUpdate: (update: SkillUpdate) => Promise<void>;
  applyAllUpdates: () => Promise<void>;
  skipVersion: (skillId: string, version: string) => Promise<void>;
  rollback: (skillId: string, agent: string) => Promise<void>;
  clearError: () => void;
}

export const useUpdatesStore = create<UpdatesState>((set, get) => ({
  updates: [],
  isChecking: false,
  lastChecked: null,
  error: null,

  checkForUpdates: async () => {
    set({ isChecking: true, error: null });
    try {
      const result = await api.updates.check();
      set({
        updates: result.available_updates,
        lastChecked: result.last_checked,
        isChecking: false,
        error: result.error,
      });
    } catch (error) {
      set({ error: String(error), isChecking: false });
    }
  },

  applyUpdate: async (update) => {
    try {
      await api.updates.apply(update);
      set((state) => ({
        updates: state.updates.filter(
          (u) => !(u.skill_id === update.skill_id && u.agent === update.agent)
        ),
      }));
    } catch (error) {
      set({ error: String(error) });
      throw error;
    }
  },

  applyAllUpdates: async () => {
    const { updates } = get();
    try {
      const results = await api.updates.applyAll(updates);
      // Filter out successfully updated skills
      const failedIndices = results
        .map((r, i) => (r.Err ? i : -1))
        .filter((i) => i !== -1);

      set((state) => ({
        updates: state.updates.filter((_, i) => failedIndices.includes(i)),
        error: failedIndices.length > 0
          ? `${failedIndices.length} update(s) failed`
          : null,
      }));
    } catch (error) {
      set({ error: String(error) });
      throw error;
    }
  },

  skipVersion: async (skillId, version) => {
    try {
      await api.updates.skip(skillId, version);
      set((state) => ({
        updates: state.updates.filter((u) => u.skill_id !== skillId),
      }));
    } catch (error) {
      set({ error: String(error) });
      throw error;
    }
  },

  rollback: async (skillId, agent) => {
    try {
      await api.updates.rollback(skillId, agent);
    } catch (error) {
      set({ error: String(error) });
      throw error;
    }
  },

  clearError: () => set({ error: null }),
}));

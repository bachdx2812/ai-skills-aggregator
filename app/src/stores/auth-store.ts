import { create } from 'zustand';
import { persist } from 'zustand/middleware';
import type { User, AuthState } from '@/lib/types';
import { api } from '@/lib/api';

interface AuthStore {
  user: User | null;
  authState: AuthState;
  isLoading: boolean;

  // Actions
  login: () => Promise<void>;
  logout: () => Promise<void>;
  checkAuthStatus: () => Promise<void>;
}

export const useAuthStore = create<AuthStore>()(
  persist(
    (set) => ({
      user: null,
      authState: { type: 'logged_out' },
      isLoading: false,

      login: async () => {
        set({ authState: { type: 'logging_in' }, isLoading: true });
        try {
          const user = await api.auth.login();
          set({
            user,
            authState: { type: 'logged_in', user },
            isLoading: false,
          });
        } catch (error) {
          set({
            authState: { type: 'error', message: String(error) },
            isLoading: false,
          });
        }
      },

      logout: async () => {
        set({ isLoading: true });
        try {
          await api.auth.logout();
          set({
            user: null,
            authState: { type: 'logged_out' },
            isLoading: false,
          });
        } catch (error) {
          set({ isLoading: false });
        }
      },

      checkAuthStatus: async () => {
        try {
          const user = await api.auth.getCurrentUser();
          if (user) {
            set({ user, authState: { type: 'logged_in', user } });
          } else {
            set({ user: null, authState: { type: 'logged_out' } });
          }
        } catch {
          set({ user: null, authState: { type: 'logged_out' } });
        }
      },
    }),
    { name: 'ai-skills-auth' }
  )
);

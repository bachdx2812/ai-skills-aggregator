import { create } from 'zustand';
import { persist } from 'zustand/middleware';
import type { AgentType } from '@/lib/types';

type Theme = 'light' | 'dark' | 'system';
type ActiveTab = 'local' | 'registry' | 'settings';

interface UIState {
  selectedSkillId: string | null;
  selectedFileId: string | null;
  searchQuery: string;
  filterAgent: AgentType | null;
  sidebarCollapsed: boolean;
  detailPanelOpen: boolean;
  theme: Theme;
  activeTab: ActiveTab;

  // Actions
  selectSkill: (id: string | null) => void;
  setSelectedFile: (fileId: string | null) => void;
  setSearchQuery: (query: string) => void;
  setFilterAgent: (agent: AgentType | null) => void;
  setSidebarCollapsed: (collapsed: boolean) => void;
  toggleSidebar: () => void;
  toggleDetailPanel: () => void;
  setTheme: (theme: Theme) => void;
  setActiveTab: (tab: ActiveTab) => void;
}

export const useUIStore = create<UIState>()(
  persist(
    (set) => ({
      selectedSkillId: null,
      selectedFileId: null,
      searchQuery: '',
      filterAgent: null,
      sidebarCollapsed: false,
      detailPanelOpen: true,
      theme: 'system',
      activeTab: 'local',

      selectSkill: (id) => set({ selectedSkillId: id, selectedFileId: null, detailPanelOpen: !!id }),
      setSelectedFile: (fileId) => set({ selectedFileId: fileId, detailPanelOpen: !!fileId }),
      setSearchQuery: (query) => set({ searchQuery: query }),
      setFilterAgent: (agent) => set({ filterAgent: agent }),
      setSidebarCollapsed: (collapsed) => set({ sidebarCollapsed: collapsed }),
      toggleSidebar: () => set((s) => ({ sidebarCollapsed: !s.sidebarCollapsed })),
      toggleDetailPanel: () => set((s) => ({ detailPanelOpen: !s.detailPanelOpen })),
      setTheme: (theme) => set({ theme }),
      setActiveTab: (tab) => set({ activeTab: tab }),
    }),
    { name: 'ai-skills-ui' }
  )
);

import { useEffect } from 'react';
import { useUIStore } from '@/stores/ui-store';
import { useSkillsStore } from '@/stores/skills-store';
import { Sidebar } from '@/components/layout/Sidebar';
import { MainPanel } from '@/components/layout/MainPanel';
import { DetailPanel } from '@/components/layout/DetailPanel';

function App() {
  const { theme, sidebarCollapsed, selectedFileId } = useUIStore();
  const { scanSkills, loadAgentConfigs, isLoading } = useSkillsStore();

  // Load skills and agent configs on mount
  useEffect(() => {
    loadAgentConfigs();
    scanSkills();
  }, [scanSkills, loadAgentConfigs]);

  // Apply theme
  useEffect(() => {
    const root = document.documentElement;
    if (theme === 'dark' || (theme === 'system' && window.matchMedia('(prefers-color-scheme: dark)').matches)) {
      root.classList.add('dark');
    } else {
      root.classList.remove('dark');
    }
  }, [theme]);

  return (
    <div className="flex h-screen bg-surface-0">
      {/* Sidebar */}
      <aside
        className={`flex-shrink-0 transition-all duration-200 ease-out ${
          sidebarCollapsed ? 'w-14' : 'w-60'
        }`}
      >
        <Sidebar />
      </aside>

      {/* Main Content */}
      <main className="flex-1 flex overflow-hidden">
        <div className="flex-1 overflow-auto">
          <MainPanel />
        </div>

        {/* Detail Panel - only show when a file is selected */}
        {selectedFileId && (
          <div className="w-[520px] border-l border-subtle overflow-hidden animate-slide-in">
            <DetailPanel />
          </div>
        )}
      </main>

      {/* Loading overlay */}
      {isLoading && (
        <div className="fixed inset-0 bg-black/40 backdrop-blur-sm flex items-center justify-center z-50">
          <div className="bg-surface-1 rounded-xl p-6 shadow-2xl border border-subtle animate-fade-in">
            <div className="spinner mx-auto" />
            <p className="mt-3 text-sm text-secondary">Loading skills...</p>
          </div>
        </div>
      )}
    </div>
  );
}

export default App;

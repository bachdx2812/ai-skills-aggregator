import { useSkillsStore } from '@/stores/skills-store';
import { useUIStore } from '@/stores/ui-store';
import type { AgentType } from '@/lib/types';

const AGENT_CONFIG: Record<string, { color: string; gradient: string }> = {
  Claude: { color: 'bg-orange-500', gradient: 'from-orange-500 to-orange-600' },
  Cursor: { color: 'bg-purple-500', gradient: 'from-purple-500 to-purple-600' },
  ContinueDev: { color: 'bg-blue-500', gradient: 'from-blue-500 to-blue-600' },
  Aider: { color: 'bg-green-500', gradient: 'from-green-500 to-green-600' },
  Windsurf: { color: 'bg-cyan-500', gradient: 'from-cyan-500 to-cyan-600' },
};

function getAgentKey(agent: AgentType): string {
  return typeof agent === 'string' ? agent : agent.Custom;
}

export function Sidebar() {
  const { skills, agentConfigs } = useSkillsStore();
  const { sidebarCollapsed, filterAgent, setSidebarCollapsed, setFilterAgent, selectedSkillId, selectSkill } = useUIStore();

  // Count skills per agent
  const skillCounts = skills.reduce((acc, skill) => {
    const key = getAgentKey(skill.agent);
    acc[key] = (acc[key] || 0) + 1;
    return acc;
  }, {} as Record<string, number>);

  const totalSkills = skills.length;

  const handleAgentClick = (agent: AgentType | null) => {
    setFilterAgent(agent);
  };

  const handleBack = () => {
    setFilterAgent(null);
  };

  // Get filtered skills for current agent
  const filteredSkills = filterAgent
    ? skills.filter((s) => getAgentKey(s.agent) === getAgentKey(filterAgent))
    : [];

  return (
    <div className="h-full flex flex-col bg-surface-1 border-r border-subtle">
      {/* Header */}
      <div className="h-14 px-4 border-b border-subtle flex items-center justify-between flex-shrink-0">
        {filterAgent && !sidebarCollapsed ? (
          <button
            onClick={handleBack}
            className="flex items-center gap-2 text-secondary hover:text-primary transition-colors"
          >
            <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 19l-7-7 7-7" />
            </svg>
            <span className="text-sm font-medium">Back</span>
          </button>
        ) : (
          !sidebarCollapsed && (
            <div className="flex items-center gap-2">
              <div className="w-6 h-6 rounded-md bg-gradient-to-br from-[#58a6ff] to-[#a371f7] flex items-center justify-center">
                <svg className="w-3.5 h-3.5 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13 10V3L4 14h7v7l9-11h-7z" />
                </svg>
              </div>
              <h1 className="font-semibold text-primary text-sm">AI Skills</h1>
            </div>
          )
        )}
        <button
          onClick={() => setSidebarCollapsed(!sidebarCollapsed)}
          className="p-1.5 rounded-md hover:bg-surface-2 text-muted hover:text-secondary transition-colors"
          title={sidebarCollapsed ? 'Expand sidebar' : 'Collapse sidebar'}
        >
          <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            {sidebarCollapsed ? (
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13 5l7 7-7 7M5 5l7 7-7 7" />
            ) : (
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M11 19l-7-7 7-7m8 14l-7-7 7-7" />
            )}
          </svg>
        </button>
      </div>

      {/* Navigation */}
      <nav className="flex-1 overflow-y-auto p-2">
        {filterAgent ? (
          // Show skills for selected agent
          <div className="animate-fade-in">
            {/* Agent header */}
            <div className="px-2 py-3 mb-1">
              <div className="flex items-center gap-3">
                <div className={`w-9 h-9 rounded-lg bg-gradient-to-br ${AGENT_CONFIG[getAgentKey(filterAgent)]?.gradient || 'from-gray-500 to-gray-600'} flex items-center justify-center text-white text-sm font-semibold shadow-sm`}>
                  {getAgentKey(filterAgent).charAt(0)}
                </div>
                {!sidebarCollapsed && (
                  <div>
                    <h2 className="text-sm font-semibold text-primary">
                      {agentConfigs.find((c) => getAgentKey(c.agent) === getAgentKey(filterAgent))?.name || getAgentKey(filterAgent)}
                    </h2>
                    <p className="text-xs text-muted">
                      {filteredSkills.length} skill{filteredSkills.length !== 1 ? 's' : ''}
                    </p>
                  </div>
                )}
              </div>
            </div>

            {/* Skills list */}
            <div className="space-y-0.5">
              {filteredSkills.map((skill) => (
                <SkillItem
                  key={skill.id}
                  name={skill.name}
                  fileCount={skill.file_count}
                  isFolder={skill.is_folder}
                  isCollapsed={sidebarCollapsed}
                  onClick={() => selectSkill(skill.id)}
                  isSelected={selectedSkillId === skill.id}
                />
              ))}
              {filteredSkills.length === 0 && !sidebarCollapsed && (
                <div className="px-3 py-8 text-center">
                  <svg className="w-10 h-10 mx-auto mb-2 text-muted opacity-50" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z" />
                  </svg>
                  <p className="text-sm text-muted">No skills found</p>
                </div>
              )}
            </div>
          </div>
        ) : (
          // Show agents list
          <div className="animate-fade-in">
            {/* All Skills option */}
            <button
              onClick={() => handleAgentClick(null)}
              className="w-full flex items-center gap-3 px-3 py-2.5 rounded-lg hover:bg-surface-2 text-secondary hover:text-primary transition-colors mb-1"
            >
              <div className="w-9 h-9 rounded-lg bg-gradient-to-br from-[#58a6ff] to-[#a371f7] flex items-center justify-center shadow-sm">
                <svg className="w-5 h-5 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10" />
                </svg>
              </div>
              {!sidebarCollapsed && (
                <>
                  <div className="flex-1 text-left">
                    <p className="text-sm font-medium">All Skills</p>
                    <p className="text-xs text-muted">{totalSkills} total</p>
                  </div>
                  <svg className="w-4 h-4 text-muted" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 5l7 7-7 7" />
                  </svg>
                </>
              )}
            </button>

            <div className="my-2 mx-3 border-t border-subtle" />

            {/* Agent folders */}
            {agentConfigs
              .filter((config) => config.enabled)
              .map((config) => {
                const agentKey = getAgentKey(config.agent);
                const count = skillCounts[agentKey] || 0;
                const agentConfig = AGENT_CONFIG[agentKey] || { color: 'bg-gray-500', gradient: 'from-gray-500 to-gray-600' };

                return (
                  <button
                    key={agentKey}
                    onClick={() => handleAgentClick(config.agent)}
                    className="w-full flex items-center gap-3 px-3 py-2.5 rounded-lg hover:bg-surface-2 text-secondary hover:text-primary transition-colors"
                    title={sidebarCollapsed ? `${config.name} (${count})` : undefined}
                  >
                    <div className={`w-9 h-9 rounded-lg bg-gradient-to-br ${agentConfig.gradient} flex items-center justify-center shadow-sm`}>
                      <svg className="w-5 h-5 text-white" fill="currentColor" viewBox="0 0 20 20">
                        <path d="M2 6a2 2 0 012-2h5l2 2h5a2 2 0 012 2v6a2 2 0 01-2 2H4a2 2 0 01-2-2V6z" />
                      </svg>
                    </div>
                    {!sidebarCollapsed && (
                      <>
                        <div className="flex-1 text-left">
                          <p className="text-sm font-medium">{config.name}</p>
                          <p className="text-xs text-muted">
                            {count} skill{count !== 1 ? 's' : ''}
                          </p>
                        </div>
                        <svg className="w-4 h-4 text-muted" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 5l7 7-7 7" />
                        </svg>
                      </>
                    )}
                  </button>
                );
              })}
          </div>
        )}
      </nav>

      {/* Footer */}
      <div className="p-2 border-t border-subtle flex-shrink-0">
        <button
          className="w-full flex items-center gap-3 px-3 py-2 rounded-lg hover:bg-surface-2 text-secondary hover:text-primary transition-colors"
          title="Settings"
        >
          <div className="w-8 h-8 rounded-lg bg-surface-2 flex items-center justify-center">
            <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
            </svg>
          </div>
          {!sidebarCollapsed && <span className="text-sm font-medium">Settings</span>}
        </button>
      </div>
    </div>
  );
}

// Skill item component with improved selection states
function SkillItem({
  name,
  fileCount,
  isFolder,
  isCollapsed,
  onClick,
  isSelected,
}: {
  name: string;
  fileCount: number;
  isFolder: boolean;
  isCollapsed: boolean;
  onClick: () => void;
  isSelected: boolean;
}) {
  return (
    <button
      onClick={onClick}
      className={`
        w-full flex items-center gap-2.5 px-3 py-2 rounded-lg transition-all
        ${isSelected
          ? 'bg-[#58a6ff]/12 text-[#58a6ff] border-l-2 border-l-[#58a6ff] pl-[10px]'
          : 'hover:bg-surface-2 text-secondary hover:text-primary border-l-2 border-l-transparent pl-[10px]'
        }
      `}
      title={isCollapsed ? `${name} (${fileCount} files)` : undefined}
    >
      {isFolder ? (
        <svg className={`w-4 h-4 flex-shrink-0 ${isSelected ? 'text-[#d29922]' : 'text-[#d29922]'}`} fill="currentColor" viewBox="0 0 20 20">
          <path d="M2 6a2 2 0 012-2h5l2 2h5a2 2 0 012 2v6a2 2 0 01-2 2H4a2 2 0 01-2-2V6z" />
        </svg>
      ) : (
        <svg className={`w-4 h-4 flex-shrink-0 ${isSelected ? 'text-[#58a6ff]' : 'text-muted'}`} fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
        </svg>
      )}
      {!isCollapsed && (
        <>
          <span className={`flex-1 text-left text-sm truncate ${isSelected ? 'font-medium' : ''}`}>{name}</span>
          <span className={`text-xs ${isSelected ? 'text-[#58a6ff]/70' : 'text-muted'}`}>{fileCount}</span>
        </>
      )}
    </button>
  );
}

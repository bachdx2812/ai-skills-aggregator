import type { Skill, AgentType } from '@/lib/types';

interface SkillCardProps {
  skill: Skill;
  isSelected: boolean;
  onClick: () => void;
}

function getAgentKey(agent: AgentType): string {
  return typeof agent === 'string' ? agent : agent.Custom;
}

export function SkillCard({ skill, isSelected, onClick }: SkillCardProps) {
  const agentKey = getAgentKey(skill.agent);

  return (
    <div
      onClick={onClick}
      className={`p-4 rounded-lg border cursor-pointer transition-all ${
        isSelected
          ? 'border-blue-500 bg-blue-50 dark:bg-blue-900/20 ring-1 ring-blue-500'
          : 'border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800 hover:border-gray-300 dark:hover:border-gray-600 hover:shadow-sm'
      }`}
    >
      <div className="flex items-start justify-between gap-3">
        <div className="flex-1 min-w-0">
          {/* Folder icon and name */}
          <div className="flex items-center gap-2">
            {skill.is_folder ? (
              <svg className="w-5 h-5 text-yellow-500 flex-shrink-0" fill="currentColor" viewBox="0 0 20 20">
                <path d="M2 6a2 2 0 012-2h5l2 2h5a2 2 0 012 2v6a2 2 0 01-2 2H4a2 2 0 01-2-2V6z" />
              </svg>
            ) : (
              <svg className="w-5 h-5 text-gray-400 flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
              </svg>
            )}
            <h4 className="font-medium text-gray-900 dark:text-white truncate">{skill.name}</h4>
          </div>

          {/* Description */}
          {skill.description && (
            <p className="mt-1 text-sm text-gray-600 dark:text-gray-400 line-clamp-2 ml-7">{skill.description}</p>
          )}

          {/* Tags */}
          {skill.tags && skill.tags.length > 0 && (
            <div className="mt-2 ml-7 flex flex-wrap gap-1">
              {skill.tags.slice(0, 3).map((tag) => (
                <span
                  key={tag}
                  className="px-2 py-0.5 text-xs rounded-full bg-gray-100 dark:bg-gray-700 text-gray-600 dark:text-gray-400"
                >
                  {tag}
                </span>
              ))}
              {skill.tags.length > 3 && (
                <span className="px-2 py-0.5 text-xs text-gray-500 dark:text-gray-500">
                  +{skill.tags.length - 3}
                </span>
              )}
            </div>
          )}
        </div>

        {/* Right side badges */}
        <div className="flex flex-col items-end gap-2">
          {/* File count for folders */}
          {skill.is_folder && (
            <span className="px-2 py-0.5 text-xs rounded bg-gray-100 text-gray-600 dark:bg-gray-700 dark:text-gray-400">
              {skill.file_count} file{skill.file_count !== 1 ? 's' : ''}
            </span>
          )}

          {/* Local/Remote indicator */}
          <span
            className={`px-2 py-0.5 text-xs rounded ${
              skill.is_local
                ? 'bg-green-100 text-green-700 dark:bg-green-900/30 dark:text-green-400'
                : 'bg-orange-100 text-orange-700 dark:bg-orange-900/30 dark:text-orange-400'
            }`}
          >
            {skill.is_local ? 'Local' : 'Installed'}
          </span>
        </div>
      </div>

      {/* Footer */}
      <div className="mt-3 pt-3 border-t border-gray-100 dark:border-gray-700 flex items-center justify-between text-xs text-gray-500 dark:text-gray-400">
        <span className="truncate max-w-[200px]" title={skill.folder_path}>
          {skill.folder_path.split('/').pop()}
        </span>
        <span>{agentKey}</span>
      </div>
    </div>
  );
}

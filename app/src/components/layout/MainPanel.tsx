import { useState } from 'react';
import { useSkillsStore } from '@/stores/skills-store';
import { useUIStore } from '@/stores/ui-store';
import type { AgentType, SkillFile, SkillFormat } from '@/lib/types';

function getAgentKey(agent: AgentType): string {
  return typeof agent === 'string' ? agent : agent.Custom;
}

function formatFileSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

const FORMAT_ICONS: Record<SkillFormat, { label: string; class: string }> = {
  Markdown: { label: 'MD', class: 'file-icon-md' },
  Json: { label: '{ }', class: 'file-icon-json' },
  Yaml: { label: 'YML', class: 'file-icon-yaml' },
  Python: { label: 'PY', class: 'file-icon-py' },
  PlainText: { label: 'TXT', class: 'file-icon-txt' },
};

export function MainPanel() {
  const { skills, deleteSkill, createFile } = useSkillsStore();
  const { filterAgent, selectedSkillId, selectedFileId, setSelectedFile, selectSkill } = useUIStore();
  const [showDeleteConfirm, setShowDeleteConfirm] = useState(false);
  const [showAddFile, setShowAddFile] = useState(false);
  const [newFileName, setNewFileName] = useState('');
  const [isDeleting, setIsDeleting] = useState(false);
  const [isCreating, setIsCreating] = useState(false);

  const selectedSkill = skills.find((s) => s.id === selectedSkillId);

  const handleDeleteSkill = async () => {
    if (!selectedSkill) return;
    setIsDeleting(true);
    try {
      await deleteSkill(selectedSkill.folder_path);
      selectSkill(null);
      setShowDeleteConfirm(false);
    } catch (error) {
      alert(`Failed to delete skill: ${error}`);
    } finally {
      setIsDeleting(false);
    }
  };

  const handleAddFile = async () => {
    if (!selectedSkill || !newFileName.trim()) return;
    setIsCreating(true);
    try {
      const fileName = newFileName.endsWith('.md') ? newFileName : `${newFileName}.md`;
      const newFile = await createFile(selectedSkill.folder_path, fileName, '# New File\n\nAdd your content here.\n');
      setShowAddFile(false);
      setNewFileName('');
      // Select the newly created file to open it
      if (newFile?.file_path) {
        setSelectedFile(newFile.file_path);
      }
    } catch (error) {
      alert(`Failed to create file: ${error}`);
    } finally {
      setIsCreating(false);
    }
  };

  // Show welcome state when no skill is selected
  if (!selectedSkill) {
    return (
      <div className="h-full bg-surface-0 flex flex-col items-center justify-center p-8">
        <div className="text-center max-w-md animate-fade-in">
          {/* Logo / Icon */}
          <div className="w-16 h-16 mx-auto mb-6 rounded-2xl bg-gradient-to-br from-[#58a6ff] to-[#a371f7] flex items-center justify-center shadow-lg">
            <svg className="w-8 h-8 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M9.663 17h4.673M12 3v1m6.364 1.636l-.707.707M21 12h-1M4 12H3m3.343-5.657l-.707-.707m2.828 9.9a5 5 0 117.072 0l-.548.547A3.374 3.374 0 0014 18.469V19a2 2 0 11-4 0v-.531c0-.895-.356-1.754-.988-2.386l-.548-.547z" />
            </svg>
          </div>

          <h2 className="text-xl font-semibold text-primary mb-2">
            AI Skills Manager
          </h2>

          <p className="text-secondary text-sm mb-8">
            {filterAgent
              ? 'Select a skill from the sidebar to view its files.'
              : 'Select an agent from the sidebar to browse skills.'}
          </p>

          {/* Agent Quick Access */}
          <div className="grid grid-cols-2 gap-3 text-left">
            <AgentCard
              name="Claude Code"
              shortName="C"
              color="from-orange-500 to-orange-600"
              count={skills.filter((s) => s.agent === 'Claude').length}
              onClick={() => useUIStore.getState().setFilterAgent('Claude')}
            />
            <AgentCard
              name="Cursor"
              shortName="Cu"
              color="from-purple-500 to-purple-600"
              count={skills.filter((s) => s.agent === 'Cursor').length}
              onClick={() => useUIStore.getState().setFilterAgent('Cursor')}
            />
          </div>

          <p className="mt-8 text-xs text-muted">
            {skills.length} skills loaded
          </p>
        </div>
      </div>
    );
  }

  // Show skill details and file browser
  return (
    <div className="h-full bg-surface-0 flex flex-col">
      {/* Skill Header */}
      <div className="p-5 bg-surface-1 border-b border-subtle">
        <div className="flex items-start gap-4">
          {/* Skill Icon */}
          <div className="w-12 h-12 rounded-xl bg-[#d29922]/15 flex items-center justify-center flex-shrink-0">
            <svg className="w-6 h-6 text-[#d29922]" fill="currentColor" viewBox="0 0 20 20">
              <path d="M2 6a2 2 0 012-2h5l2 2h5a2 2 0 012 2v6a2 2 0 01-2 2H4a2 2 0 01-2-2V6z" />
            </svg>
          </div>

          <div className="flex-1 min-w-0">
            <h1 className="text-lg font-semibold text-primary truncate">{selectedSkill.name}</h1>
            <p className="text-sm text-muted mt-0.5">{getAgentKey(selectedSkill.agent)}</p>

            {/* Metadata badges */}
            <div className="mt-3 flex flex-wrap gap-2">
              <span className={`badge ${selectedSkill.is_local ? 'badge-green' : 'badge-orange'}`}>
                {selectedSkill.is_local ? 'Local' : 'Installed'}
              </span>
              <span className="badge badge-default">
                {selectedSkill.file_count} file{selectedSkill.file_count !== 1 ? 's' : ''}
              </span>
              {selectedSkill.version && (
                <span className="badge badge-blue">
                  v{selectedSkill.version}
                </span>
              )}
            </div>
          </div>
        </div>

        {/* Description */}
        {selectedSkill.description && (
          <p className="mt-4 text-sm text-secondary leading-relaxed">{selectedSkill.description}</p>
        )}

        {/* Path */}
        <p className="mt-3 text-xs text-muted font-mono truncate" title={selectedSkill.folder_path}>
          {selectedSkill.folder_path}
        </p>
      </div>

      {/* File Browser */}
      <div className="flex-1 overflow-auto p-4">
        <div className="flex items-center justify-between mb-3 px-1">
          <h3 className="text-xs font-medium text-muted uppercase tracking-wide">
            Files ({selectedSkill.file_count})
          </h3>
        </div>

        <div className="space-y-1.5">
          {selectedSkill.files.map((file) => (
            <FileItem
              key={file.file_path}
              file={file}
              isSelected={selectedFileId === file.file_path}
              onClick={() => setSelectedFile(file.file_path)}
            />
          ))}
        </div>

        {selectedSkill.files.length === 0 && (
          <div className="text-center py-12 text-muted">
            <svg className="w-12 h-12 mx-auto mb-3 opacity-50" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z" />
            </svg>
            <p className="text-sm">No files in this skill folder</p>
          </div>
        )}
      </div>

      {/* Footer Actions */}
      <div className="p-4 bg-surface-1 border-t border-subtle flex items-center justify-between">
        <button
          onClick={() => setShowDeleteConfirm(true)}
          className="btn btn-danger text-sm"
          disabled={!selectedSkill.is_local}
          title={!selectedSkill.is_local ? 'Cannot delete installed skills' : undefined}
        >
          <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
          </svg>
          Delete
        </button>
        <button
          onClick={() => setShowAddFile(true)}
          className="btn btn-primary text-sm"
          disabled={!selectedSkill.is_local}
          title={!selectedSkill.is_local ? 'Cannot modify installed skills' : undefined}
        >
          <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 4v16m8-8H4" />
          </svg>
          Add File
        </button>
      </div>

      {/* Delete Confirmation Dialog */}
      {showDeleteConfirm && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
          <div className="bg-surface-1 rounded-xl p-6 w-96 shadow-xl border border-subtle">
            <h3 className="text-lg font-semibold text-primary mb-2">Delete Skill?</h3>
            <p className="text-sm text-secondary mb-4">
              Are you sure you want to delete <strong>{selectedSkill.name}</strong>? This action cannot be undone.
            </p>
            <div className="flex justify-end gap-3">
              <button
                onClick={() => setShowDeleteConfirm(false)}
                className="btn btn-secondary text-sm"
                disabled={isDeleting}
              >
                Cancel
              </button>
              <button
                onClick={handleDeleteSkill}
                className="btn btn-danger text-sm"
                disabled={isDeleting}
              >
                {isDeleting ? 'Deleting...' : 'Delete'}
              </button>
            </div>
          </div>
        </div>
      )}

      {/* Add File Dialog */}
      {showAddFile && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
          <div className="bg-surface-1 rounded-xl p-6 w-96 shadow-xl border border-subtle">
            <h3 className="text-lg font-semibold text-primary mb-4">Add New File</h3>
            <input
              type="text"
              value={newFileName}
              onChange={(e) => setNewFileName(e.target.value)}
              placeholder="filename.md"
              className="w-full px-3 py-2 bg-surface-0 border border-subtle rounded-lg text-primary text-sm focus:outline-none focus:border-[#58a6ff] focus:ring-1 focus:ring-[#58a6ff]/50"
              autoFocus
              onKeyDown={(e) => {
                if (e.key === 'Enter' && newFileName.trim()) handleAddFile();
                if (e.key === 'Escape') setShowAddFile(false);
              }}
            />
            <div className="flex justify-end gap-3 mt-4">
              <button
                onClick={() => { setShowAddFile(false); setNewFileName(''); }}
                className="btn btn-secondary text-sm"
                disabled={isCreating}
              >
                Cancel
              </button>
              <button
                onClick={handleAddFile}
                className="btn btn-primary text-sm"
                disabled={isCreating || !newFileName.trim()}
              >
                {isCreating ? 'Creating...' : 'Create'}
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}

// Agent card component for welcome screen
function AgentCard({
  name,
  shortName,
  color,
  count,
  onClick,
}: {
  name: string;
  shortName: string;
  color: string;
  count: number;
  onClick: () => void;
}) {
  return (
    <button
      onClick={onClick}
      className="group p-4 bg-surface-1 rounded-xl border border-subtle hover:border-[var(--border-default)] hover:bg-surface-2 transition-all text-left"
    >
      <div className={`w-10 h-10 rounded-lg bg-gradient-to-br ${color} flex items-center justify-center mb-3 shadow-sm group-hover:scale-105 transition-transform`}>
        <span className="text-white font-semibold text-sm">{shortName}</span>
      </div>
      <p className="text-sm font-medium text-primary">{name}</p>
      <p className="text-xs text-muted mt-0.5">
        {count} skill{count !== 1 ? 's' : ''}
      </p>
    </button>
  );
}

// File item component with improved selection states
function FileItem({
  file,
  isSelected,
  onClick,
}: {
  file: SkillFile;
  isSelected: boolean;
  onClick: () => void;
}) {
  const icon = FORMAT_ICONS[file.format] || FORMAT_ICONS.PlainText;

  return (
    <button
      onClick={onClick}
      className={`
        w-full flex items-center gap-3 p-3 rounded-lg transition-all
        border text-left group
        ${isSelected
          ? 'bg-[#58a6ff]/10 border-[#58a6ff]/40 shadow-sm'
          : 'bg-surface-1 border-subtle hover:bg-surface-2 hover:border-[var(--border-default)]'
        }
      `}
    >
      {/* File type icon */}
      <div className={`file-icon file-icon-lg ${icon.class}`}>
        {icon.label}
      </div>

      {/* File info */}
      <div className="flex-1 min-w-0">
        <p className={`text-sm font-medium truncate ${
          isSelected ? 'text-[#58a6ff]' : 'text-primary'
        }`}>
          {file.name}
        </p>
        <div className="flex items-center gap-2 mt-0.5">
          <span className="text-xs text-muted">
            {formatFileSize(file.size)}
          </span>
          {file.is_entry && (
            <span className="text-xs text-[#58a6ff] flex items-center gap-1">
              <svg className="w-3 h-3" fill="currentColor" viewBox="0 0 20 20">
                <path fillRule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z" clipRule="evenodd" />
              </svg>
              Entry
            </span>
          )}
        </div>
      </div>

      {/* Arrow indicator */}
      <svg
        className={`w-4 h-4 flex-shrink-0 transition-transform group-hover:translate-x-0.5 ${
          isSelected ? 'text-[#58a6ff]' : 'text-muted'
        }`}
        fill="none"
        stroke="currentColor"
        viewBox="0 0 24 24"
      >
        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 5l7 7-7 7" />
      </svg>
    </button>
  );
}

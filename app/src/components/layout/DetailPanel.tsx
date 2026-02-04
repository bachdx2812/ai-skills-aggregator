import { useEffect, useState } from 'react';
import { useSkillsStore } from '@/stores/skills-store';
import { useUIStore } from '@/stores/ui-store';
import { SkillEditor } from '@/components/skills/SkillEditor';
import { api } from '@/lib/api';
import type { SkillFormat } from '@/lib/types';

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

export function DetailPanel() {
  const { skills, updateSkill } = useSkillsStore();
  const { selectedSkillId, selectedFileId, setSelectedFile } = useUIStore();

  const [content, setContent] = useState('');
  const [originalContent, setOriginalContent] = useState('');
  const [isLoading, setIsLoading] = useState(false);
  const [isSaving, setIsSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const selectedSkill = skills.find((s) => s.id === selectedSkillId);
  const selectedFile = selectedSkill?.files.find((f) => f.file_path === selectedFileId);
  const hasChanges = content !== originalContent;

  // Load file content when selection changes
  useEffect(() => {
    if (selectedFile) {
      setIsLoading(true);
      setError(null);
      api.skills
        .readContent(selectedFile.file_path)
        .then((c) => {
          setContent(c);
          setOriginalContent(c);
        })
        .catch((err) => setError(err.toString()))
        .finally(() => setIsLoading(false));
    } else {
      setContent('');
      setOriginalContent('');
    }
  }, [selectedFile?.file_path]);

  const handleSave = async () => {
    if (!selectedFile || !hasChanges) return;

    setIsSaving(true);
    setError(null);

    try {
      await updateSkill(selectedFile.file_path, content);
      setOriginalContent(content);
    } catch (err) {
      setError((err as Error).message);
    } finally {
      setIsSaving(false);
    }
  };

  const handleClose = () => {
    if (hasChanges) {
      if (!window.confirm('You have unsaved changes. Discard them?')) {
        return;
      }
    }
    setSelectedFile(null);
  };

  const handleRevert = () => {
    if (window.confirm('Revert all changes?')) {
      setContent(originalContent);
    }
  };

  // No file selected - show empty state
  if (!selectedFile) {
    return (
      <div className="h-full flex flex-col items-center justify-center bg-surface-0 p-8">
        <div className="text-center animate-fade-in">
          <div className="w-16 h-16 mx-auto mb-4 rounded-2xl bg-surface-2 flex items-center justify-center">
            <svg className="w-8 h-8 text-muted" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={1.5}
                d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"
              />
            </svg>
          </div>
          <p className="text-base font-medium text-secondary">No file selected</p>
          <p className="text-sm text-muted mt-1">Select a file to view and edit</p>
        </div>
      </div>
    );
  }

  const icon = FORMAT_ICONS[selectedFile.format] || FORMAT_ICONS.PlainText;

  return (
    <div className="h-full flex flex-col bg-surface-0">
      {/* File Header */}
      <div className="h-14 px-4 border-b border-subtle flex items-center justify-between flex-shrink-0 bg-surface-1">
        <div className="flex items-center gap-3 min-w-0">
          <div className={`file-icon ${icon.class}`}>
            {icon.label}
          </div>
          <div className="min-w-0">
            <div className="flex items-center gap-2">
              <h2 className="font-medium text-primary text-sm truncate">{selectedFile.name}</h2>
              {hasChanges && (
                <span className="w-2 h-2 rounded-full bg-[#d29922] flex-shrink-0" title="Unsaved changes" />
              )}
            </div>
            <div className="flex items-center gap-2 text-xs text-muted">
              <span>{formatFileSize(selectedFile.size)}</span>
              {selectedFile.is_entry && (
                <>
                  <span className="text-muted/50">|</span>
                  <span className="text-[#58a6ff]">Entry file</span>
                </>
              )}
              {!selectedSkill?.is_local && (
                <>
                  <span className="text-muted/50">|</span>
                  <span className="text-[#d29922]">Read-only</span>
                </>
              )}
            </div>
          </div>
        </div>

        <button
          onClick={handleClose}
          className="p-1.5 rounded-md hover:bg-surface-2 text-muted hover:text-secondary transition-colors"
          title="Close file"
        >
          <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
      </div>

      {/* Editor Area */}
      <div className="flex-1 overflow-hidden">
        {isLoading ? (
          <div className="h-full flex items-center justify-center">
            <div className="text-center">
              <div className="spinner mx-auto mb-3" />
              <p className="text-sm text-muted">Loading file...</p>
            </div>
          </div>
        ) : error ? (
          <div className="h-full flex flex-col items-center justify-center p-8">
            <div className="w-14 h-14 rounded-2xl bg-[#f85149]/10 flex items-center justify-center mb-4">
              <svg className="w-7 h-7 text-[#f85149]" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth={2}
                  d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z"
                />
              </svg>
            </div>
            <p className="text-sm font-medium text-[#f85149] mb-1">Failed to load file</p>
            <p className="text-xs text-muted text-center max-w-xs">{error}</p>
            <button
              onClick={() => window.location.reload()}
              className="mt-4 btn btn-secondary text-xs"
            >
              Retry
            </button>
          </div>
        ) : (
          <SkillEditor
            content={content}
            onChange={setContent}
            format={selectedFile.format}
            readOnly={!selectedSkill?.is_local}
          />
        )}
      </div>

      {/* Footer Actions */}
      <div className="h-12 px-4 border-t border-subtle flex items-center justify-between flex-shrink-0 bg-surface-1">
        {/* Left side - file info */}
        <div className="flex items-center gap-3 text-xs text-muted">
          <span>{selectedFile.format}</span>
          {hasChanges && <span className="text-[#d29922]">Modified</span>}
        </div>

        {/* Right side - actions */}
        <div className="flex items-center gap-2">
          {hasChanges && (
            <button
              onClick={handleRevert}
              className="btn btn-ghost text-xs px-3 py-1.5"
            >
              Discard
            </button>
          )}
          <button
            onClick={handleSave}
            disabled={!hasChanges || isSaving || !selectedSkill?.is_local}
            className={`btn text-xs px-4 py-1.5 ${
              hasChanges && selectedSkill?.is_local
                ? 'btn-primary'
                : 'bg-surface-2 text-muted cursor-not-allowed'
            }`}
          >
            {isSaving ? (
              <>
                <div className="w-3 h-3 border border-white/30 border-t-white rounded-full animate-spin" />
                Saving...
              </>
            ) : (
              <>
                <svg className="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 13l4 4L19 7" />
                </svg>
                Save
              </>
            )}
          </button>
        </div>
      </div>
    </div>
  );
}

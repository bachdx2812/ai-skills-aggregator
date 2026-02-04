import { useEffect, useRef } from 'react';
import { EditorView, basicSetup } from 'codemirror';
import { EditorState } from '@codemirror/state';
import { markdown } from '@codemirror/lang-markdown';
import { json } from '@codemirror/lang-json';
import { yaml } from '@codemirror/lang-yaml';
import { python } from '@codemirror/lang-python';
import { oneDark } from '@codemirror/theme-one-dark';
import type { SkillFormat } from '@/lib/types';

interface SkillEditorProps {
  content: string;
  onChange: (content: string) => void;
  format: SkillFormat;
  readOnly?: boolean;
}

function getLanguageExtension(format: SkillFormat) {
  switch (format) {
    case 'Markdown':
      return markdown();
    case 'Json':
      return json();
    case 'Yaml':
      return yaml();
    case 'Python':
      return python();
    default:
      return [];
  }
}

export function SkillEditor({ content, onChange, format, readOnly = false }: SkillEditorProps) {
  const editorRef = useRef<HTMLDivElement>(null);
  const viewRef = useRef<EditorView | null>(null);

  // Check if dark mode is active
  const isDarkMode = document.documentElement.classList.contains('dark');

  useEffect(() => {
    if (!editorRef.current) return;

    // Clean up previous editor
    if (viewRef.current) {
      viewRef.current.destroy();
    }

    const extensions = [
      basicSetup,
      getLanguageExtension(format),
      EditorView.updateListener.of((update) => {
        if (update.docChanged) {
          onChange(update.state.doc.toString());
        }
      }),
      EditorView.lineWrapping,
      EditorState.readOnly.of(readOnly),
    ];

    // Add dark theme if in dark mode
    if (isDarkMode) {
      extensions.push(oneDark);
    }

    // Add custom styling
    extensions.push(
      EditorView.theme({
        '&': {
          height: '100%',
          fontSize: '14px',
        },
        '.cm-scroller': {
          fontFamily: 'ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace',
          overflow: 'auto',
        },
        '.cm-content': {
          padding: '16px 0',
        },
        '.cm-line': {
          padding: '0 16px',
        },
      })
    );

    const state = EditorState.create({
      doc: content,
      extensions,
    });

    viewRef.current = new EditorView({
      state,
      parent: editorRef.current,
    });

    return () => {
      viewRef.current?.destroy();
    };
  }, [format, isDarkMode, readOnly]);

  // Update content when it changes externally
  useEffect(() => {
    if (viewRef.current && content !== viewRef.current.state.doc.toString()) {
      viewRef.current.dispatch({
        changes: {
          from: 0,
          to: viewRef.current.state.doc.length,
          insert: content,
        },
      });
    }
  }, [content]);

  return (
    <div
      ref={editorRef}
      className={`h-full overflow-hidden ${readOnly ? 'opacity-75' : ''}`}
      style={{ backgroundColor: isDarkMode ? '#282c34' : '#ffffff' }}
    />
  );
}

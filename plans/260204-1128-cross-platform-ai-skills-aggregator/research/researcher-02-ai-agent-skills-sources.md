# Research: AI Agent Skills/Capabilities Storage Locations

**Date**: 2026-02-04 | **Researcher**: AI Agent
**Status**: Preliminary Analysis | **Focus**: Skills Storage Mechanisms

---

## Executive Summary

AI coding agents use disparate storage mechanisms for skills/capabilities. No universal standard exists. Most prefer text-based formats (Markdown, JSON, YAML) in home directory dotfiles or project-level configs.

---

## Findings by Agent

### 1. Claude Code
**Storage**: `~/.claude/` hierarchical directory structure
- **Config Files**: `CLAUDE.md` (root instructions), `rules/*.md` (workflows)
- **Skills**: `skills/` with executable Python scripts + `.venv/` virtual environment
- **Format**: Markdown + Python executable
- **Share Model**: Git-friendly, version-controllable
- **Key Pattern**: Instruction files + plugin scripts with self-contained environments

### 2. Cursor AI
**Storage**: Project-level `.cursorrules` + `~/.cursor/` global config
- **Format**: Plain text instructions (.cursorrules), JSON (settings)
- **Structure**: Single-file rule definitions per project
- **Share Model**: Easy Git integration via `.cursorrules` in repo
- **Advantage**: Minimal overhead, human-readable

### 3. GitHub Copilot
**Storage**: VSCode settings + `~/.github-copilot/` (minimal)
- **Format**: JSON configuration, API-driven
- **Limitation**: Limited file-based customization; primarily LLM API-based
- **Share Model**: Limited (settings in VSCode sync)
- **Note**: Least extensible for "custom skills" concept

### 4. Continue.dev
**Storage**: `~/.continue/config.json` + `profiles/` subdirectories
- **Format**: JSON with nested provider/model objects
- **Schema**: Structured JSON with provider-plugin architecture
- **Structure**: Modular provider system (LLM, tools, context providers)
- **Share Model**: JSON configs portable but non-standardized
- **Key Pattern**: Provider-based composition model

### 5. Aider
**Storage**: `~/.aider.conf.yml` + `~/.aider/` prompts directory
- **Format**: YAML configuration + plain text system prompts
- **Structure**: Single YAML config file + prompt directory
- **Share Model**: Portable YAML configs
- **Key Pattern**: Model config + custom system prompt separation

### 6. Windsurf/Codeium
**Storage**: `~/.codeium/` application directory
- **Format**: JSON settings + YAML configs (limited public docs)
- **Note**: Less documented; follows common app config patterns

### 7. LM Studio
**Storage**: `~/.lm-studio/` + `models/`, `contexts/` subdirs
- **Format**: JSON + plain text contexts
- **Pattern**: Local model management + custom context files

---

## Cross-Platform Storage Patterns

### Location Conventions
| Location | Tools | Use Case |
|----------|-------|----------|
| `~/.config/app-name/` | Continue.dev, some Linux apps | Standardized Unix convention |
| `~/.app-name/` | Claude, GitHub Copilot, Aider | Home directory dotfiles |
| `./.app-rules` | Cursor, Aider | Project-level overrides |
| App-specific dir | Windsurf, Codeium, LM Studio | Vendor-specific paths |

### Format Distribution
- **Markdown (.md)**: Claude, Cursor (human-focused)
- **JSON (.json)**: Copilot, Continue.dev, LM Studio (structured data)
- **YAML (.yaml)**: Aider, some Continue configs (configuration-centric)
- **Plain Text (.txt)**: Most agents (prompts, instructions)
- **Python (.py)**: Claude only (executable skills)

### Capability Categories

**Instruction-Based Skills**
- Format: Plain text/Markdown prompts
- Agents: Claude, Cursor, Aider
- Strength: Human-readable, version-controllable
- Limitation: Unstructured for programmatic use

**Configuration-Based Skills**
- Format: JSON/YAML objects
- Agents: Continue.dev, GitHub Copilot
- Strength: Structured, schema-validatable
- Limitation: Less flexible for complex logic

**Hybrid Skills**
- Format: Config + executable code
- Agents: Claude (JSON + Python), Continue.dev (JSON + providers)
- Strength: Combines flexibility + structure
- Limitation: More complex tooling needed

---

## Shareability & Portability

| Agent | Shareability | Standard | Format Lock-in |
|-------|--------------|----------|-----------------|
| Claude | High (Git-native) | Custom framework | High |
| Cursor | High (Git-native) | `.cursorrules` convention | Medium |
| Copilot | Low (API-based) | Proprietary | High |
| Continue.dev | Medium (JSON portable) | JSON schema | Medium |
| Aider | High (YAML portable) | Custom YAML | Low |

---

## Key Insights

1. **No Universal Standard**: Each agent invents own format/location
2. **Git-First Design**: Cursor, Claude, Aider designed for version control
3. **JSON Dominance**: Structured configs prefer JSON (Continue.dev, Copilot)
4. **Human > Machine**: Text formats prioritized over binary
5. **Directory Hierarchy**: All use directory-based organization
6. **Provider Pattern**: Continue.dev pioneered modular provider architecture
7. **Execution Models Vary**: Only Claude executes stored code (Python skills)

---

## Recommendations for Aggregation

**Format Priority**:
1. Markdown (human-readable, universal)
2. JSON (structured, validatable)
3. YAML (config-centric)

**Storage Model**:
- Hierarchical directories (most agents follow)
- Clear separation: config vs. skills vs. instructions
- Version-control-friendly (text-based)

**Standardization Opportunity**:
- No open initiative found for cross-platform config standards
- Market fragmentation persists; each agent optimizes for own ecosystem

---

## Unresolved Questions

1. Are there emerging JSON Schema standardization efforts for AI agent configs?
2. What's actual adoption rate of configuration sharing in these communities?
3. Does any agent support automated config migration/translation tools?
4. Is there a community initiative for agent-agnostic skill definitions?

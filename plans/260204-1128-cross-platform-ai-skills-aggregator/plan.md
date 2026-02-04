---
title: "Cross-Platform AI Skills Aggregator"
description: "Desktop app to discover, manage, and sync AI agent skills across Claude, Cursor, Continue.dev, Aider, and more"
status: pending
priority: P1
effort: 48h
branch: main
tags: [tauri, rust, react, typescript, ai-skills, cross-platform, github-oauth]
created: 2026-02-04
---

# Cross-Platform AI Skills Aggregator

## Overview
Desktop application for aggregating, managing, and synchronizing AI coding agent skills/configs across multiple platforms (Claude Code, Cursor, Continue.dev, Aider, Windsurf).

**Tech Stack**: Tauri 2.x (Rust backend) + React + TypeScript + TailwindCSS

## NEW: Community Features
- **Local skill creation**: Create and manage skills locally
- **GitHub OAuth login**: Optional authentication (app works without login)
- **Publish to registry**: Share skills publicly via community registry (requires login)
- **Author attribution**: Published skills include GitHub username

## Key Research Insights
- No universal config standard exists; each agent uses own format (MD, JSON, YAML)
- Storage locations: `~/.claude/`, `~/.cursor/`, `~/.continue/`, `~/.aider/`, etc.
- Git-friendly text formats prioritized by most agents
- Only Claude executes stored code (Python skills); others use instructions/configs

## Phase Overview

| Phase | Title | Effort | Status |
|-------|-------|--------|--------|
| 01 | [Project Setup](./phase-01-project-setup.md) | 3h | pending |
| 02 | [Core Architecture](./phase-02-core-architecture.md) | 6h | pending |
| 03 | [Local Skills Discovery](./phase-03-local-skills-discovery.md) | 6h | pending |
| 04 | [Remote Skills Registry](./phase-04-remote-skills-registry.md) | 7h | pending |
| 05 | [Skills CRUD Operations](./phase-05-skills-crud-operations.md) | 6h | pending |
| 06 | [UI Implementation](./phase-06-ui-implementation.md) | 10h | pending |
| 07 | [Update System](./phase-07-update-system.md) | 4h | pending |
| 08 | [Testing & Deployment](./phase-08-testing-deployment.md) | 3h | pending |
| 09 | [GitHub Authentication](./phase-09-github-authentication.md) | 4h | pending |

**Total Effort**: 48h (increased from 40h for auth + publish features)

## Critical Dependencies
1. Tauri CLI + Rust toolchain installed
2. Node.js 20+ for React frontend
3. Platform-specific build tools (Xcode/MSVC/GCC)

## Success Criteria
- [ ] Scans and displays skills from 5+ AI agents
- [ ] CRUD operations work for all supported formats
- [ ] Remote skill install/update functional
- [ ] Cross-platform builds (macOS, Linux, Windows)
- [ ] <50MB bundle size
- [ ] **App works fully without GitHub login**
- [ ] **GitHub OAuth login works (PKCE flow)**
- [ ] **Local skill creation with metadata**
- [ ] **Publish skill to registry (requires login)**

## Risk Summary
- **High**: Format fragmentation requires robust parsers
- **Medium**: Tauri Rust learning curve
- **Medium**: OAuth PKCE flow complexity for desktop
- **Low**: Cross-platform file path handling
- **Low**: Keychain access varies by OS

## Reports
- [Framework Research](./research/researcher-01-cross-platform-frameworks.md)
- [AI Agent Skills Sources](./research/researcher-02-ai-agent-skills-sources.md)

## Validation Summary

**Validated:** 2026-02-04
**Questions asked:** 6

### Confirmed Decisions
- **Framework**: Tauri (Rust + React) - proceed with plan as-is
- **Registry**: GitHub-based registry (skills as files in repos, no backend needed)
- **MVP Scope**: Full scope (all 9 phases, 48h estimate)
- **Rust Experience**: None - expect extended Phase 1-2 for learning curve
- **Agent Priority**: Claude Code + Cursor first, others added incrementally
- **Registry Config**: Configurable (users can add custom registries)

### Action Items
- [ ] Add Rust learning resources to Phase 01 (beginner-friendly docs)
- [ ] Adjust Phase 03 to prioritize Claude + Cursor parsers, defer others
- [ ] Update Phase 04 to emphasize GitHub-based registry (no custom API backend needed)
- [ ] Add registry URL settings UI to Phase 06

### Risk Adjustment
- **Rust learning curve**: Elevated to HIGH risk (no prior experience)
  - Mitigation: Use Tauri starter templates, follow official docs closely
  - Consider: Pair programming or Rust mentorship if blocked

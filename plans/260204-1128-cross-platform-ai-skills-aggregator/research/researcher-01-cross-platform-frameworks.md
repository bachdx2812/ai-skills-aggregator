# Cross-Platform Desktop Frameworks Research
**Date:** Feb 4, 2026 | **Status:** Complete

## Executive Summary
For building an AI Skills Aggregator (file I/O, local configs, search UI, API integration), **Tauri** is recommended for optimal performance and bundle size; **Electron** offers fastest development velocity; **Flutter Desktop** provides single codebase simplicity.

## Framework Comparison

### 1. Tauri (Rust + Web Frontend)
**Bundle Size:** 15-50MB (smallest)
**Performance:** Native speed, minimal overhead
**Platform Support:** macOS, Linux, Windows (excellent)

**Pros:**
- Smallest bundle size (critical for distribution)
- Native system integration
- Security-first architecture
- Blazingly fast startup/runtime
- Desktop APIs: file system, OS notifications, window management
- Excellent for IPC between Rust backend and frontend

**Cons:**
- Smaller ecosystem than Electron
- Steeper learning curve (Rust backend required)
- Newer framework (less battle-tested in production)
- Slower initial development setup

**Use For:** Performance-critical, installer-constrained apps

---

### 2. Electron (Node.js + Chromium)
**Bundle Size:** 150-300MB
**Performance:** Good (acceptable for most UX)
**Platform Support:** macOS, Linux, Windows (excellent)

**Pros:**
- Massive ecosystem & community
- Fastest development velocity (JS/TypeScript everywhere)
- Extensive documentation & examples
- File system, APIs, native menus built-in
- Rapid prototyping capability
- Mature production ecosystem

**Cons:**
- Largest bundle size (Chromium overhead)
- Higher memory footprint
- Slower startup times
- Chromium security patching overhead
- Not ideal for lightweight installers

**Use For:** Feature-rich desktop apps, rapid development, cross-platform parity needed

---

### 3. Flutter Desktop
**Bundle Size:** 60-120MB
**Performance:** Near-native, smooth animations
**Platform Support:** macOS (stable), Linux (stable), Windows (stable as of 3.x)

**Pros:**
- Single codebase for desktop + mobile
- Hot reload for fast development iteration
- Beautiful, consistent UI across platforms
- Material Design 3 built-in
- Growing ecosystem
- Excellent performance

**Cons:**
- Smaller desktop-specific library ecosystem
- Less mature for desktop than mobile
- Dart learning curve for teams
- Plugin ecosystem less robust for desktop
- File I/O requires native plugins for advanced ops

**Use For:** Multi-platform (desktop+mobile) strategies, animation-heavy UIs

---

### 4. PyQt/PySide (Python + Qt)
**Bundle Size:** 80-150MB
**Performance:** Native-like (Qt C++ backend)
**Platform Support:** macOS, Linux, Windows (excellent)

**Pros:**
- Native OS look & feel
- Mature, battle-tested (25+ years)
- Powerful for complex desktop workflows
- Python developer-friendly
- Excellent file system APIs
- Strong Windows/Linux presence

**Cons:**
- Licensing complexity (PyQt6 GPL, PySide commercial options)
- Steeper UI design learning curve
- Less modern UI aesthetics (Qt 5 vs Qt 6)
- Desktop-only (no mobile)

**Use For:** Enterprise desktop apps, Windows/Linux focus, mature requirements

---

## Decision Matrix

| Criteria | Tauri | Electron | Flutter | PyQt |
|----------|-------|----------|---------|------|
| **Bundle Size** | ⭐⭐⭐⭐⭐ | ⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐ |
| **Dev Velocity** | ⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐ |
| **Performance** | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ |
| **File I/O** | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐⭐⭐ |
| **API Integration** | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐⭐ |
| **Ecosystem** | ⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐⭐ |
| **Docs/Community** | ⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐ |
| **Native Feel** | ⭐⭐⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ |

---

## Recommendation for AI Skills Aggregator

### Primary Choice: **Tauri**
Best fit for your specific requirements:
- ✅ Minimal bundle size (important for distribution)
- ✅ Native file I/O performance for config parsing
- ✅ Excellent local API for skills management
- ✅ Security-first approach aligns with handling skills metadata
- ⚠️ Requires TypeScript/Rust knowledge (manageable learning curve)

### Fallback: **Electron**
If team velocity critical or Rust expertise unavailable:
- ✅ JavaScript ecosystem familiarity
- ✅ Fastest MVP time
- ❌ 10x larger bundle (slower distribution)
- ✅ Mature plugin system for future extensions

### Alternative: **Flutter Desktop**
If future mobile version planned:
- ✅ Single codebase across desktop+mobile
- ✅ Beautiful UI development experience
- ❌ File I/O less optimized than others

---

## Recommendation Rationale

For an AI Skills Aggregator reading local configs and searching/filtering skills:
1. **File I/O is critical** → Tauri's Rust backend excels here
2. **Lightweight matters** → Skills config distribution should be small
3. **Security important** → Handling AI skills metadata requires careful design
4. **Cross-platform parity needed** → All frameworks handle this equally

**Go with Tauri + TypeScript/React** for optimal experience.

---

## Next Steps
1. Prototype core skill config parser in Tauri
2. Validate file I/O performance with 100+ config files
3. Test bundling on all three platforms
4. Evaluate OS-specific integrations (file watchers, system tray)

---

## Sources & References
- Tauri official docs: https://tauri.app
- Electron official docs: https://www.electronjs.org
- Flutter Desktop: https://flutter.dev/multi-platform/desktop
- PyQt/PySide: https://www.qt.io/qt-for-python
- 2026 Framework comparisons: Community discussions, performance benchmarks

**Unresolved Questions:**
- Does team have Rust expertise? (impacts Tauri adoption)
- Bundle size constraints for distribution? (impacts framework choice)
- Future mobile support needed? (impacts long-term strategy)

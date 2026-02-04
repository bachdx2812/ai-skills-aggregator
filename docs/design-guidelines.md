# Design Guidelines - AI Skills Aggregator

## Design Philosophy
Professional, minimal, developer-focused aesthetic inspired by VS Code, Linear, and Figma.

## Color Palette

### Dark Mode (Primary)
```
Background Layers:
- bg-surface-0: #0d1117 (darkest - main app bg)
- bg-surface-1: #161b22 (panels, cards)
- bg-surface-2: #21262d (elevated elements, hover states)
- bg-surface-3: #30363d (active states, inputs)

Text Colors:
- text-primary: #e6edf3 (headings, important text)
- text-secondary: #8b949e (body text)
- text-muted: #6e7681 (hints, metadata)

Border Colors:
- border-default: #30363d
- border-subtle: #21262d
- border-emphasis: #8b949e

Accent Colors:
- accent-blue: #58a6ff (primary actions, links, selection)
- accent-green: #3fb950 (success, local)
- accent-yellow: #d29922 (warnings, folders)
- accent-red: #f85149 (errors, destructive)
- accent-purple: #a371f7 (special elements)
- accent-orange: #d18616 (Claude)
- accent-cyan: #39d353 (active indicators)
```

### Light Mode
```
Background Layers:
- bg-surface-0: #ffffff
- bg-surface-1: #f6f8fa
- bg-surface-2: #eaeef2
- bg-surface-3: #d0d7de

Text Colors:
- text-primary: #1f2328
- text-secondary: #57606a
- text-muted: #8c959f

Border Colors:
- border-default: #d0d7de
- border-subtle: #eaeef2
- border-emphasis: #8c959f
```

## Typography

### Font Stack
```css
font-family: Inter, -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
font-family-mono: 'JetBrains Mono', 'SF Mono', Consolas, monospace;
```

### Scale
- text-xs: 11px / 1.5
- text-sm: 13px / 1.5
- text-base: 14px / 1.6
- text-lg: 16px / 1.5
- text-xl: 20px / 1.4
- text-2xl: 24px / 1.3

### Weights
- normal: 400 (body)
- medium: 500 (emphasis)
- semibold: 600 (headings, labels)

## Spacing

### Base Unit: 4px
- space-1: 4px
- space-2: 8px
- space-3: 12px
- space-4: 16px
- space-5: 20px
- space-6: 24px
- space-8: 32px

### Component Spacing
- Panel padding: 16px (space-4)
- Card padding: 12px (space-3)
- Button padding: 8px 16px
- List item padding: 8px 12px
- Icon-text gap: 8px (space-2)

## Border Radius
- radius-sm: 4px (badges, small elements)
- radius-md: 6px (buttons, inputs)
- radius-lg: 8px (cards, panels)
- radius-xl: 12px (modals, large cards)

## Shadows (Dark Mode)
```css
shadow-sm: 0 1px 2px rgba(0, 0, 0, 0.3);
shadow-md: 0 4px 8px rgba(0, 0, 0, 0.3);
shadow-lg: 0 8px 24px rgba(0, 0, 0, 0.4);
```

## Interactive States

### Hover States
- Background: Lighten by one surface level (e.g., surface-1 -> surface-2)
- Never reduce contrast; always maintain readable text
- Subtle transition: 150ms ease

### Selected States
- Background: accent-blue with 15% opacity
- Border: 1px solid accent-blue at 50% opacity
- Text: Keep original color or slightly brighter

### Focus States
- Ring: 2px solid accent-blue at 50%
- Offset: 2px from element

### Disabled States
- Opacity: 50%
- Cursor: not-allowed

## Component Patterns

### File List Items
```
Default:
- bg: surface-1
- border: 1px solid border-subtle
- text: text-primary

Hover:
- bg: surface-2
- border: 1px solid border-default

Selected:
- bg: accent-blue/15
- border: 1px solid accent-blue/50
- text: accent-blue (or keep text-primary)
```

### Sidebar Navigation
```
Default:
- bg: transparent
- text: text-secondary

Hover:
- bg: surface-2
- text: text-primary

Active:
- bg: surface-3
- text: text-primary
- Indicator: 2px left border accent-blue
```

### Buttons
```
Primary:
- bg: accent-blue
- text: white
- hover: lighten 10%

Secondary:
- bg: surface-2
- text: text-primary
- hover: surface-3

Ghost:
- bg: transparent
- text: text-secondary
- hover: surface-2

Danger:
- bg: transparent
- text: accent-red
- hover: accent-red/10
```

### Badges
```
- Padding: 4px 8px
- Font: text-xs, font-medium
- Border-radius: radius-sm
- Variants: filled (surface-2), accent (color/15 bg + color text)
```

## Animation

### Transitions
```css
transition-fast: 100ms ease;
transition-normal: 150ms ease;
transition-slow: 300ms ease;
```

### Interactive Elements
- Hover transitions: 150ms
- Active press: scale(0.98)
- Focus ring: 150ms fade-in

## Accessibility

### Contrast
- Minimum 4.5:1 for normal text
- Minimum 3:1 for large text and UI components
- Selection states must maintain readability

### Focus Management
- All interactive elements must be keyboard accessible
- Visible focus indicators on all focusable elements

## Tailwind Implementation

### Custom Classes
```css
/* Surface backgrounds */
.bg-surface-0 { @apply bg-[#0d1117] dark:bg-[#0d1117]; }
.bg-surface-1 { @apply bg-[#161b22] dark:bg-[#161b22]; }
.bg-surface-2 { @apply bg-[#21262d] dark:bg-[#21262d]; }
.bg-surface-3 { @apply bg-[#30363d] dark:bg-[#30363d]; }

/* Light mode overrides */
.light .bg-surface-0 { @apply bg-white; }
.light .bg-surface-1 { @apply bg-[#f6f8fa]; }
.light .bg-surface-2 { @apply bg-[#eaeef2]; }
.light .bg-surface-3 { @apply bg-[#d0d7de]; }
```

## File Format Colors
- Markdown: #58a6ff (blue)
- JSON: #d29922 (yellow)
- YAML: #a371f7 (purple)
- Python: #3fb950 (green)
- PlainText: #8b949e (gray)

## Agent Colors
- Claude: #f97316 (orange-500)
- Cursor: #a855f7 (purple-500)
- ContinueDev: #3b82f6 (blue-500)
- Aider: #22c55e (green-500)
- Windsurf: #06b6d4 (cyan-500)

# Aether Studio — Design System v2.0

## 🎨 Design Philosophy

**Core Principles:**

1. **Clarity over decoration** — Every element serves a purpose
2. **Fluid, not flashy** — Smooth animations, never jarring
3. **Cultural respect** — Honor the instruments we represent
4. **Professional, not sterile** — Warm, inviting, human

---

## 🌈 Color Palette

### Base Colors (Dark Theme)

```
Background Layers:
  --bg-canvas:     #050a12  (Deepest — main canvas)
  --bg-surface:    #0a1520  (Panels, cards)
  --bg-elevated:   #0f1e2e  (Floating modules, dropdowns)
  --bg-overlay:    #1a2a3a  (Modals, tooltips)

Borders & Dividers:
  --border-subtle: #0f1e2e
  --border-normal: #1a2a3a
  --border-strong: #2a3a4a

Text:
  --text-primary:   #e2e8f0  (Headings, important text)
  --text-secondary: #94a3b8  (Body text)
  --text-tertiary:  #64748b  (Captions, hints)
  --text-disabled:  #334155  (Disabled states)
```

### Accent Colors (Semantic)

```
Primary (Brand):
  --accent-primary:     #38bdf8  (Cyan — main actions)
  --accent-primary-dim: #0ea5e9

Success:
  --accent-success:     #00e5a0  (Green — connected, playing)
  --accent-success-dim: #00c48c

Warning:
  --accent-warning:     #fbbf24  (Amber — caution)
  --accent-warning-dim: #f59e0b

Error:
  --accent-error:       #ef5350  (Red — destructive, recording)
  --accent-error-dim:   #dc2626

Info:
  --accent-info:        #818cf8  (Purple — info, tips)
  --accent-info-dim:    #6366f1
```

### Instrument Family Colors

```
Strings (Plucked):  #ffb74d  (Warm amber)
Strings (Bowed):    #ff8a65  (Coral)
Winds:              #4fc3f7  (Sky blue)
Percussion:         #ef5350  (Vibrant red)
Keyboard:           #a78bfa  (Soft purple)
Electronic:         #b39ddb  (Neon purple)
Voice:              #f48fb1  (Pink)
```

### Regional Colors (Catalog)

```
East Africa:        #ffb74d  (Golden)
West Africa:        #ef9a9a  (Terracotta)
Middle East:        #ce93d8  (Lavender)
South Asia:         #80cbc4  (Turquoise)
East Asia:          #4fc3f7  (Sky)
Europe:             #a5d6a7  (Sage)
Americas:           #fff176  (Sunflower)
Electronic:         #b39ddb  (Neon)
```

---

## 📐 Spacing & Layout

### Grid System

```
Base unit: 4px

Spacing scale:
  --space-1:  4px   (Tight — icon padding)
  --space-2:  8px   (Compact — button padding)
  --space-3:  12px  (Cozy — card padding)
  --space-4:  16px  (Normal — section padding)
  --space-5:  20px  (Comfortable)
  --space-6:  24px  (Spacious — module padding)
  --space-8:  32px  (Generous — page margins)
  --space-10: 40px  (Extra spacious)
  --space-12: 48px  (Hero sections)
```

### Border Radius

```
  --radius-sm:  4px   (Buttons, inputs)
  --radius-md:  6px   (Cards, panels)
  --radius-lg:  8px   (Modules, modals)
  --radius-xl:  12px  (Hero cards)
  --radius-full: 9999px (Pills, avatars)
```

### Shadows (Layered Depth)

```
  --shadow-sm:  0 1px 2px rgba(0,0,0,0.3)
  --shadow-md:  0 4px 8px rgba(0,0,0,0.4)
  --shadow-lg:  0 8px 16px rgba(0,0,0,0.5)
  --shadow-xl:  0 16px 32px rgba(0,0,0,0.6)
  --shadow-glow: 0 0 20px rgba(56,189,248,0.3)
```

---

## 🔤 Typography

### Font Stack

```css
--font-sans: "Inter Variable", "Inter", system-ui, -apple-system, sans-serif;
--font-mono: "JetBrains Mono", "Fira Code", "Consolas", monospace;
```

### Type Scale

```
Display (Hero):
  font-size: 32px
  font-weight: 800
  line-height: 1.2
  letter-spacing: -0.02em

Heading 1:
  font-size: 24px
  font-weight: 700
  line-height: 1.3
  letter-spacing: -0.01em

Heading 2:
  font-size: 18px
  font-weight: 600
  line-height: 1.4

Heading 3:
  font-size: 14px
  font-weight: 600
  line-height: 1.4
  text-transform: uppercase
  letter-spacing: 0.05em

Body Large:
  font-size: 14px
  font-weight: 400
  line-height: 1.5

Body:
  font-size: 12px
  font-weight: 400
  line-height: 1.5

Caption:
  font-size: 11px
  font-weight: 400
  line-height: 1.4

Label:
  font-size: 10px
  font-weight: 600
  line-height: 1.3
  text-transform: uppercase
  letter-spacing: 0.08em

Code:
  font-family: var(--font-mono)
  font-size: 11px
  font-weight: 400
```

---

## 🎭 Component Patterns

### Glassmorphism

```css
.glass-panel {
  background: rgba(10, 21, 32, 0.7);
  backdrop-filter: blur(12px) saturate(150%);
  border: 1px solid rgba(255, 255, 255, 0.08);
  box-shadow:
    0 8px 32px rgba(0, 0, 0, 0.4),
    inset 0 1px 0 rgba(255, 255, 255, 0.05);
}
```

### Gradient Overlays

```css
.gradient-accent {
  background: linear-gradient(
    135deg,
    rgba(56, 189, 248, 0.15) 0%,
    rgba(129, 140, 248, 0.12) 100%
  );
}

.gradient-warm {
  background: linear-gradient(
    135deg,
    rgba(255, 183, 77, 0.15) 0%,
    rgba(239, 83, 80, 0.12) 100%
  );
}
```

### Hover States

```css
.interactive {
  transition: all 0.15s cubic-bezier(0.4, 0, 0.2, 1);
}

.interactive:hover {
  transform: translateY(-1px);
  box-shadow: var(--shadow-lg);
}

.interactive:active {
  transform: translateY(0);
  box-shadow: var(--shadow-sm);
}
```

### Focus States

```css
.focusable:focus-visible {
  outline: 2px solid var(--accent-primary);
  outline-offset: 2px;
}
```

---

## 🎬 Animation Principles

### Timing Functions

```
Ease Out (Entering):  cubic-bezier(0, 0, 0.2, 1)
Ease In (Exiting):    cubic-bezier(0.4, 0, 1, 1)
Ease In-Out (Moving): cubic-bezier(0.4, 0, 0.2, 1)
Spring (Playful):     cubic-bezier(0.34, 1.56, 0.64, 1)
```

### Duration Scale

```
Instant:  0ms     (Immediate feedback)
Fast:     100ms   (Hover states)
Normal:   200ms   (Transitions)
Slow:     300ms   (Complex animations)
Slower:   500ms   (Page transitions)
```

### Animation Patterns

```
Fade In:
  opacity: 0 → 1
  duration: 200ms
  easing: ease-out

Scale In:
  opacity: 0 → 1
  transform: scale(0.95) → scale(1)
  duration: 200ms
  easing: spring

Slide Up:
  opacity: 0 → 1
  transform: translateY(8px) → translateY(0)
  duration: 300ms
  easing: ease-out

Pulse (Audio Activity):
  transform: scale(1) → scale(1.05) → scale(1)
  duration: 600ms
  easing: ease-in-out
  iteration: infinite
```

---

## 🧩 Component Library

### Buttons

```
Primary:
  - Background: gradient-accent
  - Border: 1px solid rgba(56, 189, 248, 0.4)
  - Text: --accent-primary
  - Shadow: shadow-glow
  - Hover: Brighten + lift

Secondary:
  - Background: transparent
  - Border: 1px solid --border-normal
  - Text: --text-secondary
  - Hover: Background --bg-elevated

Danger:
  - Background: rgba(239, 83, 80, 0.15)
  - Border: 1px solid rgba(239, 83, 80, 0.4)
  - Text: --accent-error
  - Hover: Brighten

Ghost:
  - Background: transparent
  - Border: none
  - Text: --text-tertiary
  - Hover: Background --bg-surface
```

### Inputs

```
Text Input:
  - Background: --bg-surface
  - Border: 1px solid --border-normal
  - Padding: 8px 12px
  - Radius: --radius-sm
  - Focus: Border --accent-primary + glow

Slider:
  - Track: --bg-elevated (height: 4px)
  - Thumb: --accent-primary (size: 16px)
  - Fill: gradient-accent
  - Hover: Thumb scale(1.2)
  - Active: Glow

Knob (Rotary):
  - Size: 48px
  - Arc: 270° (from -135° to 135°)
  - Track: --border-strong
  - Fill: gradient-accent
  - Center dot: --text-primary
  - Value label: Below, --text-secondary
```

### Cards

```
Standard Card:
  - Background: --bg-surface
  - Border: 1px solid --border-subtle
  - Radius: --radius-md
  - Padding: 16px
  - Shadow: --shadow-sm
  - Hover: Border --border-normal + lift

Elevated Card:
  - Background: --bg-elevated
  - Border: 1px solid --border-normal
  - Radius: --radius-lg
  - Padding: 20px
  - Shadow: --shadow-md

Glass Card:
  - Use .glass-panel
  - Padding: 24px
  - Shadow: --shadow-lg
```

---

## 📱 Responsive Breakpoints

```
Mobile:   < 640px
Tablet:   640px - 1024px
Desktop:  1024px - 1440px
Wide:     > 1440px

Layout adjustments:
- Mobile: Single column, bottom nav
- Tablet: Collapsible sidebar
- Desktop: Full layout with floating modules
- Wide: Extra panels (browser, inspector)
```

---

## ♿ Accessibility

### Contrast Ratios

```
Text on background:
  - Primary text:   >= 7:1 (AAA)
  - Secondary text: >= 4.5:1 (AA)
  - Disabled text:  >= 3:1

Interactive elements:
  - Borders/icons:  >= 3:1
  - Focus indicators: >= 3:1
```

### Keyboard Navigation

```
Tab order: Logical, left-to-right, top-to-bottom
Focus indicators: Always visible
Shortcuts: Documented, customizable
Screen reader: ARIA labels on all interactive elements
```

---

## 🎯 Design Tokens (CSS Variables)

See `aether-dsp/ui/src/styles/tokens.css` for the complete token set.

---

**Next:** View the interactive mockups in `design/mockups/`

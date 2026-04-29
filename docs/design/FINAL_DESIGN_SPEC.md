# Aether Studio v2.0 — Final Complete Design Specification

## 🎯 COMPLETE VISUAL DESIGN

This document describes the **finished, production-ready design** for Aether Studio v2.0.

---

## 📐 FULL INTERFACE LAYOUT

```
┌─────────────────────────────────────────────────────────────────────┐
│ ◉ Aether v2.0  [🌍 Explore] [🎛 Create] [⏱ Arrange] [🎮 Perform]  │ ← Top Bar (56px)
│                                                                     │
│ ▶ Playing  📂 Load  ↩ Undo  ↪ Redo  ● REC    [VU] [●●●●●] 🟢 Live │
├─────────────────────────────────────────────────────────────────────┤
│ [Mode-specific content below]                                       │
│                                                                     │
│ [Changes based on selected mode: Explore/Create/Arrange/Perform]   │
│                                                                     │
│                                                                     │
│                                                                     │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

---

## 🌍 MODE 1: EXPLORE (Catalog)

**Default view when you open Aether Studio**

### Layout:

```
┌─────────────────────────────────────────────────────────────────────┐
│ [Top Bar with mode switcher]                                       │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  🌍 Instrument Catalog                    [Search...] [+ Custom]   │
│                                                                     │
│  ┏━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓  │
│  ┃  🇪🇹 KRAR                                          [▶] [🎹] ┃  │
│  ┃  Ethiopian 5-string lyre • Tizita tuning                   ┃  │
│  ┃  ┌─────────────────────────────────────────────────────┐   ┃  │
│  ┃  │ [Large photo of Krar player in traditional dress]  │   ┃  │
│  ┃  └─────────────────────────────────────────────────────┘   ┃  │
│  ┃  ▁▂▃▅▇▇▅▃▂▁ Live waveform animation ▁▂▃▅▇▇▅▃▂▁          ┃  │
│  ┃                                                             ┃  │
│  ┃  "The Krar is a five or six-string bowl-shaped lyre       ┃  │
│  ┃   from Ethiopia and Eritrea, traditionally played by      ┃  │
│  ┃   the Tigrinya people. Its bright, resonant tone is       ┃  │
│  ┃   central to Ethiopian folk music and azmari tradition."  ┃  │
│  ┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛  │
│                                                                     │
│  [Grid of 8 more instrument cards, 4 per row]                      │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐             │
│  │ Masenqo  │ │ Washint  │ │ Kebero   │ │ Kora     │             │
│  │ 🇪🇹       │ │ 🇪🇹       │ │ 🇪🇹       │ │ 🇸🇳       │             │
│  └──────────┘ └──────────┘ └──────────┘ └──────────┘             │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐             │
│  │ Balafon  │ │ Djembe   │ │ Oud      │ │ Qanun    │             │
│  │ 🇲🇱       │ │ 🇬🇳       │ │ 🌍       │ │ 🇹🇷       │             │
│  └──────────┘ └──────────┘ └──────────┘ └──────────┘             │
│                                                                     │
│  [East Africa] [West Africa] [Middle East] [South Asia]...        │
│  ─────────────                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

### Key Features:

- **Hero Card**: One featured instrument at a time (rotates)
- **Large Photos**: Authentic images of instruments being played
- **Waveform**: Animated, real-time visualization
- **Cultural Context**: History, origin, playing technique
- **Region Tabs**: Filter by 8 regions
- **Search**: Fuzzy matching across name, country, tags
- **Actions**: Play (preview), Try It (keyboard), Add (to canvas)

### Colors:

- Background: Deep space black (#050a12)
- Hero card: Glassmorphism with regional accent color
- East Africa: Golden amber (#ffb74d)
- Text: High contrast white (#e2e8f0)

---

## 🎹 KEYBOARD PLAYER OVERLAY

**Triggered by clicking "Try It" on any instrument**

### Layout:

```
┌─────────────────────────────────────────────────────────────────────┐
│ [Full-screen overlay, semi-transparent dark background]            │
│                                                                     │
│  🇪🇹 KRAR • Tizita Tuning                              [✕ Close]   │
│                                                                     │
│  ┌───────────────────────────────────────────────────────────────┐ │
│  │ [Faded instrument photo as background]                       │ │
│  │                                                               │ │
│  │  ┌─────────────────────────────────────────────────────────┐ │ │
│  │  │ [Visual Piano Keyboard - 2 octaves]                     │ │ │
│  │  │ C  D  E  F  G  A  B  C  D  E  F  G  A  B  C           │ │ │
│  │  │ ▓▓ ▓▓ ▓▓    ▓▓ ▓▓ ▓▓ ▓▓    ▓▓ ▓▓ ▓▓    ▓▓ ▓▓ ▓▓       │ │ │
│  │  │ ░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░ │ │ │
│  │  │ [Keys glow when pressed with ripple effect]            │ │ │
│  │  └─────────────────────────────────────────────────────────┘ │ │
│  │                                                               │ │
│  │  A S D F G H J K L  (white keys)                             │ │
│  │  W E   T Y U   O P  (black keys)                             │ │
│  └───────────────────────────────────────────────────────────────┘ │
│                                                                     │
│  Octave: [◀ 3 ▶]    Velocity: [━━━━━━━●━━] 80                    │
│                                                                     │
│  ┌───────────────────────────────────────────────────────────────┐ │
│  │ [Real-time Waveform Visualization]                            │ │
│  │ ▁▂▃▅▇▇▅▃▂▁▂▃▅▇▇▅▃▂▁▂▃▅▇▇▅▃▂▁▂▃▅▇▇▅▃▂▁                        │ │
│  └───────────────────────────────────────────────────────────────┘ │
│  ┌───────────────────────────────────────────────────────────────┐ │
│  │ [Spectrum Analyzer]                                           │ │
│  │ ▁▂▃▄▅▆▇█▇▆▅▄▃▂▁▂▃▄▅▆▇█▇▆▅▄▃▂▁                                │ │
│  └───────────────────────────────────────────────────────────────┘ │
│                                                                     │
│  [Add to Canvas]  [Record Performance]                             │
└─────────────────────────────────────────────────────────────────────┘
```

### Interactions:

- **PC Keyboard**: A-L = white keys, W-E-T-Y-U-O-P = black keys
- **Mouse Click**: Click keys to play
- **Octave**: Z = down, X = up
- **Velocity**: Slider adjusts note loudness
- **Visual Feedback**: Keys glow + ripple effect when pressed
- **Waveform**: Updates in real-time as you play
- **Spectrum**: Shows frequency content
- **ESC**: Close overlay

---

## 🎛 MODE 2: CREATE (Node Graph)

**For building audio patches**

### Layout:

```
┌─────────────────────────────────────────────────────────────────────┐
│ [Top Bar]                                                           │
├───┬─────────────────────────────────────────────────────────────────┤
│ N │                                                                 │
│ o │  ┌─────────┐                                                   │
│ d │  │ Krar    │───●  ●●●●● Particles flowing ●●●●●               │
│ e │  │ Sampler │   │                                               │
│ s │  │ [mini   │   │  ┌─────────┐                                 │
│   │  │ wave]   │   └──│ Filter  │───●                             │
│ 🎛│  └─────────┘      │ LP 2kHz │   │  ┌─────────┐               │
│ 🎚│  [Glowing]        │ [mini   │   └──│ Output  │               │
│ 📈│                   │ wave]   │      │ [VU]    │               │
│ ⏱│                   └─────────┘      └─────────┘               │
│ 🔊│                                                                 │
│ 🎚│  [Nodes pulse/glow when audio is active]                      │
│   │  [Mini waveform preview inside each node]                      │
│   │  [Animated particle flow through edges]                        │
│ [+]│                                                                 │
├───┴─────────────────────────────────────────────────────────────────┤
│ [Mini-map in bottom-right corner showing full graph]               │
└─────────────────────────────────────────────────────────────────────┘
```

### Key Features:

- **Animated Edges**: Particles flow from output to input
- **Node Glow**: Nodes pulse when audio is active
- **Mini Preview**: Waveform inside each node
- **Smart Routing**: Suggest connections based on node types
- **Collapsible Sidebar**: More canvas space
- **Context Menu**: Right-click for actions
- **Multi-select**: Shift+click or drag box
- **Copy/Paste**: Ctrl+C/V

---

## 🎹 MODE 3: ARRANGE (Timeline + Piano Roll)

**For composing melodies and arranging songs**

### Layout:

```
┌─────────────────────────────────────────────────────────────────────┐
│ [Top Bar]                                                           │
├─────────────────────────────────────────────────────────────────────┤
│ Piano Roll • Krar Melody                              [✕ Close]    │
├─────────────────────────────────────────────────────────────────────┤
│ C5 ┃▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓│
│ B4 ┃░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░│
│ A4 ┃▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓│
│ G4 ┃░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░│
│ F4 ┃▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓│
│ E4 ┃░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░│
│ D4 ┃▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓│
│    ┃  ┌──┐    ┌──┐  ┌────┐    ┌──┐                              │
│    ┃  │  │    │  │  │    │    │  │  ← Notes (draggable)         │
│    ┃  └──┘    └──┘  └────┘    └──┘                              │
│    ┃  [Scale: Tizita (Ethiopian)] [Ghost notes: ON]              │
├─────────────────────────────────────────────────────────────────────┤
│ Velocity                                                            │
│ ▂▅▇▅▃▂▅▇▅▃  ← Velocity bars (editable)                             │
├─────────────────────────────────────────────────────────────────────┤
│ [Timeline below with clips, automation, markers]                    │
└─────────────────────────────────────────────────────────────────────┘
```

### Key Features:

- **Scale Highlighting**: Only show notes in active scale (Tizita, Bati, etc.)
- **Ghost Notes**: See notes from other tracks (faded)
- **Velocity Editor**: Adjust note loudness
- **Chord Tools**: Insert chords with one click
- **Humanize**: Add random timing/velocity
- **Quantize**: Snap notes to grid

---

## 🎮 MODE 4: PERFORM (Clip Launcher)

**For live performance**

### Layout:

```
┌─────────────────────────────────────────────────────────────────────┐
│ [Minimal top bar, full-screen]                                     │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  [Clip Launcher Grid: 8x8]                                         │
│  ┌────┬────┬────┬────┬────┬────┬────┬────┐                        │
│  │ ▶  │    │ ▶  │    │    │    │    │    │  Track 1: Krar        │
│  ├────┼────┼────┼────┼────┼────┼────┼────┤                        │
│  │    │ ▶  │    │    │    │    │    │    │  Track 2: Drums       │
│  ├────┼────┼────┼────┼────┼────┼────┼────┤                        │
│  │ ▶  │    │    │ ▶  │    │    │    │    │  Track 3: Bass        │
│  ├────┼────┼────┼────┼────┼────┼────┼────┤                        │
│  │    │    │ ▶  │    │    │    │    │    │  Track 4: Melody      │
│  ├────┼────┼────┼────┼────┼────┼────┼────┤                        │
│  │    │    │    │    │    │    │    │    │  Track 5              │
│  ├────┼────┼────┼────┼────┼────┼────┼────┤                        │
│  │    │    │    │    │    │    │    │    │  Track 6              │
│  ├────┼────┼────┼────┼────┼────┼────┼────┤                        │
│  │    │    │    │    │    │    │    │    │  Track 7              │
│  ├────┼────┼────┼────┼────┼────┼────┼────┤                        │
│  │    │    │    │    │    │    │    │    │  Track 8              │
│  └────┴────┴────┴────┴────┴────┴────┴────┘                        │
│  [Large, colorful, touch-friendly cells]                           │
│  [Click to launch clip, click again to stop]                       │
│                                                                     │
│  [Master controls at bottom: Volume, BPM, Scene Launch]            │
└─────────────────────────────────────────────────────────────────────┘
```

---

## 🎨 DESIGN SYSTEM SUMMARY

### Colors:

- **Canvas**: #050a12 (deep space black)
- **Surface**: #0a1520 (panels)
- **Elevated**: #0f1e2e (floating modules)
- **Primary Accent**: #38bdf8 (cyan)
- **Success**: #00e5a0 (green)
- **Error**: #ef5350 (red)
- **Warning**: #fbbf24 (amber)

### Typography:

- **Font**: Inter Variable (sans-serif)
- **Sizes**: 10px → 32px (8-step scale)
- **Weights**: 300, 400, 500, 600, 700, 800

### Spacing:

- **Grid**: 8px base unit
- **Scale**: 4px, 8px, 12px, 16px, 24px, 32px, 48px

### Animations:

- **Duration**: 150-300ms
- **Easing**: cubic-bezier(0.4, 0, 0.2, 1)
- **Hover**: Lift + glow
- **Click**: Scale down + ripple

---

## 🚀 NEXT STEPS

This is the complete design specification. To see it in action:

1. **I'll create the HTML mockup** (next message)
2. **You open it in your browser**
3. **You interact with all modes**
4. **You provide feedback**
5. **We start building the real thing**

Ready for the HTML mockup file?

# Chi Architecture Discussion - 2025-11-29
## Markdown Viewer as Phase 2 First Application

---

## Context

**Problem:** Need a lightweight GUI markdown viewer for 4K OLED display (no heavy browsers, no ugly TUI).

**Solution:** Build it as Chi's Phase 2 first application - tests semantic text rendering architecture while solving immediate need.

**Session Goal:** Architectural thinking, not implementation. Explore design decisions before coding.

---

## The Three-Layer Architecture (Clarified)

### Overview

Chi separates rendering into **three distinct actors**, each with clear responsibilities:

```
Markdown App ──[bubbles]──> Spirit ──[render commands]──> Renderer
(Domain Expert)          (Orchestrator)                (Graphics Specialist)
```

### 1. Markdown App (Domain Expert, Presentation-Naive)

**Knows:**
- Markdown syntax
- Formatting rules (H1 should be larger/bold, code needs monospace)
- Content structure

**Doesn't Know:**
- Pixels, screens, geometry
- Where output goes (monitor? VR? remote display?)
- How to actually render fonts

**Sends to Spirit:**
```rust
Bubble {
    text: "Getting Started",
    semantic_hint: Heading(level: 1),
    style_request: "1.5x base font size, bold weight"
}
```

**Key principle:** Makes **recommendations**, not commands.
- "Hey Spirit, this should be bigger/bold, but YOU decide the details"
- Not: "Render at 21pt Inter Bold at pixel coordinates X,Y"

**Example messages:**
- H1 heading → "1.5x size, bold weight"
- Paragraph → "normal size, regular weight"
- Code block → "monospace font, slightly smaller"

### 2. Spirit (Orchestrator, Spatial Intelligence)

**Knows:**
- Scene layout and spatial relationships
- Where outputs exist (monitors, VR headsets, remote displays)
- Viewing context (distance, device capabilities)

**Doesn't Know:**
- How to render fonts/geometry
- Graphics pipelines, shaders, triangulation

**Receives:** Semantic bubbles from apps

**Decides:**
- "This markdown goes on a smoky glass pane at coordinates X,Y,Z"
- "Heading text goes on the metallic cylinder surface"
- "Base font size is 14pt for this viewing distance"

**Sends to Renderer:**
```rust
RenderCommand {
    bubble_id: 42,
    text: "Getting Started",
    font: "Inter Bold",
    size: 21pt,  // (14pt base * 1.5)
    surface: SmokyGlassPane {
        position: [x, y, z],
        normal: [...]
    }
}
```

**Key principle:** Interprets app recommendations based on **context**.
- Same bubble might be 21pt on monitor, 32pt in VR (closer viewing distance)
- Same content might be on flat pane (2D) or curved cylinder (VR immersive)

### 3. Renderer (Graphics Specialist, Semantic-Naive)

**Knows:**
- wgpu, GPU pipelines
- Triangulation, shaders, textures, materials
- Graphics execution

**Doesn't Know:**
- What content means (doesn't know what "markdown" is)
- Why something is bold or large
- Application semantics

**Has Access To:**
- Pre-triangulated font library (`.chi_font` assets)
- Material library (glass, metallic, matte)
- Shader library

**Receives:** Render commands from Spirit

**Executes:**
1. Lookup "Inter Bold, glyph 'G'" → retrieve pre-baked triangles
2. Apply material (smoky glass shader, refraction effects)
3. Position geometry on surface (cylinder, pane, arbitrary mesh)
4. Upload to GPU, render frame

**Can Be:**
- Local (same machine as Spirit)
- Remote (VR headset over network)
- Multiple (4K monitor + VR headset simultaneously)

---

## Complete Flow Example

**Scenario:** User opens KEYBINDINGS.md containing `# Hyprland Keybindings`

```
┌─────────────────────────────────────────────────────────────┐
│ 1. File on Disk                                             │
│    KEYBINDINGS.md: "# Hyprland Keybindings"                 │
└─────────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────────┐
│ 2. Markdown App Parses                                      │
│    - Detects H1 heading                                     │
│    - Knows: "This should be larger and bold"                │
│    - Creates bubble                                         │
└─────────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────────┐
│ 3. Bubble Sent to Spirit (TLV over TCP)                     │
│                                                              │
│    Bubble {                                                 │
│        content: "Hyprland Keybindings",                     │
│        structure: Heading { level: 1 },                     │
│        style_request: {                                     │
│            size_multiplier: 1.5,                            │
│            weight: Bold                                     │
│        }                                                    │
│    }                                                        │
└─────────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────────┐
│ 4. Spirit Makes Decisions                                   │
│    - Target: Monitor 1 (4K OLED)                            │
│    - Base font: Inter, 14pt                                 │
│    - Surface: Flat rectangular pane, white background       │
│    - Position: Top-left of pane [0, 0]                      │
│    - Calculates: 14pt * 1.5 = 21pt                          │
└─────────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────────┐
│ 5. Spirit Sends Render Command to Local Renderer            │
│                                                              │
│    RenderCommand {                                          │
│        font: "Inter Bold",                                  │
│        size: 21pt,                                          │
│        text: "Hyprland Keybindings",                        │
│        surface: FlatPane { color: white },                  │
│        position: [0, 0]                                     │
│    }                                                        │
└─────────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────────┐
│ 6. Renderer Executes                                        │
│    a. Load Inter Bold from:                                 │
│       ~/.local/share/chi/fonts/Inter-Bold.chi_font          │
│    b. Shape text with rustybuzz → glyph IDs                 │
│       "H" = glyph 42, "y" = glyph 89, etc.                  │
│    c. Lookup pre-baked triangles for each glyph             │
│    d. Position geometry on flat pane                        │
│    e. Apply white material shader                           │
│    f. Upload vertex/index buffers to GPU                    │
│    g. Render frame with wgpu                                │
└─────────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────────┐
│ 7. User Sees Beautiful Heading on 4K OLED                   │
│    - Proper proportional font (Inter Bold)                  │
│    - Smooth anti-aliasing                                   │
│    - Correct size (21pt, visually larger than body)         │
│    - No ugly TUI, no heavy browser                          │
└─────────────────────────────────────────────────────────────┘
```

---

## Why This Architecture Works

### Separation of Concerns
- **Markdown app** = semantic formatting expert (what's a heading?)
- **Spirit** = spatial placement expert (where does it go?)
- **Renderer** = graphics execution expert (how to draw it?)

Each actor does ONE thing well, doesn't overstep boundaries.

### Network Transparency
- Markdown app and Spirit can be on different machines
- Spirit and Renderer can be on different machines
- Multiple renderers can consume same Spirit output

**Example distributed setup:**
```
Main Machine:     Markdown App + Spirit
Remote VR:        Renderer (displays in VR headset)
Remote Monitor:   Renderer (displays on secondary screen)
```

### Shared Asset Library
- All renderers access same pre-baked font library
- No duplication (each renderer doesn't triangulate fonts independently)
- Fonts are "blueprints" (Unity) or "prefabs" (Unreal) - reusable assets

### Flexibility Through Context
Same markdown bubble renders differently based on context:

**2D Monitor:**
- Flat white pane
- 14pt base font
- Standard anti-aliasing

**VR Headset:**
- Curved immersive surface
- 20pt base font (closer viewing distance)
- Enhanced depth cues

**Terminal Fallback:**
- ASCII rendering
- No graphics needed

**Spirit decides based on target device, app doesn't change.**

---

## Font Rendering Pipeline (Conceptual)

### The Four Stages

#### 1. Font Loading (One-Time)
- Read `.ttf` or `.otf` file from disk
- Parse font tables (glyph outlines, metrics, kerning)

**Question:** Raw TTF every time, or pre-process to custom format?

#### 2. Text Shaping (Per String)
- **Tool:** rustybuzz
- **Input:** "Hello world" + font
- **Process:**
  - Picks correct glyphs (ligatures: "fi" → single glyph)
  - Applies kerning ("AV" spacing ≠ "AA" spacing)
  - Handles complex scripts (Arabic shaping, Devanagari)
  - Calculates glyph positions
- **Output:** `[glyph_id, x_offset, y_offset]` array

This is **language/script-aware**, not just dumb character mapping.

#### 3. Glyph Triangulation (Per Glyph, Cacheable)
- **Tool:** lyon
- **Input:** Glyph outline (Bézier curves from font)
- **Process:** Tessellate curves → triangle mesh
- **Output:** Vertex buffer (triangles ready for GPU)

**This is expensive - do ONCE per glyph, cache forever.**

Different quality levels:
- Coarse: 100 triangles/glyph (small text, distant)
- Medium: 300 triangles/glyph (normal reading)
- Fine: 800 triangles/glyph (large text, focus)
- Ultra: 2000 triangles/glyph (extreme zoom)

#### 4. GPU Rendering (Per Frame)
- **Tool:** wgpu
- **Input:** Triangle geometry
- **Process:**
  - Upload vertex/index buffers
  - Apply shaders (color, material, lighting)
  - Rasterize triangles to pixels
  - Anti-aliasing, alpha blending
- **Output:** Pixels on screen

GPU handles scaling, anti-aliasing automatically.

---

## Font Storage Strategies

### Option A: Runtime Triangulation (Flexible)

**Flow:**
1. App startup: Load `Inter.ttf` (raw font file)
2. User opens markdown file
3. Shape text with rustybuzz → glyph IDs
4. Triangulate glyphs on-demand with lyon
5. Cache triangulated glyphs in memory (HashMap<GlyphId, TriangleMesh>)
6. Render with wgpu

**Pros:**
- Simple to implement
- Standard font files (shareable, editable)
- Different apps can use different fonts easily
- No custom tooling needed

**Cons:**
- Slow startup (~50-100ms to triangulate common glyphs)
- Each app duplicates work (every app triangulates same font)
- Memory overhead (each app has separate cache)

**When to use:** MVP development, prototyping

---

### Option B: Pre-Baked Font Assets (Optimized)

**Offline Process (Font Baker Tool):**
```
Input:  Inter-Regular.ttf
Output: Inter-Regular.chi_font

Contents:
- All glyphs pre-triangulated (~800 glyphs for Latin extended)
- Multiple LODs (100/300/800 triangle versions)
- ESDT distance fields (for far-away text in VR)
- Metadata (metrics, baseline, x-height)
- Compressed binary format
```

**Runtime Flow:**
1. App startup: Load `Inter-Regular.chi_font` (binary blob)
2. Shape text with rustybuzz → glyph IDs
3. Look up triangles (instant, already done)
4. Render with wgpu

**Pros:**
- Fast startup (just load binary, no tessellation)
- Multiple apps share same font assets (no duplication)
- Can include multiple LODs, distance fields
- Optimized format (compressed, GPU-ready)

**Cons:**
- Need to build Font Baker tool first
- Custom format (not standard TTF)
- Changing fonts requires re-baking
- Larger storage footprint (pre-baked data > source TTF)

**When to use:** Production, performance-critical, shared assets

---

### Recommended Hybrid Approach

**For Weekend MVP (Markdown Viewer):**
- Use **Option A** (runtime triangulation)
- Keep it simple, prove the concept
- Load Inter.ttf directly, triangulate on startup
- Single quality level (~300 triangles/glyph)

**Future (Chi Phase 2 Refinement):**
- Build Font Baker tool (offline utility)
- Pre-bake commonly used fonts (Inter, JetBrains Mono, Roboto)
- Spirit shares font assets across apps (no duplication)
- Support multiple LODs (quality vs performance trade-offs)

---

## Font Sharing Architecture

### Asset Locations

**Option 1: Chi-specific directory**
```
~/.local/share/chi/fonts/
    Inter-Regular.ttf           # Raw TTF (Option A)
    Inter-Bold.ttf
    JetBrainsMono-Regular.ttf

    Inter-Regular.chi_font      # Pre-baked (Option B, future)
    Inter-Bold.chi_font
```

**Option 2: Use system fonts**
```
/usr/share/fonts/               # Fedora system fonts
~/.local/share/fonts/           # User-installed fonts (Iosevka lives here)
```

**Recommended:** Hybrid
- Read from system fonts first (standard TTFs)
- Fall back to `~/.local/share/chi/fonts/` for `.chi_font` assets
- Leverage existing font infrastructure

### Who Loads Fonts?

**Currently (Phase 2 Start - Decentralized):**
- Each app loads its own fonts
- Markdown viewer loads Inter + JetBrains Mono independently
- Spirit doesn't render yet (just routes bubbles)
- No sharing (each app has separate font cache)

**Future (Spirit-Managed - Centralized):**
- Spirit loads fonts ONCE
- Apps send semantic bubbles ("render this as H1")
- Spirit decides font, size, color
- Spirit performs triangulation
- Renderers receive pre-shaped geometry
- No duplication, shared font cache across all apps

**Trade-off:**
- Decentralized = simpler, faster to prototype
- Centralized = efficient, avoids duplication, but Spirit becomes heavyweight

---

## Multi-Font Support (Markdown Requirements)

Markdown documents require **mixing fonts**:

### Font Assignments

1. **Body text** → Inter Regular (proportional, readable)
2. **Headings** → Inter Bold (same family, heavier weight)
3. **Inline code** → JetBrains Mono Regular (monospace)
4. **Code blocks** → JetBrains Mono Regular (monospace)

### Rendering Example

```markdown
# Getting Started                    ← Inter Bold, 21pt
This is a paragraph with `code`.    ← Inter Regular 14pt + JetBrains Mono 13pt
```

**Challenges:**

#### Baseline Alignment
- Inter has one baseline/x-height
- JetBrains Mono has different metrics
- When mixing inline code in paragraph, baselines must align vertically
- Solution: Calculate baseline offset, adjust Y positions during shaping

#### Fallback Fonts
- User pastes emoji: 😀
- Inter doesn't have emoji glyphs
- Must fall back to Noto Color Emoji (or similar)
- Detection: rustybuzz returns `.notdef` glyph ID → trigger fallback
- Chain: Inter → Noto Emoji → Last resort (.notdef square)

#### Performance
- Each font needs separate rustybuzz shaper instance
- Each font needs separate glyph cache
- Switching fonts mid-paragraph requires re-shaping
- Cache intelligently to avoid redundant work

---

## Proportional Font Challenges

### Why Proportional is Harder Than Monospace

**Monospace (easy):**
- Every character = same width
- Line length = `char_count × char_width`
- No wrapping surprises
- Alignment is trivial

**Proportional (hard):**
- Every character = different width
  - "iii" is narrow
  - "WWW" is wide
- Kerning changes spacing dynamically
  - "AV" tighter than "AA"
- Ligatures change glyph count
  - "fi" → one glyph, not two
- Line breaking requires measuring actual rendered width

### Layout Problems to Solve

#### 1. Line Wrapping
**Problem:** When does paragraph text wrap to next line?

**Solution:**
```
1. Start with empty line
2. Shape next word with rustybuzz
3. Measure shaped width (sum of glyph advances)
4. If current_line_width + word_width > max_width:
   - Finalize current line
   - Start new line with word
5. Otherwise: Add word to current line
6. Repeat
```

Requires **pre-shaping** to know widths before final layout.

#### 2. Justification
**Options:**
- Left-aligned (simple, recommended for MVP)
- Right-aligned (rare for body text)
- Centered (for headings)
- Justified (complex, requires inter-word spacing adjustment)

**For MVP:** Left-aligned only. Defer justification.

#### 3. Widow/Orphan Prevention
**Typography rules:**
- Widow: Last line of paragraph with single word (ugly)
- Orphan: First line of paragraph alone at bottom of page (ugly)

**Do we care?**
- TeX: Yes, sophisticated algorithms
- Web browsers: Basic rules
- **Chi MVP:** No, defer to future

**Recommendation:** Keep layout simple initially, add refinements later.

---

## The Spirit Distribution Problem

### The Challenge

You have multiple machines:
- **Main** (7800X3D + RTX 4080) - development, primary workspace
- **think** (i7-8700, 32GB) - network testbed, potential Gemma host
- **jetsone** (Orin Nano, ARM) - rendering node, VR potential
- **Future:** VR headset (Quest 3?), additional displays

**Spirit is a process. It can't literally be everywhere.**

**So where does it run?**

### Three Architectural Models

---

#### Model 1: Centralized Spirit (Single Authority)

```
┌─────────────────────────────────────────┐
│  Main Machine (7800X3D)                 │
│                                         │
│  ┌─────────┐      ┌──────────┐        │
│  │ MD App  │─────▶│  Spirit  │        │
│  └─────────┘      └────┬─────┘        │
│                        │               │
└────────────────────────┼───────────────┘
                         │
           ┌─────────────┼─────────────┐
           │             │             │
           ▼             ▼             ▼
    ┌──────────┐  ┌──────────┐  ┌──────────┐
    │ Renderer │  │ Renderer │  │ Renderer │
    │  (local) │  │ (jetsone)│  │   (VR)   │
    └──────────┘  └──────────┘  └──────────┘
```

**Behavior:**
- One Spirit instance runs on main machine
- Receives bubbles from all apps
- Makes all decisions (spatial layout, font selection, sizing)
- Sends render commands to all renderers (local + remote)

**Pros:**
- **Simple:** Single source of truth
- **Consistent:** All decisions in one place
- **Easy to reason about:** No distributed consensus needed

**Cons:**
- **Single point of failure:** Spirit down = entire system down
- **Network latency:** Remote renderers wait for Spirit over network
- **Main machine must always be on:** Can't use jetsone standalone

**When to use:** MVP development, single-user scenarios

---

#### Model 2: Distributed Spirits (Federated)

```
┌───────────────┐      ┌───────────────┐      ┌───────────────┐
│ Main Machine  │      │    think      │      │   jetsone     │
│               │      │               │      │               │
│ ┌───┐ ┌─────┐│      │ ┌─────┐       │      │ ┌─────┐ ┌───┐│
│ │MD │─│Spir.││◀────▶││Spir.│       │◀────▶││Spir.│─│VR ││
│ │App│ │     ││      ││     │       │      ││     │ │Ren││
│ └───┘ └──┬──┘│      │└──┬──┘       │      │└──┬──┘ └───┘│
│          │    │      │   │          │      │   │         │
│       ┌──▼──┐ │      │┌──▼──┐       │      │┌──▼──┐      │
│       │Ren. │ │      ││Ren. │       │      ││Ren. │      │
│       └─────┘ │      │└─────┘       │      │└─────┘      │
└───────────────┘      └───────────────┘      └───────────────┘
         ↕                     ↕                      ↕
         └─────────────────────┴──────────────────────┘
                    (CRDT synchronization)
```

**Behavior:**
- Each machine runs its own Spirit instance
- Spirits sync state via CRDT (eventual consistency)
- No single authority - peer-to-peer federation
- Each Spirit can make independent decisions

**Pros:**
- **No single point of failure:** Any Spirit can continue if others fail
- **Low latency:** Local Spirit → local Renderer (no network hop)
- **Works offline:** Machines can disconnect, sync later

**Cons:**
- **Complex:** Consensus protocols, conflict resolution
- **Ambiguous authority:** Which Spirit decides on conflicts?
- **Synchronization overhead:** Network traffic between Spirits
- **Harder to reason about:** Distributed state is inherently complex

**When to use:** Multi-user collaboration, high-availability requirements

---

#### Model 3: Hybrid (Primary + Shadow Spirits)

```
┌─────────────────────────────────────────┐
│  Main Machine (Primary Spirit)          │
│                                         │
│  ┌─────────┐      ┌──────────┐        │
│  │ MD App  │─────▶│  Spirit  │        │
│  └─────────┘      └────┬─────┘        │
│                        │ (publish      │
│                        │  scene state) │
└────────────────────────┼───────────────┘
                         │
           ┌─────────────┼─────────────┐
           │             │             │
           ▼             ▼             ▼
    ┌──────────┐  ┌──────────┐  ┌──────────┐
    │ Renderer │  │ Shadow   │  │ Shadow   │
    │  (local) │  │ Spirit   │  │ Spirit   │
    │          │  │ +Renderer│  │ +Renderer│
    └──────────┘  └────┬─────┘  └────┬─────┘
                       │             │
                  (subscribes)  (subscribes)
```

**Behavior:**

**Primary Spirit (main machine):**
- Receives bubbles from apps (only Primary accepts app connections)
- Makes **spatial decisions** (which display, 3D coordinates, scene layout)
- Publishes scene state to Shadow Spirits (one-way stream)

**Shadow Spirits (jetsone, VR headset):**
- Subscribe to scene updates from Primary (read-only)
- Make **local rendering decisions** (font size for viewing distance, LOD selection)
- Control local Renderer directly (low latency, no network hop)
- Can continue rendering if Primary disconnects (frozen state, no new bubbles)

**Apps (markdown viewer, etc.):**
- Only talk to Primary Spirit (don't know Shadows exist)
- Primary handles all distribution

**Pros:**
- **Clear authority:** Primary decides, no ambiguity
- **Low rendering latency:** Shadow decides locally, no round-trip to Primary
- **Graceful degradation:** Shadows continue working with stale data if Primary fails
- **Simpler than full federation:** One-way data flow (Primary → Shadows)

**Cons:**
- **Primary is critical:** Can't create new bubbles if Primary is down
- **Shadows might diverge:** If network partitions, Shadows work with stale state
- **More complex than centralized:** Still need synchronization protocol

**When to use:** Multi-display setups, VR + monitor simultaneously, production Chi

---

### Recommendation: Model 3 (Hybrid) for Chi

**Reasoning:**

1. **Matches Chi's use case:**
   - Primary workspace (main machine) where apps run
   - Multiple displays (monitor, VR, remote screens)
   - Need low-latency rendering on each device

2. **Good trade-offs:**
   - Simpler than full federation (clear authority)
   - More flexible than pure centralized (Shadows make local decisions)
   - Graceful degradation (Shadows work offline)

3. **Clear separation:**
   - Primary = app-facing, spatial intelligence
   - Shadow = rendering-facing, display optimization
   - Apps don't deal with distribution complexity

---

## Protocol Implications of Hybrid Model

### App → Primary Spirit

**Protocol:** TLV over TCP (already exists from Phase 1)

**Message Types:**
```rust
enum AppMessage {
    CreateBubble { id, content, structure, style_hint },
    UpdateBubble { id, content_delta },
    DeleteBubble { id },
    QueryBubble { id },  // Future: get current state
}
```

**Example:**
```
[Type: CreateBubble][Length: 128][Payload: {
    id: "uuid-1234",
    content: "Getting Started",
    structure: Heading(1),
    style_hint: { size_mult: 1.5, weight: Bold }
}]
```

---

### Primary Spirit → Shadow Spirits

**New protocol needed (scene synchronization)**

**Message Types:**
```rust
enum ShadowMessage {
    SceneUpdate { timestamp, bubbles },       // Full state snapshot
    BubbleCreated { id, bubble },             // Incremental update
    BubbleUpdated { id, delta },              // Incremental update
    BubbleDeleted { id },                     // Incremental update
    SpatialLayout { bubble_id, surface, pos },// Where to render
}
```

**Design choices:**

**A. Full state snapshots (simple)**
- Primary sends entire scene state periodically
- Shadows replace their state
- Inefficient but simple, no complex merging

**B. Incremental updates (efficient)**
- Primary sends only changes since last update
- Shadows apply deltas to their state
- More efficient, but requires sequence numbers/versioning

**C. CRDT-based (robust)**
- Use CRDT data structures (Yjs, Automerge)
- Eventual consistency guarantees
- Handles network partitions gracefully
- Most complex, but most robust

**Recommendation for MVP:** Option A (snapshots)
- Keep it simple initially
- Switch to incremental/CRDT later if performance demands

---

### Shadow Spirit → Renderer

**Protocol:** Local IPC (same machine, low latency)

**Options:**

**A. Message passing (simple)**
- Unix domain sockets
- Named pipes
- Standard serialization (TLV, JSON)

**B. Shared memory (fast)**
- Zero-copy data transfer
- mmap shared buffers
- Synchronization with mutexes/semaphores

**Message Types:**
```rust
enum RenderCommand {
    RenderText {
        font_id,
        size,
        text,
        position,
        surface_id
    },
    RenderGeometry {
        mesh_id,
        transform,
        material_id
    },
    UpdateCamera {
        position,
        orientation,
        fov
    },
    ClearSurface { surface_id },
}
```

**Recommendation:** Start with message passing (A), optimize to shared memory later if needed.

---

## Open Questions & Design Decisions Needed

### 1. App → Spirit Protocol Details

**What's in a Bubble? (Data Model)**

**Option A: Minimal (just content + style hints)**
```rust
Bubble {
    id: UUID,
    content: String,
    style: { bold: bool, size_mult: f32, font_hint: String }
}
```
- Lost: Semantic structure (don't know this was an H1)
- Simpler protocol
- Spirit can't reinterpret semantics

**Option B: Rich (semantic structure preserved)**
```rust
Bubble {
    id: UUID,
    content: String,
    structure: Structure,  // Heading(1), Paragraph, Code, List, etc.
    style_hint: StyleHint, // Optional, Spirit can ignore
}
```
- Retained: "This is an H1" - Spirit can style however it wants
- Richer protocol
- Enables semantic queries (find all headings, extract outline)

**Trade-off:**
- Option A = faster to build, less flexible
- Option B = more future-proof, aligns with Chi's semantic philosophy

**Recommendation:** Option B (rich semantics)
- Aligns with Chi's vision
- Costs little extra complexity now
- Enables powerful features later (search, outline, LLM queries)

---

### 2. Stateful vs Stateless Protocol

**Stateful (Spirit remembers bubbles):**
```
App: CreateBubble("H1: Getting Started")
     → Spirit stores in scene graph
App: UpdateBubble("H1: Getting Started!!!")
     → Spirit updates stored state
```

**Pros:**
- Efficient (send deltas, not full state)
- Enables queries (App: "what's bubble X?")
- Spirit can make decisions based on full scene

**Cons:**
- Spirit must maintain state (memory overhead)
- State can diverge if messages lost
- Need cleanup mechanism (garbage collect old bubbles)

---

**Stateless (Spirit processes, doesn't store):**
```
App: RenderBubble("H1: Getting Started")
     → Spirit routes to renderer, forgets
App: RenderBubble("H1: Getting Started!!!")
     → Spirit routes again, overwrites previous
```

**Pros:**
- Simpler Spirit implementation
- No state management complexity
- No memory overhead

**Cons:**
- Must resend entire state every frame (inefficient)
- Spirit can't answer queries (no history)
- Hard to do spatial reasoning (no scene graph)

---

**Recommendation:** Stateful (Spirit maintains scene graph)
- Necessary for spatial decisions (where to place things)
- Enables rich features (queries, search, outline)
- Aligns with distributed systems (Shadows need state)

---

### 3. Bidirectional Communication (Input Events)

**Scenario:** User clicks on text in VR headset

**Flow:**
```
1. User clicks in VR
2. Renderer detects hit (raycast against geometry)
3. Renderer tells Shadow Spirit: "Click on bubble X at position Y"
4. Shadow Spirit tells Primary Spirit: "Click event on bubble X"
5. Primary Spirit tells App: "User clicked bubble X"
6. App responds (e.g., navigate to section, open link)
```

**Questions:**

**A. Should Shadows forward input to Primary?**
- Yes = Primary is source of truth for app interactions
- No = Shadow handles locally (lower latency, but state divergence risk)

**B. Should Apps receive input events?**
- Yes = Apps are interactive (editors, viewers with navigation)
- No = Apps are write-only (just emit content, don't receive events)

**C. What input event types?**
- Click/tap (select bubble)
- Hover (preview, tooltips)
- Scroll (viewport navigation)
- Keyboard (text input for editors)
- Gesture (VR-specific: pinch, grab, throw)

**Recommendation for Markdown Viewer MVP:**
- **No input events initially** (view-only, no interaction)
- **Add later:** Click to navigate links, scroll to navigate document
- **Future:** Full editing in writer apps

---

### 4. Font Asset Distribution

**Scenario:** jetsone Renderer needs Inter Bold font

**Option A: Pre-installed on each renderer**
```
jetsone:      ~/.local/share/chi/fonts/Inter-Bold.chi_font
Main machine: ~/.local/share/chi/fonts/Inter-Bold.chi_font
```

**Pros:** Fast, no network dependency
**Cons:** Manual sync, version drift, duplication

---

**Option B: Spirit serves font assets on-demand**
```
1. Shadow Spirit tells Renderer: "Use Inter Bold"
2. Renderer checks local cache: not found
3. Renderer asks Shadow: "Give me Inter Bold"
4. Shadow asks Primary: "Give me Inter Bold"
5. Primary streams .chi_font data
6. Renderer caches locally
```

**Pros:** Automatic distribution, version consistency
**Cons:** Network overhead, latency on first use

---

**Option C: Hybrid (local cache + fallback)**
```
1. Renderer checks local cache first
2. If found: use immediately
3. If not found: request from Spirit, cache for next time
```

**Pros:** Best of both (fast + automatic)
**Cons:** More complex cache invalidation

**Recommendation:** Option C (hybrid caching)
- Optimizes for common case (fonts already cached)
- Handles updates gracefully (Spirit can push new versions)
- Aligns with web browser model (familiar pattern)

---

### 5. Material/Shader Assignment

**Question:** Who decides what material/shader to use?

**Option A: Spirit specifies material**
```rust
RenderCommand {
    text: "Hello",
    material: "smoky_glass",  // Spirit decides
    surface: Pane { ... }
}
```

**Pros:** Spirit has full control over aesthetics
**Cons:** Spirit needs to know all available materials

---

**Option B: Renderer chooses based on surface**
```rust
RenderCommand {
    text: "Hello",
    surface: GlassPane,  // Renderer picks material
}
```

**Pros:** Renderer can optimize for hardware (VR might use simpler shaders)
**Cons:** Inconsistent rendering (different renderers might look different)

---

**Option C: Material library with fallbacks**
```rust
RenderCommand {
    text: "Hello",
    material: MaterialRequest {
        preferred: "smoky_glass_v2",
        fallback: "matte",
    }
}
```

**Pros:** Balance of control and flexibility
**Cons:** More complex protocol

**Recommendation:** Start with Option B (Renderer chooses)
- Simpler initially (Spirit doesn't manage materials)
- Renderer can optimize for device capabilities
- Can add Option C later for precise control

---

### 6. Performance: Geometry vs Atlas

**Question:** How to render text for maximum performance?

**Option A: Geometry-based (vector triangles)**
- Triangulate glyphs with lyon
- Upload triangle meshes to GPU
- Render with vertex/fragment shaders

**Pros:**
- Scalable (zoom infinitely, stays sharp)
- True vector rendering
- Enables fancy effects (extrusion, beveling)

**Cons:**
- More triangles (60k for typical document)
- Slower for very small text

---

**Option B: Atlas-based (texture sprites)**
- Pre-render glyphs to texture atlas (like sprite sheet)
- Render quads with texture lookups
- Standard approach (web browsers, games)

**Pros:**
- Very fast (just quads, not thousands of triangles)
- Less memory on GPU
- Battle-tested approach

**Cons:**
- Not scalable (zoom causes pixelation)
- Loses vector properties
- Requires atlas generation

---

**Option C: Hybrid LOD (best of both)**
```
Small text (<16pt):  Atlas-based (fast, quality acceptable)
Medium text (16-48pt): Geometry, coarse LOD (300 tri/glyph)
Large text (>48pt):   Geometry, fine LOD (800 tri/glyph)
```

**Pros:** Optimizes for each case
**Cons:** Most complex, needs both pipelines

**Recommendation for MVP:** Option A (geometry only)
- Simpler, one pipeline
- 60k triangles is trivial for modern GPUs
- Can optimize later if profiling shows bottleneck

**Future:** Add Option C if VR/distant text becomes important

---

## Sources and Sinks (Reader vs Writer Apps)

### Definitions

**Source:** Produces bubbles
- Markdown reader (file → bubbles)
- User input (typing → bubbles)
- LLM generation (prompt → bubbles)
- Network stream (remote content → bubbles)

**Sink:** Consumes bubbles
- Renderer (bubbles → pixels)
- File writer (bubbles → markdown file)
- Network sender (bubbles → remote Spirit)
- Search indexer (bubbles → search database)

**Bidirectional:** Both source and sink
- Markdown editor (file ↔ user edits ↔ display)
- Collaborative whiteboard (network ↔ bubbles ↔ display)
- LLM chat (user input → LLM → output bubbles → display)

---

### Architectural Question

**Does Spirit care about source vs sink?**

**Option A: Spirit is agnostic**
- Apps send `CreateBubble`, `UpdateBubble`, `DeleteBubble`
- Spirit doesn't know if bubble came from file, user, LLM
- Spirit routes to renderers
- Apps are responsible for persistence

**Option B: Spirit tracks provenance**
```rust
Bubble {
    content: "Hello",
    provenance: {
        source: File("/path/to/doc.md"),
        author: User("Ulli"),
        timestamp: "2025-11-29T10:00:00Z"
    }
}
```

- Spirit knows where bubbles came from
- Enables queries: "Show me all LLM-generated content"
- Spirit can route differently (e.g., LLM content gets special styling)

**Recommendation:** Option B (track provenance)
- Aligns with Phase 2 design (bubbles have lineage)
- Minimal overhead (just metadata)
- Enables powerful features (filter by source, audit trail)

---

### Writer App Flow (Markdown Editor)

**Scenario:** User edits markdown file

**Flow:**
```
1. User opens file
2. App reads file, parses markdown
3. App creates bubbles, sends to Spirit
4. Spirit routes to Renderer
5. User sees rendered markdown

6. User clicks to edit
7. Renderer sends input event to Spirit
8. Spirit forwards to App
9. App enters edit mode
10. User types
11. App updates bubbles incrementally
12. Spirit routes updates to Renderer
13. User sees live preview

14. User saves
15. App writes bubbles back to markdown file
```

**Key insight:** Spirit is always involved (bubbles flow through Spirit)

**Alternative (simpler for MVP):**
- App renders locally (doesn't use Spirit for preview)
- Only sends to Spirit for final display
- Trade-off: Loses network transparency, but simpler

---

## Next Steps & Prioritization

### For Markdown Viewer MVP (This Weekend)

**Critical decisions needed:**
1. ✅ Architecture clarified (3-layer: App/Spirit/Renderer)
2. ⚠️ **Bubble data model** (Option B: rich semantics recommended)
3. ⚠️ **Spirit distribution** (Option 3: hybrid Primary+Shadow recommended)
4. ⚠️ **Font strategy** (Option A: runtime triangulation for MVP)
5. ⚠️ **Rendering approach** (Option A: geometry-based recommended)

**Can defer:**
- Material/shader assignment (Renderer chooses)
- Input events (view-only MVP)
- Font asset distribution (local files only)
- Writer apps (focus on reader first)
- CRDT synchronization (single Spirit for MVP)

---

### Implementation Order (If Building This Weekend)

**Phase 1: Standalone Renderer (No Spirit)**
- Just prove font rendering works
- Load Inter.ttf, triangulate with lyon, render with wgpu
- Hard-coded text: "Hello World"
- Goal: See beautiful text on 4K OLED

**Phase 2: Markdown Parsing**
- Add pulldown-cmark (markdown parser)
- Parse KEYBINDINGS.md
- Convert to bubble structs (in-memory, no network)
- Render all bubbles with appropriate fonts/sizes

**Phase 3: Spirit Integration (Optional)**
- Add TLV protocol (already exists from Phase 1)
- Send bubbles to Spirit over TCP
- Spirit routes to Renderer
- Network transparency proven

**Phase 4: Polish**
- Scrolling
- Window resizing
- Multiple markdown files
- Link highlighting

---

### Open Discussions for Tomorrow

1. **Bubble data model finalization**
   - Exactly what fields?
   - Serialization format (TLV payload structure)?

2. **Spirit distribution for MVP**
   - Start with centralized (Model 1)?
   - Or design for hybrid (Model 3) from day one?

3. **Font rendering trade-offs**
   - Accept ~100ms startup for triangulation?
   - Or build Font Baker tool first?

4. **Scope boundary**
   - Just viewer (read-only)?
   - Or interactive (click links, scroll)?

5. **Integration vs standalone**
   - Build as Chi app (uses Spirit)?
   - Or standalone (direct wgpu, prove rendering first)?

---

## Summary of Key Insights

### 1. Three-Layer Separation Works
- **App** = domain expert (markdown syntax)
- **Spirit** = orchestrator (spatial placement)
- **Renderer** = graphics specialist (GPU execution)
- Clean boundaries, each actor does ONE thing

### 2. Recommendations, Not Commands
- Apps suggest presentation ("1.5x larger")
- Spirit interprets based on context (viewing distance, device)
- Flexible, adapts to different outputs

### 3. Hybrid Spirit Distribution (Primary + Shadow)
- Primary on main machine (accepts apps)
- Shadows on remote devices (optimize rendering locally)
- Balances simplicity and performance

### 4. Pre-Baked Fonts Are Future, Runtime Is MVP
- Runtime triangulation: Simple, flexible, good for prototyping
- Pre-baked assets: Fast, shareable, good for production
- Start simple, optimize later

### 5. Proportional Fonts Are Hard But Worth It
- Line wrapping requires pre-shaping
- Baseline alignment across mixed fonts
- Fallback fonts for missing glyphs
- But result is beautiful, readable text (not ugly TUI)

### 6. Rich Semantics Enable Power
- Preserve markdown structure (H1, paragraph, code)
- Don't just render visually
- Enables search, outline, LLM queries
- Future-proof architecture

---

## Token Budget Note

**Current usage:** ~104k / 200k tokens (52%)

**Remaining:** ~96k tokens (enough for continued discussion tomorrow)

This document captures the full architectural discussion. Tomorrow we can:
- Refine specific protocols
- Make final decisions on open questions
- Start implementation (if ready)
- Or continue deeper architectural exploration

---

*Session Date: 2025-11-29*
*Participants: Ulli (human), Sonny (Claude Sonnet 4.5)*
*Context: Chi Phase 2 - Markdown Viewer as first semantic text application*

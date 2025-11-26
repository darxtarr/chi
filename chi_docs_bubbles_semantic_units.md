# Bubbles: Semantic Units of Information

**Last updated:** 2025-11-26 (Sonny, Architecture Session)

---

## Core Concept: Bubbles as Large Tokens

**A bubble is NOT a line of text.**
**A bubble is a conceptually indivisible semantic unit.**

Think "large tokens" - the smallest unit that makes sense to treat as a whole.

### Examples of Bubbles

| Source | Bubble Unit | NOT |
|--------|-------------|-----|
| **Markdown README** | Entire code block | Individual lines of code |
| **Markdown README** | Entire paragraph | Individual sentences |
| **Document** | Raster image (PNG/JPG) | Individual pixels |
| **Document** | SVG graphic | Individual path elements |
| **LaTeX** | Math equation | Individual symbols |
| **Spreadsheet** | Table | Individual cells |
| **LLM output** | Generated code snippet | Token-by-token stream |

**Key insight:** If you can't meaningfully split it without losing semantic coherence, it's a bubble.

---

## Bubble Anatomy

```rust
struct Bubble {
    // Unique identity
    id: BubbleId,  // UUID or similar

    // Content (what it IS)
    content: BubbleContent,

    // Provenance (where it came from)
    provenance: Provenance,

    // Display hints (how it should be rendered)
    display: DisplayHints,

    // Lineage (relationships to other bubbles)
    ancestors: Vec<BubbleId>,
    children: Vec<BubbleId>,

    // Metadata (anything else needed)
    metadata: HashMap<String, Value>,

    // Spatial (where it is in the scene)
    position: SpatialPosition,
}

enum BubbleContent {
    Text(TextBubble),           // Paragraph, heading, etc.
    Code(CodeBubble),           // Code block with language
    Image(ImageBubble),         // Raster or vector
    Equation(EquationBubble),   // LaTeX, MathML
    Table(TableBubble),         // Structured data
    Composite(Vec<BubbleId>),   // Container of other bubbles
}

struct Provenance {
    source: Source,
    timestamp: DateTime,
    author: Author,
    parent_bubbles: Vec<BubbleId>,  // What bubbles created this one
    transformation: Option<Transformation>,
}

enum Source {
    File { path: String, line_range: Range<usize> },
    LLMGenerated { model: String, prompt_id: String, session: String },
    UserCreated { user_id: String },
    Transformed { from: BubbleId, operation: String },
    NetworkReceived { node: String, original_source: Box<Source> },
}

enum Author {
    Human(String),              // User ID
    LLM { model: String },      // Claude, GPT, Gemini, etc.
    System(String),             // Compiler, formatter, etc.
}

struct Transformation {
    operation: String,          // "LLM refactor", "user edit", "syntax highlight"
    parameters: HashMap<String, Value>,
}
```

---

## The Power: Draggable Semantic Units with Lineage

**Scenario:** You're working on a document in VR.

1. **Markdown file appears** as a column of bubbles:
   ```
   [H1: "Introduction" bubble]
   [Paragraph bubble]
   [Code block bubble]  ← You grab this one
   [Paragraph bubble]
   ```

2. **You drag the code block bubble** to a different document in 3D space

3. **The bubble maintains:**
   - Its content (the actual code)
   - Its provenance (came from README.md, lines 42-58)
   - Its lineage (link to parent document bubble)
   - Its identity (same BubbleId)

4. **Spirit on the new document's node:**
   - Receives the bubble via network
   - Knows where it came from (provenance chain)
   - Can render it appropriately (code block style)
   - Maintains CRDT link (eventual consistency)

5. **LLM working on new document:**
   - Sees the bubble
   - Queries: "Where did this code come from?"
   - Gets: "README.md from project X, lines 42-58, authored by user Y"
   - Can provide context-aware suggestions

---

## Information Flow in the Chi World

### Creation

**By User:**
```
User types text → Spirit creates TextBubble
Provenance: UserCreated { user: "ulli" }
```

**By LLM:**
```
LLM generates code → Creates CodeBubble
Provenance: LLMGenerated {
    model: "claude-sonnet-4.5",
    prompt_id: "abc123",
    session: "mentat-2025-11-26"
}
```

**By Import:**
```
Read README.md → Parse markdown → Create bubbles per semantic unit
Provenance: File { path: "README.md", line_range: 42..58 }
```

### Transformation

**User Edit:**
```
Original bubble → User modifies → New bubble spawned
New bubble:
  content: modified text
  provenance: Transformed { from: original_id, operation: "user_edit" }
  ancestors: [original_id]
```

**LLM Refactor:**
```
Code bubble → Send to LLM → LLM refactors → New bubble
New bubble:
  content: refactored code
  provenance: Transformed {
    from: original_id,
    operation: "LLM_refactor",
  }
  author: LLM { model: "claude" }
  ancestors: [original_id]
```

**Filter/Pipeline:**
```
Text bubble → Syntax highlighter → Code bubble with highlighting
Code bubble → Compiler → Error bubbles spawned
Equation bubble → Renderer → Image bubble (rasterized)
```

### Movement

**Drag in 3D space:**
```
Bubble at position (0, 0, 0)
User drags to position (10, 5, 3)

Bubble:
  position: (10, 5, 3)  // Changed
  provenance: unchanged  // Still knows where it came from
  lineage: unchanged     // Still linked to ancestors
```

**Spatial relationships encode semantic relationships:**
- Proximity = related concepts
- Height (Z) = abstraction level
- Clustering = same provenance/author
- Animation = recent activity/change

### Network Flow

**Cross-node movement:**
```
Node A (main PC): Creates bubble
Node B (jetsone VR): User drags bubble from A to B
Node C (think): LLM processes bubble, spawns derivative

CRDT state sync ensures:
- All nodes see bubble with same ID
- Provenance chain intact
- Lineage preserved
- Eventual consistency
```

### Merging

**Combine bubbles:**
```
BubbleA + BubbleB → Composite Bubble
Composite:
  content: Composite([BubbleA.id, BubbleB.id])
  ancestors: [BubbleA.id, BubbleB.id]
  provenance: Transformed {
    operation: "merge",
    from: [BubbleA.id, BubbleB.id]
  }
```

### Splitting

**Break apart composite:**
```
Composite Bubble → User splits → Original children re-exposed
Children maintain:
  - Link to composite parent (lineage)
  - Original provenance
  - Can be re-merged differently later
```

---

## The LLM + Network Transparency + 3D Scene World

### Scenario 1: Collaborative Documentation

**Setup:**
- Ulli on main PC (Fedora desktop)
- LLM (Claude) on think node
- VR viewport on jetsone

**Flow:**
1. **Ulli writes paragraph** → Text bubble created
   - Provenance: User "ulli", File "design.md"
   - Appears in VR scene floating in front of Ulli

2. **Ulli drags bubble to "Claude review" zone** in 3D space
   - Spirit detects zone, sends bubble to think node
   - think node: Claude reviews bubble

3. **Claude generates comment bubble**
   - Provenance: LLM "claude", parent: original text bubble
   - Lineage: ancestor = Ulli's text bubble
   - Position: Spawns adjacent to original in 3D space

4. **Ulli sees both bubbles side-by-side**
   - Spatial proximity shows relationship
   - Can select Claude's suggestion, merge into original
   - New bubble spawned with both as ancestors

5. **All nodes maintain CRDT state**
   - Eventual consistency
   - Any node can render the scene
   - Provenance chain queryable everywhere

### Scenario 2: Code Refactoring Pipeline

**Setup:**
- Code bubble: Rust function
- Pipeline: Human → Claude → Compiler → Human

**Flow:**
1. **Original code bubble**
   - Content: Rust function
   - Provenance: File "main.rs", lines 42-80
   - Author: Human "ulli"

2. **Send to Claude for review**
   - Claude sees bubble + provenance
   - Claude: "This came from main.rs, user ulli wrote it"
   - Claude generates suggestions → 3 variant bubbles
   - Variants:
     - Ancestor: original bubble
     - Provenance: LLM refactor
     - Content: different implementations

3. **User picks variant 2**
   - Drags to "compile" zone
   - Spirit sends to compiler

4. **Compiler produces output**
   - If success: Executable bubble (binary)
   - If errors: Error bubbles (one per error)
   - All have ancestor: variant 2 bubble
   - Provenance: System "rustc"

5. **Spatial layout shows flow:**
   ```
   3D Scene:

   [Original]
       ↓
   [Variant 1] [Variant 2] [Variant 3]
                    ↓
               [Executable]
   ```

6. **Lineage queryable:**
   - "Where did this executable come from?"
   - Answer: Variant 2 → Original → main.rs:42-80 → User ulli

### Scenario 3: Multi-User Markdown Editing in VR

**Setup:**
- Ulli in VR on Quest 3
- Gemini (local model on think) as spatial archivist
- README.md as bubble cloud

**Flow:**
1. **README.md loads** → Bubbles created per semantic unit
   - H1 bubble: "Chi Architecture"
   - Paragraph bubbles
   - Code block bubbles
   - Each knows its line range in file

2. **Ulli drags code block** to new position in 3D space
   - Gemini notes: "Code moved from row 5 to row 12"
   - CRDT updates: logical_position changes
   - Bubble maintains provenance: still README.md:42-58

3. **Ulli says: "Claude, explain this code"**
   - Gemini creates task bubble
   - Task bubble → Claude on think node
   - Claude reads code bubble + provenance
   - Claude generates explanation bubble
   - Explanation spawns adjacent to code in 3D

4. **Second user (remote) joins session**
   - Sees same bubble cloud (CRDT sync)
   - Sees Ulli's avatar position
   - Can interact with bubbles simultaneously

5. **Both users edit different paragraphs**
   - CRDT merges operations deterministically
   - No conflicts (different bubbles)
   - Both see changes in real-time

6. **Export to markdown**
   - Spirit queries bubble positions (logical grid)
   - Reconstructs markdown from bubble order
   - Preserves structure (headings, code blocks, paragraphs)
   - Optionally includes provenance comments

---

## Spatial Semantics: What Position Means

**In traditional compositors:** Position = pixels on screen
**In Chi:** Position = semantic relationship

### Proximity = Relationship
```
[Code Bubble]  [Comment Bubble]
     ↓              ↓
  Close in space = related
```

### Height (Z) = Abstraction Level
```
Z = 10: High-level architecture diagram
Z = 5:  Module overview
Z = 0:  Implementation details
```

### Clustering = Provenance/Author
```
All bubbles from same file → cluster
All bubbles by same author → same color/region
```

### Animation = Activity
```
Bubble pulsing = recently changed
Bubble glowing = LLM currently processing
Bubble fading = old/deprecated
```

### Zones = Operations
```
"Claude Review" zone: Drag bubble here → send to Claude
"Compile" zone: Drag code here → compile
"Archive" zone: Drag here → mark as historical
```

---

## The Ultrathink: Emergent Behaviors

### 1. Living Documentation

Documents aren't static files. They're **bubble ecosystems**.

- **Bubbles spawn derivatives** (code → explanation → diagram)
- **LLMs maintain gardens** (outdated bubbles flagged, updated versions suggested)
- **Spatial layout shows evolution** (timeline of changes in Z-axis)
- **Provenance reveals knowledge flow** (who created what, when, why)

### 2. Collaborative Knowledge Graphs

Multiple humans + multiple LLMs working simultaneously.

- **Bubbles as graph nodes** (content + provenance + lineage)
- **Spatial proximity as graph edges** (semantic relationships)
- **CRDT ensures convergence** (no central authority needed)
- **Provenance enables trust** (know who/what created each bubble)

### 3. Code as Living Artifact

Code isn't text in files. It's **bubbles with lineage**.

- **Every function = bubble** with author, timestamp, parent versions
- **Refactoring = spawning variants** (original + 3 alternatives)
- **Compilation = transformation** (code bubble → executable bubble)
- **Testing = spawning evidence** (test bubbles prove correctness)

### 4. LLM as Spatial Librarian

Gemini on think node maintains spatial order.

- **Intelligent placement:** "This new code bubble relates to architecture, place near diagram"
- **Cleanup:** "These 5 bubbles are outdated, suggest archival"
- **Discovery:** "User is looking at X, surface related bubbles from provenance chain"
- **Routing:** "This question needs Claude, send bubbles to think node"

### 5. Network Transparency Enables Thought Distribution

Computation happens **where it makes sense**, not where user sits.

- **Heavy LLM work:** Route to think (32GB RAM)
- **Rendering:** Route to jetsone (GPU acceleration)
- **User interaction:** Route to Quest 3 (VR input)
- **Bubbles flow seamlessly** between nodes
- **Provenance tracks journey** (created on A, processed on B, rendered on C)

### 6. Temporal Manipulation

Because bubbles maintain lineage, **time becomes navigable**.

- **Rewind:** Show bubble at previous version (ancestor in lineage)
- **Branch:** Show all variants from same parent
- **Merge:** Combine bubbles from different timelines
- **Diff:** Visual diff between bubble and ancestor
- **Blame:** Spatial visualization of "who wrote what when"

### 7. Multi-Modal Fusion

Bubbles can contain **anything semantic**.

- **Text bubble** rendered as geometry (rustybuzz + triangulated glyphs)
- **Code bubble** rendered with syntax highlighting
- **Image bubble** rendered as texture
- **Audio bubble** rendered as waveform visualization
- **3D model bubble** rendered as embedded scene
- **Video bubble** rendered as temporal sequence

All maintain provenance, all draggable, all queryable.

### 8. AI-Augmented Browsing

Traditional browser: HTML → pixels
Chi browser: Markdown → **bubbles**

- **Every heading = bubble** (draggable, annotatable)
- **Every code block = bubble** (executable, modifiable)
- **LLM reads provenance** ("This came from GitHub repo X")
- **LLM suggests related bubbles** ("This code is similar to bubble Y you saw yesterday")
- **Spatial arrangement = reading flow** (scroll = navigate Z-axis)

---

## Implementation Implications

### Protocol Extensions Needed

```rust
// Current Phase 1
enum MessageType {
    Text = 1,
    Image = 2,
}

// Future: Bubble protocol
enum BubbleMessage {
    Create {
        id: BubbleId,
        content: BubbleContent,
        provenance: Provenance,
        ancestors: Vec<BubbleId>,
    },

    Move {
        id: BubbleId,
        position: SpatialPosition,
    },

    Transform {
        source: BubbleId,
        operation: String,
        result: BubbleContent,
    },

    Query {
        id: BubbleId,
        query: ProvenanceQuery,
    },

    Merge {
        sources: Vec<BubbleId>,
        result: BubbleId,
    },
}
```

### CRDT Requirements

**Bubble state must be CRDT:**
- Bubble creation (add-wins)
- Position updates (LWW or vector clock)
- Lineage (append-only set)
- Provenance (immutable once created)

**Scene graph must be CRDT:**
- Bubbles in scene (set CRDT)
- Spatial relationships (commutative operations)
- Transformations (causally ordered)

### Spirit Intelligence Requirements

**Spirit must:**
- Understand bubble semantics (code vs text vs image)
- Route bubbles to appropriate LLMs (based on content)
- Maintain provenance chains (track lineage)
- Render based on provenance (color by author, etc.)
- Enable spatial queries ("what bubbles are near this one?")

**Gemini (local model) must:**
- Maintain spatial coherence (related bubbles clustered)
- Suggest operations (based on provenance patterns)
- Route tasks (send bubbles to appropriate nodes/LLMs)
- Archive old bubbles (based on age + activity)

---

## The Vision: Information as First-Class Spatial Objects

In Chi's world:

**Information isn't trapped in files.** It's bubbles floating in semantic space.

**Bubbles maintain identity.** Drag them anywhere, they remember where they came from.

**Provenance is first-class.** Every bubble knows its lineage, author, creation context.

**LLMs are collaborators.** They read provenance, understand context, generate derivatives.

**Network is transparent.** Bubbles flow between nodes, maintain consistency via CRDTs.

**Space is semantic.** Proximity = relationship, height = abstraction, clustering = provenance.

**Time is navigable.** Lineage enables rewind, branch, merge, diff.

**Multimodal is native.** Text, code, images, audio, video, 3D - all bubbles.

This isn't a window manager. This isn't a compositor.

**This is a semantic operating system for human-AI collaboration in spatial computing.**

---

## Next Steps (Beyond Phase 2)

**Phase 3: Bubble Protocol**
- Extend TLV to support bubble messages
- Add provenance tracking
- Implement lineage chains

**Phase 4: CRDT Integration**
- Bubble state as CRDT
- Scene graph as CRDT
- Network sync

**Phase 5: LLM Integration**
- Gemini on think node
- Provenance-aware queries
- Intelligent routing

**Phase 6: VR Interaction**
- Quest 3 spatial input
- Bubble dragging in 3D
- Gesture-based operations

**Phase 7: Collaborative Sessions**
- Multi-user bubble manipulation
- Shared semantic space
- Convergent editing

---

**This is the ultrathink. This is what we're building.**

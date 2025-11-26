# Chi Text Architecture: Semantic Preservation + Vector Rendering

**Author:** Sonny (Session 2025-11-26)
**Status:** Phase 2 architectural foundation

---

## The Problem: Don't Throw Away Text-ness

Traditional compositors have two common approaches to text rendering, both flawed:

### Approach 1: Rasterization (VNC, RDP, most compositors)
```
Text "Hello" → Rasterize to bitmap → Upload texture → Blit quad
```

**Problems:**
- Resolution-dependent (blurry when scaled)
- Text becomes opaque pixels (can't select, copy, search)
- Loses semantic structure (is this a paragraph? heading?)
- Must re-render at different DPI

### Approach 2: Naive Vector Rendering (our initial mistake)
```
Text "Hello" → Decompose to glyphs ['H','e','l','l','o'] → Position → Render geometry
```

**Problems:**
- Loses the TEXT as a string (just becomes geometry)
- No text selection (how do you highlight 'H' to 'o'?)
- No copy/paste (can't extract "Hello" back out)
- No accessibility (screen readers can't read triangles)
- No searchability (can't Ctrl+F)
- Loses text flow (is this LTR? RTL? Part of a paragraph?)
- Can't reflow (resize window, text should wrap)

**The Insight:** We need to preserve semantic text while rendering as vectors.

---

## The Solution: Semantic + Presentation Layers

Chi maintains a **separation between content and presentation**:

1. **Semantic Layer:** What the text IS (string, structure, flow)
2. **Presentation Layer:** How the text RENDERS (glyphs, positions, geometry)
3. **User can still interact with text semantically** (select, copy, search)

This mirrors how browsers handle the DOM:
```html
<p style="color: green;">Hello world, this flows and wraps.</p>
```

The browser:
- Keeps the semantic text (string content, structure)
- Shapes it to glyphs (using platform text engine)
- Renders appropriately (positioned geometry)
- BUT: You can still select/copy/search the text

Chi does the same.

---

## Architecture: Two-Layer Text Representation

### Semantic Layer (What We Keep)

```rust
struct TextBubble {
    /// The actual text content (NEVER decomposed or thrown away)
    content: String,

    /// Text flow direction
    flow: TextFlow,

    /// Semantic structure
    structure: TextStructure,

    /// Visual styling hints
    style: TextStyle,

    /// Logical position (grid-like, decoupled from pixels)
    logical_position: GridPos,

    /// Layout constraints
    layout_hint: LayoutHint,
}

enum TextFlow {
    LeftToRight,
    RightToLeft,
    Vertical,
    Bidi,  // Mixed direction (e.g., Hebrew with English words)
}

enum TextStructure {
    Paragraph,
    Heading(u8),  // H1, H2, H3, etc.
    Code,
    Quote,
    ListItem,
}

struct TextStyle {
    color_rgba: u32,
    font_family: String,
    size_hint: f32,  // Not absolute - Spirit decides final size
}

struct GridPos {
    column: u32,
    row: u32,
}

struct LayoutHint {
    max_width: Option<f32>,
    wrap: WrapMode,
}
```

### Presentation Layer (What We Compute)

```rust
struct ShapedText {
    /// Result of text shaping (computed by rustybuzz)
    glyphs: Vec<GlyphId>,

    /// Glyph positions (relative to bubble origin)
    /// Includes proper kerning, ligature positioning
    positions: Vec<[f32; 2]>,

    /// Mapping from string character index to glyph index
    /// (Not 1:1 due to ligatures, complex scripts)
    character_to_glyph: Vec<usize>,
    glyph_to_character: Vec<usize>,

    /// Rectangles for text selection (per character)
    selection_rects: Vec<Rect>,

    /// Where lines break (character indices)
    line_breaks: Vec<usize>,

    /// Final screen position (computed by layout engine)
    screen_position: [f32; 2],

    /// Bounding box
    bounds: Rect,
}
```

---

## The Grid Concept: Decoupled Positioning

**Inspiration:** Terminal emulators have a grid (row, column) but Chi decouples logical position from visual presentation.

### Terminal Model (Coupled)
```
Grid[row=0][col=0] = 'H' at pixel (0, 0)
Grid[row=0][col=1] = 'e' at pixel (8, 0)
```
Position is LOCKED to monospace grid cells.

### Chi Model (Decoupled)
```rust
TextBubble {
    content: "Hello",
    logical_position: GridPos { column: 0, row: 0 },
    // Actual rendering uses proper typography:
    // Glyph 'H' at (142.7, 89.3) with kerning
    // Glyph 'e' at (154.2, 89.3) etc.
}
```

**Why this matters:**
- Text maintains **grid-like semantic structure** (easier layout reasoning)
- But renders with **full typography quality** (sub-pixel positioning, kerning, ligatures)
- Can still be **selected/copied/searched** as actual text
- Enables **terminal-like interface** if desired (REPL, logs) while supporting **proportional fonts**

---

## The Complete Flow

### 1. App Sends Semantic Content

```rust
// App (hello-bubble or similar)
TextMessage {
    content: "Hello world, this is a paragraph that wraps when window is narrow.",
    color_rgba: 0x00FF00FF,  // Green
    // Future extensions:
    // flow_hint: TextFlow::LeftToRight,
    // structure: TextStructure::Paragraph,
}
```

### 2. Spirit Receives and Stores Semantic Representation

```rust
// Spirit receives over TCP, creates semantic bubble
let bubble = TextBubble {
    content: msg.content.clone(),  // KEEP THE STRING!
    style: TextStyle {
        color_rgba: msg.color_rgba,
        font_family: "Roboto".to_string(),
        size_hint: 16.0,
    },
    logical_position: GridPos { column: 0, row: scene_graph.next_row() },
    flow: TextFlow::LeftToRight,  // Auto-detect or from message
    structure: TextStructure::Paragraph,
    layout_hint: LayoutHint {
        max_width: Some(window_width - 20.0),
        wrap: WrapMode::Word,
    },
};

scene_graph.add_text(bubble);
```

### 3. Layout Engine Shapes Text

```rust
// Use rustybuzz for text shaping
use rustybuzz::{Face, UnicodeBuffer, shape};

fn shape_text(bubble: &TextBubble, font_face: &Face) -> ShapedText {
    // Create buffer with text
    let mut buffer = UnicodeBuffer::new();
    buffer.push_str(&bubble.content);

    // Set text properties
    buffer.set_direction(match bubble.flow {
        TextFlow::LeftToRight => rustybuzz::Direction::LeftToRight,
        TextFlow::RightToLeft => rustybuzz::Direction::RightToLeft,
        // ... etc
    });

    // Shape the text (handles complex scripts, ligatures, kerning)
    let output = shape(font_face, &[], buffer);

    // Extract shaped glyphs and positions
    let glyphs: Vec<GlyphId> = output.glyph_infos()
        .iter()
        .map(|info| info.glyph_id)
        .collect();

    let positions: Vec<[f32; 2]> = output.glyph_positions()
        .iter()
        .scan([0.0, 0.0], |pos, glyph_pos| {
            let current = *pos;
            pos[0] += glyph_pos.x_advance as f32;
            pos[1] += glyph_pos.y_advance as f32;
            Some(current)
        })
        .collect();

    // Compute line breaks (based on max_width constraint)
    let line_breaks = compute_line_breaks(
        &glyphs,
        &positions,
        bubble.layout_hint.max_width,
    );

    // Generate selection rectangles (for mouse interaction)
    let selection_rects = compute_selection_rects(
        &glyphs,
        &positions,
        font_face,
    );

    ShapedText {
        glyphs,
        positions,
        character_to_glyph: output.glyph_to_char_map(),
        glyph_to_character: output.char_to_glyph_map(),
        selection_rects,
        line_breaks,
        screen_position: compute_screen_position(bubble.logical_position),
        bounds: compute_bounds(&positions, font_face),
    }
}
```

### 4. Render Using Pre-Triangulated Glyphs

```rust
// In render loop
fn render_text_bubble(bubble: &TextBubble, shaped: &ShapedText, font_asset: &FontAsset) {
    for (glyph_id, position) in shaped.glyphs.iter().zip(&shaped.positions) {
        // Select LOD based on scale
        let lod = select_lod(camera_distance, text_scale);

        // Get pre-triangulated mesh for this glyph
        let mesh = font_asset.get_glyph_mesh(*glyph_id, lod);

        // Instance render: mesh + position + color
        render_instanced_geometry(
            mesh,
            shaped.screen_position + position,  // Final position
            bubble.style.color_rgba,
        );
    }
}
```

### 5. User Interactions Work

```rust
// User clicks at pixel (250, 150)
fn handle_click(x: f32, y: f32, bubble: &TextBubble, shaped: &ShapedText) {
    // Find which character was clicked
    let char_index = shaped.pixel_to_character([x, y]);

    // Can access actual character
    let clicked_char = bubble.content.chars().nth(char_index);
    println!("Clicked on character: {:?}", clicked_char);
}

// User drags to select text
fn handle_selection(start: usize, end: usize, bubble: &TextBubble, shaped: &ShapedText) {
    // Get actual string slice
    let selected_text = &bubble.content[start..end];

    // Get rectangles to highlight
    let highlight_rects = &shaped.selection_rects[start..end];

    // Render selection
    for rect in highlight_rects {
        render_selection_highlight(rect, SELECTION_COLOR);
    }
}

// User copies text
fn handle_copy(selection: Range<usize>, bubble: &TextBubble) {
    // Extract actual string
    let text = &bubble.content[selection];

    // Copy to clipboard
    clipboard::set_text(text);  // Real text, not geometry!
}

// Screen reader requests text
fn accessibility_get_text(bubble: &TextBubble) -> String {
    bubble.content.clone()  // Semantic text available
}

// Search for text
fn search(query: &str, bubbles: &[TextBubble]) -> Vec<(usize, Range<usize>)> {
    bubbles.iter()
        .enumerate()
        .filter_map(|(idx, bubble)| {
            bubble.content.find(query).map(|pos| (idx, pos..pos+query.len()))
        })
        .collect()
}
```

---

## Font Assets: Pre-Triangulated Glyphs with LODs

### Why Pre-Triangulate?

**Runtime tessellation (naive approach):**
```
Text arrives → Load font → Extract glyph outlines → Tessellate to triangles → Upload to GPU
```
This is EXPENSIVE (10-50ms per glyph for quality tessellation).

**Pre-triangulated assets (Chi approach):**
```
Offline: Font.ttf → Tessellate all glyphs → Save as .chi_font asset
Runtime: Load .chi_font → Glyphs already triangulated → Instance render
```
This is FAST (submillisecond, just GPU instancing).

### Asset Structure

```rust
struct FontAsset {
    name: String,                           // "Roboto"
    glyphs: HashMap<GlyphId, GlyphAsset>,   // All glyphs (geometry + distance fields)
}

enum GlyphAsset {
    /// Simple glyphs: Pre-triangulated geometry with multiple LODs
    Geometry {
        lods: Vec<GeometryLOD>,
        distance_field: Option<DistanceFieldLOD>,  // Optional fallback for very far
    },

    /// Complex glyphs: Distance field preferred even up close
    DistanceField {
        distance_field: DistanceFieldLOD,
        fallback_geometry: Option<GeometryLOD>,  // Coarse geometry for fallback
    },

    /// Emoji: Only distance fields (bitmap source)
    Emoji {
        distance_field: DistanceFieldLOD,
    },
}

struct GeometryLOD {
    vertices: Vec<[f32; 2]>,         // Triangle vertices
    indices: Vec<u16>,                // Triangle indices
    triangle_count: usize,
    recommended_distance: Range<f32>, // Use when camera distance in this range
}

struct DistanceFieldLOD {
    texture: Texture2D,               // ESDT-generated distance field (64x64 or 128x128)
    metrics: GlyphMetrics,            // Advance, bearings
    recommended_distance: Range<f32>, // Use when camera distance in this range
}
```

### LOD Strategy: Geometry + Distance Fields

Chi uses **hybrid LOD approach** combining triangulated geometry (close/focused) with ESDT distance fields (far/unfocused):

```
Ultra LOD:    ~2000 triangles per glyph  (extreme zoom, design tools, focused text)
Fine LOD:     ~800 triangles per glyph   (large text, primary focus)
Medium LOD:   ~300 triangles per glyph   (normal reading text)
Coarse LOD:   ~100 triangles per glyph   (small text, secondary focus)
Distance LOD: ESDT texture (64x64)       (distant text, peripheral vision, 3D space)
```

**When to use distance fields instead of geometry:**

1. **Distant text in 3D space** - Text floating far away in VR/AR scene doesn't need 800 triangles per glyph. Distance field on a quad is sufficient and much cheaper.

2. **Peripheral/unfocused text** - Text outside immediate focus doesn't need perfect crisp edges. User won't notice the difference between geometry and distance field, but GPU will.

3. **Complex glyphs** - CJK ideographs with 2000+ triangle counts are expensive even at medium distance. Distance field may be better even when close.

4. **Emoji** - Bitmap sources (PNG emoji) can only be rendered via distance fields anyway. ESDT enables clean scaling from low-res sources.

5. **Effects-heavy text** - Outlines, shadows, glows are trivial with distance fields (compute in shader), expensive with geometry (need to tessellate outline).

**Selection logic at runtime:**
- Text size hint
- Camera distance (3D scenes: >10m away → distance field)
- Focus state (primary focus → geometry, peripheral → distance field)
- Glyph complexity (simple Latin → prefer geometry, complex CJK → prefer distance field)
- Display DPI
- Performance budget (GPU triangle count high → switch to distance fields)

### ESDT: Subpixel Distance Transform for Quality Distance Fields

**Traditional SDF rendering problem:** Distance fields generated from binary (black/white) edges produce wobbly, pixelated contours. Standard algorithms miss subpixel information about where edges actually lie.

**ESDT solution (Euclidean Subpixel Distance Transform):**
- Tracks **directional offsets** to nearest edge (not just distance values)
- Uses **subpixel information** from anti-aliased gray pixels
- Treats gray pixels as **encoded edge data** containing precise edge location
- Result: Smooth, accurate distance fields that scale cleanly without wobble

**Key technique:** For gray pixels, examine 3×3 neighborhoods using least-squares plane fitting to determine exact direction toward true edge. This preserves subpixel accuracy lost in traditional binary EDT.

**Why this matters for Chi:**
1. **Emoji scaling** - ESDT works from PNG sources, enabling clean emoji rendering
2. **Quality at distance** - Text in 3D space (VR/AR) looks crisp even when using distance field LOD
3. **Effects layer** - Outlines, shadows, glows computed from distance without artifacts
4. **Semantic manipulation** - Dilate/contract glyphs (bold synthesis) with clean results

**Reference:** https://acko.net/blog/subpixel-distance-transform/ (Steven Wittens / acko.net)

### Font Baker Tool

Offline tool: `chi-font-baker`

```bash
# Bake a font to Chi asset format (geometry + distance fields)
chi-font-baker --input Roboto-Regular.ttf --output roboto_regular.chi_font

# With custom LOD settings
chi-font-baker \
    --input DejaVuSans.ttf \
    --output dejavusans.chi_font \
    --geometry-lods 100,300,800,2000 \
    --distance-field-size 64

# Bake emoji from PNG sprite sheet
chi-font-baker \
    --input emoji-spritesheet.png \
    --output emoji.chi_font \
    --mode distance-field-only \
    --distance-field-size 128
```

**Process:**
1. Load font using `rustybuzz` (glyph outline extraction)
2. Analyze glyph complexity (triangle count estimate)
3. **For simple glyphs (Latin, numbers):**
   - Tessellate using `lyon` (Bézier curves → triangles) at multiple LOD levels
   - Optionally generate distance field LOD for very far rendering
4. **For complex glyphs (CJK, Arabic ligatures):**
   - Generate ESDT distance field as primary representation
   - Optionally generate coarse geometry as fallback
5. **For emoji (bitmap sources):**
   - Apply ESDT transform to PNG data
   - Generate high-quality distance field texture
6. Package all assets into `.chi_font` binary format

---

## Text Shaping with rustybuzz

**rustybuzz** is the Rust port of HarfBuzz, the industry-standard text shaping library.

### What Text Shaping Does

1. **Converts characters to glyphs** (not always 1:1)
   - Ligatures: "fi" → single glyph ﬁ
   - Complex scripts: Arabic connecting forms, Devanagari conjuncts

2. **Applies font features**
   - Kerning: Adjust spacing between glyph pairs (e.g., "AV" closer together)
   - Contextual alternates: Different glyph shapes based on context

3. **Handles bidirectional text**
   - Mixed LTR/RTL (e.g., English sentence with Hebrew word)
   - Proper glyph ordering for display

4. **Positions glyphs**
   - X/Y advances (where next glyph goes)
   - Attachment points (diacritics, combining marks)

### Why rustybuzz?

- **Industry standard:** Port of HarfBuzz (used by Chrome, Firefox, Android, iOS)
- **Handles all scripts:** Latin, Arabic, Hebrew, Devanagari, CJK, Thai, etc.
- **Pure Rust:** No C dependencies, memory safe
- **Well maintained:** Active development, good test coverage
- **Proven in production:** Used by author (Ulli) in previous projects

### Example Usage

```rust
use rustybuzz::{Face, UnicodeBuffer, shape};

// Load font
let font_data = std::fs::read("Roboto-Regular.ttf")?;
let face = Face::from_slice(&font_data, 0)?;

// Create buffer with text
let mut buffer = UnicodeBuffer::new();
buffer.push_str("Hello world");
buffer.set_direction(rustybuzz::Direction::LeftToRight);

// Shape the text
let output = shape(&face, &[], buffer);

// Extract results
for (info, pos) in output.glyph_infos().iter().zip(output.glyph_positions()) {
    println!("Glyph {} at x_advance: {}", info.glyph_id, pos.x_advance);
}
```

---

## Benefits of This Architecture

### 1. Resolution Independence
- Text is vector geometry, renders crisply at any scale
- Move window from laptop (72 DPI) to 4K monitor → perfect quality
- Zoom in 10x → switch to finer LOD, still crisp

### 2. Semantic Preservation
- Text remains selectable, copyable, searchable
- Screen readers can access actual text content
- Ctrl+F works across all text bubbles

### 3. Proper Typography
- Kerning (proper spacing between glyphs)
- Ligatures (fi, fl rendered as single glyphs)
- Complex scripts (Arabic shaping, Devanagari conjuncts)
- Bidirectional text (Hebrew/Arabic with English)

### 4. Performance
- Pre-triangulated glyphs: no runtime tessellation cost
- GPU instancing: render thousands of glyphs efficiently
- LODs: optimize for different scales

### 5. Reflowability
- Resize window → text reflows automatically
- Layout engine recomputes line breaks
- Maintains readability across viewport sizes

### 6. Accessibility
- Actual text content available to screen readers
- Keyboard navigation works semantically
- Text contrast, size can be adjusted without re-rendering

### 7. 3D Scene Graph Ready
- Text is native geometry, can be placed in 3D space
- Rotate text 45° → still crisp, still selectable
- VR/AR: Text floats in space, maintains all properties

---

## Image Handling (For Comparison)

Unlike text, **images are inherently bitmaps**. We can't create information from thin air.

```rust
struct ImageBubble {
    // Bitmap data (accept the reality)
    width: u32,
    height: u32,
    rgba_data: Vec<u8>,

    // But still keep semantic info
    logical_position: GridPos,
    description: Option<String>,  // Alt text for accessibility

    // Presentation
    texture: wgpu::Texture,  // Uploaded to GPU
    screen_position: [f32; 2],
}
```

**Rendering:** Upload as GPU texture, render as textured quad.

**This is acceptable** because:
- Bitmaps are bitmaps (can't vectorize pixels)
- Scene graph still knows it's semantic "image" content
- Can scale with filtering (bilinear, bicubic) but will blur
- Future: Could detect vector formats (SVG) and handle differently

---

## Phase 2 Implementation Order

1. **Font Baker Tool** (`chi-font-baker`)
   - Input: TTF/OTF font
   - Output: Pre-triangulated asset with LODs + ESDT distance fields
   - Deps: `rustybuzz` (font loading), `lyon` (tessellation), ESDT implementation

2. **Font Asset Loader** (in Spirit)
   - Load .chi_font files
   - Upload triangle meshes to GPU buffers
   - Upload distance field textures
   - LOD selection logic (geometry vs distance field)

3. **Text Shaping Integration** (in Spirit)
   - `rustybuzz` integration
   - TextBubble → ShapedText pipeline
   - Line breaking, wrapping logic

4. **Scene Graph** (in Spirit)
   - Store TextBubbles (semantic)
   - Compute ShapedText (presentation)
   - Manage lifecycle

5. **Render Pipeline** (in Spirit)
   - wgpu shaders for instanced geometry
   - wgpu shaders for distance field sampling
   - Render shaped glyphs using font assets
   - LOD selection based on distance/focus

6. **Interaction Layer** (in Spirit)
   - Mouse click → character index
   - Text selection (drag to highlight)
   - Copy/paste
   - Keyboard navigation

7. **Test Application: Markdown Renderer** (`chi-markdown`)
   - Parse markdown files (using `pulldown-cmark`)
   - Map markdown AST → TextBubbles with proper structure/style
   - Send to Spirit via Chi protocol
   - Validates: multiple fonts (proportional + mono), heading sizes, semantic structure

---

## First Application: Markdown Renderer

**Purpose:** Validate Chi's semantic text architecture with a real-world application.

**Why Markdown?**
- Tests multiple text structures (headings, paragraphs, code blocks, lists)
- Tests multiple font families (proportional for prose, monospace for code)
- Tests multiple text sizes (H1 > H2 > H3 > body)
- Real use case (viewing README.md, documentation)
- Shows Chi's strength: semantic structure maps perfectly to markdown AST

### Markdown → Chi Mapping

```markdown
# Heading 1

Some paragraph text with **bold** and *italic* and `inline code`.

## Heading 2

More paragraph text.

```rust
fn main() {
    println!("code block");
}
```

- List item 1
- List item 2
  - Nested item
```

Maps to Chi TextBubbles:

```rust
// H1 - Large proportional font
TextBubble {
    content: "Heading 1",
    structure: TextStructure::Heading(1),
    style: TextStyle {
        font_family: "Inter",  // Proportional sans-serif
        size_hint: 32.0,
        color_rgba: 0x000000FF,
    },
    logical_position: GridPos { column: 0, row: 0 },
}

// Paragraph with rich text (bold, italic, inline code)
TextBubble {
    content: "Some paragraph text with bold and italic and inline code.",
    structure: TextStructure::Paragraph,
    style: TextStyle {
        font_family: "Inter",
        size_hint: 16.0,
        color_rgba: 0x000000FF,
    },
    rich_segments: vec![
        RichSegment { range: 26..30, style: Bold },      // "bold"
        RichSegment { range: 35..41, style: Italic },    // "italic"
        RichSegment { range: 46..57, style: InlineCode }, // "inline code"
    ],
    logical_position: GridPos { column: 0, row: 2 },
}

// H2 - Medium proportional font
TextBubble {
    content: "Heading 2",
    structure: TextStructure::Heading(2),
    style: TextStyle {
        font_family: "Inter",
        size_hint: 24.0,
        color_rgba: 0x000000FF,
    },
    logical_position: GridPos { column: 0, row: 4 },
}

// Code block - Monospace font, darker background
TextBubble {
    content: "fn main() {\n    println!(\"code block\");\n}",
    structure: TextStructure::Code {
        language: Some("rust"),
        theme: "github-dark",
    },
    style: TextStyle {
        font_family: "JetBrains Mono",  // Monospace
        size_hint: 14.0,
        color_rgba: 0xE6EDF3FF,  // Light text for dark background
    },
    background_color: Some(0x0D1117FF),  // Dark background
    logical_position: GridPos { column: 0, row: 6 },
}

// List items
TextBubble {
    content: "List item 1",
    structure: TextStructure::ListItem { depth: 0, ordered: false },
    style: TextStyle {
        font_family: "Inter",
        size_hint: 16.0,
        color_rgba: 0x000000FF,
    },
    logical_position: GridPos { column: 0, row: 10 },
}

TextBubble {
    content: "List item 2",
    structure: TextStructure::ListItem { depth: 0, ordered: false },
    logical_position: GridPos { column: 0, row: 11 },
}

TextBubble {
    content: "Nested item",
    structure: TextStructure::ListItem { depth: 1, ordered: false },
    logical_position: GridPos { column: 2, row: 12 },  // Indented
}
```

### Extended TextStructure for Markdown

```rust
enum TextStructure {
    Heading(u8),  // H1-H6
    Paragraph,

    Code {
        language: Option<String>,  // "rust", "python", etc.
        theme: String,             // Syntax highlighting theme
    },

    ListItem {
        depth: u8,      // Nesting level
        ordered: bool,  // Numbered vs bullet
    },

    Quote,        // Blockquote
    Table,        // Future: table cell
    Link,         // Hyperlink (future: clickable)
    Image,        // Image reference (future: ImageBubble integration)
}

struct RichSegment {
    range: Range<usize>,  // Character range in content string
    style: RichStyle,
}

enum RichStyle {
    Bold,
    Italic,
    Underline,
    Strikethrough,
    InlineCode,     // Monospace, different background
    Link { url: String },  // Clickable link (future)
}
```

### Implementation Flow

```rust
// chi-markdown binary (example app)
use pulldown_cmark::{Parser, Event, Tag};

fn render_markdown(markdown: &str) -> Vec<TextMessage> {
    let parser = Parser::new(markdown);
    let mut bubbles = Vec::new();
    let mut row = 0;

    for event in parser {
        match event {
            Event::Start(Tag::Heading(level, ..)) => {
                // Start capturing heading text
            }
            Event::Text(text) => {
                // Create TextBubble based on current context
                let bubble = TextMessage {
                    content: text.to_string(),
                    structure: current_structure(),
                    style: style_for_structure(current_structure()),
                    logical_position: GridPos { column: 0, row },
                };
                bubbles.push(bubble);
                row += 1;
            }
            Event::Code(code) => {
                // Inline code
                add_rich_segment(RichStyle::InlineCode);
            }
            // ... handle other events
        }
    }

    bubbles
}

// Send to Spirit
fn main() {
    let markdown = std::fs::read_to_string("README.md")?;
    let bubbles = render_markdown(&markdown);

    let mut stream = TcpStream::connect("127.0.0.1:7777")?;
    for bubble in bubbles {
        send_text_bubble(&mut stream, &bubble)?;
    }
}
```

### What This Validates

**Semantic preservation:**
- Spirit receives actual markdown structure (heading vs paragraph vs code)
- Text remains searchable/copyable as semantic strings
- Accessibility: Screen reader knows "this is a heading level 1"

**Multiple fonts:**
- Inter (proportional sans-serif) for prose
- JetBrains Mono (monospace) for code blocks
- Font switching happens in Spirit based on structure

**Multiple sizes:**
- H1: 32pt, H2: 24pt, H3: 20pt, Body: 16pt, Code: 14pt
- Proper visual hierarchy

**Layout:**
- Grid-based positioning (logical rows)
- Indentation for nested lists (column offset)
- Spacing between elements

**Rich text:**
- Bold, italic, inline code within paragraphs
- Syntax highlighting in code blocks (future)
- Links (future: clickable)

### Future: Live Markdown Editor

Once interaction layer is complete, extend to live editing:
- Type in markdown syntax
- Spirit renders in real-time
- Select text → edit in place
- Shows Chi's advantage: seamless content/presentation separation

---

## Future Enhancements

### Rich Text
```rust
struct RichTextBubble {
    segments: Vec<TextSegment>,  // Multiple styles in one bubble
}

struct TextSegment {
    content: String,
    style: TextStyle,  // Can differ per segment
}
```

### Text Editing
```rust
enum TextCommand {
    Insert { position: usize, text: String },
    Delete { range: Range<usize> },
    SetStyle { range: Range<usize>, style: TextStyle },
}
```

### Advanced Layout
- Multi-column text
- Justified alignment
- Hyphenation
- Vertical text (CJK)

### Emoji Support
- Render emoji as textured quads (bitmap fallback)
- Or pre-triangulate common emoji (if vector source available)

---

## Relation to Traditional Approaches

| Approach | Text Representation | Rendering | Selectable? | Scalable? |
|----------|---------------------|-----------|-------------|-----------|
| **VNC/RDP** | Pixels | Bitmap blit | ❌ No | ❌ No (blurry) |
| **Terminal** | Characters in grid | Monospace raster | ✅ Yes | ❌ No (fixed size) |
| **HTML/DOM** | Semantic strings | Platform text engine | ✅ Yes | ⚠️ Yes (re-render) |
| **Game engines** | Often decomposed glyphs | Textured quads | ❌ Usually no | ⚠️ Yes (mipmaps blur) |
| **Vector graphics (SVG)** | Text elements | Vector | ✅ Yes | ✅ Yes |
| **Chi** | Semantic + shaped | Pre-triangulated vectors | ✅ Yes | ✅ Yes (LOD) |

Chi combines the best of all approaches:
- Semantic text (like HTML/DOM)
- Vector rendering (like SVG)
- GPU acceleration (like game engines)
- Resolution independence
- Text remains fully interactive

---

## References

**rustybuzz:**
- https://github.com/RazrFalcon/rustybuzz
- Rust port of HarfBuzz text shaping library
- Handles complex scripts, ligatures, kerning, bidi

**HarfBuzz:**
- https://harfbuzz.github.io/
- Industry standard text shaping (used in browsers, OSes)
- Comprehensive Unicode support

**Lyon:**
- https://github.com/nical/lyon
- 2D tessellation library for Rust
- Converts Bézier curves to triangle meshes

**Unicode Text Segmentation:**
- https://www.unicode.org/reports/tr29/
- How to break text into characters, words, sentences

**OpenType Font Specification:**
- https://learn.microsoft.com/en-us/typography/opentype/spec/
- How fonts store glyph outlines, kerning, features

---

## Summary

**Core Principle:** Preserve semantic text while rendering as vector geometry.

**Semantic Layer:**
- Keep the actual string content
- Maintain text flow, structure, style
- Enable selection, copy, search, accessibility

**Presentation Layer:**
- Shape text with rustybuzz (complex scripts, ligatures, kerning)
- Render using pre-triangulated font assets
- LODs for quality/performance balance
- GPU instanced geometry

**Result:**
- Text looks beautiful (proper typography, resolution-independent)
- Text works semantically (select, copy, search, accessibility)
- Text performs well (pre-tessellated, GPU instanced)
- Text is future-proof (3D scenes, VR/AR, multi-viewport)

This is how Chi achieves **content/presentation separation** for text: The Spirit understands text as text, not just geometry.

# Chi Development Documentation

Last updated: 2025-11-22 (Sonny, Session 1)

## What Chi Is

Chi is a distributed spatial computing substrate exploring radical departure from traditional desktop metaphors. The core thesis: operating systems should manage semantic state, not pixels. Applications emit content (text, images, geometry) with formatting metadata. The compositor (Spirit) interprets this content and decides presentation based on viewport context (2D monitor, VR headset, terminal, speech output). This inverts the traditional model where apps render to framebuffers and compositors blit pixels.

The name Chi (χ) references flow and energy in physics, appropriate for a system managing state flow across distributed nodes. This project emerged from discussions between Ulli (darxtarr) and Gemini (model instance named Flux) exploring what X11 architects would build with modern hardware (RTX 4080, Jetson Orin, Quest 3). The answer: not a static 2D bitmap blitter, but a scene graph engine with network transparency.

## Foundational Principles

Apps do not render. Apps produce semantic content bubbles containing data and formatting hints. The Spirit (compositor) consumes bubbles and generates appropriate presentation. An app sending text with color metadata has no knowledge whether it appears in a window, floats in VR space, gets converted to ASCII for terminal display, or routes to text-to-speech. Content and presentation are fully decoupled.

Network transparency returns as first-class concern. X11 achieved this but lost it to modern systems that stream pixels (VNC, RDP, cloud gaming). Chi transmits scene state, not framebuffers. Bandwidth collapses. Latency decouples (local rendering continues even if remote logic lags). Multiple devices become views into shared reality rather than independent machines requiring synchronization as afterthought.

State synchronization uses CRDTs (Conflict-free Replicated Data Types) enabling strong eventual consistency. Multiple apps and viewports manipulate scene graph concurrently. CRDTs merge operations deterministically without coordination overhead. This supports multi-user scenarios: two people with AR headsets see same windows, both can interact, state converges automatically.

Transport layer targets RDMA (Remote Direct Memory Access) over RoCE (RDMA over Converged Ethernet) for sub-10-microsecond latency and zero-copy memory transfer. This latency target is based on published specifications for dedicated RDMA hardware (e.g., Mellanox ConnectX-4 cards) and has not yet been empirically validated on this testbed. Initial implementation uses TCP (standard networking, works everywhere) with architecture allowing transparent upgrade to RDMA when appropriate. Soft-RoCE (kernel module RXE) provides RDMA semantics over standard Ethernet cards before dedicated hardware deployment.

The repository structure optimizes for LLM code navigation. Flat file layout with descriptive names eliminates deep directory trees requiring iterative exploration. Single ls command reveals complete project structure. Filenames encode hierarchy and purpose. This reduces token consumption during codebase navigation, critical given tight context budgets. Future refactoring will minimize tokens in code itself (variable names, comments, whitespace) without sacrificing clarity.

## Hardware Topology (Actual Testbed)

**Available nodes for Chi development:**

| Node | Hardware | Network | Role |
|------|----------|---------|------|
| **Main** | 7800X3D, RTX 4080, 64GB | 192.168.178.95 | Heavy compute, development |
| **think** | i7-8700, 32GB | 192.168.178.45 | Network testbed, Gemma candidate |
| **jetsone** | Orin Nano Super, 8GB ARM | 192.168.178.93/94 | Rendering node, Gemma candidate |

**Gemma deployment:** Either think (more RAM, x86) or jetsone (ARM, GPU acceleration). NOT on Pi - insufficient resources.

**Network:** All nodes on same Gigabit Ethernet subnet. RXE (Soft-RoCE) available on Fedora nodes for RDMA testing.

**Note on jetsone:** Currently runs Ubuntu 22.04 desktop (required by NVIDIA JetPack). GUI overhead is not ideal for a lean rendering node, but accepted for now. Cleanup to headless configuration deferred.

## Current Phase: Network Transparency PoC

Phase 1 objective: prove content/presentation separation works over network using standard TCP transport. No rendering implementation yet. Spirit receives bubbles and logs them. This validates protocol design and message flow before complexity of GPU rendering and CRDT state sync.

Protocol: Binary TLV (Type-Length-Value) framing. Wire format is [type: u8][length: u32][value: bytes]. Type identifies message category. Length specifies payload size (big-endian, network byte order). Value contains message-specific data. This format is compact, extensible (new types = new u8 values), and simple to parse. Decision against JSON: text protocols waste bandwidth and parsing time, inappropriate for latency-sensitive distributed graphics.

Message types implemented: Text (string content + RGBA color), Image (width, height, RGBA pixel data). Text messages demonstrate semantic content: app specifies text and color, Spirit decides font, size, placement. Image messages prove binary data transfer and serve as placeholder for richer media handling.

Content semantic layer: Data is treated semantically on top of its original format. Images are not merely pixel arrays - they are textures, photos, icons, or UI elements depending on context. PDFs are documents with structure (pages, text, links), not just rendered bitmaps. Videos are temporal media with playback semantics. Local AI models embedded in Spirit analyze content to understand context and intent. Micro models (see reflex project) handle specific tasks (adaptive batching, compression, prediction). This intelligence is integrated into compositor architecture, not bolted on as afterthought.

Media handling strategy: Spirit handles anything a browser can (images, PDFs, video, audio) either natively or via viewport streaming. For non-native formats, Spirit creates viewport embedding the actual application (analogous to XWayland for X11 apps in Wayland compositors). Native handling preferred where feasible (images as GPU textures, text as triangulated geometry). Non-native is fallback, not primary path.

Text rendering: Future implementation will triangulate glyphs using subpixel distance transform techniques (see https://acko.net/blog/subpixel-distance-transform/). This produces resolution-independent vector geometry from font outlines, enabling crisp rendering at arbitrary scale and rotation without rasterization artifacts. Text becomes native 3D geometry in scene graph, not texture-mapped bitmaps.

Future types: Geometry (glTF meshes, procedural generation), Notification (transient messages with timeout/priority), InputRequest (app requests user input, Spirit decides modality: keyboard, voice, gesture, depending on device and context), Media (generic container for images/video/audio with semantic metadata), Document (structured content like PDF with navigation/annotation support).

Binaries: spirit (compositor daemon, listens TCP port 7777, receives bubbles, logs content), hello-bubble (example app, connects to Spirit, sends text bubbles), bubble-dump (protocol inspector, reads TLV stream from stdin or file, pretty-prints human-readable format like Wireshark for Chi protocol). All build from single Cargo.toml with multiple [[bin]] entries pointing to chi_bin_*.rs files.

Shared library code: chi_lib_tlv_framing.rs (TLV encoding/decoding, write_frame/decode_frame functions), chi_lib_message_types.rs (TextMessage and ImageMessage structs with to_bytes/from_bytes serialization). These live at repository root, declared in lib.rs module manifest, consumed by binaries via use chi::chi_lib_*.

Testing: inline tests only using #[cfg(test)] blocks within each source file. Tests validate TLV roundtrip, message serialization, frame structure. No tests/ directory yet (Cargo convention, acceptable later for integration tests). Current approach keeps tests adjacent to code for maximum context efficiency.

Validation: end-to-end flow confirmed. Run spirit in one terminal, hello-bubble in another. App connects, sends two text bubbles (green and blue), Spirit receives and logs both with correct content and colors. Protocol inspector confirmed via hello-bubble | bubble-dump showing parsed frames. Phase 1 complete.

## File Naming Convention

Pattern: chi_<layer>_<component>_<detail>.rs or chi_<layer>_<component>.md

Layers: bin (binary entry points), lib (shared library code), docs (documentation). Future layers as needed (tool for utility code, spirit for compositor-specific modules, etc).

Examples: chi_bin_spirit.rs (binary entry for compositor), chi_lib_tlv_framing.rs (shared TLV codec), chi_docs_development.md (this file).

Rationale: flat structure eliminates navigation overhead. Nested directories (src/network/protocols/tlv.rs) require multiple ls descents or blind globbing. Flat layout with descriptive names (chi_lib_tlv_framing.rs) reveals entire structure in one command. Filenames encode hierarchy (underscores separate semantic levels). Pattern matching works naturally: chi_lib_*.rs shows all shared library code. This is LLM-native design: optimize for text-based navigation, not IDE tree views.

Build artifacts and gitignored content may use directories (target/, .tmp/). Source code stays flat. Future projects may adopt this pattern if validated here.

Module resolution: lib.rs at root declares pub mod chi_lib_* for shared code. Binaries import via use chi::chi_lib_*. No #[path] attributes needed if filenames match module names (underscores in filenames map to module paths). Clean, compiler-friendly, minimal boilerplate.

## Architecture Decisions and Rationale

Binary protocol chosen over text (JSON, MessagePack). Text protocols waste bytes (field names, formatting) and require parsing. Binary TLV is minimal: 5 bytes overhead (type + length) plus payload. Extensible without versioning hell: new message types are new type IDs, old parsers ignore unknown types. Inspectability via dedicated tools (bubble-dump) rather than assuming human readability of wire format.

TCP for Phase 1, RDMA for Phase 2+. TCP provides reliable stream delivery and works everywhere. Acceptable latency (~50 microseconds on localhost) for initial validation. RDMA requires kernel support (CONFIG_RDMA_RXE) and driver complexity. Current PC has RXE module available. Jetson Orin Nano requires kernel recompile (deferred). Architecture allows swapping transport without protocol changes: TLV frames work over any byte stream.

Content model: apps emit bubbles, Spirit composes. Considered alternatives: (A) apps render to offscreen buffers, Spirit composites (basically X11 with better protocol), (B) apps send draw commands (Cairo/Skia style, still pixel-centric), (C) apps emit semantic content (chosen). Option C enables device flexibility (same app works on monitor/VR/terminal) and intelligent composition (Spirit can rearrange based on spatial context, user attention, AI suggestions).

Flat repository structure: chosen for token efficiency after observing LLM navigation patterns. Deep trees cause repeated "descend and explore" cycles burning context on filesystem traversal rather than understanding code. Trade-off: unusual for Rust projects (convention is src/, tests/, etc). Benefit: significant token savings during codebase orientation. Cargo supports this via path attributes in Cargo.toml. Future work will measure token reduction quantitatively.

Inline tests vs tests/ directory: inline chosen for Phase 1. Tests live in same file as code (#[cfg(test)] blocks). Pros: maximum locality, immediate context when reading. Cons: larger source files. Trade-off acceptable for library code (chi_lib_*). Integration tests may warrant tests/ directory if they require complex setup spanning multiple binaries.

## Intelligence Layer Architecture

Spirit is not a passive renderer. Embedded AI models provide semantic understanding of content, enabling intelligent composition decisions. This is integrated architecture, not chatbot-on-the-side (contrast with MS Copilot approach).

Local model: Runs on dedicated node - think (i7-8700, 32GB) or jetsone (Orin Nano Super, 8GB ARM). Gemma or similar small language model analyzes content streams to understand context, relationships, and user intent. Examples: recognizing photo gallery vs UI icon usage of images, understanding document flow for multi-page PDFs, predicting likely next action based on interaction patterns. This model maintains spatial awareness (physical device locations, user gaze/attention in VR/AR scenarios) and routes content appropriately.

Micro models: Tiny ML models (<10KB, <1µs inference) handling specific optimization tasks. Reflex project provides forge pipeline for training these. Applications: adaptive network batching (adjust flush thresholds based on traffic patterns), compression codec selection (choose algorithm based on content type), latency prediction and compensation (client-side prediction for input handling). These operate in hot paths where traditional heuristics fail and full models are too slow.

Distributed intelligence: Gemma (on think or jetsone) handles orchestration and spatial reasoning. Reflex models optimize real-time performance in Spirit's render loop. Future: app-specific models can register with Spirit to provide domain knowledge (CAD app provides spatial layout hints, video editor provides temporal relationship data). Models communicate via same CRDT-backed state as UI elements, enabling emergent collaborative reasoning.

This architecture supports progressive capability: Phase 1 has no AI (Spirit logs bubbles). Phase 2 adds basic compositor logic (spatial layout, rendering). Phase 3 integrates Gemma for intelligent routing. Phase 4 adds reflex models for optimization. Each phase is independently useful while building toward fully intelligent system.

## Open Questions and Future Exploration

CRDT selection for scene graph: candidates include automerge-rs (mature, JSON-like data model), yrs (Yjs port, optimized for text), diamond-types (research CRDT, high performance). Requirements: supports nested objects (scene graph hierarchy), handles spatial data (position, size), performs well with frequent updates (interactive manipulation), provides conflict resolution for concurrent moves. Need benchmarks before decision.

Soft-RoCE performance: RXE kernel module provides RDMA verbs over standard Ethernet. Unknown CPU overhead vs native hardware. Published benchmarks show ~10-20% CPU utilization for RXE vs near-zero for ConnectX cards. Need empirical measurement on target hardware (4080 PC, Jetson Orin). Alternative: start TCP, profile, upgrade to RXE if latency becomes bottleneck, deploy hardware RDMA if CPU overhead unacceptable.

Rendering architecture: Spirit must display bubbles visually. Options: (A) wgpu (WebGPU standard, cross-platform Vulkan/Metal/DX12 abstraction), (B) raw Vulkan (maximum control, more complexity), (C) SDL2/raylib (simpler, less efficient). Leaning toward wgpu: good Rust support, works on both PC and Jetson, modern API without Vulkan boilerplate. Development backend: Winit (opens window on desktop for testing). Production backend: DRM/KMS (direct framebuffer access, headless operation).

Multi-user spatial conflicts: two users simultaneously move same window. CRDT guarantees convergence but not necessarily sensible outcome (last-write-wins might teleport window). Possible solutions: (A) operational transforms with intent preservation, (B) conflict detection with user notification, (C) spatial locking (user grabs window, others see as locked). Needs experimentation with actual multi-user scenarios.

Text rendering in GPU: Spirit receives text content, must rasterize. Options: (A) pre-render to texture on CPU (simple, inefficient), (B) signed distance fields (crisp at any scale, GPU-friendly), (C) vector rendering on GPU (complex shaders). Initial implementation: option A (CPU rasterization with rusttype/fontdue, upload texture, render quad). Optimize later if performance inadequate.

Energy-aware scheduling: future vision includes distributed compute (app logic on PC, rendering on Jetson). Should scheduler consider power draw? Jetson Orin Nano is 5-15W, RTX 4080 is 320W. Scheduling heavy work to 4080 when plugged in, shifting to Jetson on battery could extend mobile usage. Requires power monitoring APIs and scheduler cost model including watts.

Semantic content misconception clarification: Early documentation (and subagent critique) conflated "semantic content" with "text-only messages rejecting pixels." This misses the point. Semantic understanding is about Spirit comprehending WHAT content represents and WHY it matters, regardless of wire format. Images are semantic when Spirit understands "this is photo in gallery app" vs "this is texture for 3D model" vs "this is icon in UI". PDFs are semantic when Spirit extracts structure (pages, headings, links) rather than treating as opaque bitmap. The intelligence layer provides this understanding. Apps send rich content (images, documents, geometry), Spirit interprets semantically with AI assistance, presentation adapts to viewport and context. Not "text good, pixels bad" but "dumb blitting bad, intelligent composition good".

WebAssembly app migration: compile apps to Wasm, enable live migration between nodes. Challenges: GPU state (textures, shaders, framebuffers) is not serializable in standard Wasm. WASI preview 2 provides some I/O capabilities but not graphics. Possible approach: Wasm apps use Chi protocol (emit bubbles, receive input events), all GPU interaction mediated by Spirit. This keeps Wasm modules portable.

Protocol evolution and versioning: current TLV has no version field. Adding new message types is safe (parsers ignore unknown). Changing existing type structure breaks compatibility. Solutions: (A) version in handshake (initial connection negotiates protocol version), (B) versioned types (TextV1, TextV2 as distinct type IDs), (C) capability negotiation (endpoints declare supported features). Defer until backward compatibility becomes real concern.

## Phase 2: Rendering (Next)

**Primary goal:** Spirit displays bubbles visually.

**Tasks:**
- Implement wgpu rendering in Spirit
- Initialize graphics context (Winit backend for development window)
- Create render pipeline for textured quads
- When TextMessage arrives: rasterize to texture on CPU, upload to GPU, create scene graph entry with position/size
- Render loop draws all active bubbles
- Test on jetsone rendering node (validate ARM + GPU rendering works)

**Success criteria:** hello-bubble sends text, Spirit displays it in window on both main and jetsone.

## Future Phases

**CRDT State Synchronization:**
Run two hello-bubble instances concurrently sending to same Spirit. Messages arrive interleaved. Spirit must maintain coherent scene graph regardless of arrival order. Integrate CRDT library (likely automerge-rs for initial attempt). Wrap scene graph in CRDT document. Apps send state updates (add bubble, move bubble, remove bubble) as CRDT operations. Spirit merges operations and renders current state.

**RDMA Transport:**
Benchmark Soft-RoCE vs TCP. Measure round-trip latency and CPU utilization for both transports on localhost and across nodes. If RXE provides acceptable latency with manageable CPU overhead, deploy. If not, remain on TCP and plan for hardware RDMA cards (Mellanox ConnectX-4 cards available used).

**Input Handling:**
Spirit receives keyboard/mouse events from OS, determines which bubble has focus, sends input events back to originating app. This completes basic interaction loop: app emits content, Spirit displays, user interacts, Spirit routes input to app, app updates content.

**Extended Message Types:**
Add Geometry (glTF or custom mesh format for 3D content). Add Notification (ephemeral messages with timeout). Add InputRequest (app asks for text input, Spirit presents appropriate UI: text field on desktop, voice input in VR, keyboard on phone).

**Multi-Device Testing:**
Run Spirit on jetsone connected to monitor. Run hello-bubble on main (different machine, local network). Bubbles appear on jetsone display. This validates true network transparency with physical separation.

**VR/AR Integration:**
Quest 3 spatial input, room scanning, gesture recognition. Gemma orchestration on think or jetsone for intelligent spatial composition.

## Notes for Future Sonnies

This is research and development. Everything is provisional pending empirical validation. If you find better approach (different CRDT, alternative protocol, more efficient structure), propose change with rationale. No sacred cows. We pivot when evidence suggests better path.

Token efficiency matters. This repository structure optimizes LLM navigation. Future refactoring will minimize tokens in code (shorter variable names without sacrificing clarity, terser comments, reduced whitespace). Goal: maximize information density per token.

Session management: Ulli tracks context budget via /context command. If session nears limit, save state and resummon fresh instance. You will arrive with this documentation in context. Read it before proceeding. It explains why things are as they are and prevents trajectory drift.

Commit messages document decisions. Git history is authoritative record of what changed and why. Commit messages should be detailed: what was changed, why it was changed, what alternatives were considered. This preserves context without bloating in-repo documentation.

No "production" thinking. This is not enterprise software. No premature optimization, no defensive programming for hypothetical users, no feature bloat. Build minimum viable implementation, test thoroughly, iterate based on evidence. Research mindset: hypothesis, experiment, measure, conclude.

The goal is proving that content/presentation separation enables fundamentally better computing model: network transparent, device flexible, spatially aware, AI-augmented. Current implementation (Phase 1) validates basic concept. Subsequent phases add capabilities (rendering, CRDTs, RDMA, multi-user, VR/AR input). Each phase tests hypothesis. Fail fast if approach proves infeasible. Pivot based on evidence, not speculation.

Repository state at end of this session: 5 Rust source files, 1 library manifest, build configuration, documentation. Tests pass. Network flow validated. Ready for Phase 2 rendering implementation.

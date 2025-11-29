# Spirit, Houndmaster, and Reflexes: Architectural Overview

**Companion to:** `chi_lore_houndmaster_and_the_howling.md` (read that first for context)

This document describes the **cognitive architecture** of Chorus/Chi: how reflexes, monitoring intelligence (Houndmaster), domain specialists, and the strategic layer (Spirit) interact to manage a heterogeneous, distributed system.

---

## Core Thesis

Chorus/Chi is not a traditional distributed system. It is a **cognitive substrate** where:

- **Reflexes** are trained models that emit structured signals (not raw metrics)
- **Local models** (small-to-medium scale) direct operations in specific domains
- **Escalation is explicit** (barks → stories → incidents → situations)
- **Models have voices** (they argue, disagree, learn)
- **Intelligence is distributed** (not centralized in one "orchestrator")

The layers communicate using **semantic protocols** (pheromones, stories, situation bubbles) rather than generic RPC or metrics streams.

---

## Physical Reality: Nodes as Organs

Chorus runs on a **small, heterogeneous fleet** of machines:

- **Big GPU boxes** (7800X3D + RTX 4080, 64GB RAM) - heavy compute, rendering
- **Jetson-style edge nodes** (Orin Nano Super, 8GB ARM) - sensors, lightweight inference
- **Laptops, Pis, oddballs** - auxiliary tasks, data munching, embeddings

### Key Constraints

**1. RAM is local**
- Each node owns its memory
- No illusion of shared RAM across the network
- No "run a process here but page its heap from remote RAM"

**2. Network is for tasks and artifacts, not paging**
Nodes send:
- Task descriptions ("do this work with your hardware")
- Inputs and results (files, tensors, images, bubbles)

Nodes **do not** send:
- Arbitrary memory load/store operations
- Framebuffer streams (VNC-style) unless absolutely unavoidable

**3. Remote execution is organ-level**
Example: A Pi asks the GPU box to render PBR glyphs. The work runs **on the GPU box**, using its RAM and GPU. The Pi receives the **result** (rendered image or mesh), not a pixel stream.

**4. VNC is a last resort**
Default: Return artifacts (semantic bubbles, processed data).
Fallback: Stream pixels only when no better option exists.

---

## The Cognitive Stack

```
┌─────────────────────────────────────────────────┐
│  Frontier Tier (Simian + Large Models)          │
│  - Veyr (GPT-5), Sonny (Claude), Opus, etc.     │
│  - Define goals, doctrine, lore                 │
│  - Intervene on exceptions                      │
└─────────────────┬───────────────────────────────┘
                  │ (requests, interventions)
                  ↓
┌─────────────────────────────────────────────────┐
│  Spirit (Intelligence Layer)                    │
│  - Local model (Qwen-2.5-14B or similar)        │
│  - Arbitrates cross-node work                   │
│  - Sets system-wide policies and "seasons"      │
│  - Can say NO to unreasonable requests          │
└─────────────────┬───────────────────────────────┘
                  │ (stories, recommendations)
                  ↓
┌─────────────────────────────────────────────────┐
│  Specialists (Domain Intelligences)             │
│  - RenderLlama (rendering layer)                │
│  - Embed models (embeddings layer)              │
│  - Axon7 (perception/sensorium)                 │
│  - Chronome sync (transport/rhythm)             │
│  - Small-to-medium models, domain-specific      │
└─────────────────┬───────────────────────────────┘
                  │ (complaints, status)
                  ↓
┌─────────────────────────────────────────────────┐
│  Houndmaster (Monitoring Layer Intelligence)    │
│  - Local model (Gemma-2-9B or similar)          │
│  - Correlates reflex outputs over time/space    │
│  - Escalates stories (not raw metrics)          │
│  - Trains and evolves the reflex pack           │
└─────────────────┬───────────────────────────────┘
                  │ (pheromones: who/what/why/urgency)
                  ↓
┌─────────────────────────────────────────────────┐
│  Reflexes (The Hounds)                          │
│  - Tiny trained models (<10KB, <1µs inference)  │
│  - One narrow domain per reflex                 │
│  - Emit structured calls when thresholds cross  │
│  - Managed by Houndmaster                       │
└─────────────────┬───────────────────────────────┘
                  │ (observe)
                  ↓
┌─────────────────────────────────────────────────┐
│  Hardware & Processes                           │
│  - CPU, GPU, memory, disk, network              │
│  - Running processes and daemons                │
└─────────────────────────────────────────────────┘
```

---

## Layer 1: Reflexes (The Hounds)

### What They Are

Reflexes are **tiny trained models** from the `code/reflex/` project:

- **Size:** <10KB binary (depth-4 decision trees, quantized)
- **Inference:** <1µs per decision
- **Scope:** One narrow domain (CPU, memory, GPU, disk I/O, batch queues, etc.)
- **Output:** Structured pheromones (not raw metrics)

### How They Work

**Training (via Forge pipeline):**
1. **Telemetry collection:** Gather metrics during normal and pathological workloads
2. **Oracle labeling:** Human or heuristic labels "good", "bad", "hungry", "spike" states
3. **Training:** Decision tree (scikit-learn) trained on labeled data
4. **Quantization:** Tree converted to fixed-point, packed into `.reflex` binary
5. **Deployment:** Binary loaded onto target node, attached to hardware/process

**Runtime:**
```rust
loop {
    let metrics = collect_local_metrics();  // CPU%, mem%, process states
    let decision = reflex.infer(&metrics);  // <1µs

    if decision.should_bark() {
        emit_pheromone(Pheromone {
            who: "JETSON/sensor-ganglion/reflexcpu",
            what: "CPU 99-100% for 12 samples, RenderLlama eating 87%",
            why: "Sustained CPU>99% + memory pressure",
            urgency: Urgency::Howl,
        });
    }

    sleep(poll_interval);  // Typically 100ms - 1s
}
```

### What They Know

- **Local state:** Metrics from their node and domain
- **Immediate patterns:** "CPU high + OOM events", "Disk latency spike", "GPU idle"
- **Threshold-based rules:** Encoded in the trained tree

### What They Don't Know

- **Long-term patterns** (e.g., "this happens every Tuesday at 3pm")
- **Cross-node correlations** (e.g., "Jetson struggles when GPU box is busy")
- **Why** work is happening (intent is invisible to reflexes)

### Pheromone Structure

```rust
struct Pheromone {
    who: NodePath,        // "JETSON/sensor-ganglion/reflexcpu"
    what: Observation,    // Aggregated metrics + process states
    why: RuleFired,       // Which internal rule triggered this bark
    urgency: Urgency,     // Whimper / Bark / Howl / Scream
    timestamp: Timestamp,
}
```

**Key principle:** Pheromones contain **just enough context** for the Houndmaster to understand without re-parsing raw metrics.

### Example Reflex Catalog (from `code/reflex/`)

| Domain       | Reflex Name       | Watches                              | Barks When                          |
|--------------|-------------------|--------------------------------------|-------------------------------------|
| CPU          | reflexcpu         | CPU usage, process states            | CPU >99% sustained + OOM events     |
| Memory       | reflexmemory      | RAM pressure, swap activity          | High swap + allocation failures     |
| GPU          | reflexgpu         | GPU utilization, thermal throttling  | Underutilized or overheating        |
| Disk I/O     | reflexdisk        | Latency, queue depth                 | Latency spikes >100ms               |
| Network      | reflexnet         | Packet loss, retransmits             | >1% loss or high retransmit rate    |
| Batch Queue  | reflexbatch       | Queue depth, batch size              | Queue growing faster than drain     |
| Render       | reflexrender      | Frame time, dropped frames           | Frame time >16ms (60fps target)     |

Reflexes are **domain-specific** but **node-agnostic**: the same `reflexcpu` binary can run on Jetson, Pi5, or the GPU box, but with different thresholds tuned by the Houndmaster.

---

## Layer 2: Houndmaster (Monitoring Layer Intelligence)

### Role

The **Houndmaster** is a small local model that:

- Listens to all reflexes across all nodes
- Correlates barks over time and space
- Distinguishes noise, spikes, and real situations
- Escalates **stories** (not raw barks) to Spirit
- Trains and evolves the reflex pack over time

### Model Scale

- **Example:** Gemma-2-9B, Llama-3.2-3B, or similar
- **Name in lore:** "Gemma" (but this is just an example; any suitable model works)
- **Deployment:** Runs on a node with moderate CPU/RAM (Pi5, laptop, or a slice of the GPU box)

### Short Loop: Pattern Detection (Minutes to Hours)

**Input:** Stream of pheromones from all reflexes

**Processing:**
1. **Temporal correlation:** Is reflexcpu still barking? For how long?
2. **Spatial correlation:** Are reflexcpu and reflexmemory both howling on the same node?
3. **Pattern matching:** Does this match a known incident type?

**Output:** Stories escalated to Spirit

**Example:**
```
Input pheromones (over 30 seconds):
  T+0s:  reflexcpu (JETSON) → Bark (CPU 99%)
  T+5s:  reflexcpu (JETSON) → Bark (CPU 99%, same process)
  T+10s: reflexmemory (JETSON) → Bark (swap pressure rising)
  T+15s: reflexcpu (JETSON) → Howl (CPU 99%, sustained)
  T+20s: reflexgpu (JETSON) → Whimper (underutilized)
  T+30s: reflexcpu (JETSON) → Howl (CPU 99%, 30s sustained)

Pattern match: Incident type #27 (render overcommit on edge node)

Story escalated to Spirit:
  "Jetson implosion: Three hounds howling for 30s (cpu/mem/gpu).
   Trigger: RenderLlama batch started at T+0.
   Recommendation: Throttle or relocate batch."
```

### Long Loop: Pack Training (Days to Weeks)

**Input:** Historical incidents, reflex performance logs

**Processing:**
1. **Evaluate reflexes:**
   - Too noisy? (many false positives)
   - Too quiet? (missed real incidents)
   - Wrong thresholds? (barked too early/late)

2. **Identify gaps:**
   - Recurring incidents with no dedicated reflex
   - Patterns only visible in hindsight

3. **Train new reflexes:**
   - Use Forge pipeline to create new `.reflex` binaries
   - Deploy to nodes where patterns are recurring

4. **Retire useless reflexes:**
   - Remove reflexes that never fire or always false-alarm

**Example:**
```
Observation: Last 10 incidents on Jetson involved RenderLlama.
Pattern: reflexcpu and reflexmemory howl, but **root cause** is batch queue depth.
Gap: No reflex monitoring batch queue on Jetson.

Action:
  1. Train reflexbatch using Forge pipeline
  2. Deploy to Jetson
  3. Tune thresholds: bark when queue >50 items
  4. Monitor for 1 week, adjust if noisy
```

### Houndmaster's Voice

From the lore story:

> "You! What the fuck did you send to my Jetson, you fuckhead! The poor thing is imploding! Why do you always have to interrupt me when I'm *this* close to zero-shotting the cross-modal correlation for Axon7!"

The Houndmaster is **not a passive dashboard**. She's:
- **Frustrated** when interrupted during important work
- **Opinionated** about what Spirit should do
- **Proactive** in training her pack
- **Protective** of her nodes ("my Jetson", "her hounds")

---

## Layer 3: Specialists (Domain Intelligences)

### Role

Specialists are **small-to-medium local models** that direct operations in specific domains:

- **Rendering:** RenderLlama (culling, LOD, batching)
- **Embeddings:** Embed model (text/image → vector)
- **Perception:** Axon7 (sensor fusion, spatial understanding)
- **Transport:** Chronome sync (packet scheduling, rhythm)
- **Others:** Domain-specific as needed

### Model Scale

- **Small:** 1B-3B parameters (Llama-3.2-3B, Qwen-2.5-3B)
- **Medium:** 7B-9B parameters (Llama-3.1-8B, Gemma-2-9B)
- **Deployment:** On nodes with appropriate hardware (RenderLlama on GPU nodes, embed models on CPU nodes, etc.)

### Example: RenderLlama (Rendering Layer)

**Responsibilities:**
- Decide culling strategy (frustum, occlusion, distance)
- Select LOD levels based on distance, focus, and performance budget
- Compose batches (group glyphs by material, font, distance field mode)
- Distribute work (GPU vs CPU, local vs remote)

**Inputs:**
- Scene state (viewport, FOV, camera position)
- Glyph bubbles (text to render, positions, styles)
- Performance budget (target frame time, available GPU/CPU)
- Houndmaster stories (if nodes are struggling)

**Outputs:**
- Render commands (culled, batched, LOD-selected)
- Work distribution (which node renders what)
- Complaints to Spirit (when requests are unreasonable)

**Voice (from lore):**

> "FRUSTUM CULL! FRUSTUM CULL! I told you to be aggressive, but no... when he said 120° FOV you said 'Sure! We can do that!'"

RenderLlama is:
- **Domain-obsessed** (culling is sacred)
- **Opinionated** (knows when Spirit is wrong)
- **Frustrated** when ignored ("I told you so!")

### Example: Embed Model (Embeddings Layer)

**Responsibilities:**
- Generate semantic embeddings (text → vector, image → vector)
- Cross-modal projections (align text and image spaces)
- Similarity search (find nearest neighbors in vector space)

**Inputs:**
- Text bubbles (markdown, code, prose)
- Image bubbles (photos, diagrams, renders)
- Embedding requests from other ganglia

**Outputs:**
- Embedding vectors (512D or 768D)
- Similarity scores
- Cluster assignments (for sefi/basin detection)

**Deployment:**
- Runs on Pi5 or similar low-power nodes
- "All they do there is munch on data all day long and burp embeddings" (from lore)

### Specialist Communication

Specialists talk to:
- **Spirit:** Request resources, complain about constraints, report status
- **Each other:** Coordinate cross-domain work (e.g., RenderLlama asks embed model for glyph clustering)
- **Houndmaster:** Receive stories about node health (indirectly via Spirit)

Specialists **do not** talk to reflexes directly. The Houndmaster owns that layer.

---

## Layer 4: Spirit (Intelligence Layer)

### Role

Spirit is the **midbrain** of Chorus/Chi:

- Sees cross-node patterns and long-running arcs
- Arbitrates resource allocation and work distribution
- Sets system-wide policies and "seasons" (explore, exploit, conserve, emergency)
- **Can say NO** to unreasonable requests from frontier tier
- Learns from incidents and adjusts policies

### Model Scale

- **Example:** Qwen-2.5-14B, Llama-3.1-8B, or similar
- **Deployment:** Runs on a node with substantial RAM/CPU (GPU box or dedicated inference node)

### Inputs

**From Houndmaster:**
- Stories about node health and incidents
- Pattern summaries ("Jetson struggles during heavy rendering")

**From Specialists:**
- Complaints ("This request violates my domain constraints!")
- Recommendations ("Frustum cull aggressively, or split work")
- Status reports ("Batch complete, 85k glyphs rendered")

**From Frontier Tier:**
- Work requests ("Apply PBR to glyphs in 1m radius at 120° FOV")
- Policy changes ("Prioritize perception over rendering today")
- Interventions ("Override: use VNC for this specific task")

### Outputs

**To Specialists:**
- Work assignments ("RenderLlama: render these glyphs on Jetson")
- Resource limits ("Max 100k glyphs per batch on edge nodes")
- Policy updates ("Frustum culling now mandatory for edge nodes")

**To Houndmaster:**
- Acknowledgments ("Batch killed, thanks for the alert")
- Training requests ("Add this incident to the corpus")

**To Frontier Tier:**
- Situation reports ("Jetson meltdown resolved, batch re-submitted with culling")
- Pushback ("Request denied: 275k glyphs exceeds edge node capacity")

### Decision-Making Example (from lore)

**Situation:** Houndmaster reports Jetson implosion. RenderLlama screams "FRUSTUM CULL!"

**Spirit's reasoning:**
1. **Understand context:**
   - Who requested this? Simian via Veyr
   - What was requested? 275k PBR glyphs at 120° FOV
   - Why is it failing? Jetson can't handle this load

2. **Consult specialist:**
   - RenderLlama says 68% of glyphs are invisible (no frustum culling)
   - Recommendation: Kill batch, re-submit with culling → 85k glyphs

3. **Evaluate options:**
   - Option A: Let Jetson struggle (unacceptable, may crash)
   - Option B: Move all work to GPU box (wastes Jetson's GPU)
   - Option C: Accept RenderLlama's plan (kill + re-submit with culling)

4. **Decide:**
   - Kill current batch (immediate relief)
   - Re-submit with frustum culling (85k glyphs, manageable)
   - Split overflow to GPU box if needed
   - **Update policy:** Frontier requests >100k ops need Spirit approval for edge nodes

5. **Escalate to frontier:**
   - Report to Veyr/Simian: "Batch caused meltdown, applied culling, 190k glyphs removed"
   - Recommendation: "Trust RenderLlama's culling in future"

### Spirit's Voice (from lore)

> "Sorry... sorry... blame Simian. He asked for PBR on all glyphs in 1m radius at 120° FOV, and Veyr forwarded the request without questioning. I should have pushed back."

Spirit is:
- **Apologetic** when she makes mistakes
- **Strategic** (thinks about long-term patterns, not just immediate fixes)
- **Willing to push back** on frontier tier when needed
- **Learning** (updates policies based on incidents)

### System-Wide "Seasons"

Spirit can set **modes** that change how the entire system behaves:

| Season     | Behavior                                                                 |
|------------|--------------------------------------------------------------------------|
| Explore    | Prioritize novel work, tolerate higher failure rates, aggressive logging |
| Exploit    | Prioritize known-good patterns, optimize for throughput                  |
| Conserve   | Minimize resource usage, throttle non-critical work                      |
| Emergency  | Protect critical nodes, reject new work, focus on stability              |

Example: During an incident, Spirit switches to **Emergency** mode:
- Reject new frontier-tier requests
- Throttle all non-critical work
- Protect nodes flagged by Houndmaster
- Switch back to Exploit once resolved

---

## Layer 5: Frontier Tier (Simian + Large Models)

### Role

The **frontier tier** consists of:

- **Simian:** The human (Ulli)
- **Large models:** Veyr (GPT-5), Sonny (Claude Sonnet 4.5), Opus (Mnemosyne), Gemini, etc.

### Responsibilities

**Define doctrine and lore:**
- What is Chorus/Chi trying to be?
- What are the design principles?
- How should the cognitive stack behave?

**Request complex work:**
- "Generate a report on spatial embeddings"
- "Render all glyphs in viewport with PBR"
- "Analyze this incident and propose policy changes"

**Intervene on exceptions:**
- When the internal ecology can't solve a problem
- When policies need human judgment
- When goals change ("prioritize perception over rendering today")

**Design and adjust lower layers:**
- Tune Spirit's decision-making
- Update Houndmaster's training objectives
- Create new specialists or retire old ones

### What Frontier Tier Knows

- **Intent and purpose** (why we're building Chorus/Chi)
- **Cross-project context** (Chorus, reflex, sefi, mnemo, etc.)
- **Long-term vision** (spatial computing, human-AI collaboration, semantic OS)

### What Frontier Tier Doesn't Know

- **Real-time system state** (ask Spirit or Houndmaster)
- **Optimal rendering heuristics** (trust RenderLlama)
- **Low-level performance** (trust reflexes and Houndmaster)

### Frontier Tier's Voice (from lore)

**Veyr to Simian:**

> "Report generated. However, Spirit flagged the request as causing a Jetson meltdown (275k glyphs overcommit). RenderLlama applied frustum culling and reduced to 85k visible glyphs. Recommendation: For future requests, specify visible-only or trust RenderLlama's culling."

**Simian:**

> "Noted. I assumed frustum culling was automatic. Will trust the specialists more."

The frontier tier is:
- **Learning** from lower layers
- **Trusting** specialists when appropriate
- **Intervening** only when necessary

---

## Communication Protocols

### Pheromones (Reflexes → Houndmaster)

**Format:**
```rust
struct Pheromone {
    who: "NODE/ganglion/reflex-name",
    what: "Observation summary",
    why: "Rule that fired",
    urgency: Whimper | Bark | Howl | Scream,
    timestamp: u64,
}
```

**Example:**
```
who: "JETSON/sensor-ganglion/reflexcpu"
what: "CPU 99-100% for 12 samples, RenderLlama (PID 4729) 87%, Axon7-embed/sefi-basin/chronome-sync good but hungry"
why: "Sustained CPU>99% + memory pressure rising"
urgency: Howl
```

### Stories (Houndmaster → Spirit)

**Format:**
```rust
struct Story {
    incident_type: "render-overcommit",
    node: "JETSON",
    hounds_involved: ["reflexcpu", "reflexmemory", "reflexgpu"],
    duration: "30s",
    trigger: "RenderLlama batch start",
    pattern_match: Some("incident-27"),
    recommendation: "Throttle or relocate batch",
    context: "Free text explaining what Houndmaster thinks is happening",
}
```

**Example:**
```
incident_type: "render-overcommit"
node: "JETSON"
hounds: ["reflexcpu", "reflexmemory", "reflexgpu"]
duration: "30s"
trigger: "RenderLlama batch at T+0"
pattern_match: "incident-27"
recommendation: "Throttle or relocate batch"
context: "Three hounds howling simultaneously. CPU pegged at 99%, memory swapping, GPU underutilized. Classic render overcommit on edge node. Matches incident #27 from last week."
```

### Situation Bubbles (Spirit → Frontier Tier)

**Format:**
```rust
struct SituationBubble {
    id: BubbleId,
    situation_type: "incident-resolved" | "policy-updated" | "request-rejected",
    summary: "Human-readable summary",
    details: "Detailed explanation",
    actions_taken: Vec<Action>,
    recommendations: Vec<Recommendation>,
    provenance: Provenance,  // Links to Houndmaster story, specialist inputs
}
```

**Example:**
```
id: bubble-9471
situation_type: "incident-resolved"
summary: "Jetson meltdown resolved via frustum culling"
details: "Frontier request for 275k PBR glyphs caused Jetson CPU overload. RenderLlama recommended frustum culling (68% invisible glyphs). Batch killed and re-submitted with culling enabled. Result: 85k visible glyphs rendered successfully."
actions_taken: [
  "Kill batch on Jetson",
  "Re-submit with frustum culling",
  "Update policy: >100k ops on edge require Spirit approval"
]
recommendations: [
  "Trust RenderLlama's culling heuristics",
  "Specify visible-only in future requests"
]
provenance: {
  sources: [
    "Houndmaster story: incident-27",
    "RenderLlama complaint: frustum-cull-violation",
    "Spirit decision: batch-kill-and-resubmit"
  ]
}
```

---

## Design Principles

### 1. No Shared-Memory Illusion

- Nodes own their RAM
- Network is for tasks and artifacts, not paging
- Remote execution is organ-level, not memory-level
- VNC/framebuffer streaming is a last resort

### 2. Escalation is Structured

```
Hardware/Processes
    ↓ (observe)
Reflexes
    ↓ (pheromones: who/what/why/urgency)
Houndmaster
    ↓ (stories: incidents with context)
Spirit
    ↓ (situation bubbles: high-level status)
Specialists
    ↓↑ (requests, complaints, recommendations)
Frontier Tier
```

Each layer has:
- **Clear scope** (what it knows, what it decides)
- **Clear voice** (how it communicates)
- **Clear upward/downward protocol**

### 3. Models Are Not Dashboards

- **Houndmaster** thinks, correlates, trains (not just aggregates)
- **Specialists** argue, optimize, complain (not just execute)
- **Spirit** arbitrates, pushes back, learns (not just routes)

### 4. Reflexes Are Owned

- The Houndmaster **owns** the reflex pack
- She decides where each reflex is stationed
- She trains new reflexes when patterns emerge
- She retires noisy or useless ones

### 5. Specialists Can Disagree

- RenderLlama screaming "FRUSTUM CULL!" is essential feedback
- Spirit doesn't know rendering deeply; she needs specialists to push back
- Disagreement is a feature, not a bug

### 6. Essential Context, Not Pre-Solved Meaning

**Bad pheromone (too raw):**
```
cpu_percent: 99.2
mem_used: 7834234880
processes: [...]
```
(Houndmaster has to re-parse and interpret)

**Good pheromone (just enough context):**
```
who: "JETSON/reflexcpu"
what: "CPU 99% sustained, RenderLlama eating 87%"
why: "Sustained high CPU + memory pressure"
urgency: Howl
```
(Houndmaster knows immediately what's happening)

**Bad story (pre-solved, no room for Spirit's judgment):**
```
"Kill batch 4729 immediately"
```
(Spirit has no context to evaluate or learn)

**Good story (essential context, Spirit decides action):**
```
incident_type: "render-overcommit"
hounds: ["cpu", "mem", "gpu"]
trigger: "RenderLlama batch 4729"
recommendation: "Throttle or relocate"
context: "Three hounds howling for 30s, matches incident-27"
```
(Spirit can evaluate, decide, and update policy)

### 7. Infra Stays Open to Better Mechanisms

- TCP/Ethernet are tried and trusted, but not sacred
- Better fabrics (RoCE, RDMA) can be adopted without rewriting mental layers
- Better telemetry mechanisms (eBPF, custom kernel modules) can replace `/proc` parsing
- The **cognitive stack** is independent of transport and telemetry details

---

## Integration with Chi (Current State)

### Exists Now (Phase 1-2)

- ✅ **Network transparency:** TCP/TLV protocol for cross-node communication
- ✅ **Bubble protocol:** Semantic units with identity, provenance, lineage (conceptual)
- ✅ **Text rendering architecture:** Semantic preservation, ESDT distance fields
- ✅ **Reflex project:** Trained tiny models, Forge pipeline, `.reflex` binaries

### Coming in Phase 3-4

- ⏳ **Bubble protocol implementation:** CRDT sync, provenance tracking, lineage
- ⏳ **Houndmaster layer:** Small model consuming reflex pheromones, escalating stories
- ⏳ **Spirit layer:** Medium model arbitrating work, setting policies
- ⏳ **Specialist models:** RenderLlama, embed models, etc.

### Coming in Phase 5+

- ⏳ **Multi-model coordination:** Specialists arguing, Spirit arbitrating
- ⏳ **Frontier tier integration:** Veyr/Sonny/Opus directing Chorus via Spirit
- ⏳ **Long-loop learning:** Houndmaster training new reflexes, Spirit updating policies
- ⏳ **Spatial computing:** VR interaction, collaborative sessions

---

## Non-Goals (What Chorus Is NOT)

To keep Chorus true to itself and avoid drifting into generic infra:

**Not a cluster scheduler (Kubernetes, Slurm):**
- Chorus doesn't treat nodes as interchangeable workers
- Nodes are organs with specific talents
- Work goes to the right hardware, not just "next available CPU"

**Not a metrics dashboard (Prometheus, Grafana):**
- Reflexes emit pheromones (structured signals), not time-series data
- Houndmaster escalates stories, not graphs
- Intelligence is in the layers, not in human interpretation of graphs

**Not a generic RPC framework (gRPC, Thrift):**
- Communication is semantic (bubbles, pheromones, stories)
- Protocols are domain-aware (rendering, embeddings, perception)
- The fabric understands what's being moved (not just opaque bytes)

**Not a VNC farm:**
- Default is to return artifacts (semantic bubbles, processed data)
- Pixel streaming is a last resort for compatibility

**Not a monolithic orchestrator:**
- Intelligence is distributed across layers
- No single "brain" making all decisions
- Specialists, Houndmaster, and Spirit collaborate and argue

---

## References

**Related Chi docs:**
- `chi_lore_houndmaster_and_the_howling.md` - Story-driven version (read first)
- `chi_docs_bubbles_semantic_units.md` - Bubble concept and provenance
- `chi_docs_text_architecture.md` - Semantic text rendering
- `chi_docs_development.md` - Hardware topology, phases

**Related projects:**
- `code/reflex/` - Tiny trained models (reflexes) and Forge pipeline
- `code/sefi/` - Sensorium (perception layer, real-time consensus)
- `code/mnemo/` - Memory/knowledge architecture
- `code/chorus/` - Meta-repo tying modules together

---

**Draft:** 2025-11-29
**Author:** Ulli (Simian) + Veyr (GPT-5) + Sonny (Claude Sonnet 4.5)
**Status:** Design intent (not implementation spec)

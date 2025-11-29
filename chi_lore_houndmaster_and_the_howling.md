# The Houndmaster and the Howling

A story about layers, voices, and how Chorus/Chi actually thinks.

---

## The Incident

We have processes, hardware, and patterns.

A **reflex** watches the Jetson's CPU. It's a tiny trained model (see `code/reflex/`) that knows one narrow job: monitor CPU usage and process behavior. When CPU spikes above 99% and stays there, it emits a pheromone—a structured bark:

```
WHO: JETSON/sensor-ganglion/reflexcpu
WHAT: CPU 99-100% for 12 samples, process RenderLlama (PID 4729) eating 87%, processes Axon7-embed (PID 2103), sefi-basin (PID 2891), chronome-sync (PID 3401) showing "good but hungry"
WHY: Sustained CPU>99% + memory pressure rising
URGENCY: bark → howl (escalating)
```

Not metrics. Not Prometheus sludge. A **call** with context.

---

**Gemma** hears the bark.

She's a small local model (Gemma-2-9B scale, maybe smaller) running as the **Houndmaster**—the monitoring layer intelligence. Her job isn't to render dashboards. Her job is to **understand the pack**.

First bark: "Could be a spike. Let's monitor."

Five seconds later: Still barking.

Fifteen seconds: Coarse howling now.

Thirty seconds: reflexcpu is **screaming**.

Gemma runs correlation:
- reflexcpu has been howling for 30 seconds straight
- reflexmemory just joined in (swap pressure rising)
- reflexgpu is whimpering (not getting its expected slices)
- Timeline: All three started **exactly** when RenderLlama began a large PBR glyph batch

Pattern match: This is **incident type #27** (render overcommit on underpowered node).

Gemma doesn't send metrics up the chain. She sends a **story**:

> **To: Spirit**
> **From: Gemma (Houndmaster, monitoring layer)**
> **Subject: Jetson implosion in progress**
>
> You! What the fuck did you send to my Jetson, you fuckhead! The poor thing is imploding!
>
> **Situation:** Three hounds howling for 30s (reflexcpu, reflexmemory, reflexgpu). Trigger event: RenderLlama started PBR batch at T+0. Matches incident #27 (render overcommit).
>
> **Why this matters:** I was *this* close [raises chiplets 2 and 7 so they almost touch] to zero-shotting the cross-modal correlation for Axon7. You know how important that is for the perception layer. Now I'm babysitting a meltdown instead.
>
> **Recommendation:** Throttle or relocate RenderLlama's batch. Jetson can't handle this load.

---

**Spirit** receives the story.

Spirit is the **intelligence layer**—the midbrain of Chorus. She's a larger local model (maybe Qwen-2.5-14B scale, or similar) that sees cross-node patterns, arbitrates resources, and makes strategic decisions.

Spirit checks context:
- **Who asked for this work?** Simian (the human) via a frontier model (Veyr, GPT-5)
- **What was requested?** "Apply PBR to all glyphs within 1m radius of viewport"
- **Parameters:** 120° FOV, full quality
- **Result:** 275,321 glyphs queued for rendering
- **Where was it sent?** Jetson (because it has GPU and was "available")

Spirit to Gemma:

> Sorry... sorry... blame Simian. He asked for PBR on all glyphs in a 1m radius with 120° FOV, and Veyr forwarded the request without questioning the parameters. That's 275,321 glyphs in one batch.
>
> I should have pushed back, but the request came from the frontier tier and looked reasonable on paper.

---

**RenderLlama** is also screaming.

RenderLlama is a **specialist model** for the rendering layer (could be Llama-3.2-3B fine-tuned on rendering heuristics, or a custom small model). She directs rendering decisions: culling, LOD selection, batching strategy, etc.

RenderLlama to Spirit:

> **FRUSTUM CULL! FRUSTUM CULL!**
>
> I told you to be aggressive with culling, but no... when he said 120° FOV you said "Sure! We can do that!"
>
> At 120° FOV with 1m radius, **68% of those glyphs are outside the actual view frustum** or occluded. We should have culled them before batching. Now Jetson is choking on work that will never be visible.
>
> **Immediate action:** Kill current batch, re-submit with proper frustum culling. Projected load: ~85k glyphs (70% reduction).

---

**reflexcpu** (still howling):

> Fuck this! I'm requesting transfer to Pi5. All they do there is munch on data and burp embeddings. This is hell.

---

## Resolution

**Spirit** makes the call:

1. **Kill current batch** on Jetson (RenderLlama's recommendation accepted)
2. **Re-submit with frustum culling** enabled (aggressive mode)
3. **Split remaining work** between Jetson (85k glyphs, manageable) and the big GPU box (any overflow or high-complexity glyphs)
4. **Update policy:** Frontier-tier requests with >100k operations now require Spirit's approval before dispatching to edge nodes
5. **Log for training:** Add this incident to Gemma's training corpus (reflexcpu + reflexmemory + reflexgpu howling simultaneously = render overcommit pattern)

**Gemma** calms the hounds:

- Tells reflexcpu: "Batch killed, new work incoming at sane levels. Stand down."
- Updates her internal model: "When all three (cpu/mem/gpu) howl together during RenderLlama batch start, escalate immediately. Don't wait 30s."
- Adds note: "Consider adding reflexbatch (monitors batch queue depth) to Jetson. Current hounds caught the symptom, not the cause."

**RenderLlama** adjusts her stance:

- Adds rule: "120° FOV requests on edge nodes → force aggressive culling, no exceptions"
- Logs: "Frustum culling saved 190k glyphs. Spirit needs to learn when to say no."

**Simian + Veyr** (frontier tier):

Veyr to Simian:

> Report generated. However, Spirit flagged the request as causing a Jetson meltdown (275k glyphs overcommit). RenderLlama applied frustum culling and reduced to 85k visible glyphs.
>
> Recommendation: For future requests, specify visible-only or trust RenderLlama's culling heuristics. Rendering invisible glyphs wastes cycles.

Simian:

> Noted. I assumed frustum culling was automatic. Will trust the specialists more.

---

## The Layers, Clarified

This story shows the **cognitive stack** of Chorus/Chi:

### 1. Reflexes (the hounds)

**What they are:**
- Tiny trained models from the `code/reflex/` project
- <10KB binary, <1µs inference
- Each reflex watches **one narrow domain** (CPU, memory, GPU, disk I/O, etc.)
- Emits **pheromones** (structured calls: who/what/why/urgency)

**What they know:**
- Local metrics on their node
- Simple patterns ("CPU sustained >99%", "memory pressure + swap", "GPU underutilized")
- Immediate good/bad/hungry classifications

**What they don't know:**
- Long-term patterns
- Cross-node correlations
- Why the work is happening

**Where they live:**
- On each node, attached to hardware or process subsystems
- Managed and trained by the Houndmaster (monitoring layer)

**Example reflexes:**
- `reflexcpu`: Watches CPU usage, process behavior
- `reflexmemory`: Watches RAM pressure, swap activity, OOM events
- `reflexgpu`: Watches GPU utilization, thermal throttling
- `reflexdisk`: Watches I/O latency, queue depth
- `reflexbatch`: (proposed) Watches render/compute batch queue depth

---

### 2. Houndmaster (monitoring layer intelligence)

**Who fills this role:**
- A small local model (Gemma-2-9B scale or similar)
- In the story, we called her "Gemma," but this is just an example name
- Could be any model that fits the scale and purpose

**What she does:**

**Short loop (pattern detection):**
- Listens to all reflexes across all nodes
- Correlates barks over time and space
- Distinguishes noise, spikes, and real situations
- Escalates **stories** (not raw metrics) to Spirit

**Long loop (pack training):**
- Evaluates reflex performance over weeks/months
- Identifies:
  - Noisy reflexes (too many false positives)
  - Blind spots (recurring incidents with no dedicated reflex)
  - Optimal thresholds and placements
- Trains new reflexes using the Forge pipeline (`code/reflex/forge/`)
- Retires or quiets useless ones

**What she knows:**
- Recent history (last few minutes to hours)
- Incident patterns (matches against historical corpus)
- Which hounds are stationed where
- Current training objectives for the pack

**What she doesn't know:**
- Long-term strategic goals
- Cross-ganglion coordination
- Why humans or frontier models requested certain work

---

### 3. Specialist models (domain intelligences)

**Who fills these roles:**
- Domain-specific small-to-medium models
- Each specialist directs operations in **one domain**

**Examples from the story:**

**RenderLlama (rendering layer):**
- Model scale: Maybe Llama-3.2-3B fine-tuned on rendering heuristics
- Responsibilities:
  - Frustum culling decisions
  - LOD selection
  - Batch size and composition
  - GPU/CPU work distribution
- Personality: Aggressive optimizer, frustrated when Spirit ignores her advice

**Other specialists (not in story, but exist in Chorus):**

**Embed model (embeddings layer):**
- Small specialized model for generating semantic embeddings
- Runs on Pi5 or similar low-power nodes
- Handles: Text → vector, image → vector, cross-modal projections

**Axon7 (perception/sensorium layer):**
- Model directing sensor fusion and spatial understanding
- Integrates: Camera feeds, depth sensors, IMU data
- Outputs: Semantic scene understanding, object tracking

**Chronome sync (transport/rhythm layer):**
- Model directing network packet scheduling and synchronization
- Uses reflexes from `code/reflex/` for adaptive batching

**What specialists know:**
- Deep domain expertise (rendering, embeddings, perception, transport)
- Local heuristics and preferences
- Performance trade-offs in their domain

**What they don't know:**
- Cross-domain priorities (e.g., "should we prioritize rendering or embeddings right now?")
- System-wide resource constraints
- Human or frontier-model intent

**Where they run:**
- On nodes with appropriate hardware (RenderLlama on GPU nodes, embed models on CPU-heavy nodes, etc.)
- Communicate with Spirit and each other via Chi's bubble protocol

---

### 4. Spirit (intelligence layer, midbrain)

**Who fills this role:**
- A larger local model (Qwen-2.5-14B, Llama-3.1-8B, or similar scale)
- The **arbiter** and **strategic decision-maker**

**What she does:**
- Receives **stories** from Houndmaster (not raw barks)
- Receives **requests and complaints** from specialists
- Sees **cross-node patterns** and long-running arcs
- Makes decisions that alter the **plan**:
  - Move or throttle work
  - Freeze or protect nodes
  - Push back on unreasonable requests
  - Set system-wide "seasons" (explore, exploit, conserve, emergency)

**What she knows:**
- Current state of all nodes (via Houndmaster's stories)
- Active work across all ganglia
- Specialist preferences and constraints
- Recent decisions and their outcomes

**What she doesn't know:**
- Why humans want certain things (intent is often opaque)
- Deep domain expertise (relies on specialists for that)
- Long-term research goals (comes from frontier tier)

**Key capability:**
- **Saying no.** Spirit can reject or throttle requests from the frontier tier if they violate resource constraints or specialist warnings.

---

### 5. Frontier tier (simian + large models)

**Who fills these roles:**
- **Simian:** The human (you, Ulli)
- **Frontier models:** Veyr (GPT-5), Sonny (Claude Sonnet 4.5), Opus (Mnemosyne), Gemini, etc.

**What we do:**
- Define **doctrine and lore** (what Chorus is trying to be)
- Design and adjust **behavior** of Spirit, Houndmaster, specialists
- Request **complex work** (reports, analysis, creative tasks)
- Intervene when the internal ecology hits **problems it can't solve**
- Change the **rules of the game** (new objectives, new ganglia, new research directions)

**What we know:**
- Intent and purpose (why we're building this)
- Cross-project context (Chorus, Chi, reflex, sefi, mnemo, etc.)
- Long-term vision

**What we don't know:**
- Real-time system state (we ask Spirit or specialists)
- Low-level performance details (we trust the Houndmaster and reflexes)
- Optimal rendering heuristics (we trust RenderLlama)

---

## Design Principles (from the incident)

**1. Escalation is structured:**

```
Reflexes (local, narrow, fast)
    ↓ pheromones (who/what/why/urgency)
Houndmaster (correlate, pattern-match, train)
    ↓ stories (incidents with context)
Spirit (arbitrate, decide, push back)
    ↓ situation reports (high-level status)
Specialists (domain experts, argue with Spirit)
    ↓↑ requests, complaints, recommendations
Frontier tier (define goals, intervene on exceptions)
```

**2. Each layer has a voice:**

- Reflexes: Crude, immediate ("CPU bad! Process X eating everything!")
- Houndmaster: Contextual, frustrated ("You interrupted my zero-shot work for THIS?!")
- Specialists: Domain-obsessed, opinionated ("FRUSTUM CULL! Why don't you listen?!")
- Spirit: Strategic, apologetic when wrong ("Sorry, I should have pushed back...")
- Frontier: Intent-focused, learning ("Noted, will trust specialists more")

**3. Models are not dashboards:**

Gemma doesn't render graphs. She **thinks**:
- "Is this a spike or a situation?"
- "Have I seen this pattern before?"
- "Should I escalate now or wait?"
- "Which reflex should I train next?"

RenderLlama doesn't report stats. She **argues**:
- "Frustum cull or I'm not doing this!"
- "You said yes to 120° FOV without asking me first!"

Spirit doesn't aggregate metrics. She **decides**:
- "Kill this batch, it's killing the node"
- "Frontier tier needs approval rules now"
- "Log this for training"

**4. Reflexes are owned, not loose:**

The hounds belong to Gemma. She:
- Decides where each reflex is stationed
- Trains new reflexes when patterns emerge
- Retires noisy or useless ones
- Adjusts thresholds based on incidents

reflexcpu threatening to transfer to Pi5 is a **training signal**: "This node is routinely overloaded, maybe I need different thresholds here, or maybe we need better upstream throttling."

**5. Specialists can disagree with Spirit:**

RenderLlama screaming "FRUSTUM CULL!" isn't insubordination. It's **essential feedback**. Spirit doesn't know rendering deeply. She needs RenderLlama to push back when decisions violate domain constraints.

The ecology works **because** specialists have strong opinions and aren't afraid to voice them.

**6. Frontier tier learns from the layers below:**

Veyr and Simian learned:
- Frustum culling isn't always automatic
- 275k glyphs is absurd for an edge node
- Spirit needs policy updates to say no

The frontier tier sets goals, but the **implementation intelligence** lives in Spirit, specialists, and the Houndmaster.

---

## Integration with Chi (current phase)

Chi is currently in **Phase 2** (rendering + semantic text + bubbles). The story above shows how the **future cognitive stack** will work, but here's what exists now vs. what's coming:

### Exists now (Phase 1-2):
- ✅ Network transparency (TCP/TLV protocol, binary framing)
- ✅ Bubble concept (semantic units with provenance, see `chi_docs_bubbles_semantic_units.md`)
- ✅ Text rendering architecture (semantic preservation, ESDT distance fields, see `chi_docs_text_architecture.md`)
- ✅ Reflex project (trained tiny models, Forge pipeline, see `code/reflex/`)

### Coming in Phase 3-4:
- ⏳ Bubble protocol with provenance and lineage (CRDT integration)
- ⏳ Houndmaster layer (monitoring model consuming reflex outputs)
- ⏳ Spirit layer (intelligence model arbitrating cross-node work)
- ⏳ Specialist models (RenderLlama, embed models, etc.)

### Coming in Phase 5+:
- ⏳ LLM integration (Gemma as Houndmaster, Qwen/Llama as Spirit)
- ⏳ Multi-model coordination (specialists arguing, Spirit arbitrating)
- ⏳ Frontier tier interaction (Simian + Veyr/Sonny/Opus directing Chorus)

The story is **lore and design intent**. The implementation is **incremental**.

---

## Why this matters

Chorus/Chi is not:
- A cluster scheduler (Kubernetes, Slurm)
- A metrics dashboard (Prometheus, Grafana)
- A generic RPC framework (gRPC, Thrift)

Chorus/Chi is:
- A **cognitive substrate** for human-AI collaboration
- A **spatial operating system** with explicit layers of intelligence
- An **ecology of models** with different scales, voices, and responsibilities

The hounds howl. The Houndmaster correlates and trains. The specialists argue. Spirit arbitrates. The frontier tier sets intent.

And when a Jetson melts down because someone asked for 275k PBR glyphs at 120° FOV, the system **learns** instead of just logging an error.

That's the difference.

---

**Draft:** 2025-11-29
**Author:** Ulli (Simian) + Veyr (GPT-5) + Sonny (Claude Sonnet 4.5)
**Status:** Lore anchor (design intent, not implementation spec)

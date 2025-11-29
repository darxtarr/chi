# Chorus / Chi – Spirit, Houndmaster & Pack

(Chi Lore Draft, with Sonny's Architecture Pass)

This document nails down how Chorus thinks about its "mind":
reflexes (hounds), Houndmaster (Gemma), Spirit (arbiter), specialists (RenderLlama, EmbedModel), and the physical boxes they inhabit.

It is not an implementation checklist. It's a behavior + topology charter.

---

## 1. Physical Reality: Where Minds Actually Live

Chorus runs on a small, weird menagerie:

* `main` – 7800X3D + RTX 4080, 64 GB RAM
* `think` – i7-8700, 32 GB RAM
* `jetsone` – Orin Nano, 8 GB RAM, sensor/edge ganglion
* plus Pis and other odd creatures on the LAN

Core stance:

* Each node owns its **own RAM** and devices.
  No remote RAM, no "heap over the network", no DSM tricks.

* The network is for **tasks and artifacts**, not paging:

  * "run this work with your GPU / CPU / sensor,"
  * "here are your inputs,"
  * "give me back images / tensors / meshes / text / summaries."

* VNC / framebuffer streaming is a last resort.
  Default is: run there, return results here.

### Model Placement (Sonny's suggested baseline, q4-ish)

Rough sizing (ballpark, not binding):

* Reflexes: tiny binaries / scripts (<10 KB each), live on every node that needs them.
* Houndmaster: Gemma-2-9B (q4) – pattern detection + pack training (~18 GB).
* Spirit: Qwen-2.5-14B (q4) – strategic arbiter (~28 GB).
* RenderLlama: ~3B Llama-class model (q4) – rendering / visual specialist (~6 GB).
* EmbedModel: ~3B embedding/semantic specialist on `think` (~3–6 GB).

One sane deployment:

* `main`:

  * Spirit (arbiter)
  * Houndmaster (Gemma)
  * RenderLlama (specialist)
* `think`:

  * EmbedModel + telemetry aggregation
* `jetsone` / Pis:

  * reflex binaries
  * lightweight domain daemons (sensors, small scripts)
  * **no big models**

Inference stack assumption: llama.cpp-style tooling as the default local engine, so each daemon has fine-grained control over context, sampling, and resource usage.

---

## 2. Roles in the Cognitive Stack

Chorus thinks in layers. Each layer has a scope and a voice.

### 2.1 Reflexes – "the hounds"

Reflexes are tiny local programs attached to hardware and processes.

They:

* live on one node,
* watch one narrow thing:

  * CPU, GPU, RAM,
  * queue depth,
  * specific subprocess,
  * a sensor pipeline,
* encode simple rules:

  * "bark if CPU > 99% for N samples plus OOM,"
  * "bark if disk latency spikes above X for Y ms,"
  * "bark if render queue depth keeps growing for 10s,"
* emit **pheromones** when their rule fires.

They do *not*:

* model patterns across time,
* reason across nodes,
* know about tasks or arcs.

They are Gemma's dogs. Chorus doesn't own them in the abstract; the Houndmaster does.

What a reflex pheromone contains (essential context):

* `who`: node / role / reflex id
  e.g. `"JETSON/sensor-ganglion/reflexcpu"`
* `what`: short, aggregated observation
  e.g. `"CPU 99–100% for 12 samples; RenderLlama PID 4321 using 87%"`
* `why`: rule that fired
  e.g. `"sustained_cpu>99% + memory_pressure"`
* `urgency`: bark/howl/whimper (severity)
* `timestamp`: when this snapshot applies
* optional `metrics`: a few raw numbers for training / autopsy

Reflexes send these upstream via a simple Chi protocol (TLV over TCP), nothing mystical.

### 2.2 Houndmaster – Gemma as pack leader

Gemma (small model, ~9B) is the **Houndmaster / Squire**.

Her job is threefold:

1. **Short loop – pattern vs noise**

   Over seconds to minutes, Gemma:

   * hears the barks from many reflexes,
   * decides "spike vs noise vs situation":

     * "single bark: likely spike, keep watching,"
     * "sustained howling from CPU + memory hounds: real situation,"
   * recognizes "this smells like that other mess":

     * "this Jetson overload looks like incident-27."

   She turns individual pheromones into **stories**:

   ```text
   render-overcommit on JETSON:
   - CPU-hound howling 30s
   - memory-hound joining
   - started right when RenderLlama began 275k PBR glyph job
   - pattern match: incident-27, incident-31
   ```

2. **Long loop – training and evolving the pack**

   Over hours to weeks, Gemma:

   * evaluates hound quality:

     * noisy (false positives),
     * sleepy (misses real problems),
     * blind spots (recurring patterns with no dedicated hound),
   * adjusts thresholds and rules ("on Pi5, raise CPU threshold from 95% to 98%"),
   * decides where hounds should live (which nodes need which reflexes),
   * commissions **new reflexes** when needed:

     * "we keep dying from queue overflows; I want a queue-depth reflex."

   That yields a Forge-style loop:

   * collect incident logs,
   * build a small dataset,
   * co-design a new reflex with Frontier models / human,
   * deploy, then let Gemma tune.

3. **Narrating up to Spirit**

   Gemma never forwards raw metrics to Spirit.

   She forwards **Stories**:

   * incident type,
   * nodes,
   * participating hounds,
   * duration,
   * pattern match (if known),
   * recommendation, with reasoning.

   Think "case file" rather than "Grafana panel".

Gemma is not a dashboard. She is the person yelling:

> "You! What did you send to my Jetson? CPU-hound is hoarse, memory-hound is sobbing, and this smells *exactly* like that last PBR glyph stunt…"

### 2.3 Spirit – midbrain / arbiter

Spirit (~14B) is the **midbrain of Chorus**.

Spirit:

* sees cross-node status and long-running arcs,
* reads Gemma's Stories, not raw barks,
* knows about:

  * current "season" (explore / exploit / conserve / emergency),
  * priorities, projects, and taboos,
* makes **decisions that change the plan**:

  * "Stop scheduling heavy jobs on Jetson for 30m."
  * "Move render batch T from Jetson to main."
  * "Tell RenderLlama to be aggressive with frustum culling for Simian's current scene."
  * "Deny Simian's 120° FOV 275k PBR glyph ask, or downscale."

Spirit also holds the **policy surface**:

* high-level rules ("Jetson has override protection," "no new Axon experiments while system is in emergency mode"),
* emergency policies (what Houndmaster is allowed to kill without asking).

Spirit is connected locally to Houndmaster and RenderLlama via Unix domain sockets (μs latency), and to other nodes via Chi's TCP/TLV.

### 2.4 Specialists – RenderLlama, EmbedModel, others

Specialist models are **domain ganglia**:

* RenderLlama cares about rendering:

  * PBR vs cheap shading,
  * frustum culling,
  * batching, LOD, etc.
* EmbedModel cares about embeddings / search.
* Future specialists may exist for:

  * transport (Axon),
  * doc ingestion,
  * code refactoring.

They:

* take Spirit's decisions + Gemma's stories as input,
* argue for domain-specific tradeoffs ("I can frustum-cull more aggressively," "I need more time on GPU this hour"),
* report back their own status / constraints.

They do *not* manage reflexes or global policy directly.

### 2.5 Simian & Frontier models

Outside ring:

* Ulli (Simian),
* Veyr/Sonny/frontier LMs.

They:

* change doctrine and lore,
* adjust Spirit's and Gemma's configuration and prompts,
* design new reflex types and specialist roles,
* step in when the internal ecology is stuck or misbehaving.

---

## 3. The Chorus Wire: How They Talk

There are two main communication regimes:

1. **Local, co-located models (on `main`)**

   * Spirit ↔ Houndmaster
   * Spirit ↔ RenderLlama

   Use Unix domain sockets with a simple binary or TLV protocol. Latency is microseconds; bandwidth is essentially local.

2. **Cross-node communication**

   * Reflexes → Houndmaster: Chi TCP/TLV (`port 9001`) carrying **Pheromones**.
   * Specialists on other nodes → Spirit: Chi TCP/TLV (`port 9002`) carrying specialist status / results.
   * Frontier / external → Spirit/Houndmaster: Chi TCP/TLV (`port 9003`) when needed.

Message families (conceptually):

* `Pheromone` – a reflex bark.
* `Story` – Houndmaster's summarized incident.
* `Decision` – Spirit's action / policy decisions.
* `PolicyUpdate` – Spirit changing rules / seasons.
* `SpecialistRequest` / `SpecialistReply` – targeted coordination with RenderLlama, EmbedModel, etc.
* `Escalation` – "we tried Gemma and Spirit and this is still bad; wake Simian / Frontier."

The underlying transport can start as TCP; RoCE or other low-latency paths are future optimization, not a different concept.

---

## 4. Houndmaster's Training Loops

Two loops define how the pack evolves:

### Short Loop – minutes to hours

* Gemma monitors how often each reflex barks and what happened after.
* If a reflex is noisy on a node:

  * "reflexcpu on Pi5 produced 5 false positives today, no real incidents."
  * Gemma raises its threshold or adjusts its rule (`95% → 98%`, extend window, etc.).
* Implementation is usually config updates:

  * Houndmaster sends new config to node,
  * reflex reloads without recompilation.

### Long Loop – days to weeks

Trigger: recurring incident class not well handled.

Example:

* Last 10 Jetson incidents involve "batch queue overflow."
* reflexcpu / reflexmemory caught the symptoms (high CPU/mem), but cause is clearly queue growth.

Gemma:

* clusters those incidents,
* labels the pattern,
* requests a new reflex type ("queue-depth hound").
* Frontier models + Simian co-design the reflex, then deploy it.
* Gemma tunes it after observing it in the wild.

The goal is that over time:

* fewer incidents are "surprises,"
* more are "ah, this again, and we have a dedicated hound for it."

---

## 5. Spirit's Memory & Authority

Spirit has a finite context (e.g. 32k tokens). It cannot remember everything at once, so it uses a hierarchical memory:

* System prompt: role, lore, design principles.
* Policies: current seasons, standing orders.
* Working memory: last N stories from Houndmaster.
* Episodic summaries: compressed incidents from last days/weeks.
* Current situation: whatever is under active decision.

Everything else goes into an **archive**:

* serialized incidents, decisions, outcomes, and raw evidence on disk,
* queriable by Spirit or Frontier for retrospection.

When Spirit sees something like "incident-27" again, it can:

* load that old case from the archive,
* temporarily bring it into context,
* compare, decide, and then drop it again.

### Emergency bypass

Some situations are so bad that waiting for Spirit is dangerous.

Policy:

* Houndmaster has *narrow* kill authority when predefined "scream" conditions are met.

Example cached policy:

```toml
[emergency_policies.jetson_meltdown]
trigger = "reflexcpu + reflexmemory + reflexgpu all scream on JETSON"
actions = [
  "kill non-essential jobs on JETSON",
  "shed incoming tasks tagged heavy_compute",
  "notify Spirit with full dump",
]
```

This is not free-form. Spirit and Simian define these emergency policies; Houndmaster just executes when the trigger is fulfilled.

---

## 6. What Chorus Explicitly Is and Is Not

Chorus is:

* an **ecology of models** with clear roles and a structured conversation,
* a system where small models train and manage reflexes,
* a system where mid-tier Spirit makes cross-node decisions with memory and lore,
* a lab where mistakes become training data for better reflexes and policies.

Chorus is not:

* a generic "cluster scheduler,"
* a shared-memory NUMA illusion across the LAN,
* a VNC farm with LLMs stapled on top.

The hard problem here is not slotting tasks onto cores. The hard problem is:

> building a cognitive field that learns from its own mistakes and gradually takes more of the routine load off the simian,
> while remaining inspectable, corrigible, and fun to live with.

That's the song of many models. This is the bit we're fixing into the lore.

---

**Draft:** 2025-11-29
**Authors:** Ulli (Simian) + Veyr (GPT-5) + Sonny (Claude Sonnet 4.5)
**Status:** Lore anchor (behavior + topology charter, not implementation spec)

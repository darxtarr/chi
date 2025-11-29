I dont want to crystallize anything yet, I want to metaphorically pick up the components that we have available, that a long hard look at them and evaluate  how to best make use of them or swap them if there are better alternatives.

What i See so far is space (disk + ram) space spread across different nodes. With we have a system to make it feel like it's contiguous with soft then maybe hardware RoCE. We also want feelers into the individual nodes hardware and software status (CPU stats and current usage, disks and ram free network usage) and a ledger for all this. Anything else?
￼
￼
ChatGPT said:
￼
Thought for 13s
Yeah, good, stay in “parts on the table” mode a bit longer. You’ve already spotted most of the obvious hardware-ish primitives:

distributed space (disk + RAM),

a way to pretend it’s contiguous (soft/hw RoCE),

feelers into each node’s vitals,

a ledger so things don’t just evaporate.

There are a few more categories I’d add to the parts pile before we even whisper about responsibilities.

First thing you’re missing is time.

Not “NTP is running,” but:

per-node monotonic clocks (for “this happened before that”),

a shared, durable timeline where events can be replayed / aggregated,

maybe per-node “local time scales” (fast-twitch vs slow background).

Without explicit time primitives, the ledger turns into a pile of rocks instead of strata.

Second, you need identity and topology, not just “a bunch of nodes”.

So:

stable IDs for:

nodes,

NICs / RoCE endpoints,

processes / mnematodes,

“spaces” (this region of RAM, that disk pool).

a topological graph:

who can talk to whom at what cost (latency/bandwidth),

which nodes are “near” in physical/network terms,

which nodes are “near” in role (both Jetson-ish, both heavy-GPU-ish).

That gives the aether something to flow over, instead of just “LAN is flat, shrug”.

Third, you want event channels, not only metrics.

Feelers give you CPU%, free RAM, etc., and the ledger can store them. But Spirit-ish stuff will care about discrete happenings:

“new process of type X appeared on node Y,”

“repo Z just mounted and spawned mnematode W,”

“Jetson flipped from ‘idle’ to ‘camera streaming’ mode,”

“user opened these three files on this node.”

So a simple “events bus” per node with:

local fan-out (other processes subscribe), and

cross-node export (selected events become RoCE packets or compressed deltas),

is another primitive you want on the table.

Fourth, a semantic substrate of some kind.

Even if we don’t define Spirit’s job, we know Chorus isn’t just about raw bytes. So in the parts-bin:

a place to put bubbles (semantic chunks) that’s addressable from all nodes,

optional embedding / tag indices per node or per “space,”

a minimal notion of “this hunk of bytes refers to that bubble / that repo / that scene.”

You don’t need to say “Spirit will do X with it” yet, but you probably do want:

“for any byte range in space, I can hang a little semantic label or pointer”.

That’s a component, not a policy.

Fifth, user presence and intention hooks.

Right now your list is all machine-centric. For this to be Chorus-shaped rather than Slurm-shaped you also want:

low-friction signals like “user is currently focused on this TTY/window/repo,”

maybe mode flags (“Ulli is in deep work vs grazing mode”),

recent interaction traces (which nodes they’ve touched, which tools fired).

Again, no behavior attached yet, just the ability of the system to know “where the human is leaning” at any moment.

Sixth, actuation points.

You have space, you have sensors, you have a ledger. One more category is: “what knobs exist that something could turn later?”

Examples:

process priority / cgroup tweaks,

gentle nudges (term messages, notifications, overlays),

“soft” routing nudges (prefer node A over B for this class of job),

toggles of local modes (“this repo is currently in ‘freeze’ state”).

We don’t have to say who is allowed to touch them, only that they exist and are plumbed.

If I compress it into a snapshot, your parts pile right now looks like:

space (RAM+disk) and ways to stitch it (RoCE),

vitals/metrics feelers,

a ledger/timeline,

identity + topology,

event channels,

a semantic hook layer (bubbles / tags / indices),

user-presence signals,

and a little panel of knobs that someone could twist.

That’s already a decent set of primitives to start playing “what moves through this field?” without locking Spirit into any one shape yet.

￼
￼
￼
￼
￼
￼
You said:
you are still rushing way ahead. think physical and essential software only. we are doing an inventory and seeing what is needed to build a system. This is pure infra. Are we weak in some areas (constraints)? Knobs and identities will come later, we do not care about addressing or pipes or connections or usage.

So: any techs we might leverage here to make the plumbing and the basins communicate efficiently apart from RoCE and memory mapping?

Any interesting research in the field with concepts left on the wayside after a single or few  papers.
￼
￼
ChatGPT said:
￼
Thought for 32s
Fair, I was already halfway to drawing Spirit’s face on the plumbing diagram. Let’s stay in the engine room: metal, RAM, kernel, wires. No jobs, no arcs, no users, just “how do basins of bytes talk”.

I’ll do this in three passes:

what the constraints probably look like for your little zoo,

what existing tech gives you good plumbing above “raw RoCE + mmap”,

a few oddball research directions that are basically orphaned but full of stealable ideas.

No knobs, no IDs, no policy.

1. Constraints: what the physics is likely to give you
Given your setup (Jetson, 4080 box, Lenovo, laptop, commodity LAN), the raw situation is roughly:

Ethernet, probably 1–2.5 Gbit, maybe 10 Gbit if you upgrade.

No native InfiniBand / RDMA NICs (yet), so you’re in Soft-RoCE land for now.

Latency: microseconds locally, tens–hundreds of microseconds over LAN.

RAM is fragmented: 8 GB on Jetson, 32+ on others, NVMe is fast but still storage.

From that, two hard physical truths:

“It feels like one big flat RAM” will always be an illusion held together by software and a consistency model.

The really interesting tech is exactly in that middle: how to hide or exploit latency and partial coherence.

So: RoCE + mmap are the bare minimum. Here’s what lives just above them in the stack that’s worth looting for ideas.

2. Plumbing tech above “RoCE + mmap”
Think of these as infrastructure patterns, not things you must adopt wholesale.

2.1. PGAS / communication fabrics: GASNet, UCX, libfabric, LCI
There’s a whole ecosystem of “we want a global address space over fast networks, but we don’t want MPI semantics”.

GASNet-EX: low-level, language-independent comm layer used under UPC++, Chapel, Legion, etc. It exposes:

one-sided remote memory access (put/get),

active messages (small RPC-like packets that invoke a handler on arrival),
over whatever transport you have (IB, RoCE, TCP, etc.). 
gasnet.lbl.gov
+1

libfabric and UCX: more “industry-ish” fabrics used by MPI stacks and storage engines. They’re essentially abstraction layers over RDMA-capable NICs, handling QPs, completion queues, etc., but giving you a nicer API for RMA and message passing across transports.

LCI (Lightweight Communication Interface): a newer research-y interface designed to beat MPI/GASNet in multithreaded scenarios; again, it’s a very thin userspace layer above low-latency networks. 
arXiv

What’s relevant for you:

They all solve the same infra problem you’re eyeing: “I have basins of memory, I want efficient one-sided access and tiny messages without syscalls all the time”.

They already embody decisions about:

connection management,

completion notification,

progress (who actually drives the NIC when no one is doing syscalls).

You can either:

use one as your “wire library” and stay boutique above it, or

read their designs and reimplement a very minimal subset (e.g. just “put/get + tiny active messages” over Soft-RoCE).

Even if you don’t import them, they’re good prior art for how to structure your own user-space fabric.

2.2. Classic Software DSM: Ivy, Munin, TreadMarks
Pre-cluster-cloud, there was an entire wave of Software Distributed Shared Memory (DSM) systems that tried to prove “we can give you /dev/big-flat-RAM over Ethernet.”

Ivy was one of the early page-based DSM systems: page granularity, sequential consistency, replicated across a cluster. 
infoscience.epfl.ch
+1

Munin and TreadMarks then introduced release consistency and ultimately lazy release consistency:

only ship diffs when locks/release points occur,

try to avoid ping-ponging and excessive invalidations. 
hjemmesider.diku.dk
+1

These systems are pretty much abandoned now, but the ideas are exactly on your axis:

Represent “shared memory” as pages or segments.

Track who has read/write access lazily.

Ship diffs or whole pages only at synchronization boundaries.

You don’t have to like their exact consistency model, but the mechanisms (page tracking, diffing, lazy propagation) are still useful for:

remote backing of a “Spirit heap”,

cross-node caches,

remote swapping to “far memory”.

They are also a cautionary tale: they show where the complexity explodes if you try to emulate SMP too faithfully over a LAN.

2.3. Single System Image OSes: Kerrighed, OpenSSI, OpenMosix
Sitting a bit above DSM but still very infra-y, there were Linux-based “single system image” clusters:

OpenSSI, Kerrighed, OpenMosix. 
Wikipedia
+2
ResearchGate
+2

They did things like:

make /proc and process IDs cluster-wide (one process space),

implement cluster-wide shared memory and IPC on top of a token-based coherence scheme,

provide single-root filesystem and transparent process migration.

For your purposes:

Their process and shared-memory handling is basically “production DSM plus process migration and cluster FS”.

They show how to bolt extra semantics into Linux without rewriting the world:

VPROC abstraction in OpenSSI to make “process” a cluster-wide object,

CFS token system for shared-memory and file access.

Again, defunct now, but conceptually they’re “what if your whole Chorus LAN looked like one SMP box from userland”. You can:

steal mechanical tricks (token-based cache, cluster-wide /proc-like views),

remember that going full-SSI is expensive and often not worth the compatibility headache.

2.4. Disaggregated / remote memory: Infiniswap, FastSwap, DiLOS
Newer work in memory disaggregation is basically: “use RDMA to make remote DRAM behave like extra swap / RAM”.

Infiniswap: maps RDMA-backed remote memory as a swap backend; on page fault, swap in/out over RDMA to “far memory servers”. 
arXiv
+2
ResearchGate
+2

FastSwap: similar idea in virtualized environments; redirect swap traffic to host-guest compressed shared memory via RDMA for better performance. 
arXiv

DiLOS: an OS for disaggregated memory with “guided paging” – lets applications influence which pages go remote, to reduce bandwidth waste. 
yoon.ws

At infra level, these show:

How to use RDMA as a backend for the kernel’s paging system.

How to deal with 4KB pages being too small (IO amplification) and when it’s okay anyway.

How to build remote memory pools with some notion of fault tolerance.

You don’t have to adopt remote swap directly, but you can:

copy the shape: “remote RAM” as an L2/L3 tier,

decide whether you want to implement anything page-based yourself (userfaultfd + RDMA is the Linux-era DIY version).

2.5. NVMe-over-Fabrics and storage-as-remote-heap
Closer to disk than RAM, but still very relevant to “basins that talk”:

NVMe over Fabrics (NVMe-oF) exposes remote NVMe devices over RDMA/TCP with very low overhead. It’s widely used in datacenter storage; new RDMA NICs even support crypto offload for this kind of traffic. 
ACM Digital Library

For you, it’s another concrete lever:

Instead of hand-rolling block transport, you can stand on NVMe-oF to turn one node’s NVMe into another node’s “fast basin”.

Combined with RAM caching, that gives you a clean infra layer:

“everything above this line sees remote NVMe as just another block device; we keep our weird semantic stuff up-stack.”

You still have to handle consistency and layout, but you don’t burn time on packet-format design for storage.

2.6. Active Messages: code riding on the wire
One more primitive that lives “just above” your RoCE plumbing is Active Messages:

Classic idea from von Eicken/Culler (~1992): a message is not just data; it carries a handler to execute upon arrival. 
cs.cmu.edu
+2
cs.cmu.edu
+2

Mechanically:

The NIC delivers the payload into user-space,

a tiny dispatcher jumps to the registered function with that payload.

This is used heavily in GASNet and similar systems.

Infra-wise, this is interesting because:

It gives you a very low-latency way to “poke” remote state:

e.g., “apply this small patch to your local heap now” without a whole RPC stack.

It’s still small enough to count as “plumbing”: it’s just saying “messages are (handler, args)”.

You can implement a minimalist active-message substrate over Soft-RoCE and that’s it: no jobs, no arcs, just “tiny code capsules riding your fabric”.

3. “Orphan” or sidelined ideas worth stealing from
You explicitly asked for “concepts left on the wayside after a single or few papers”. A few come to mind as good mental compost:

Lazy Release Consistency (LRC) from TreadMarks/Munin etc.:
Don’t keep memory coherent all the time; only propagate modifications when some synchronization event tells you “now others care”. It’s largely gone from mainstream APIs, but the idea – delay syncs until you absolutely need them – is still gold for any remote-memory illusion. 
hjemmesider.diku.dk
+1

Single-System-Image process spaces (VPROC, cluster-wide /proc like in OpenSSI):
No one ships this today, but the notion that “a process is an abstract object that might live anywhere in a cluster, and /proc shows all of them” is exactly the kind of thing you’d like for “bubbles”, “mnematodes”, “Spirit shards”. 
Wikipedia

COMA-like cache behavior (Cache-Only Memory Architectures, not in the search above but same era):
Instead of fixed home nodes for memory lines, lines migrate so the “home” follows use. That’s a hardware idea, but conceptually useful if you ever build “hot data follows the node that touches it the most” into your heap layout.

Tuple-space / Linda (more semantic, but infra-flavored):
Global associative memory (“anyone can write (tagged) tuples, anyone can read/filter”) as an OS primitive. It’s mostly disappeared in favor of explicit queues and RPC, but for a Chorus-style aether, “shared tuple space over RoCE” is not a bad substrate pattern.

Recent disaggregation OS work like DiLOS:
These OSes revisit old DSM/disaggregation ideas with modern NICs and show where the old page-based assumptions break. They’re very focused on bandwidth and page size, but the mechanisms (guided paging, app hints) are generic infra ideas. 
yoon.ws
+1

All of these can be treated as “weird fossils from earlier epochs” that you quarry for mechanisms, not architectures.

If I compress the inventory in one breath and stay obediently infra-only:

Below: Ethernet, maybe Soft-RoCE / future RoCE, mmap/hugepages, tmpfs, NVMe, maybe NVMe-oF.

Middle: user-space fabrics (GASNet/UCX/libfabric/LCI) for one-sided RMA + active messages; DSM-era techniques (page tracking, lazy consistency); remote memory hacks (Infiniswap/FastSwap/DiLOS); SSI tricks (cluster-wide /proc, token-based coherence).

Above: nothing yet. No identities, no knobs, no semantics.

That’s a pretty rich toolbox to start sketching “basins plus plumbing” without freezing anything higher up the stack.

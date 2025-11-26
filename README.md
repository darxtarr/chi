# Chi (χ) - Distributed Spatial Computing Substrate

Apps emit semantic content. The Spirit decides presentation. No pixels, no framebuffers, pure state synchronization over network.

Read chi_docs_development.md for complete architectural documentation.

---

## Status

**Phase:** 1 - Network transparency PoC (complete)
**Next:** Phase 2 - CRDTs + wgpu rendering

---

## Architecture

```
App (hello-bubble)               Spirit (compositor)
      |                                 |
      | Semantic content (text, images) |
      |  -> Binary TLV protocol         |
      |  -> TCP network                 |
      |                                 |
      +-------------------------------->|
                                        |
                                 [Receives bubbles]
                                 [Future: renders to wgpu]
```

**Core principle:** Apps don't know about screens, pixels, or windows.
- App sends: "Display this text in green"
- Spirit decides: Window? VR? Terminal? Speech?

---

## Quick Start

Build and run:
```bash
cargo build --release

# Terminal 1: Start the Spirit (listens on TCP:7777)
./target/release/spirit

# Terminal 2: Send bubbles from an app
./target/release/hello-bubble

# Inspect protocol (optional)
./target/release/hello-bubble | ./target/release/bubble-dump
```

---

## File Structure

**Flat, LLM-optimized naming:**

```
chi_bin_spirit.rs          - The Spirit compositor (TCP listener)
chi_bin_hello_bubble.rs    - Example app (sends text bubbles)
chi_bin_bubbledump.rs      - Protocol inspector tool

chi_lib_tlv_framing.rs     - TLV (Type-Length-Value) codec
chi_lib_message_types.rs   - Message definitions (Text, Image)

lib.rs                     - Module manifest
Cargo.toml                 - Build config (3 binaries, 1 lib)
```

**Naming convention:**
- `chi_bin_*.rs` = Binary entry point
- `chi_lib_*.rs` = Shared library code
- `chi_<binary>_*.rs` = Binary-specific code (future)

One `ls -al` shows everything. No nested folders.

---

## Protocol

**Wire format:** Binary TLV (Type-Length-Value)

```
[type: u8][length: u32][payload: bytes]
```

**Message types:**
1. **Text** - String + RGBA color (semantic, not rendered by app)
2. **Image** - Width, height, RGBA pixel data
3. Future: Geometry, notifications, input requests

**Transport:** Currently TCP (standard networking). Future: RDMA/RoCE targeting <10µs latency (based on ConnectX-4 specs, not yet tested).

---

## Building Blocks (Current)

**Phase 1 deliverables:**
- ✅ Binary protocol (TLV framing)
- ✅ Text + Image message types
- ✅ Network transparency (app ↔ compositor over TCP)
- ✅ Inspector tooling (bubble-dump)
- ✅ Example app (hello-bubble)

**Phase 2 (Next):**
- wgpu rendering (Spirit actually displays bubbles)
- Initial testing on jetsone rendering node

**Future Phases:**
- CRDTs for distributed state sync (multiple apps, convergence)
- RDMA transport upgrade (zero-copy networking)
- VR/AR input integration (Quest 3)
- Local AI orchestration (Gemma on think/jetsone)

---

## Philosophy

**Content/Presentation Separation (OS-level)**

Traditional:
- App renders to framebuffer → compositor blits pixels

Chi:
- App emits semantic content → Spirit composes reality based on viewport

**Why this matters:**
- Network transparency (send state, not pixels)
- Device flexibility (2D monitor, VR headset, terminal - same app)
- AI orchestration (Spirit can intelligently place/route content)

---

## Notes for Future Sonnies

**Context efficiency:** This repo uses flat structure to minimize LLM token usage.
- One `ls -al` shows all source files
- Filenames encode hierarchy (no grep dance)
- History in git commits, not markdown files

**Tests:** Inline `#[cfg(test)]` only. Run with `cargo test --lib`.

**Commit style:** Decisions go in commit messages, not separate docs.

---

## Project Status

Chi is a standalone distributed spatial computing substrate. Future integration with other distributed systems projects is possible but not currently scoped.

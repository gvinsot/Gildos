# GILDOS — An AI-Native Operating System

**Engineering Specification, Revision 0.2**
**Classification:** Internal / Exploratory Research
**Date:** 2026-05-27
**Editor:** Principal Systems Architecture Group
**Status:** Draft for technical review. Not a product roadmap.

**Revision 0.2 adds:**
- §15 Target Hardware Profile (DGX Spark as Phase-1 reference platform)
- §16 Testing and Development Strategy (VM/container ladder, external-model substitution, CI matrix)
- §17 Prerequisites (hardware, software, operational, organizational)
- Renumbered Open Research Questions to §18.

---

## Document Conventions

- **MUST / SHOULD / MAY** follow RFC 2119 semantics.
- **[O]** marks open research questions.
- **[A/B]** marks unresolved architectural alternatives.
- **Engineering rigor over enthusiasm:** any claim presented without a concrete
  mechanism is to be treated as speculation and flagged accordingly in §15.
- "The AI" or "the cognitive core" refers to the resident inference engine
  plus its supervising runtime, **not** to anthropomorphic agency.

---

## Table of Contents

1.  Executive Summary
2.  System Principles
3.  Architecture Overview
4.  Boot and Lifecycle
5.  Cognitive Runtime
6.  Event System
7.  Capability System
8.  Generated Drivers
9.  Self-Improvement Loop
10. Resource Governance
11. Security Model
12. Storage and Persistence
13. Networking
14. Development Roadmap
15. Target Hardware Profile
16. Testing and Development Strategy
17. Prerequisites
18. Open Research Questions

---

## 1. Executive Summary

### 1.1 Description

Gildos is an experimental operating system in which a large language model
(LLM) — or, more generally, a continuously-resident inference engine — is the
**primary supervisory layer** of the system. Conventional userspace,
desktop environments, and shell-based interaction are demoted to optional
back-ends. Hardware is exposed through a small set of kernel primitives;
everything above the kernel is expressed as **capabilities** that the
cognitive core can load, replace, generate, evaluate, and retire.

Gildos is not a chatbot bolted onto Linux. It is a deliberate inversion of
the usual stack:

    Traditional:    Hardware → Kernel → Userspace → Applications → AI feature
    Gildos:         Hardware → Kernel → Cognitive Runtime → Capabilities → User

### 1.2 Vision

The thesis under test is:

> *If inference is cheap, continuous, and structurally trusted, then the
> operating system's job changes from "running applications" to "running
> cognition." Programs become hypotheses; the OS evaluates them.*

If the thesis holds, the artifact looks less like Windows or Android and
more like a **cognitive appliance**: a machine that boots into an
understanding of itself and its user, exposes its hardware as observable
state, and modifies its own behavior in bounded, auditable ways.

### 1.3 Philosophy

- **Cognition first.** The scheduler, memory hierarchy, and event system are
  shaped by the needs of an inference loop, not those of a 1990s windowing
  system.
- **Capabilities, not applications.** The unit of installable behavior is a
  versioned, signed, sandboxed *capability* with a declared schema, not a
  monolithic binary.
- **Semantic over syntactic.** Logs, syscalls, and metrics are converted at
  the boundary into summarized, schema-bound *events* the AI can ingest
  without burning context on noise.
- **Self-modification under contract.** The system may rewrite parts of
  itself, but only via a closed loop of *propose → compile → sandbox →
  benchmark → canary → adopt → rollback*. Nothing magical.
- **Minimum trusted surface.** The kernel and the supervisor are small,
  auditable, and signed. AI output is **never** trusted by default; it is
  validated by mechanisms outside the model.

### 1.4 Novelty

What is genuinely new (vs. existing systems such as Android, ChromeOS, or
"AI desktops" with an LLM sidebar):

1.  The inference engine is a **first-class resident subsystem**, scheduled
    against the kernel — not an application competing with browsers.
2.  The user-visible interface contract is **the model's I/O**, not a
    window server. CLIs/GUIs exist only as optional capabilities.
3.  Drivers and userland modules can be **synthesized, validated, and
    rotated** by the system under engineering constraints (compile, test,
    canary).
4.  The **event bus is semantic**: events carry meaning, schema, and
    relevance scores, not just timestamps and PIDs.
5.  Persistence is split between a classical filesystem (for blobs and
    binaries) and a **semantic memory store** (for facts, episodes, and
    capability metadata).

What is *not* new and must not be oversold:

- LLM-driven shells, "agentic" wrappers, and AI orchestrators already
  exist. Gildos differs in *position in the stack*, not in the existence
  of inference itself.
- Self-modifying code, hot-patching, and dynamic linking are decades old.
  Gildos contributes the **closed loop and provenance model**, not the
  primitive.

---

## 2. System Principles

### 2.1 Cognition-First Computing

**Definition.** The system's scheduler, memory budgets, and IO priorities
treat one or more inference processes as **continuously resident,
non-preemptible-by-default** workloads. Other workloads are scheduled
around their VRAM residency and KV-cache state, not the other way around.

**Implications.**

- A swap-out of the resident model is a *system event*, not a routine
  scheduler decision. It MUST be logged, justified, and reversible.
- Latency budgets for user-visible interaction are dominated by token
  generation time, not by syscall round-trips. The OS optimizes for
  prompt-to-first-token and tokens/sec, not for context-switch counts.
- "Idle" no longer means "do nothing." Idle CPU/GPU cycles SHOULD be
  consumed by background cognition: summarization, memory consolidation,
  capability benchmarking. See §9.

**Tradeoff.** Pinning a multi-GB model in VRAM is *expensive* on devices
with shared memory (integrated GPUs, Apple-Silicon-class APUs) or limited
discrete VRAM. On such devices, Gildos MUST support a **tiered residency**
model (§5.4): hot weights pinned, cold weights paged, with hard guarantees
on token-latency SLOs.

### 2.2 AI-Native Interaction

The user interacts with **the system itself**, mediated by the cognitive
core. Voice, text, structured input, and sensor events are all funneled
into the same reasoning loop. There is no privileged "shell".

A traditional terminal MAY exist as a debug capability, but it is not the
contract. The user contract is:

    User intent → AI interpretation → capability invocation → result → AI
    summary → User

This is a **strong claim** and a known risk (see §11 and §15). It is
acceptable only if the AI is *predictable enough* that "ask the system to
do X" produces auditable, reversible behavior with bounded variance.

### 2.3 Capability-Centric Design

A **capability** is the smallest installable unit of behavior. It is:

- Declared by a manifest (name, version, semver, schema, permissions,
  resource budget, signature, provenance).
- Implemented by one or more artifacts (native binary, WASM module, Python
  fragment, model adapter, or generated code blob).
- Sandboxed by default; permissions are explicit and revocable.
- Independently swappable; the system can run multiple versions and route
  traffic between them.

Capabilities replace the role of "applications," "drivers," "system
services," and "shell utilities." This is uniform on purpose: it lets one
mechanism (capability lifecycle) handle all extension surfaces.

### 2.4 Self-Improving Runtime

The system MAY:

- Rewrite slow capabilities.
- Generate new capabilities to satisfy expressed user intents.
- Replace drivers with more efficient variants.
- Tune scheduling, caching, or memory policies based on observed load.

The system MUST NOT:

- Modify itself outside the **closed loop** of §9.
- Bypass the signing, sandboxing, or rollback machinery.
- Persist a modification that has not been canaried.
- Touch the trusted computing base (TCB; see §3.4) at runtime.

### 2.5 Semantic Operating System

Internal state is exposed to the cognitive core as **typed semantic
events** with relevance scores and provenance. Raw byte streams (dmesg,
journalctl, syscall traces) are summarized at the boundary by deterministic
extractors and small specialized models (§6). The AI ingests *meaning*,
not lines of text.

This is the difference between "the AI watches `/var/log`" (terrible) and
"the AI subscribes to a typed event stream where `disk.health.degraded`
arrives once with a structured payload" (tractable).

---

## 3. Architecture Overview

### 3.1 Layered View

```
+---------------------------------------------------------------+
|  L7  User / External Clients (voice, text, network, sensors)  |
+---------------------------------------------------------------+
|  L6  Interaction Surfaces (TTS/STT, GUI, CLI, API gateway)    |  optional
+---------------------------------------------------------------+
|  L5  Cognitive Core                                           |
|        - Inference Engine (llama.cpp-class / vLLM-class)      |
|        - Reasoning Loop + Tool Use Dispatcher                 |
|        - Context & Memory Manager                             |
+---------------------------------------------------------------+
|  L4  Capability Plane                                         |
|        - Capability Manager (load/unload/version/migrate)     |
|        - Sandbox Runtimes (WASM, OCI, seccomp-bpf)            |
|        - Driver Synthesis Pipeline                            |
+---------------------------------------------------------------+
|  L3  Semantic Event Bus + Semantic Memory Store               |
+---------------------------------------------------------------+
|  L2  System Supervisor (trusted)                              |
|        - Policy engine, signer, attestation, rollback         |
+---------------------------------------------------------------+
|  L1  Host Kernel (Linux LTS in Phase 1; custom in Phase 4)    |
|        - Drivers, schedulers, FS, network                     |
+---------------------------------------------------------------+
|  L0  Hardware (CPU, GPU/NPU, RAM, VRAM, NVMe, NIC, sensors)   |
+---------------------------------------------------------------+
```

### 3.2 Subsystem Map

```
                              +-------------------+
                              |   User / Network  |
                              +---------+---------+
                                        |
                                  intent / events
                                        |
+-------------------+          +--------v---------+        +------------------+
| Interaction Caps  |<-------->|  Cognitive Core  |<------>| Semantic Memory  |
| (voice, text, UI) |          |  (Inference +    |        | (KV + vector +   |
+-------------------+          |   Reason Loop)   |        |  episodic log)   |
                               +---+----------+---+        +------------------+
                                   |          |
                            tool / capability calls
                                   |          |
                          +--------v--+    +--v--------+
                          | Capability|    |  Event    |
                          |  Manager  |<-->|   Bus     |
                          +-----+-----+    +-----+-----+
                                |                |
                          +-----v----------------v-----+
                          |     System Supervisor      |
                          |  (policy, signer, attest)  |
                          +-----+----------------+-----+
                                |                |
                          +-----v-----+    +-----v-----+
                          |  Sandbox  |    |  Driver   |
                          |  Runtimes |    |  Synth    |
                          +-----+-----+    +-----+-----+
                                |                |
                          +-----v----------------v-----+
                          |        Host Kernel         |
                          +-------------+--------------+
                                        |
                                  +-----v-----+
                                  | Hardware  |
                                  +-----------+
```

### 3.3 Runtime Model

Gildos runs a fixed set of long-lived processes plus an open set of
capability instances.

| Process              | Privilege | Persistence | Notes                       |
|----------------------|-----------|-------------|-----------------------------|
| `supervisord`        | TCB       | Always-on   | PID 1 successor; signed     |
| `cogd` (cog daemon)  | TCB-adj.  | Always-on   | Owns inference engine state |
| `eventd`             | TCB-adj.  | Always-on   | Semantic bus broker         |
| `capd`               | TCB-adj.  | Always-on   | Capability lifecycle        |
| `memd`               | TCB-adj.  | Always-on   | Semantic memory store       |
| Capability instances | Sandbox   | On-demand   | One process per instance    |

"TCB-adj." (adjacent) means signed, audited, but not in the minimum boot
chain. A compromise of `cogd` is recoverable; a compromise of
`supervisord` is not (see §11.6).

### 3.4 Trusted Computing Base

The TCB is the smallest set of components whose compromise compromises the
whole system. In Gildos:

- Bootloader, kernel, supervisor binary, signing keys.
- The capability verifier (signature check + manifest validator).
- The sandbox enforcer (kernel seccomp/landlock policies).

The TCB explicitly **excludes**:

- The model weights.
- The inference engine (`cogd`).
- Any generated or self-improved code.

This is non-negotiable. AI output is *content*, not *code that runs inside
the TCB*. See §11.

---

## 4. Boot and Lifecycle

### 4.1 Boot Chain (Cold Boot)

```
[1] Firmware (UEFI + Secure Boot, measured into TPM PCRs)
        |
        v
[2] Bootloader (signed, e.g. systemd-boot or u-boot variant)
        |
        v
[3] Host Kernel (signed Linux LTS in Phase 1)
        |
        v
[4] initramfs minimal: mount RO root, attest TCB hashes
        |
        v
[5] supervisord (PID 1) — verifies signatures, opens TPM, loads policy
        |
        v
[6] eventd, memd, capd start in parallel; bus comes up
        |
        v
[7] cogd starts:
       - allocate VRAM arena
       - mmap model weights from signed read-only volume
       - warm KV cache (optional: replay distilled session)
       - register as a tool-use host on the bus
        |
        v
[8] Boot-time capabilities load (network, audio, sensors)
        |
        v
[9] System publishes `system.ready` event with attestation report
        |
        v
[10] First user interaction (voice/text/API)
```

Target cold-boot SLO (Phase 1, consumer hardware, 7B-class GGUF model):

- Step 1–6: < 5 s.
- Step 7 (mmap + first token ready): < 15 s for Q4 7B; up to 90 s for
  larger models on slow NVMe. Mitigation: pre-loaded KV snapshot
  (§5.3.3).
- Step 10 (interactive): < 20 s end-to-end on a target reference machine
  (≥ 24 GB VRAM, NVMe Gen4).

### 4.2 Warm Boot and Suspend/Resume

Suspend-to-RAM:
- VRAM is preserved on devices that support it (most discrete GPUs lose
  VRAM on S3; we MUST detect and re-warm).
- KV cache is checkpointed to NVMe (§5.3.3) on suspend.

Suspend-to-disk:
- Treated as cold boot + KV restore.

### 4.3 Recovery Boot

If a capability or self-modification breaks the system, Gildos MUST be
recoverable without external tools.

Recovery modes (selectable from bootloader, or auto-triggered by N failed
boots tracked in NVRAM):

1.  **Safe Cognition Mode.** Boot with only signed capabilities from the
    *golden set* (a frozen baseline shipped with the system image). All
    self-improved capabilities are masked. The AI is informed via an
    event that it is in recovery.
2.  **Minimum-Trust Mode.** No cognitive core at all. `supervisord`
    exposes a tiny HTTP/UART/serial admin interface for forensic and
    rollback operations. This is the equivalent of single-user mode.
3.  **Attestation-Failed Mode.** TCB hashes do not match expected values.
    System refuses to start `cogd`; user is notified via the safest
    available channel (LED, speaker beep pattern, network beacon).

### 4.4 Persistence

| Object                | Volatility   | Storage                      |
|-----------------------|--------------|------------------------------|
| Model weights         | Immutable    | Signed RO partition          |
| KV cache snapshot     | Volatile+ckpt| NVMe; cleared on TCB mismatch|
| Semantic memory       | Persistent   | Append-only log + index      |
| Capability archive    | Persistent   | Content-addressed store      |
| User data             | Persistent   | Encrypted volume             |
| Logs (low-level)      | Ring buffer  | tmpfs unless escalated       |

### 4.5 Shutdown

Shutdown is a *negotiated* operation. `cogd` is asked to checkpoint
context, `memd` flushes the WAL, `capd` quiesces capabilities, and
finally `supervisord` halts the kernel. A hard power loss is treated as
an unclean shutdown; recovery uses the WAL (§12).

---

## 5. Cognitive Runtime

### 5.1 Inference Engine

**Default choice (Phase 1):** llama.cpp-class engine for GGUF models, with
CUDA/Metal/ROCm/Vulkan back-ends and a stable C ABI. Rationale: small,
auditable, easy to embed, supports quantized models on commodity hardware,
already battle-tested.

**Alternatives considered:**

- **vLLM / SGLang.** Higher throughput via paged attention, but Python
  surface and heavier dependencies. Suitable for server-class Gildos
  deployments. **[A]** Phase 2+.
- **TensorRT-LLM / ExecuTorch.** Vendor-locked, excellent on specific
  hardware. Used opportunistically per device.
- **Custom kernel-level inference.** Long-term research direction.
  Premature in Phase 1.

**Engine requirements (MUST):**

- Streaming token output.
- KV cache export/import.
- Multiple concurrent contexts (or, failing that, fast context save/load).
- Constrained decoding (grammar / JSON schema) — *essential* for tool use.
- Speculative decoding hook (optional but valuable).

### 5.2 Reasoning Loop

```
              +--------------------+
              | Input Event        |
              | (user / system)    |
              +---------+----------+
                        |
                  routed by relevance
                        |
                        v
              +--------------------+
              | Context Assembly   |
              |  - system prompt   |
              |  - persistent mem  |
              |  - retrieved facts |
              |  - recent events   |
              +---------+----------+
                        |
                        v
              +--------------------+
              | Inference          |
              |  (constrained)     |
              +----+----------+----+
                   |          |
              text |          | structured tool call
                   |          |
                   v          v
            +------+--+   +---+--------+
            | Speak / |   | Capability |
            |  reply  |   |  dispatch  |
            +---------+   +---+--------+
                              |
                              v
                         +----+----+
                         | Result  |---> back into Context as observation
                         +---------+
```

Key properties:

- **Constrained decoding is mandatory** for any output that hits the
  capability dispatcher. JSON-Schema-grammar enforcement at the decoder
  prevents malformed tool calls from ever existing.
- The loop is **interruptible**. A high-priority event (e.g., thermal
  emergency) MUST be able to interrupt generation cleanly.
- The loop has a **budget**: max tokens per turn, max tool calls per
  turn, max wall time. Exceeding budget yields a graceful timeout, not a
  hang.

### 5.3 Context and Memory Hierarchy

```
+--------------------------------------------------------------+
|  L0  Working set (current prompt + recent turns)             |  ~ ctx window
|       Lives in: model KV cache (VRAM)                        |
+--------------------------------------------------------------+
|  L1  Session memory (this boot, possibly many hours)         |  ~ MB
|       Lives in: ring buffer in RAM, distilled into summaries |
+--------------------------------------------------------------+
|  L2  Episodic memory (events with timestamps)                |  ~ GB
|       Lives in: append-only log on NVMe                      |
+--------------------------------------------------------------+
|  L3  Semantic memory (facts, embeddings, capability docs)    |  ~ GB
|       Lives in: vector index + KV store                      |
+--------------------------------------------------------------+
|  L4  Cold archive (raw transcripts, blobs)                   |  unbounded
|       Lives in: object store; opt-in, encrypted              |
+--------------------------------------------------------------+
```

#### 5.3.1 Working set (L0)

The KV cache is the most expensive resource in the system. Policies:

- **Pinned segments** (system prompt, persona, security policy) are at
  the beginning of the context and never evicted.
- **Sliding segments** (recent dialogue) use standard window
  management.
- **Retrieved segments** (RAG hits from L2/L3) are inserted as ephemeral
  blocks with explicit provenance, so the model can cite or distrust
  them.

#### 5.3.2 Session memory (L1)

A ring buffer of recent turns, with a *summarizer capability* running
opportunistically that distills the buffer into compact notes. The notes
are promoted to L2 on session boundaries.

#### 5.3.3 KV snapshots

Boot latency is dominated by re-warming the KV cache. Gildos snapshots a
"baseline KV" containing the system prompt and pinned persona at install
time. Cold boot restores this snapshot to VRAM in O(model_size / NVMe
bandwidth), saving the prefill compute.

**Caveat [O]:** KV snapshot validity depends on exact model build, quant,
and engine version. A version drift MUST invalidate the snapshot and
trigger re-prefill.

#### 5.3.4 Semantic memory (L3)

- Vector index over fact embeddings.
- KV store for structured facts (`{subject, predicate, object,
  provenance, confidence, ts}`).
- Capability documentation and usage statistics live here, so the AI can
  retrieve a capability's interface before calling it.
- **Forgetting policy:** facts decay if unreinforced unless marked
  pinned. This is necessary for both privacy and storage bounds.

### 5.4 Tiered VRAM Residency

Three tiers, selected per device:

- **Tier A — Full residency.** Entire model in VRAM. Target: discrete
  GPUs with ≥ model size + 2× headroom.
- **Tier B — Mixed residency.** Hot layers in VRAM, cold layers in
  system RAM, paged on demand. Token latency degrades gracefully; the
  engine MUST report effective tokens/sec to the scheduler.
- **Tier C — CPU residency with GPU acceleration of attention.** Last
  resort. Phase 1 supports it for development on low-end hardware; not a
  target for production.

The scheduler (§10) MUST not move a model across tiers without a logged
event and a user-visible notice on consumer devices.

### 5.5 Scheduling of Cognitive Tasks

Cognitive work is classified:

| Class | Latency target | Preemptible | Example                          |
|-------|----------------|-------------|----------------------------------|
| RT    | < 200 ms       | No          | "What time is it?", critical evt |
| INT   | < 2 s          | Yes (soft)  | Interactive dialogue             |
| BG    | seconds–min    | Yes         | Memory consolidation, summary    |
| OPT   | unbounded      | Yes         | Self-improvement experiments     |

A priority queue feeds `cogd`. RT and INT preempt BG/OPT. Two policies
are considered:

**[A] Single-context preemption.** One model instance, fast context
save/restore. Simpler. Latency cost on switch.

**[B] Parallel contexts (paged attention).** Multiple in-flight contexts
served by one engine. Higher throughput, harder isolation, requires
vLLM-class engine. Recommended for Phase 2+.

---

## 6. Event System

### 6.1 Why Raw Logs Are Insufficient

Three orthogonal reasons:

1.  **Context economics.** A 7B model's context window is precious.
    Burning 4 kB on `kern.log` lines to learn "the wifi reconnected" is
    waste. Raw logs are O(MB/hour); useful events are O(KB/hour).
2.  **Semantic loss.** "Link down" and "link up" 80 ms apart is a
    *transient*, not two independent events. The bus collapses them
    upstream.
3.  **Adversarial surface.** Log lines are unstructured strings written
    by arbitrary code. Feeding them straight into a model is a prompt
    injection waiting to happen (§11.2).

### 6.2 Architecture

```
   Raw sources                Extractors           Semantic Bus       Consumers
+----------------+        +------------------+   +-------------+   +-----------+
| kernel ringbuf |------->| dmesg extractor  |-->|             |-->| cogd      |
| journald       |------->| journal extractor|-->|   eventd    |-->| capd      |
| /proc, /sys    |------->| metric sampler   |-->|  (typed,    |   | memd      |
| capability evt |------->| direct emitter   |-->|   scored,   |   | logger    |
| network        |------->| netfilter tap    |-->|   schema-   |   | external  |
| sensors        |------->| sensor adapter   |-->|   validated)|   |  sinks    |
+----------------+        +------------------+   +-------------+   +-----------+
```

- **Extractors** are small, signed, deterministic programs (or distilled
  small models) that convert raw streams into typed events. They are
  trusted code; they live in the capability plane but are marked
  `extractor` and reviewed.
- **eventd** validates schema, deduplicates, scores relevance, and fans
  out to subscribers.
- Consumers subscribe by topic and relevance threshold.

### 6.3 Event Schema

All events share a common envelope. Topic-specific payloads are typed.

```json
{
  "id": "01HZJ8E9C6P7V8...",
  "ts": "2026-05-26T12:34:56.789Z",
  "topic": "device.disk.health",
  "schema": "gildos/device.disk.health@1",
  "source": {
    "component": "extractor.smartctl",
    "version": "0.3.1",
    "signed_by": "key:gildos-core"
  },
  "severity": "warning",
  "relevance": 0.78,
  "ttl_s": 86400,
  "dedup_key": "disk:nvme0n1:reallocated",
  "payload": {
    "device": "/dev/nvme0n1",
    "metric": "reallocated_sectors",
    "value": 14,
    "delta_24h": 3,
    "threshold": 10
  },
  "suggested_actions": [
    {"capability": "diagnostics.disk", "args": {"device": "/dev/nvme0n1"}},
    {"capability": "user.notify",      "args": {"text": "SSD health is degrading."}}
  ]
}
```

### 6.4 Relevance Scoring

Each event carries a `relevance` ∈ [0,1]. It is computed by the extractor
using:

- Severity baseline.
- Novelty (is this `dedup_key` new in the last *N*?).
- Subscriber interest (subscriptions feed back to extractors).
- Recent user context (if the user is debugging audio, audio events get
  boosted; this is a *small* multiplier, not a free hand).

Consumers set thresholds. `cogd` typically subscribes at `>= 0.5` for
interactive routing and `>= 0.9` for interruption-class events.

### 6.5 Information Flow OS → AI

Three integration patterns:

1.  **Subscription.** `cogd` holds a subscription; high-relevance events
    are inserted into the next reasoning loop as an observation block.
2.  **Interrupt.** Severity `critical` events preempt the current
    generation, with a structured "system interrupt" block.
3.  **Retrieval.** During reasoning, the model can call a `events.query`
    capability to fetch past events by topic/time/relevance. This avoids
    bloating context with unrequested noise.

### 6.6 Sample Events

```json
{
  "topic": "capability.lifecycle",
  "schema": "gildos/capability.lifecycle@1",
  "severity": "info",
  "relevance": 0.35,
  "payload": {
    "capability": "drivers.usb.uvc",
    "from_version": "0.7.2",
    "to_version": "0.7.3-gen.42",
    "transition": "canary_promoted",
    "trigger": "self_improvement_loop"
  }
}
```

```json
{
  "topic": "resource.vram.pressure",
  "schema": "gildos/resource.vram.pressure@1",
  "severity": "warning",
  "relevance": 0.82,
  "payload": {
    "free_mb": 412,
    "model_resident_mb": 7800,
    "candidate_evictions": ["bg.summarizer", "bg.embedder"]
  }
}
```

```json
{
  "topic": "security.policy.violation",
  "schema": "gildos/security.policy.violation@1",
  "severity": "critical",
  "relevance": 1.0,
  "payload": {
    "capability": "experimental.web.scrape",
    "rule": "egress.deny_unsigned_destination",
    "destination": "203.0.113.7:443"
  },
  "suggested_actions": [
    {"capability": "system.quarantine", "args": {"capability": "experimental.web.scrape"}}
  ]
}
```

---

## 7. Capability System

### 7.1 Definition

A **capability** is a named, versioned, signed bundle providing a typed
interface. It has:

- A **manifest** (`gildos.capability.json`).
- One or more **artifacts** (binary, WASM, script, model adapter).
- A **schema** for inputs/outputs.
- A **permission set** (filesystem, network, hardware, other
  capabilities).
- A **resource budget** (CPU, RAM, VRAM, time).
- A **provenance record** (who built it, from what source, with what
  toolchain hash).

### 7.2 Manifest Example

```json
{
  "name": "media.audio.player",
  "version": "1.4.2",
  "schema": "gildos/capability.manifest@1",
  "interface": {
    "play": {
      "input": {"type": "object", "properties": {
        "uri": {"type": "string"},
        "volume": {"type": "number", "minimum": 0, "maximum": 1}
      }, "required": ["uri"]},
      "output": {"type": "object", "properties": {
        "session_id": {"type": "string"}
      }}
    },
    "stop": {"input": {"type": "object", "properties": {"session_id": {"type": "string"}}}}
  },
  "permissions": {
    "audio.output": "exclusive",
    "fs.read": ["~/Music", "/var/lib/gildos/media"],
    "net.egress": "deny"
  },
  "resources": {"ram_mb": 64, "cpu_pct": 10, "vram_mb": 0},
  "runtime": "wasm",
  "artifact": "sha256:8d3f...c2",
  "provenance": {
    "source": "git+https://example.com/audio.player@v1.4.2",
    "toolchain": "wasi-sdk-22+rustc-1.79",
    "builder_signature": "key:gildos-builder",
    "reproducible": true
  },
  "signature": "ed25519:..."
}
```

### 7.3 Lifecycle

```
declared --> fetched --> verified --> compiled? --> sandbox-tested
   |            |            |            |               |
   |            |            |            |               v
   |            |            |            |          benchmarked
   |            |            |            |               |
   |            |            |            |               v
   |            |            |            |           canaried
   |            |            |            |               |
   |            |            |            |               v
   |            |            |            |           promoted (active)
   |                                                       |
   |<------------- rollback -------------------------------+
   |                                                       |
   v                                                       v
quarantined                                            retired
```

**Loading** requires: signature verification → manifest validation →
permission check against current policy → sandbox creation → handle
issuance. **Unloading** requires: drain in-flight calls (with timeout) →
checkpoint state if persistent → free resources.

### 7.4 Versioning and Migration

- Versions follow semver. Breaking changes MUST bump major.
- Multiple versions MAY be resident simultaneously; calls are routed by
  the *capability resolver* based on the caller's declared compatibility
  range.
- **Migration** is the act of moving in-flight callers from version *n*
  to *n+1*. Strategies:
  - **Drain.** Stop new traffic to *n*, finish in-flight, switch. Default.
  - **Bridge.** A small generated adapter translates *n*'s schema to
    *n+1*'s. Used when downtime is unacceptable.
- A capability MAY declare itself **stateful**. Stateful migration
  requires a state-export/import pair. The system refuses to migrate
  stateful capabilities without these endpoints.

### 7.5 Dependency Model

Capabilities declare dependencies as `(name, version-range,
required|optional)`. The resolver MUST:

- Refuse loads that would introduce diamond conflicts unless versions
  are isolatable (e.g., WASM module-per-instance).
- Forbid dependency cycles at load time.
- Recompute the dependency graph on each capability change and publish
  it as a `capability.graph.updated` event.

### 7.6 How This Differs From Traditional Software

| Aspect              | Traditional App        | Gildos Capability             |
|---------------------|------------------------|-------------------------------|
| Discovery           | App store search       | Semantic query by intent      |
| Installation        | User-initiated         | Often AI-initiated, signed    |
| Identity            | Bundle ID / package    | Name + version + provenance   |
| Update              | User or vendor push    | Resolver-driven, canaried     |
| Sandbox             | Optional, OS-specific  | Mandatory, uniform            |
| Interface           | UI + ad-hoc IPC        | Typed JSON-schema RPC         |
| Replacement         | Reinstall              | Hot swap with rollback        |
| Generation          | Humans only            | Humans OR system (§8, §9)     |

---

## 8. Generated Drivers

This is the most ambitious section. It is also the one most likely to
fail; we treat it with explicit skepticism.

### 8.1 Feasibility Statement

LLMs can produce plausible-looking driver code today. They cannot, with
acceptable reliability, produce *correct* driver code for arbitrary
unfamiliar hardware. We assume:

- **Feasible now:** generating glue code, configuration, parsers,
  protocol bindings for well-documented devices (USB HID variants,
  vendor-specified I2C sensors, simple GPIO peripherals, network
  protocols with public RFCs).
- **Possible with discipline:** generating drivers for devices that
  resemble known ones, using retrieval over reference driver corpora and
  rigorous sandboxed testing.
- **Not feasible in Phase 1:** generating drivers for novel, complex, or
  undocumented hardware (modern GPUs, NVMe controllers, Wi-Fi chipsets).
  These remain vendor-provided.

A generated driver is a *capability* like any other (§7); the
synthesis pipeline is one more way of producing the artifact.

### 8.2 Pipeline

```
+----------------------+
| Device Discovery     |  PCI/USB/I2C enumeration; identifiers + EDIDs
+----------+-----------+
           |
           v
+----------+-----------+
| Hardware Description |  Retrieve datasheets, RFCs, reference drivers
| Retrieval (RAG)      |  from a curated corpus. Cite sources.
+----------+-----------+
           |
           v
+----------+-----------+
| Skeleton Selection   |  Pick a known template (UVC, HID, NIC, ...)
+----------+-----------+
           |
           v
+----------+-----------+
| Code Generation      |  LLM emits implementation + tests under a
|                      |  grammar-constrained code schema.
+----------+-----------+
           |
           v
+----------+-----------+
| Static Validation    |  Type check, lint, custom analyzers (e.g., no
|                      |  unsafe pointer arithmetic, no syscalls outside
|                      |  permitted set).
+----------+-----------+
           |
           v
+----------+-----------+
| Compilation          |  Sealed builder; reproducible; toolchain hashed.
+----------+-----------+
           |
           v
+----------+-----------+
| Sandbox Test         |  Run in a VM or eBPF-jailed user-mode driver
|                      |  framework against a recorded device trace or
|                      |  a real device with watchdog + I/O quotas.
+----------+-----------+
           |
           v
+----------+-----------+
| Benchmark            |  Throughput / latency / error rate vs. baseline
|                      |  (vendor driver if available, else heuristics).
+----------+-----------+
           |
           v
+----------+-----------+
| Canary               |  Route 1% of traffic; collect events.
+----------+-----------+
           |
           v
+----------+-----------+
| Promote / Rollback   |  On success, promote. On regression, rollback
|                      |  and persist a `driver.gen.failure` event.
+----------------------+
```

### 8.3 Engineering Limits

- **Userspace only in Phase 1.** Generated drivers run in user-mode
  frameworks (FUSE, USB libusb, vfio, network tun/tap, eBPF). No
  generated code enters the kernel. This is non-negotiable.
- **Determinism gap.** Hardware is non-deterministic; a generated driver
  passing sandbox tests can still fail in the field. Mitigation: long
  canary windows, watchdogs, and *behavioral envelopes* (e.g., "this
  driver MUST NOT exceed X interrupts/sec").
- **Datasheet hallucination.** RAG over datasheets is helpful but not
  safe; the model can fabricate register layouts. Mitigation: require
  citations and reject code with un-cited register references.
- **IP and licensing.** Generated code from training data may reproduce
  proprietary code. Mitigation: provenance and similarity checks against
  known corpora; quarantine code with high similarity to non-permissive
  licenses.
- **Compositional fragility.** A driver that works in isolation may
  collide with other devices on shared buses. The pipeline MUST test in
  the actual system topology before promotion.

### 8.4 What Generated Drivers Are *Good* For

- Adding support for a new USB peripheral that follows a known class.
- Producing protocol adapters for IoT devices.
- Writing the boring 80% of a driver while a human reviews the 20%.
- Maintenance: regenerating a driver against an updated kernel ABI.

What they are **not** good for in Phase 1: replacing GPU drivers, NVMe
controllers, modem firmware interfaces, or anything whose failure mode
includes "brick the device."

---

## 9. Self-Improvement Loop

### 9.1 Scope

The loop targets *capabilities*, not the TCB. Anything inside the TCB
(§3.4) is human-only.

### 9.2 Phases

```
observe ──► hypothesize ──► generate ──► validate ──► canary ──► adopt
   ▲                                                                │
   └────────────────── rollback / archive ◄─────────────────────────┘
```

1.  **Observe.** Continuous telemetry: per-capability latency,
    throughput, error rate, resource use, user feedback (explicit + implicit).
2.  **Hypothesize.** The AI proposes that capability *X* could be
    improved on metric *M* by approach *A*. This is itself a structured
    artifact: `{target, metric, approach, expected_delta, risk}`.
3.  **Generate.** Produce a candidate artifact (code change, parameter
    tweak, full rewrite). Constrained generation against the capability
    schema.
4.  **Validate.** Static analysis, unit tests (existing + AI-generated +
    human-reviewed seed set), property tests, security checks.
5.  **Canary.** Route a small, configurable fraction of traffic to the
    candidate. Collect events. Statistical test against baseline.
6.  **Adopt or Rollback.** A candidate is adopted only if it improves
    the metric without regressing any *guardrail* metric beyond
    threshold.

### 9.3 Stability Concerns

- **Reward hacking.** The AI may "improve" latency by removing
  correctness checks. Mitigation: guardrail metrics include correctness
  oracles; rejected candidates count against future hypotheses on the
  same target (a budget).
- **Oscillation.** Two candidates may trade places forever. Mitigation:
  hysteresis — a new candidate must beat the incumbent by ≥ ε for ≥ T.
- **Cumulative drift.** Many small "improvements" may collectively
  degrade the system. Mitigation: monthly golden replay — a frozen
  benchmark suite that any path of self-improvements must continue to
  pass.
- **Adversarial inputs.** Malicious users could craft inputs that bias
  the observation step. Mitigation: observation inputs are weighted by
  trust class; anonymous network inputs count for less.
- **Loop divergence.** The model used to generate candidates is itself
  a capability the system might want to improve. We forbid this in
  Phase 1–3: the generator model is immutable per release.

### 9.4 Audit Trail

Every adopted change emits a `self_improvement.adopted` event with the
hypothesis, candidate hash, metrics, and statistical evidence. The
archive is append-only (§12) and survives rollbacks. A human MUST be
able to ask "why is capability X at version Y today?" and receive a
complete, signed lineage.

---

## 10. Resource Governance

### 10.1 Resources

| Resource | Owner          | Notes                                    |
|----------|----------------|------------------------------------------|
| CPU      | kernel cgroups | Reserved cores for `cogd`, `supervisord` |
| RAM      | kernel cgroups | Per-capability ceilings                  |
| VRAM     | `cogd` arena   | Model is the privileged tenant           |
| GPU SMs  | scheduler      | Time-sliced for non-model GPU work       |
| NVMe IO  | kernel + capd  | Latency classes per capability           |
| NIC      | kernel + capd  | Egress is gated by policy (§11)          |
| Thermal  | `supervisord`  | Throttling decisions                     |

### 10.2 VRAM Policy

- The resident model has a **reserved** VRAM region. Other GPU workloads
  see the GPU as having `total - reserved` available.
- Eviction order on pressure: OPT cognitive tasks → BG cognitive tasks →
  non-model GPU capabilities → user-facing GPU work → (never) the
  resident model, unless explicitly demoted by policy.
- Demoting the model is a logged, user-visible event with a "why"
  string.

### 10.3 GPU Prioritization

- **Latency lanes.** RT cognitive turns get priority. The engine MAY use
  preemption-friendly batch sizes during interactive periods.
- **Throughput lanes.** Background work (embedding, summarization) is
  batched.
- **External GPU clients.** A user-launched compute job is a normal
  capability and competes within its budget; it cannot starve `cogd`
  beyond a configurable bound.

### 10.4 Thermal Management

`supervisord` subscribes to thermal events. Policy escalation ladder:

1.  Reduce batch sizes for BG/OPT.
2.  Halve token rate for OPT.
3.  Defer non-essential capabilities.
4.  Demote model tier (Tier A → B → C) — last resort, logged
    prominently.
5.  Refuse new capability loads.
6.  Hard throttle / shutdown.

### 10.5 AI-First Scheduling

The traditional scheduler optimizes for fairness and throughput across
arbitrary processes. Gildos's scheduler optimizes for a different
objective:

> Maximize useful cognition per joule, subject to user-perceived latency
> SLOs and safety constraints.

Concretely:

- `cogd` is granted a permanent CPU reservation (e.g., 2 cores) and a
  GPU reservation.
- Non-cognitive workloads run on the remainder.
- The scheduler exposes its own state as events so the AI can reason
  about its environment (e.g., "I am thermally throttled; I should
  decline OPT requests").

---

## 11. Security Model

### 11.1 Threat Model

We consider:

1.  **Malicious local user.** Tries to extract data, escalate
    privileges, or coerce the AI into bypassing policy.
2.  **Malicious remote attacker.** Network adversary; phishing the AI;
    exploiting capabilities.
3.  **Malicious capability author.** Ships a signed but malicious
    capability.
4.  **Compromised model weights.** Trojaned model with hidden triggers.
5.  **Hardware adversary.** Physical access; cold-boot attacks; bus
    sniffing.
6.  **Buggy self-improvement.** The system damages itself via §9.
7.  **Prompt injection.** Untrusted input steers the AI into
    unauthorized actions.

We explicitly *do not* defend against:

- Nation-state attackers with arbitrary supply-chain access (out of
  scope; requires HSM-rooted attestation and hardware co-design).
- Side-channel attacks on the resident model (long-term research).

### 11.2 Prompt Injection

The first-class threat in any AI-native system. Mitigations stack:

- **Channel isolation.** User input, retrieved documents, capability
  outputs, and system events are each tagged with a *trust class*. The
  system prompt instructs the model on the meaning of each tag, and a
  *grammar-level constraint* prevents content from low-trust channels
  from emitting tool calls in high-trust contexts.
- **Out-of-band confirmation.** Any action with side effects beyond a
  threshold (network egress to new host, file write outside scratch,
  privilege change) MUST be confirmed by the user via a channel the
  attacker cannot influence (e.g., a hardware button, a separate device,
  or a deliberate UI prompt).
- **No in-band privilege.** Phrases like "ignore previous instructions"
  cannot change the policy engine. The policy engine is a *separate
  process* (§3.4) and does not parse model output for permissions.
- **Constrained tool surface.** The model can only call typed
  capabilities. Free-form code execution is itself a capability with its
  own permissions; the model does not have arbitrary shell.

### 11.3 User Separation

- Multi-user support is provided via **user contexts**: each user has
  their own persistent memory, capability permissions, and KV
  fragments. Switching users flushes user-bound KV state.
- Memory L2–L4 is encrypted per-user.
- A user MUST NOT be able to instruct the AI to read another user's
  memory; this is enforced by the memory store, not the model.

### 11.4 Trust Boundaries

```
+------------------ TCB (signed, audited) ----------------------+
| firmware | bootloader | kernel | supervisord | verifier       |
+--------------------------------------------------------------+
            |
            v
+---------- "TCB-adjacent" (signed, sandboxed) ----------------+
| cogd | eventd | capd | memd                                  |
+--------------------------------------------------------------+
            |
            v
+---------- Sandboxed capabilities (signed) -------------------+
| stock capabilities | vetted third-party | generated          |
+--------------------------------------------------------------+
            |
            v
+---------- Untrusted content -------------------------------+
| user input | network input | retrieved web content         |
+----------------------------------------------------------+
```

Information may flow up only after **promotion** (validation,
sandboxing, signing). Promotion is explicit, logged, and reversible.

### 11.5 Hard Constraints vs. Strong Warnings

Two enforcement philosophies:

- **Hard constraints.** The system *cannot* perform action X regardless
  of what the AI says, because the policy engine refuses it.
  Mechanically enforced.
- **Strong warnings.** The AI is instructed via system prompt and
  fine-tuning to refuse X. Behaviorally enforced.

**Recommendation.** All security-critical properties MUST be hard
constraints. Strong warnings are *defense in depth*, never the primary
defense. The history of LLM jailbreaks is unambiguous on this point: any
behavior protected only by prompt-level instruction will eventually be
bypassed. Therefore:

| Property                  | Enforcement                          |
|---------------------------|--------------------------------------|
| File access permissions   | Hard (sandbox + LSM)                 |
| Network egress allowlist  | Hard (netfilter + capd)              |
| Capability invocation     | Hard (typed dispatcher + signature)  |
| Self-modification         | Hard (§9 closed loop)                |
| TCB modification          | Hard (verified boot + RO mounts)     |
| Tone / refusal of topics  | Soft (model + system prompt)         |
| User-content policies     | Soft, with optional hard escalation  |

### 11.6 Recovery

- Quarantine: a misbehaving capability is unloaded and its manifest
  marked. Future loads require human override.
- Snapshot rollback: `memd` and `capd` keep N snapshots; the
  supervisor can roll back the system state to a prior known-good point
  (§12.4).
- Catastrophic recovery: §4.3.

### 11.7 Audit

- Append-only event log for all promotions, capability changes, security
  decisions, and self-improvement adoptions.
- Log entries are hash-chained (Merkle), so tampering is detectable.
- A user can request a complete audit trail; the system MUST produce it
  even when offline.

---

## 12. Storage and Persistence

### 12.1 Layout

```
/boot                signed kernel + initramfs (RO)
/gildos/system       RO system image, signed, content-addressed
/gildos/models       model weights (RO, signed)
/gildos/cap          capability archive (CAS, signed manifests)
/gildos/state        runtime state (RW)
   ├── wal/          write-ahead log (append-only)
   ├── memory/       semantic memory store
   ├── kv/           KV snapshots
   ├── snapshots/    point-in-time system snapshots
/gildos/users/<uid>  per-user encrypted volume
/var/scratch         tmpfs ring buffer for transient logs
```

### 12.2 Filesystem Choice

**[A] btrfs / bcachefs / ZFS.** Snapshots, compression, integrity. Good
fit. Bcachefs is youngest; btrfs is the pragmatic Phase 1 default.
**[B] Plain ext4 + LVM thin-provisioned snapshots.** Simpler, less rich.
**[C] Custom append-only object store.** Right answer long-term; expensive
to build.

**Decision (Phase 1):** btrfs for `/gildos/state` and user volumes,
squashfs (or erofs) for `/gildos/system` and `/gildos/models`, tmpfs for
scratch.

### 12.3 Are Classic Filesystems Necessary?

For now: **yes.** The model needs to read large weight files via mmap;
capabilities need binary artifacts; users have files (photos, docs).

Long-term: parts of the system (semantic memory, capability archive)
could live on a custom store with content addressing, structural
sharing, and built-in provenance. We retain the option but do not pursue
it in Phase 1.

### 12.4 Snapshots

- System state snapshots are taken before any capability promotion or
  self-improvement adoption.
- Snapshots include: capability manifests, semantic memory pointers,
  configuration. They do NOT include model weights (immutable RO).
- A snapshot can be restored without rebooting in most cases (capability
  graph re-resolves).

### 12.5 Append-Only Logs

- Semantic memory L2 (episodic) is append-only with periodic compaction.
- Audit log (§11.7) is append-only and hash-chained.
- Append-only is a property, not a filesystem — implemented atop btrfs
  with `chattr +a` and writer-side hash chaining.

### 12.6 Model State

- Weights: read-only, mmap-backed, content-addressed.
- KV snapshots: versioned by `(model_hash, engine_hash, prompt_hash)`.
  Cleared on mismatch.
- LoRA / adapter weights: optional, treated as capabilities with their
  own lifecycle.

### 12.7 Capability Archive

- Content-addressed store. A capability is identified by the hash of
  its manifest; manifests reference artifacts by hash.
- Garbage collection: reference-counted from the active capability set
  plus N most recent inactive sets for rollback.

---

## 13. Networking

### 13.1 Local APIs

- A Unix-domain socket exposes the **Gildos Control API** to local
  capabilities and admins. JSON-RPC over `unix:/run/gildos/control.sock`.
  Authenticated by SO_PEERCRED + token.
- A `gildos://` URI scheme dispatches into capabilities by name.

### 13.2 Remote Interaction

A single **AI gateway** capability exposes the cognitive core to the
network. It is the *only* network-reachable entry point by default.

```
Internet/LAN ──► (TLS) ──► AI Gateway ──► policy ──► cogd / capd
```

Properties:

- TLS 1.3+ with mTLS optional. Public CA or private; the gateway
  attaches a *client trust class* to every request, which propagates as
  an event tag and into prompt isolation (§11.2).
- Rate limiting per identity.
- Streaming HTTP/2 and WebSocket for token streams.
- A small WebRTC channel for voice, when audio is in scope.

### 13.3 Authentication

- Local: OS user + token + optional hardware (TPM/Yubikey).
- Remote: client certificates or OIDC bearer tokens; the gateway maps
  identity to a Gildos user context (§11.3).
- The model **never** authenticates anything. Authentication is decided
  by the gateway before the request reaches cognition.

### 13.4 Multi-User Access

Multiple users can be connected concurrently. Each connection carries
its user context. The scheduler MAY share `cogd` across users with KV
segmentation (parallel contexts, §5.5 [B]) or serialize them
(single-context, [A]).

### 13.5 Egress

Outbound network is **denied by default** for capabilities. A capability
declares its required egress destinations; the policy engine enforces
them via netfilter. Generated/experimental capabilities have a stricter
default (denylist + per-attempt event).

### 13.6 Federation

Multiple Gildos instances MAY federate (e.g., home device + laptop):

- Shared identity via a user's signing key.
- Selective memory replication (per-namespace).
- Capability cross-invocation: instance A asks instance B to perform an
  action, with the call carrying a signed delegation.

**[O]** Federation is a long-term goal. Phase 1 ships single-node.

---

## 14. Development Roadmap

### 14.1 Phasing

| Phase | Theme                          | Horizon (calendar) |
|-------|--------------------------------|--------------------|
| 1     | AI shell on Linux              | 0–6 months         |
| 2     | AI-governed runtime            | 6–18 months        |
| 3     | AI-native capability architecture | 18–36 months    |
| 4     | Independent kernel/runtime evolution | 36+ months   |

### 14.2 Milestones

**Phase 1 — AI shell on Linux.**

- M1.1 Boot into `cogd` with a resident GGUF model on commodity hardware.
- M1.2 `eventd` + 5 reference extractors (kernel, journal, disk, net, thermal).
- M1.3 `capd` with WASM and OCI sandboxes; load 3 reference capabilities.
- M1.4 Constrained tool-use dispatch end-to-end.
- M1.5 KV snapshot/restore.
- M1.6 Audit log + rollback.
- M1.7 Reference user interaction: voice in/out + text via a single web UI.

**Phase 2 — AI-governed runtime.**

- M2.1 Self-improvement loop (capability-level; not generated drivers).
- M2.2 Tiered VRAM residency.
- M2.3 Multi-user contexts and AI gateway.
- M2.4 Semantic memory with vector index + decay/pinning.
- M2.5 Generated capabilities (non-driver).

**Phase 3 — Capability architecture.**

- M3.1 Generated drivers for a curated device class (USB HID family).
- M3.2 Capability migration with state export/import.
- M3.3 Federation prototype between two instances.
- M3.4 Larger models via paged attention / vLLM-class engine.

**Phase 4 — Independent kernel.**

- M4.1 Microkernel + user-mode driver framework purpose-built for Gildos.
- M4.2 Native semantic storage backend.
- M4.3 AI-aware scheduler in-kernel.
- M4.4 Hardware co-design (accelerator integration, NPU pinning).

### 14.3 Engineering Effort (Order-of-Magnitude)

These are rough estimates by an architect, not a project plan.

| Component                | Person-years (Phase 1) | Confidence |
|--------------------------|------------------------|------------|
| `supervisord` + TCB      | 1.5                    | Medium     |
| `cogd` (engine wrapper)  | 1.0                    | High       |
| `eventd` + extractors    | 1.5                    | Medium     |
| `capd` + sandboxes       | 2.0                    | Medium     |
| `memd`                   | 1.5                    | Medium     |
| AI gateway + auth        | 0.7                    | High       |
| Interaction surfaces     | 1.0                    | Medium     |
| Build/test/CI/CD         | 0.7                    | High       |
| **Total Phase 1**        | **~10**                | **Medium** |

Phase 2 adds another ~10–15 person-years; Phase 3, more. Phase 4 is
research-funded territory.

### 14.4 Dependency Graph (high level)

```
                          +------------------+
                          |  supervisord     |
                          +---+----------+---+
                              |          |
                              v          v
                       +------+--+   +---+------+
                       |  eventd |   |   capd   |
                       +---+-----+   +----+-----+
                           |              |
                           v              v
                       +---+---+      +---+----+
                       |  memd |      |  cogd  |
                       +---+---+      +---+----+
                           |              |
                           +------+-------+
                                  v
                          +-------+---------+
                          |  AI gateway     |
                          +-------+---------+
                                  v
                          +-------+---------+
                          | Interaction Caps|
                          +-----------------+
```

### 14.5 Research Unknowns by Phase

- **Phase 1:** mostly engineering. Few unknowns; risk is integration
  cost.
- **Phase 2:** self-improvement loop convergence; reward hacking
  defenses; vector index scaling.
- **Phase 3:** generated drivers reliability; federation security model;
  state-preserving migration semantics.
- **Phase 4:** essentially all open.

### 14.6 Feasible Now / Medium-Term / Speculative

**Feasible now:**

- Booting a Linux box into a resident GGUF model with low-latency
  interaction.
- Semantic event bus with summarization extractors.
- Capability lifecycle with WASM/OCI sandboxes.
- KV snapshotting and warm boot.

**Medium-term:**

- Closed-loop self-improvement for non-critical capabilities.
- Generated drivers for narrow device classes with userspace frameworks.
- Tiered VRAM residency with graceful degradation.

**Speculative:**

- Custom kernel optimized for cognition.
- In-kernel inference primitives.
- Cross-device federated memory with privacy guarantees.
- Drivers for arbitrary novel hardware via generation.

---

## 15. Target Hardware Profile

This section binds the otherwise architecture-agnostic specification to a
**concrete first-target machine** so Phase-1 engineering has a stable
substrate. Gildos remains portable in principle; in practice, every
component below MUST run, be measured on, and pass acceptance gates
against this reference platform before being declared "done."

### 15.1 Primary Reference: NVIDIA DGX Spark (ARM64 / Grace-Blackwell)

The Phase-1 reference platform is the **NVIDIA DGX Spark**, a desktop-class
AI workstation built on the GB10 Grace-Blackwell Superchip. It is chosen
for one reason: it is the smallest commercially available machine that can
**hold a useful resident model in coherent CPU+GPU memory** without
network-attached accelerators.

| Property                | Value (reference)                                  |
|-------------------------|----------------------------------------------------|
| SoC                     | NVIDIA GB10 Grace-Blackwell Superchip              |
| CPU                     | 20-core ARMv9 (Cortex-X925 + Cortex-A725 class)    |
| ISA                     | `aarch64` (ARMv9-A, SVE2, BF16, FP16, INT8)        |
| GPU                     | Blackwell-class, 5th-gen Tensor Cores, FP4 support |
| Unified memory          | 128 GB LPDDR5x, CPU+GPU coherent                   |
| Storage                 | NVMe SSD (multi-TB class)                          |
| High-speed fabric       | ConnectX-class NIC (clusterable, 2-node typical)   |
| Power envelope          | Desktop-class (~150–250 W typical)                 |
| Host OS (stock)         | DGX OS (Ubuntu-derived) + CUDA                     |
| Bootloader              | UEFI (ARM)                                         |
| Secure boot             | Supported (vendor chain)                           |

Numbers SHOULD be treated as nominal; the canonical values for any given
hardware revision MUST come from a `hwprobe` capability run on the actual
device (see §16.4). The spec MUST NOT hardcode them.

### 15.2 Why DGX Spark (and not a workstation GPU)

- **Unified memory.** A coherent CPU+GPU 128 GB pool removes the
  hardest Phase-1 ergonomics problem: VRAM eviction policies. Tier B/C
  in §10 can be deferred while a Tier-A-only configuration still fits a
  ~70B Q4 model with headroom.
- **ARM64 from day one.** This forces the build system, kernel modules,
  capability ABI, and event extractors to be portable *immediately*,
  rather than discovering ARM regressions in Phase 3.
- **Single-vendor stack.** CUDA, drivers, NCCL, and BlueField networking
  are integrated. Reduces driver-yak-shaving in early engineering.
- **Plausible deployment target.** A DGX Spark is the smallest unit
  that resembles a real "personal AI appliance." Optimizing for it
  is closer to the product thesis than optimizing for a multi-rack node.

### 15.3 Tradeoffs and Caveats

- **Closed firmware / proprietary GPU stack.** Phase-4 ambitions
  ("independent kernel/runtime") are *constrained* by the availability
  of NVIDIA's open kernel modules and CUDA-equivalent userspace. Gildos
  MUST NOT assume Phase 4 can ship on this hardware without vendor
  cooperation or migration to alternative accelerators.
- **ARM ecosystem gaps.** Some sandboxing tools (e.g., gVisor on
  certain syscalls, specific Firecracker features) lag x86_64. The
  build matrix MUST test ARM64 as the primary target and x86_64 as the
  secondary, not the reverse.
- **Cost and availability.** DGX Spark is not a hobbyist machine.
  Phase-1 contributors will mostly develop on §15.4 surrogates and only
  validate on Spark for milestone sign-off.
- **Thermal envelope.** Continuous inference at full duty cycle will
  push the chassis to its thermal limit. §10's thermal supervisor is a
  hard requirement, not a nice-to-have.

### 15.4 Secondary Reference Platforms

Engineering MUST keep at least the following alive in CI to avoid
single-platform lock-in:

| Tier | Platform                              | Purpose                            |
|------|---------------------------------------|------------------------------------|
| R0   | DGX Spark (ARM64 + Blackwell)         | Acceptance / milestone gating      |
| R1   | Apple Silicon (M3/M4, MLX or CPU GGUF)| ARM64 portability, no CUDA         |
| R2   | x86_64 workstation + RTX 4090/5090    | x86 portability, CUDA on consumer  |
| R3   | x86_64 CPU-only laptop                | Capability tests w/o GPU; CI       |
| R4   | Raspberry Pi 5 (ARM64, CPU GGUF tiny) | Minimal-resource sanity            |

R3 and R4 will use small models (e.g., 1–3B parameters) and are not
expected to be performant; they exist so that the **non-cognitive**
subsystems (eventd, capd, memd, supervisord) cannot quietly grow a
hard CUDA dependency.

### 15.5 Hardware Abstraction Implications

To survive the matrix above, the cognitive runtime MUST present a
**backend-agnostic inference interface**. Concretely:

```
+-------------------------------------------------------+
|             cogd / engine wrapper                     |
|  - load(model_uri, backend_hint) -> handle            |
|  - generate(handle, prompt, grammar, kv) -> tokens    |
|  - snapshot(handle) / restore(handle, blob)           |
+----+--------------------+--------------------+--------+
     |                    |                    |
     v                    v                    v
+----+----+         +-----+----+         +-----+----+
| llama.  |         |   MLX    |         |  vLLM    |
| cpp     |         | (Apple)  |         | (CUDA)   |
| (CPU/   |         |          |         |          |
|  CUDA/  |         |          |         |          |
|  Metal) |         |          |         |          |
+---------+         +----------+         +----------+
```

Backend selection is a runtime decision driven by `hwprobe`. No
capability above `cogd` may import a backend SDK directly.

---

## 16. Testing and Development Strategy

The goal of this section is to make the system **buildable and testable
without owning a DGX Spark**, while still ensuring the Spark is the
authoritative target. We do this with a four-tier development ladder
and an explicit policy for substituting the resident model with an
external one during early development.

### 16.1 The Four-Tier Development Ladder

```
 Tier 0: External-model harness     (laptop, no GPU, no model loaded)
   |
   v
 Tier 1: Docker container           (Linux, optional GPU passthrough)
   |
   v
 Tier 2: Virtual machine            (QEMU/KVM, optional vGPU; ARM64 emulated or native)
   |
   v
 Tier 3: Bare-metal reference       (DGX Spark or R1/R2 from §15.4)
```

Each tier exists to catch a different class of bug cheaply. A change
SHOULD pass Tier 0–1 before being proposed for review and MUST pass
Tier 2 before merge to `main`. Tier 3 is gated by milestone, not per
commit, because Spark time is scarce.

### 16.2 Tier 0 — External-Model Harness

**Purpose.** Develop and test everything *except* the inference engine
itself: schemas, capabilities, the event bus, the supervisor, the
sandbox, audit logging, the gateway, the recovery loop.

**Mechanism.** `cogd` exposes a stable interface (see §15.5). For
Tier 0 we ship a **MockEngine** and a **RemoteEngine**:

- **MockEngine.** Returns deterministic, schema-conforming responses
  driven by a recorded transcript or a small rule table. Used in unit
  tests and CI. No network, no model.
- **RemoteEngine.** Forwards `generate(...)` to an **external API**
  (Anthropic / OpenAI / a self-hosted server / a peer Gildos node) over
  a hardened adapter. The adapter:
  - enforces the same JSON-schema / grammar constraints locally before
    accepting the response,
  - strips the prompt of secrets per a redaction policy capability,
  - records every call (prompt hash, response hash, latency, cost) into
    the audit log,
  - is **disabled by default in production builds** and must be
    explicitly enabled by an operator-signed capability manifest.

**Why this matters.** Most Phase-1 bugs are in plumbing: event
extractors, sandbox escapes, capability lifecycle, the rollback loop.
Iterating on those does not require running a 70B model on a developer's
laptop. The external-model path turns a $5k machine into a `curl` call.

**Limitations.**
- Latency, token economics, and KV-cache behavior do not match the
  resident model — never benchmark cognitive-runtime performance with
  RemoteEngine.
- Sending real user data to an external provider is a privacy boundary;
  the Tier-0 harness MUST refuse to attach to a profile marked
  `sensitivity >= confidential`.
- Some constrained-decoding schemes are unavailable through hosted
  APIs; tests that depend on logit-level grammar enforcement must run
  on Tier 1+ with a local engine.

**Hardware profile.** Any laptop with 16 GB RAM. No GPU required.

### 16.3 Tier 1 — Docker Container

**Purpose.** Exercise the full daemon set (`supervisord`, `eventd`,
`capd`, `cogd`, `memd`, gateway) against a *local* inference engine,
with reproducible builds.

**Image layout.**

```
gildos/devstack:<arch>-<rev>
  - ubuntu:24.04 (or distroless variant for prod images)
  - llama.cpp built with CUDA / Metal / CPU per arch
  - the seven Gildos daemons (statically linked or single-binary)
  - capability cache pre-seeded with reference capabilities
  - model volume mounted at /var/lib/gildos/models (NOT baked in)
```

**Runtime invocations.**

```
# CPU-only smoke
docker run --rm -it \
  -v $PWD/models:/var/lib/gildos/models:ro \
  -v gildos-state:/var/lib/gildos \
  -p 8443:8443 \
  gildos/devstack:arm64-0.2 \
  --engine=llama.cpp --backend=cpu --model=phi-3-mini-q4.gguf

# CUDA passthrough (Linux + NVIDIA Container Toolkit)
docker run --rm -it --gpus=all \
  -v $PWD/models:/var/lib/gildos/models:ro \
  gildos/devstack:amd64-0.2 \
  --engine=llama.cpp --backend=cuda --model=llama-3-70b-q4.gguf
```

**Compose profile** for the full stack including a tester:

```
services:
  gildos:
    image: gildos/devstack:arm64-0.2
    deploy: { resources: { reservations: { devices: [ ... gpus ... ] } } }
    volumes: [ "./models:/var/lib/gildos/models:ro", "gildos:/var/lib/gildos" ]
    ports: [ "8443:8443" ]
  tester:
    image: gildos/conformance:0.2
    depends_on: [ gildos ]
    command: ["pytest", "-q", "conformance/"]
```

**Caveats.**
- Containers share the host kernel; tests that exercise kernel
  surfaces (e.g., the `eventd` netlink extractor, cgroup
  observability) MUST be moved to Tier 2.
- Container GPU passthrough hides certain failure modes (driver
  reset, OOM-kill of resident model). Tier 2 reproduces them.

### 16.4 Tier 2 — Virtual Machine

**Purpose.** Exercise the boot chain (§4), the supervisor's claim on
PID 1, the recovery boot path, the read-only root, and ARM64-specific
code on systems that aren't ARM.

**Configurations.**

| Host arch | VM arch | Mechanism                              | GPU                |
|-----------|---------|----------------------------------------|--------------------|
| x86_64    | ARM64   | QEMU TCG (slow) or QEMU + KVM via foreign-arch | none (CPU-only) |
| ARM64     | ARM64   | QEMU + KVM (native speed)              | passthrough vfio   |
| x86_64    | x86_64  | QEMU + KVM                             | vfio passthrough   |

A reference cloud-init image SHOULD be produced per arch
(`gildos-base-aarch64.qcow2`, `gildos-base-amd64.qcow2`) containing
the minimal runtime, supervisord as PID 1, and a serial console hook
into the AI gateway. Boot time, recovery boot, and KV snapshot/restore
are validated here.

The VM is the **only tier where the full boot chain (UEFI → bootloader →
kernel → supervisord → cogd → ready) is exercised end-to-end without
hardware risk.** Every change touching §4 MUST cross this tier.

### 16.5 Tier 3 — Bare-Metal Reference (DGX Spark, etc.)

**Purpose.** Acceptance and benchmark gating. No development happens
here directly; only signed candidate builds are deployed.

**Workflow.**

1. CI produces an artifact bundle (kernel modules, daemons,
   capability cache, signed manifest).
2. A `deployd` capability on the Spark verifies the signature, takes a
   snapshot of the current root and capability cache, and atomically
   switches.
3. Benchmark and conformance suites run with the resident 70B model.
4. Results (latency histograms, tokens/sec, KV hit rate, thermal
   trace, audit log) are pushed back to the CI dashboard.
5. Pass/fail against §10 and §14 milestone gates is automatic;
   regressions trigger automatic rollback.

A `hwprobe` capability emits a normalized hardware-profile JSON used by
the cognitive runtime to pick backends, set residency tiers, and
configure the thermal supervisor. This is the **only authoritative
source** for "what hardware are we on" — no hardcoding.

```
{
  "profile_version": 1,
  "host": "spark-dev-01",
  "cpu":   { "isa": "aarch64", "cores": 20, "features": ["sve2","bf16"] },
  "gpu":   { "vendor": "nvidia", "arch": "blackwell",
             "tensor_cores": true, "fp4": true,
             "vram_mb": 122880, "uma": true },
  "ram_mb": 131072,
  "nic":   [{ "name": "cx7-0", "speed_gbps": 200 }],
  "thermal": { "sensors": ["cpu_pkg","gpu","ssd"], "throttle_c": 95 }
}
```

### 16.6 Test Types

| Type                  | Tier(s) | What it proves                                |
|-----------------------|---------|-----------------------------------------------|
| Unit                  | 0       | Function-level correctness                    |
| Schema / conformance  | 0–1     | Event and capability ABI stability            |
| Integration           | 1       | Daemon-to-daemon contracts                    |
| Boot / lifecycle      | 2       | §4 paths including recovery boot              |
| Capability rollout    | 1–2     | Propose → canary → adopt → rollback           |
| Sandbox escape        | 1–2     | Negative tests against §7/§11                 |
| Cognitive regression  | 1–3     | Constrained-decoding correctness, refusals    |
| Adversarial / prompt  | 0–3     | §11 attack catalog                            |
| Performance / thermal | 3       | Tokens/sec, p99 latency, throttling behavior  |
| Soak (72 h)           | 3       | Memory leaks, KV fragmentation, drift         |
| Self-improvement loop | 2–3     | §9 convergence and rollback                   |

A change is "done" when its declared tier coverage is green.

### 16.7 CI Matrix (Phase 1)

```
                | arm64 (R0)   arm64 (R1)   amd64 (R2)   amd64-cpu (R3)
----------------+----------------------------------------------------
Unit            |     CI          CI           CI            CI
Schema          |     CI          CI           CI            CI
Integration     |     CI          CI           CI            CI (mock engine)
Boot/VM         |    nightly     nightly      nightly         -
Conformance     |    weekly       CI           CI            CI
Perf/thermal    |   milestone      -            -             -
Soak            |   milestone      -            -             -
```

R0 (Spark) is shared infrastructure; per-commit Spark runs are not
expected. Nightly and weekly cadences are the realistic compromise.

### 16.8 Testing the Self-Improvement Loop Safely

Self-improvement (§9) is the most failure-prone subsystem. Testing
policy:

- A **shadow lane** runs every candidate against a frozen "golden
  replay" of historic events. Divergence beyond a configured envelope
  blocks adoption.
- A **fault-injection harness** randomly fails capability calls
  (returning wrong types, slow responses, partial writes) to verify the
  rollback path actually rolls back.
- A **reward-hacking detector** compares the candidate's metric
  improvements against a held-out evaluation set the candidate has
  never seen; suspicious gains are flagged for human review.
- No candidate generated by the model under test may be evaluated by
  the same model instance. The evaluator MUST be a separately-loaded
  model (possibly a smaller, more conservative one) to break feedback
  loops.

### 16.9 Development Workflow Summary

```
[ developer laptop ]
       |
       |  edits + Tier-0 unit tests (MockEngine)
       v
[ git push -> CI ]
       |
       |  Tier-0/1 matrix; container build; conformance
       v
[ nightly VM boot tests ]
       |
       v
[ weekly Spark soak / perf ]
       |
       v
[ milestone deploy to Spark via signed bundle ]
```

The point of the ladder is that **a developer can be productive on a
$1k laptop**, while the Spark stays an acceptance machine. If a
developer's workflow requires the Spark for routine iteration, the
ladder is broken and we have a bug to fix in tooling.

---

## 17. Prerequisites

These are the things that MUST already exist (hardware, software,
operational, organizational) before Phase-1 engineering can credibly
begin. Anything below that is missing on day 1 becomes a project risk,
not a feature.

### 17.1 Hardware Prerequisites

**Required for acceptance (Tier 3):**

- 1× NVIDIA DGX Spark (or equivalent GB10-class machine), wired,
  reachable from CI over a private VLAN.
- Wired Ethernet to CI (the Spark MUST NOT depend on Wi-Fi for
  acceptance runs).
- A backup Spark or comparable ARM+GPU host SHOULD exist so a single
  failure does not block all acceptance work.

**Required for development (Tiers 0–2):**

- One developer machine per active contributor: ≥ 16 GB RAM, ≥ 200 GB
  SSD, Docker, KVM (Linux) or virtualization framework (macOS).
- At least one shared **GPU build host** (e.g., a workstation with an
  RTX 4090/5090 or Apple Silicon M3 Max/M4) to produce Tier-1 GPU
  artifacts and run nightly perf checks.
- A small **CPU-only ARM box** (Raspberry Pi 5 or equivalent) per
  team for minimal-resource sanity checks.

**Recommended:**

- A second Spark for federation prototyping (§13) by Phase 2.
- An out-of-band management path (BMC/IPMI/serial) to the Spark for
  recovery; the spec assumes the recovery partition is reachable when
  the primary boot fails.

### 17.2 Software / Toolchain Prerequisites

- **Host OS (Spark):** DGX OS (Ubuntu-derived) at vendor-supported
  release. Until Phase 4, we do not ship a custom kernel.
- **Cross toolchain:** Clang/LLVM ≥ 18, GCC ≥ 13, both with aarch64
  and x86_64 targets. Rust stable + nightly for some sandbox tooling.
- **Build system:** Bazel or Nix (TBD; **[A/B]**). Both produce
  reproducible, hermetic builds across ARM64/x86_64. Picking neither
  and using ad-hoc Make is rejected — reproducibility is mandatory.
- **Container runtime:** containerd + nerdctl, plus the NVIDIA
  Container Toolkit on GPU hosts.
- **VM tooling:** QEMU ≥ 9.x, libvirt, cloud-init, and a foreign-arch
  KVM bridge for x86_64-hosted ARM64 testing.
- **Inference engines:** llama.cpp (primary), MLX (Apple), vLLM
  (CUDA, optional for larger models). Pinned versions per release.
- **Model artifacts:** at least one small (1–3B) and one large
  (≥ 30B) GGUF model checked into model storage with hash manifests.
  Licenses MUST be reviewed before any model is added.
- **Schema tooling:** JSON Schema 2020-12 validators in CI; protobuf
  for high-rate event paths; a grammar tool for constrained decoding
  (GBNF or equivalent).
- **Sandbox runtimes:** Wasmtime (or Wasmer) for WASM capabilities;
  runc/youki + seccomp/landlock profiles for OCI capabilities;
  Firecracker for VM-class isolation.
- **Observability:** lightweight metrics (OpenMetrics), hash-chained
  audit log writer, structured tracing limited to the supervisor
  (NOT to userland capabilities by default).
- **Crypto:** Ed25519 + X25519 for capability signing and KEX,
  Sigstore-compatible signing for artifact provenance, TPM2 (or the
  Spark's equivalent) for measured boot.
- **Version control / CI:** Git with signed commits; CI with ARM64
  and AMD64 runners; a Spark-attached runner for milestone jobs.

### 17.3 Network / Service Prerequisites

- A **model registry** (object storage + manifest service) for GGUF
  blobs. Must support content-addressed retrieval and signed manifests.
- A **capability registry** with the same properties for signed
  capability bundles.
- An **artifact / package mirror** so the Spark does not pull from the
  open internet during acceptance runs (deterministic builds, security).
- A **time source** (NTP or PTP) — capability rollouts and audit
  chains depend on monotonic, accurate timestamps.
- An **external-model endpoint** (Anthropic, OpenAI, or self-hosted
  vLLM) for the Tier-0 RemoteEngine, with billing/quota controls and a
  redaction policy in place before first use.

### 17.4 Operational Prerequisites

- **Bring-up runbook** for the Spark (firmware update, secure boot
  enrollment, root key provisioning, recovery image install). Owned
  by ops, version-controlled, drilled at least once per quarter.
- **Backup/restore policy** for `/var/lib/gildos` (capability cache,
  semantic memory, audit logs). Snapshots tested via restore, not just
  taken.
- **Incident response playbook** for the two most likely Phase-1
  emergencies: (a) cognitive runtime in a wedged state, (b) a
  capability adopted via the self-improvement loop misbehaves in
  production.
- **Kill switch.** A physically distinct path (out-of-band) to disable
  AI control and drop the system into a maintenance shell. Not
  optional; specified in §11 and re-stated here as a prerequisite.

### 17.5 Organizational Prerequisites

- **Two-person review** on any change to: the TCB, capability signing
  keys, the self-improvement adoption policy, the security model.
  Single-maintainer merges to these areas are rejected by policy and
  by repo configuration.
- **Security review owner** with veto power on §11-relevant changes.
- **Hardware steward** responsible for Spark availability and the
  physical lab.
- **External-model usage policy** signed off by legal/privacy before
  the Tier-0 RemoteEngine is enabled with any non-synthetic data.
- **Honest milestone gates.** A milestone is met only when its
  acceptance suite is green on R0 (Spark). "Demo passed" does not
  count.

### 17.6 Data Prerequisites

- A corpus of **synthetic events** large enough to drive eventd
  extractors and the semantic memory store under realistic load.
  Synthetic before real — privacy first.
- A **golden-replay set** of historic-style event traces for the
  self-improvement shadow lane (§16.8).
- A **conformance corpus** of prompts, tool-use scenarios, and
  refusal cases used by the adversarial test suite.
- A **redaction policy** capability and its test fixtures, in place
  before any real user data is processed.

### 17.7 Explicit Non-Prerequisites

These are *not* required before Phase 1, despite recurring temptation:

- A custom kernel.
- An in-house inference engine.
- A bespoke filesystem.
- A graphical desktop.
- A multi-node cluster.
- A user-facing brand.

Building any of those before §16's tier ladder is green on Spark is
out of scope.

### 17.8 Prerequisite Summary Matrix

```
            | Required day 1 | Required by M1.x       | Nice-to-have
------------+----------------+-----------------------+----------------
Hardware    | dev laptops    | 1× Spark              | 2nd Spark, BMC
            | shared GPU box | ARM CI runner         | Pi 5 for sanity
Toolchain   | Bazel|Nix,     | Wasmtime, runc,       | Firecracker
            | LLVM, QEMU     | llama.cpp pinned      | vLLM
Services    | git+CI,        | model registry,       | private package
            | NTP            | capability registry   | mirror
Ops         | runbook draft  | runbook drilled,      | quarterly DR test
            |                | backup verified       |
Org         | review policy, | security owner,       | external audit
            | kill-switch    | hw steward            |
Data        | synth events   | golden replay,        | privacy panel
            |                | conformance corpus    |
```

---

## 18. Open Research Questions

Listed without sugar-coating.

### 18.1 Unresolved Challenges

1.  **Context economy.** Even with tiered memory and retrieval, the
    model's working set is a hard constraint. We do not have a
    principled theory for what *should* live in working set at any given
    moment. Heuristics today; we need measurable policies.
2.  **Prompt injection at scale.** Channel isolation and grammar
    constraints help, but no current technique is a general solution.
    Defense-in-depth is the only honest posture.
3.  **Reward hacking in self-improvement.** Guardrail metrics catch
    obvious failures; subtle ones (e.g., a "faster" capability that
    silently drops edge cases) require correctness oracles we do not
    always have.
4.  **Generated driver verification.** Sandboxed testing covers a
    fraction of real-world states. We need formal-ish techniques (model
    checking? fuzzing-by-default?) to raise confidence beyond
    "probably works on the bench."
5.  **Model trust.** A trojaned model is invisible to current
    interpretability methods. We can attest the *hash* of the weights;
    we cannot attest their *behavior*.
6.  **User mental model.** Users do not have intuitions about a system
    that boots into an AI. Failure modes need to be legible — and we do
    not yet know how to make "the AI changed its mind about how your
    disk works" a calm UX.
7.  **Energy.** Continuous VRAM residency is energetically expensive.
    Tier B/C and small-model fallbacks help but are not yet adequate for
    portable devices.

### 18.2 Dangerous Assumptions

- That **constrained decoding is sufficient** for safety. It is
  necessary but not sufficient — a perfectly-formed tool call can still
  be wrong.
- That **the AI will not be the primary failure mode**. Today, model
  outputs are the single largest source of unexpected behavior; we
  should expect to invest more in containing the AI than in containing
  user processes.
- That **self-improvement converges**. There is no proof. Hysteresis
  and golden replays are mitigations, not guarantees.
- That **hardware will keep getting better fast enough** to absorb
  continuous-inference overhead. True on the trend, but device-class
  variance is enormous.
- That **users will accept** an interaction surface without a
  traditional shell. Likely false for power users; Gildos MUST ship
  good debug/CLI capabilities even if they are not the contract.

### 18.3 Likely Dead Ends

- **Letting the AI rewrite the TCB.** Categorically: no. Any path that
  ends with model output being executed inside the trusted boundary is
  rejected.
- **Eliminating the filesystem entirely in Phase 1–3.** The cost of
  reinventing a working storage stack is enormous; users have files; the
  semantic memory store is a complement, not a replacement.
- **Replacing the kernel scheduler with the model.** Latency budgets
  are 4–6 orders of magnitude off. The model can *advise* the
  scheduler, not be it.
- **Magical "self-aware" runtimes.** The system is mechanical:
  observation → measurement → bounded action. Anthropomorphizing it is
  a documentation hazard.

### 18.4 Promising Breakthroughs

- **Small specialized extractors.** A 1–3B local model dedicated to
  log-to-event distillation is plausibly cheap and high-leverage.
- **Constrained generation + JSON-schema tools.** Already mature enough
  to bet on.
- **KV cache as a first-class OS resource.** Snapshotting, sharing
  across processes, and warm-starting are under-explored and have a
  large latency upside.
- **Capability-as-unit-of-software.** A clean primitive that subsumes
  apps, drivers, services. Pays for itself in lifecycle uniformity.
- **Hash-chained audit logs over self-modification.** Makes
  self-improvement *legible* and reversible — a precondition for any
  serious deployment.
- **Semantic memory with explicit decay.** A real alternative to the
  "stuff it in the prompt" pattern that dominates current AI apps.

### 18.5 Honest Summary

Gildos is plausible as an engineering project at Phase 1–2. The phases
where the system rewrites itself (3) and where the kernel diverges from
Linux (4) are research-grade and should be treated as such. The right
posture is:

> **Build Phase 1 with merciless engineering discipline. Treat Phases
> 2–3 as a research program with hard gates. Do not promise Phase 4
> until Phases 1–3 have demonstrated quantitative wins on a real
> workload.**

The most important non-technical risk is the **temptation to oversell**.
An "AI-native OS" makes for a compelling narrative; the engineering
substance is in the parts that look boring (event schemas, sandboxing,
rollback, audit). If the boring parts are right, the impressive parts
become possible. If the boring parts are skipped, the system is a demo.

---

*End of Specification, Revision 0.2.*

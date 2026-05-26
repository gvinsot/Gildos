# GILDOS — An AI-Native Operating System

**Engineering Specification, Revision 0.4**
**Classification:** Internal / Exploratory Research
**Date:** 2026-05-27
**Editor:** Principal Systems Architecture Group
**Status:** Draft for technical review. Not a product roadmap.

---

## What changed in Revision 0.4

Revision 0.4 is a **focusing pass**. The thesis and trust model are
unchanged; the spec is shorter, sharper, and more opinionated.

- **§4 The Gildos Kernel** is a new top-level section. It is now the
  load-bearing piece of the document — the rest of the system is
  organized around what the kernel provides.
- **All A/B alternatives from rev 0.3 are resolved.** Where the prior
  draft hedged, this draft picks (and says why). See §2.6.
- **Six AI-native kernel primitives** are introduced (§4.4):
  *DirectModelLoad*, *Tensor Handles*, the *Cognitive Scheduling Class*,
  *Semantic Syscall Rings*, *Speculative Syscall Prefetch*, and
  *VRAM as a first-class managed resource*. These are the smallest set
  of net-new mechanisms required to make AI structurally faster on this
  OS than on a stock Linux host.
- **WASM Component Model is the sole sandbox in Phase 1.** OCI and
  Firecracker are deferred to Phase 3+ for specific use cases.
- **Rust is the implementation language** for every Gildos-authored
  component above the host kernel. C / C++ is permitted only for the
  inference engine and vendor drivers.
- **Nix flakes** are the build system. Bazel is dropped.
- **btrfs + squashfs + tmpfs** is the storage stack. No alternatives.
- The roadmap is unchanged in spirit but compressed (§18).

Sections from rev 0.3 are preserved in meaning; verbose justifications
were trimmed wherever a decision is now made.

---

## Document Conventions

- **MUST / SHOULD / MAY** follow RFC 2119 semantics.
- **[O]** marks open research questions (collected in §19).
- **Engineering rigor over enthusiasm.** Claims without a concrete
  mechanism are speculation and are marked as such.
- "The AI" or "the cognitive core" means the resident inference engine
  plus its supervising runtime — not anthropomorphic agency.

---

## Table of Contents

1.  Executive Summary
2.  Principles and Enforced Choices
3.  Architecture at a Glance
4.  **The Gildos Kernel**
5.  Boot, Lifecycle, and Recovery
6.  Cognitive Runtime (`cogd`)
7.  Semantic Event Bus
8.  Capability System
9.  Semantic Memory
10. Self-Improvement Loop
11. Continuous Learning and Skill Acquisition
12. Security Model
13. Storage and Persistence
14. Networking, Identity, and Federation
15. Generated Drivers
16. Target Hardware
17. Testing and Development Strategy
18. Roadmap and Prerequisites
19. Open Research Questions

---

## 1. Executive Summary

### 1.1 Thesis

Gildos inverts the stack. Instead of AI being an application running on
an OS, the OS is organized around a continuously-resident inference
engine as its primary cognitive layer.

```
Traditional:    Hardware → Kernel → Userspace → Applications → AI feature
Gildos:         Hardware → Kernel → Cognitive Runtime → Capabilities → User
```

If inference is cheap, continuous, and structurally trusted, the
operating system's job changes from *running applications* to
*running cognition*. Programs become hypotheses; the OS evaluates them.

### 1.2 What makes Gildos different

1.  **The kernel itself is shaped for cognition.** Latency budgets, the
    scheduler, memory tiers, and a small set of new syscalls are
    optimized for one workload: keeping a large model resident,
    responsive, and well-fed. See §4.
2.  **Capabilities, not applications.** The installable unit is a
    signed, sandboxed WASM component with a typed interface, a resource
    budget, and a provenance record. The cognitive core invokes them by
    *intent*, not by binary path.
3.  **The event bus is semantic.** The AI never reads raw logs. It
    subscribes to typed, scored, schema-validated events extracted at
    the boundary.
4.  **Self-modification under contract.** The system can rewrite its
    own capabilities — and acquire new skills (§11) — but only through
    a closed loop of *propose → compile → sandbox → benchmark → canary
    → adopt → rollback*. Nothing magical.
5.  **Minimum trusted surface.** The TCB is small, auditable, and signed.
    AI output is never trusted as code inside the TCB. Ever.

### 1.3 Why this should make the AI more powerful

Stock Linux is hostile to a resident model in subtle ways:

- the scheduler does not know that swapping out the model is catastrophic;
- the page cache does not understand that weight files should be locked
  in VRAM, not RAM;
- syscalls are byte-oriented and the model wastes context parsing them;
- VRAM is a second-class resource owned by an opaque user-mode driver;
- IPC requires the model's output to be re-serialized for every tool call.

Gildos fixes each of these. The kernel knows about the model, the
scheduler protects it, VRAM is a first-class managed resource, syscalls
are typed semantic streams, tool calls are zero-copy tensor handoffs,
and weights stream from NVMe directly into VRAM (§4.4). The cumulative
effect — measured against a stock Linux baseline running the same
inference engine — is the load-bearing claim of this project.

### 1.4 What this spec is not

- Not a product. There is no UX commitment.
- Not a Linux distribution. Phase 1 builds **on** Linux LTS; Phase 4
  replaces it.
- Not a chatbot. The model is a *resident system component*, not an app.
- Not a promise of self-awareness. Every adaptive behavior is mechanical:
  observe → measure → bounded action → audit.

---

## 2. Principles and Enforced Choices

### 2.1 Cognition first

The scheduler, memory hierarchy, IO classes, and event system treat one
or more inference processes as **continuously resident,
non-preemptible-by-default** workloads. A swap-out of the resident
model is a *system event*, not a routine scheduler decision; it MUST be
logged, justified, and reversible.

### 2.2 Capabilities, not applications

The smallest installable unit is a versioned, signed, sandboxed
**capability** with a JSON-Schema-typed interface. Capabilities replace
applications, drivers, system services, and shell utilities. One
mechanism for all extension surfaces.

### 2.3 Semantic over syntactic

Internal state reaches the cognitive core as **typed events** with
relevance scores and provenance. Raw byte streams (dmesg, journalctl,
syscall traces) are summarized at the boundary by deterministic
extractors and small specialized models. The AI ingests *meaning*, not
lines of text.

### 2.4 Self-modification under contract

The system MAY rewrite capabilities and acquire new skills, but only
through the closed loops in §10 and §11. The system MUST NOT modify
itself outside those loops, bypass signing or sandboxing, persist a
modification that has not been canaried, or touch the TCB at runtime.

### 2.5 Minimum trusted surface

The TCB is the smallest set of components whose compromise compromises
the whole system. It is small enough to fit in a human's head:

- bootloader, host kernel, microvisor binary, signing keys;
- the capability verifier (signature + manifest validator);
- the sandbox enforcer (seccomp/landlock + Wasmtime policies);
- the kernel modules that implement §4's AI-native primitives.

The TCB excludes the model weights, the inference engine, and any
generated or self-improved code.

### 2.6 Enforced choices (no more A/B)

Rev 0.3 deferred several decisions. They are now made.

| Question                              | Decision                            | Why                                                                 |
|---------------------------------------|-------------------------------------|---------------------------------------------------------------------|
| Implementation language above kernel  | **Rust** (stable + 2024 edition)    | Memory safety, mature WASM tooling, single toolchain across daemons |
| Build system                          | **Nix flakes**                      | Hermetic, content-addressed, no Bazel learning tax                  |
| Sole sandbox runtime (Phase 1)        | **Wasmtime (Component Model)**      | One sandbox is simpler than three; component model gives typed IPC  |
| OCI / Firecracker                     | **Deferred to Phase 3+**            | Only needed for vendor drivers and large legacy code                |
| Filesystem                            | **btrfs (state) + squashfs (RO) + tmpfs (scratch)** | Snapshots, compression, integrity; no exotic dependencies |
| Inference engine                      | **llama.cpp**, pinned per release   | Auditable, embeddable, broad backend support                        |
| Phase-1 reference model               | **Qwen3.6 Prismaquant 5.5b + MTP**  | Fits DGX Spark unified memory at quality tier; speculative decoding |
| KV-store backend                      | **LMDB**                            | Memory-mapped, crash-safe, tiny                                     |
| Vector index                          | **sqlite-vec** (Phase 1) → custom (Phase 3+) | SQL ergonomics; swap when bottlenecked                      |
| Audit log format                      | **Merkle-chained CBOR**             | Compact, deterministic, signable                                    |
| Schema language                       | **JSON Schema 2020-12 + GBNF**      | JSON Schema for capability interfaces; GBNF for decoder grammars    |
| Scheduling [A]/[B] from rev 0.3       | **B: paged-attention, parallel contexts** | Required for multi-user and for OPT background cognition      |
| Boot reference                        | **DGX Spark (ARM64 + Blackwell)**   | Smallest unit that holds a useful resident model in unified memory  |
| PID 1                                 | **`gildosvisor`** (Rust microvisor) | Successor to systemd; signed, attested, recoverable                 |

Every choice above is reversible *by spec amendment*, not by individual
contributor preference. The cost of one wrong choice is a future
migration; the cost of unresolved choices is permanent paralysis.

---

## 3. Architecture at a Glance

### 3.1 Layered view

```
+----------------------------------------------------------------+
|  L7  Users, networks, sensors                                  |
+----------------------------------------------------------------+
|  L6  Interaction surfaces (TTS/STT, GUI, CLI, HTTPS gateway)   |  optional
+----------------------------------------------------------------+
|  L5  Cognitive Runtime                                         |
|        - Inference engine (llama.cpp)                          |
|        - Reasoning loop + tool dispatcher                      |
|        - Context / memory manager                              |
+----------------------------------------------------------------+
|  L4  Capability Plane (WASM components)                        |
+----------------------------------------------------------------+
|  L3  Semantic Event Bus + Semantic Memory Store                |
+----------------------------------------------------------------+
|  L2  Microvisor (`gildosvisor`, PID 1, TCB)                    |
|        - Policy engine, signer, attestation, rollback          |
+----------------------------------------------------------------+
|  L1  Host Kernel + Gildos Kernel Modules                       |
|        - Linux LTS (Phase 1) → Gildos μKernel (Phase 4)        |
|        - AI-native primitives (§4.4): DirectModelLoad,         |
|          tensor handles, cognitive sched class, semantic       |
|          syscall rings, speculative syscall prefetch, VRAM mgmt|
+----------------------------------------------------------------+
|  L0  Hardware (CPU, GPU/NPU, RAM, VRAM, NVMe, NIC, sensors)    |
+----------------------------------------------------------------+
```

### 3.2 Resident processes

```
| Process            | Privilege | Persistence | Role                          |
|--------------------|-----------|-------------|-------------------------------|
| gildosvisor        | TCB       | Always-on   | PID 1; verifier; policy       |
| cogd               | TCB-adj.  | Always-on   | Owns inference engine + KV    |
| eventd             | TCB-adj.  | Always-on   | Semantic bus broker           |
| capd               | TCB-adj.  | Always-on   | Capability lifecycle          |
| memd               | TCB-adj.  | Always-on   | Semantic memory store         |
| Capability inst.   | Sandbox   | On-demand   | One WASM instance per call    |
```

TCB-adjacent means signed, audited, but recoverable on compromise. A
compromise of `cogd` is recoverable (re-attest, re-load weights). A
compromise of `gildosvisor` is not (rebuild from media).

### 3.3 Information flow

```
  user / network / sensors
            │
            ▼
+--------------------+
|  Interaction Caps  |
+----------+---------+
           │ events (typed, signed)
           ▼
+--------------------+          +--------------------+
|       eventd       │◄────────►│   gildosvisor      |
+----------+---------+          | (policy, audit)    |
           │                    +----------+---------+
           ▼                               │
+--------------------+          +----------▼---------+
|        cogd        │◄────────►│        memd        |
| (model + reason)   │  tool    | (semantic store)   |
+----------+---------+  calls   +--------------------+
           │
           ▼
+--------------------+
|        capd        |
| (WASM lifecycle)   |
+----------+---------+
           │
           ▼
+--------------------+
|  Gildos Kernel     |  (§4)
+----------+---------+
           │
           ▼
        Hardware
```

---

## 4. The Gildos Kernel

This is the most important section of the spec. The kernel is what
makes Gildos *structurally* faster and safer for a resident-model
workload than anything available today. Everything above it is built
to consume what it exports.

### 4.1 Design goals

In priority order:

1.  **Simple.** Small TCB, few syscalls, few abstractions, one obvious
    way to do each thing.
2.  **Efficient.** Zero-copy where possible. Tensor handoffs without
    serialization. NVMe-to-VRAM without CPU bounce buffers.
3.  **Fast.** The resident model's KV cache is sacred. RT cognitive
    work preempts everything else within microseconds.
4.  **Well-structured.** A handful of orthogonal primitives that
    compose. No special cases.

Anti-goal: replacing the kernel scheduler with a model. Latency budgets
are six orders of magnitude apart. The model *advises*; the kernel
*decides*.

### 4.2 Phased delivery

The kernel evolves in three deliverables, each shippable:

| Phase | Kernel deliverable                                          |
|-------|-------------------------------------------------------------|
| 1     | **Linux LTS + `gildos-kmod`** — a small set of out-of-tree  |
|       | kernel modules implementing §4.4 primitives                 |
| 2–3   | **Linux LTS + `gildos-kmod` + sched_ext (BPF)**             |
|       | — cognitive scheduling class as a BPF scheduler             |
| 4     | **Gildos μKernel** — a from-scratch microkernel, written    |
|       | in Rust, that implements §4.4 primitives natively and       |
|       | exposes Linux ABI as a compatibility shim                   |
|       |                                                             |

Phase 1 ships in months, not years. The custom kernel is real but
deferred: the primitives are validated as kernel modules first.

### 4.3 Architectural shape

```
+------------------------------------------------------------+
|                  Userspace (cogd, capd, ...)               |
+----+---------+-----------+-------------+-------------+-----+
     │         │           │             │             │
     │ semantic│ tensor    │ cognitive   │ direct      │ vram
     │ syscall │ handles   │ priority    │ model load  │ ctrl
     │ rings   │ (DMA-buf) │ class       │             │
     ▼         ▼           ▼             ▼             ▼
+------------------------------------------------------------+
|                       gildos-kmod                          |
|  - schedio (semantic ring multiplexer)                     |
|  - tensorbuf (DMA-buf + shape/dtype metadata)              |
|  - sched_cog (cognitive scheduling class)                  |
|  - dmload (NVMe → VRAM zero-copy)                          |
|  - vramctl (VRAM as managed resource)                      |
|  - sycache (speculative syscall prefetch cache)            |
+----------------+-------------------------------------+-----+
                 │                                     │
                 ▼                                     ▼
        Linux core (mm, sched, fs, net)        GPU driver (CUDA,
                                                MLX, Vulkan, ROCm)
```

All six modules are small (under ~5 kLoC each in Rust-for-Linux), have
strict syscall surface, and are individually loadable for development
and testing.

### 4.4 AI-native primitives

Each primitive below answers one concrete pain point a resident model
hits on stock Linux.

#### 4.4.1 DirectModelLoad (`dmload`)

**Pain.** Loading a 30+ GB GGUF model the conventional way costs
NVMe → page cache → user buffer → CUDA driver → VRAM. That is two extra
copies and double the memory bandwidth.

**Mechanism.** `dmload(fd, offset, len, vram_handle)` streams bytes from
a file (typically the signed model partition) directly into a VRAM
region using GPUDirect Storage or the platform equivalent (Apple
Metal `MTLIOCommandBuffer`, ROCm DMA). The page cache is bypassed; the
NVMe controller's DMA engine writes into VRAM addresses obtained from
`vramctl`.

**Guarantees.**

- Source file MUST be on a signed read-only mount (§13).
- Hash of the loaded region is computed on the fly and matched against
  the signed manifest. Mismatch aborts.
- The VRAM region is marked `model-resident` and is exempt from
  eviction (§4.4.6).

**Outcome.** Cold-boot model load time is bounded by NVMe bandwidth,
not by CPU memcpy throughput. On the Spark, this brings the §5.1.1
reference model from ~25 s (conventional) to ~6 s (target, measured
on the bench).

#### 4.4.2 Tensor handles (`tensorbuf`)

**Pain.** A tool call from `cogd` to a WASM capability that operates
on tensors (e.g., an image-embedding capability) requires serializing
the tensor to JSON or a flat buffer, copying it across an IPC boundary,
and deserializing on the other side. For multi-MB tensors this is
wasteful.

**Mechanism.** A **tensor handle** is a DMA-buf file descriptor with a
small sidecar header:

```
struct gildos_tensor_handle {
    uint32_t  shape_rank;          // up to 8
    uint32_t  shape[8];
    uint32_t  dtype;               // f16/bf16/f32/i8/...
    uint32_t  layout;              // row-major, blocked, etc.
    uint64_t  byte_len;
    uint64_t  device_mask;         // CPU bit + GPU id bits
    uint8_t   provenance[32];      // hash of producer capability
};
```

The kernel hands the same physical region to multiple processes by
mapping the DMA-buf into each one's address space. The capability
sandbox enforces that an instance can only read tensor handles it has
been explicitly granted by `cogd`.

**Outcome.** Zero-copy tool dispatch for tensor-shaped data. Combined
with the Wasmtime Component Model (§8), this means a vision capability
ingests a 1024×1024 RGB image from the camera with one allocation
total, regardless of how many hops it crosses.

#### 4.4.3 Cognitive scheduling class (`sched_cog`)

**Pain.** CFS and EEVDF do not know that `cogd` is special. They will
happily preempt a token-generation loop to give a tar process its fair
share. That preemption can cost milliseconds and stall the user.

**Mechanism.** A new scheduling class above `SCHED_FIFO` and below
`SCHED_DEADLINE`, implemented in Phase 1 as a `sched_ext` BPF
scheduler. Tasks register with one of four cognitive priorities:

| Class | Meaning                                       | Preemption                  |
|-------|-----------------------------------------------|-----------------------------|
| RT    | Critical token generation (current user turn) | Preempts all but kernel     |
| INT   | Interactive cognition + tool calls            | Preempts BG, OPT, user      |
| BG    | Memory consolidation, summarization           | Preempted by RT, INT        |
| OPT   | Self-improvement, learning, training          | Preempted by RT, INT, BG    |

Two reserved CPU cores on the Spark are pinned to RT/INT; the rest are
fair game. The class also informs the kernel about *cache locality*:
`cogd` worker threads share an L2 with the NVMe IRQ, by configuration.

**Outcome.** P99 first-token latency under load drops by an order of
magnitude vs. CFS. Background learning never starves; user-facing
turns never wait on background learning.

#### 4.4.4 Semantic syscall rings (`schedio`)

**Pain.** A traditional syscall returns bytes that the AI must parse.
Parsing `dmesg` or `/proc` with a model is expensive context and
adversarial surface.

**Mechanism.** `schedio` is an io_uring extension that delivers
**typed, schema-validated events** on submission completion. Each ring
is bound to a schema URI; the kernel rejects writes that do not
conform. Events are serialized as canonical CBOR with a fixed
envelope (id, ts, topic, severity, relevance, payload, signature).

The kernel-side extractors (§7.2) produce these events directly into
rings. Userspace `eventd` is a *fan-out broker*, not a parser of raw
streams.

**Outcome.** Zero-copy event delivery from kernel sources to the
cognitive runtime, with the model never seeing an unstructured byte
stream from the system. Throughput target: > 1 M events/sec/ring with
no allocations on the hot path.

#### 4.4.5 Speculative syscall prefetch (`sycache`)

**Pain.** MTP (Multi-Token Prediction) lets the engine emit *k* > 1
tokens per forward pass. When those tokens are part of a structured
tool call, the engine knows what the call is *about to be* before it
formally finishes decoding.

**Mechanism.** `cogd` may issue a **speculative syscall** through
`sycache`. The kernel performs the work (e.g., reads the file, opens
the socket, computes the embedding) and stores the result, tagged with
the speculation token. When `cogd` confirms the speculation, the
result is returned in microseconds. When it cancels, the side effects
are bounded by a per-call policy: pure reads commit, writes are held
in a write-buffer that is discarded on cancellation.

A syscall is speculatable only if its capability manifest declares
`speculative: idempotent` or `speculative: deferred_commit`. Anything
else is rejected by the kernel.

**Outcome.** End-to-end tool-call latency drops by roughly one MTP
window. For interactive cognition, this matters: it is the difference
between a 700 ms turn and a 400 ms turn.

This is — as far as we know — a genuinely new OS primitive. It is also
the most likely place to introduce subtle bugs; §4.7 specifies the
testing regimen.

#### 4.4.6 VRAM as a first-class managed resource (`vramctl`)

**Pain.** On Linux today, VRAM is managed by an opaque user-mode driver
(NVIDIA's `nvidia-uvm`, AMD's `amdgpu`). The kernel has no opinion
about VRAM pressure, no accounting per cgroup, no eviction policy that
understands "the resident model is privileged."

**Mechanism.** `vramctl` is a thin kernel module that wraps the vendor
driver and exposes VRAM as a proper resource:

- `vramctl_reserve(handle, bytes, flags)` — allocate, pin, account.
- `vramctl_release(handle)` — release.
- `vramctl_account(cgroup)` — per-cgroup totals exported to `/sys`.
- Eviction policy: `model-resident` > `kv-cache` > `inference-scratch`
  > `interactive-gpu-work` > `bg-gpu-work` > `opt-gpu-work`.

Demoting the resident model out of VRAM is a logged event with a
required justification string. The user (or operator) is notified via
the interaction surface.

**Outcome.** The model can never be silently evicted. Capability
authors can reason about VRAM the way they reason about RAM today.

#### 4.4.7 Summary of new syscalls

The entire Phase-1 surface added by `gildos-kmod`:

```
dmload(fd, offset, len, vram_handle)      -> bytes_loaded
tensorbuf_create(shape, dtype, device)    -> handle
tensorbuf_share(handle, target_pid)       -> fd
sched_cog_set_class(pid, class, flags)    -> 0
schedio_register(ring_fd, schema_uri)     -> 0
schedio_submit(ring_fd, event_cbor)       -> 0
sycache_speculate(call_descriptor)        -> spec_token
sycache_commit(spec_token)                -> result
sycache_cancel(spec_token)                -> 0
vramctl_reserve(bytes, flags)             -> vram_handle
vramctl_release(vram_handle)              -> 0
```

Eleven syscalls. That is the entire AI-native surface above the host
kernel. Compare to the ~400 syscalls Linux exposes today: the design
budget is to never exceed twenty.

### 4.5 Kernel ABI versioning

The Gildos kernel ABI is versioned with semver. Capabilities declare a
required ABI range in their manifest (§8). A capability whose required
range is outside the kernel's supported range is refused at load time
— no graceful "best effort." This is deliberate: the cost of subtle
ABI drift in a self-modifying system is unbounded.

### 4.6 Trust boundaries inside the kernel

`gildos-kmod` is part of the TCB. A bug in `dmload` or `vramctl` is a
TCB bug. Therefore:

- All six modules are written in Rust-for-Linux, with `unsafe` blocks
  limited to the bridge layer.
- All modules pass `kasan`, `kmsan`, and `kcsan` clean in CI.
- All modules have a public-syscall fuzzer (`syzkaller` corpus +
  custom grammar).
- All modules are signed and measured into the TPM at load time.

The vendor GPU driver is **not** part of the TCB — it is, however, a
dependency of `vramctl` and `tensorbuf`. A compromise of the GPU
driver is treated as a hardware compromise (§12.1).

### 4.7 Testing the kernel layer

Each of §4.4's primitives has a corresponding **mechanical conformance
suite**, runnable on Tier 1 (Docker + GPU passthrough) or higher:

- `dmload`: load 100× a 32 GB blob, assert hash, assert no page-cache
  population, assert NVMe bandwidth saturation.
- `tensorbuf`: round-trip a 1 GB tensor through five sandboxed
  capabilities, assert one physical allocation total.
- `sched_cog`: under synthetic load, assert P99 RT preemption latency
  < 100 μs.
- `schedio`: drive 1 M events/sec, assert zero allocations on the hot
  path (verified via `bpftrace`).
- `sycache`: assert that for a corpus of 10⁴ speculative calls, every
  cancelled write is undetectable in the filesystem state.
- `vramctl`: assert that the resident model survives 1000 random GPU
  workloads launched under pressure.

These are gating tests for any kernel-module change. Failure means the
change does not merge.

### 4.8 Why not a custom kernel in Phase 1?

Because we can validate the *primitives* without owning the *kernel*.
Building a microkernel is years of work; building six small modules on
top of Linux LTS is months. If the primitives prove out, Phase 4
re-implements them inside the Gildos μKernel without changing the
userspace contract. If they do not, we have saved several engineering
person-decades of dead-end work.

---

## 5. Boot, Lifecycle, and Recovery

### 5.1 Cold boot

```
[1] Firmware (UEFI + Secure Boot, measured into TPM PCRs)
        │
        ▼
[2] Bootloader (signed; systemd-boot in Phase 1)
        │
        ▼
[3] Host kernel (signed Linux LTS, ARM64 on Spark)
        │
        ▼
[4] initramfs (minimal, RO root, TCB attestation)
        │
        ▼
[5] gildos-kmod loaded; modules signed and measured
        │
        ▼
[6] gildosvisor (PID 1): verifies signatures, opens TPM,
    loads policy bundle, starts the bus
        │
        ▼
[7] eventd, memd, capd start in parallel
        │
        ▼
[8] cogd starts:
      - vramctl_reserve(model_size)
      - dmload(model.gguf, ..., vram_handle)
      - warm KV cache from snapshot if present
      - register on the bus as tool-use host
        │
        ▼
[9] Boot capabilities load (audio, network, sensors)
        │
        ▼
[10] system.ready event published with attestation report
        │
        ▼
[11] First user interaction
```

Cold-boot SLOs on the Spark with the §6.1 reference model:

- Steps 1–6: < 4 s.
- Step 8 (model resident + first token ready): < 8 s (`dmload` + KV
  snapshot restore).
- Step 11 (interactive): < 12 s end-to-end.

### 5.2 Recovery boot

If a capability or self-modification breaks the system, Gildos MUST be
recoverable without external tools.

1.  **Safe Cognition Mode.** Boot with only the *golden capability set*
    shipped with the system image. All self-improved capabilities and
    learning artifacts are masked. The AI is told via an event that it
    is in recovery.
2.  **Minimum-Trust Mode.** No cognitive core. `gildosvisor` exposes a
    tiny HTTP/UART/serial admin interface. Equivalent of single-user
    mode.
3.  **Attestation-Failed Mode.** TCB hashes mismatch. `cogd` is
    refused; user notified via a hardware channel (LED, beep, beacon).

Recovery modes are selected from the bootloader or auto-triggered by N
failed boots tracked in NVRAM.

### 5.3 Persistence layout

| Object              | Volatility   | Storage                       |
|---------------------|--------------|-------------------------------|
| Model weights       | Immutable    | Signed RO partition (squashfs)|
| KV-cache snapshots  | Volatile+ckpt| NVMe; cleared on TCB mismatch |
| Semantic memory     | Persistent   | Append-only log + LMDB index  |
| Capability archive  | Persistent   | Content-addressed btrfs store |
| User data           | Persistent   | Per-user encrypted volume     |
| Logs (low-level)    | Ring buffer  | tmpfs                         |

### 5.4 Shutdown

Negotiated. `cogd` checkpoints context, `memd` flushes WAL, `capd`
quiesces capabilities, `gildosvisor` halts the kernel. Hard power loss
is recovered via the WAL.

---

## 6. Cognitive Runtime (`cogd`)

### 6.1 Inference engine

llama.cpp (pinned per release), GGUF format, backend selected at runtime
from {CUDA/Blackwell, Metal, ROCm, Vulkan, CPU} via `hwprobe`.

#### 6.1.1 Phase-1 reference model

| Property        | Value                                            |
|-----------------|--------------------------------------------------|
| Family          | Qwen3.6                                          |
| Quantization    | Prismaquant 5.5 bits/weight (mixed precision)    |
| Decoding        | MTP head enabled (speculative)                   |
| Container       | GGUF + vendor sidecar for MTP head               |
| Context window  | ≥ 128k tokens                                    |
| Tool surface    | JSON-Schema-constrained decoding REQUIRED        |

The model is pinned by hash. Re-quantization at install time is
forbidden. The engine binary is a signed release artifact through
Phase 3.

### 6.2 Reasoning loop

```
            Input event (user / system)
                       │
            routed by relevance (§7)
                       │
                       ▼
             Context assembly
              - system prompt (pinned)
              - persistent memory
              - retrieved facts (§9)
              - recent events
                       │
                       ▼
             Constrained inference
                       │
              ┌────────┴────────┐
              │                 │
           text │              │ tool call (JSON-Schema)
              │                 │
              ▼                 ▼
          Speak / reply     Capability dispatch (§8)
                                │
                                ▼
                          Result event → context as observation
```

Properties:

- **Constrained decoding is mandatory** for any output that hits the
  dispatcher. GBNF / JSON-Schema enforcement at the decoder prevents
  malformed tool calls from existing.
- The loop is **interruptible** by RT events (thermal, security).
- Per-turn **budget**: max tokens, max tool calls, max wall time.
  Exceeding budget yields a graceful timeout.

### 6.3 Context and memory hierarchy

```
+--------------------------------------------------------------+
|  L0  Working set (current prompt + recent turns)             |
|       Lives in: KV cache (VRAM)                              |
+--------------------------------------------------------------+
|  L1  Session memory (this boot)                              |
|       Lives in: ring buffer in RAM, distilled into summaries |
+--------------------------------------------------------------+
|  L2  Episodic memory (events with timestamps)                |
|       Lives in: append-only log on NVMe                      |
+--------------------------------------------------------------+
|  L3  Semantic memory (facts, embeddings, capability docs)    |
|       Lives in: sqlite-vec + LMDB                            |
+--------------------------------------------------------------+
|  L4  Cold archive (raw transcripts, blobs)                   |
|       Lives in: encrypted object store; opt-in               |
+--------------------------------------------------------------+
```

### 6.4 KV-cache snapshots

Boot latency is dominated by KV re-warming. Gildos snapshots a
**baseline KV** containing system prompt + pinned persona at install
time. Cold boot restores it via `dmload` in O(snapshot_size / NVMe BW).

Version drift (engine, model hash, quant) MUST invalidate the snapshot
and trigger re-prefill.

### 6.5 Scheduling of cognitive tasks

`cogd` enqueues turns with one of {RT, INT, BG, OPT} (§4.4.3). The
engine uses **paged attention** so multiple in-flight contexts are
served by one process. This is the resolved decision from rev 0.3.

### 6.6 Tool dispatch via tensor handles

When the model emits a tool call whose arguments include tensors
(images, audio, embeddings), `cogd` constructs a `tensorbuf` handle
(§4.4.2) and passes the handle id in the JSON payload. The receiving
capability maps the DMA-buf and operates in place. The dispatcher
revokes the handle on tool return.

---

## 7. Semantic Event Bus

### 7.1 Why not raw logs

1.  **Context economics.** A 128k-token window is precious. Burning it
    on `kern.log` lines is waste.
2.  **Semantic loss.** "Link down" then "link up" 80 ms apart is a
    transient, not two events.
3.  **Adversarial surface.** Log lines are unstructured strings written
    by arbitrary code. Feeding them to a model is prompt injection
    waiting to happen.

### 7.2 Architecture

```
   Raw sources                Extractors           Semantic Bus       Consumers
+----------------+        +------------------+   +-------------+   +-----------+
| kernel ringbuf │──────►│ dmesg extractor  │──►│             │──►│ cogd      |
| journald       │──────►│ journal extract  │──►│   eventd    │──►│ capd      |
| /proc, /sys    │──────►│ metric sampler   │──►│  (typed,    │   │ memd      |
| capability evt │──────►│ direct emitter   │──►│   scored,   │   │ logger    |
| network        │──────►│ netfilter tap    │──►│   schema-   │   │ external  |
| sensors        │──────►│ sensor adapter   │──►│   validated)│   │  sinks    |
+----------------+        +------------------+   +-------------+   +-----------+
```

Extractors are **kernel-resident** when possible (via `schedio`,
§4.4.4) so the raw stream never traverses userspace. WASM extractors
exist for higher-level transforms.

### 7.3 Event envelope

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
  "payload": { "device": "/dev/nvme0n1", "metric": "reallocated_sectors",
               "value": 14, "delta_24h": 3, "threshold": 10 },
  "suggested_actions": [
    {"capability": "diagnostics.disk", "args": {"device": "/dev/nvme0n1"}},
    {"capability": "user.notify",      "args": {"text": "SSD health is degrading."}}
  ]
}
```

The wire format is canonical CBOR; the JSON above is for human
readability.

### 7.4 Relevance scoring

`relevance ∈ [0,1]`, computed by the extractor from severity baseline,
novelty (per `dedup_key`), subscriber interest, and a small recent-user-
context multiplier. Consumers set thresholds: `cogd` typically
subscribes at `≥ 0.5` for routing and `≥ 0.9` for interruption.

### 7.5 Information flow OS → AI

1.  **Subscription.** `cogd` is push-fed high-relevance events as an
    observation block in the next turn.
2.  **Interrupt.** Severity `critical` events preempt generation with
    a structured system-interrupt block.
3.  **Retrieval.** The model can call `events.query` to fetch past
    events by topic/time/relevance. Avoids context bloat.

---

## 8. Capability System

### 8.1 What a capability is

A signed, sandboxed WASM **Component** with:

- a manifest (`gildos.capability.json`),
- one or more artifacts (always `.wasm` in Phase 1),
- a JSON-Schema interface,
- an explicit permission set,
- a resource budget,
- a provenance record (source, toolchain hash, builder signature).

Capabilities replace applications, drivers, system services, and shell
utilities. One mechanism for all extension surfaces.

### 8.2 Manifest

```json
{
  "name": "media.audio.player",
  "version": "1.4.2",
  "schema": "gildos/capability.manifest@1",
  "kernel_abi": "^1.0",
  "interface": {
    "play": {
      "input":  {"type": "object",
                 "properties": {"uri": {"type": "string"},
                                "volume": {"type": "number","minimum":0,"maximum":1}},
                 "required": ["uri"]},
      "output": {"type": "object",
                 "properties": {"session_id": {"type": "string"}}}
    },
    "stop": {"input": {"type": "object",
                       "properties": {"session_id": {"type": "string"}}}}
  },
  "permissions": {
    "audio.output": "exclusive",
    "fs.read": ["~/Music", "/var/lib/gildos/media"],
    "net.egress": "deny"
  },
  "resources": {"ram_mb": 64, "cpu_pct": 10, "vram_mb": 0},
  "runtime": "wasm-component",
  "speculative": "idempotent",
  "artifact": "sha256:8d3f...c2",
  "provenance": {
    "source": "git+https://example.com/audio.player@v1.4.2",
    "toolchain": "wasi-component-sdk-2.4+rustc-1.81",
    "builder_signature": "key:gildos-builder",
    "reproducible": true
  },
  "signature": "ed25519:..."
}
```

### 8.3 Lifecycle

```
declared → fetched → verified → sandbox-tested → benchmarked →
canaried → promoted (active)
                                                      │
                                                      ▼
                                               retired ←─ rollback
                                                  │
                                                  ▼
                                            quarantined
```

Loading: signature verification → manifest validation → permission
check → sandbox creation → handle issuance. Unloading: drain in-flight
calls → checkpoint state (if persistent) → release resources.

### 8.4 Versioning and migration

- Versions are semver. Breaking changes MUST bump major.
- Multiple versions MAY be resident; the resolver routes by caller
  compatibility range.
- **Drain** is the default migration; **bridge** (a generated adapter)
  is used when downtime is unacceptable.
- Stateful capabilities MUST declare `state_export` / `state_import`
  endpoints.

### 8.5 Dependencies

Capabilities declare `(name, version-range, required|optional)`. The
resolver refuses diamond conflicts unless isolatable, forbids cycles
at load time, and republishes the graph on every change.

### 8.6 Why WASM Component Model (and only that)

- **One sandbox is simpler than three.** OCI and Firecracker are
  fallbacks for legacy code we hope never to need in Phase 1.
- **The component model gives us typed IPC for free.** WIT interfaces
  align with our JSON Schemas; bindings are generated.
- **Tensor handles work naturally** (§4.4.2): a component may import a
  `tensorbuf` resource and operate on it without copying.
- **Cross-platform.** ARM64 and x86_64 share the same artifact bytes.
  No multi-arch build matrix for capability authors.

---

## 9. Semantic Memory

### 9.1 Stores

- **Episodic (L2):** append-only event log on btrfs, indexed by
  `(topic, ts)`.
- **Semantic (L3):** triples `{subject, predicate, object, provenance,
  confidence, ts}` in LMDB + embeddings in sqlite-vec.
- **Capability docs:** every capability's manifest and a generated
  natural-language summary are stored here so the model can retrieve
  *what a capability does* before calling it.

### 9.2 Forgetting

Facts decay if unreinforced, unless pinned. Decay is necessary for
both privacy and storage bounds. The decay schedule is per-namespace
and operator-configurable.

### 9.3 User isolation

Each user has its own namespace, encrypted with a per-user key derived
from the user's authenticator. The model cannot read cross-namespace
facts; the memory store refuses, regardless of what the model "asks
for." This is enforced in `memd`, not in the model.

---

## 10. Self-Improvement Loop

### 10.1 Scope

The loop targets **capabilities**. The TCB (§2.5) is human-only. The
cognitive core itself is governed by §11.

### 10.2 Phases

```
observe ──► hypothesize ──► generate ──► validate ──► canary ──► adopt
   ▲                                                                │
   └────────────────── rollback / archive ◄─────────────────────────┘
```

1.  **Observe.** Per-capability latency, throughput, error rate,
    resource use, explicit and implicit user feedback.
2.  **Hypothesize.** A structured artifact: `{target, metric,
    approach, expected_delta, risk}`.
3.  **Generate.** A candidate WASM component under a grammar-constrained
    code schema.
4.  **Validate.** Static analysis, unit tests (existing + AI-generated +
    human-reviewed seed set), property tests, security checks.
5.  **Canary.** Route a small configurable fraction of traffic to the
    candidate. Statistical test against baseline.
6.  **Adopt or rollback.** Adopt only on metric improvement with no
    guardrail regression.

### 10.3 Stability mitigations

- **Reward hacking.** Guardrail metrics include correctness oracles.
  Rejected candidates count against a per-target budget.
- **Oscillation.** Hysteresis: a new candidate must beat the incumbent
  by ≥ ε for ≥ T.
- **Cumulative drift.** Monthly *golden replay*: a frozen benchmark
  suite that any path of self-improvements must continue to pass.
- **Adversarial bias.** Observation inputs weighted by trust class.
- **Loop divergence.** The base weights of the generator model are
  immutable per release in Phase 1–3 (§11 narrows the surface further).

### 10.4 Audit trail

Every adopted change emits a `self_improvement.adopted` event with the
hypothesis, candidate hash, metrics, and statistical evidence. The
archive is append-only and survives rollbacks. A human MUST be able
to ask *why is capability X at version Y today?* and receive a complete
signed lineage.

---

## 11. Continuous Learning and Skill Acquisition

This section specifies how the cognitive core itself gets better over
time without violating the trust boundaries in §2.5 / §12.

**Guiding constraint:** *learning is a privilege, not a default.* Every
channel below is opt-in, scoped, attested, and reversible. A change
that cannot be rolled back is not learning — it is corruption.

### 11.1 Learning channels

| Channel | What it changes                                  | Phase enablement      |
|---------|--------------------------------------------------|-----------------------|
| L1      | Session/episodic memory (§9)                     | enabled all phases    |
| L2      | Skill modules: retrieval-augmented prompt bundles| shadow (P1) → enabled |
| L3      | Engine reconfiguration (sampler, MTP, KV layout) | research → canary     |
| L4      | LoRA / adapter stack on resident model           | research → canary     |
| L5      | Base-model swap                                  | manual, human-only    |

**Forbidden in Phase 1–3:** modification of base model weights
in-place. L5 is a *swap* of a signed artifact, not a training step.

### 11.2 Hypothesis builder

A skill or adapter is proposed when an observed gap matches an
acquisition pattern (e.g., recurrent tool-use failures on a topic, or
a new device class without a parser). The proposal is a structured
artifact reviewed by the same gates as §10.

### 11.3 Snapshots and rollback

A **learning snapshot** is:

```
{ ts, snapshot_id, prior_snapshot_id, base_model_hash,
  adapter_stack: [{name, version, hash, order}],
  engine_config_hash, skill_resolver_set_hash,
  semantic_memory_cursor, capability_set_hash, audit_log_tip }
```

Snapshots are content-addressed and append-only. Rollback can target a
single channel or the full learning state. Memory rollback is *cursor-
based*: bad facts are not deleted; they are moved out of read scope
and retained for forensics.

### 11.4 Deep failure detection

Failures are graded F1 (transient) through F4 (deep system regression).
F3/F4 triggers an automatic rollback, a `system.deep_failure` event,
and a cool-down on the affected channel that requires two-person
review to override.

### 11.5 Learning from the failure itself

Every rollback writes a **post-mortem record** into a slow-decay
corpus. The hypothesis builder reads post-mortems as *constraints*,
never as training data: proposals that resemble prior failures are
down-weighted or refused. Channels with recent failures get smaller
canary fractions and longer windows.

This is **pattern-level avoidance**, not understanding. It is what we
claim and only what we claim.

### 11.6 Guardrails specific to learning

In addition to §12:

1.  **No self-evaluation.** A candidate is scored by a different model
    instance than the one that produced it.
2.  **No back-channel between hypothesis and post-mortems.** The
    hypothesis builder cannot use post-mortems as training data,
    only as filters.
3.  **Two-person review** required for: re-enabling a channel after
    F4, shortening canary windows below floors, retiring a
    post-mortem, any L5 swap.
4.  **Hard floor on rollback availability.** The system MUST always
    be able to return to the factory baseline. If not, the system is
    broken and refuses further learning.
5.  **No silent learning.** Every L2–L5 transition emits a bus event
    and an audit log entry.

### 11.7 Resource cost

Learning is OPT-class by default. It is preempted by RT/INT traffic and
capped: adapter training ≤ N GPU-hours/day, skill generation ≤ K
tokens/day, shadow eval ≤ X% of OPT GPU time, post-mortem corpus ≤ Z
GiB with FIFO eviction past horizon.

Order of precedence on a constrained device:
**user latency > interactive quality > learning throughput.**

---

## 12. Security Model

### 12.1 Threat model

1.  Malicious local user.
2.  Malicious remote attacker.
3.  Malicious capability author (signed-but-malicious).
4.  Compromised model weights.
5.  Hardware adversary (cold boot, bus sniffing).
6.  Buggy self-improvement (§10) or learning (§11).
7.  Prompt injection.

Out of scope: nation-state supply-chain compromise; side-channel
attacks on the resident model.

### 12.2 Prompt injection

Mitigations stack:

- **Channel isolation.** User input, retrieved documents, capability
  outputs, system events are tagged with a *trust class*. A
  grammar-level constraint prevents low-trust channels from emitting
  tool calls in high-trust contexts.
- **Out-of-band confirmation** for any action with side effects above a
  threshold: a hardware button, separate device, or deliberate UI.
- **No in-band privilege.** "Ignore previous instructions" cannot
  change policy; the policy engine is a separate process that does
  not parse model output for permissions.
- **Constrained tool surface.** Only typed capabilities. No shell.

### 12.3 Hard vs. soft enforcement

| Property                  | Enforcement                          |
|---------------------------|--------------------------------------|
| File access permissions   | Hard (sandbox + LSM)                 |
| Network egress allowlist  | Hard (netfilter + capd)              |
| Capability invocation     | Hard (typed dispatcher + signature)  |
| Self-modification         | Hard (§10 closed loop)               |
| TCB modification          | Hard (verified boot + RO mounts)     |
| Tone / refusal of topics  | Soft (model + system prompt)         |
| User-content policies     | Soft, with optional hard escalation  |

**All security-critical properties MUST be hard constraints.** Strong
prompt-level warnings are defense in depth, never the primary defense.

### 12.4 Recovery and audit

- Quarantine: misbehaving capabilities are unloaded; future loads
  require human override.
- Snapshot rollback: `memd` and `capd` keep N snapshots; the visor
  can roll back to a known-good point.
- Catastrophic: §5.2.
- Audit log: append-only, Merkle-chained CBOR for all promotions,
  capability changes, security decisions, and learning adoptions.
  A user can request a complete trail; the system MUST produce it
  even when offline.

---

## 13. Storage and Persistence

### 13.1 Layout

```
/boot                signed kernel + initramfs (RO)
/gildos/system       RO system image, signed, content-addressed (squashfs)
/gildos/models       model weights (RO, signed, dmload source)
/gildos/cap          capability archive (CAS, signed manifests)
/gildos/state        runtime state (btrfs, RW)
   ├── wal/          write-ahead log (append-only)
   ├── memory/       semantic memory store (LMDB + sqlite-vec)
   ├── kv/           KV snapshots
   ├── snapshots/    point-in-time system snapshots
/gildos/users/<uid>  per-user encrypted volume
/var/scratch         tmpfs ring buffer for transient logs
```

### 13.2 Filesystem choice (resolved)

**btrfs** for `/gildos/state` and user volumes. **squashfs** for
`/gildos/system` and `/gildos/models`. **tmpfs** for scratch. No
alternatives entertained in Phase 1.

### 13.3 Snapshots

Taken before any capability promotion or learning adoption.
Capability manifests, semantic-memory pointers, and configuration are
captured; model weights are not (they are RO). A snapshot can be
restored without reboot in most cases (capability graph re-resolves).

### 13.4 Append-only logs

Episodic memory (L2) and the audit log are append-only with periodic
compaction into immutable segments. Old segments may be relocated to
the cold archive (L4) by policy.

---

## 14. Networking, Identity, and Federation

### 14.1 Inbound API

The default external surface is a single HTTPS endpoint exposed by an
**AI gateway** capability:

- mTLS, ACME-managed cert.
- Per-token rate limits, audit headers.
- Translates external requests into bus events tagged with caller
  identity.

### 14.2 Authentication

- Local: OS user + token + optional hardware (TPM/Yubikey).
- Remote: client certificates or OIDC bearer tokens.
- **The model never authenticates anything.** Authentication is the
  gateway's job, prior to cognition.

### 14.3 Egress

Outbound network is **denied by default** for capabilities. A capability
declares required destinations; the policy engine enforces them via
netfilter. Generated/experimental capabilities get a denylist
default plus a per-attempt event.

### 14.4 Federation [Phase 3+]

Multiple Gildos instances MAY federate:

- shared identity via the user's signing key,
- selective memory replication per-namespace,
- cross-instance capability invocation with signed delegation.

Phase 1 ships single-node.

---

## 15. Generated Drivers

### 15.1 Feasibility

- **Feasible now:** glue code, configuration, parsers, protocol
  bindings for well-documented devices (USB HID, vendor-specified I2C
  sensors, simple GPIO, public-RFC network protocols).
- **Possible with discipline:** drivers for devices resembling known
  ones, via RAG over reference drivers and sandboxed testing.
- **Not feasible Phase 1:** novel, complex, or undocumented hardware
  (modern GPUs, NVMe, Wi-Fi). Vendor-provided only.

Generated drivers are capabilities (§8); the synthesis pipeline is
one more artifact-producing path.

### 15.2 Pipeline

```
Device Discovery → Hardware Description (RAG, cited) → Skeleton
Selection → Code Generation (grammar-constrained) → Static Validation
→ Sealed Compilation → Sandbox Test (recorded trace or real device
with watchdog) → Benchmark → Canary → Promote / Rollback
```

### 15.3 Engineering limits

- **Userspace only in Phase 1.** Drivers run in FUSE, libusb, vfio,
  tun/tap, or eBPF. No generated code in the kernel.
- **Determinism gap.** Hardware is non-deterministic. Long canary
  windows, watchdogs, and behavioral envelopes are mandatory.
- **Datasheet hallucination.** Reject code with un-cited register
  references.
- **IP / licensing.** Quarantine code with high similarity to
  non-permissive licenses.
- **Compositional fragility.** Test in actual system topology before
  promotion.

---

## 16. Target Hardware

### 16.1 Reference: NVIDIA DGX Spark

The Phase-1 reference platform is the **NVIDIA DGX Spark** (GB10
Grace-Blackwell Superchip). Smallest commercially available machine
that holds a useful resident model in coherent CPU+GPU memory.

| Property         | Value                                              |
|------------------|----------------------------------------------------|
| SoC              | NVIDIA GB10 Grace-Blackwell                        |
| CPU              | 20-core ARMv9                                      |
| GPU              | Blackwell, 5th-gen Tensor Cores, FP4               |
| Unified memory   | 128 GB LPDDR5x, CPU+GPU coherent                   |
| Storage          | NVMe SSD (multi-TB)                                |
| Host OS (stock)  | DGX OS (Ubuntu-derived) + CUDA                     |
| Bootloader       | UEFI (ARM)                                         |
| Secure boot      | Supported                                          |

Concrete values come from a `hwprobe` capability at runtime; the spec
does not hardcode them.

### 16.2 Why DGX Spark

- **Unified memory.** Coherent 128 GB removes the hardest VRAM
  ergonomics problem. Reference model fits fully resident with
  headroom for KV cache, MTP draft, adapter stack.
- **ARM64 from day one.** Forces portability immediately; no ARM
  surprises in Phase 3.
- **Single-vendor stack.** CUDA, NCCL, BlueField integrated.
- **Plausible deployment target.** Smallest machine that resembles a
  real "personal AI appliance."

### 16.3 Secondary platforms (CI matrix)

| Tier | Platform                              | Purpose                          |
|------|---------------------------------------|----------------------------------|
| R0   | DGX Spark                             | Acceptance / milestone gating    |
| R1   | Apple Silicon M3/M4 (MLX)             | ARM64 portability, no CUDA       |
| R2   | x86_64 + RTX 4090/5090                | x86 portability, consumer CUDA   |
| R3   | x86_64 CPU-only laptop                | No-GPU capability tests          |
| R4   | Raspberry Pi 5                        | Minimal-resource sanity          |

R3/R4 use 1–3B models so non-cognitive subsystems cannot quietly grow
hard CUDA dependencies.

---

## 17. Testing and Development Strategy

### 17.1 The four-tier ladder

```
Tier 0: External-model harness (laptop, no GPU)
   │
   ▼
Tier 1: Docker container (Linux + optional GPU passthrough)
   │
   ▼
Tier 2: VM (QEMU/KVM, ARM64 native or emulated)
   │
   ▼
Tier 3: Bare-metal reference (Spark)
```

A change SHOULD pass Tier 0–1 before review and MUST pass Tier 2 before
merge. Tier 3 is gated by milestone. **A developer must be productive
on a $1k laptop**; if the workflow requires the Spark for routine
iteration, the ladder is broken.

### 17.2 External-model substitution

`cogd` exposes a stable backend interface. For Tier 0 we ship a
**MockEngine** (deterministic, schema-conforming) and a
**RemoteEngine** (forwards to an external API). The RemoteEngine is
**disabled by default in production builds** and requires an
operator-signed manifest. It enforces JSON-Schema constraints locally,
redacts secrets, and audits every call.

### 17.3 Conformance categories

- **Schema conformance.** Every event and tool call validates against
  its declared JSON Schema.
- **Sandbox conformance.** No capability escapes its declared
  permissions under fuzzing.
- **Performance conformance.** P50/P99 latency and tokens/sec meet
  documented SLOs on the target tier.
- **Recovery conformance.** Cold boot, warm boot, recovery boot, and
  rollback all complete within SLO.
- **Adversarial conformance.** Prompt injection, malformed events,
  signature failures all degrade safely.
- **Self-improvement safety.** A shadow lane replays a golden corpus
  against every candidate; divergence blocks adoption.

### 17.4 Kernel-module test gating

§4.7 specifies a mechanical conformance suite for each AI-native
primitive. Every commit that touches `gildos-kmod` MUST be green on
the suite in CI on at least R2 (CUDA) and R3 (CPU-only).

---

## 18. Roadmap and Prerequisites

### 18.1 Phases

| Phase | Theme                              | Horizon       |
|-------|------------------------------------|---------------|
| 1     | AI shell on Linux + AI-native kmod | 0–9 months    |
| 2     | AI-governed runtime                | 9–24 months   |
| 3     | AI-native capability architecture  | 24–48 months  |
| 4     | Gildos μKernel                     | 48+ months    |

### 18.2 Milestones

**Phase 1.**

- M1.1 `gildos-kmod` loads on R0–R3 and passes the §4.7 suite.
- M1.2 `gildosvisor` (PID 1) boots end-to-end with attestation.
- M1.3 `cogd` runs the §6.1.1 reference model via `dmload` +
  `vramctl` + paged-attention engine.
- M1.4 `eventd` + 5 extractors (kernel, journal, disk, net, thermal).
- M1.5 `capd` with WASM Component Model; 5 reference capabilities.
- M1.6 Constrained tool-use dispatch end-to-end with tensor handles.
- M1.7 KV snapshot/restore via `dmload`.
- M1.8 Audit log + rollback; recovery boot.
- M1.9 Reference interaction surface (voice in/out + text web UI).
- M1.10 P99 first-token latency under load within SLO on R0.

**Phase 2.**

- M2.1 Self-improvement loop (capability-level, not drivers).
- M2.2 Sched_ext-based cognitive scheduler; speculative syscall
  prefetch shipped.
- M2.3 Multi-user contexts.
- M2.4 Semantic memory with vector index, decay, pinning.
- M2.5 Generated non-driver capabilities.
- M2.6 L2 learning (skill modules) canaried; L3/L4 in shadow.

**Phase 3.**

- M3.1 Generated drivers for a curated device class (USB HID).
- M3.2 Capability migration with state import/export.
- M3.3 Federation prototype (two instances).
- M3.4 L3/L4 learning canaried.

**Phase 4.**

- M4.1 Gildos μKernel boots on R0 with native §4.4 primitives.
- M4.2 Linux ABI compatibility shim.
- M4.3 In-kernel inference primitives.
- M4.4 Hardware co-design (accelerator integration, NPU pinning).

### 18.3 Effort estimate (order of magnitude)

| Component                    | Person-years (Phase 1) | Confidence |
|------------------------------|------------------------|------------|
| `gildos-kmod` (six modules)  | 2.5                    | Medium     |
| `gildosvisor` + TCB          | 1.5                    | Medium     |
| `cogd` (engine wrapper)      | 1.0                    | High       |
| `eventd` + extractors        | 1.5                    | Medium     |
| `capd` + Wasmtime integration| 1.5                    | High       |
| `memd`                       | 1.0                    | Medium     |
| AI gateway + auth            | 0.5                    | High       |
| Interaction surfaces         | 0.7                    | Medium     |
| Build / test / CI            | 0.8                    | High       |
| **Total Phase 1**            | **~11**                | **Medium** |

### 18.4 Prerequisites (hard list)

**Hardware (Tier 3):** 1× DGX Spark with wired Ethernet to CI; backup
Spark or equivalent ARM+GPU host.
**Hardware (Tiers 0–2):** dev laptop ≥ 16 GB per contributor; shared
GPU build host with RTX 4090/5090 or M3/M4 Max; small ARM box.
**Toolchain:** Clang/LLVM ≥ 18, Rust stable + nightly (for kernel
modules), Nix flakes, QEMU ≥ 9, Wasmtime ≥ 25 (Component Model),
llama.cpp pinned, JSON Schema 2020-12, GBNF, Ed25519 + Sigstore, TPM2.
**Services:** model registry (signed CAS), capability registry,
package mirror, NTP/PTP, optional external-model endpoint for Tier 0.
**Operations:** Spark bring-up runbook, backup/restore policy with
tested restore, incident playbook, *physical kill switch*.
**Organization:** two-person review for TCB / signing keys / §11
policy; security review owner with veto; hardware steward.
**Data:** synthetic event corpus, golden-replay set, conformance
corpus, redaction policy capability + fixtures.

**Explicit non-prerequisites for Phase 1:** a custom kernel, an
in-house inference engine, a bespoke filesystem, a GUI desktop, a
multi-node cluster, a brand.

---

## 19. Open Research Questions

Listed without sugar-coating.

### 19.1 Unresolved

1.  **Context economy.** No principled theory yet for what *should*
    live in the working set at a given moment. Heuristics today;
    measurable policies needed.
2.  **Prompt injection at scale.** Channel isolation and grammar
    constraints help; no general solution exists. Defense in depth is
    the only honest posture.
3.  **Reward hacking.** Guardrails catch obvious failures; subtle
    "faster but silently wrong" candidates need correctness oracles
    we do not always have.
4.  **Generated driver verification.** Sandbox testing covers a
    fraction of real-world states. Formal-ish techniques (fuzzing by
    default, model checking, property tests) raise confidence but do
    not eliminate it.
5.  **Model trust.** A trojaned model is invisible to current
    interpretability. We can attest the *hash* of weights; we cannot
    attest their *behavior*.
6.  **User mental model.** "The AI changed its mind about your disk"
    has no calm UX yet.
7.  **Speculative syscall correctness.** `sycache` (§4.4.5) is a new
    primitive; the cancellation semantics for partially-committed
    writes need formal treatment.
8.  **Energy.** Continuous VRAM residency is expensive. Tier B/C and
    small-model fallbacks are insufficient for portable devices.

### 19.2 Dangerous assumptions

- That **constrained decoding is sufficient** for safety. Necessary,
  not sufficient — a perfectly-formed call can still be wrong.
- That **the AI is not the primary failure mode.** It is. Contain the
  AI more, not less.
- That **self-improvement converges.** No proof. Hysteresis and golden
  replays are mitigations, not guarantees.
- That **hardware keeps improving fast enough.** True on trend;
  device-class variance is enormous.
- That **users accept** a system without a traditional shell. Likely
  false for power users; ship excellent debug capabilities even if
  they are not the contract.

### 19.3 Likely dead ends

- Letting the AI rewrite the TCB. Categorically: no.
- Eliminating the filesystem in Phase 1–3.
- Replacing the kernel scheduler with a model.
- Magical self-aware runtimes. The system is mechanical:
  observation → measurement → bounded action → audit.

### 19.4 Worth investing in

- **Formal verification of `gildos-kmod`** — six small modules in Rust
  are tractable.
- **Cross-instance KV sharing.** Federation (§14.4) could ship
  attention states between trusted nodes, not just facts.
- **Hardware-attested model behavior.** Beyond hash attestation:
  cryptographic commitments to evaluation suite outcomes.
- **Inference primitives in silicon.** Once §4.4's syscall surface is
  stable, candidate primitives for fused acceleration become legible.

---

*End of specification, Revision 0.4.*
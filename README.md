# Gildos

An experimental AI-native operating system.

Gildos is a research effort to invert the conventional OS stack: instead
of AI being an application running on top of an OS, the OS is organized
around a continuously-resident inference engine as its primary cognitive
layer. Programs become *capabilities* — versioned, signed, sandboxed
units that the cognitive core can load, replace, generate, and retire
under a closed loop of *propose → compile → sandbox → benchmark →
canary → adopt → rollback*.

This repository currently contains the engineering specification only.
There is no code yet — the project starts with rigorous design.

## Documents

- [`SPECIFICATION.md`](./SPECIFICATION.md) — Revision **0.6**. A
  re-integration pass: every section that was previously appended in
  rev 0.5's "Addendum" trailer is now woven into its logical position
  in the body of the document. No new content; structurally clean.
  Covers nine AI-native kernel primitives, the 90-day MVG, end-to-end
  user-turn walkthrough, and a glossary.

## Nine AI-native kernel primitives (§4.4)

| # | Primitive                          | What it fixes                                  |
|---|------------------------------------|-----------------------------------------------|
| 1 | DirectModelLoad (`dmload`)         | NVMe → VRAM zero-copy weight streaming        |
| 2 | Tensor handles (`tensorbuf`)       | Zero-copy tool dispatch for tensor-shaped IO  |
| 3 | Cognitive scheduling (`sched_cog`) | Protect the model from CFS preemption         |
| 4 | Semantic syscall rings (`schedio`) | Typed events, never raw bytes to the model    |
| 5 | Speculative prefetch (`sycache`)   | Pre-execute tool calls during MTP decoding    |
| 6 | VRAM as resource (`vramctl`)       | Per-cgroup VRAM accounting + eviction policy  |
| 7 | Cognitive Fork (`kvfork`)          | Copy-on-write KV clone for tree-of-thoughts   |
| 8 | Persistent Prompts (`promptpin`)   | Shared, signed, pre-tokenized system prompts  |
| 9 | Cognitive cgroups (`cogcg`)        | Tokens/sec, context-tokens, GPU-time quotas   |

## The MVG, in one paragraph

The Minimum Viable Gildos (§1.5) is the 90-day artifact that proves the
design works: an x86_64 workstation with an RTX 4090, Linux LTS +
`gildos-kmod` (four primitives mandatory, five stubbed), `gildosvisor`
as PID 1, `cogd` with a 7–8B Qwen-class model, `eventd` with three
extractors, `capd` with three reference capabilities, `memd` on LMDB,
and a single HTTPS gateway. Demo: ask the system for the GPU
temperature and subscribe to a thermal threshold — and have the
notification fire within a second when it trips. Cold boot to first
token ≤ 20 s, P50 turn latency ≤ 600 ms, model unevictable under
concurrent GPU load, audit log exportable.

## Enforced architectural choices (§2.6)

- **Rust** above the kernel; C/C++ only for the inference engine and
  vendor drivers.
- **Nix flakes** as the build system.
- **WASM Component Model** (Wasmtime) as the sole sandbox in Phase 1.
- **btrfs + squashfs + tmpfs** as the storage stack.
- **llama.cpp** pinned per release as the inference engine.
- **Qwen3.6 Prismaquant 5.5b + MTP** as the Phase-1 reference model.
- **DGX Spark (ARM64 + Blackwell)** as the Phase-1 reference platform.
- **Paged-attention, parallel contexts** as the scheduling model.
- **`gildosvisor`** (a small Rust microvisor) as PID 1.

## Phases

| Phase | Theme                              | Horizon       |
|-------|------------------------------------|---------------|
| 1     | AI shell on Linux + AI-native kmod | 0–9 months    |
| 2     | AI-governed runtime                | 9–24 months   |
| 3     | AI-native capability architecture  | 24–48 months  |
| 4     | Gildos μKernel                     | 48+ months    |

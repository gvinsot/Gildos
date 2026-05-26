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

- [`SPECIFICATION.md`](./SPECIFICATION.md) — Revision **0.4**. A focused
  rewrite that introduces a dedicated **Gildos Kernel** section, resolves
  all prior A/B architectural alternatives, and specifies six AI-native
  kernel primitives (DirectModelLoad, tensor handles, cognitive
  scheduling class, semantic syscall rings, speculative syscall prefetch,
  VRAM as a first-class managed resource).

## Status

Exploratory. Specification draft. Not a product roadmap.

## The kernel idea, in one paragraph

Stock Linux is hostile to a resident model in subtle ways: the scheduler
does not know that swapping out the model is catastrophic; VRAM is owned
by an opaque user-mode driver; syscalls are byte-oriented and the model
wastes context parsing them; tool calls require re-serializing every
tensor; weights are double-copied through the page cache on load. Gildos
fixes each of these with a small set of new kernel primitives —
delivered in Phase 1 as Rust-for-Linux modules (`gildos-kmod`), and in
Phase 4 as a from-scratch microkernel that exposes the same userspace
contract. Eleven new syscalls total; design budget never exceeds twenty.

## Enforced architectural choices

Picked, in writing, in §2.6 of the spec:

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

| Phase | Theme                                  | Horizon       |
|-------|----------------------------------------|---------------|
| 1     | AI shell on Linux + AI-native kmod     | 0–9 months    |
| 2     | AI-governed runtime                    | 9–24 months   |
| 3     | AI-native capability architecture      | 24–48 months  |
| 4     | Gildos μKernel                         | 48+ months    |

See the specification for milestones, dependencies, prerequisites, and
open research questions per phase.
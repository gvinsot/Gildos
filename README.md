# Gildos

An experimental AI-native operating system.

Gildos is a research effort to invert the conventional OS stack: instead of
AI being an application running on top of an OS, the OS is organized
around a continuously-resident inference engine as its primary cognitive
layer. Programs become *capabilities* — versioned, signed, sandboxed
units that the cognitive core can load, replace, generate, and retire
under a closed loop of *propose → compile → sandbox → benchmark →
canary → adopt → rollback*.

This repository currently contains the engineering specification only.
There is no code yet — the project starts with rigorous design.

## Documents

- [`SPECIFICATION.md`](./SPECIFICATION.md) — Revision 0.1 engineering
  whitepaper. Covers architecture, boot, cognitive runtime, semantic
  event bus, capability system, generated drivers, self-improvement
  loop, resource governance, security model, storage, networking,
  roadmap, and open research questions.

## Status

Exploratory. Specification draft. Not a product roadmap.

## Phases

| Phase | Theme                                  | Horizon       |
|-------|----------------------------------------|---------------|
| 1     | AI shell on Linux                      | 0–6 months    |
| 2     | AI-governed runtime                    | 6–18 months   |
| 3     | AI-native capability architecture      | 18–36 months  |
| 4     | Independent kernel/runtime evolution   | 36+ months    |

See the specification for milestones, dependencies, and open research
questions per phase.

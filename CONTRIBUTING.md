# Contributing to Gildos

Gildos is in the engineering-specification phase. The first runnable
deliverable is the **Minimum Viable Gildos** (MVG, see
`SPECIFICATION.md` §1.5). This file tells you how to get from zero to
a green `cargo check` on the Phase-1 scaffold.

## 1. Get the dev shell

We use **Nix flakes** (pinned in `flake.nix`). On any Linux box (or
macOS with `nix-darwin`):

```bash
nix develop
# you should now have rustc nightly, cargo, clang, lld, qemu, wasmtime
```

If you do not have Nix, install rustup nightly + clang 18 manually and
keep your toolchain pinned to the versions listed in `flake.nix`.

## 2. Build the workspace

```bash
cargo check          # typecheck all nine kernel-primitive crates
cargo nextest run    # run unit tests (most are stubs in Phase-1)
```

## 3. The four-tier ladder

See `SPECIFICATION.md` §17. The short version:

| Tier | Where                         | What runs                                   |
|------|-------------------------------|---------------------------------------------|
| 0    | Your laptop, no GPU           | Schemas, capabilities, sandbox, MockEngine  |
| 1    | Docker + optional GPU         | Full daemon stack, real inference engine    |
| 2    | QEMU/KVM VM, ARM64 or x86_64  | Boot chain end-to-end, recovery boot        |
| 3    | DGX Spark (or RTX 4090 MVG)   | Acceptance gating, milestone sign-off       |

A change SHOULD pass Tier 0–1 before review and MUST pass Tier 2 before
merge to `main`. Tier 3 is per-milestone, not per-commit.

## 4. The kernel primitives (`gildos-kmod/`)

Nine small crates, one per primitive in `SPECIFICATION.md` §4.4:

- `dmload`       — DirectModelLoad (§4.4.1) — **MVG-mandatory**
- `tensorbuf`    — Tensor handles (§4.4.2)
- `sched_cog`    — Cognitive scheduling class (§4.4.3) — **MVG-mandatory**
- `schedio`      — Semantic syscall rings (§4.4.4) — **MVG-mandatory**
- `sycache`      — Speculative syscall prefetch (§4.4.5)
- `vramctl`      — VRAM as managed resource (§4.4.6) — **MVG-mandatory**
- `kvfork`       — Cognitive Fork (§4.4.7)
- `promptpin`    — Persistent Prompt Objects (§4.4.8)
- `cogcg`        — Cognitive cgroups (§4.4.9)

Each crate currently exposes the **userspace type definitions** (the
data the caller passes to the syscall) plus stubbed function bodies
that return `Err(...)`. The kernel-side implementation will land once
Rust-for-Linux upstream stabilises the relevant APIs; until then, the
daemon code is written against these types and tested with the
MockEngine (`SPECIFICATION.md` §17.2).

## 5. Two-person review

Hard policy. Any change to:

- the TCB (`gildosvisor`, the verifier, the sandbox enforcer),
- the capability signing keys,
- the §11 learning policy,
- the security model,

requires two reviewers, configured at the repo level. Single-maintainer
merges to these areas are rejected.

## 6. Commit style

- Imperative mood, present tense.
- Reference the spec section: `spec(rev0.x):` or `code(§4.4.3):` etc.
- Keep the subject line ≤ 70 characters; explain *why* in the body.

## 7. Where to start

Pick an issue tagged `good-first-issue` in the MVG milestone, or one of:

- Wire a real implementation of `gildos-kmod/sched_cog` against
  `sched_ext` (the BPF scheduler) on a Linux 6.x dev box.
- Implement the `dmload` userspace shim using GPUDirect Storage on
  CUDA; benchmark against the §4.7 conformance criteria.
- Build a `MockEngine` for `cogd` that satisfies the §17.2 contract.
- Write three reference WASM capabilities (`user.notify`, `clock.now`,
  `events.query`) that pass the MVG demo (§1.5).

Welcome aboard.

## 8. CI (to be wired by a maintainer)

The repository's CI gate is intentionally NOT committed in this scaffold
commit — automated agents typically lack the GitHub `workflow` scope
required to publish under `.github/workflows/`. A maintainer SHOULD add
`.github/workflows/ci.yml` with the following job matrix:

```yaml
name: ci
on: { push: { branches: [main] }, pull_request: {} }
jobs:
  cargo-check:
    strategy:
      matrix:
        include:
          - { os: ubuntu-latest,    target: x86_64-unknown-linux-gnu }
          - { os: ubuntu-24.04-arm, target: aarch64-unknown-linux-gnu }
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@nightly
        with: { components: rustfmt,clippy,rust-src, targets: "${{ matrix.target }}" }
      - run: cargo check  --workspace --all-targets --target ${{ matrix.target }}
      - run: cargo clippy --workspace --all-targets --target ${{ matrix.target }} -- -D warnings
      - run: cargo fmt --all -- --check
```

This is the §4.7 / §17 ladder's lowest tier, gated on every PR.

# SEMUI v0.1.0

**SEMUI** is a semantic normalization and compilation layer for interfaces.

It turns UI source that is indirect, renderer-bound, and difficult to reason about into an explicit, canonical representation that can be verified, regenerated, and eventually compiled into other runtimes.

In the long run, the ambition is:

```text
UI source
-> semantic normalization
-> canonical UI IR
-> reliable emitters
-> multiple runtimes
```

Right now, the project is proving the first hard thing first:

```text
HTML/CSS -> SEMUI Scene IR -> HTML/CSS
```

That is not a toy milestone. It is the foundational invariant the rest of the system depends on.

---

## Why this exists

Modern UI source is full of indirection.

- CSS describes rules, not final visual truth
- layout emerges from algorithms
- inheritance and defaults hide final values
- browser behavior is powerful, but not a good interchange format
- source that looks simple is often much harder to port, diff, or regenerate faithfully than it seems

That creates a gap between:

- the UI as authored
- the UI as actually resolved
- the UI as needed by compilers, verification systems, editors, and AI systems

SEMUI is meant to fill that gap.

Its core idea is simple:

> turn implicit interface behavior into explicit canonical structure

Once a UI is normalized into a stable semantic form, it becomes much easier to:

- verify fidelity
- regenerate it deterministically
- compare it semantically
- extract reusable structure
- retarget it into other rendering systems

---

## What SEMUI is

SEMUI is:

- a **normalization layer** for interfaces
- a **compiler-style pipeline** for UI structure
- a **canonical scene representation** for approved UI
- a foundation for later **component, variant, and state extraction**
- a bridge between source UI and target runtimes

SEMUI is not:

- a design tool
- a browser replacement
- a new CSS
- a general-purpose UI framework
- a guarantee of byte-for-byte source preservation

The project is about preserving **visual and structural intent** through explicit normalization, not preserving every authored token of the original HTML and CSS.

---

## Current thesis

The working thesis of the project is:

```text
HTML/CSS is where design happens.
SEMUI is where approved design becomes explicit.
Emitters are where that explicit design is executed elsewhere.
```

That thesis is intentionally ambitious, but the implementation strategy is disciplined:

```text
fidelity -> abstraction -> capability
```

First prove the static fidelity loop.
Then extract reusable abstractions.
Then grow into richer semantics and multi-runtime compilation.

---

## Where the project is right now

SEMUI is currently in its **v0.1 static fidelity loop** phase.

That phase is focused on proving a bounded claim:

> `HTML/CSS -> SEMUI -> HTML/CSS` can round-trip a canonical fixture corpus with negligible meaningful drift.

This means the current implementation is centered on:

- parsing fixture HTML and CSS
- resolving a bounded CSS subset
- normalizing layout into explicit geometry
- extracting a canonical **Scene IR**
- emitting canonical HTML/CSS
- verifying round-trip equivalence
- diagnosing unsupported input instead of silently pretending it worked

This is already real compiler work. It is not the final shape of the system, but it is the right base layer.

What exists now is best understood as **scene normalization**, not yet full semantic abstraction.

That distinction matters.

Today, SEMUI is proving that a UI scene can become explicit, deterministic, and verifiable.

Later phases will prove that those scenes can be abstracted into reusable components, variants, and states.

---

## What the current system does

The current pipeline is organized like a compiler.

```text
Source HTML/CSS
-> source graph
-> style resolver
-> layout normalization
-> Scene IR extraction
-> canonical HTML/CSS emission
-> verification
```

### Source graph

The system parses HTML and CSS into structured source artifacts:

- HTML nodes
- CSS rules and declarations
- source provenance such as document IDs and DOM paths

This is the intake layer.

### Resolver

The resolver applies a bounded cascade and inheritance model to produce per-element computed style.

This is where CSS stops being indirect and starts becoming explicit.

### Layout

The layout stage normalizes supported layout behavior into explicit geometry and box data.

For v0.1, this is intentionally bounded and focused on the subset needed by the fixture corpus.

### Scene IR extraction

The extractor converts the normalized scene into a canonical IR.

This IR records explicit per-node facts such as:

- node kind
- hierarchy
- layout
- paint
- typography
- control kind
- provenance

At the current stage, this is a **scene-level IR**, not yet a reusable component system.

### Emitter

The emitter generates canonical HTML and CSS from the IR.

This is not trying to recover the original source exactly.
It is trying to regenerate an equivalent scene in a stable, normalized form.

### Verification

The verifier runs the full round-trip loop and checks whether the source scene and the re-emitted scene remain equivalent within the declared loss budget.

That includes:

- structural checks
- computed-style checks
- visual checks

---

## The IR

The current IR is a **canonical Scene IR**.

That means it is designed to capture a resolved UI scene as explicit data, not to preserve authored source form exactly.

A scene contains ordered nodes with explicit:

- parent relationships
- layout
- paint
- typography
- control semantics
- provenance

This is the first crucial layer.

In later milestones, the system is expected to grow upward into richer semantic layers such as:

- component extraction
- variant extraction
- state modeling
- bounded interaction semantics
- cross-runtime emitters

But the current system is intentionally proving the scene layer first.

That is the right order.

---

## What v0.1 does and does not promise

### v0.1 does promise

- a bounded static HTML/CSS support envelope
- a canonical fixture corpus
- explicit supported-subset documentation
- acceptance gates for structural, computed-style, and visual fidelity
- diagnostics for unsupported constructs
- a real normalization and emission loop

### v0.1 does not promise

- full browser rendering compatibility
- arbitrary JavaScript execution
- responsive design
- pseudo-class and pseudo-element behavior
- animation fidelity
- universal HTML/CSS ingestion
- source-preserving re-emission

The point of v0.1 is not completeness.

The point is to prove that a bounded but meaningful UI subset can be normalized into a canonical representation with very little loss.

---

## Why the bounded proof matters

A lot of ambitious interface projects fail because they start by claiming too much.

SEMUI is taking the opposite route.

The project is trying to earn its larger claims by first proving one narrow but important invariant:

> approved static UI can be normalized into an explicit canonical scene and regenerated with bounded drift

If that is true, then a lot becomes possible:

- semantic UI diffs
- stable review artifacts
- reusable component inference
- design system extraction
- cross-runtime emitters
- AI systems that operate on explicit interface structure instead of loose prompts

So the small proof is not a detour.
It is the base of the ladder.

---

## Repository focus

The repository is currently organized around the bounded proof.

Key areas include:

- `src/source_graph/` — source parsing and intake
- `src/resolver/` — cascade and inheritance resolution
- `src/layout/` — geometry normalization
- `src/extractor/` — Scene IR extraction
- `src/emitter/` — canonical emission
- `src/verification/` — round-trip proof harness
- `src/diagnostics/` — unsupported construct reporting
- `src/release/` — corpus proof and golden artifact workflows
- `src/ir/` — the SEMUI IR schema

And the current proof corpus lives under:

- `fixtures/v0.1/`

with project-level milestone contracts in:

- `docs/v0.1-supported-css-subset.md`
- `docs/v0.1-loss-budget.md`
- `docs/v0.1-acceptance-gate.md`
- `docs/v0.1-fixture-scenes.md`

---

## Canonical fixtures

The v0.1 proof uses a small canonical fixture corpus.

These scenes are intentionally compact, diagnostic, and reviewable. They are designed to stress the normalization boundary without dragging in the full web platform too early.

The current corpus includes scenes such as:

- absolute-positioned card layouts
- vertical information cards
- horizontal action rows
- nested inset panels
- typography-led specimens
- compact notification/toast surfaces

The fixture corpus is not just test data.
It is the contract surface for the current milestone.

---

## Roadmap

### v0.1 — Static fidelity loop

Prove:

```text
HTML/CSS -> Scene IR -> HTML/CSS
```

with a bounded support envelope and measurable loss budget.

Focus:

- static scenes
- explicit geometry
- paint
- typography
- block and flex subset
- canonical verification

### v0.2 — Component and variant extraction

Build analysis passes that move beyond raw scene structure:

- repeated structure detection
- base component inference
- variant extraction
- override modeling

Goal:

> SEMUI expresses reusable abstractions, not only resolved scenes

### v0.3 — Richer layout semantics

Move from scene facts toward stronger semantic layout vocabulary:

- stack relationships
- spacing intent
- containment semantics
- alignment abstractions

### v0.4 — State representation

Introduce explicit representation for bounded interactive states such as:

- hover
- active
- selected
- disabled
- focus, where in scope

### v0.5+ — Interaction and retargeting

Grow toward:

- bounded interaction semantics
- runtime emitters
- cross-platform compilation
- richer semantic tooling around normalized UI

---

## Long-term direction

The larger vision for SEMUI is not just “HTML/CSS in, HTML/CSS out.”

The long-term direction is:

```text
approved UI
-> canonical semantic representation
-> reliable regeneration, analysis, and retargeting
```

That could eventually support:

- cross-runtime UI cloning
- semantic interface diffs
- design system mining
- AI-native UI editing on explicit structure
- constrained code generation
- verification-first UI workflows

The aspiration is big, but the implementation strategy stays grounded:

- prove one layer
- lock the contract
- expand deliberately

---

## Guiding principles

### 1. Explicitness over hidden behavior

If a property, layout relationship, or visual fact matters, SEMUI should make it explicit.

### 2. Determinism over cleverness

The system should prefer stable, inspectable normalization to magical inference that cannot be verified.

### 3. Bounded promises

Support envelopes and loss budgets should be declared, not implied.

### 4. Fidelity before abstraction

A bad abstraction layer on top of unstable scene normalization is worse than no abstraction layer at all.

### 5. Semantic growth by phases

Component extraction, variants, states, and runtime compilation should grow from a proven base, not replace it prematurely.

---

## Running the project

This is a Rust crate.

At the current stage, the main workflow is test- and proof-oriented.

Typical local command:

```bash
cargo test
```

As the system matures, this will likely expand into dedicated commands for:

- fixture verification
- golden artifact generation
- diagnostics reports
- milestone proof summaries

---

## Project status in one sentence

SEMUI is building a canonical interface layer by proving that a bounded HTML/CSS scene can be normalized, regenerated, and verified with negligible meaningful drift.

---

## Final summary

SEMUI is trying to establish a missing layer in interface systems:

```text
Design source
-> semantic normalization
-> canonical UI representation
-> reliable execution elsewhere
```

Today, that means a disciplined Scene IR and a real static fidelity loop.

Tomorrow, if that layer holds, it becomes the foundation for reusable UI semantics, cross-runtime compilation, and AI systems that can work on interfaces as explicit structure instead of guesswork.

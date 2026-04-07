# SEMUI: Semantic UI Normalization and Compilation System

## Overview

SEMUI is a system for transforming user interfaces into a **canonical, explicit, semantic representation of visual structure**, and using that representation to generate equivalent interfaces across different rendering systems.

It introduces a missing layer in UI development:

```text
UI Source (HTML/CSS/JS)
→ Semantic Extraction (omni-ast)
→ SEMUI (canonical visual IR)
→ Target Emitters (WGPU, Dioxus, SwiftUI, etc.)
```

SEMUI is not a design tool.
It is not a UI framework.

It is:

> **a semantic normalization and compilation layer for visual interfaces**

---

## Core Principle

> Turn implicit visual behavior into explicit semantic structure.

Modern UI systems are fundamentally indirect:

- CSS describes rules, not results
- Layout emerges from algorithms
- Inheritance and cascade hide final values
- Rendering depends on platform-specific engines

This makes UI:

- hard to reason about
- hard to port
- hard for AI to reproduce precisely

SEMUI resolves this by producing:

> **a fully explicit, renderer-independent representation of what the UI actually is**

---

## System Architecture

### 1. omni-ast (Semantic Substrate)

omni-ast is the **primitive receptor layer**.

It provides:

- parsing (HTML, CSS, JS, etc.)
- AST normalization
- selector and reference resolution primitives
- structural graph extraction
- semantic classification (taxonomy, roles)
- provenance tracking (source spans)
- error-tolerant extraction

omni-ast answers:

> “What exists in the source?”

SEMUI builds on omni-ast in the same way SEMMAP does:

- shared semantic extraction (~70%)
- domain-specific interpretation (~30%)

---

### 2. SEMUI Extraction Layer

SEMUI converts parsed source into a **semantic visual graph**.

This includes:

- DOM hierarchy (explicit)
- resolved styles (no cascade ambiguity)
- layout relationships (explicit positioning or constraints)
- geometry (width, height, bounds)
- paint (fill, border, shadow, opacity)
- typography (fully resolved)
- interaction states (hover, pressed, focus, etc.)
- transitions and animations
- component boundaries (inferred via structure/fingerprinting)

This stage removes:

- CSS cascade ambiguity
- inheritance ambiguity
- layout indirection
- implicit browser defaults

Output is:

> **fully resolved visual truth**

---

### 3. SEMUI IR (Intermediate Representation)

SEMUI IR is the canonical format.

It represents UI as structured semantic objects:

- **components**
- **nodes**
- **variants**
- **states**

Each node contains:

- frame (geometry)
- shape (primitive + radius)
- paint (fill, border, shadow)
- layout (alignment, constraints)
- content (text, icon)
- typography
- interaction
- transitions
- semantics (role, intent)

Example:

```text
COMPONENT BUTTON_BASE:
  frame: 140x40
  shape: rounded-rect 10
  content-layout: center

VARIANT BUTTON_PRIMARY extends BUTTON_BASE:
  fill: #111827
  label: "Continue" #FFFFFF
```

SEMUI IR is:

- explicit
- deterministic
- portable
- human-readable
- machine-executable

It becomes the **source of truth after design approval**.

---

### 4. Execution Modes (Critical Layer)

SEMUI is designed to be consumed by both:

- deterministic compilers
- large language models

However:

> LLMs are not naturally deterministic. They are trained to be helpful and creative.

This creates a failure mode:

- adding extra UI
- inventing states
- modifying layout
- applying “best practices”

To solve this, SEMUI introduces **execution modes**.

---

## Strict Compiler Mode

```text
MODE: STRICT_UI_COMPILER

RULES:
- Output only what is explicitly described
- Do not add demo sections, wrappers, headings, or examples
- Do not add hover, focus, active, pressed, or disabled states unless specified
- Do not add transitions, transforms, shadows, outlines, or effects unless specified
- Do not substitute fonts beyond declared fallbacks
- Do not center, pad, margin, or position elements unless specified
- Do not invent accessibility styles or interaction polish
- Do not expand scope beyond the described components
- Preserve component inheritance and variant structure
- Prefer minimal code over illustrative code
```

### Output Constraints

```text
OUTPUT_RULES:
  preserve-inheritance: true
  no-extra-styles: true
  no-implicit-shadow: true
  no-implicit-padding: true
  no-implicit-font-substitution: true
  center-label-by-flex: true
  use-border-box: true
```

---

## Why Execution Mode Matters

Experiments show:

- Without strict mode → models behave like designers
- With strict mode → models behave like compilers

Therefore:

```text
SEMUI IR
+ Execution Mode
= Convergent UI generation
```

This is a core property of the system—not an implementation detail.

---

### 5. Target Emitters

Emitters convert SEMUI IR into platform-specific implementations:

- WGPU / custom renderers
- Dioxus (Rust)
- SwiftUI
- Jetpack Compose
- Flutter
- HTML/CSS (regeneration)

Emitters are responsible for:

- mapping primitives to framework constructs
- approximating unsupported features
- preserving structure and variants
- maintaining visual fidelity

No design inference is required at this stage.

---

## Workflow

### Step 1: Design (HTML/CSS)

```text
Prompt → AI generates HTML/CSS
Human reviews → approves visual output
```

HTML is used because:

- it is the strongest UI medium for LLMs
- rendering is immediate and reliable

---

### Step 2: Normalize (SEMUI)

```text
HTML/CSS → omni-ast → SEMUI extraction → SEMUI IR
```

This step:

- resolves all implicit behavior
- produces explicit visual structure

---

### Step 3: Compile

```text
SEMUI IR + STRICT MODE → emitter → target UI
```

---

## Design Goals

### 1. Determinism

Outputs should converge across models and compilers.

### 2. Explicitness

No reliance on:

- defaults
- cascade
- hidden layout behavior

### 3. Modularity

UI must be expressed as:

- components
- variants
- overrides

### 4. Extractability

Must work on:

- real-world HTML/CSS
- imperfect inputs
- partial systems

### 5. Fidelity

Goal is:

> visually indistinguishable output across runtimes

---

## Relationship to SEMMAP and omni-ast

- **omni-ast** → semantic extraction engine
- **SEMMAP** → semantic understanding of codebases
- **SEMUI** → semantic understanding of interfaces

All follow the same pattern:

```text
implicit system
→ semantic normalization
→ structured representation
→ reliable AI/system interaction
```

---

## Key Insight

SEMUI is not:

- a better CSS
- a UI DSL
- a design language

It is:

> **the explicit, normalized result of rendering systems**

---

## What This Enables

- cross-platform UI cloning
- semantic UI diffs (not CSS diffs)
- automatic component extraction
- design system generation
- AI-native UI editing
- visual regression validation
- deterministic UI codegen

---

## Final Model

```text
HTML/CSS → where AI designs
SEMUI → where design becomes law
Emitters → where law is executed
```

---

## Summary

SEMUI introduces a missing abstraction layer:

```text
Design
→ Semantic Normalization
→ Implementation
```

It transforms UI from:

- implicit
- renderer-bound
- non-deterministic

into:

- explicit
- portable
- deterministic

This enables reliable UI generation across systems, and establishes a foundation for AI-native interface development.

## Non-Goals (v0)

SEMUI does not attempt to:

- implement a full browser rendering engine
- execute arbitrary JavaScript logic
- support all of CSS or edge-case browser behavior
- solve responsive design in v0
- replace existing UI frameworks
- guarantee identical code output across targets

SEMUI is focused on:

> extracting and reproducing **visual and structural intent**, not emulating the full web platform

---

## Execution Contract

SEMUI is not just a representation format.
It requires a defined execution mode to ensure deterministic behavior when used with LLMs.

Execution mode is part of the system contract.

Without it:

- models introduce interpretation
- outputs diverge
- fidelity degrades

With it:

- models behave as compilers
- outputs converge

```text
SEMUI IR
+ Execution Mode
= Reliable UI generation
```

---

## Authoring Model

SEMUI is **not primarily hand-authored**.

Typical usage:

```text
HTML/CSS (designed + approved)
→ converted to SEMUI
→ optionally refined
→ compiled to target runtimes
```

SEMUI is:

> a generated canonical layer, not the primary design surface

---

## Phased Development Roadmap

SEMUI will be built incrementally, prioritizing validation over completeness.

### v0.1 — Static Fidelity Loop

- HTML/CSS → SEMUI → HTML/CSS
- absolute positioning
- basic layout (flex optional)
- typography, color, borders, radius
- no interactions or dynamic data

Goal:

> zero meaningful visual drift in round-trip

---

### v0.2 — Component & Variant Extraction

- detect repeated structures
- infer base components
- extract variants (color/text differences)

Goal:

> SEMUI expresses reusable abstractions, not raw DOM

---

### v0.3 — Layout Semantics

- explicit stack (vertical/horizontal)
- alignment rules
- spacing (gap vs margin)
- container relationships

Goal:

> remove reliance on browser layout behavior

---

### v0.4 — State Representation

- hover
- active
- selected
- disabled

Goal:

> support interactive visual parity

---

### v0.5 — Interaction & Data (Bounded)

- click → action
- toggle state
- open/close modal
- simple collections (lists)

Goal:

> support real application surfaces without full JS execution

---

## First Success Metric

The first proof of SEMUI is:

> HTML/CSS → SEMUI → HTML/CSS reproduces the approved UI with negligible visual drift.

This is the foundational invariant.

All further capabilities build on this.

---

## Guiding Principle

SEMUI is not built for completeness first.

It is built for:

```text
fidelity → abstraction → capability
```

Each phase must produce usable, verifiable results before expanding scope.

---

## Product Direction

SEMUI is valuable if it enables:

```text
approved HTML/CSS
→ instant semantic normalization
→ reliable cross-runtime UI generation
```

The long-term goal is not just translation.

It is:

> a stable, semantic interface between design, AI systems, and runtime implementations

---

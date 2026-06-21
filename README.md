# SEMUI

SEMUI exists to let you author and approve a UI once in HTML/CSS, then reproduce
that UI faithfully in other frameworks and rendering runtimes without manually
redesigning it.

HTML/CSS is the universal authoring frontend. Scene IR is the portable boundary.
Target emitters execute that resolved visual specification in environments such
as native toolkits, GPU renderers, and application frameworks.

The current pipeline is:

```text
HTML/CSS
  -> source graph
  -> bounded style resolution
  -> geometry normalization
  -> Scene IR
  -> target emitters
  -> HTML/CSS, native UI, GPU renderers, application frameworks
```

## Current status

SEMUI is a prototype, not a browser engine and not a proven UI compiler.
The current HTML emitter is a fidelity test and reference backend, not the final
product destination.

Implemented:

- fixture HTML and CSS parsing
- a small type/class selector and cascade subset
- explicit-pixel and border-box normalization
- Scene IR extraction and JSON serialization
- canonical HTML/CSS emission
- internal IR round-trip regression tests
- diagnostics for unsupported input, with an executable coverage matrix
- a deterministic browser proof: computed-style, geometry, and scene-cropped
  visual diffs over the canonical corpus, reproduced in CI
- a reproducible release evidence bundle reporting every gate per scene

Not yet implemented:

- general CSS layout beyond the declared subset
- responsive behavior or JavaScript
- component, variant, state, or interaction inference
- non-HTML runtime emitters

The internal round trip is a fast regression layer, not the fidelity oracle.
Release-level fidelity is judged from the deterministic browser observations
above; the v0.1 corpus passes all six gates today.

## Repository layout

- `src/source_graph/`: input parsing
- `src/resolver/`: bounded cascade and inheritance
- `src/layout/`: declared geometry normalization
- `src/ir/`: Scene IR contract
- `src/extractor/`: source-to-IR mapping
- `src/emitter/`: canonical HTML/CSS generation
- `src/verification/`: internal round-trip checks
- `src/diagnostics/`: unsupported-input reporting
- `src/release/`: corpus proof plumbing
- `fixtures/v0.1/`: canonical fixture corpus
- `docs/v0.1-*.md`: current scope and acceptance contracts
- `docs/product-direction.md`: durable product objective and success criteria
- `docs/product-opportunities.md`: downstream capabilities enabled by the core

## Development

```bash
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test --all-targets
```

Tests must not rewrite checked-in fixtures or golden artifacts. Golden
generation is an explicit maintenance operation, separate from verification.

Issues, execution plans, and architecture decisions are canonical in Ishoo:

```bash
ishoo status
ishoo plan show
ishoo decision list
```

## Direction

The immediate milestone is narrow: make HTML/CSS to Scene IR normalization
honest, deterministic, and browser-verified for the declared subset. That is
foundation work for portability, not the product endpoint.

Scene IR remains the sole product contract until that milestone passes.
The next milestone defines the target-emitter contract, implements a materially
non-browser reference target, and proves the same canonical scenes across
runtimes. Component inference and richer semantics remain optional layers above
that portable scene truth.

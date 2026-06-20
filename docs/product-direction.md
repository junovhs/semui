# Product Direction

## Objective

Author and approve a UI once in HTML/CSS, then reproduce it faithfully across
different frameworks and rendering runtimes without manually rebuilding the
design.

HTML/CSS provides the mature authoring environment. SEMUI resolves its implicit
browser behavior into a canonical, runtime-neutral Scene IR. Independent target
emitters consume that IR.

```text
HTML/CSS authoring
  -> resolved Scene IR
  -> HTML reference emitter
  -> native toolkit emitter
  -> GPU renderer emitter
  -> framework adapters
```

## Why Scene IR exists

Direct source-to-source generation makes every target independently interpret
CSS, layout, inheritance, browser defaults, and control behavior. Those
interpretations drift.

SEMUI resolves that ambiguity once. Emitters receive explicit structure,
geometry, paint, typography, content, and control semantics instead of raw CSS.

## Success criteria

The project succeeds when:

1. A supported HTML/CSS scene produces deterministic Scene IR.
2. The HTML reference emitter reconstructs it within the browser fidelity
   budget.
3. A materially non-browser emitter reconstructs the same scene within a
   declared cross-runtime fidelity budget.
4. Additional targets can be implemented as adapters over Scene IR without
   modifying source parsing or duplicating CSS resolution.

Supporting every browser feature or every runtime is not required. The contract
is bounded portability: declared inputs, explicit loss, and measurable output.

## Sequencing

Fidelity comes before portability because every target otherwise inherits an
untrustworthy source model. Portability comes before component inference
because cross-runtime execution is the core product; components, variants, and
tokens are higher-level capabilities built on the same scene truth.

The governing Ishoo decisions are `DEC-01`, `DEC-02`, and `DEC-04`.

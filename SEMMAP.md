# project -- Semantic Map

**Purpose:** Parse HTML and CSS fixture sources into a traceable source graph for the SEMUI v0.1 Rust implementation.

## Legend

`[ENTRY]` Application entry point

`[CORE]` Core business logic

`[TYPE]` Data structures and types

`[UTIL]` Utility functions

`[HOTSPOT]` High fan-in file imported by 4+ others - request this file early in any task

`[GLOBAL-UTIL]` High fan-in utility imported from 3+ distinct domains

`[DOMAIN-CONTRACT]` Shared contract imported mostly by one subsystem

`[ROLE:model]` Primary domain model or state-holding data structure.

`[ROLE:controller]` Coordinates commands, events, or request handling.

`[ROLE:rendering]` Produces visual output or drawing behavior.

`[ROLE:view]` Represents a reusable UI view or presentation component.

`[ROLE:dialog]` Implements dialog-oriented interaction flow.

`[ROLE:config]` Defines configuration loading or configuration schema behavior.

`[ROLE:os-integration]` Bridges the application to OS-specific APIs or services.

`[ROLE:utility]` Provides cross-cutting helper logic without owning core flow.

`[ROLE:bootstrap]` Initializes the application or wires subsystem startup.

`[ROLE:build-only]` Supports the build toolchain rather than runtime behavior.

`[COUPLING:pure]` Logic stays within the language/runtime without external surface coupling.

`[COUPLING:mixed]` Blends pure logic with side effects or boundary interactions.

`[COUPLING:ui-coupled]` Depends directly on UI framework, rendering, or windowing APIs.

`[COUPLING:os-coupled]` Depends directly on operating-system services or platform APIs.

`[COUPLING:build-only]` Only relevant during build, generation, or compilation steps.

`[BEHAVIOR:owns-state]` Maintains durable in-memory state for a subsystem.

`[BEHAVIOR:mutates]` Changes application or model state in response to work.

`[BEHAVIOR:renders]` Produces rendered output, drawing commands, or visual layout.

`[BEHAVIOR:dispatches]` Routes commands, events, or control flow to other units.

`[BEHAVIOR:observes]` Listens to callbacks, notifications, or external signals.

`[BEHAVIOR:persists]` Reads from or writes to durable storage.

`[BEHAVIOR:spawns-worker]` Creates background workers, threads, or async jobs.

`[BEHAVIOR:sync-primitives]` Coordinates execution with locks, channels, or wait primitives.

`[SURFACE:filesystem]` Touches filesystem paths, files, or directory traversal.

`[SURFACE:ntfs]` Uses NTFS-specific filesystem semantics or metadata.

`[SURFACE:win32]` Touches Win32 platform APIs or Windows-native handles.

`[SURFACE:shell]` Integrates with shell commands, shell UX, or command launch surfaces.

`[SURFACE:clipboard]` Reads from or writes to the system clipboard.

`[SURFACE:gdi]` Uses GDI drawing primitives or related graphics APIs.

`[SURFACE:control]` Represents or manipulates widget/control surfaces.

`[SURFACE:view]` Represents a view-level presentation surface.

`[SURFACE:dialog]` Represents a dialog/window interaction surface.

`[SURFACE:document]` Represents document-oriented editing or display surfaces.

`[SURFACE:frame]` Represents application frame/window chrome surfaces.

`[BEHAVIOR:async]` Uses async/await patterns for concurrent execution.

`[BEHAVIOR:panics-on-error]` Contains unwrap/expect/panic patterns that abort on failure.

`[BEHAVIOR:logs-and-continues]` Logs errors and continues without propagating or aborting.

`[BEHAVIOR:returns-nil-on-error]` Returns nil/null/None on error instead of propagating.

`[BEHAVIOR:swallows-errors]` Catches errors without re-raising or propagating them.

`[BEHAVIOR:propagates-errors]` Propagates errors to callers via Result, throw, or raise.

`[SURFACE:http-handler]` Implements HTTP request handling or web endpoint logic.

`[SURFACE:database]` Interacts with database services or ORMs.

`[SURFACE:external-api]` Makes outbound calls to external HTTP APIs or services.

`[SURFACE:template]` Uses template engines for rendering output.

`[QUALITY:undocumented]` Has public symbols without documentation.

`[QUALITY:complex-flow]` Contains functions with high cognitive complexity.

`[QUALITY:error-boundary]` Concentrated error handling — many panic, swallow, or propagation sites.

`[QUALITY:concurrency-heavy]` Uses multiple concurrency primitives (async, locks, spawn).

`[QUALITY:syntax-degraded]` Parse errors detected — semantic analysis may be incomplete.

## Layer 0 -- Config

`Cargo.toml`
Workspace configuration.

`SEMMAP.md`
Generated semantic map.

`neti.toml`
Configuration for neti.

## Layer 1 -- Domain (Engine)

`src/emitter/css.rs`
CSS generation from IR nodes. [HOTSPOT] [COUPLING:pure] [QUALITY:complex-flow]
Exports: build_css, px
Semantic: pure computation

`src/emitter/html.rs`
HTML generation from IR nodes. [HOTSPOT] [COUPLING:pure]
Exports: build_html
Semantic: pure computation

`src/extractor/map.rs`
Mapping functions from resolver/layout types to SEMUI IR types. [HOTSPOT] [COUPLING:pure] [BEHAVIOR:propagates-errors]
Exports: to_layout_default, to_paint_default, to_typography, to_layout
Semantic: pure computation that propagates errors

`src/fixture_manifest.rs`
model for fixture manifest via file I/O. [COUPLING:mixed] [BEHAVIOR:persists,propagates-errors] [QUALITY:undocumented]
Exports: FixtureManifest.manifest_root, FixtureSceneEntry, FixtureManifestError, FixtureManifestError.fmt
Semantic: side-effecting adapter that propagates errors

`src/ir/layout.rs`
CSS `position` value supported in v0.1. [COUPLING:pure] [QUALITY:undocumented]
Exports: BoxSizing, FlexDirection, JustifyContent, AlignItems
Semantic: pure computation

`src/ir/paint.rs`
Implements Border functionality. [TYPE] [HOTSPOT]
Exports: Border, Color, Paint

`src/ir/typography.rs`
Resolved CSS `line-height`.
Exports: LineHeight, Typography

`src/layout/model.rs`
Explicit geometry for a single element node in the v0.1 subset. [TYPE]
Exports: LaidOutNode, LaidOutScene, Geometry

`src/resolver/cascade.rs`
Parses color. [HOTSPOT] [COUPLING:pure]
Exports: apply_declaration, apply_inheritance, parse_color, parse_px
Semantic: pure computation

`src/resolver/model.rs`
Fully resolved CSS properties for a single HTML element node in the v0.1 subset. [TYPE] [HOTSPOT] [GLOBAL-UTIL] [COUPLING:pure]
Exports: ComputedStyle, ComputedStyle.default
Semantic: pure computation

`src/resolver/selector.rs`
CSS specificity for the v0.1 subset: (class_count, type_count). [COUPLING:pure]
Exports: selector_matches, Specificity, specificity
Semantic: pure computation

`src/source_graph.rs`
utility for source graph via file I/O. [HOTSPOT] [GLOBAL-UTIL] [COUPLING:mixed] [BEHAVIOR:persists,propagates-errors] [QUALITY:error-boundary]
Exports: load_scene_source_graph, SceneSourceGraph.from_strings, SceneSourceGraph.load
Touch: Contains inline Rust tests alongside runtime code.
Semantic: side-effecting adapter that propagates errors

`src/source_graph/css.rs`
Parses css document. [HOTSPOT] [COUPLING:pure] [BEHAVIOR:propagates-errors]
Exports: parse_css_document
Semantic: pure computation that propagates errors

`src/source_graph/html.rs`
Parses html document. [COUPLING:pure] [BEHAVIOR:propagates-errors] [QUALITY:error-boundary]
Exports: parse_html_document
Semantic: pure computation that propagates errors

`src/source_graph/html_support.rs`
Parses start tag. [COUPLING:pure] [BEHAVIOR:propagates-errors] [QUALITY:undocumented]
Exports: find_tag_end, parse_start_tag, StartTag
Semantic: pure computation that propagates errors

`src/source_graph/model.rs`
Implements source graph error.from. [TYPE] [COUPLING:pure] [QUALITY:undocumented]
Exports: HtmlNodeKind, SourceDocumentKind, SceneSourceGraph, SourceGraphError
Semantic: pure computation

## Layer 2 -- Adapters / Infra

`src/fixture_manifest/parse.rs`
Implements parse functionality. [UTIL] [COUPLING:pure] [BEHAVIOR:propagates-errors] [QUALITY:error-boundary]
Semantic: pure computation that propagates errors

## Layer 3 -- App / Entrypoints

`src/diagnostics/mod.rs`
Static analysis pass: report constructs that are silently dropped or fall back to defaults during v0.1 normalization. [ENTRY] [HOTSPOT] [GLOBAL-UTIL] [COUPLING:mixed] [BEHAVIOR:owns-state]
Exports: DiagnosticKind, analyze, Diagnostic
Touch: Contains inline Rust tests alongside runtime code.
Semantic: side-effecting stateful module

`src/emitter/mod.rs`
Emit minimal HTML and CSS from a [`SceneIr`] (strict mode, v0.1). [ENTRY] [HOTSPOT] [GLOBAL-UTIL] [COUPLING:pure]
Exports: EmittedScene, emit
Touch: Contains inline Rust tests alongside runtime code.
Semantic: pure computation

`src/extractor/mod.rs`
Extracts a [`SceneIr`] from a [`LaidOutScene`] + [`SceneSourceGraph`]. [ENTRY] [HOTSPOT] [GLOBAL-UTIL] [COUPLING:mixed] [BEHAVIOR:owns-state] [QUALITY:complex-flow]
Exports: extract_ir, ExtractorError, ExtractorError.fmt
Touch: Contains inline Rust tests alongside runtime code.
Semantic: side-effecting stateful module

`src/ir/mod.rs`
SEMUI v0.1 Intermediate Representation schema and serialization contract. [TYPE] [HOTSPOT] [GLOBAL-UTIL] [COUPLING:pure] [QUALITY:undocumented]
Exports: ExecutionMode, ControlKind, SceneIr, SceneIr.from_json
Touch: Contains inline Rust tests alongside runtime code.
Semantic: pure computation

`src/layout/mod.rs`
Layout and geometry computation for the SEMUI v0.1 static subset. [ENTRY] [HOTSPOT] [GLOBAL-UTIL] [COUPLING:pure]
Exports: compute_layout
Touch: Contains inline Rust tests alongside runtime code.
Semantic: pure computation

`src/lib.rs`
Re-exports the public API surface. [ENTRY]
Exports: diagnostics, emitter, extractor, layout

`src/release/mod.rs`
v0.1 release proof: run the full corpus, generate golden artifacts, and capture the evidence that round-trip fidelity is within budget. [ENTRY] [HOTSPOT] [COUPLING:mixed] [BEHAVIOR:persists,propagates-errors] [QUALITY:error-boundary]
Exports: CorpusProof.total_ir_nodes, write_golden_artifacts, run_corpus_proof, CorpusProof.all_pass
Touch: Contains inline Rust tests alongside runtime code.
Semantic: side-effecting adapter that propagates errors

`src/resolver/mod.rs`
Static-scene style resolver for the SEMUI v0.1 subset. [ENTRY] [HOTSPOT] [GLOBAL-UTIL] [COUPLING:pure]
Exports: ComputedStyle, ResolvedNode, ResolverError, ResolverError.fmt
Touch: Contains inline Rust tests alongside runtime code.
Semantic: pure computation

`src/verification/mod.rs`
Round-trip regression harness for the v0.1 fixture corpus. [ENTRY] [HOTSPOT] [COUPLING:pure] [BEHAVIOR:propagates-errors] [QUALITY:error-boundary]
Exports: verify_round_trip, VerificationResult, Drift
Touch: Contains inline Rust tests alongside runtime code.
Semantic: pure computation that propagates errors

## Layer 4 -- Tests

`src/diagnostics/tests/integration.rs`
Integration tests: run analyze() against the real fixture corpus. [COUPLING:pure]
Semantic: pure computation

`src/diagnostics/tests/mod.rs`
Module definitions for mod. [ENTRY]

`src/diagnostics/tests/unit.rs`
Unit tests for the diagnostics analysis logic. [COUPLING:pure]
Semantic: pure computation

`src/emitter/tests/css.rs`
Unit tests for CSS emission (css.rs helpers). [COUPLING:pure]
Semantic: pure computation

`src/emitter/tests/html.rs`
Unit tests for HTML emission (html.rs). [COUPLING:mixed]
Semantic: side-effecting

`src/emitter/tests/integration.rs`
Integration tests: emit -> HTML/CSS from real fixture IR. [COUPLING:pure]
Semantic: pure computation

`src/emitter/tests/mod.rs`
Module definitions for mod. [ENTRY]

`src/extractor/tests/integration.rs`
Integration tests for extract_ir against the fixture corpus. [COUPLING:pure]
Semantic: pure computation

`src/extractor/tests/map.rs`
Unit tests for the mapping layer (map.rs). [COUPLING:mixed]
Semantic: side-effecting

`src/extractor/tests/mod.rs`
Module definitions for mod. [ENTRY]

`src/ir/tests/contract.rs`
Tests for super. [COUPLING:pure]
Semantic: pure computation

`src/ir/tests/mod.rs`
Tests for super. [ENTRY] [COUPLING:pure]
Semantic: pure computation

`src/ir/tests/roundtrip.rs`
Exercises every field in IrNode: box, control, and text nodes with full layout, paint, typography, and provenance spans. [COUPLING:pure]
Semantic: pure computation

`src/layout/tests/geometry.rs`
Tests for crate. [COUPLING:pure]
Semantic: pure computation

`src/layout/tests/integration.rs`
Tests for crate. [COUPLING:pure]
Semantic: pure computation

`src/layout/tests/mod.rs`
Module definitions for mod. [ENTRY]

`src/release/tests/mod.rs`
Module definitions for mod. [ENTRY]

`src/release/tests/proof.rs`
Release proof tests — final acceptance gate for v0.1. [COUPLING:pure]
Semantic: pure computation

`src/resolver/tests/cascade.rs`
Tests for crate. [COUPLING:pure]
Semantic: pure computation

`src/resolver/tests/integration.rs`
The profile_card_absolute fixture is the primary anchor scene and the most demanding for the resolver: absolute positioning, compound selectors, cascade between shared and specific rules, and font-family inheritance. [COUPLING:pure]
Semantic: pure computation

`src/resolver/tests/mod.rs`
Module definitions for mod. [ENTRY]

`src/resolver/tests/selector.rs`
Tests for crate. [COUPLING:pure]
Semantic: pure computation

`src/source_graph/tests.rs`
Tests for crate. [COUPLING:mixed] [BEHAVIOR:panics-on-error]
Semantic: side-effecting that panics on error

`src/verification/tests/mod.rs`
Module definitions for mod. [ENTRY]

`src/verification/tests/roundtrip.rs`
Round-trip regression tests for the v0.1 fixture corpus. [COUPLING:pure]
Semantic: pure computation


## DependencyGraph

```yaml
DependencyGraph:
  # --- Entrypoints ---
  lib.rs:
    Imports: [diagnostics/mod.rs, emitter/mod.rs, extractor/mod.rs, fixture_manifest.rs, ir/mod.rs, layout/mod.rs, release/mod.rs, resolver/mod.rs, source_graph.rs, verification/mod.rs]
    ImportedBy: []
  # --- High Fan-In Hotspots ---
  diagnostics/mod.rs:
    Imports: [diagnostics/tests/mod.rs, source_graph.rs]
    ImportedBy: [diagnostics/tests/integration.rs, lib.rs, proof.rs, release/mod.rs, unit.rs]
  emitter/mod.rs:
    Imports: [emitter/css.rs, emitter/html.rs, emitter/tests/mod.rs, ir/mod.rs]
    ImportedBy: [emitter/tests/integration.rs, lib.rs, release/mod.rs, tests/css.rs, tests/html.rs, verification/mod.rs]
  extractor/mod.rs:
    Imports: [extractor/map.rs, extractor/tests/mod.rs, ir/mod.rs, layout/mod.rs, source_graph.rs]
    ImportedBy: [emitter/tests/integration.rs, extractor/tests/integration.rs, lib.rs, release/mod.rs, tests/map.rs, verification/mod.rs]
  ir/mod.rs:
    Imports: [ir/tests/mod.rs, layout.rs, paint.rs, typography.rs]
    ImportedBy: [emitter/css.rs, emitter/html.rs, emitter/mod.rs, emitter/tests/integration.rs, extractor/map.rs, extractor/mod.rs, extractor/tests/integration.rs, ir/tests/roundtrip.rs, lib.rs, proof.rs, release/mod.rs, tests/css.rs, tests/html.rs, tests/map.rs, verification/mod.rs]
  layout/mod.rs:
    Imports: [layout/model.rs, layout/tests/mod.rs, resolver/mod.rs]
    ImportedBy: [emitter/tests/integration.rs, extractor/map.rs, extractor/mod.rs, extractor/tests/integration.rs, geometry.rs, layout/tests/integration.rs, lib.rs, paint.rs, release/mod.rs, tests/map.rs, verification/mod.rs]
  resolver/mod.rs:
    Imports: [resolver/cascade.rs, resolver/model.rs, resolver/selector.rs, resolver/tests/mod.rs]
    ImportedBy: [emitter/tests/integration.rs, extractor/map.rs, extractor/tests/integration.rs, geometry.rs, layout/mod.rs, layout/model.rs, layout/tests/integration.rs, lib.rs, release/mod.rs, resolver/tests/integration.rs, tests/cascade.rs, tests/map.rs, tests/selector.rs, verification/mod.rs]
  resolver/model.rs:
    Imports: []
    ImportedBy: [geometry.rs, parse.rs, resolver/mod.rs, tests/cascade.rs, tests/map.rs]
  source_graph.rs:
    Imports: [fixture_manifest.rs, html_support.rs, source_graph/css.rs, source_graph/html.rs, source_graph/model.rs, tests.rs]
    ImportedBy: [diagnostics/mod.rs, diagnostics/tests/integration.rs, emitter/tests/integration.rs, extractor/mod.rs, extractor/tests/integration.rs, fixture_manifest.rs, layout/tests/integration.rs, lib.rs, proof.rs, release/mod.rs, resolver/tests/integration.rs, tests.rs, unit.rs, verification/mod.rs, verification/tests/roundtrip.rs]
  verification/mod.rs:
    Imports: [emitter/mod.rs, extractor/mod.rs, ir/mod.rs, layout/mod.rs, resolver/mod.rs, source_graph.rs, verification/tests/mod.rs]
    ImportedBy: [lib.rs, release/mod.rs, verification/tests/roundtrip.rs]
  # --- Layer 0 -- Config ---
  Cargo.toml, SEMMAP.md, neti.toml:
    Imports: []
    ImportedBy: []
  # --- Layer 1 -- Domain (Engine) ---
  emitter/css.rs:
    Imports: [ir/mod.rs]
    ImportedBy: [emitter/mod.rs, tests/css.rs]
  emitter/html.rs:
    Imports: [ir/mod.rs]
    ImportedBy: [emitter/mod.rs, tests/html.rs]
  extractor/map.rs:
    Imports: [ir/mod.rs, layout/mod.rs, paint.rs, resolver/mod.rs]
    ImportedBy: [extractor/mod.rs, tests/map.rs]
  fixture_manifest.rs:
    Imports: [parse.rs, source_graph.rs]
    ImportedBy: [lib.rs, source_graph.rs, source_graph/model.rs]
  html_support.rs:
    Imports: []
    ImportedBy: [source_graph.rs, source_graph/html.rs]
  layout.rs, typography.rs:
    Imports: []
    ImportedBy: [ir/mod.rs]
  layout/model.rs:
    Imports: [resolver/mod.rs]
    ImportedBy: [layout/mod.rs]
  paint.rs:
    Imports: [layout/mod.rs]
    ImportedBy: [extractor/map.rs, ir/mod.rs, tests/css.rs]
  resolver/cascade.rs:
    Imports: []
    ImportedBy: [resolver/mod.rs, tests/cascade.rs]
  resolver/selector.rs:
    Imports: []
    ImportedBy: [resolver/mod.rs]
  source_graph/css.rs:
    Imports: []
    ImportedBy: [source_graph.rs, tests.rs]
  source_graph/html.rs:
    Imports: [html_support.rs]
    ImportedBy: [source_graph.rs]
  source_graph/model.rs:
    Imports: [fixture_manifest.rs]
    ImportedBy: [source_graph.rs]
  # --- Layer 2 -- Adapters / Infra ---
  parse.rs:
    Imports: [resolver/model.rs]
    ImportedBy: [fixture_manifest.rs]
  # --- Layer 3 -- App / Entrypoints ---
  release/mod.rs:
    Imports: [diagnostics/mod.rs, emitter/mod.rs, extractor/mod.rs, ir/mod.rs, layout/mod.rs, release/tests/mod.rs, resolver/mod.rs, source_graph.rs, verification/mod.rs]
    ImportedBy: [lib.rs, proof.rs]
  # --- Tests ---
  contract.rs:
    Imports: []
    ImportedBy: [ir/tests/mod.rs]
  diagnostics/tests/integration.rs, unit.rs:
    Imports: [diagnostics/mod.rs, source_graph.rs]
    ImportedBy: [diagnostics/tests/mod.rs]
  diagnostics/tests/mod.rs:
    Imports: [diagnostics/tests/integration.rs, unit.rs]
    ImportedBy: [diagnostics/mod.rs]
  emitter/tests/integration.rs:
    Imports: [emitter/mod.rs, extractor/mod.rs, ir/mod.rs, layout/mod.rs, resolver/mod.rs, source_graph.rs]
    ImportedBy: [emitter/tests/mod.rs]
  emitter/tests/mod.rs:
    Imports: [emitter/tests/integration.rs, tests/css.rs, tests/html.rs]
    ImportedBy: [emitter/mod.rs]
  extractor/tests/integration.rs:
    Imports: [extractor/mod.rs, ir/mod.rs, layout/mod.rs, resolver/mod.rs, source_graph.rs]
    ImportedBy: [extractor/tests/mod.rs]
  extractor/tests/mod.rs:
    Imports: [extractor/tests/integration.rs, tests/map.rs]
    ImportedBy: [extractor/mod.rs]
  geometry.rs:
    Imports: [layout/mod.rs, resolver/mod.rs, resolver/model.rs]
    ImportedBy: [layout/tests/mod.rs]
  ir/tests/mod.rs:
    Imports: [contract.rs, ir/tests/roundtrip.rs]
    ImportedBy: [ir/mod.rs]
  ir/tests/roundtrip.rs:
    Imports: [ir/mod.rs]
    ImportedBy: [ir/tests/mod.rs]
  layout/tests/integration.rs:
    Imports: [layout/mod.rs, resolver/mod.rs, source_graph.rs]
    ImportedBy: [layout/tests/mod.rs]
  layout/tests/mod.rs:
    Imports: [geometry.rs, layout/tests/integration.rs]
    ImportedBy: [layout/mod.rs]
  proof.rs:
    Imports: [diagnostics/mod.rs, ir/mod.rs, release/mod.rs, source_graph.rs]
    ImportedBy: [release/tests/mod.rs]
  release/tests/mod.rs:
    Imports: [proof.rs]
    ImportedBy: [release/mod.rs]
  resolver/tests/integration.rs:
    Imports: [resolver/mod.rs, source_graph.rs]
    ImportedBy: [resolver/tests/mod.rs]
  resolver/tests/mod.rs:
    Imports: [resolver/tests/integration.rs, tests/cascade.rs, tests/selector.rs]
    ImportedBy: [resolver/mod.rs]
  tests.rs:
    Imports: [source_graph.rs, source_graph/css.rs]
    ImportedBy: [source_graph.rs]
  tests/cascade.rs:
    Imports: [resolver/cascade.rs, resolver/mod.rs, resolver/model.rs]
    ImportedBy: [resolver/tests/mod.rs]
  tests/css.rs:
    Imports: [emitter/css.rs, emitter/mod.rs, ir/mod.rs, paint.rs]
    ImportedBy: [emitter/tests/mod.rs]
  tests/html.rs:
    Imports: [emitter/html.rs, emitter/mod.rs, ir/mod.rs]
    ImportedBy: [emitter/tests/mod.rs]
  tests/map.rs:
    Imports: [extractor/map.rs, extractor/mod.rs, ir/mod.rs, layout/mod.rs, resolver/mod.rs, resolver/model.rs]
    ImportedBy: [extractor/tests/mod.rs]
  tests/selector.rs:
    Imports: [resolver/mod.rs]
    ImportedBy: [resolver/tests/mod.rs]
  verification/tests/mod.rs:
    Imports: [verification/tests/roundtrip.rs]
    ImportedBy: [verification/mod.rs]
  verification/tests/roundtrip.rs:
    Imports: [source_graph.rs, verification/mod.rs]
    ImportedBy: [verification/tests/mod.rs]
```

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

`src/fixture_manifest.rs`
model for fixture manifest via file I/O. [COUPLING:mixed] [BEHAVIOR:persists,propagates-errors] [QUALITY:undocumented]
Exports: FixtureManifest.manifest_root, FixtureSceneEntry, FixtureManifestError, FixtureManifestError.fmt
Semantic: side-effecting adapter that propagates errors

`src/ir/layout.rs`
CSS `position` value supported in v0.1. [COUPLING:pure] [QUALITY:undocumented]
Exports: BoxSizing, FlexDirection, JustifyContent, AlignItems
Semantic: pure computation

`src/ir/paint.rs`
Implements Border functionality. [TYPE]
Exports: Border, Color, Paint

`src/ir/typography.rs`
Resolved CSS `line-height`.
Exports: LineHeight, Typography

`src/resolver/cascade.rs`
Parses color. [HOTSPOT] [COUPLING:pure]
Exports: apply_declaration, apply_inheritance, parse_color, parse_px
Semantic: pure computation

`src/resolver/model.rs`
Fully resolved CSS properties for a single HTML element node in the v0.1 subset. [TYPE] [HOTSPOT] [COUPLING:pure]
Exports: ComputedStyle, ComputedStyle.default
Semantic: pure computation

`src/resolver/selector.rs`
CSS specificity for the v0.1 subset: (class_count, type_count). [COUPLING:pure]
Exports: selector_matches, Specificity, specificity
Semantic: pure computation

`src/source_graph.rs`
utility for source graph via file I/O. [HOTSPOT] [GLOBAL-UTIL] [COUPLING:mixed] [BEHAVIOR:persists,propagates-errors] [QUALITY:error-boundary]
Exports: load_scene_source_graph, SceneSourceGraph.load
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

`src/ir/mod.rs`
SEMUI v0.1 Intermediate Representation schema and serialization contract. [TYPE] [COUPLING:pure] [QUALITY:undocumented]
Exports: ExecutionMode, ControlKind, SceneIr, SceneIr.from_json
Touch: Contains inline Rust tests alongside runtime code.
Semantic: pure computation

`src/lib.rs`
Implements ir functionality. [ENTRY]
Exports: ir, resolver

`src/resolver/mod.rs`
Static-scene style resolver for the SEMUI v0.1 subset. [ENTRY] [HOTSPOT] [COUPLING:pure]
Exports: ComputedStyle, ResolvedNode, ResolverError, ResolverError.fmt
Touch: Contains inline Rust tests alongside runtime code.
Semantic: pure computation

## Layer 4 -- Tests

`src/ir/tests/contract.rs`
Tests for super. [COUPLING:pure]
Semantic: pure computation

`src/ir/tests/mod.rs`
Tests for super. [ENTRY] [COUPLING:pure]
Semantic: pure computation

`src/ir/tests/roundtrip.rs`
Exercises every field in IrNode: box, control, and text nodes with full layout, paint, typography, and provenance spans. [COUPLING:pure]
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


## DependencyGraph

```yaml
DependencyGraph:
  # --- Entrypoints ---
  lib.rs:
    Imports: [fixture_manifest.rs, ir/mod.rs, resolver/mod.rs, source_graph.rs]
    ImportedBy: []
  # --- High Fan-In Hotspots ---
  resolver/mod.rs:
    Imports: [resolver/cascade.rs, resolver/model.rs, resolver/selector.rs, resolver/tests/mod.rs]
    ImportedBy: [integration.rs, lib.rs, tests/cascade.rs, tests/selector.rs]
  resolver/model.rs:
    Imports: []
    ImportedBy: [parse.rs, resolver/mod.rs, tests/cascade.rs]
  source_graph.rs:
    Imports: [css.rs, fixture_manifest.rs, html.rs, html_support.rs, source_graph/model.rs, tests.rs]
    ImportedBy: [fixture_manifest.rs, integration.rs, lib.rs, tests.rs]
  # --- Layer 0 -- Config ---
  Cargo.toml, SEMMAP.md, neti.toml:
    Imports: []
    ImportedBy: []
  # --- Layer 1 -- Domain (Engine) ---
  css.rs:
    Imports: []
    ImportedBy: [source_graph.rs, tests.rs]
  fixture_manifest.rs:
    Imports: [parse.rs, source_graph.rs]
    ImportedBy: [lib.rs, source_graph.rs, source_graph/model.rs]
  html.rs:
    Imports: [html_support.rs]
    ImportedBy: [source_graph.rs]
  html_support.rs:
    Imports: []
    ImportedBy: [html.rs, source_graph.rs]
  layout.rs, paint.rs, typography.rs:
    Imports: []
    ImportedBy: [ir/mod.rs]
  resolver/cascade.rs:
    Imports: []
    ImportedBy: [resolver/mod.rs, tests/cascade.rs]
  resolver/selector.rs:
    Imports: []
    ImportedBy: [resolver/mod.rs]
  source_graph/model.rs:
    Imports: [fixture_manifest.rs]
    ImportedBy: [source_graph.rs]
  # --- Layer 2 -- Adapters / Infra ---
  parse.rs:
    Imports: [resolver/model.rs]
    ImportedBy: [fixture_manifest.rs]
  # --- Layer 3 -- App / Entrypoints ---
  ir/mod.rs:
    Imports: [ir/tests/mod.rs, layout.rs, paint.rs, typography.rs]
    ImportedBy: [lib.rs, roundtrip.rs]
  # --- Tests ---
  contract.rs:
    Imports: []
    ImportedBy: [ir/tests/mod.rs]
  integration.rs:
    Imports: [resolver/mod.rs, source_graph.rs]
    ImportedBy: [resolver/tests/mod.rs]
  ir/tests/mod.rs:
    Imports: [contract.rs, roundtrip.rs]
    ImportedBy: [ir/mod.rs]
  resolver/tests/mod.rs:
    Imports: [integration.rs, tests/cascade.rs, tests/selector.rs]
    ImportedBy: [resolver/mod.rs]
  roundtrip.rs:
    Imports: [ir/mod.rs]
    ImportedBy: [ir/tests/mod.rs]
  tests.rs:
    Imports: [css.rs, source_graph.rs]
    ImportedBy: [source_graph.rs]
  tests/cascade.rs:
    Imports: [resolver/cascade.rs, resolver/mod.rs, resolver/model.rs]
    ImportedBy: [resolver/tests/mod.rs]
  tests/selector.rs:
    Imports: [resolver/mod.rs]
    ImportedBy: [resolver/tests/mod.rs]
```

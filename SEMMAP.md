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

`advice.md`
Support file for advice.

`concept.md`
Support file for concept.

## Layer 1 -- Domain (Engine)

`src/fixture_manifest.rs`
model for fixture manifest via file I/O. [COUPLING:mixed] [BEHAVIOR:persists,propagates-errors] [QUALITY:undocumented]
Exports: FixtureManifest.manifest_root, FixtureSceneEntry, FixtureManifestError, FixtureManifestError.fmt
Semantic: side-effecting adapter that propagates errors

`src/source_graph.rs`
model for source graph via file I/O. [TYPE] [COUPLING:mixed] [BEHAVIOR:persists,panics-on-error,propagates-errors] [QUALITY:undocumented,complex-flow,error-boundary]
Exports: load_scene_source_graph, HtmlNodeKind, SceneSourceGraph.load, SourceDocumentKind
Touch: Contains inline Rust tests alongside runtime code.
Semantic: side-effecting adapter that panics on error

## Layer 3 -- App / Entrypoints

`src/lib.rs`
Library root and public exports. [ENTRY]

## Layer 4 -- Tests

`test.md`
Support file for test.


## DependencyGraph

```yaml
DependencyGraph:
  # --- Entrypoints ---
  lib.rs:
    Imports: [fixture_manifest.rs, source_graph.rs]
    ImportedBy: []
  # --- Layer 0 -- Config ---
  Cargo.toml, advice.md, concept.md:
    Imports: []
    ImportedBy: []
  # --- Layer 1 -- Domain (Engine) ---
  fixture_manifest.rs:
    Imports: [source_graph.rs]
    ImportedBy: [lib.rs, source_graph.rs]
  source_graph.rs:
    Imports: [fixture_manifest.rs]
    ImportedBy: [fixture_manifest.rs, lib.rs]
  # --- Tests ---
  test.md:
    Imports: []
    ImportedBy: []
```

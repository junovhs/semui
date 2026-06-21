pub mod diagnostics;
pub mod emitter;
pub mod extractor;
mod fixture_manifest;
pub mod ir;
pub mod layout;
pub mod release;
pub mod resolver;
mod source_graph;
pub mod target;
pub mod verification;

pub use fixture_manifest::{
    FixtureManifest, FixtureManifestError, FixtureScene, FixtureSceneEntry, load_fixture_manifest,
};
pub use source_graph::{
    CssDeclaration, CssRule, HtmlNode, HtmlNodeKind, SceneSourceGraph, SourceDocument,
    SourceDocumentKind, TraceSpan, load_scene_source_graph,
};

//! SEMUI v0.1 Intermediate Representation schema and serialization contract.
//!
//! This module defines the canonical IR for the v0.1 round-trip proof:
//! `HTML/CSS → SceneIr → HTML/CSS`. All extraction (`EXT-01`) and emission
//! (`EMIT-01`) work against this single contract. The JSON serialization of
//! [`SceneIr`] is the golden artifact format referenced in
//! `fixtures/v0.1/manifest.toml` as `expected_semui`.
//!
//! # Schema stability
//!
//! `schema_version = 1` covers this exact field set. Any additive or breaking
//! change must bump the version and be tracked as a new issue.

pub mod layout;
pub mod paint;
pub mod typography;

pub use layout::{
    AlignItems, AlignSelf, BoxSizing, Cursor, Display, EdgeInset, FlexDirection, JustifyContent,
    Layout, Position,
};
pub use paint::{Border, Color, Paint};
pub use typography::{LineHeight, Typography};

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Execution mode
// ---------------------------------------------------------------------------

/// Execution context for a scene. `v0.1` only supports [`ExecutionMode::Static`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionMode {
    /// No JavaScript, no responsive breakpoints, no animation. Layout is
    /// fully determined by HTML and CSS alone.
    Static,
}

// ---------------------------------------------------------------------------
// Provenance
// ---------------------------------------------------------------------------

/// Provenance pointer from an [`IrNode`] back to its origin in the source graph.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceRef {
    /// Index into the `SceneSourceGraph` document list (HTML = 0, CSS = 1 by convention).
    pub doc_id: usize,
    /// Selector-style path to the element, e.g. `"body > div.card > span"`.
    pub dom_path: String,
    /// Byte-level location in the source document. `None` when span tracking
    /// was not performed (e.g. for synthetic nodes).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub span: Option<SourceSpan>,
}

/// Byte-level location in a source document.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceSpan {
    pub start: usize,
    pub end: usize,
    pub line: usize,
    pub column: usize,
}

// ---------------------------------------------------------------------------
// Node kind
// ---------------------------------------------------------------------------

/// Coarse semantic kind of an IR node.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NodeKind {
    /// Generic container element (e.g. `<div>`, `<span>`).
    Box,
    /// Leaf text run. Always has `text_content` set.
    Text,
    /// Native UI control. Always has `control_kind` set.
    Control,
}

/// Specific kind of a native UI control node.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ControlKind {
    Button,
}

// ---------------------------------------------------------------------------
// IR node and scene
// ---------------------------------------------------------------------------

/// A single resolved visual node in the SEMUI IR.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IrNode {
    /// Stable identifier unique within the scene, assigned in pre-order
    /// (e.g. `"n0"`, `"n1"`, `"n2"`).
    pub id: String,
    pub kind: NodeKind,
    /// `None` for the root node; otherwise the `id` of the parent [`IrNode`].
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<String>,
    /// Present only when `kind` is [`NodeKind::Control`].
    #[serde(skip_serializing_if = "Option::is_none")]
    pub control_kind: Option<ControlKind>,
    /// Verbatim text content. Present only when `kind` is [`NodeKind::Text`].
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_content: Option<String>,
    pub layout: Layout,
    pub paint: Paint,
    /// `None` on container boxes that carry no direct text content.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub typography: Option<Typography>,
    /// Provenance pointer back to the source graph.
    pub source: SourceRef,
}

/// The top-level SEMUI IR for a single scene.
///
/// Serialize this to JSON to produce a `scene.semui.json` golden artifact as
/// required by `fixtures/v0.1/manifest.toml`. The `nodes` list is in pre-order
/// so a parent always appears before its children.
///
/// # Example
///
/// ```no_run
/// # use semui::ir::SceneIr;
/// let json = std::fs::read_to_string("scene.semui.json").unwrap();
/// let ir: SceneIr = SceneIr::from_json(&json).unwrap();
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SceneIr {
    /// Always `1` for the v0.1 schema. Bump on breaking changes.
    pub schema_version: u32,
    /// Matches the `id` field in `manifest.toml`, e.g. `"profile_card_absolute"`.
    pub scene_id: String,
    /// Corpus identifier, e.g. `"v0.1"`.
    pub corpus: String,
    pub execution_mode: ExecutionMode,
    /// Pre-order node list. Index 0 is always the scene root.
    pub nodes: Vec<IrNode>,
}

impl SceneIr {
    /// Serialize to pretty-printed JSON.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Deserialize from a JSON string.
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

#[cfg(test)]
mod tests;

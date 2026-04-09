use serde::{Deserialize, Serialize};

use super::paint::Color;

/// Resolved CSS `line-height`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum LineHeight {
    /// CSS `line-height: normal` — UA-defined, not resolved to a pixel value.
    Normal,
    /// Explicit pixel length, e.g. `{"kind":"length","value":24.0}`.
    Length { value: f32 },
}

/// Resolved typography properties for a node that carries visible text.
///
/// Present on [`super::NodeKind::Text`] nodes and [`super::NodeKind::Control`]
/// nodes whose label text is part of the visual contract.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Typography {
    /// Ordered CSS font-family fallback list, e.g. `["Inter", "sans-serif"]`.
    pub font_family: Vec<String>,
    /// Resolved `font-size` in CSS pixels.
    pub font_size: f32,
    /// CSS `font-weight` as an integer (e.g. `400`, `700`).
    pub font_weight: u16,
    pub line_height: LineHeight,
    /// Resolved foreground text color (`color` property).
    pub color: Color,
}

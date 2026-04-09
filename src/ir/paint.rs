use serde::{Deserialize, Serialize};

use super::layout::Cursor;

/// A normalized hex color string, e.g. `"#1a2b3c"` or `"#ffffff"`.
///
/// Values are stored in lowercase with a leading `#`. Callers are responsible
/// for normalizing before comparison; the struct itself does no canonicalization.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Color(pub String);

/// Uniform border applied to all four sides of a box (v0.1 does not track
/// per-side borders separately).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Border {
    /// Border width in CSS pixels.
    pub width: f32,
    pub color: Color,
}

/// Resolved paint properties for an [`super::IrNode`].
///
/// Foreground text color is on [`super::Typography`], not here. `Paint` covers
/// the box-level visual surface.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Paint {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub background_color: Option<Color>,
    /// Uniform border. `None` means no visible border.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub border: Option<Border>,
    /// CSS `border-radius` in pixels (uniform corner radius).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub border_radius: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<Cursor>,
}

use crate::HtmlNode;
use crate::resolver::ComputedStyle;

/// Explicit geometry for a single element node in the v0.1 subset.
///
/// All lengths are in CSS pixels. `None` means the dimension could not be
/// determined from CSS alone (content-driven or unspecified).
///
/// Margin and padding are stored as `[top, right, bottom, left]`.
#[derive(Debug, Clone, PartialEq)]
pub struct Geometry {
    /// Resolved x-position for `position: absolute` elements (CSS `left`).
    /// Always `None` for in-flow and flex-item elements in v0.1.
    pub explicit_x: Option<f32>,
    /// Resolved y-position for `position: absolute` elements (CSS `top`).
    /// Always `None` for in-flow and flex-item elements in v0.1.
    pub explicit_y: Option<f32>,
    /// Declared CSS `width` in pixels.
    pub width: Option<f32>,
    /// Declared CSS `height` in pixels.
    pub height: Option<f32>,
    /// Declared CSS `min-width` in pixels.
    pub min_width: Option<f32>,
    /// Inner content-box width after subtracting padding and border (border-box).
    /// Equal to `width` for content-box elements.
    pub content_width: Option<f32>,
    /// Inner content-box height after subtracting padding and border (border-box).
    /// Equal to `height` for content-box elements.
    pub content_height: Option<f32>,
    /// Margin edges: `[top, right, bottom, left]` in pixels.
    pub margin: [f32; 4],
    /// Padding edges: `[top, right, bottom, left]` in pixels.
    pub padding: [f32; 4],
    /// Uniform border width in pixels.
    pub border_width: f32,
    /// Uniform border radius in pixels.
    pub border_radius: f32,
}

/// A single element node with its resolved CSS properties and explicit geometry.
#[derive(Debug, Clone)]
pub struct LaidOutNode {
    pub node: HtmlNode,
    pub style: ComputedStyle,
    pub geometry: Geometry,
}

/// A fully laid-out scene: all element nodes in document order, each with
/// their computed style and explicit geometry.
#[derive(Debug, Clone)]
pub struct LaidOutScene {
    pub scene_id: String,
    pub nodes: Vec<LaidOutNode>,
}

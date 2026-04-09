use serde::{Deserialize, Serialize};

/// CSS `position` value supported in v0.1.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Position {
    Static,
    Absolute,
}

/// CSS `display` value supported in v0.1.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Display {
    Block,
    Flex,
    /// CSS `inline-flex`.
    InlineFlex,
}

/// CSS `box-sizing` value.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BoxSizing {
    BorderBox,
    ContentBox,
}

/// CSS `flex-direction` value supported in v0.1.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FlexDirection {
    Row,
    Column,
}

/// CSS `align-items` value supported in v0.1.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AlignItems {
    Center,
}

/// CSS `justify-content` value supported in v0.1.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JustifyContent {
    Center,
}

/// CSS `align-self` value supported in v0.1.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AlignSelf {
    Auto,
    FlexStart,
}

/// CSS `cursor` value supported in v0.1.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Cursor {
    Auto,
    Pointer,
}

/// Resolved pixel lengths for all four sides of a box model edge.
///
/// All values are in CSS pixels with no unit suffix. Default is zero on all sides.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EdgeInset {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

impl EdgeInset {
    /// Zero inset on all sides.
    pub const fn zero() -> Self {
        Self {
            top: 0.0,
            right: 0.0,
            bottom: 0.0,
            left: 0.0,
        }
    }

    /// Uniform inset on all sides.
    pub fn uniform(px: f32) -> Self {
        Self {
            top: px,
            right: px,
            bottom: px,
            left: px,
        }
    }
}

/// Resolved layout properties for an [`super::IrNode`].
///
/// All lengths are in CSS pixels. `None` means the property was not explicitly
/// set; the downstream emitter should apply its own default.
///
/// Flex properties are `None` on non-flex nodes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Layout {
    pub position: Position,
    pub display: Display,
    pub box_sizing: BoxSizing,
    /// CSS `top`, present only when `position` is [`Position::Absolute`].
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top: Option<f32>,
    /// CSS `left`, present only when `position` is [`Position::Absolute`].
    #[serde(skip_serializing_if = "Option::is_none")]
    pub left: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_width: Option<f32>,
    pub margin: EdgeInset,
    pub padding: EdgeInset,
    /// Present only when `display` is [`Display::Flex`] or [`Display::InlineFlex`].
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flex_direction: Option<FlexDirection>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub align_items: Option<AlignItems>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub justify_content: Option<JustifyContent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub align_self: Option<AlignSelf>,
    /// CSS `gap` in pixels, present only on flex containers.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gap: Option<f32>,
}

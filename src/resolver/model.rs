/// Fully resolved CSS properties for a single HTML element node in the v0.1 subset.
///
/// All lengths are in CSS pixels (`f32`). Colors are normalized lowercase hex strings.
/// Inherited properties that were not explicitly set carry the parent's resolved value.
/// `None` for optional properties means the property was neither declared nor inherited.
#[derive(Debug, Clone, PartialEq)]
pub struct ComputedStyle {
    // --- Position / layout ---
    /// CSS `position`. Initial value: `"static"`.
    pub position: String,
    /// CSS `display`. Initial value: `"block"`.
    pub display: String,
    /// CSS `box-sizing`. Initial value: `"content-box"`.
    pub box_sizing: String,
    /// CSS `top` in pixels. Present only when `position` is `"absolute"`.
    pub top: Option<f32>,
    /// CSS `left` in pixels. Present only when `position` is `"absolute"`.
    pub left: Option<f32>,
    pub width: Option<f32>,
    pub height: Option<f32>,
    pub min_width: Option<f32>,

    // --- Box model edges ---
    pub margin_top: f32,
    pub margin_right: f32,
    pub margin_bottom: f32,
    pub margin_left: f32,
    pub padding_top: f32,
    pub padding_right: f32,
    pub padding_bottom: f32,
    pub padding_left: f32,

    // --- Flex ---
    pub flex_direction: Option<String>,
    pub align_items: Option<String>,
    pub justify_content: Option<String>,
    pub align_self: Option<String>,
    pub gap: Option<f32>,

    // --- Paint ---
    pub background_color: Option<String>,
    pub border_width: f32,
    pub border_color: Option<String>,
    pub border_radius: f32,
    pub cursor: Option<String>,

    // --- Typography — inheritable ---
    pub color: Option<String>,
    pub font_family: Option<String>,
    pub font_size: Option<f32>,
    pub font_weight: Option<u16>,
    /// Raw CSS `line-height` value, e.g. `"normal"` or `"24px"`.
    pub line_height: Option<String>,
}

impl Default for ComputedStyle {
    fn default() -> Self {
        Self {
            position: "static".to_owned(),
            display: "block".to_owned(),
            box_sizing: "content-box".to_owned(),
            top: None,
            left: None,
            width: None,
            height: None,
            min_width: None,
            margin_top: 0.0,
            margin_right: 0.0,
            margin_bottom: 0.0,
            margin_left: 0.0,
            padding_top: 0.0,
            padding_right: 0.0,
            padding_bottom: 0.0,
            padding_left: 0.0,
            flex_direction: None,
            align_items: None,
            justify_content: None,
            align_self: None,
            gap: None,
            background_color: None,
            border_width: 0.0,
            border_color: None,
            border_radius: 0.0,
            cursor: None,
            color: None,
            font_family: None,
            font_size: None,
            font_weight: None,
            line_height: None,
        }
    }
}

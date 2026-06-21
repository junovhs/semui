use super::model::ComputedStyle;

/// Apply a single CSS declaration to `style`, expanding shorthands in place.
///
/// Unknown or unsupported properties are silently ignored; diagnostics are
/// the responsibility of `DIA-01`. The `inherit` keyword is skipped here
/// and handled by the inheritance pass in `resolve_scene`.
pub fn apply_declaration(style: &mut ComputedStyle, property: &str, value: &str) {
    let value = value.trim();
    if value == "inherit" {
        return;
    }

    match property {
        // --- Shorthands ---
        "margin" => expand_edges(
            &mut style.margin_top,
            &mut style.margin_right,
            &mut style.margin_bottom,
            &mut style.margin_left,
            value,
        ),
        "padding" => expand_edges(
            &mut style.padding_top,
            &mut style.padding_right,
            &mut style.padding_bottom,
            &mut style.padding_left,
            value,
        ),
        "background" => style.background_color = parse_color(value),
        "border" => expand_border(style, value),
        // `appearance: none` is an accepted source-level reset. It has no IR
        // field because controls are always emitted with the canonical reset.
        "appearance" => {}

        // --- Position / layout longhands ---
        "position" => style.position = value.to_owned(),
        "display" => style.display = value.to_owned(),
        "box-sizing" => style.box_sizing = value.to_owned(),
        "top" => style.top = parse_px(value),
        "left" => style.left = parse_px(value),
        "width" => style.width = parse_px(value),
        "height" => style.height = parse_px(value),
        "min-width" => style.min_width = parse_px(value),

        // --- Box model longhands ---
        "margin-top" => style.margin_top = parse_px(value).unwrap_or(0.0),
        "margin-right" => style.margin_right = parse_px(value).unwrap_or(0.0),
        "margin-bottom" => style.margin_bottom = parse_px(value).unwrap_or(0.0),
        "margin-left" => style.margin_left = parse_px(value).unwrap_or(0.0),
        "padding-top" => style.padding_top = parse_px(value).unwrap_or(0.0),
        "padding-right" => style.padding_right = parse_px(value).unwrap_or(0.0),
        "padding-bottom" => style.padding_bottom = parse_px(value).unwrap_or(0.0),
        "padding-left" => style.padding_left = parse_px(value).unwrap_or(0.0),

        // --- Flex ---
        "flex-direction" => style.flex_direction = Some(value.to_owned()),
        "align-items" => style.align_items = Some(value.to_owned()),
        "justify-content" => style.justify_content = Some(value.to_owned()),
        "align-self" => style.align_self = Some(value.to_owned()),
        "gap" => style.gap = parse_px(value),

        // --- Paint longhands ---
        "background-color" => style.background_color = parse_color(value),
        "border-width" => style.border_width = parse_px(value).unwrap_or(0.0),
        "border-color" => style.border_color = parse_color(value),
        "border-radius" => style.border_radius = parse_px(value).unwrap_or(0.0),
        "cursor" => style.cursor = Some(value.to_owned()),

        // --- Typography — inheritable ---
        "color" => style.color = parse_color(value),
        "font-family" => style.font_family = Some(value.to_owned()),
        "font-size" => style.font_size = parse_px(value),
        "font-weight" => style.font_weight = value.parse::<u16>().ok(),
        "line-height" => style.line_height = Some(value.to_owned()),

        // Unknown properties are silently ignored (DIA-01 handles diagnostics)
        _ => {}
    }
}

/// Propagate inheritable typography fields from `parent` into `child` for any
/// field that is still `None` after cascade.
pub fn apply_inheritance(child: &mut ComputedStyle, parent: &ComputedStyle) {
    if child.color.is_none() {
        child.color = parent.color.clone();
    }
    if child.font_family.is_none() {
        child.font_family = parent.font_family.clone();
    }
    if child.font_size.is_none() {
        child.font_size = parent.font_size;
    }
    if child.font_weight.is_none() {
        child.font_weight = parent.font_weight;
    }
    if child.line_height.is_none() {
        child.line_height = parent.line_height.clone();
    }
}

// ---------------------------------------------------------------------------
// Value parsers
// ---------------------------------------------------------------------------

/// Parse a CSS pixel length into an `f32`. Accepts `"0"`, `"8px"`, `"320px"`.
/// Returns `None` for non-pixel values or parse failures.
pub fn parse_px(value: &str) -> Option<f32> {
    let s = value.trim();
    if s == "0" {
        return Some(0.0);
    }
    s.strip_suffix("px")
        .and_then(|n| n.trim().parse::<f32>().ok())
}

/// Normalize a hex color string to lowercase. Returns `None` for non-hex values.
pub fn parse_color(value: &str) -> Option<String> {
    let s = value.trim();
    if s.starts_with('#') {
        Some(s.to_lowercase())
    } else {
        None
    }
}

// ---------------------------------------------------------------------------
// Shorthand helpers
// ---------------------------------------------------------------------------

fn expand_edges(top: &mut f32, right: &mut f32, bottom: &mut f32, left: &mut f32, value: &str) {
    let parts: Vec<f32> = value.split_whitespace().filter_map(parse_px).collect();
    match parts.as_slice() {
        [all] => {
            *top = *all;
            *right = *all;
            *bottom = *all;
            *left = *all;
        }
        [tb, lr] => {
            *top = *tb;
            *right = *lr;
            *bottom = *tb;
            *left = *lr;
        }
        [t, lr, b] => {
            *top = *t;
            *right = *lr;
            *bottom = *b;
            *left = *lr;
        }
        [t, r, b, l] => {
            *top = *t;
            *right = *r;
            *bottom = *b;
            *left = *l;
        }
        _ => {}
    }
}

fn expand_border(style: &mut ComputedStyle, value: &str) {
    let s = value.trim();
    if s == "0" || s == "none" {
        style.border_width = 0.0;
        style.border_color = None;
        return;
    }
    for part in s.split_whitespace() {
        if let Some(px) = parse_px(part) {
            style.border_width = px;
        } else if let Some(color) = parse_color(part) {
            style.border_color = Some(color);
        }
        // "solid", "dashed", border-style keywords are ignored for v0.1
    }
}

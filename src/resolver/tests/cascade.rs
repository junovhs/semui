use crate::resolver::cascade::{apply_declaration, apply_inheritance, parse_color, parse_px};
use crate::resolver::model::ComputedStyle;

// --- parse_px ---

#[test]
fn parse_px_handles_zero_without_unit() {
    assert_eq!(parse_px("0"), Some(0.0));
}

#[test]
fn parse_px_handles_pixel_values() {
    assert_eq!(parse_px("8px"), Some(8.0));
    assert_eq!(parse_px("320px"), Some(320.0));
    assert_eq!(parse_px("0px"), Some(0.0));
}

#[test]
fn parse_px_returns_none_for_non_pixel() {
    assert_eq!(parse_px("auto"), None);
    assert_eq!(parse_px("normal"), None);
    assert_eq!(parse_px("#fff"), None);
}

// --- parse_color ---

#[test]
fn parse_color_normalizes_hex_to_lowercase() {
    assert_eq!(parse_color("#F3F4F6"), Some("#f3f4f6".to_owned()));
    assert_eq!(parse_color("#fff"), Some("#fff".to_owned()));
}

#[test]
fn parse_color_returns_none_for_non_hex() {
    assert_eq!(parse_color("red"), None);
    assert_eq!(parse_color("transparent"), None);
    assert_eq!(parse_color("none"), None);
}

// --- apply_declaration: longhands ---

#[test]
fn apply_position_sets_field() {
    let mut s = ComputedStyle::default();
    apply_declaration(&mut s, "position", "absolute");
    assert_eq!(s.position, "absolute");
}

#[test]
fn apply_display_sets_field() {
    let mut s = ComputedStyle::default();
    apply_declaration(&mut s, "display", "flex");
    assert_eq!(s.display, "flex");
}

#[test]
fn apply_color_normalizes_hex() {
    let mut s = ComputedStyle::default();
    apply_declaration(&mut s, "color", "#1D4ED8");
    assert_eq!(s.color, Some("#1d4ed8".to_owned()));
}

#[test]
fn apply_font_weight_parses_integer() {
    let mut s = ComputedStyle::default();
    apply_declaration(&mut s, "font-weight", "600");
    assert_eq!(s.font_weight, Some(600));
}

#[test]
fn apply_inherit_is_skipped() {
    let mut s = ComputedStyle::default();
    apply_declaration(&mut s, "font-family", "inherit");
    assert_eq!(s.font_family, None, "inherit must not set a value; inheritance handles it");
}

// --- apply_declaration: margin shorthand expansion ---

#[test]
fn margin_one_value_sets_all_sides() {
    let mut s = ComputedStyle::default();
    apply_declaration(&mut s, "margin", "8px");
    assert_eq!((s.margin_top, s.margin_right, s.margin_bottom, s.margin_left), (8.0, 8.0, 8.0, 8.0));
}

#[test]
fn margin_two_values_sets_tb_lr() {
    let mut s = ComputedStyle::default();
    apply_declaration(&mut s, "margin", "8px 16px");
    assert_eq!((s.margin_top, s.margin_right, s.margin_bottom, s.margin_left), (8.0, 16.0, 8.0, 16.0));
}

#[test]
fn padding_four_values_set_individually() {
    let mut s = ComputedStyle::default();
    apply_declaration(&mut s, "padding", "4px 8px 12px 16px");
    assert_eq!((s.padding_top, s.padding_right, s.padding_bottom, s.padding_left), (4.0, 8.0, 12.0, 16.0));
}

// --- apply_declaration: border shorthand expansion ---

#[test]
fn border_zero_clears_width_and_color() {
    let mut s = ComputedStyle {
        border_width: 2.0,
        border_color: Some("#000000".to_owned()),
        ..ComputedStyle::default()
    };
    apply_declaration(&mut s, "border", "0");
    assert_eq!(s.border_width, 0.0);
    assert_eq!(s.border_color, None);
}

#[test]
fn border_shorthand_extracts_width_and_color() {
    let mut s = ComputedStyle::default();
    apply_declaration(&mut s, "border", "1px solid #e5e7eb");
    assert_eq!(s.border_width, 1.0);
    assert_eq!(s.border_color, Some("#e5e7eb".to_owned()));
}

// --- apply_declaration: background shorthand ---

#[test]
fn background_shorthand_sets_background_color() {
    let mut s = ComputedStyle::default();
    apply_declaration(&mut s, "background", "#f3f4f6");
    assert_eq!(s.background_color, Some("#f3f4f6".to_owned()));
}

// --- apply_inheritance ---

#[test]
fn inheritance_copies_typography_from_parent() {
    let parent = ComputedStyle {
        color: Some("#111827".to_owned()),
        font_family: Some("Inter, sans-serif".to_owned()),
        font_size: Some(16.0),
        font_weight: Some(400),
        line_height: Some("24px".to_owned()),
        ..ComputedStyle::default()
    };
    let mut child = ComputedStyle::default();
    apply_inheritance(&mut child, &parent);
    assert_eq!(child.color, Some("#111827".to_owned()));
    assert_eq!(child.font_family, Some("Inter, sans-serif".to_owned()));
    assert_eq!(child.font_size, Some(16.0));
    assert_eq!(child.font_weight, Some(400));
    assert_eq!(child.line_height, Some("24px".to_owned()));
}

#[test]
fn inheritance_does_not_overwrite_explicitly_set_value() {
    let parent = ComputedStyle {
        color: Some("#111827".to_owned()),
        ..ComputedStyle::default()
    };
    let mut child = ComputedStyle {
        color: Some("#ffffff".to_owned()),
        ..ComputedStyle::default()
    };
    apply_inheritance(&mut child, &parent);
    assert_eq!(child.color, Some("#ffffff".to_owned()), "explicit child color must not be overwritten");
}

#[test]
fn inheritance_does_not_propagate_non_inherited_properties() {
    let parent = ComputedStyle {
        background_color: Some("#f0f0f0".to_owned()),
        border_width: 2.0,
        position: "absolute".to_owned(),
        ..ComputedStyle::default()
    };
    let mut child = ComputedStyle::default();
    apply_inheritance(&mut child, &parent);
    assert_eq!(child.background_color, None, "background-color must not inherit");
    assert_eq!(child.border_width, 0.0, "border-width must not inherit");
    assert_eq!(child.position, "static", "position must not inherit");
}

//! Unit tests for the mapping layer (map.rs).

use crate::extractor::map;
use crate::ir::typography::LineHeight as TLineHeight;
use crate::resolver::ComputedStyle;
use crate::layout::{LaidOutScene, compute_layout};
use crate::resolver::{ResolvedNode, ResolvedScene};
use crate::HtmlNode;
use crate::HtmlNodeKind;

fn elem(id: usize) -> HtmlNode {
    HtmlNode {
        id,
        parent_id: None,
        kind: HtmlNodeKind::Element,
        name: Some("div".to_owned()),
        text: None,
        attributes: Vec::new(),
        dom_path: format!("0/{id}"),
        document_id: 0,
    }
}

fn laid_out_single(style: ComputedStyle) -> LaidOutScene {
    let scene = ResolvedScene {
        scene_id: "test".to_owned(),
        nodes: vec![ResolvedNode { node: elem(0), style }],
    };
    compute_layout(&scene)
}

// --- to_layout ---

#[test]
fn to_layout_absolute_position() {
    let style = ComputedStyle {
        position: "absolute".to_owned(),
        left: Some(10.0),
        top: Some(20.0),
        width: Some(100.0),
        height: Some(50.0),
        ..ComputedStyle::default()
    };
    let scene = laid_out_single(style.clone());
    let geo = &scene.nodes[0].geometry;
    let layout = map::to_layout(&style, geo);
    assert_eq!(layout.left, Some(10.0));
    assert_eq!(layout.top, Some(20.0));
    assert_eq!(layout.width, Some(100.0));
    assert_eq!(layout.height, Some(50.0));
}

#[test]
fn to_layout_static_has_no_explicit_coords() {
    let style = ComputedStyle {
        position: "static".to_owned(),
        left: Some(5.0),
        top: Some(5.0),
        ..ComputedStyle::default()
    };
    let scene = laid_out_single(style.clone());
    let geo = &scene.nodes[0].geometry;
    let layout = map::to_layout(&style, geo);
    assert_eq!(layout.left, None);
    assert_eq!(layout.top, None);
}

// --- to_paint ---

#[test]
fn to_paint_border_emitted_when_width_nonzero() {
    let style = ComputedStyle {
        border_width: 2.0,
        border_color: Some("#ff0000".to_owned()),
        ..ComputedStyle::default()
    };
    let paint = map::to_paint(&style);
    assert!(paint.border.is_some(), "border should be Some when width > 0");
    assert_eq!(paint.border.as_ref().map(|b| b.width),        Some(2.0));
    assert_eq!(paint.border.as_ref().map(|b| b.color.0.as_str()), Some("#ff0000"));
}

#[test]
fn to_paint_no_border_when_width_zero() {
    let style = ComputedStyle {
        border_width: 0.0,
        border_color: Some("#000".to_owned()),
        ..ComputedStyle::default()
    };
    let paint = map::to_paint(&style);
    assert!(paint.border.is_none());
}

#[test]
fn to_paint_border_radius_only_when_nonzero() {
    let style = ComputedStyle {
        border_radius: 8.0,
        ..ComputedStyle::default()
    };
    let paint = map::to_paint(&style);
    assert_eq!(paint.border_radius, Some(8.0));

    let style2 = ComputedStyle { border_radius: 0.0, ..ComputedStyle::default() };
    let paint2 = map::to_paint(&style2);
    assert!(paint2.border_radius.is_none());
}

// --- to_typography ---

#[test]
fn to_typography_none_when_required_field_missing() {
    // Missing font_size → None
    let style = ComputedStyle {
        font_family: Some("Inter".to_owned()),
        font_size: None,
        font_weight: Some(400),
        color: Some("#000".to_owned()),
        ..ComputedStyle::default()
    };
    assert!(map::to_typography(&style).is_none());
}

#[test]
fn to_typography_line_height_normal_by_default() {
    let style = ComputedStyle {
        font_family: Some("Inter".to_owned()),
        font_size: Some(16.0),
        font_weight: Some(400),
        color: Some("#000".to_owned()),
        line_height: None,
        ..ComputedStyle::default()
    };
    let typo = map::to_typography(&style);
    assert!(typo.is_some(), "typography should resolve");
    assert_eq!(typo.map(|t| t.line_height), Some(TLineHeight::Normal));
}

#[test]
fn to_typography_line_height_px_parsed() {
    let style = ComputedStyle {
        font_family: Some("Inter".to_owned()),
        font_size: Some(16.0),
        font_weight: Some(400),
        color: Some("#000".to_owned()),
        line_height: Some("24px".to_owned()),
        ..ComputedStyle::default()
    };
    let typo = map::to_typography(&style);
    assert!(typo.is_some(), "typography should resolve");
    assert_eq!(typo.map(|t| t.line_height), Some(TLineHeight::Length { value: 24.0 }));
}

#[test]
fn to_typography_font_family_split_on_comma() {
    let style = ComputedStyle {
        font_family: Some("Inter, sans-serif".to_owned()),
        font_size: Some(16.0),
        font_weight: Some(400),
        color: Some("#000".to_owned()),
        ..ComputedStyle::default()
    };
    let typo = map::to_typography(&style);
    assert!(typo.is_some(), "typography should resolve");
    assert_eq!(typo.map(|t| t.font_family), Some(vec!["Inter".to_owned(), "sans-serif".to_owned()]));
}

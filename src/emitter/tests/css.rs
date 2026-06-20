//! Unit tests for CSS emission (css.rs helpers).

use crate::emitter::css::{build_css, px};
use crate::ir::layout::{
    AlignItems, BoxSizing, Display, EdgeInset, FlexDirection, Layout, Position,
};
use crate::ir::paint::{Border, Color, Paint};
use crate::ir::typography::{LineHeight, Typography};
use crate::ir::{ControlKind, IrNode, NodeKind, SourceRef};

fn default_layout() -> Layout {
    Layout {
        position: Position::Static,
        display: Display::Block,
        box_sizing: BoxSizing::ContentBox,
        top: None,
        left: None,
        width: None,
        height: None,
        min_width: None,
        margin: EdgeInset::zero(),
        padding: EdgeInset::zero(),
        flex_direction: None,
        align_items: None,
        justify_content: None,
        align_self: None,
        gap: None,
    }
}

fn default_paint() -> Paint {
    Paint {
        background_color: None,
        border: None,
        border_radius: None,
        cursor: None,
    }
}

fn source_ref() -> SourceRef {
    SourceRef {
        doc_id: 0,
        dom_path: "0/1".to_owned(),
        span: None,
    }
}

fn control_node(id: &str) -> IrNode {
    IrNode {
        id: id.to_owned(),
        kind: NodeKind::Control,
        parent_id: None,
        control_kind: Some(ControlKind::Button),
        text_content: None,
        layout: default_layout(),
        paint: default_paint(),
        typography: None,
        source: source_ref(),
    }
}

fn box_node(id: &str, layout: Layout, paint: Paint) -> IrNode {
    IrNode {
        id: id.to_owned(),
        kind: NodeKind::Box,
        parent_id: None,
        control_kind: None,
        text_content: None,
        layout,
        paint,
        typography: None,
        source: source_ref(),
    }
}

// --- appearance reset ---

#[test]
fn button_control_emits_appearance_none() {
    let css = build_css(&[control_node("n0")]);
    assert!(css.contains("appearance: none"), "css={css}");
}

#[test]
fn button_appearance_none_precedes_other_decls() {
    let mut node = control_node("n0");
    node.paint.background_color = Some(Color("#2563eb".to_owned()));
    let css = build_css(&[node]);
    let app = css.find("appearance: none");
    let bg = css.find("background-color");
    assert!(
        app.is_some() && app < bg,
        "appearance: none must precede background-color: {css}"
    );
}

#[test]
fn box_node_does_not_emit_appearance_none() {
    let layout = Layout {
        width: Some(100.0),
        ..default_layout()
    };
    let css = build_css(&[box_node("n0", layout, default_paint())]);
    assert!(!css.contains("appearance"), "css={css}");
}

// --- px() helper ---

#[test]
fn px_integer_value_has_no_decimal() {
    assert_eq!(px(24.0), "24px");
    assert_eq!(px(0.0), "0px");
}

#[test]
fn px_fractional_value_preserves_decimal() {
    assert_eq!(px(1.5), "1.5px");
}

// --- CSS rule emission ---

#[test]
fn absolute_position_emits_position_absolute() {
    let layout = Layout {
        position: Position::Absolute,
        left: Some(10.0),
        top: Some(20.0),
        ..default_layout()
    };
    let css = build_css(&[box_node("n0", layout, default_paint())]);
    assert!(css.contains("position: absolute"), "css={css}");
    assert!(css.contains("left: 10px"), "css={css}");
    assert!(css.contains("top: 20px"), "css={css}");
}

#[test]
fn static_position_not_emitted() {
    let css = build_css(&[box_node("n0", default_layout(), default_paint())]);
    assert!(!css.contains("position"), "css={css}");
}

#[test]
fn flex_display_emitted() {
    let layout = Layout {
        display: Display::Flex,
        ..default_layout()
    };
    let css = build_css(&[box_node("n0", layout, default_paint())]);
    assert!(css.contains("display: flex"), "css={css}");
}

#[test]
fn border_box_emitted() {
    let layout = Layout {
        box_sizing: BoxSizing::BorderBox,
        ..default_layout()
    };
    let css = build_css(&[box_node("n0", layout, default_paint())]);
    assert!(css.contains("box-sizing: border-box"), "css={css}");
}

#[test]
fn zero_margin_not_emitted() {
    let css = build_css(&[box_node("n0", default_layout(), default_paint())]);
    assert!(
        !css.contains("margin"),
        "zero margin must not appear: css={css}"
    );
}

#[test]
fn uniform_padding_uses_shorthand() {
    let layout = Layout {
        padding: EdgeInset::uniform(16.0),
        ..default_layout()
    };
    let css = build_css(&[box_node("n0", layout, default_paint())]);
    assert!(css.contains("padding: 16px"), "css={css}");
    // Uniform shorthand must not repeat the value four times
    assert!(!css.contains("16px 16px"), "css={css}");
}

#[test]
fn asymmetric_margin_uses_four_value_shorthand() {
    let layout = Layout {
        margin: EdgeInset {
            top: 4.0,
            right: 8.0,
            bottom: 12.0,
            left: 16.0,
        },
        ..default_layout()
    };
    let css = build_css(&[box_node("n0", layout, default_paint())]);
    assert!(css.contains("margin: 4px 8px 12px 16px"), "css={css}");
}

#[test]
fn border_emits_shorthand() {
    let paint = Paint {
        border: Some(Border {
            width: 1.0,
            color: Color("#ff0000".to_owned()),
        }),
        ..default_paint()
    };
    let css = build_css(&[box_node("n0", default_layout(), paint)]);
    assert!(css.contains("border: 1px solid #ff0000"), "css={css}");
}

#[test]
fn background_color_emitted() {
    let paint = Paint {
        background_color: Some(Color("#fff".to_owned())),
        ..default_paint()
    };
    let css = build_css(&[box_node("n0", default_layout(), paint)]);
    assert!(css.contains("background-color: #fff"), "css={css}");
}

#[test]
fn flex_direction_column_emitted() {
    let layout = Layout {
        display: Display::Flex,
        flex_direction: Some(FlexDirection::Column),
        align_items: Some(AlignItems::Center),
        ..default_layout()
    };
    let css = build_css(&[box_node("n0", layout, default_paint())]);
    assert!(css.contains("flex-direction: column"), "css={css}");
    assert!(css.contains("align-items: center"), "css={css}");
}

#[test]
fn typography_decls_emitted_when_present() {
    let mut node = box_node("n0", default_layout(), default_paint());
    node.typography = Some(Typography {
        font_family: vec!["Inter".to_owned(), "sans-serif".to_owned()],
        font_size: 16.0,
        font_weight: 400,
        line_height: LineHeight::Normal,
        color: Color("#333".to_owned()),
    });
    let css = build_css(&[node]);
    assert!(css.contains("font-family: Inter, sans-serif"), "css={css}");
    assert!(css.contains("font-size: 16px"), "css={css}");
    assert!(css.contains("font-weight: 400"), "css={css}");
    assert!(css.contains("line-height: normal"), "css={css}");
    assert!(css.contains("color: #333"), "css={css}");
}

#[test]
fn text_node_produces_no_css_rule() {
    let text_node = IrNode {
        id: "n0".to_owned(),
        kind: NodeKind::Text,
        parent_id: None,
        control_kind: None,
        text_content: Some("Hello".to_owned()),
        layout: default_layout(),
        paint: default_paint(),
        typography: None,
        source: source_ref(),
    };
    let css = build_css(&[text_node]);
    assert!(css.is_empty(), "text node must not produce CSS: css={css}");
}

#[test]
fn empty_rule_is_omitted() {
    // A node with all defaults (static, block, content-box, no paint, no typo)
    // has nothing to emit → the CSS output should be empty.
    let css = build_css(&[box_node("n0", default_layout(), default_paint())]);
    assert!(
        css.is_empty(),
        "all-default node must produce no CSS: css={css}"
    );
}

#[test]
fn rule_uses_node_id_as_selector() {
    let layout = Layout {
        width: Some(100.0),
        ..default_layout()
    };
    let css = build_css(&[box_node("n5", layout, default_paint())]);
    assert!(css.contains(".n5 {"), "css={css}");
}

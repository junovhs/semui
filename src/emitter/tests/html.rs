//! Unit tests for HTML emission (html.rs).

use crate::emitter::html::build_html;
use crate::ir::layout::{BoxSizing, Display, EdgeInset, Layout, Position};
use crate::ir::paint::Paint;
use crate::ir::{ControlKind, ExecutionMode, IrNode, NodeKind, SceneIr, SourceRef};

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

fn minimal_scene(nodes: Vec<IrNode>) -> SceneIr {
    SceneIr {
        schema_version: 1,
        scene_id: "test".to_owned(),
        corpus: "v0.1".to_owned(),
        execution_mode: ExecutionMode::Static,
        nodes,
    }
}

// ---------------------------------------------------------------------------
// Document structure
// ---------------------------------------------------------------------------

#[test]
fn output_starts_with_doctype() {
    let scene = minimal_scene(vec![]);
    let html = build_html(&scene);
    assert!(html.starts_with("<!doctype html>"), "html={html}");
}

#[test]
fn output_contains_stylesheet_link() {
    let scene = minimal_scene(vec![]);
    let html = build_html(&scene);
    assert!(html.contains("styles.css"), "html={html}");
}

// ---------------------------------------------------------------------------
// Element tag selection
// ---------------------------------------------------------------------------

#[test]
fn box_node_emits_div() {
    let node = IrNode {
        id: "n0".to_owned(),
        kind: NodeKind::Box,
        parent_id: None,
        control_kind: None,
        text_content: None,
        layout: default_layout(),
        paint: default_paint(),
        typography: None,
        source: source_ref(),
    };
    let html = build_html(&minimal_scene(vec![node]));
    assert!(html.contains("<div class=\"n0\""), "html={html}");
}

#[test]
fn button_control_emits_button_tag() {
    let node = IrNode {
        id: "n0".to_owned(),
        kind: NodeKind::Control,
        parent_id: None,
        control_kind: Some(ControlKind::Button),
        text_content: None,
        layout: default_layout(),
        paint: default_paint(),
        typography: None,
        source: source_ref(),
    };
    let html = build_html(&minimal_scene(vec![node]));
    assert!(html.contains("<button class=\"n0\""), "html={html}");
    assert!(html.contains("</button>"), "html={html}");
}

// ---------------------------------------------------------------------------
// Text nodes
// ---------------------------------------------------------------------------

#[test]
fn text_node_content_appears_inside_parent() {
    let parent = IrNode {
        id: "n0".to_owned(),
        kind: NodeKind::Box,
        parent_id: None,
        control_kind: None,
        text_content: None,
        layout: default_layout(),
        paint: default_paint(),
        typography: None,
        source: source_ref(),
    };
    let text = IrNode {
        id: "n1".to_owned(),
        kind: NodeKind::Text,
        parent_id: Some("n0".to_owned()),
        control_kind: None,
        text_content: Some("Hello World".to_owned()),
        layout: default_layout(),
        paint: default_paint(),
        typography: None,
        source: source_ref(),
    };
    let html = build_html(&minimal_scene(vec![parent, text]));
    assert!(html.contains("Hello World"), "html={html}");
    // Text must be inside the parent div
    let div_pos = html.find("<div class=\"n0\">");
    let text_pos = html.find("Hello World");
    assert!(div_pos.is_some(), "div not found in html={html}");
    assert!(text_pos.is_some(), "text not found in html={html}");
    assert!(text_pos > div_pos, "text must come after opening div tag");
}

#[test]
fn special_chars_in_text_are_escaped() {
    let parent = IrNode {
        id: "n0".to_owned(),
        kind: NodeKind::Box,
        parent_id: None,
        control_kind: None,
        text_content: None,
        layout: default_layout(),
        paint: default_paint(),
        typography: None,
        source: source_ref(),
    };
    let text = IrNode {
        id: "n1".to_owned(),
        kind: NodeKind::Text,
        parent_id: Some("n0".to_owned()),
        control_kind: None,
        text_content: Some("a < b & c > d".to_owned()),
        layout: default_layout(),
        paint: default_paint(),
        typography: None,
        source: source_ref(),
    };
    let html = build_html(&minimal_scene(vec![parent, text]));
    assert!(html.contains("a &lt; b &amp; c &gt; d"), "html={html}");
}

// ---------------------------------------------------------------------------
// Nesting
// ---------------------------------------------------------------------------

#[test]
fn child_element_nested_inside_parent() {
    let parent = IrNode {
        id: "n0".to_owned(),
        kind: NodeKind::Box,
        parent_id: None,
        control_kind: None,
        text_content: None,
        layout: default_layout(),
        paint: default_paint(),
        typography: None,
        source: source_ref(),
    };
    let child = IrNode {
        id: "n1".to_owned(),
        kind: NodeKind::Box,
        parent_id: Some("n0".to_owned()),
        control_kind: None,
        text_content: None,
        layout: default_layout(),
        paint: default_paint(),
        typography: None,
        source: source_ref(),
    };
    let html = build_html(&minimal_scene(vec![parent, child]));
    let outer = html.find("<div class=\"n0\">");
    let inner = html.find("<div class=\"n1\">");
    assert!(outer.is_some(), "outer div not found in html={html}");
    assert!(inner.is_some(), "inner div not found in html={html}");
    assert!(inner > outer, "child must appear after parent opening tag");
}

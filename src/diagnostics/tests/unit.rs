//! Unit tests for the diagnostics analysis logic.

use crate::diagnostics::{analyze, DiagnosticKind};
use crate::source_graph::{
    CssDeclaration, CssRule, SceneSourceGraph, SourceDocument, SourceDocumentKind, TraceSpan,
};
use std::path::PathBuf;

fn zero_span() -> TraceSpan {
    TraceSpan { start: 0, end: 0, line: 1, column: 1 }
}

fn decl(property: &str, value: &str) -> CssDeclaration {
    CssDeclaration {
        property: property.to_owned(),
        value: value.to_owned(),
        span: zero_span(),
    }
}

fn rule(id: usize, selectors: &[&str], declarations: &[(&str, &str)]) -> CssRule {
    CssRule {
        id,
        selectors: selectors.iter().map(|s| s.to_string()).collect(),
        declarations: declarations.iter().map(|(p, v)| decl(p, v)).collect(),
        span: zero_span(),
        document_id: 1,
    }
}

fn fake_graph(rules: Vec<CssRule>) -> SceneSourceGraph {
    SceneSourceGraph {
        scene_id: "test".to_owned(),
        scene_root: PathBuf::from("."),
        html: SourceDocument {
            id: 0,
            kind: SourceDocumentKind::Html,
            path: PathBuf::from("test.html"),
            contents: String::new(),
        },
        css: SourceDocument {
            id: 1,
            kind: SourceDocumentKind::Css,
            path: PathBuf::from("test.css"),
            contents: String::new(),
        },
        html_nodes: Vec::new(),
        css_rules: rules,
    }
}

// ---------------------------------------------------------------------------
// Unsupported properties
// ---------------------------------------------------------------------------

#[test]
fn unknown_property_produces_unsupported_property_diagnostic() {
    let graph = fake_graph(vec![rule(0, &[".foo"], &[("appearance", "none")])]);
    let diags = analyze(&graph);
    assert_eq!(diags.len(), 1);
    assert_eq!(diags[0].kind, DiagnosticKind::UnsupportedProperty);
    assert_eq!(diags[0].property.as_deref(), Some("appearance"));
}

#[test]
fn known_property_produces_no_unsupported_property_diagnostic() {
    let graph = fake_graph(vec![rule(0, &[".foo"], &[("display", "flex")])]);
    let diags = analyze(&graph);
    assert!(diags.is_empty(), "diags={diags:?}");
}

#[test]
fn multiple_unknown_properties_each_produce_a_diagnostic() {
    let graph = fake_graph(vec![rule(
        0,
        &[".foo"],
        &[("transform", "rotate(45deg)"), ("opacity", "0.5")],
    )]);
    let diags = analyze(&graph);
    assert_eq!(diags.len(), 2);
    let props: Vec<_> = diags.iter().filter_map(|d| d.property.as_deref()).collect();
    assert!(props.contains(&"transform"), "props={props:?}");
    assert!(props.contains(&"opacity"), "props={props:?}");
}

// ---------------------------------------------------------------------------
// Unsupported values
// ---------------------------------------------------------------------------

#[test]
fn display_grid_produces_unsupported_value_diagnostic() {
    let graph = fake_graph(vec![rule(0, &[".foo"], &[("display", "grid")])]);
    let diags = analyze(&graph);
    assert_eq!(diags.len(), 1);
    assert_eq!(diags[0].kind, DiagnosticKind::UnsupportedValue);
    assert_eq!(diags[0].value.as_deref(), Some("grid"));
}

#[test]
fn position_relative_produces_unsupported_value_diagnostic() {
    let graph = fake_graph(vec![rule(0, &[".foo"], &[("position", "relative")])]);
    let diags = analyze(&graph);
    assert_eq!(diags.len(), 1);
    assert_eq!(diags[0].kind, DiagnosticKind::UnsupportedValue);
}

#[test]
fn display_flex_is_supported_no_diagnostic() {
    let graph = fake_graph(vec![rule(0, &[".foo"], &[("display", "flex")])]);
    let diags = analyze(&graph);
    assert!(diags.is_empty(), "diags={diags:?}");
}

#[test]
fn position_absolute_is_supported_no_diagnostic() {
    let graph = fake_graph(vec![rule(0, &[".foo"], &[("position", "absolute")])]);
    let diags = analyze(&graph);
    assert!(diags.is_empty(), "diags={diags:?}");
}

// ---------------------------------------------------------------------------
// Unsupported selectors
// ---------------------------------------------------------------------------

#[test]
fn id_selector_produces_unsupported_selector_diagnostic() {
    let graph = fake_graph(vec![rule(0, &["#main"], &[("display", "flex")])]);
    let diags = analyze(&graph);
    assert!(
        diags.iter().any(|d| d.kind == DiagnosticKind::UnsupportedSelector),
        "diags={diags:?}"
    );
}

#[test]
fn pseudo_class_produces_unsupported_selector_diagnostic() {
    let graph = fake_graph(vec![rule(0, &["a:hover"], &[("color", "#000")])]);
    let diags = analyze(&graph);
    assert!(
        diags.iter().any(|d| d.kind == DiagnosticKind::UnsupportedSelector),
        "diags={diags:?}"
    );
}

#[test]
fn descendant_combinator_produces_unsupported_selector_diagnostic() {
    let graph = fake_graph(vec![rule(0, &["div span"], &[("color", "#000")])]);
    let diags = analyze(&graph);
    assert!(
        diags.iter().any(|d| d.kind == DiagnosticKind::UnsupportedSelector),
        "diags={diags:?}"
    );
}

#[test]
fn attribute_selector_produces_unsupported_selector_diagnostic() {
    let graph = fake_graph(vec![rule(0, &["input[type=\"text\"]"], &[("color", "#000")])]);
    let diags = analyze(&graph);
    assert!(
        diags.iter().any(|d| d.kind == DiagnosticKind::UnsupportedSelector),
        "diags={diags:?}"
    );
}

#[test]
fn simple_class_selector_is_supported_no_diagnostic() {
    let graph = fake_graph(vec![rule(0, &[".btn"], &[("display", "flex")])]);
    let diags = analyze(&graph);
    assert!(diags.is_empty(), "diags={diags:?}");
}

#[test]
fn compound_class_type_selector_is_supported_no_diagnostic() {
    let graph = fake_graph(vec![rule(0, &["button.primary"], &[("display", "flex")])]);
    let diags = analyze(&graph);
    assert!(diags.is_empty(), "diags={diags:?}");
}

// ---------------------------------------------------------------------------
// Empty scene
// ---------------------------------------------------------------------------

#[test]
fn empty_rules_produces_no_diagnostics() {
    let graph = fake_graph(vec![]);
    assert!(analyze(&graph).is_empty());
}

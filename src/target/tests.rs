//! Contract tests for the runtime-neutral target boundary (`RET-01`).
//!
//! These cover the six properties the proof-of-done requires: traversal,
//! resources, controls, unsupported capabilities, deterministic output, and
//! declared loss. The [`MockTarget`] proves a non-HTML adapter can satisfy the
//! interface from `&SceneIr` alone, and [`HtmlTarget`] proves the reference
//! emitter is contract-equivalent.

use std::collections::BTreeSet;

use super::*;
use crate::emitter::{HtmlTarget, emit};
use crate::ir::{
    Border, Color, ControlKind, Cursor, Display, EdgeInset, ExecutionMode, FlexDirection, IrNode,
    Layout, LineHeight, NodeKind, Paint, Position, SceneIr, SourceRef, Typography,
};

// ---------------------------------------------------------------------------
// Fixtures
// ---------------------------------------------------------------------------

fn base_layout() -> Layout {
    Layout {
        position: Position::Static,
        display: Display::Block,
        box_sizing: crate::ir::BoxSizing::BorderBox,
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

fn source(path: &str) -> SourceRef {
    SourceRef {
        doc_id: 0,
        dom_path: path.to_string(),
        span: None,
    }
}

/// A scene that exercises every v0.1 capability: a flex container with a border
/// and radius, a text child, and an absolutely positioned button child.
fn sample_scene() -> SceneIr {
    let root = IrNode {
        id: "n0".to_string(),
        kind: NodeKind::Box,
        parent_id: None,
        control_kind: None,
        text_content: None,
        layout: Layout {
            display: Display::Flex,
            flex_direction: Some(FlexDirection::Column),
            ..base_layout()
        },
        paint: Paint {
            background_color: Some(Color("#ffffff".to_string())),
            border: Some(Border {
                width: 1.0,
                color: Color("#cccccc".to_string()),
            }),
            border_radius: Some(8.0),
            cursor: None,
        },
        typography: None,
        source: source("body > div"),
    };
    let text = IrNode {
        id: "n1".to_string(),
        kind: NodeKind::Text,
        parent_id: Some("n0".to_string()),
        control_kind: None,
        text_content: Some("Hello".to_string()),
        layout: base_layout(),
        paint: Paint {
            background_color: None,
            border: None,
            border_radius: None,
            cursor: None,
        },
        typography: Some(Typography {
            font_family: vec!["Inter".to_string(), "sans-serif".to_string()],
            font_size: 14.0,
            font_weight: 400,
            line_height: LineHeight::Length { value: 20.0 },
            color: Color("#111111".to_string()),
        }),
        source: source("body > div > #text"),
    };
    let button = IrNode {
        id: "n2".to_string(),
        kind: NodeKind::Control,
        parent_id: Some("n0".to_string()),
        control_kind: Some(ControlKind::Button),
        text_content: None,
        layout: Layout {
            position: Position::Absolute,
            top: Some(4.0),
            left: Some(8.0),
            ..base_layout()
        },
        paint: Paint {
            background_color: Some(Color("#2563eb".to_string())),
            border: None,
            border_radius: None,
            cursor: Some(Cursor::Pointer),
        },
        typography: Some(Typography {
            font_family: vec!["Inter".to_string(), "sans-serif".to_string()],
            font_size: 13.0,
            font_weight: 600,
            line_height: LineHeight::Length { value: 18.0 },
            color: Color("#ffffff".to_string()),
        }),
        source: source("body > div > button"),
    };
    SceneIr {
        schema_version: 1,
        scene_id: "sample".to_string(),
        corpus: "v0.1".to_string(),
        execution_mode: ExecutionMode::Static,
        // Deliberately not in pre-order to prove `preorder` reorders by tree.
        nodes: vec![root, button, text],
    }
}

// ---------------------------------------------------------------------------
// A mock non-HTML target
// ---------------------------------------------------------------------------

/// A minimal target that supports only block layout, background, and typography.
/// It produces a deterministic textual draw list purely from the IR, proving an
/// adapter needs no access to source HTML/CSS.
struct MockTarget;

impl TargetEmitter for MockTarget {
    type Artifact = String;

    fn target_id(&self) -> &'static str {
        "mock"
    }

    fn capabilities(&self) -> TargetCapabilities {
        TargetCapabilities::from_capabilities([
            Capability::BlockLayout,
            Capability::Background,
            Capability::Typography,
        ])
    }

    fn emit(&self, scene: &SceneIr) -> TargetEmission<String> {
        let lines: Vec<String> = preorder(scene)
            .into_iter()
            .map(|node| match &node.text_content {
                Some(text) => format!("{}:text:{text}", node.id),
                None => format!("{}:{:?}", node.id, node.kind),
            })
            .collect();
        TargetEmission {
            artifact: lines.join("\n"),
            declared_loss: capability_gaps(scene, &self.capabilities()),
        }
    }
}

// ---------------------------------------------------------------------------
// Traversal
// ---------------------------------------------------------------------------

#[test]
fn preorder_visits_parents_before_children_in_source_order() {
    let scene = sample_scene();
    let ids: Vec<&str> = preorder(&scene).iter().map(|n| n.id.as_str()).collect();
    // n0 root, then its children in the order they appear in `nodes` (button n2
    // is stored before text n1), regardless of the unsorted input vector.
    assert_eq!(ids, vec!["n0", "n2", "n1"]);
}

// ---------------------------------------------------------------------------
// Resources
// ---------------------------------------------------------------------------

#[test]
fn collect_resources_merges_font_weights_and_sorts_colors() {
    let resources = collect_resources(&sample_scene());
    assert_eq!(
        resources.fonts,
        vec![FontRequest {
            family_stack: vec!["Inter".to_string(), "sans-serif".to_string()],
            weights: vec![400, 600],
        }]
    );
    assert_eq!(
        resources.colors,
        vec!["#111111", "#2563eb", "#cccccc", "#ffffff"]
    );
}

// ---------------------------------------------------------------------------
// Controls and required capabilities
// ---------------------------------------------------------------------------

#[test]
fn scene_capabilities_reports_every_required_family() {
    let caps: Vec<Capability> = scene_capabilities(&sample_scene()).into_iter().collect();
    assert_eq!(caps, Capability::all().to_vec());
}

// ---------------------------------------------------------------------------
// Unsupported capabilities and declared loss
// ---------------------------------------------------------------------------

#[test]
fn capability_gaps_report_unsupported_constructs_per_node() {
    let scene = sample_scene();
    let gaps = capability_gaps(&scene, &MockTarget.capabilities());
    let observed: Vec<(&str, Capability)> = gaps
        .iter()
        .map(|gap| (gap.node_id.as_str(), gap.capability))
        .collect();
    assert_eq!(
        observed,
        vec![
            ("n0", Capability::FlexLayout),
            ("n0", Capability::Border),
            ("n0", Capability::BorderRadius),
            ("n2", Capability::AbsolutePositioning),
            ("n2", Capability::ButtonControl),
        ]
    );
}

#[test]
fn mock_emission_declares_the_same_loss_it_cannot_render() {
    let scene = sample_scene();
    let emission = MockTarget.emit(&scene);
    assert_eq!(
        emission.declared_loss,
        capability_gaps(&scene, &MockTarget.capabilities())
    );
    assert!(!emission.declared_loss.is_empty());
}

#[test]
fn a_target_that_supports_nothing_declares_every_required_capability_as_loss() {
    let scene = sample_scene();
    let gaps = capability_gaps(&scene, &TargetCapabilities::none());
    // Every capability any node requires becomes loss; nothing is silently kept.
    let lost: BTreeSet<Capability> = gaps.iter().map(|gap| gap.capability).collect();
    assert_eq!(lost, scene_capabilities(&scene));
}

// ---------------------------------------------------------------------------
// Deterministic output
// ---------------------------------------------------------------------------

#[test]
fn mock_target_emits_deterministically() {
    let scene = sample_scene();
    assert_eq!(MockTarget.emit(&scene), MockTarget.emit(&scene));
    assert_eq!(
        MockTarget.emit(&scene).artifact,
        "n0:Box\nn2:Control\nn1:text:Hello"
    );
}

// ---------------------------------------------------------------------------
// HTML target is contract-equivalent
// ---------------------------------------------------------------------------

#[test]
fn html_target_artifact_equals_strict_emit_with_no_loss() {
    let scene = sample_scene();
    let emission = HtmlTarget.emit(&scene);
    assert_eq!(emission.artifact, emit(&scene));
    assert!(
        emission.declared_loss.is_empty(),
        "the HTML reference target supports the full v0.1 subset"
    );
    assert_eq!(HtmlTarget.target_id(), "html");
}

// ---------------------------------------------------------------------------
// Conformance fixture format
// ---------------------------------------------------------------------------

#[test]
fn expected_conformance_strips_provenance_and_round_trips_through_json() {
    let scene = sample_scene();
    let conformance = expected_conformance(&scene);

    assert_eq!(conformance.scene_id, "sample");
    assert_eq!(conformance.conventions, Conventions::v0_1());
    assert_eq!(
        conformance.required_capabilities,
        Capability::all().to_vec()
    );
    // Pre-order, provenance dropped.
    let ids: Vec<&str> = conformance.nodes.iter().map(|n| n.id.as_str()).collect();
    assert_eq!(ids, vec!["n0", "n2", "n1"]);

    let json = serde_json::to_string_pretty(&conformance).expect("serializable");
    assert!(
        !json.contains("dom_path"),
        "conformance format must not carry source provenance"
    );
    let restored: ConformanceScene = serde_json::from_str(&json).expect("deserializable");
    assert_eq!(restored, conformance);
}

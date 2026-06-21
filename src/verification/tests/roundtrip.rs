//! Round-trip regression tests for the v0.1 fixture corpus.

use std::path::PathBuf;

use serde_json::{Value, json};

use super::super::compare_ir;
use crate::ir::SceneIr;
use crate::load_scene_source_graph;
use crate::verification::VerificationResult;
use crate::verification::verify_round_trip;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn profile_card_ir() -> Result<SceneIr, Box<dyn std::error::Error>> {
    let json = std::fs::read_to_string(
        repo_root().join("fixtures/v0.1/profile_card_absolute/expected/scene.semui.json"),
    )?;
    Ok(SceneIr::from_json(&json)?)
}

fn assert_field_drift(
    baseline: &SceneIr,
    object_pointer: &str,
    field: &str,
    replacement: Value,
    expected_path: &str,
) {
    let mut changed = serde_json::to_value(baseline).expect("fixture IR must serialize");
    changed
        .pointer_mut(object_pointer)
        .and_then(Value::as_object_mut)
        .unwrap_or_else(|| panic!("missing object at {object_pointer}"))
        .insert(field.to_owned(), replacement);
    let changed: SceneIr = serde_json::from_value(changed).expect("mutation must remain valid IR");
    let comparison = compare_ir(baseline, &changed);
    let drift: Vec<_> = comparison
        .structural_drift
        .iter()
        .chain(&comparison.semantic_ir_drift)
        .collect();

    assert_eq!(
        drift.len(),
        1,
        "unexpected drift for {expected_path}: {drift:?}"
    );
    assert!(
        drift[0].message.starts_with(expected_path),
        "expected {expected_path}, got {}",
        drift[0].message
    );
}

fn assert_internal_gates_pass(scene_id: &str, result: &VerificationResult) {
    let drift = result
        .structural
        .drift
        .iter()
        .chain(&result.semantic_ir.drift)
        .map(|d| format!("  - {}", d.message))
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        result.structural.pass && result.semantic_ir.pass,
        "{scene_id} internal gates failed:\n{drift}"
    );
}

// ---------------------------------------------------------------------------
// Per-fixture round-trip assertions
// ---------------------------------------------------------------------------

#[test]
fn profile_card_absolute_round_trip_passes() -> Result<(), Box<dyn std::error::Error>> {
    let graph = load_scene_source_graph(repo_root(), "profile_card_absolute")?;
    let result = verify_round_trip(&graph)?;

    assert_internal_gates_pass("profile_card_absolute", &result);
    Ok(())
}

#[test]
fn stacked_info_card_round_trip_passes() -> Result<(), Box<dyn std::error::Error>> {
    let graph = load_scene_source_graph(repo_root(), "stacked_info_card")?;
    let result = verify_round_trip(&graph)?;

    assert_internal_gates_pass("stacked_info_card", &result);
    Ok(())
}

#[test]
fn action_row_variants_round_trip_passes() -> Result<(), Box<dyn std::error::Error>> {
    let graph = load_scene_source_graph(repo_root(), "action_row_variants")?;
    let result = verify_round_trip(&graph)?;

    assert_internal_gates_pass("action_row_variants", &result);
    Ok(())
}

#[test]
fn nested_panel_inset_round_trip_passes() -> Result<(), Box<dyn std::error::Error>> {
    let graph = load_scene_source_graph(repo_root(), "nested_panel_inset")?;
    let result = verify_round_trip(&graph)?;

    assert_internal_gates_pass("nested_panel_inset", &result);
    Ok(())
}

#[test]
fn typography_specimen_round_trip_passes() -> Result<(), Box<dyn std::error::Error>> {
    let graph = load_scene_source_graph(repo_root(), "typography_specimen")?;
    let result = verify_round_trip(&graph)?;

    assert_internal_gates_pass("typography_specimen", &result);
    Ok(())
}

// ---------------------------------------------------------------------------
// Structural invariants
// ---------------------------------------------------------------------------

#[test]
fn round_trip_preserves_node_count() -> Result<(), Box<dyn std::error::Error>> {
    let graph = load_scene_source_graph(repo_root(), "profile_card_absolute")?;
    let result = verify_round_trip(&graph)?;

    assert_eq!(
        result.pass1_node_count, result.pass2_node_count,
        "node count must be preserved across round-trip"
    );
    Ok(())
}

#[test]
fn round_trip_result_carries_correct_scene_id() -> Result<(), Box<dyn std::error::Error>> {
    let graph = load_scene_source_graph(repo_root(), "profile_card_absolute")?;
    let result = verify_round_trip(&graph)?;
    assert_eq!(result.scene_id, "profile_card_absolute");
    Ok(())
}

// ---------------------------------------------------------------------------
// Complete semantic Scene IR comparison contract
// ---------------------------------------------------------------------------

#[test]
fn semantic_comparison_reports_every_field_with_an_exact_path()
-> Result<(), Box<dyn std::error::Error>> {
    let baseline = profile_card_ir()?;
    let cases = [
        ("", "schema_version", json!(2), "scene.schema_version"),
        ("", "scene_id", json!("changed"), "scene.scene_id"),
        ("", "corpus", json!("changed"), "scene.corpus"),
        ("/nodes/0", "id", json!("changed"), "node[0].id"),
        ("/nodes/0", "kind", json!("control"), "node[0].kind"),
        (
            "/nodes/0",
            "parent_id",
            json!("parent"),
            "node[0].parent_id",
        ),
        (
            "/nodes/0",
            "control_kind",
            json!("button"),
            "node[0].control_kind",
        ),
        (
            "/nodes/0",
            "text_content",
            json!("changed"),
            "node[0].text_content",
        ),
        (
            "/nodes/0/layout",
            "position",
            json!("static"),
            "node[0].layout.position",
        ),
        (
            "/nodes/0/layout",
            "display",
            json!("flex"),
            "node[0].layout.display",
        ),
        (
            "/nodes/0/layout",
            "box_sizing",
            json!("content_box"),
            "node[0].layout.box_sizing",
        ),
        ("/nodes/0/layout", "top", json!(21.0), "node[0].layout.top"),
        (
            "/nodes/0/layout",
            "left",
            json!(25.0),
            "node[0].layout.left",
        ),
        (
            "/nodes/0/layout",
            "width",
            json!(321.0),
            "node[0].layout.width",
        ),
        (
            "/nodes/0/layout",
            "height",
            json!(181.0),
            "node[0].layout.height",
        ),
        (
            "/nodes/0/layout",
            "min_width",
            json!(100.0),
            "node[0].layout.min_width",
        ),
        (
            "/nodes/0/layout/margin",
            "top",
            json!(1.0),
            "node[0].layout.margin.top",
        ),
        (
            "/nodes/0/layout/margin",
            "right",
            json!(1.0),
            "node[0].layout.margin.right",
        ),
        (
            "/nodes/0/layout/margin",
            "bottom",
            json!(1.0),
            "node[0].layout.margin.bottom",
        ),
        (
            "/nodes/0/layout/margin",
            "left",
            json!(1.0),
            "node[0].layout.margin.left",
        ),
        (
            "/nodes/0/layout/padding",
            "top",
            json!(1.0),
            "node[0].layout.padding.top",
        ),
        (
            "/nodes/0/layout/padding",
            "right",
            json!(1.0),
            "node[0].layout.padding.right",
        ),
        (
            "/nodes/0/layout/padding",
            "bottom",
            json!(1.0),
            "node[0].layout.padding.bottom",
        ),
        (
            "/nodes/0/layout/padding",
            "left",
            json!(1.0),
            "node[0].layout.padding.left",
        ),
        (
            "/nodes/0/layout",
            "flex_direction",
            json!("column"),
            "node[0].layout.flex_direction",
        ),
        (
            "/nodes/0/layout",
            "align_items",
            json!("center"),
            "node[0].layout.align_items",
        ),
        (
            "/nodes/0/layout",
            "justify_content",
            json!("center"),
            "node[0].layout.justify_content",
        ),
        (
            "/nodes/0/layout",
            "align_self",
            json!("flex_start"),
            "node[0].layout.align_self",
        ),
        ("/nodes/0/layout", "gap", json!(1.0), "node[0].layout.gap"),
        (
            "/nodes/0/paint",
            "background_color",
            json!("#000000"),
            "node[0].paint.background_color",
        ),
        (
            "/nodes/0/paint/border",
            "width",
            json!(2.0),
            "node[0].paint.border.width",
        ),
        (
            "/nodes/0/paint/border",
            "color",
            json!("#000000"),
            "node[0].paint.border.color",
        ),
        (
            "/nodes/0/paint",
            "border_radius",
            json!(17.0),
            "node[0].paint.border_radius",
        ),
        (
            "/nodes/0/paint",
            "cursor",
            json!("pointer"),
            "node[0].paint.cursor",
        ),
        (
            "/nodes/1/typography",
            "font_family",
            json!(["serif"]),
            "node[1].typography.font_family",
        ),
        (
            "/nodes/1/typography",
            "font_size",
            json!(21.0),
            "node[1].typography.font_size",
        ),
        (
            "/nodes/1/typography",
            "font_weight",
            json!(600),
            "node[1].typography.font_weight",
        ),
        (
            "/nodes/1/typography",
            "line_height",
            json!({"kind": "length", "value": 21.0}),
            "node[1].typography.line_height",
        ),
        (
            "/nodes/1/typography",
            "color",
            json!("#000000"),
            "node[1].typography.color",
        ),
    ];

    for (object, field, replacement, expected) in cases {
        assert_field_drift(&baseline, object, field, replacement, expected);
    }

    Ok(())
}

#[test]
fn semantic_comparison_excludes_source_provenance() -> Result<(), Box<dyn std::error::Error>> {
    let baseline = profile_card_ir()?;
    let mut changed = serde_json::to_value(&baseline)?;
    changed["nodes"][0]["source"]["dom_path"] = json!("emitted/parse/path");
    changed["nodes"][0]["source"]["doc_id"] = json!(99);
    let changed: SceneIr = serde_json::from_value(changed)?;

    let comparison = compare_ir(&baseline, &changed);
    assert!(comparison.structural_drift.is_empty());
    assert!(comparison.semantic_ir_drift.is_empty());
    Ok(())
}

#[test]
fn semantic_comparison_reports_node_count_drift() -> Result<(), Box<dyn std::error::Error>> {
    let baseline = profile_card_ir()?;
    let mut changed = baseline.clone();
    changed.nodes.pop();

    let comparison = compare_ir(&baseline, &changed);
    assert_eq!(comparison.structural_drift.len(), 1);
    assert!(comparison.semantic_ir_drift.is_empty());
    assert!(
        comparison.structural_drift[0]
            .message
            .starts_with("node count:")
    );
    Ok(())
}

#[test]
fn comparison_partitions_structural_and_semantic_drift() -> Result<(), Box<dyn std::error::Error>> {
    let baseline = profile_card_ir()?;

    let mut structural_json = serde_json::to_value(&baseline)?;
    structural_json["nodes"][1]["parent_id"] = json!("changed-parent");
    let structural: SceneIr = serde_json::from_value(structural_json)?;
    let comparison = compare_ir(&baseline, &structural);
    assert_eq!(comparison.structural_drift.len(), 1);
    assert!(comparison.semantic_ir_drift.is_empty());

    let mut semantic_json = serde_json::to_value(&baseline)?;
    semantic_json["nodes"][0]["layout"]["width"] = json!(999.0);
    let semantic: SceneIr = serde_json::from_value(semantic_json)?;
    let comparison = compare_ir(&baseline, &semantic);
    assert!(comparison.structural_drift.is_empty());
    assert_eq!(comparison.semantic_ir_drift.len(), 1);
    Ok(())
}

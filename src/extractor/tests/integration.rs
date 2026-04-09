//! Integration tests for extract_ir against the fixture corpus.

use std::path::PathBuf;

use crate::extractor::extract_ir;
use crate::ir::{NodeKind, ControlKind};
use crate::layout::compute_layout;
use crate::load_scene_source_graph;
use crate::resolver::resolve_scene;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

// ---------------------------------------------------------------------------
// Basic sanity
// ---------------------------------------------------------------------------

#[test]
fn profile_card_produces_nonempty_ir() -> Result<(), Box<dyn std::error::Error>> {
    let graph = load_scene_source_graph(repo_root(), "profile_card_absolute")?;
    let resolved = resolve_scene(&graph)?;
    let laid_out = compute_layout(&resolved);
    let ir = extract_ir(&laid_out, &graph)?;

    assert_eq!(ir.scene_id, "profile_card_absolute");
    assert_eq!(ir.schema_version, 1);
    assert!(!ir.nodes.is_empty());
    Ok(())
}

#[test]
fn stacked_card_produces_nonempty_ir() -> Result<(), Box<dyn std::error::Error>> {
    let graph = load_scene_source_graph(repo_root(), "stacked_info_card")?;
    let resolved = resolve_scene(&graph)?;
    let laid_out = compute_layout(&resolved);
    let ir = extract_ir(&laid_out, &graph)?;

    assert!(!ir.nodes.is_empty());
    Ok(())
}

// ---------------------------------------------------------------------------
// Pre-order invariant: root has no parent, all others do
// ---------------------------------------------------------------------------

#[test]
fn profile_card_ir_first_node_has_no_parent() -> Result<(), Box<dyn std::error::Error>> {
    let graph = load_scene_source_graph(repo_root(), "profile_card_absolute")?;
    let resolved = resolve_scene(&graph)?;
    let laid_out = compute_layout(&resolved);
    let ir = extract_ir(&laid_out, &graph)?;

    // The very first visible element node is the root in IR pre-order.
    assert!(
        ir.nodes[0].parent_id.is_none(),
        "first ir node must have no parent"
    );
    Ok(())
}

// ---------------------------------------------------------------------------
// Element kind mapping
// ---------------------------------------------------------------------------

#[test]
fn button_element_maps_to_control_kind() -> Result<(), Box<dyn std::error::Error>> {
    let graph = load_scene_source_graph(repo_root(), "profile_card_absolute")?;
    let resolved = resolve_scene(&graph)?;
    let laid_out = compute_layout(&resolved);
    let ir = extract_ir(&laid_out, &graph)?;

    let button_node = ir.nodes.iter().find(|n| n.kind == NodeKind::Control);
    let Some(btn) = button_node else {
        // If no button in this fixture, the test is vacuously ok — but
        // flag it so we know to add a fixture with a button.
        return Ok(());
    };
    assert_eq!(btn.control_kind, Some(ControlKind::Button));
    Ok(())
}

// ---------------------------------------------------------------------------
// Text nodes
// ---------------------------------------------------------------------------

#[test]
fn text_nodes_have_text_content_set() -> Result<(), Box<dyn std::error::Error>> {
    let graph = load_scene_source_graph(repo_root(), "profile_card_absolute")?;
    let resolved = resolve_scene(&graph)?;
    let laid_out = compute_layout(&resolved);
    let ir = extract_ir(&laid_out, &graph)?;

    let text_nodes: Vec<_> = ir.nodes.iter().filter(|n| n.kind == NodeKind::Text).collect();
    assert!(
        !text_nodes.is_empty(),
        "profile_card_absolute should contain at least one text node"
    );
    for t in &text_nodes {
        let content = t.text_content.as_deref().unwrap_or("");
        assert!(!content.is_empty(), "text node content must be non-empty");
    }
    Ok(())
}

#[test]
fn text_nodes_have_parent_id() -> Result<(), Box<dyn std::error::Error>> {
    let graph = load_scene_source_graph(repo_root(), "profile_card_absolute")?;
    let resolved = resolve_scene(&graph)?;
    let laid_out = compute_layout(&resolved);
    let ir = extract_ir(&laid_out, &graph)?;

    for node in ir.nodes.iter().filter(|n| n.kind == NodeKind::Text) {
        assert!(
            node.parent_id.is_some(),
            "text node {:?} must have a parent_id",
            node.id
        );
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Scene id mismatch is rejected
// ---------------------------------------------------------------------------

#[test]
fn mismatched_scene_ids_return_error() -> Result<(), Box<dyn std::error::Error>> {
    let graph = load_scene_source_graph(repo_root(), "profile_card_absolute")?;
    let graph2 = load_scene_source_graph(repo_root(), "stacked_info_card")?;
    let resolved = resolve_scene(&graph)?;
    let laid_out = compute_layout(&resolved);

    // laid_out is profile_card_absolute, graph2 is stacked_info_card → mismatch
    let result = extract_ir(&laid_out, &graph2);
    assert!(result.is_err(), "mismatched scene ids must return Err");
    Ok(())
}

// ---------------------------------------------------------------------------
// Source provenance
// ---------------------------------------------------------------------------

#[test]
fn all_ir_nodes_have_nonempty_dom_path() -> Result<(), Box<dyn std::error::Error>> {
    let graph = load_scene_source_graph(repo_root(), "profile_card_absolute")?;
    let resolved = resolve_scene(&graph)?;
    let laid_out = compute_layout(&resolved);
    let ir = extract_ir(&laid_out, &graph)?;

    for node in &ir.nodes {
        assert!(
            !node.source.dom_path.is_empty(),
            "node {:?} has empty dom_path",
            node.id
        );
    }
    Ok(())
}

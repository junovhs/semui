//! Round-trip regression tests for the v0.1 fixture corpus.

use std::path::PathBuf;

use crate::load_scene_source_graph;
use crate::verification::verify_round_trip;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

// ---------------------------------------------------------------------------
// Per-fixture round-trip assertions
// ---------------------------------------------------------------------------

#[test]
fn profile_card_absolute_round_trip_passes() -> Result<(), Box<dyn std::error::Error>> {
    let graph = load_scene_source_graph(repo_root(), "profile_card_absolute")?;
    let result = verify_round_trip(&graph)?;

    assert!(
        result.pass,
        "profile_card_absolute round-trip failed:\n{}",
        result
            .drift
            .iter()
            .map(|d| format!("  - {}", d.message))
            .collect::<Vec<_>>()
            .join("\n")
    );
    Ok(())
}

#[test]
fn stacked_info_card_round_trip_passes() -> Result<(), Box<dyn std::error::Error>> {
    let graph = load_scene_source_graph(repo_root(), "stacked_info_card")?;
    let result = verify_round_trip(&graph)?;

    assert!(
        result.pass,
        "stacked_info_card round-trip failed:\n{}",
        result
            .drift
            .iter()
            .map(|d| format!("  - {}", d.message))
            .collect::<Vec<_>>()
            .join("\n")
    );
    Ok(())
}

#[test]
fn action_row_variants_round_trip_passes() -> Result<(), Box<dyn std::error::Error>> {
    let graph = load_scene_source_graph(repo_root(), "action_row_variants")?;
    let result = verify_round_trip(&graph)?;

    assert!(
        result.pass,
        "action_row_variants round-trip failed:\n{}",
        result
            .drift
            .iter()
            .map(|d| format!("  - {}", d.message))
            .collect::<Vec<_>>()
            .join("\n")
    );
    Ok(())
}

#[test]
fn nested_panel_inset_round_trip_passes() -> Result<(), Box<dyn std::error::Error>> {
    let graph = load_scene_source_graph(repo_root(), "nested_panel_inset")?;
    let result = verify_round_trip(&graph)?;

    assert!(
        result.pass,
        "nested_panel_inset round-trip failed:\n{}",
        result
            .drift
            .iter()
            .map(|d| format!("  - {}", d.message))
            .collect::<Vec<_>>()
            .join("\n")
    );
    Ok(())
}

#[test]
fn typography_specimen_round_trip_passes() -> Result<(), Box<dyn std::error::Error>> {
    let graph = load_scene_source_graph(repo_root(), "typography_specimen")?;
    let result = verify_round_trip(&graph)?;

    assert!(
        result.pass,
        "typography_specimen round-trip failed:\n{}",
        result
            .drift
            .iter()
            .map(|d| format!("  - {}", d.message))
            .collect::<Vec<_>>()
            .join("\n")
    );
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
